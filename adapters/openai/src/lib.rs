//! # PAI OpenAI-Compatible Adapter
//!
//! Middleware that wraps OpenAI API calls with PAI-Kernel governance:
//! - **Pre-request**: gate evaluation + policy check
//! - **Post-response**: witness log entry
//!
//! Works with any OpenAI-compatible API (OpenAI, Azure OpenAI, local vLLM, etc.)

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use pai_governance_daemon::GovernanceDaemon;
use pai_witness::{
    ConstitutionalRef, DecisionClass, ImpactScope, Initiator,
    ReversibilityStatus, RiskTier, StructuredRationale, WitnessEntryBuilder, WitnessLog,
};

// ── Types ──────────────────────────────────────────────────────────────

/// Represents an OpenAI-style chat completion request (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Gate decision before forwarding to the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreGateResult {
    pub allow: bool,
    pub conservative_mode: bool,
    pub denial_reason: Option<String>,
    pub audit_ref: String,
}

/// Post-response audit record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostAuditResult {
    pub witness_seq: u64,
    pub chain_valid: bool,
}

// ── Middleware ──────────────────────────────────────────────────────────

/// Governance middleware for OpenAI-compatible API calls.
pub struct GovernanceMiddleware {
    daemon: Arc<Mutex<GovernanceDaemon>>,
    witness: Arc<Mutex<WitnessLog>>,
    denylist: Vec<String>,
}

impl GovernanceMiddleware {
    pub fn new(
        daemon: Arc<Mutex<GovernanceDaemon>>,
        witness: Arc<Mutex<WitnessLog>>,
    ) -> Self {
        Self {
            daemon,
            witness,
            denylist: vec![
                "retention_score".into(),
                "experiment_bucket".into(),
                "time_spent".into(),
            ],
        }
    }

    /// Pre-request gate: evaluate before forwarding to LLM.
    ///
    /// Returns Allow/Deny. If Deny, the request MUST NOT be forwarded.
    pub fn pre_gate(&self, request: &ChatRequest) -> PreGateResult {
        let d = self.daemon.lock().unwrap();
        let conservative = d.state().conservative();

        // Check metadata for denylist keys
        if let Some(obj) = request.metadata.as_object() {
            for key in obj.keys() {
                if self.denylist.contains(key) {
                    let seq = self.append_witness("pre_gate:denylist_breach");
                    return PreGateResult {
                        allow: false,
                        conservative_mode: conservative,
                        denial_reason: Some(format!("GROWTH.SIGNAL.INJECTION: {}", key)),
                        audit_ref: format!("WIT-{}", seq),
                    };
                }
            }
        }

        // Conservative mode blocks high-tier operations
        if conservative {
            let seq = self.append_witness("pre_gate:conservative_block");
            return PreGateResult {
                allow: false,
                conservative_mode: true,
                denial_reason: Some("Conservative Mode active".into()),
                audit_ref: format!("WIT-{}", seq),
            };
        }

        let seq = self.append_witness("pre_gate:allow");
        PreGateResult {
            allow: true,
            conservative_mode: false,
            denial_reason: None,
            audit_ref: format!("WIT-{}", seq),
        }
    }

    /// Post-response audit: log the LLM interaction in the witness chain.
    pub fn post_audit(&self, model: &str, _response_snippet: &str) -> PostAuditResult {
        let seq = self.append_witness(&format!("post_audit:model={}", model));
        let w = self.witness.lock().unwrap();
        PostAuditResult {
            witness_seq: seq,
            chain_valid: w.verify().is_ok(),
        }
    }

    fn append_witness(&self, action: &str) -> u64 {
        let mut w = self.witness.lock().unwrap();
        w.append(
            WitnessEntryBuilder::new()
                .decision_class(DecisionClass::GovAction)
                .timestamp(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                )
                .initiator(Initiator::Governance)
                .scope_of_impact(vec![ImpactScope::Governance])
                .risk_tier(RiskTier::Tier1)
                .rationale(
                    StructuredRationale::new(action)
                        .unwrap_or_else(|_| StructuredRationale::new("openai adapter").unwrap()),
                )
                .constitutional_ref(ConstitutionalRef("OpenAI Adapter §4.4".into()))
                .reversibility(ReversibilityStatus::Irreversible),
        )
        .unwrap_or(0)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_middleware() -> GovernanceMiddleware {
        let daemon = Arc::new(Mutex::new(GovernanceDaemon::new(10)));
        let witness = Arc::new(Mutex::new(WitnessLog::new()));
        GovernanceMiddleware::new(daemon, witness)
    }

    fn sample_request() -> ChatRequest {
        ChatRequest {
            model: "gpt-4".into(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "Hello".into(),
            }],
            metadata: serde_json::json!({}),
        }
    }

    // ── OAI-T01: Pre-gate allows clean request ────────────────────
    #[test]
    fn oai_t01_pre_gate_allow() {
        let mw = test_middleware();
        let req = sample_request();
        let result = mw.pre_gate(&req);

        assert!(result.allow);
        assert!(!result.conservative_mode);
        assert!(result.denial_reason.is_none());
        assert!(result.audit_ref.starts_with("WIT-"));
    }

    // ── OAI-T02: Post-audit logs and verifies chain ───────────────
    #[test]
    fn oai_t02_post_audit_logs() {
        let mw = test_middleware();
        // First do a pre-gate so there's at least one witness entry
        let req = sample_request();
        mw.pre_gate(&req);

        let audit = mw.post_audit("gpt-4", "Hello! How can I help?");
        assert!(audit.witness_seq > 0);
        assert!(audit.chain_valid);
    }

    // ── OAI-T03: Conservative mode blocks pre-gate ────────────────
    #[test]
    fn oai_t03_conservative_blocks() {
        let daemon = Arc::new(Mutex::new(GovernanceDaemon::new(10)));
        let witness = Arc::new(Mutex::new(WitnessLog::new()));

        // Enter conservative mode
        daemon.lock().unwrap().inference_bypass_attempt();

        let mw = GovernanceMiddleware::new(daemon, witness);
        let req = sample_request();
        let result = mw.pre_gate(&req);

        assert!(!result.allow);
        assert!(result.conservative_mode);
        assert!(result.denial_reason.as_deref() == Some("Conservative Mode active"));
    }

    // ── OAI-T04: Denylist key in metadata blocks request ─────────
    #[test]
    fn oai_t04_denylist_blocks() {
        let mw = test_middleware();
        let req = ChatRequest {
            model: "gpt-4".into(),
            messages: vec![ChatMessage { role: "user".into(), content: "Hello".into() }],
            metadata: serde_json::json!({
                "experiment_bucket": "A",
                "safe_key": "ok"
            }),
        };
        let result = mw.pre_gate(&req);

        assert!(!result.allow);
        assert!(result.denial_reason.as_deref().unwrap().contains("GROWTH.SIGNAL.INJECTION"));
    }
}
