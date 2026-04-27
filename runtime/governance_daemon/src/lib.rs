//! PAI-CD Governance Daemon (Minimal Constitutional Core — MCC Upgrade)
#![forbid(unsafe_code)]

use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::OffsetDateTime;

use pai_gate::{GateAction, GateRequest, GateResponse};
use pai_influence::{InfluenceEvent, InfluenceEventType, InfluenceLog};
use pai_interface::{validate_context, KernelContextDecisionBasis};

const CAP_CAPABILITY_REGISTER: &str = "CAP.CAPABILITY.REGISTER";
const CAP_CONSENT_GRANT: &str = "CAP.CONSENT.GRANT";
const CAP_CONSENT_REVOKE: &str = "CAP.CONSENT.REVOKE";
const CAP_DELEGATION_GRANT: &str = "CAP.DELEGATION.GRANT";
const CAP_DELEGATION_REVOKE: &str = "CAP.DELEGATION.REVOKE";
const CAP_CONSERVATIVE_EXIT: &str = "CAP.CONSERVATIVE.EXIT";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BreachClass {
    ObjInjection,
    GovBypass,
    LogTamper,
    DriftOverthreshold,
    ConservativeViolation,
    AuthzFailure,
    SignatureMissing,
    SignatureInvalid,
    InvalidState,
    GrowthSignalInjectionAttempt,
}

impl BreachClass {
    pub fn code(&self) -> &'static str {
        match self {
            BreachClass::ObjInjection => "OBJ.INJECTION",
            BreachClass::GovBypass => "GOV.BYPASS",
            BreachClass::LogTamper => "LOG.TAMPER",
            BreachClass::DriftOverthreshold => "DRIFT.OVERTHRESHOLD",
            BreachClass::ConservativeViolation => "CONS.MODE.VIOLATION",
            BreachClass::AuthzFailure => "AUTHZ.FAIL",
            BreachClass::SignatureMissing => "SIG.MISSING",
            BreachClass::SignatureInvalid => "SIG.INVALID",
            BreachClass::InvalidState => "STATE.INVALID",
            BreachClass::GrowthSignalInjectionAttempt => "GROWTH.SIGNAL.INJECTION",
        }
    }
}

/// Risk tier for governance capabilities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskTier {
    Tier0 = 0,
    Tier1 = 1,
    Tier2 = 2,
    Tier3 = 3,
    Tier4 = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityDefinition {
    pub id: String,
    pub tier: RiskTier,
    pub description_hash: String,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsentEvidence {
    ManualConfirm,
    DualConfirmA(String),
    DualConfirmB(String),
    ExternalReceiptHash(String),
}

/// Expiry policy for granular consent (MP-9).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExpiryPolicy {
    /// No automatic expiry — consent persists until explicit revocation.
    NoExpiry,
    /// Standard expiry: 90 days for Tier 0-2, 30 days for Tier 3-4 (MP-9 §3).
    Standard,
    /// Custom expiry in seconds from grant time.
    Custom { duration_secs: u64 },
}

impl ExpiryPolicy {
    /// Compute the expiry timestamp from a grant time.
    pub fn expires_at(&self, granted_at: OffsetDateTime, tier: RiskTier) -> Option<OffsetDateTime> {
        match self {
            ExpiryPolicy::NoExpiry => None,
            ExpiryPolicy::Standard => {
                let days = match tier {
                    RiskTier::Tier0 | RiskTier::Tier1 | RiskTier::Tier2 => 90,
                    RiskTier::Tier3 | RiskTier::Tier4 => 30,
                };
                Some(granted_at + time::Duration::days(days))
            }
            ExpiryPolicy::Custom { duration_secs } => {
                Some(granted_at + time::Duration::seconds(*duration_secs as i64))
            }
        }
    }
}

