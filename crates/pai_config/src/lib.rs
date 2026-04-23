//! # PAI Startup & Configuration (Component #6)
//!
//! Per PHASE1-TZ-001 Section 9 — config file parsing + CLI definitions.
//!
//! Parses `pai-kernel.toml`, provides defaults, validates structure.
//! CLI argument parsing uses `clap` (in the binary crate); this library
//! provides the config model and loader.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Error ──────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(String),
    #[error("config parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Config model ───────────────────────────────────────────────────────

/// Top-level configuration per Section 9.1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelConfig {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    #[serde(default = "default_storage")]
    pub storage: StorageConfig,
    #[serde(default = "default_policy")]
    pub policy: PolicyConfig,
    #[serde(default = "default_drift")]
    pub drift: DriftConfig,
    #[serde(default = "default_logging")]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    #[serde(default = "default_bind")]
    pub bind: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageConfig {
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: String,
    pub postgres_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyConfig {
    #[serde(default = "default_rego_dir")]
    pub rego_dir: String,
    #[serde(default = "default_reload_interval")]
    pub reload_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriftConfig {
    #[serde(default = "default_obj_changes")]
    pub objective_changes_30d: u64,
    #[serde(default = "default_class_changes")]
    pub classification_changes_30d: u64,
    #[serde(default = "default_consecutive")]
    pub consecutive_upgrades_without_gap: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

// ── Defaults ───────────────────────────────────────────────────────────

fn default_server() -> ServerConfig {
    ServerConfig { bind: default_bind(), port: default_port(), tls: false }
}
fn default_storage() -> StorageConfig {
    StorageConfig { backend: default_backend(), sqlite_path: default_sqlite_path(), postgres_url: None }
}
fn default_policy() -> PolicyConfig {
    PolicyConfig { rego_dir: default_rego_dir(), reload_interval_secs: default_reload_interval() }
}
fn default_drift() -> DriftConfig {
    DriftConfig {
        objective_changes_30d: default_obj_changes(),
        classification_changes_30d: default_class_changes(),
        consecutive_upgrades_without_gap: default_consecutive(),
    }
}
fn default_logging() -> LoggingConfig {
    LoggingConfig { level: default_log_level(), format: default_log_format() }
}

fn default_bind() -> String { "127.0.0.1".into() }
fn default_port() -> u16 { 9100 }
fn default_backend() -> String { "sqlite".into() }
fn default_sqlite_path() -> String { "./pai-kernel.db".into() }
fn default_rego_dir() -> String { "./policies/".into() }
fn default_reload_interval() -> u64 { 30 }
fn default_obj_changes() -> u64 { 5 }
fn default_class_changes() -> u64 { 3 }
fn default_consecutive() -> u64 { 2 }
fn default_log_level() -> String { "info".into() }
fn default_log_format() -> String { "json".into() }

// ── Loading ────────────────────────────────────────────────────────────

impl KernelConfig {
    /// Load configuration from a TOML file path.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound(format!(
                "Configuration file not found: {}. \
                 Create a pai-kernel.toml or pass --config <path>.",
                path.display()
            )));
        }
        let content = std::fs::read_to_string(path)?;
        Self::parse_toml(&content)
    }

    /// Parse from a TOML string.
    pub fn parse_toml(toml_str: &str) -> Result<Self, ConfigError> {
        toml::from_str(toml_str).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Return the default configuration (all defaults).
    pub fn default_config() -> Self {
        Self {
            server: default_server(),
            storage: default_storage(),
            policy: default_policy(),
            drift: default_drift(),
            logging: default_logging(),
        }
    }

    /// Socket address for the server.
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.bind, self.server.port)
    }
}

/// CLI command enum (mirrors Section 9.2).
#[derive(Debug, Clone, PartialEq)]
pub enum CliCommand {
    /// Start the server (default).
    Serve { config_path: Option<String> },
    /// Verify witness chain integrity.
    Verify,
    /// Export bundle to stdout.
    Export,
    /// Print version info.
    Version,
}

