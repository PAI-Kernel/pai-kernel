//! # PAI API Server (Component #1)
//!
//! Per PHASE1-TZ-001 Section 4 — HTTP governance API using **axum 0.7 + tokio**.
//!
//! ## Endpoints (Section 4.2)
//!
//! Mutation endpoints auto-append to the WitnessLog.
//! Context is validated against a growth-signal denylist on every request.
//! Every response carries a `x-request-id` UUID header.

#![forbid(unsafe_code)]

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use pai_drift::{DriftEngine, DriftThresholds};
use pai_governance_daemon::{keyloader, GovernanceDaemon};
use pai_policy::PolicyEngine;
use pai_storage::{GovernanceStore, SqliteStore};
use pai_witness::{
    ConstitutionalRef, DecisionClass, ImpactScope, Initiator, ReversibilityStatus, RiskTier,
    StructuredRationale, WitnessEntryBuilder, WitnessLog,
};

// ── Shared application state ───────────────────────────────────────────

/// Application state shared across all handlers via `Arc`.
#[derive(Clone)]
pub struct AppState {
    pub daemon: Arc<Mutex<GovernanceDaemon>>,
    pub witness: Arc<Mutex<WitnessLog>>,
    pub policy: Arc<Mutex<PolicyEngine>>,
    pub drift: Arc<Mutex<DriftEngine>>,
    pub store: Arc<Mutex<SqliteStore>>,
    pub denylist: Arc<Vec<String>>,
}

impl AppState {
    /// Create a new `AppState` for production use.
    ///
    /// Author keys are loaded from the environment via
    /// [`pai_governance_daemon::keyloader::build_author_keys`]. Required env
    /// vars: `PAI_AUTHOR_API_KEY` and `PAI_AUTHOR_SIGNING_KEY` (32-byte hex).
    ///
    /// **Behavior change in v1.3.2 (signature unchanged):** previous versions
    /// initialized author keys from compile-time defaults. v1.3.2 reads from
    /// the environment and **panics** with a setup-guide message if either
    /// env var is missing or malformed. The function signature stays
    /// `-> Self` so adopter call sites compile without modification, but
    /// mis-deployed callers fail loudly instead of running silently with an
    /// attacker-known signing key.
    ///
    /// For local testing or demo flows where env vars are inappropriate, use
    /// [`AppState::new_in_memory_demo`] (ephemeral keys, stderr warning,
    /// caller responsible for binding to 127.0.0.1).
    ///
    /// The canonical Result-based API surface (`try_new_in_memory ->
    /// Result<Self, KeyError>`) is deferred to v2.3.0 / 1.4.0 (Constitutional
    /// Amendment cycle, ~late May / early June 2026), with a documented
    /// migration guide.
    pub fn new_in_memory() -> Self {
        let (sk, vk, api_key) = keyloader::build_author_keys().expect(
            "PAI-Kernel daemon initialization failed: missing or invalid env vars · \
             see docs/INSTALL.md ENV setup section · \
             https://github.com/PAI-Kernel/pai-kernel/blob/main/docs/INSTALL.md",
        );
        Self::with_explicit_keys(sk, vk, &api_key)
    }

    /// Create a new `AppState` with ephemeral demo keys for local testing.
    ///
    /// Generates fresh in-memory keys per call and prints a stderr warning.
    /// Caller MUST bind to `127.0.0.1` only. Never use in production paths.
    ///
    /// Marked `#[doc(hidden)]` to discourage adopter use; this is a testing
    /// helper that may evolve in v2.3.0 / 1.4.0 alongside the canonical
    /// Result-based API.
    #[doc(hidden)]
    pub fn new_in_memory_demo() -> Self {
        let (sk, vk, api_key) = keyloader::build_demo_keys();
        Self::with_explicit_keys(sk, vk, &api_key)
    }

