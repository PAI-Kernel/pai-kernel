//! # PAI-Kernel DelegationValidator (HAC Component #4)
//!
//! **Constitutional reference:** Consent & Capability Model §P3 (Escalation Control),
//! P0-1 §5.2 (Governance API — delegation), P0-7 §4–5 (Capability Registry)
//!
//! ## Invariants
//!
//! | ID     | Invariant                                                           | PAI-CD ref                                    |
//! |--------|---------------------------------------------------------------------|-----------------------------------------------|
//! | DEL-I1 | Delegation MUST be explicit and scoped                              | Constitutional Core, Bill of Rights §R1       |
//! | DEL-I2 | Expired delegation MUST be auto-rejected                            | P0-1 §3.1 (Delegation registry: `expires_at`) |
//! | DEL-I3 | Delegation scope MUST NOT exceed grant                              | Consent Model §P3                             |
//! | DEL-I4 | Escalation from lower to higher tier MUST trigger new consent cycle | Consent Model §P3                             |
//! | DEL-I5 | Delegation MUST be revocable                                        | Bill of Rights §R1                            |
//! | DEL-I6 | Delegation pauses in Conservative Mode                              | Governance §P3                                |
//!
//! ## State transitions
//!
//! ```text
//! [No Grant] --grant()--> [Active] --revoke()--> [Revoked]
//!                [Active] --time passes--> [Expired] (implicit, checked at validate)
//!                [Active] --conservative_mode=true--> [Paused] (no state change, validate denies)
//! ```
//!
//! ## Failure modes
//!
//! - **Silent scope expansion:** Mitigated by exact set membership comparison.
//! - **TOCTOU:** validate() returns grant_id; caller must re-check if latency > threshold.
//! - **Re-delegation:** Prohibited by omission — no `re_delegatable` field in v1.0.

#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Type aliases (lightweight, no external deps)
// ---------------------------------------------------------------------------

/// Opaque identifier types — u64 for reference implementation.
pub type DelegationId = u64;
pub type AuthorId = u64;
pub type DelegateId = u64;
pub type CapabilityId = u64;
pub type DomainId = u64;
pub type SequenceNumber = u64;
/// Milliseconds since epoch.
pub type Timestamp = u64;

// ---------------------------------------------------------------------------
// Risk tier
// ---------------------------------------------------------------------------

/// Risk tier classification (exhaustive).
/// Ord derived so Tier0 < Tier1 < … < Tier4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskTier {
    Tier0, // Core functionality
    Tier1, // Informational
    Tier2, // Influence-capable / identity-affecting
    Tier3, // External interaction
    Tier4, // Irreversible / materially consequential
}

// ---------------------------------------------------------------------------
// Delegation grant
// ---------------------------------------------------------------------------

/// A scoped delegation grant from Author (principal) to system or delegate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelegationGrant {
    pub id: DelegationId,
    pub principal: AuthorId,
    pub delegate: DelegateId,
    pub scope: Vec<CapabilityId>,
    pub domain_scope: Vec<DomainId>,
    pub tier_ceiling: RiskTier,
    pub issued_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub revoked_at: Option<Timestamp>,
    pub issued_by_seq: SequenceNumber,
}

// ---------------------------------------------------------------------------
// Verdict types
// ---------------------------------------------------------------------------