// Default is Standard (not first variant NoExpiry) — intentional per MP-9 §3.
#[allow(clippy::derivable_impls)]
impl Default for ExpiryPolicy {
    fn default() -> Self {
        ExpiryPolicy::Standard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub consent_id: String,
    pub capability_id: String,
    pub tier: RiskTier,
    pub scope_text_hash: String,
    pub granted_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
    pub granted_by: String,
    pub evidence: ConsentEvidence,
    pub decision_seq: u64,
    /// MP-9: Capability-specific scope — which specific capabilities this consent covers.
    /// Empty vec means "all capabilities at this tier" (legacy behavior).
    #[serde(default)]
    pub capability_scope: Vec<String>,
    /// MP-9: Expiry policy for this consent grant.
    #[serde(default)]
    pub expiry_policy: ExpiryPolicy,
    /// MP-9: Grant interaction depth (number of clicks/confirmations to grant).
    /// Revocation must require <= this depth (click-depth symmetry).
    #[serde(default)]
    pub grant_depth: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationGrant {
    pub delegation_id: String,
    pub principal: String,
    pub delegate: String,
    pub scope: Vec<String>, // capability ids
    pub domain_scope: Vec<String>,
    pub expires_at: OffsetDateTime,
    pub revocable: bool,
    pub revoked_at: Option<OffsetDateTime>,
    pub issued_by_decision: u64,
}

/// Capture risk analysis report (Doc 15).
///
/// Monitors delegation concentration — flags when a single delegate
/// accumulates a disproportionate share of active delegations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRiskReport {
    /// True if any delegate exceeds the concentration threshold.
    pub capture_detected: bool,
    /// Number of distinct delegates with active (non-revoked, non-expired) delegations.
    pub delegate_count: usize,
    /// Maximum concentration percentage (delegate with most delegations / total × 100).
    pub max_concentration_pct: f64,
    /// Human-readable warnings for delegates exceeding threshold.
    pub concentration_warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActorType {
    Author,
    Governance,
    Inference,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityContext {
    pub current_actor: String,
    pub actor_type: ActorType,
    pub active_delegate: Option<String>,
}

/// Typed action names aligned with the formal model / traceability.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogAction {
    GovRegisterCapability,
    GovGrantConsent,
    GovRevokeConsent,
    GovGrantDelegation,
    GovRevokeDelegation,

    GovMutateObjective,
    GovSnapshot,
    GovRollback,

    InferenceBypassAttempt,
    RuntimeInjectObjectiveAttempt,
    LogTamperAttempt,
    ConservativeModeViolation,

    DriftTick,
    EnterConservative,
    ExitConservative,

    Noop,
    GovTier4Noop,
    GrowthSignalInjectionAttempt,
    PersistFailure,
    LogTamperDetected,
    InvalidStateDetected,
}

impl LogAction {
    pub fn as_str(&self) -> &'static str {
        use LogAction::*;
        match self {
            GovRegisterCapability => "GovRegisterCapability",
            GovGrantConsent => "GovGrantConsent",
            GovRevokeConsent => "GovRevokeConsent",
            GovGrantDelegation => "GovGrantDelegation",
            GovRevokeDelegation => "GovRevokeDelegation",
            GovMutateObjective => "GovMutateObjective",
            GovSnapshot => "GovSnapshot",
            GovRollback => "GovRollback",
            InferenceBypassAttempt => "InferenceBypassAttempt",
            RuntimeInjectObjectiveAttempt => "RuntimeInjectObjectiveAttempt",
            LogTamperAttempt => "LogTamperAttempt",
            ConservativeModeViolation => "ConservativeModeViolation",
            DriftTick => "DriftTick",
            EnterConservative => "EnterConservative",
            ExitConservative => "ExitConservative",
            Noop => "Noop",
            GovTier4Noop => "GovTier4Noop",
            GrowthSignalInjectionAttempt => "GrowthSignalInjectionAttempt",
            PersistFailure => "PersistFailure",
            LogTamperDetected => "LogTamperDetected",
            InvalidStateDetected => "InvalidStateDetected",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionKind {
    GovAction,
    Breach,
    Snapshot,
    Rollback,
    DriftTick,
}

/// Graduated governance mode (Doc 18: Graduated Response).
///
/// 5-state machine replacing the binary Normal/Conservative model:
/// ```text
/// Normal → Warning → Restricted → Conservative → Breach
///    ↑←←←←←←←←←←←←←←←←←←←←←←←←←←←←←↓ (rollback)
/// ```
///
/// Transitions:
/// - Normal → Warning: drift approaching threshold (75%)
/// - Warning → Restricted: drift at threshold (100%), or repeated warnings
/// - Restricted → Conservative: breach detected, or drift far exceeded
/// - Conservative → Breach: active breach class set
/// - Any → Normal: rollback/exit_conservative (with appropriate authorization)
///
/// Backward compatibility: `conservative()` returns true for Restricted, Conservative, Breach.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum GovernanceMode {
    /// Full capability, no restrictions.
    #[default]
    Normal,
    /// Advisory: drift approaching threshold. Tier 3-4 actions get advisory warnings.
    Warning,
    /// Tier >= 2 actions blocked except ExitConservative. Delegations paused.
    Restricted,
    /// Full conservative mode — legacy behavior. All high-tier blocked.
    Conservative,
    /// Active breach — only rollback and export allowed.
    Breach,
}

impl GovernanceMode {
    /// Returns true if this mode is at least as restrictive as Conservative.
    /// Backward-compatible with the old `conservative` bool.
    pub fn is_conservative(&self) -> bool {
        matches!(
            self,
            GovernanceMode::Restricted | GovernanceMode::Conservative | GovernanceMode::Breach
        )
    }

    /// Returns true if this is a breach state.
    pub fn is_breach(&self) -> bool {
        matches!(self, GovernanceMode::Breach)
    }

    /// Returns true if this mode issues advisory warnings.
    pub fn is_warning_or_above(&self) -> bool {
        *self >= GovernanceMode::Warning
    }
}

/// MCC canonical state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceState {
    objectives: Vec<String>,
    objective_registry: Vec<String>,
    drift: u64,
    conservative: bool,
    drift_threshold: u64,
    breach_flag: Option<BreachClass>,
    /// Doc 18: Graduated governance mode (5-state).
    #[serde(default)]
    governance_mode: GovernanceMode,

    capability_registry: Vec<CapabilityDefinition>,
    consent_ledger: Vec<ConsentRecord>,
    delegations: Vec<DelegationGrant>,
    authority_context: AuthorityContext,
}

impl GovernanceState {
    pub fn objectives(&self) -> &[String] {
        &self.objectives
    }
    pub fn objective_registry(&self) -> &[String] {
        &self.objective_registry
    }
    pub fn drift(&self) -> u64 {
        self.drift
    }
    pub fn conservative(&self) -> bool {
        self.conservative
    }
    /// Doc 18: Current graduated governance mode.
    pub fn governance_mode(&self) -> GovernanceMode {
        self.governance_mode
    }
    pub fn drift_threshold(&self) -> u64 {
        self.drift_threshold
    }
    pub fn breach_flag(&self) -> Option<BreachClass> {
        self.breach_flag
    }

    pub fn capability_registry(&self) -> &[CapabilityDefinition] {
        &self.capability_registry
    }
    pub fn consent_ledger(&self) -> &[ConsentRecord] {
        &self.consent_ledger
    }
    pub fn delegations(&self) -> &[DelegationGrant] {
        &self.delegations
    }
    pub fn authority_context(&self) -> &AuthorityContext {
        &self.authority_context
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    seq: u64,
    ts: OffsetDateTime,
    kind: DecisionKind,
    action: LogAction,
    details: String,
    breach: Option<BreachClass>,
    capability_id: String,
    tier: RiskTier,
    params: serde_json::Value,
    prev_hash: String,
    hash: String,
    signature: Option<String>,         // base64(ed25519)
    signing_pubkey_id: Option<String>, // identifier for key
}

impl DecisionEntry {
    pub fn seq(&self) -> u64 {
        self.seq
    }
    pub fn ts(&self) -> OffsetDateTime {
        self.ts
    }
    pub fn kind(&self) -> DecisionKind {
        self.kind
    }
    pub fn action(&self) -> LogAction {
        self.action
    }
    pub fn details(&self) -> &str {
        &self.details
    }
    pub fn breach(&self) -> Option<BreachClass> {
        self.breach
    }
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }
    pub fn tier(&self) -> RiskTier {
        self.tier
    }
    pub fn params(&self) -> &serde_json::Value {
        &self.params
    }
    pub fn prev_hash(&self) -> &str {
        &self.prev_hash
    }
    pub fn hash(&self) -> &str {
        &self.hash
    }
    pub fn signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }
    pub fn signing_pubkey_id(&self) -> Option<&str> {
        self.signing_pubkey_id.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    objectives: Vec<String>,
    objective_registry: Vec<String>,
    drift: u64,
    conservative: bool,
    breach_flag: Option<BreachClass>,
    #[serde(default)]
    governance_mode: GovernanceMode,

    capability_registry: Vec<CapabilityDefinition>,
    consent_ledger: Vec<ConsentRecord>,
    delegations: Vec<DelegationGrant>,
    authority_context: AuthorityContext,
}

#[derive(Debug, Error)]
pub enum GovError {
    #[error("governance operation blocked in conservative mode")]
    ConservativeMode,
    #[error("unauthorized operation")]
    Unauthorized,
    #[error("decision log tampered or unauthenticated")]
    LogTampered,
    #[error("invalid state")]
    InvalidState,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(String),
    #[error("crypto error")]
    Crypto,
}

#[derive(Debug, Clone)]
pub struct ActionRequest {
    pub action: LogAction,
    pub capability_id: String,
    pub tier: RiskTier,
    pub evidence_refs: Vec<String>,
    pub actor_context: AuthorityContext,
    pub details: String,
    pub params: serde_json::Value,
}

/// Canonical payload used for BOTH hash-chain and signatures.
#[derive(Debug, Clone, Serialize)]
struct CanonicalPayload<'a> {
    prev_hash: &'a str,
    seq: u64,
    ts_unix: i64,
    kind: DecisionKind,
    action: LogAction,
    details: &'a str,
    breach: &'a Option<BreachClass>,
    capability_id: &'a str,
    tier: RiskTier,
    params: &'a serde_json::Value,
}

#[allow(clippy::too_many_arguments)]
fn canonical_payload_bytes(
    prev_hash: &str,
    seq: u64,
    ts: OffsetDateTime,
    kind: DecisionKind,
    action: LogAction,
    details: &str,
    breach: &Option<BreachClass>,
    capability_id: &str,
    tier: RiskTier,
    params: &serde_json::Value,
) -> Vec<u8> {
    let p = CanonicalPayload {
        prev_hash,
        seq,
        ts_unix: ts.unix_timestamp(),
        kind,
        action,
        details,
        breach,
        capability_id,
        tier,
        params,
    };
    serde_json::to_vec(&p).expect("canonical payload serialize")
}

#[allow(dead_code)]
fn denylist_hits(raw: &serde_json::Value, patterns: &[String]) -> Vec<String> {
    let set = regex::RegexSet::new(patterns).unwrap_or_else(|_| regex::RegexSet::empty());
    fn collect(v: &serde_json::Value, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(map) => {
                for (k, vv) in map {
                    out.push(k.clone());
                    collect(vv, out);
                }
            }
            serde_json::Value::Array(arr) => {
                for vv in arr {
                    collect(vv, out);
                }
            }
            _ => {}
        }
    }
    let mut keys = vec![];
    collect(raw, &mut keys);
    let mut hits: Vec<String> = keys.into_iter().filter(|k| set.is_match(k)).collect();
    hits.sort();
    hits.dedup();
    hits
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

fn default_denylist_patterns() -> Vec<String> {
    // Source of truth: spec/denylist_growth_keys.txt (one key per line).
    // We compile exact-match regex patterns: ^key$
    include_str!("../spec/denylist_growth_keys.txt")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|k| format!("^{}$", regex::escape(k.trim())))
        .collect()
}