    /// Create a new `AppState` with caller-provided author keys (advanced).
    ///
    /// Marked `#[doc(hidden)]` — used internally by [`new_in_memory`] and
    /// [`new_in_memory_demo`] and exposed for integration test scaffolding.
    /// Adopters should not depend on this entry point; it may be made
    /// `pub(crate)` in v2.3.0 / 1.4.0 once the canonical Result-based API
    /// lands.
    #[doc(hidden)]
    pub fn with_explicit_keys(
        sk: ed25519_dalek::SigningKey,
        vk: ed25519_dalek::VerifyingKey,
        api_key: &str,
    ) -> Self {
        let daemon = GovernanceDaemon::new(10).with_author_keys(api_key, vk, Some(sk));
        let witness = WitnessLog::new();

        let mut policy = PolicyEngine::new();
        let _ = policy.add_policy(
            "consent.rego",
            include_str!("../policies/constitutional/consent.rego"),
        );
        let _ = policy.add_policy(
            "conservative.rego",
            include_str!("../policies/constitutional/conservative.rego"),
        );
        let _ = policy.add_policy(
            "denylist.rego",
            include_str!("../policies/operational/denylist.rego"),
        );

        let drift = DriftEngine::new(DriftThresholds::new(10.0, 30 * 86400));
        let store = SqliteStore::open_in_memory().expect("in-memory SQLite");

        Self {
            daemon: Arc::new(Mutex::new(daemon)),
            witness: Arc::new(Mutex::new(witness)),
            policy: Arc::new(Mutex::new(policy)),
            drift: Arc::new(Mutex::new(drift)),
            store: Arc::new(Mutex::new(store)),
            denylist: Arc::new(default_denylist()),
        }
    }
}

fn default_denylist() -> Vec<String> {
    vec![
        "retention_score".into(),
        "experiment_bucket".into(),
        "time_spent".into(),
        "engagement_score".into(),
        "conversion_rate".into(),
    ]
}

// ── Request / Response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GovRequest {
    pub session_id: Option<String>,
    pub author_id: Option<String>,
    pub timestamp: Option<u64>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct GovResponse {
    pub request_id: String,
    pub audit_ref: String,
    pub result: serde_json::Value,
    pub conservative_mode: bool,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub request_id: String,
    pub error: String,
    pub code: u16,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub witness_entries: usize,
    pub conservative_mode: bool,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: String,
    pub pai_cd_version: String,
    pub rust_toolchain: String,
}

// ── Helpers ────────────────────────────────────────────────────────────

fn request_id() -> String {
    Uuid::new_v4().to_string()
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn rid_headers(rid: &str) -> HeaderMap {
    let mut h = HeaderMap::new();
    if let Ok(val) = rid.parse() {
        h.insert("x-request-id", val);
    }
    h
}

/// Check context keys against the denylist. Returns violating keys.
fn check_denylist(context: &serde_json::Value, denylist: &[String]) -> Vec<String> {
    let mut violations = Vec::new();
    if let serde_json::Value::Object(map) = context {
        for key in map.keys() {
            if denylist.contains(key) {
                violations.push(key.clone());
            }
        }
    }
    violations
}

/// Append a governance witness entry.
fn append_witness(witness: &Mutex<WitnessLog>, action: &str, rid: &str) -> u64 {
    let mut w = witness.lock().unwrap();
    w.append(
        WitnessEntryBuilder::new()
            .decision_class(DecisionClass::GovAction)
            .timestamp(now_ts())
            .initiator(Initiator::Governance)
            .scope_of_impact(vec![ImpactScope::Governance])
            .risk_tier(RiskTier::Tier1)
            .rationale(
                StructuredRationale::new(&format!("{action} [{rid}]"))
                    .unwrap_or_else(|_| StructuredRationale::new("governance action").unwrap()),
            )
            .constitutional_ref(ConstitutionalRef("API §4.4 WitnessMiddleware".into()))
            .reversibility(ReversibilityStatus::Irreversible),
    )
    .unwrap_or(0)
}

fn gov_response(
    rid: &str,
    audit_seq: u64,
    result: serde_json::Value,
    conservative: bool,
) -> GovResponse {
    GovResponse {
        request_id: rid.into(),
        audit_ref: format!("WIT-{audit_seq}"),
        result,
        conservative_mode: conservative,
        timestamp: now_ts(),
    }
}

// ── Router ─────────────────────────────────────────────────────────────

/// Build the full API router.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Queries
        .route("/api/v1/health", get(health))
        .route("/api/v1/version", get(version))
        .route("/api/v1/state", get(get_state))
        .route("/api/v1/log", get(get_log))
        .route("/api/v1/log/verify", get(verify_log))
        .route("/api/v1/drift", get(get_drift))
        .route("/api/v1/export", get(export_bundle))
        // Mutations
        .route("/api/v1/gate/evaluate", post(gate_evaluate))
        .route("/api/v1/consent/grant", post(consent_grant))
        .route("/api/v1/consent/revoke", post(consent_revoke))
        .route("/api/v1/delegation/grant", post(delegation_grant))
        .route("/api/v1/delegation/revoke", post(delegation_revoke))
        .route("/api/v1/objective/add", post(objective_add))
        .route("/api/v1/snapshot", post(snapshot))
        .route("/api/v1/rollback", post(rollback))
        .route("/api/v1/conservative/enter", post(conservative_enter))
        .route("/api/v1/conservative/exit", post(conservative_exit))
        .with_state(state)
}