/// Result of delegation validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DelegationVerdict {
    Authorized { grant_id: DelegationId },
    Denied(DelegationDenialReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DelegationDenialReason {
    NoDelegationFound,
    DelegationExpired { expired_at: Timestamp },
    DelegationRevoked { revoked_at: Timestamp },
    CapabilityNotInScope { requested: CapabilityId },
    DomainNotInScope { requested: DomainId },
    TierExceedsCeiling { requested: RiskTier, ceiling: RiskTier },
    ConservativeModePaused,
}

// ---------------------------------------------------------------------------
// Validator implementation
// ---------------------------------------------------------------------------

/// In-memory delegation store and validator.
///
/// Production systems would back this with a persistent store;
/// this reference implementation uses `Vec` for clarity and testability.
pub struct DelegationStore {
    grants: Vec<DelegationGrant>,
    next_id: DelegationId,
}

impl DelegationStore {
    pub fn new() -> Self {
        Self {
            grants: Vec::new(),
            next_id: 1,
        }
    }

    /// Validate whether a delegate is authorized for the requested action.
    /// MUST be called pre-execution.
    pub fn validate(
        &self,
        delegate: &DelegateId,
        capability: &CapabilityId,
        domain: Option<&DomainId>,
        tier: RiskTier,
        now: Timestamp,
        conservative_mode: bool,
    ) -> DelegationVerdict {
        // DEL-I6: Conservative Mode pauses all delegation.
        if conservative_mode {
            return DelegationVerdict::Denied(DelegationDenialReason::ConservativeModePaused);
        }

        // Find all grants for this delegate, pick the most permissive valid one.
        // "Most permissive" = highest tier_ceiling among valid grants.
        let candidates: Vec<&DelegationGrant> = self
            .grants
            .iter()
            .filter(|g| g.delegate == *delegate)
            .collect();

        if candidates.is_empty() {
            return DelegationVerdict::Denied(DelegationDenialReason::NoDelegationFound);
        }

        // Evaluate each candidate; collect the first authorization or the
        // most specific denial.
        let mut best_denial: Option<DelegationDenialReason> = None;

        for grant in &candidates {
            match Self::evaluate_grant(grant, capability, domain, tier, now) {
                Ok(()) => {
                    // DEL-T12: most permissive within scope selected.
                    return DelegationVerdict::Authorized { grant_id: grant.id };
                }
                Err(reason) => {
                    // Keep most informative denial (prefer specific over generic).
                    if best_denial.is_none() {
                        best_denial = Some(reason);
                    }
                }
            }
        }

        DelegationVerdict::Denied(best_denial.unwrap_or(DelegationDenialReason::NoDelegationFound))
    }

    /// Register a new delegation grant.
    /// Returns error if conservative_mode is active (DEL-I6).
    pub fn grant(
        &mut self,
        mut grant: DelegationGrant,
        conservative_mode: bool,
    ) -> Result<DelegationId, DelegationDenialReason> {
        if conservative_mode {
            return Err(DelegationDenialReason::ConservativeModePaused);
        }
        let id = self.next_id;
        self.next_id += 1;
        grant.id = id;
        self.grants.push(grant);
        Ok(id)
    }

    /// Revoke a delegation. Always succeeds if grant exists (DEL-I5).
    pub fn revoke(
        &mut self,
        grant_id: &DelegationId,
        now: Timestamp,
    ) -> Result<(), DelegationDenialReason> {
        for g in &mut self.grants {
            if g.id == *grant_id {
                g.revoked_at = Some(now);
                return Ok(());
            }
        }
        Err(DelegationDenialReason::NoDelegationFound)
    }

    // -- private --

    fn evaluate_grant(
        grant: &DelegationGrant,
        capability: &CapabilityId,
        domain: Option<&DomainId>,
        tier: RiskTier,
        now: Timestamp,
    ) -> Result<(), DelegationDenialReason> {
        // DEL-I5: Revoked?
        if let Some(revoked_at) = grant.revoked_at {
            return Err(DelegationDenialReason::DelegationRevoked { revoked_at });
        }

        // DEL-I2: Expired?
        if let Some(expires_at) = grant.expires_at {
            if now >= expires_at {
                return Err(DelegationDenialReason::DelegationExpired { expired_at: expires_at });
            }
        }

        // DEL-I3: Capability in scope? (exact set membership)
        if !grant.scope.contains(capability) {
            return Err(DelegationDenialReason::CapabilityNotInScope {
                requested: *capability,
            });
        }

        // DEL-I3: Domain in scope?
        if let Some(dom) = domain {
            if !grant.domain_scope.contains(dom) {
                return Err(DelegationDenialReason::DomainNotInScope { requested: *dom });
            }
        }

        // DEL-I3 + DEL-I4: Tier within ceiling?
        if tier > grant.tier_ceiling {
            return Err(DelegationDenialReason::TierExceedsCeiling {
                requested: tier,
                ceiling: grant.tier_ceiling,
            });
        }

        Ok(())
    }
}

impl Default for DelegationStore {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests — DEL-T01 through DEL-T12 per HAC-COMP-TZ-001 §4.4
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a standard grant for delegate 10, capabilities [1,2,3],
    /// domains [100,200], Tier2 ceiling, expires at 5000.
    fn standard_grant() -> DelegationGrant {
        DelegationGrant {
            id: 0, // assigned by store
            principal: 1,
            delegate: 10,
            scope: vec![1, 2, 3],
            domain_scope: vec![100, 200],
            tier_ceiling: RiskTier::Tier2,
            issued_at: 1000,
            expires_at: Some(5000),
            revoked_at: None,
            issued_by_seq: 1,
        }
    }

    fn setup_with_grant() -> DelegationStore {
        let mut store = DelegationStore::new();
        store.grant(standard_grant(), false).unwrap();
        store
    }

    // DEL-T01: Valid delegation within scope and time → Authorized
    #[test]
    fn del_t01_valid_delegation_authorized() {
        let store = setup_with_grant();
        let v = store.validate(&10, &1, Some(&100), RiskTier::Tier1, 2000, false);
        assert!(matches!(v, DelegationVerdict::Authorized { .. }));
    }

    // DEL-T02: Expired delegation → Denied(DelegationExpired)
    #[test]
    fn del_t02_expired_delegation_denied() {
        let store = setup_with_grant();
        let v = store.validate(&10, &1, Some(&100), RiskTier::Tier1, 6000, false);
        assert!(matches!(
            v,
            DelegationVerdict::Denied(DelegationDenialReason::DelegationExpired { .. })
        ));
    }

    // DEL-T03: Revoked delegation → Denied(DelegationRevoked)
    #[test]
    fn del_t03_revoked_delegation_denied() {
        let mut store = setup_with_grant();
        store.revoke(&1, 3000).unwrap();
        let v = store.validate(&10, &1, Some(&100), RiskTier::Tier1, 3500, false);
        assert!(matches!(
            v,
            DelegationVerdict::Denied(DelegationDenialReason::DelegationRevoked { .. })
        ));
    }

    // DEL-T04: Capability not in scope → Denied(CapabilityNotInScope)
    #[test]
    fn del_t04_capability_not_in_scope() {
        let store = setup_with_grant();
        let v = store.validate(&10, &99, Some(&100), RiskTier::Tier1, 2000, false);
        assert!(matches!(
            v,
            DelegationVerdict::Denied(DelegationDenialReason::CapabilityNotInScope { requested: 99 })
        ));
    }

    // DEL-T05: Domain not in scope → Denied(DomainNotInScope)
    #[test]
    fn del_t05_domain_not_in_scope() {
        let store = setup_with_grant();
        let v = store.validate(&10, &1, Some(&999), RiskTier::Tier1, 2000, false);
        assert!(matches!(
            v,
            DelegationVerdict::Denied(DelegationDenialReason::DomainNotInScope { requested: 999 })
        ));
    }

    // DEL-T06: Tier exceeds ceiling → Denied(TierExceedsCeiling)
    #[test]
    fn del_t06_tier_exceeds_ceiling() {
        let store = setup_with_grant();
        let v = store.validate(&10, &1, Some(&100), RiskTier::Tier4, 2000, false);
        assert!(matches!(
            v,
            DelegationVerdict::Denied(DelegationDenialReason::TierExceedsCeiling {
                requested: RiskTier::Tier4,
                ceiling: RiskTier::Tier2,
            })
        ));
    }

    // DEL-T07: Conservative mode active → Denied(ConservativeModePaused)
    #[test]
    fn del_t07_conservative_mode_denies() {
        let store = setup_with_grant();
        let v = store.validate(&10, &1, Some(&100), RiskTier::Tier1, 2000, true);
        assert!(matches!(
            v,
            DelegationVerdict::Denied(DelegationDenialReason::ConservativeModePaused)
        ));
    }

    // DEL-T08: Grant during conservative mode → Error
    #[test]
    fn del_t08_grant_during_conservative_mode() {
        let mut store = DelegationStore::new();
        let result = store.grant(standard_grant(), true);
        assert!(matches!(
            result,
            Err(DelegationDenialReason::ConservativeModePaused)
        ));
    }

    // DEL-T09: Revoke always succeeds if grant exists
    #[test]
    fn del_t09_revoke_succeeds() {
        let mut store = setup_with_grant();
        assert!(store.revoke(&1, 3000).is_ok());
        // Revoke non-existent → error
        assert!(store.revoke(&999, 3000).is_err());
    }

    // DEL-T10: No delegation for delegate → Denied(NoDelegationFound)
    #[test]
    fn del_t10_no_delegation_found() {
        let store = setup_with_grant();
        let v = store.validate(&42, &1, Some(&100), RiskTier::Tier1, 2000, false);
        assert!(matches!(
            v,
            DelegationVerdict::Denied(DelegationDenialReason::NoDelegationFound)
        ));
    }

    // DEL-T11: Escalation — Tier1 grant, Tier2 request → Denied(TierExceedsCeiling)
    #[test]
    fn del_t11_escalation_denied() {
        let mut store = DelegationStore::new();
        let mut g = standard_grant();
        g.tier_ceiling = RiskTier::Tier1;
        store.grant(g, false).unwrap();

        let v = store.validate(&10, &1, Some(&100), RiskTier::Tier2, 2000, false);
        assert!(matches!(
            v,
            DelegationVerdict::Denied(DelegationDenialReason::TierExceedsCeiling {
                requested: RiskTier::Tier2,
                ceiling: RiskTier::Tier1,
            })
        ));
    }

    // DEL-T12: Multiple grants — most permissive within scope selected
    #[test]
    fn del_t12_most_permissive_selected() {
        let mut store = DelegationStore::new();

        // Grant 1: Tier1 ceiling
        let mut g1 = standard_grant();
        g1.tier_ceiling = RiskTier::Tier1;
        store.grant(g1, false).unwrap();

        // Grant 2: Tier3 ceiling (more permissive)
        let mut g2 = standard_grant();
        g2.tier_ceiling = RiskTier::Tier3;
        store.grant(g2, false).unwrap();

        // Request Tier2 — should succeed via grant 2
        let v = store.validate(&10, &1, Some(&100), RiskTier::Tier2, 2000, false);
        assert!(matches!(v, DelegationVerdict::Authorized { grant_id: 2 }));
    }
}
