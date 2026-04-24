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
    /// Write a default pai-kernel.toml config + policies/ skeleton
    /// into the current directory (or the config path passed via --config).
    /// Does not overwrite existing files unless --force is passed.
    Init {
        /// Overwrite existing config file if present
        #[arg(long)]
        force: bool,
    },
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
        Some(Command::Init { force }) => {
            init_scaffold(&cli.config, force);
            return;
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

// ── Init scaffold ─────────────────────────────────────────────────────
//
// `pai_governance_daemon init` creates a default config file + a minimal
// policies/ directory in the current working directory. Useful for first-run
// adopters who want zero-config startup.

const DEFAULT_CONFIG_TOML: &str = r#"# pai-kernel.toml — default configuration generated by `pai_governance_daemon init`
# Adjust these values to match your deployment. See INSTALL.md + KNOWN_LIMITATIONS.md.

[server]
bind = "127.0.0.1"
port = 9100
tls = false

[storage]
backend = "sqlite"
sqlite_path = "./pai-kernel.db"

[policy]
rego_dir = "./policies/"
reload_interval_secs = 30

[drift]
objective_changes_30d = 5
classification_changes_30d = 3
consecutive_upgrades_without_gap = 2

[logging]
level = "info"
format = "json"
"#;

const PLACEHOLDER_POLICY: &str = r#"# Minimal constitutional-layer policy placeholder.
# The daemon loads all *.rego files from rego_dir at startup.
# Replace with your own Rego modules for production use.
# See: https://github.com/PAI-Kernel/pai-kernel/tree/main/policies

package pai.constitutional

default allow = false

allow {
    input.kind == "query"
}
"#;

fn init_scaffold(config_path: &str, force: bool) {
    use std::fs;
    use std::path::Path;

    let config = Path::new(config_path);
    let policies_dir = config
        .parent()
        .map(|p| p.join("policies"))
        .unwrap_or_else(|| Path::new("policies").to_path_buf());

    // Config file
    if config.exists() && !force {
        eprintln!(
            "pai-kernel.toml already exists at {}. Use --force to overwrite.",
            config.display()
        );
        std::process::exit(1);
    }
    if let Some(parent) = config.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("failed to create {}: {}", parent.display(), e));
        }
    }
    fs::write(config, DEFAULT_CONFIG_TOML)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", config.display(), e));
    println!("Created: {}", config.display());

    // policies/ skeleton
    if !policies_dir.exists() {
        fs::create_dir_all(&policies_dir).unwrap_or_else(|e| {
            panic!("failed to create {}: {}", policies_dir.display(), e)
        });
        let placeholder = policies_dir.join("placeholder.rego");
        fs::write(&placeholder, PLACEHOLDER_POLICY)
            .unwrap_or_else(|e| panic!("failed to write {}: {}", placeholder.display(), e));
        println!("Created: {}/", policies_dir.display());
        println!("Created: {}", placeholder.display());
    } else if force {
        // Don't touch existing policies under --force
        println!("Skipped: {}/ (already exists)", policies_dir.display());
    }

    println!();
    println!("Scaffold ready. Next:");
    println!("  pai_governance_daemon --config {}", config.display());
    println!();
    println!("Documentation: https://github.com/PAI-Kernel/pai-kernel/blob/main/INSTALL.md");
}
