//! Phase 1 integration tests — live daemon via HTTP.
//!
//! Each test starts the daemon on a random port, exercises the API, then shuts down.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use pai_api::{create_router, AppState};
use pai_drift::{DriftEngine, DriftThresholds};
use pai_governance_daemon::GovernanceDaemon;
use pai_policy::PolicyEngine;
use pai_storage::SqliteStore;
use pai_witness::WitnessLog;

/// Start a live server on a random port, return the base URL and a shutdown handle.
async fn start_server() -> (String, tokio::sync::oneshot::Sender<()>) {
    let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
    let vk = sk.verifying_key();
    let daemon = GovernanceDaemon::new(10).with_author_keys("API_KEY", vk, Some(sk));

    let mut policy = PolicyEngine::new();
    let _ = policy.add_policy(
        "consent.rego",
        include_str!("../../../policies/constitutional/consent.rego"),
    );
    let _ = policy.add_policy(
        "conservative.rego",
        include_str!("../../../policies/constitutional/conservative.rego"),
    );
    let _ = policy.add_policy(
        "denylist.rego",
        include_str!("../../../policies/operational/denylist.rego"),
    );

    let state = AppState {
        daemon: Arc::new(Mutex::new(daemon)),
        witness: Arc::new(Mutex::new(WitnessLog::new())),
        policy: Arc::new(Mutex::new(policy)),
        drift: Arc::new(Mutex::new(DriftEngine::new(DriftThresholds::new(
            10.0,
            30 * 86400,
        )))),
        store: Arc::new(Mutex::new(
            SqliteStore::open_in_memory().expect("in-memory SQLite"),
        )),
        denylist: Arc::new(vec![
            "retention_score".into(),
            "experiment_bucket".into(),
            "time_spent".into(),
            "engagement_score".into(),
            "conversion_rate".into(),
        ]),
    };

    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind to random port");
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async { rx.await.ok(); })
            .await
            .ok();
    });

    // Brief pause for server readiness
    tokio::time::sleep(Duration::from_millis(50)).await;

    (base, tx)
}

// ── INT-T01: Health endpoint returns 200 ──────────────────────────────

#[tokio::test]
async fn int_t01_health_200() {
    let (base, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/api/v1/health"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");

    let _ = shutdown.send(());
}

// ── INT-T02: Version endpoint ─────────────────────────────────────────

#[tokio::test]
async fn int_t02_version() {
    let (base, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/api/v1/version"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["version"].is_string());
    assert!(body["pai_cd_version"].is_string());

    let _ = shutdown.send(());
}

// ── INT-T03: Gate evaluate with valid JSON → GateResponse ─────────────

#[tokio::test]
async fn int_t03_gate_evaluate_valid() {
    let (base, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/api/v1/gate/evaluate"))
        .json(&serde_json::json!({
            "session_id": "test",
            "context": {"latency_ms": 50}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["request_id"].is_string());
    assert!(body["audit_ref"].is_string());
    assert!(body["result"]["allow"].is_boolean());

    let _ = shutdown.send(());
}

// ── INT-T04: Gate evaluate with denylist key → 400 + breach ───────────

#[tokio::test]
async fn int_t04_denylist_breach() {
    let (base, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/api/v1/gate/evaluate"))
        .json(&serde_json::json!({
            "session_id": "test",
            "context": {
                "latency_ms": 50,
                "retention_score": 0.9
            }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["result"]["breach"], "GROWTH.SIGNAL.INJECTION");

    let _ = shutdown.send(());
}

// ── INT-T05: Drift endpoint → DriftReport ─────────────────────────────

#[tokio::test]
async fn int_t05_drift_report() {
    let (base, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/api/v1/drift"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["composite_score"].is_number());
    assert!(body["threshold"].is_number());

    let _ = shutdown.send(());
}

// ── INT-T06: Conservative mode enter/check/exit ───────────────────────

#[tokio::test]
async fn int_t06_conservative_mode() {
    let (base, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    // Enter conservative mode
    let resp = client
        .post(format!("{base}/api/v1/conservative/enter"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Verify state
    let resp = client
        .get(format!("{base}/api/v1/state"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["conservative"], true);

    let _ = shutdown.send(());
}

// ── INT-T07: Export → valid ExportBundle JSON ─────────────────────────

#[tokio::test]
async fn int_t07_export_bundle() {
    let (base, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/api/v1/export"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["exported_at"].is_number());

    let _ = shutdown.send(());
}

// ── INT-T08: Log verify → chain_valid: true ───────────────────────────

#[tokio::test]
async fn int_t08_log_verify() {
    let (base, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/api/v1/log/verify"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["chain_valid"], true);

    let _ = shutdown.send(());
}

// ── CLI-T01: version subcommand ───────────────────────────────────────

#[test]
fn cli_t01_version() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_pai_governance_daemon"))
        .arg("version")
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1.3.1"), "stdout: {stdout}");
    assert!(stdout.contains("PAI-CD"), "stdout: {stdout}");
}

// ── CLI-T02: verify with empty DB ─────────────────────────────────────

#[test]
fn cli_t02_verify_empty() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_pai_governance_daemon"))
        .arg("verify")
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OK") || stdout.contains("valid") || stdout.contains("empty"),
        "stdout: {stdout}");
}

// ── CLI-T03: export outputs valid JSON ────────────────────────────────

#[test]
fn cli_t03_export_json() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_pai_governance_daemon"))
        .arg("export")
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "export must produce valid JSON: {stdout}");
    let val = parsed.unwrap();
    assert!(val["witness_log"].is_array());
    assert!(val["governance_state"].is_object());
}