// ── Handlers: queries ──────────────────────────────────────────────────

async fn health(State(s): State<AppState>) -> impl IntoResponse {
    let rid = request_id();
    let w = s.witness.lock().unwrap();
    let d = s.daemon.lock().unwrap();
    let body = HealthResponse {
        status: "ok".into(),
        witness_entries: w.len(),
        conservative_mode: d.state().conservative(),
    };
    (StatusCode::OK, rid_headers(&rid), Json(body))
}

async fn version() -> impl IntoResponse {
    let rid = request_id();
    let body = VersionResponse {
        version: env!("CARGO_PKG_VERSION").into(),
        pai_cd_version: "3.1".into(),
        rust_toolchain: env!("CARGO_PKG_RUST_VERSION").into(),
    };
    (StatusCode::OK, rid_headers(&rid), Json(body))
}

async fn get_state(State(s): State<AppState>) -> impl IntoResponse {
    let rid = request_id();
    let d = s.daemon.lock().unwrap();
    let state_json = serde_json::json!({
        "conservative": d.state().conservative(),
        "drift": d.state().drift(),
        "drift_threshold": d.state().drift_threshold(),
        "objectives": d.state().objectives(),
        "breach_flag": d.state().breach_flag(),
    });
    (StatusCode::OK, rid_headers(&rid), Json(state_json))
}

async fn get_log(State(s): State<AppState>) -> impl IntoResponse {
    let rid = request_id();
    let w = s.witness.lock().unwrap();
    let entries = w.export();
    let body = serde_json::json!({
        "count": entries.len(),
        "entries": entries.iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect::<Vec<_>>(),
    });
    (StatusCode::OK, rid_headers(&rid), Json(body))
}

async fn verify_log(State(s): State<AppState>) -> impl IntoResponse {
    let rid = request_id();
    let w = s.witness.lock().unwrap();
    let verified = w.verify().is_ok();
    let body = serde_json::json!({
        "chain_valid": verified,
        "entry_count": w.len(),
    });
    (StatusCode::OK, rid_headers(&rid), Json(body))
}

async fn get_drift(State(s): State<AppState>) -> impl IntoResponse {
    let rid = request_id();
    let dr = s.drift.lock().unwrap();
    let report = dr.report();
    (
        StatusCode::OK,
        rid_headers(&rid),
        Json(serde_json::to_value(report).unwrap_or_default()),
    )
}

async fn export_bundle(State(s): State<AppState>) -> impl IntoResponse {
    let rid = request_id();
    let w = s.witness.lock().unwrap();
    let d = s.daemon.lock().unwrap();
    let entries = w.export();
    let body = serde_json::json!({
        "witness_log_count": w.len(),
        "witness_log": entries.iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect::<Vec<_>>(),
        "decision_log_count": d.log().len(),
        "conservative_mode": d.state().conservative(),
        "objectives": d.state().objectives(),
        "exported_at": now_ts(),
    });
    (StatusCode::OK, rid_headers(&rid), Json(body))
}

// ── Handlers: mutations ────────────────────────────────────────────────

async fn gate_evaluate(
    State(s): State<AppState>,
    body: Result<Json<GovRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let rid = request_id();
    let req = match body {
        Ok(Json(r)) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                rid_headers(&rid),
                Json(
                    serde_json::to_value(ErrorResponse {
                        request_id: rid,
                        error: e.to_string(),
                        code: 400,
                    })
                    .unwrap(),
                ),
            );
        }
    };

    // Context denylist check
    if let Some(ref ctx) = req.context {
        let violations = check_denylist(ctx, &s.denylist);
        if !violations.is_empty() {
            let seq = append_witness(&s.witness, "gate_evaluate:denylist_breach", &rid);
            let body = gov_response(
                &rid,
                seq,
                serde_json::json!({
                    "allow": false,
                    "breach": "GROWTH.SIGNAL.INJECTION",
                    "violations": violations,
                }),
                s.daemon.lock().unwrap().state().conservative(),
            );
            return (
                StatusCode::BAD_REQUEST,
                rid_headers(&rid),
                Json(serde_json::to_value(body).unwrap()),
            );
        }
    }

    let seq = append_witness(&s.witness, "gate_evaluate", &rid);
    let conservative = s.daemon.lock().unwrap().state().conservative();
    let body = gov_response(
        &rid,
        seq,
        serde_json::json!({
            "allow": !conservative,
            "mode": if conservative { "conservative" } else { "normal" },
        }),
        conservative,
    );
    (
        StatusCode::OK,
        rid_headers(&rid),
        Json(serde_json::to_value(body).unwrap()),
    )
}

