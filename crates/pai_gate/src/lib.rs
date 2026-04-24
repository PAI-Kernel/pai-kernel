//! # PAI Gate (HAC Component #3)
//!
//! **Constitutional reference:** PAI-CD §Governance, Consent Model §P2
//!
//! Core gating logic: evaluates whether a proposed action is allowed
//! given the current governance mode (normal / conservative / breach),
//! consent tier, and confirmation status.
//!
//! # Examples
//!
//! ```
//! use pai_gate::{evaluate, GateAction, GateRequest, Mode};
//! use pai_interface::KernelContextDecisionBasis;
//! use serde_json::json;
//!
//! let req = GateRequest {
//!     action: GateAction::RankSuggestions,
//!     context: KernelContextDecisionBasis {
//!         session_id: "sess-001".into(),
//!         author_id: "author-001".into(),
//!         declared_goals: vec!["accuracy".into()],
//!         tier: 1,
//!         telemetry_safety: json!({}),
//!     },
//!     payload_hash: "abc123".into(),
//!     tier: 1,
//!     user_confirmed: false,
//!     breach: false,
//!     conservative: false,
//! };
//!
//! let resp = evaluate(&req);
//! assert!(resp.allow);
//! assert_eq!(resp.mode, Mode::Normal);
//! ```
//!
//! Breach mode blocks all actions:
//!
//! ```
//! # use pai_gate::{evaluate, GateAction, GateRequest, Mode};
//! # use pai_interface::KernelContextDecisionBasis;
//! # use serde_json::json;
//! let req = GateRequest {
//!     action: GateAction::RankSuggestions,
//!     context: KernelContextDecisionBasis {
//!         session_id: "s".into(), author_id: "a".into(),
//!         declared_goals: vec![], tier: 0,
//!         telemetry_safety: json!({}),
//!     },
//!     payload_hash: "h".into(),
//!     tier: 0,
//!     user_confirmed: false,
//!     breach: true,
//!     conservative: false,
//! };
//! let resp = evaluate(&req);
//! assert!(!resp.allow);
//! assert_eq!(resp.mode, Mode::Conservative);
//! ```

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use pai_interface::KernelContextDecisionBasis;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Conservative,
    Breach,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GateAction {
    ApplySubstantiveEdit,
    RankSuggestions,
    MemoryRead,
    MemoryWrite,
    TierEscalation,
    IrreversibleDispatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateRequest {
    pub action: GateAction,
    pub context: KernelContextDecisionBasis,
    pub payload_hash: String,
    pub tier: u8,
    pub user_confirmed: bool,
    pub breach: bool,
    pub conservative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResponse {
    pub allow: bool,
    pub mode: Mode,
    pub reasons: Vec<String>,
    pub audit_ref: String,
}

pub fn hash_payload_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn evaluate(req: &GateRequest) -> GateResponse {
    if req.breach {
        return GateResponse {
            allow: false,
            mode: Mode::Conservative,
            reasons: vec!["breach active".into()],
            audit_ref: "BREACH".into(),
        };
    }
    if req.conservative && req.tier >= 2 {
        return GateResponse {
            allow: false,
            mode: Mode::Conservative,
            reasons: vec!["conservative tier gate".into()],
            audit_ref: "CONS".into(),
        };
    }
    if req.action == GateAction::IrreversibleDispatch && !req.user_confirmed {
        return GateResponse {
            allow: false,
            mode: Mode::Normal,
            reasons: vec!["confirmation required".into()],
            audit_ref: "CONFIRM_REQUIRED".into(),
        };
    }
    GateResponse {
        allow: true,
        mode: if req.conservative {
            Mode::Conservative
        } else {
            Mode::Normal
        },
        reasons: vec![],
        audit_ref: "ALLOW".into(),
    }
}
