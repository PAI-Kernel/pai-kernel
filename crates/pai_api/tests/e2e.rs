//! # End-to-End Integration Tests
//!
//! Per PHASE1-TZ-001 Section 12: full lifecycle tests spanning
//! inference request → gate evaluate → policy check → HAC enforcement →
//! storage persist → witness log → verify chain.
//!
//! Three scenarios:
//!   E2E-T01: Normal flow
//!   E2E-T02: Breach flow (growth signal injection)
//!   E2E-T03: Conservative Mode flow

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use pai_api::{create_router, AppState};
use pai_config::KernelConfig;
use pai_export::{ExportBuilder, verify_bundle_integrity};
use pai_drift::DriftThresholds;
use pai_storage::GovernanceStore;

async fn body_json(resp: axum::http::Response<Body>) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn post_json(uri: &str, body: &serde_json::Value) -> Request<Body> {
    Request::post(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(body).unwrap()))
        .unwrap()
}

// ── E2E-T01: Normal flow ───────────────────────────────────────────────
//
// 1. Config loads with defaults
// 2. Gate evaluate with clean context → allowed
// 3. Consent grant → witness entry
// 4. Delegation grant → witness entry
// 5. State reflects operations
// 6. Verify witness chain integrity
// 7. Export bundle → valid JSON + integrity hash
#[tokio::test]
async fn e2e_t01_normal_flow() {
    // 1. Config
    let cfg = KernelConfig::default_config();
    assert_eq!(cfg.bind_addr(), "127.0.0.1:9100");

    // 2. Gate evaluate — clean context
    let state = AppState::new_in_memory();
    let app = create_router(state.clone());
    let resp = app
        .oneshot(post_json(
            "/api/v1/gate/evaluate",
            &serde_json::json!({
                "session_id": "E2E-S1",
                "author_id": "AUTHOR",
                "context": { "latency_ms": 50 }
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert!(body["result"]["allow"].as_bool().unwrap_or(false));

    // 3. Consent grant → witness entry
    let app2 = create_router(state.clone());
    let resp2 = app2
        .oneshot(post_json(
            "/api/v1/consent/grant",
            &serde_json::json!({ "session_id": "E2E-S1" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    // 4. Delegation grant → witness entry
    let app3 = create_router(state.clone());
    let resp3 = app3
        .oneshot(post_json(
            "/api/v1/delegation/grant",
            &serde_json::json!({ "session_id": "E2E-S1" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp3.status(), StatusCode::OK);

    // 5. State reflects operations
    let app4 = create_router(state.clone());
    let resp4 = app4
        .oneshot(Request::get("/api/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let health = body_json(resp4).await;
    assert_eq!(health["status"], "ok");
    assert!(health["witness_entries"].as_u64().unwrap() >= 3);

    // 6. Verify witness chain
    let app5 = create_router(state.clone());
    let resp5 = app5
        .oneshot(
            Request::get("/api/v1/log/verify")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let verify = body_json(resp5).await;
    assert_eq!(verify["chain_valid"], true);

    // 7. Export bundle
    let d = state.daemon.lock().unwrap();
    let w = state.witness.lock().unwrap();
    let thresholds = DriftThresholds::new(10.0, 30 * 86400);
    let bundle = ExportBuilder::new(&d, &w, &thresholds).build();
    assert!(verify_bundle_integrity(&bundle));
    assert!(bundle.metadata.witness_chain_verified);
    let json = serde_json::to_string(&bundle);
    assert!(json.is_ok(), "export must produce valid JSON");
}

// ── E2E-T02: Breach flow (growth signal injection) ─────────────────────
//
// 1. Send gate evaluate with denylist keys → 400 + breach
// 2. Breach recorded in witness log
// 3. State shows breach details
// 4. Witness chain still valid after breach
#[tokio::test]
async fn e2e_t02_breach_flow() {
    let state = AppState::new_in_memory();

    // 1. Growth signal injection → 400
    let app = create_router(state.clone());
    let resp = app
        .oneshot(post_json(
            "/api/v1/gate/evaluate",
            &serde_json::json!({
                "session_id": "E2E-BREACH",
                "context": {
                    "latency_ms": 10,
                    "retention_score": 0.95,
                    "experiment_bucket": "A"
                }
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_json(resp).await;
    assert_eq!(body["result"]["breach"], "GROWTH.SIGNAL.INJECTION");

    // 2. Breach recorded in witness log
    let w = state.witness.lock().unwrap();
    assert!(w.len() > 0, "breach must create witness entry");

    // 3. Verify breach was logged
    assert!(w.verify().is_ok(), "witness chain valid after breach");
    drop(w);

    // 4. Further clean requests still work
    let app2 = create_router(state.clone());
    let resp2 = app2
        .oneshot(post_json(
            "/api/v1/gate/evaluate",
            &serde_json::json!({
                "session_id": "E2E-CLEAN",
                "context": { "safe_key": "safe_value" }
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
}

// ── E2E-T03: Conservative Mode flow ────────────────────────────────────
//
// 1. Enter Conservative Mode via API
// 2. Conservative Mode persists in storage
// 3. Health endpoint shows conservative_mode: true
// 4. Gate evaluate reflects conservative state
// 5. Exit Conservative Mode
// 6. Health shows conservative_mode: false
// 7. Full witness chain valid throughout
#[tokio::test]
async fn e2e_t03_conservative_mode_flow() {
    let state = AppState::new_in_memory();

    // 1. Enter Conservative Mode
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

    // 2. Persisted in storage
    {
        let store = state.store.lock().unwrap();
        let (active, breach) = store.load_conservative_mode().unwrap();
        assert!(active, "conservative mode must persist");
        assert!(breach.is_some());
    }

    // 3. Health shows conservative_mode
    let app2 = create_router(state.clone());
    let resp2 = app2
        .oneshot(Request::get("/api/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let health = body_json(resp2).await;
    assert_eq!(health["conservative_mode"], true);

    // 4. Gate evaluate reflects conservative state
    let app3 = create_router(state.clone());
    let resp3 = app3
        .oneshot(post_json(
            "/api/v1/gate/evaluate",
            &serde_json::json!({ "session_id": "E2E-CONS" }),
        ))
        .await
        .unwrap();
    let gate = body_json(resp3).await;
    assert_eq!(gate["conservative_mode"], true);

    // 5. Exit Conservative Mode
    let app4 = create_router(state.clone());
    let resp4 = app4
        .oneshot(
            Request::post("/api/v1/conservative/exit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp4.status(), StatusCode::OK);

    // 6. Health shows conservative_mode: false
    let app5 = create_router(state.clone());
    let resp5 = app5
        .oneshot(Request::get("/api/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let health2 = body_json(resp5).await;
    assert_eq!(health2["conservative_mode"], false);

    // 7. Full witness chain valid throughout
    let w = state.witness.lock().unwrap();
    assert!(w.verify().is_ok(), "witness chain must remain valid through entire flow");
    assert!(w.len() >= 2, "enter + exit = at least 2 witness entries");
}