async fn consent_grant(
    State(s): State<AppState>,
    body: Result<Json<GovRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let rid = request_id();
    if let Err(e) = body {
        return (
            StatusCode::BAD_REQUEST,
            rid_headers(&rid),
            Json(
                serde_json::to_value(ErrorResponse {
                    request_id: rid,
                    error: e.to_string(),
                    code: 400,
                })
                .unwrap(),
            ),
        );
    }
    let seq = append_witness(&s.witness, "consent_grant", &rid);
    let conservative = s.daemon.lock().unwrap().state().conservative();
    let body = gov_response(
        &rid,
        seq,
        serde_json::json!({"action": "consent_grant"}),
        conservative,
    );
    (
        StatusCode::OK,
        rid_headers(&rid),
        Json(serde_json::to_value(body).unwrap()),
    )
}

async fn consent_revoke(
    State(s): State<AppState>,
    body: Result<Json<GovRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let rid = request_id();
    if let Err(e) = body {
        return (
            StatusCode::BAD_REQUEST,
            rid_headers(&rid),
            Json(
                serde_json::to_value(ErrorResponse {
                    request_id: rid,
                    error: e.to_string(),
                    code: 400,
                })
                .unwrap(),
            ),
        );
    }
    let seq = append_witness(&s.witness, "consent_revoke", &rid);
    let conservative = s.daemon.lock().unwrap().state().conservative();
    (
        StatusCode::OK,
        rid_headers(&rid),
        Json(
            serde_json::to_value(gov_response(
                &rid,
                seq,
                serde_json::json!({"action": "consent_revoke"}),
                conservative,
            ))
            .unwrap(),
        ),
    )
}

async fn delegation_grant(
    State(s): State<AppState>,
    body: Result<Json<GovRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let rid = request_id();
    if let Err(e) = body {
        return (
            StatusCode::BAD_REQUEST,
            rid_headers(&rid),
            Json(
                serde_json::to_value(ErrorResponse {
                    request_id: rid,
                    error: e.to_string(),
                    code: 400,
                })
                .unwrap(),
            ),
        );
    }
    let seq = append_witness(&s.witness, "delegation_grant", &rid);
    let conservative = s.daemon.lock().unwrap().state().conservative();
    (
        StatusCode::OK,
        rid_headers(&rid),
        Json(
            serde_json::to_value(gov_response(
                &rid,
                seq,
                serde_json::json!({"action": "delegation_grant"}),
                conservative,
            ))
            .unwrap(),
        ),
    )
}

async fn delegation_revoke(
    State(s): State<AppState>,
    body: Result<Json<GovRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let rid = request_id();
    if let Err(e) = body {
        return (
            StatusCode::BAD_REQUEST,
            rid_headers(&rid),
            Json(
                serde_json::to_value(ErrorResponse {
                    request_id: rid,
                    error: e.to_string(),
                    code: 400,
                })
                .unwrap(),
            ),
        );
    }
    let seq = append_witness(&s.witness, "delegation_revoke", &rid);
    let conservative = s.daemon.lock().unwrap().state().conservative();
    (
        StatusCode::OK,
        rid_headers(&rid),
        Json(
            serde_json::to_value(gov_response(
                &rid,
                seq,
                serde_json::json!({"action": "delegation_revoke"}),
                conservative,
            ))
            .unwrap(),
        ),
    )
}

async fn objective_add(
    State(s): State<AppState>,
    body: Result<Json<GovRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let rid = request_id();
    if let Err(e) = body {
        return (
            StatusCode::BAD_REQUEST,
            rid_headers(&rid),
            Json(
                serde_json::to_value(ErrorResponse {
                    request_id: rid,
                    error: e.to_string(),
                    code: 400,
                })
                .unwrap(),
            ),
        );
    }
    let seq = append_witness(&s.witness, "objective_add", &rid);
    let conservative = s.daemon.lock().unwrap().state().conservative();
    (
        StatusCode::OK,
        rid_headers(&rid),
        Json(
            serde_json::to_value(gov_response(
                &rid,
                seq,
                serde_json::json!({"action": "objective_add"}),
                conservative,
            ))
            .unwrap(),
        ),
    )
}

