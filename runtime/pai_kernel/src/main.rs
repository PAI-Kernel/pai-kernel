//! PAI-Kernel Governance Sidecar — Phase 1 Live Daemon
//!
//! Wires all existing crates into a live axum server process.
//! CLI: pai-kernel [verify|export|version] via clap.
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use clap::{Parser, Subcommand};
use tracing::info;

use pai_api::{create_router, AppState};
use pai_config::KernelConfig;
use pai_drift::{DriftEngine, DriftThresholds};
use pai_export::ExportBuilder;
use pai_governance_daemon::GovernanceDaemon;
use pai_policy::PolicyEngine;
use pai_storage::{GovernanceStore, SqliteStore};
use pai_witness::WitnessLog;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PAI_CD_VERSION: &str = "3.1";

// ── CLI ───────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "pai-kernel", version = VERSION, about = "PAI-Kernel Governance Sidecar")]
struct Cli {
    /// Path to configuration file
    #[arg(long, default_value = "./pai-kernel.toml")]
    config: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Verify the witness chain integrity
    Verify,
    /// Export the full governance bundle as JSON to stdout
    Export,
    /// Print version information
    Version,
}

// ── Main ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Version) => {
            println!("PAI-Kernel Governance Sidecar v{VERSION}");
            println!("PAI-CD: v{PAI_CD_VERSION}");
            println!("Rust: 1.86.0");
        }
        Some(Command::Verify) => {
            let config = load_config(&cli.config);
            let store = open_store(&config);
            let decisions = store.decisions().expect("failed to load decisions");
            if decisions.is_empty() {
                println!("Witness chain: empty (no entries). OK.");
                return;
            }
            for i in 1..decisions.len() {
                if decisions[i].prev_hash() != decisions[i - 1].hash() {
                    eprintln!("Chain broken at seq {}", decisions[i].seq());
                    std::process::exit(1);
                }
            }
            println!(
                "Witness chain: valid. {} decision entries verified.",
                decisions.len()
            );
        }
        Some(Command::Export) => {
            let config = load_config(&cli.config);
            let daemon = build_daemon(&config);
            let witness = WitnessLog::new();
            let thresholds = DriftThresholds::new(10.0, 30 * 86400);
            let bundle = ExportBuilder::new(&daemon, &witness, &thresholds).build();
            let json =
                serde_json::to_string_pretty(&bundle).expect("failed to serialize export bundle");
            println!("{json}");
        }
        None => {
            run_server(cli.config).await;
        }
    }
}

// ── Server ────────────────────────────────────────────────────────────

async fn run_server(config_path: String) {
    let config = load_config(&config_path);
    init_tracing(&config);

    info!(version = VERSION, pai_cd = PAI_CD_VERSION, "Starting PAI-Kernel");

    let store = open_store(&config);

    // Restore conservative mode from storage
    let (conservative_active, _breach) = store
        .load_conservative_mode()
        .unwrap_or((false, None));

    let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
    let vk = sk.verifying_key();
    let mut daemon = GovernanceDaemon::new(10).with_author_keys("API_KEY", vk, Some(sk));

    if conservative_active {
        daemon.inference_bypass_attempt();
    }

    // Init policy engine
    let mut policy = PolicyEngine::new();
    let rego_path = std::path::Path::new(&config.policy.rego_dir);
    if rego_path.is_dir() {
        match policy.load_policy_tree(rego_path) {
            Ok(count) => info!(count, rego_dir = %config.policy.rego_dir, "Loaded policies"),
            Err(e) => tracing::warn!(error = %e, "Failed to load policy tree"),
        }
    }

    // Init drift engine
    let drift = DriftEngine::new(DriftThresholds::new(10.0, 30 * 86400));

    // Build AppState (same pattern as pai_api tests)
    let state = AppState {
        daemon: Arc::new(Mutex::new(daemon)),
        witness: Arc::new(Mutex::new(WitnessLog::new())),
        policy: Arc::new(Mutex::new(policy)),
        drift: Arc::new(Mutex::new(drift)),
        store: Arc::new(Mutex::new(store)),
        denylist: Arc::new(default_denylist()),
    };

    let app = create_router(state);

    let addr = config.bind_addr();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {addr}: {e}");
            std::process::exit(1);
        });

    info!(
        addr = %addr,
        "PAI-Kernel Governance Sidecar v{VERSION} listening on {addr}"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");

    info!("PAI-Kernel shutdown complete");
}

// ── Helpers ───────────────────────────────────────────────────────────

fn load_config(path: &str) -> KernelConfig {
    let p = std::path::Path::new(path);
    if p.exists() {
        KernelConfig::from_file(p).unwrap_or_else(|e| {
            eprintln!("Warning: failed to parse config: {e}, using defaults");
            KernelConfig::default_config()
        })
    } else {
        KernelConfig::default_config()
    }
}

fn open_store(config: &KernelConfig) -> SqliteStore {
    SqliteStore::open(&config.storage.sqlite_path).unwrap_or_else(|_| {
        SqliteStore::open_in_memory().expect("in-memory SQLite must succeed")
    })
}

fn build_daemon(config: &KernelConfig) -> GovernanceDaemon {
    let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
    let vk = sk.verifying_key();
    let mut daemon = GovernanceDaemon::new(10).with_author_keys("API_KEY", vk, Some(sk));
    let store = open_store(config);
    let (active, _) = store.load_conservative_mode().unwrap_or((false, None));
    if active {
        daemon.inference_bypass_attempt();
    }
    daemon
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

fn init_tracing(config: &KernelConfig) {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));
    if config.logging.format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    info!("Received shutdown signal");
}
