//! # PAI Policy Engine (Component #2)
//!
//! Per PHASE1-TZ-001 Section 5 — OPA/Rego integration using the `regorus`
//! crate (pure-Rust OPA evaluator, no external OPA server required).
//!
//! Constitutional rules and operational policies are `.rego` files.
//! The [`PolicyEngine`] loads them, then evaluates queries with JSON input.
//! OPA is the **decision layer**; HAC primitives are the **enforcement layer**.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Error ──────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("policy evaluation error: {0}")]
    Evaluation(String),
    #[error("policy load error: {0}")]
    Load(String),
    #[error("invalid input: {0}")]
    Input(String),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

// ── Decision ───────────────────────────────────────────────────────────

/// Result of a policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyDecision {
    pub allow: bool,
    pub deny_reasons: Vec<String>,
}

// ── Engine ─────────────────────────────────────────────────────────────

/// Pure-Rust OPA/Rego policy engine backed by `regorus`.
pub struct PolicyEngine {
    engine: regorus::Engine,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self { engine: regorus::Engine::new() }
    }

    /// Load a `.rego` policy from source.
    pub fn add_policy(&mut self, path: &str, source: &str) -> Result<(), PolicyError> {
        self.engine
            .add_policy(path.into(), source.into())
            .map_err(|e| PolicyError::Load(e.to_string()))?;
        Ok(())
    }

    /// Load all `.rego` files from a directory.
    pub fn load_dir(&mut self, dir: &std::path::Path) -> Result<usize, PolicyError> {
        let mut count = 0usize;
        let entries = std::fs::read_dir(dir)
            .map_err(|e| PolicyError::Load(format!("{}: {}", dir.display(), e)))?;
        for entry in entries {
            let entry = entry.map_err(|e| PolicyError::Load(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rego") {
                self.engine
                    .add_policy_from_file(&path)
                    .map_err(|e| PolicyError::Load(format!("{}: {}", path.display(), e)))?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Load from standard `policies/{constitutional,operational}/` tree.
    pub fn load_policy_tree(&mut self, base: &std::path::Path) -> Result<usize, PolicyError> {
        let mut total = 0;
        for sub in &["constitutional", "operational"] {
            let dir = base.join(sub);
            if dir.is_dir() { total += self.load_dir(&dir)?; }
        }
        Ok(total)
    }

    /// Set the `data` document.
    pub fn set_data_json(&mut self, json: &str) -> Result<(), PolicyError> {
        self.engine.add_data_json(json).map_err(|e| PolicyError::Input(e.to_string()))
    }

    /// Clear data document.
    pub fn clear_data(&mut self) { self.engine.clear_data(); }

    /// Evaluate a raw rule path with JSON input.
    pub fn eval_rule(&mut self, rule: &str, input_json: &str) -> Result<regorus::Value, PolicyError> {
        self.engine.set_input_json(input_json).map_err(|e| PolicyError::Input(e.to_string()))?;
        self.engine.eval_rule(rule.into()).map_err(|e| PolicyError::Evaluation(e.to_string()))
    }

    /// Evaluate a policy with `allow`/`deny` convention.
    pub fn evaluate(&mut self, package_path: &str, input_json: &str) -> Result<PolicyDecision, PolicyError> {
        self.engine.set_input_json(input_json).map_err(|e| PolicyError::Input(e.to_string()))?;

        let allow_val = self.engine.eval_rule(format!("data.{}.allow", package_path))
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        let allow = matches!(&allow_val, regorus::Value::Bool(true));

        // `deny contains msg` → Set; `deny[msg]` → Object. Handle both.
        let deny_reasons = match self.engine.eval_rule(format!("data.{}.deny", package_path)) {
            Ok(regorus::Value::Set(set)) => set.iter().filter_map(|v| match v {
                regorus::Value::String(s) => Some(s.to_string()), _ => None,
            }).collect(),
            Ok(regorus::Value::Object(map)) => map.iter().filter_map(|(k, _)| match k {
                regorus::Value::String(s) => Some(s.to_string()), _ => None,
            }).collect(),
            _ => vec![],
        };

        Ok(PolicyDecision { allow, deny_reasons })
    }

    /// Evaluate as bool.
    pub fn eval_bool(&mut self, rule: &str, input_json: &str) -> Result<bool, PolicyError> {
        Ok(matches!(self.eval_rule(rule, input_json)?, regorus::Value::Bool(true)))
    }

    /// Evaluate as string.
    pub fn eval_string(&mut self, rule: &str, input_json: &str) -> Result<String, PolicyError> {
        match self.eval_rule(rule, input_json)? {
            regorus::Value::String(s) => Ok(s.to_string()),
            other => Ok(format!("{other}")),
        }
    }
}

impl Default for PolicyEngine { fn default() -> Self { Self::new() } }

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_with_all_policies() -> PolicyEngine {
        let mut e = PolicyEngine::new();
        e.add_policy("consent.rego", include_str!("../../../policies/constitutional/consent.rego")).unwrap();
        e.add_policy("delegation.rego", include_str!("../../../policies/constitutional/delegation.rego")).unwrap();
        e.add_policy("conservative.rego", include_str!("../../../policies/constitutional/conservative.rego")).unwrap();
        e.add_policy("classification.rego", include_str!("../../../policies/constitutional/classification.rego")).unwrap();
        e.add_policy("drift.rego", include_str!("../../../policies/constitutional/drift.rego")).unwrap();
        e.add_policy("coordination.rego", include_str!("../../../policies/constitutional/coordination.rego")).unwrap();
        e.add_policy("denylist.rego", include_str!("../../../policies/operational/denylist.rego")).unwrap();
        e.add_policy("tier_gates.rego", include_str!("../../../policies/operational/tier_gates.rego")).unwrap();
        e
    }

    #[test]
    fn opa_t01_consent_allow() {
        let mut e = engine_with_all_policies();
        let d = e.evaluate("pai.constitutional.consent", r#"{
            "consent_record": { "explicit": true, "tier": 2, "revoked": false },
            "required_tier": 2
        }"#).unwrap();
        assert!(d.allow);
        assert!(d.deny_reasons.is_empty());
    }

    #[test]
    fn opa_t02_consent_no_record_deny() {
        let mut e = engine_with_all_policies();
        let d = e.evaluate("pai.constitutional.consent", r#"{ "required_tier": 2 }"#).unwrap();
        assert!(!d.allow);
        assert!(d.deny_reasons.iter().any(|r| r.contains("no consent record")), "reasons: {:?}", d.deny_reasons);
    }

    #[test]
    fn opa_t03_consent_revoked_deny() {
        let mut e = engine_with_all_policies();
        let d = e.evaluate("pai.constitutional.consent", r#"{
            "consent_record": { "explicit": true, "tier": 2, "revoked": true },
            "required_tier": 2
        }"#).unwrap();
        assert!(!d.allow);
        assert!(d.deny_reasons.iter().any(|r| r.contains("revoked")), "reasons: {:?}", d.deny_reasons);
    }

    #[test]
    fn opa_t04_consent_silence_deny() {
        let mut e = engine_with_all_policies();
        let d = e.evaluate("pai.constitutional.consent", r#"{
            "consent_record": { "explicit": false, "tier": 2, "revoked": false },
            "required_tier": 2
        }"#).unwrap();
        assert!(!d.allow);
        assert!(d.deny_reasons.iter().any(|r| r.contains("silence")), "reasons: {:?}", d.deny_reasons);
    }

    #[test]
    fn opa_t05_conservative_blocks_tier2() {
        let mut e = engine_with_all_policies();
        let d = e.evaluate("pai.constitutional.conservative",
            r#"{ "conservative_mode": true, "required_tier": 2 }"#).unwrap();
        assert!(!d.allow);
        assert!(d.deny_reasons.iter().any(|r| r.contains("Conservative Mode")), "reasons: {:?}", d.deny_reasons);

        let d2 = e.evaluate("pai.constitutional.conservative",
            r#"{ "conservative_mode": true, "required_tier": 1 }"#).unwrap();
        assert!(d2.allow, "tier 1 allowed in conservative mode");

        let d3 = e.evaluate("pai.constitutional.conservative",
            r#"{ "conservative_mode": false, "required_tier": 3 }"#).unwrap();
        assert!(d3.allow, "tier 3 allowed when not conservative");
    }

    #[test]
    fn opa_t06_denylist_growth_key() {
        let mut e = engine_with_all_policies();
        e.set_data_json(r#"{ "denylist_patterns": ["retention_score", "experiment_bucket", "time_spent"] }"#).unwrap();

        let d = e.evaluate("pai.operational.denylist",
            r#"{ "context_keys": ["session_id", "retention_score", "latency_ms"] }"#).unwrap();
        assert!(!d.allow, "denylist key must deny");

        let breach = e.eval_bool("data.pai.operational.denylist.breach",
            r#"{ "context_keys": ["retention_score"] }"#).unwrap();
        assert!(breach, "must signal breach");

        let clean = e.evaluate("pai.operational.denylist",
            r#"{ "context_keys": ["session_id", "author_id"] }"#).unwrap();
        assert!(clean.allow, "clean context passes");
    }

    #[test]
    fn opa_t07_delegation_expired_deny() {
        let mut e = engine_with_all_policies();
        let d = e.evaluate("pai.constitutional.delegation", r#"{
            "delegation": { "revoked": false, "expires_at": 1000, "scope": ["CAP.OBJECTIVE.ADD"] },
            "current_time": 2000,
            "requested_capability": "CAP.OBJECTIVE.ADD"
        }"#).unwrap();
        assert!(!d.allow);
        assert!(d.deny_reasons.iter().any(|r| r.contains("expired")), "reasons: {:?}", d.deny_reasons);

        let v = e.evaluate("pai.constitutional.delegation", r#"{
            "delegation": { "revoked": false, "expires_at": 5000, "scope": ["CAP.OBJECTIVE.ADD"] },
            "current_time": 2000,
            "requested_capability": "CAP.OBJECTIVE.ADD"
        }"#).unwrap();
        assert!(v.allow, "valid delegation must allow");
    }

    #[test]
    fn opa_t08_classification_bias_consequential() {
        let mut e = engine_with_all_policies();
        let class = e.eval_string("data.pai.constitutional.classification.classification",
            r#"{ "domain_type": "consequential", "bias_detected": true }"#).unwrap();
        assert_eq!(class, "recommendation");

        let class2 = e.eval_string("data.pai.constitutional.classification.classification",
            r#"{ "domain_type": "consequential", "bias_detected": false }"#).unwrap();
        assert_eq!(class2, "informational");

        let tier = e.eval_rule("data.pai.constitutional.classification.consent_required",
            r#"{ "domain_type": "consequential", "bias_detected": true }"#).unwrap();
        assert_eq!(tier, regorus::Value::from(2));
    }

    #[test]
    fn opa_t09_policy_hot_reload() {
        let mut e = PolicyEngine::new();
        e.add_policy("r.rego", "package pai.test.reload\ndefault allow := true").unwrap();
        let d1 = e.evaluate("pai.test.reload", r#"{}"#).unwrap();
        assert!(d1.allow);

        let mut e2 = PolicyEngine::new();
        e2.add_policy("r.rego", r#"
            package pai.test.reload
            default allow := false
            deny contains msg if { msg := "policy updated: all denied" }
        "#).unwrap();
        let d2 = e2.evaluate("pai.test.reload", r#"{}"#).unwrap();
        assert!(!d2.allow);
        assert!(d2.deny_reasons.iter().any(|r| r.contains("policy updated")), "reasons: {:?}", d2.deny_reasons);
    }

    #[test]
    fn opa_t10_all_policies_loadable() {
        let constitutional = [
            ("consent.rego", include_str!("../../../policies/constitutional/consent.rego")),
            ("delegation.rego", include_str!("../../../policies/constitutional/delegation.rego")),
            ("conservative.rego", include_str!("../../../policies/constitutional/conservative.rego")),
            ("classification.rego", include_str!("../../../policies/constitutional/classification.rego")),
            ("drift.rego", include_str!("../../../policies/constitutional/drift.rego")),
            ("coordination.rego", include_str!("../../../policies/constitutional/coordination.rego")),
        ];
        let operational = [
            ("denylist.rego", include_str!("../../../policies/operational/denylist.rego")),
            ("tier_gates.rego", include_str!("../../../policies/operational/tier_gates.rego")),
        ];
        for (name, src) in constitutional.iter().chain(operational.iter()) {
            let mut e = PolicyEngine::new();
            e.add_policy(name, src).unwrap_or_else(|err| panic!("load {}: {}", name, err));
        }
        let _e = engine_with_all_policies();
        assert_eq!(constitutional.len() + operational.len(), 8);
    }
}