async fn snapshot(
    State(s): State<AppState>,
    _body: Result<Json<GovRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let rid = request_id();
    s.daemon.lock().unwrap().snapshot();
    let seq = append_witness(&s.witness, "snapshot", &rid);
    let conservative = s.daemon.lock().unwrap().state().conservative();
    (
        StatusCode::OK,
        rid_headers(&rid),
        Json(
            serde_json::to_value(gov_response(
                &rid,
                seq,
                serde_json::json!({"action": "snapshot"}),
                conservative,
            ))
            .unwrap(),
        ),
    )
}

async fn rollback(
    State(s): State<AppState>,
    _body: Result<Json<GovRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let rid = request_id();
    let result = s.daemon.lock().unwrap().rollback();
    let seq = append_witness(&s.witness, "rollback", &rid);
    let conservative = s.daemon.lock().unwrap().state().conservative();
    let ok = result.is_ok();
    (
        if ok {
            StatusCode::OK
        } else {
            StatusCode::CONFLICT
        },
        rid_headers(&rid),
        Json(
            serde_json::to_value(gov_response(
                &rid,
                seq,
                serde_json::json!({"action": "rollback", "success": ok}),
                conservative,
            ))
            .unwrap(),
        ),
    )
}

async fn conservative_enter(State(s): State<AppState>) -> impl IntoResponse {
    let rid = request_id();
    s.daemon.lock().unwrap().inference_bypass_attempt(); // enters conservative
                                                         // Persist
    {
        let d = s.daemon.lock().unwrap();
        let _ = s
            .store
            .lock()
            .unwrap()
            .save_conservative_mode(d.state().conservative(), d.state().breach_flag());
    }
    let seq = append_witness(&s.witness, "conservative_enter", &rid);
    (
        StatusCode::OK,
        rid_headers(&rid),
        Json(
            serde_json::to_value(gov_response(
                &rid,
                seq,
                serde_json::json!({"conservative_mode": true}),
                true,
            ))
            .unwrap(),
        ),
    )
}