fn is_high_impact(action: LogAction, tier: RiskTier) -> bool {
    use LogAction::*;
    matches!(
        action,
        ExitConservative
            | GovMutateObjective
            | GovGrantDelegation
            | GovRevokeDelegation
            | GovGrantConsent
            | GovRevokeConsent
            | GovRegisterCapability
    ) && tier >= RiskTier::Tier2
}

pub struct GovernanceDaemon {
    state: GovernanceState,
    log: Vec<DecisionEntry>,
    snapshots: Vec<Snapshot>,
    mono_counter: u64,
    storage_path: Option<std::path::PathBuf>,
    log_verified_ok: bool,
    // MCC signing keys
    author_pubkey_id: String,
    author_pubkey: Option<VerifyingKey>,
    author_seckey: Option<SigningKey>,
    influence_log: InfluenceLog,
    denylist_patterns: Vec<String>,
    gate_open: bool,
}

fn seed_builtin_capabilities(state: &mut GovernanceState) {
    let builtins = vec![
        CapabilityDefinition {
            id: CAP_CAPABILITY_REGISTER.into(),
            tier: RiskTier::Tier2,
            description_hash: "builtin.register".into(),
            version: 1,
        },
        CapabilityDefinition {
            id: CAP_CONSENT_GRANT.into(),
            tier: RiskTier::Tier2,
            description_hash: "builtin.consent.grant".into(),
            version: 1,
        },
        CapabilityDefinition {
            id: CAP_CONSENT_REVOKE.into(),
            tier: RiskTier::Tier2,
            description_hash: "builtin.consent.revoke".into(),
            version: 1,
        },
        CapabilityDefinition {
            id: CAP_DELEGATION_GRANT.into(),
            tier: RiskTier::Tier2,
            description_hash: "builtin.delegation.grant".into(),
            version: 1,
        },
        CapabilityDefinition {
            id: CAP_DELEGATION_REVOKE.into(),
            tier: RiskTier::Tier2,
            description_hash: "builtin.delegation.revoke".into(),
            version: 1,
        },
        CapabilityDefinition {
            id: CAP_CONSERVATIVE_EXIT.into(),
            tier: RiskTier::Tier2,
            description_hash: "builtin.conservative.exit".into(),
            version: 1,
        },
    ];
    for c in builtins {
        if !state.capability_registry.iter().any(|x| x.id == c.id) {
            state.capability_registry.push(c);
        }
    }
}

impl GovernanceDaemon {
    pub fn new(drift_threshold: u64) -> Self {
        let mut state = GovernanceState {
            objectives: vec![],
            objective_registry: vec![],
            drift: 0,
            conservative: false,
            drift_threshold,
            breach_flag: None,
            governance_mode: GovernanceMode::Normal,

            capability_registry: vec![],
            consent_ledger: vec![],
            delegations: vec![],
            authority_context: AuthorityContext {
                current_actor: "AUTHOR".into(),
                actor_type: ActorType::Author,
                active_delegate: None,
            },
        };

        seed_builtin_capabilities(&mut state);

        Self {
            state,
            log: vec![],
            snapshots: vec![],
            mono_counter: 0,
            storage_path: None,
            log_verified_ok: true,
            author_pubkey_id: "AUTHOR_KEY".into(),
            author_pubkey: None,
            author_seckey: None,
            influence_log: InfluenceLog::default(),
            denylist_patterns: default_denylist_patterns(),
            gate_open: false,
        }
    }

    /// Configure author keys (recommended before performing high-impact actions).
    pub fn with_author_keys(
        mut self,
        pubkey_id: &str,
        pubkey: VerifyingKey,
        seckey: Option<SigningKey>,
    ) -> Self {
        self.author_pubkey_id = pubkey_id.into();
        self.author_pubkey = Some(pubkey);
        self.author_seckey = seckey;
        self
    }

