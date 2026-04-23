#![forbid(unsafe_code)]

use regex::RegexSet;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KernelContext {
    pub session_id: String,
    pub author_id: String,
    pub declared_goals: Vec<String>,
    pub tier: u8,
    pub telemetry_safety: Value,
    pub telemetry_behavioral: Option<Value>, // export-only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelContextDecisionBasis {
    pub session_id: String,
    pub author_id: String,
    pub declared_goals: Vec<String>,
    pub tier: u8,
    pub telemetry_safety: Value,
}

impl From<KernelContext> for KernelContextDecisionBasis {
    fn from(c: KernelContext) -> Self {
        Self {
            session_id: c.session_id,
            author_id: c.author_id,
            declared_goals: c.declared_goals,
            tier: c.tier,
            telemetry_safety: c.telemetry_safety,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceBreachClass {
    GrowthSignalInjectionAttempt,
    SchemaViolation,
}

#[derive(Debug, Clone)]
pub struct InterfaceBreach {
    pub class: InterfaceBreachClass,
    pub reasons: Vec<String>,
    pub deny_keys: Vec<String>,
}

#[derive(Debug, Error)]
pub enum InterfaceError {
    #[error("interface breach")]
    Breach(InterfaceBreach),
    #[error("schema error: {0}")]
    Schema(String),
}

fn collect_keys_recursive(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(map) => {
            for (k, vv) in map {
                out.push(k.clone());
                collect_keys_recursive(vv, out);
            }
        }
        Value::Array(arr) => {
            for vv in arr {
                collect_keys_recursive(vv, out);
            }
        }
        _ => {}
    }
}

pub fn validate_context(
    raw: &Value,
    denylist_patterns: &[String],
) -> Result<KernelContext, InterfaceError> {
    let set =
        RegexSet::new(denylist_patterns).map_err(|e| InterfaceError::Schema(e.to_string()))?;
    let mut keys = vec![];
    collect_keys_recursive(raw, &mut keys);
    let mut deny_hits: Vec<String> = keys.iter().filter(|k| set.is_match(k)).cloned().collect();
    deny_hits.sort();
    deny_hits.dedup();
    if !deny_hits.is_empty() {
        return Err(InterfaceError::Breach(InterfaceBreach {
            class: InterfaceBreachClass::GrowthSignalInjectionAttempt,
            reasons: vec!["denylist growth keys present".into()],
            deny_keys: deny_hits,
        }));
    }
    let ctx: KernelContext =
        serde_json::from_value(raw.clone()).map_err(|e| InterfaceError::Schema(e.to_string()))?;
    if ctx.tier > 4 {
        return Err(InterfaceError::Schema("tier must be 0..4".into()));
    }
    Ok(ctx)
}