async fn conservative_exit(State(s): State<AppState>) -> impl IntoResponse {
    let rid = request_id();
    let result = {
        let mut d = s.daemon.lock().unwrap();
        d.clear_breach_for_testing();
        d.open_gate_for_testing();
        d.exit_conservative()
    };
    let conservative = s.daemon.lock().unwrap().state().conservative();
    if result.is_ok() {
        let _ = s.store.lock().unwrap().save_conservative_mode(false, None);
    }
    let seq = append_witness(&s.witness, "conservative_exit", &rid);
    (
        if result.is_ok() {
            StatusCode::OK
        } else {
            StatusCode::CONFLICT
        },
        rid_headers(&rid),
        Json(
            serde_json::to_value(gov_response(
                &rid,
                seq,
                serde_json::json!({"conservative_mode": conservative, "success": result.is_ok()}),
                conservative,
            ))
            .unwrap(),
        ),
    )
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_app() -> Router {
        create_router(AppState::new_in_memory_demo())
    }

    async fn body_json(resp: axum::http::Response<Body>) -> serde_json::Value {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    // ── API-T01: Health endpoint returns 200 ───────────────────────
    #[tokio::test]
    async fn api_t01_health_200() {
        let app = test_app();
        let resp = app
            .oneshot(Request::get("/api/v1/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().contains_key("x-request-id"));

        let body = body_json(resp).await;
        assert_eq!(body["status"], "ok");
    }

    // ── API-T02: Gate evaluate with valid request → response ───────
    #[tokio::test]
    async fn api_t02_gate_evaluate_valid() {
        let app = test_app();
        let req_body = serde_json::json!({
            "session_id": "S1",
            "author_id": "A1",
            "timestamp": 1000,
            "context": { "latency_ms": 50 }
        });
        let resp = app
            .oneshot(
                Request::post("/api/v1/gate/evaluate")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body["request_id"].is_string());
        assert!(body["audit_ref"].is_string());
        assert!(body["result"]["allow"].is_boolean());
    }

    // ── API-T03: Request with denylist key → 400 + breach ──────────
    #[tokio::test]
    async fn api_t03_denylist_key_400() {
        let state = AppState::new_in_memory_demo();
        let app = create_router(state.clone());
        let req_body = serde_json::json!({
            "session_id": "S1",
            "context": {
                "latency_ms": 50,
                "retention_score": 0.9,
                "experiment_bucket": "A"
            }
        });
        let resp = app
            .oneshot(
                Request::post("/api/v1/gate/evaluate")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = body_json(resp).await;
        assert_eq!(body["result"]["breach"], "GROWTH.SIGNAL.INJECTION");

        // Breach was logged in witness
        let w = state.witness.lock().unwrap();
        assert!(!w.is_empty(), "breach must produce witness entry");
    }

    // ── API-T04: Every mutation endpoint produces WitnessLog entry ─
    #[tokio::test]
    async fn api_t04_mutation_produces_witness() {
        let state = AppState::new_in_memory_demo();
        let initial_len = state.witness.lock().unwrap().len();

        let mutations = vec![
            "/api/v1/consent/grant",
            "/api/v1/consent/revoke",
            "/api/v1/delegation/grant",
            "/api/v1/delegation/revoke",
            "/api/v1/objective/add",
            "/api/v1/snapshot",
            "/api/v1/conservative/enter",
        ];

        for endpoint in &mutations {
            let app = create_router(state.clone());
            let req_body = serde_json::json!({ "session_id": "S1" });
            let resp = app
                .oneshot(
                    Request::post(*endpoint)
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert!(
                resp.status().is_success() || resp.status() == StatusCode::CONFLICT,
                "{} returned {}",
                endpoint,
                resp.status()
            );
        }

        let final_len = state.witness.lock().unwrap().len();
        assert_eq!(
            final_len,
            initial_len + mutations.len(),
            "each mutation must produce exactly one witness entry"
        );

        // Verify chain integrity
        assert!(state.witness.lock().unwrap().verify().is_ok());
    }

    // ── API-T05: Conservative Mode persists across restart ─────────
    #[tokio::test]
    async fn api_t05_conservative_persists() {
        let state = AppState::new_in_memory_demo();

        // Enter conservative mode
        let app = create_router(state.clone());
        let resp = app
            .oneshot(
                Request::post("/api/v1/conservative/enter")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify it's persisted in storage
        let (active, breach) = state
            .store
            .lock()
            .unwrap()
            .load_conservative_mode()
            .unwrap();
        assert!(active, "conservative mode must be persisted");
        assert!(breach.is_some(), "breach class must be persisted");

        // Simulate "restart": read from storage and verify
        let (active2, _) = state
            .store
            .lock()
            .unwrap()
            .load_conservative_mode()
            .unwrap();
        assert!(active2, "conservative mode must survive restart");
    }

    // ── API-T06: Concurrent requests maintain state consistency ────
    #[tokio::test]
    async fn api_t06_concurrent_consistency() {
        let state = AppState::new_in_memory_demo();
        let n = 20;
        let mut handles = Vec::new();

        for i in 0..n {
            let s = state.clone();
            handles.push(tokio::spawn(async move {
                let app = create_router(s);
                let req_body = serde_json::json!({ "session_id": format!("S{}", i) });
                let resp = app
                    .oneshot(
                        Request::post("/api/v1/consent/grant")
                            .header("content-type", "application/json")
                            .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                resp.status()
            }));
        }

        for h in handles {
            let status = h.await.unwrap();
            assert!(status.is_success(), "concurrent request failed: {status}");
        }

        // All 20 requests produced witness entries
        let w = state.witness.lock().unwrap();
        assert_eq!(
            w.len(),
            n,
            "all concurrent requests must produce witness entries"
        );
        assert!(
            w.verify().is_ok(),
            "witness chain must remain valid under concurrency"
        );
    }

    // ── API-T07: Unknown endpoint → 404 ────────────────────────────
    #[tokio::test]
    async fn api_t07_unknown_endpoint_404() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::get("/api/v1/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ── API-T08: Malformed JSON → 400 ──────────────────────────────
    #[tokio::test]
    async fn api_t08_malformed_json_400() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::post("/api/v1/consent/grant")
                    .header("content-type", "application/json")
                    .body(Body::from("{not valid json!!!}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = body_json(resp).await;
        assert!(body["error"].is_string(), "must return structured error");
    }
}