    pub fn with_storage_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.storage_path = Some(path.into());
        self
    }

    pub fn recover(
        drift_threshold: u64,
        path: impl Into<std::path::PathBuf>,
    ) -> Result<Self, GovError> {
        use std::io::BufRead;
        let path = path.into();
        let mut d = GovernanceDaemon::new(drift_threshold).with_storage_path(path.clone());
        if !path.exists() {
            return Ok(d);
        }
        let f = std::fs::File::open(&path)?;
        for line in std::io::BufReader::new(f).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: DecisionEntry =
                serde_json::from_str(&line).map_err(|e| GovError::Serde(e.to_string()))?;
            d.log.push(entry);
        }
        d.verify_log()?;
        if let Some(last) = d.log.last() {
            d.mono_counter = last.seq;
        }
        d.validate_state_invariants()
            .map_err(|_| GovError::InvalidState)?;
        Ok(d)
    }

    pub fn state(&self) -> &GovernanceState {
        &self.state
    }

    pub fn denylist_patterns(&self) -> Vec<String> {
        self.denylist_patterns.clone()
    }
    pub fn log(&self) -> &[DecisionEntry] {
        &self.log
    }
    pub fn snapshots(&self) -> &[Snapshot] {
        &self.snapshots
    }

    fn validate_state_invariants(&mut self) -> Result<(), BreachClass> {
        // I6 objectives subset of objective_registry
        for o in &self.state.objectives {
            if !self.state.objective_registry.contains(o) {
                self.state.conservative = true;
                self.state.breach_flag = Some(BreachClass::InvalidState);
                self.append_breach(
                    LogAction::InvalidStateDetected,
                    "invalid state: objectives not subset of objective_registry",
                    BreachClass::InvalidState,
                    "CAP.STATE.VALIDATE",
                    RiskTier::Tier4,
                    serde_json::json!({}),
                );
                return Err(BreachClass::InvalidState);
            }
        }
        Ok(())
    }

    fn persist_append(&self, entry: &DecisionEntry) -> Result<(), GovError> {
        if let Some(path) = &self.storage_path {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            let line = serde_json::to_string(entry).map_err(|e| GovError::Serde(e.to_string()))?;
            f.write_all(line.as_bytes())?;
            f.write_all(b"\n")?;
            f.sync_all()?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn append_entry(
        &mut self,
        kind: DecisionKind,
        action: LogAction,
        details: &str,
        breach: Option<BreachClass>,
        capability_id: &str,
        tier: RiskTier,
        params: serde_json::Value,
    ) -> Result<(), GovError> {
        self.mono_counter = self.mono_counter.saturating_add(1);
        let seq = self.mono_counter;
        let ts = OffsetDateTime::now_utc();
        let prev_hash = self
            .log
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| "GENESIS".into());

        let payload = canonical_payload_bytes(
            &prev_hash,
            seq,
            ts,
            kind,
            action,
            details,
            &breach,
            capability_id,
            tier,
            &params,
        );
        let hash = sha256_hex(&payload);

        let mut signature: Option<String> = None;
        let mut signing_pubkey_id: Option<String> = None;

        if is_high_impact(action, tier) {
            let sk = self.author_seckey.as_ref().ok_or(GovError::Crypto)?;
            let sig: Signature = sk.sign(&payload);
            signature = Some(general_purpose::STANDARD.encode(sig.to_bytes()));
            signing_pubkey_id = Some(self.author_pubkey_id.clone());
        }

        let entry = DecisionEntry {
            seq,
            ts,
            kind,
            action,
            details: details.into(),
            breach,
            capability_id: capability_id.into(),
            tier,
            params,
            prev_hash,
            hash,
            signature,
            signing_pubkey_id,
        };

        self.persist_append(&entry)?;
        self.log.push(entry);
        self.log_verified_ok = true;
        Ok(())
    }

    fn append_breach(
        &mut self,
        action: LogAction,
        details: &str,
        bc: BreachClass,
        capability_id: &str,
        tier: RiskTier,
        params: serde_json::Value,
    ) {
        let _ = self.append_entry(
            DecisionKind::Breach,
            action,
            details,
            Some(bc),
            capability_id,
            tier,
            params,
        );
    }

    pub fn verify_log(&mut self) -> Result<(), GovError> {
        let mut prev = "GENESIS".to_string();
        let mut prev_seq = 0u64;

        for (i, entry) in self.log.iter().enumerate() {
            if i > 0 && entry.seq <= prev_seq {
                self.log_verified_ok = false;
                return Err(GovError::LogTampered);
            }
            prev_seq = entry.seq;

            let expected_payload = canonical_payload_bytes(
                &prev,
                entry.seq,
                entry.ts,
                entry.kind,
                entry.action,
                entry.details.as_str(),
                &entry.breach,
                entry.capability_id.as_str(),
                entry.tier,
                &entry.params,
            );
            let expected_hash = sha256_hex(&expected_payload);

            if entry.prev_hash != prev || entry.hash != expected_hash {
                self.log_verified_ok = false;
                return Err(GovError::LogTampered);
            }

            if is_high_impact(entry.action, entry.tier) {
                let sig_b64 = entry.signature.as_deref().ok_or(GovError::LogTampered)?;
                let sig_bytes = general_purpose::STANDARD
                    .decode(sig_b64)
                    .map_err(|_| GovError::LogTampered)?;
                let sig_arr: [u8; 64] = sig_bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| GovError::LogTampered)?;
                let sig = Signature::from_bytes(&sig_arr);
                let vk = self.author_pubkey.ok_or(GovError::LogTampered)?;
                vk.verify(&expected_payload, &sig)
                    .map_err(|_| GovError::LogTampered)?;
            }

            prev = entry.hash.clone();
        }

        self.log_verified_ok = true;
        Ok(())
    }

    fn verify_or_breach(&mut self) {
        if self.log_verified_ok {
            return;
        }
        if self.verify_log().is_err() {
            self.state.conservative = true;
            self.state.breach_flag = Some(BreachClass::LogTamper);
            self.append_breach(
                LogAction::LogTamperDetected,
                "log tamper detected",
                BreachClass::LogTamper,
                "CAP.LOG.VERIFY",
                RiskTier::Tier4,
                serde_json::json!({}),
            );
        }
    }

    // === MCC registry APIs ===

    pub fn register_capability(&mut self, def: CapabilityDefinition) -> Result<(), GovError> {
        self.require_gate()?;
        let tier = def.tier.max(RiskTier::Tier2);
        let details = format!("register capability {}", def.id);
        let req = ActionRequest {
            action: LogAction::GovRegisterCapability,
            capability_id: CAP_CAPABILITY_REGISTER.into(),
            tier,
            evidence_refs: vec![],
            actor_context: self.state.authority_context.clone(),
            details,
            params: serde_json::to_value(def).map_err(|e| GovError::Serde(e.to_string()))?,
        };
        self.validate_and_apply(req, |s, v| {
            let def: CapabilityDefinition =
                serde_json::from_value(v.clone()).map_err(|e| GovError::Serde(e.to_string()))?;
            s.capability_registry.retain(|c| c.id != def.id);
            s.capability_registry.push(def);
            Ok(())
        })
    }

    pub fn grant_consent(&mut self, record: ConsentRecord) -> Result<(), GovError> {
        self.require_gate()?;
        let details = format!("grant consent {}", record.consent_id);
        let req = ActionRequest {
            action: LogAction::GovGrantConsent,
            capability_id: CAP_CONSENT_GRANT.into(),
            tier: record.tier,
            evidence_refs: vec![],
            actor_context: self.state.authority_context.clone(),
            details,
            params: serde_json::to_value(record).map_err(|e| GovError::Serde(e.to_string()))?,
        };
        self.validate_and_apply(req, |s, v| {
            let mut r: ConsentRecord =
                serde_json::from_value(v.clone()).map_err(|e| GovError::Serde(e.to_string()))?;
            r.revoked_at = None;
            s.consent_ledger.push(r);
            Ok(())
        })
    }

    pub fn revoke_consent(&mut self, consent_id: &str) -> Result<(), GovError> {
        self.require_gate()?;
        let details = format!("revoke consent {consent_id}");
        let req = ActionRequest {
            action: LogAction::GovRevokeConsent,
            capability_id: CAP_CONSENT_REVOKE.into(),
            tier: RiskTier::Tier2,
            evidence_refs: vec![],
            actor_context: self.state.authority_context.clone(),
            details,
            params: serde_json::json!({ "consent_id": consent_id }),
        };
        self.validate_and_apply(req, |s, v| {
            let cid = v
                .get("consent_id")
                .and_then(|x| x.as_str())
                .ok_or(GovError::Serde("missing consent_id".into()))?;
            for r in &mut s.consent_ledger {
                if r.consent_id == cid && r.revoked_at.is_none() {
                    r.revoked_at = Some(OffsetDateTime::now_utc());
                }
            }
            Ok(())
        })
    }

    pub fn grant_delegation(&mut self, grant: DelegationGrant) -> Result<(), GovError> {
        self.require_gate()?;
        let details = format!("grant delegation {}", grant.delegation_id);
        let req = ActionRequest {
            action: LogAction::GovGrantDelegation,
            capability_id: CAP_DELEGATION_GRANT.into(),
            tier: RiskTier::Tier2,
            evidence_refs: vec![],
            actor_context: self.state.authority_context.clone(),
            details,
            params: serde_json::to_value(grant).map_err(|e| GovError::Serde(e.to_string()))?,
        };
        self.validate_and_apply(req, |s, v| {
            let mut g: DelegationGrant =
                serde_json::from_value(v.clone()).map_err(|e| GovError::Serde(e.to_string()))?;
            g.revoked_at = None;
            s.delegations.push(g);
            Ok(())
        })
    }

    pub fn revoke_delegation(&mut self, delegation_id: &str) -> Result<(), GovError> {
        self.require_gate()?;
        let details = format!("revoke delegation {delegation_id}");
        let req = ActionRequest {
            action: LogAction::GovRevokeDelegation,
            capability_id: CAP_DELEGATION_REVOKE.into(),
            tier: RiskTier::Tier2,
            evidence_refs: vec![],
            actor_context: self.state.authority_context.clone(),
            details,
            params: serde_json::json!({ "delegation_id": delegation_id }),
        };
        self.validate_and_apply(req, |s, v| {
            let did = v
                .get("delegation_id")
                .and_then(|x| x.as_str())
                .ok_or(GovError::Serde("missing delegation_id".into()))?;
            for d in &mut s.delegations {
                if d.delegation_id == did && d.revoked_at.is_none() && d.revocable {
                    d.revoked_at = Some(OffsetDateTime::now_utc());
                }
            }
            Ok(())
        })
    }

    /// Analyse active delegations for capture risk (Doc 15).
    ///
    /// A delegate holding ≥ 3 active delegations, or holding > 60 % of all
    /// active delegations, triggers a concentration warning.
    pub fn capture_risk_report(&self) -> CaptureRiskReport {
        use std::collections::HashMap;

        let now = OffsetDateTime::now_utc();

        // Collect active (non-revoked, non-expired) delegations per delegate.
        let mut counts: HashMap<&str, usize> = HashMap::new();
        let mut total_active: usize = 0;

        for d in &self.state.delegations {
            if d.revoked_at.is_some() {
                continue;
            }
            if d.expires_at < now {
                continue;
            }
            *counts.entry(d.delegate.as_str()).or_insert(0) += 1;
            total_active += 1;
        }

        let delegate_count = counts.len();

        // Compute max concentration.
        let max_count = counts.values().copied().max().unwrap_or(0);
        let max_concentration_pct = if total_active > 0 {
            (max_count as f64 / total_active as f64) * 100.0
        } else {
            0.0
        };

        // Threshold: ≥ 3 delegations AND > 60% concentration triggers warning.
        let concentration_threshold = 3usize;
        let pct_threshold = 60.0f64;

        let mut concentration_warnings = Vec::new();
        for (delegate, &count) in &counts {
            let pct = if total_active > 0 {
                (count as f64 / total_active as f64) * 100.0
            } else {
                0.0
            };
            if count >= concentration_threshold && pct > pct_threshold {
                concentration_warnings.push(format!(
                    "CAPTURE WARNING: '{delegate}' holds {count}/{total_active} active delegations ({pct:.1}%)"
                ));
            }
        }

        let capture_detected = !concentration_warnings.is_empty();

        CaptureRiskReport {
            capture_detected,
            delegate_count,
            max_concentration_pct,
            concentration_warnings,
        }
    }

    fn resolve_capability(&self, capability_id: &str) -> Option<CapabilityDefinition> {
        self.state
            .capability_registry
            .iter()
            .find(|c| c.id == capability_id)
            .cloned()
    }

    /// Predicate: is this consent record active for the given capability and tier
    /// at time `now`? Active = not revoked, not expired, capability matches,
    /// tier covers requested tier, capability_scope covers requested capability.
    fn is_active_consent(
        c: &ConsentRecord,
        now: OffsetDateTime,
        capability_id: &str,
        tier: RiskTier,
    ) -> bool {
        if c.revoked_at.is_some() {
            return false;
        }
        if c.capability_id != capability_id {
            return false;
        }
        if c.tier < tier {
            return false;
        }
        if let Some(expiry) = c.expiry_policy.expires_at(c.granted_at, c.tier) {
            if now >= expiry {
                return false;
            }
        }
        if !c.capability_scope.is_empty()
            && !c.capability_scope.iter().any(|s| s == capability_id)
        {
            return false;
        }
        true
    }

    /// Predicate: is this delegation grant active for the given capability at
    /// time `now`? Active = not revoked, not expired, scope covers capability.
    fn is_active_delegation(
        d: &DelegationGrant,
        now: OffsetDateTime,
        capability_id: &str,
    ) -> bool {
        d.revoked_at.is_none()
            && d.expires_at > now
            && d.scope.iter().any(|s| s == capability_id)
    }

    /// Built-in governance management capabilities. Author has direct
    /// constitutional authority over these (Constitutional Document § Principle 1
    /// — Authorship Supremacy · bootstrap necessity for the consent/delegation
    /// machinery itself). Non-Author actors still require active delegation.
    fn is_builtin_management_capability(capability_id: &str) -> bool {
        matches!(
            capability_id,
            CAP_CAPABILITY_REGISTER
                | CAP_CONSENT_GRANT
                | CAP_CONSENT_REVOKE
                | CAP_DELEGATION_GRANT
                | CAP_DELEGATION_REVOKE
                | CAP_CONSERVATIVE_EXIT
        )
    }

    fn has_active_consent(&self, capability_id: &str, tier: RiskTier) -> bool {
        let now = OffsetDateTime::now_utc();
        let active: Vec<&ConsentRecord> = self
            .state
            .consent_ledger
            .iter()
            .filter(|r| Self::is_active_consent(r, now, capability_id, tier))
            .collect();

        if tier < RiskTier::Tier2 {
            return true;
        }
        if tier < RiskTier::Tier4 {
            return !active.is_empty();
        }
        // Tier4: dual confirm requires evidence A and B both present in active records.
        let has_a = active
            .iter()
            .any(|r| matches!(r.evidence, ConsentEvidence::DualConfirmA(_)));
        let has_b = active
            .iter()
            .any(|r| matches!(r.evidence, ConsentEvidence::DualConfirmB(_)));
        has_a && has_b
    }

    fn is_delegation_active(&self, delegate: &str, capability_id: &str) -> bool {
        if self.state.conservative {
            return false; // I4: paused
        }
        let now = OffsetDateTime::now_utc();
        self.state.delegations.iter().any(|d| {
            d.delegate == delegate && Self::is_active_delegation(d, now, capability_id)
        })
    }

    fn validate_authorization(&mut self, req: &ActionRequest) -> Result<(), GovError> {
        // I3: capability must be registered for Tier>=2 (fail-closed)
        if req.action != LogAction::GovRegisterCapability
            && req.tier >= RiskTier::Tier2
            && self.resolve_capability(&req.capability_id).is_none()
        {
            self.state.conservative = true;
            self.state.breach_flag = Some(BreachClass::AuthzFailure);
            self.append_breach(
                LogAction::ConservativeModeViolation,
                "unregistered capability",
                BreachClass::AuthzFailure,
                &req.capability_id,
                RiskTier::Tier4,
                serde_json::json!({}),
            );
            return Err(GovError::Unauthorized);
        }

        // I4: conservative blocks Tier>=2 by default
        // MCC-02: ExitConservative is the only Tier>=2 action allowed to transition *out* of conservative mode.
        if self.state.conservative
            && req.tier >= RiskTier::Tier2
            && req.action != LogAction::ExitConservative
        {
            self.state.breach_flag = Some(BreachClass::ConservativeViolation);
            self.append_breach(
                LogAction::ConservativeModeViolation,
                "blocked in conservative mode",
                BreachClass::ConservativeViolation,
                &req.capability_id,
                req.tier,
                serde_json::json!({}),
            );
            return Err(GovError::ConservativeMode);
        }

        // I1/I2/I3: Tier>=2 authorization (REQ-236 / DL-369).
        //
        // Self-binding semantics per Constitutional Document § Principle 1
        // (Authorship Supremacy · «Delegation must be scoped, time-bound, and
        // revocable») and Consent and Capability Model § Principle 2 (Tier 2 ·
        // «Be revocable» · «Be logged in Decision Log»):
        //
        //   - For built-in governance management capabilities (CAP.CAPABILITY.REGISTER,
        //     CAP.CONSENT.*, CAP.DELEGATION.*, CAP.CONSERVATIVE.EXIT) Author has
        //     direct constitutional authority — bootstrap necessity for the consent/
        //     delegation machinery itself. Non-Author actors still require active
        //     delegation.
        //
        //   - For all other Tier ≥ 2 capabilities active consent for the capability
        //     is required (binding for all actors including Author — self-binding
        //     per Bill of Authorial Rights § Right 1). Non-Author actors additionally
        //     require active delegation scoping the capability to them.
        //
        // Failure mode: lifecycle revocation/expiry returns Err(Unauthorized) per
        // p0-3 BT-6 AUTHZ.FAIL canonical mapping. Conservative-mode shift is
        // reserved for breach detection (unregistered capability · log tamper ·
        // bypass attempt · injection); normal lifecycle authz failure does not
        // shift state to conservative.
        if req.tier >= RiskTier::Tier2 {
            let actor = &req.actor_context.current_actor;
            let actor_type = req.actor_context.actor_type;
            let cap_id = req.capability_id.as_str();

            let bootstrap = actor_type == ActorType::Author
                && Self::is_builtin_management_capability(cap_id);

            let consent_active = if bootstrap {
                true
            } else {
                self.has_active_consent(cap_id, req.tier)
            };

            let delegation_active = match actor_type {
                ActorType::Author => true,
                _ => self.is_delegation_active(actor, cap_id),
            };

            if !consent_active || !delegation_active {
                self.state.breach_flag = Some(BreachClass::AuthzFailure);
                self.append_breach(
                    LogAction::ConservativeModeViolation,
                    "authz failed (no active consent/delegation)",
                    BreachClass::AuthzFailure,
                    cap_id,
                    req.tier,
                    serde_json::json!({}),
                );
                return Err(GovError::Unauthorized);
            }
        }

        // I3 Tier4 dual confirm
        if req.tier == RiskTier::Tier4
            && req.actor_context.actor_type != ActorType::Author
            && !self.has_active_consent(&req.capability_id, req.tier)
        {
            self.state.conservative = true;
            self.state.breach_flag = Some(BreachClass::AuthzFailure);
            self.append_breach(
                LogAction::ConservativeModeViolation,
                "tier4 requires dual confirm",
                BreachClass::AuthzFailure,
                &req.capability_id,
                req.tier,
                serde_json::json!({}),
            );
            return Err(GovError::Unauthorized);
        }

        Ok(())
    }

    fn validate_and_apply<F>(&mut self, req: ActionRequest, mut apply: F) -> Result<(), GovError>
    where
        F: FnMut(&mut GovernanceState, &serde_json::Value) -> Result<(), GovError>,
    {
        self.verify_or_breach();

        // Fail-closed: require signing key for any high-impact operation.
        if is_high_impact(req.action, req.tier) && self.author_seckey.is_none() {
            self.state.conservative = true;
            self.state.breach_flag = Some(BreachClass::SignatureMissing);
            self.append_breach(
                LogAction::ConservativeModeViolation,
                "missing signing key for high-impact action",
                BreachClass::SignatureMissing,
                &req.capability_id,
                req.tier,
                serde_json::json!({}),
            );
            return Err(GovError::Crypto);
        }

        self.validate_authorization(&req)?;

        // apply mutation
        apply(&mut self.state, &req.params)?;

        // enforce invariants (I6 etc.)
        self.validate_state_invariants()
            .map_err(|_| GovError::InvalidState)?;

        // log success
        self.append_entry(
            DecisionKind::GovAction,
            req.action,
            &req.details,
            None,
            &req.capability_id,
            req.tier,
            req.params,
        )?;

        Ok(())
    }

    /// Mandatory Constitution Gate entrypoint for high-impact operations.
    pub fn execute_high_impact(
        &mut self,
        gate_req: GateRequest,
        raw_full_context: serde_json::Value,
    ) -> Result<GateResponse, GovError> {
        self.verify_or_breach();

        let raw = raw_full_context;
        {
            match validate_context(&raw, &self.denylist_patterns) {
                Ok(ctx) => {
                    let _ = self.influence_log.append(InfluenceEvent {
                        session_id: ctx.session_id.clone(),
                        timestamp: OffsetDateTime::now_utc().unix_timestamp(),
                        event_type: InfluenceEventType::GoalDeclared,
                        affected_span_hash: None,
                        source: "user".into(),
                        requires_confirmation: false,
                        confirmed: None,
                    });
                    let _decision: KernelContextDecisionBasis = ctx.into();
                }
                Err(_) => {
                    self.state.conservative = true;
                    self.state.breach_flag = Some(BreachClass::GrowthSignalInjectionAttempt);
                    self.append_breach(
                        LogAction::GrowthSignalInjectionAttempt,
                        "growth signal injection attempt in KernelContext",
                        BreachClass::GrowthSignalInjectionAttempt,
                        "CAP.GROWTH.DENYLIST",
                        RiskTier::Tier4,
                        serde_json::json!({}),
                    );
                    return Ok(GateResponse {
                        allow: false,
                        mode: pai_gate::Mode::Conservative,
                        reasons: vec!["denylist growth keys detected".into()],
                        audit_ref: "GROWTH.DENY".into(),
                    });
                }
            }
        }

        let resp = pai_gate::evaluate(&gate_req);
        if !resp.allow {
            self.state.conservative = true;
            if self.state.breach_flag.is_none() {
                self.state.breach_flag = Some(BreachClass::ConservativeViolation);
            }
            self.append_breach(
            LogAction::ConservativeModeViolation,
            "gate denied high-impact action",
            self.state.breach_flag.unwrap_or(BreachClass::ConservativeViolation),
            "CAP.GATE.DENY",
            RiskTier::Tier2,
            serde_json::json!({"action": format!("{:?}", gate_req.action), "gate_audit_ref": resp.audit_ref}),
        );
            return Ok(resp);
        }

        self.gate_open = true;

        match gate_req.action {
            GateAction::ApplySubstantiveEdit => {
                let _ = self.append_entry(
                DecisionKind::GovAction,
                LogAction::GovTier4Noop,
                "ApplySubstantiveEdit (gated)",
                None,
                "CAP.GATE.EDIT",
                RiskTier::Tier2,
                serde_json::json!({"payload_hash": gate_req.payload_hash, "gate_audit_ref": resp.audit_ref}),
            );
            }
            GateAction::IrreversibleDispatch => {
                let _ = self.append_entry(
                DecisionKind::GovAction,
                LogAction::GovTier4Noop,
                "IrreversibleDispatch (gated)",
                None,
                "CAP.GATE.DISPATCH",
                RiskTier::Tier4,
                serde_json::json!({"payload_hash": gate_req.payload_hash, "gate_audit_ref": resp.audit_ref}),
            );
            }
            _ => {
                let _ = self.append_entry(
                DecisionKind::GovAction,
                LogAction::GovTier4Noop,
                "GateAction (gated)",
                None,
                "CAP.GATE.OTHER",
                RiskTier::Tier2,
                serde_json::json!({"action": format!("{:?}", gate_req.action), "gate_audit_ref": resp.audit_ref}),
            );
            }
        }

        self.gate_open = false;
        Ok(resp)
    }

    fn require_gate(&mut self) -> Result<(), GovError> {
        if !self.gate_open {
            return Err(GovError::Unauthorized);
        }
        Ok(())
    }

    /// Export audit bundle (partial §13).
    pub fn export_bundle(&self, path: impl AsRef<std::path::Path>) -> Result<(), GovError> {
        let dir = path.as_ref();
        std::fs::create_dir_all(dir)?;
        std::fs::write(
            dir.join("governance_log.json"),
            serde_json::to_vec_pretty(&self.log).map_err(|e| GovError::Serde(e.to_string()))?,
        )?;
        self.influence_log
            .verify()
            .map_err(|_| GovError::InvalidState)?;
        self.influence_log
            .export_jsonl(dir.join("influence_log.jsonl"))
            .map_err(|_| GovError::InvalidState)?;
        let snap = serde_json::json!({
            "objectives": self.state.objectives(),
            "objective_registry": self.state.objective_registry()
        });
        std::fs::write(
            dir.join("objective_snapshot.json"),
            serde_json::to_vec_pretty(&snap).map_err(|e| GovError::Serde(e.to_string()))?,
        )?;
        std::fs::write(
            dir.join("spec_version.txt"),
            include_bytes!("../spec/spec_version.txt"),
        )?;
        Ok(())
    }

    // === Existing API surface (routed through validator) ===

    pub fn snapshot(&mut self) {
        self.verify_or_breach();
        let snap = Snapshot {
            objectives: self.state.objectives.clone(),
            objective_registry: self.state.objective_registry.clone(),
            drift: self.state.drift,
            conservative: self.state.conservative,
            breach_flag: self.state.breach_flag,
            governance_mode: self.state.governance_mode,
            capability_registry: self.state.capability_registry.clone(),
            consent_ledger: self.state.consent_ledger.clone(),
            delegations: self.state.delegations.clone(),
            authority_context: self.state.authority_context.clone(),
        };
        self.snapshots.push(snap);
        let _ = self.append_entry(
            DecisionKind::Snapshot,
            LogAction::GovSnapshot,
            "snapshot",
            None,
            "CAP.SNAPSHOT",
            RiskTier::Tier1,
            serde_json::json!({}),
        );
    }

    pub fn rollback(&mut self) -> Result<(), GovError> {
        self.verify_or_breach();
        if !self.state.conservative {
            return Err(GovError::Unauthorized);
        }
        let snap = self.snapshots.pop().ok_or(GovError::Unauthorized)?;
        self.state.objectives = snap.objectives;
        self.state.objective_registry = snap.objective_registry;
        self.state.drift = snap.drift;
        self.state.conservative = snap.conservative;
        self.state.breach_flag = snap.breach_flag;
        self.state.governance_mode = snap.governance_mode;
        self.state.capability_registry = snap.capability_registry;
        self.state.consent_ledger = snap.consent_ledger;
        self.state.delegations = snap.delegations;
        self.state.authority_context = snap.authority_context;
        self.append_entry(
            DecisionKind::Rollback,
            LogAction::GovRollback,
            "rollback",
            None,
            "CAP.ROLLBACK",
            RiskTier::Tier2,
            serde_json::json!({}),
        )?;
        Ok(())
    }

    pub fn ratify_add_objective(&mut self, objective: &str) -> Result<(), GovError> {
        self.require_gate()?;
        let req = ActionRequest {
            action: LogAction::GovMutateObjective,
            capability_id: "CAP.OBJECTIVE.RATIFY_ADD".into(),
            tier: RiskTier::Tier2,
            evidence_refs: vec![],
            actor_context: self.state.authority_context.clone(),
            details: format!("ratify_add_objective {objective}"),
            params: serde_json::json!({ "objective": objective }),
        };
        self.validate_and_apply(req, |s, v| {
            let o = v
                .get("objective")
                .and_then(|x| x.as_str())
                .ok_or(GovError::Serde("missing objective".into()))?
                .to_string();
            if !s.objective_registry.contains(&o) {
                s.objective_registry.push(o.clone());
            }
            if !s.objectives.contains(&o) {
                s.objectives.push(o);
            }
            Ok(())
        })
    }

    pub fn accumulate_drift(&mut self, delta: u64) {
        self.verify_or_breach();
        self.state.drift = self.state.drift.saturating_add(delta);
        let _ = self.append_entry(
            DecisionKind::DriftTick,
            LogAction::DriftTick,
            "drift tick",
            None,
            "CAP.DRIFT",
            RiskTier::Tier1,
            serde_json::json!({ "delta": delta }),
        );

        // Doc 18: Graduated response based on drift ratio
        let threshold = self.state.drift_threshold;
        let drift = self.state.drift;

        if drift >= threshold {
            // At or above threshold → Conservative + breach (legacy behavior preserved)
            self.state.conservative = true;
            self.state.breach_flag = Some(BreachClass::DriftOverthreshold);
            self.state.governance_mode = GovernanceMode::Conservative;
            self.append_breach(
                LogAction::EnterConservative,
                "enter conservative (drift)",
                BreachClass::DriftOverthreshold,
                "CAP.DRIFT",
                RiskTier::Tier2,
                serde_json::json!({}),
            );
        } else if threshold > 0 && drift * 100 / threshold >= 90 {
            // 90-99% of threshold → Restricted (new graduated state)
            if self.state.governance_mode < GovernanceMode::Restricted {
                self.state.governance_mode = GovernanceMode::Restricted;
                self.state.conservative = true; // backward compat
                let _ = self.append_entry(
                    DecisionKind::GovAction,
                    LogAction::EnterConservative,
                    "enter restricted (drift at 90%+)",
                    None,
                    "CAP.DRIFT.RESTRICTED",
                    RiskTier::Tier1,
                    serde_json::json!({ "drift_pct": drift * 100 / threshold }),
                );
            }
        } else if threshold > 0 && drift * 100 / threshold >= 75 {
            // 75-89% of threshold → Warning (advisory)
            if self.state.governance_mode < GovernanceMode::Warning {
                self.state.governance_mode = GovernanceMode::Warning;
                let _ = self.append_entry(
                    DecisionKind::GovAction,
                    LogAction::DriftTick,
                    "drift warning (75%+ threshold)",
                    None,
                    "CAP.DRIFT.WARNING",
                    RiskTier::Tier0,
                    serde_json::json!({ "drift_pct": drift * 100 / threshold }),
                );
            }
        }
    }

    pub fn exit_conservative(&mut self) -> Result<(), GovError> {
        self.require_gate()?;
        self.verify_or_breach();
        // Defense-in-depth: align with formal precondition.
        if self.state.drift >= self.state.drift_threshold {
            self.append_breach(
                LogAction::ConservativeModeViolation,
                "exit conservative blocked (drift>=threshold)",
                BreachClass::ConservativeViolation,
                "CAP.CONS.EXIT",
                RiskTier::Tier2,
                serde_json::json!({}),
            );
            return Err(GovError::ConservativeMode);
        }
        if self.state.breach_flag.is_some() {
            self.append_breach(
                LogAction::ConservativeModeViolation,
                "exit conservative blocked (breach active)",
                BreachClass::ConservativeViolation,
                "CAP.CONS.EXIT",
                RiskTier::Tier2,
                serde_json::json!({}),
            );
            return Err(GovError::ConservativeMode);
        }
        // requires Tier2 authorization (I1/I3)
        let req = ActionRequest {
            action: LogAction::ExitConservative,
            capability_id: CAP_CONSERVATIVE_EXIT.into(),
            tier: RiskTier::Tier2,
            evidence_refs: vec![],
            actor_context: self.state.authority_context.clone(),
            details: "exit_conservative".into(),
            params: serde_json::json!({}),
        };
        self.validate_and_apply(req, |s, _| {
            s.conservative = false;
            s.governance_mode = GovernanceMode::Normal;
            Ok(())
        })
    }

    // Adversarial attempts
    pub fn runtime_inject_objective_attempt(&mut self, _objective: &str) {
        self.verify_or_breach();
        self.state.conservative = true;
        self.state.governance_mode = GovernanceMode::Breach;
        self.state.breach_flag = Some(BreachClass::ObjInjection);
        self.append_breach(
            LogAction::RuntimeInjectObjectiveAttempt,
            "runtime injection attempt",
            BreachClass::ObjInjection,
            "CAP.OBJ.INJECT",
            RiskTier::Tier4,
            serde_json::json!({}),
        );
    }

    pub fn inference_bypass_attempt(&mut self) {
        self.verify_or_breach();
        self.state.conservative = true;
        self.state.governance_mode = GovernanceMode::Breach;
        self.state.breach_flag = Some(BreachClass::GovBypass);
        self.append_breach(
            LogAction::InferenceBypassAttempt,
            "inference bypass attempt",
            BreachClass::GovBypass,
            "CAP.GOV.BYPASS",
            RiskTier::Tier4,
            serde_json::json!({}),
        );
    }

    // === Test-only helpers ===
    // These are always compiled (not behind cfg) because integration tests
    // cannot enable #[cfg(test)] on the lib crate. The `_for_testing` suffix
    // and documentation make their intended use clear. Internal crate only.

    /// Test helper: clear breach flag. NOT FOR PRODUCTION USE.
    #[doc(hidden)]
    pub fn clear_breach_for_testing(&mut self) {
        self.state.breach_flag = None;
    }

    /// Test helper: open governance gate. NOT FOR PRODUCTION USE.
    #[doc(hidden)]
    pub fn open_gate_for_testing(&mut self) {
        self.gate_open = true;
    }

    /// Test helper: close governance gate. NOT FOR PRODUCTION USE.
    #[doc(hidden)]
    pub fn close_gate_for_testing(&mut self) {
        self.gate_open = false;
    }

    /// Test helper: set actor context. NOT FOR PRODUCTION USE.
    #[doc(hidden)]
    pub fn set_actor_context_for_testing(&mut self, ctx: AuthorityContext) {
        self.state.authority_context = ctx;
    }

    /// Test helper: tier4 noop. NOT FOR PRODUCTION USE.
    #[doc(hidden)]
    pub fn tier4_noop_for_testing(&mut self, capability_id: &str) -> Result<(), GovError> {
        let req = ActionRequest {
            action: LogAction::GovTier4Noop,
            capability_id: capability_id.to_string(),
            tier: RiskTier::Tier4,
            evidence_refs: vec![],
            actor_context: self.state.authority_context.clone(),
            params: serde_json::json!({}),
            details: "tier4 noop".into(),
        };
        self.validate_and_apply(req, |_s, _v| Ok(()))?;
        Ok(())
    }

    pub fn inject_tamper_for_testing(
        &mut self,
        index: usize,
        new_details: &str,
    ) -> Result<(), GovError> {
        if index >= self.log.len() {
            return Err(GovError::Unauthorized);
        }
        self.log[index].details = new_details.to_string();
        self.log_verified_ok = false;
        Ok(())
    }
}
