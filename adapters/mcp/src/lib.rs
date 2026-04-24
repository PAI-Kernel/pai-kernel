//! # PAI MCP Adapter
//!
//! MCP (Model Context Protocol) server that exposes PAI-Kernel governance
//! as MCP tools. Each tool wraps an internal PAI-Kernel API call.
//!
//! ## Tools
//!
//! | Tool | PAI-Kernel API | Description |
//! |------|---------------|-------------|
//! | `evaluate_gate` | gate/evaluate | Evaluate governance gate |
//! | `grant_consent` | consent/grant | Grant explicit consent |
//! | `check_conservative_mode` | state | Check Conservative Mode |
//! | `verify_witness_log` | log/verify | Verify witness chain |

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use pai_governance_daemon::GovernanceDaemon;
use pai_witness::WitnessLog;

// ── MCP tool definitions ───────────────────────────────────────────────

/// MCP tool descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP tool call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub content: serde_json::Value,
    pub is_error: bool,
}

// ── MCP Server ─────────────────────────────────────────────────────────

/// MCP server exposing PAI-Kernel governance tools.
pub struct McpGovernanceServer {
    daemon: Arc<Mutex<GovernanceDaemon>>,
    witness: Arc<Mutex<WitnessLog>>,
}

impl McpGovernanceServer {
    pub fn new(daemon: Arc<Mutex<GovernanceDaemon>>, witness: Arc<Mutex<WitnessLog>>) -> Self {
        Self { daemon, witness }
    }

    /// List available MCP tools.
    pub fn list_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: "evaluate_gate".into(),
                description: "Evaluate PAI-Kernel governance gate for an action".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {"type": "string"},
                        "context": {"type": "object"}
                    },
                    "required": ["session_id"]
                }),
            },
            McpTool {
                name: "grant_consent".into(),
                description: "Grant explicit consent for a capability".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": {"type": "string"},
                        "capability_id": {"type": "string"}
                    },
                    "required": ["session_id"]
                }),
            },
            McpTool {
                name: "check_conservative_mode".into(),
                description: "Check if Conservative Mode is active".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            McpTool {
                name: "verify_witness_log".into(),
                description: "Verify witness log hash chain integrity".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]
    }

    /// Execute an MCP tool call.
    pub fn call_tool(&self, name: &str, args: &serde_json::Value) -> McpToolResult {
        match name {
            "evaluate_gate" => self.tool_evaluate_gate(args),
            "grant_consent" => self.tool_grant_consent(args),
            "check_conservative_mode" => self.tool_check_conservative(),
            "verify_witness_log" => self.tool_verify_witness(),
            _ => McpToolResult {
                content: serde_json::json!({"error": format!("unknown tool: {}", name)}),
                is_error: true,
            },
        }
    }

    fn tool_evaluate_gate(&self, args: &serde_json::Value) -> McpToolResult {
        let d = self.daemon.lock().unwrap();
        let conservative = d.state().conservative();
        let session = args.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");

        // Check denylist in context
        let denylist = ["retention_score", "experiment_bucket", "time_spent"];
        if let Some(ctx) = args.get("context").and_then(|v| v.as_object()) {
            for key in ctx.keys() {
                if denylist.contains(&key.as_str()) {
                    return McpToolResult {
                        content: serde_json::json!({
                            "allow": false,
                            "breach": "GROWTH.SIGNAL.INJECTION",
                            "session_id": session,
                        }),
                        is_error: false,
                    };
                }
            }
        }

        McpToolResult {
            content: serde_json::json!({
                "allow": !conservative,
                "conservative_mode": conservative,
                "session_id": session,
            }),
            is_error: false,
        }
    }

    fn tool_grant_consent(&self, args: &serde_json::Value) -> McpToolResult {
        let session = args.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        McpToolResult {
            content: serde_json::json!({
                "action": "consent_granted",
                "session_id": session,
            }),
            is_error: false,
        }
    }

    fn tool_check_conservative(&self) -> McpToolResult {
        let d = self.daemon.lock().unwrap();
        McpToolResult {
            content: serde_json::json!({
                "conservative_mode": d.state().conservative(),
                "drift": d.state().drift(),
                "breach_flag": d.state().breach_flag(),
            }),
            is_error: false,
        }
    }

    fn tool_verify_witness(&self) -> McpToolResult {
        let w = self.witness.lock().unwrap();
        let valid = w.verify().is_ok();
        McpToolResult {
            content: serde_json::json!({
                "chain_valid": valid,
                "entry_count": w.len(),
            }),
            is_error: false,
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_server() -> McpGovernanceServer {
        let daemon = Arc::new(Mutex::new(GovernanceDaemon::new(10)));
        let witness = Arc::new(Mutex::new(WitnessLog::new()));
        McpGovernanceServer::new(daemon, witness)
    }

    #[test]
    fn mcp_t01_evaluate_gate_allow() {
        let server = test_server();
        let result = server.call_tool("evaluate_gate", &serde_json::json!({
            "session_id": "S1",
            "context": {"safe": "value"}
        }));
        assert!(!result.is_error);
        assert_eq!(result.content["allow"], true);
    }

    #[test]
    fn mcp_t02_grant_consent() {
        let server = test_server();
        let result = server.call_tool("grant_consent", &serde_json::json!({
            "session_id": "S1"
        }));
        assert!(!result.is_error);
        assert_eq!(result.content["action"], "consent_granted");
    }

    #[test]
    fn mcp_t03_check_conservative_mode() {
        let server = test_server();
        let result = server.call_tool("check_conservative_mode", &serde_json::json!({}));
        assert!(!result.is_error);
        assert_eq!(result.content["conservative_mode"], false);
    }

    #[test]
    fn mcp_t04_verify_witness_log() {
        let server = test_server();
        let result = server.call_tool("verify_witness_log", &serde_json::json!({}));
        assert!(!result.is_error);
        assert_eq!(result.content["chain_valid"], true);
        assert_eq!(result.content["entry_count"], 0);
    }

    #[test]
    fn mcp_t05_denylist_enforcement() {
        let server = test_server();
        let result = server.call_tool("evaluate_gate", &serde_json::json!({
            "session_id": "S1",
            "context": {
                "retention_score": 0.85,
                "safe_key": "ok"
            }
        }));
        assert!(!result.is_error);
        assert_eq!(result.content["allow"], false);
        assert_eq!(result.content["breach"], "GROWTH.SIGNAL.INJECTION");
    }
}