/// Parse CLI arguments (simple parser, clap-compatible interface).
pub fn parse_cli_args(args: &[String]) -> CliCommand {
    if args.len() <= 1 {
        return CliCommand::Serve { config_path: None };
    }
    match args[1].as_str() {
        "verify" => CliCommand::Verify,
        "export" => CliCommand::Export,
        "version" => CliCommand::Version,
        "--config" => {
            let path = args.get(2).cloned();
            CliCommand::Serve { config_path: path }
        }
        _ => CliCommand::Serve {
            config_path: if args[1] == "--config" { args.get(2).cloned() } else { None },
        },
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── CFG-T01: Default config → starts on 127.0.0.1:9100 ────────
    #[test]
    fn cfg_t01_default_config() {
        let cfg = KernelConfig::default_config();
        assert_eq!(cfg.server.bind, "127.0.0.1");
        assert_eq!(cfg.server.port, 9100);
        assert_eq!(cfg.bind_addr(), "127.0.0.1:9100");
        assert!(!cfg.server.tls);
        assert_eq!(cfg.storage.backend, "sqlite");
    }

    // ── CFG-T02: Custom config path → loads correctly ──────────────
    #[test]
    fn cfg_t02_custom_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom.toml");
        std::fs::write(
            &path,
            r#"
[server]
bind = "0.0.0.0"
port = 8080
tls = true

[storage]
backend = "postgres"
sqlite_path = "./data.db"
postgres_url = "postgres://localhost/pai"

[drift]
objective_changes_30d = 10
"#,
        )
        .unwrap();

        let cfg = KernelConfig::from_file(&path).unwrap();
        assert_eq!(cfg.server.bind, "0.0.0.0");
        assert_eq!(cfg.server.port, 8080);
        assert!(cfg.server.tls);
        assert_eq!(cfg.storage.backend, "postgres");
        assert_eq!(cfg.storage.postgres_url.as_deref(), Some("postgres://localhost/pai"));
        assert_eq!(cfg.drift.objective_changes_30d, 10);
        // Defaults for omitted sections
        assert_eq!(cfg.policy.rego_dir, "./policies/");
        assert_eq!(cfg.logging.level, "info");
    }

    // ── CFG-T03: Missing config → error with helpful message ───────
    #[test]
    fn cfg_t03_missing_config_error() {
        let result = KernelConfig::from_file(std::path::Path::new("/nonexistent/pai-kernel.toml"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("not found"),
            "error must mention 'not found': {}",
            msg
        );
        assert!(
            msg.contains("pai-kernel.toml") || msg.contains("--config"),
            "error must be helpful: {}",
            msg
        );
    }

    // ── CFG-T04: `pai-kernel verify` → chain integrity CLI command ─
    #[test]
    fn cfg_t04_verify_cli() {
        let args: Vec<String> = vec!["pai-kernel".into(), "verify".into()];
        let cmd = parse_cli_args(&args);
        assert_eq!(cmd, CliCommand::Verify);
    }

    // ── CFG-T05: `pai-kernel export` → export CLI command ──────────
    #[test]
    fn cfg_t05_export_cli() {
        let args: Vec<String> = vec!["pai-kernel".into(), "export".into()];
        let cmd = parse_cli_args(&args);
        assert_eq!(cmd, CliCommand::Export);

        // Also test version
        let args2: Vec<String> = vec!["pai-kernel".into(), "version".into()];
        assert_eq!(parse_cli_args(&args2), CliCommand::Version);

        // Default (no args) → Serve
        let args3: Vec<String> = vec!["pai-kernel".into()];
        assert_eq!(
            parse_cli_args(&args3),
            CliCommand::Serve { config_path: None }
        );

        // --config path
        let args4: Vec<String> = vec![
            "pai-kernel".into(),
            "--config".into(),
            "/etc/pai.toml".into(),
        ];
        assert_eq!(
            parse_cli_args(&args4),
            CliCommand::Serve {
                config_path: Some("/etc/pai.toml".into())
            }
        );
    }
}
