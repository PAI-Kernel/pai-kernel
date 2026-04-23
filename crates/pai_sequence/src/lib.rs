//! # PAI Sequence Evaluator (MP-8 Tier Aggregation)
//!
//! **Constitutional reference:** PAI-CD v3.1, Amendment MP-8 (Tier Aggregation)
//!
//! ## Problem
//!
//! Individual actions may each pass per-action gating (Tier 0-1), but their
//! *sequence* constitutes a higher-tier operation. Example: 5 consecutive
//! Tier-1 identity changes within a short window collectively represent
//! drift-level manipulation that should require Tier-2+ consent.
//!
//! ## Solution
//!
//! Evaluate pending action sequences before execution. The composite tier
//! of a sequence is determined by:
//! 1. Maximum individual tier in the sequence
//! 2. Aggregation penalty based on action count and time concentration
//! 3. Domain overlap penalty (same domain repeated = escalation signal)
//!
//! ## Invariants
//!
//! | ID     | Invariant                                                      | PAI-CD ref |
//! |--------|----------------------------------------------------------------|------------|
//! | SEQ-I1 | Sequence tier >= max(individual tiers)                         | MP-8 §1    |
//! | SEQ-I2 | Empty sequence = Tier 0                                        | MP-8 §2    |
//! | SEQ-I3 | Single action sequence = that action's tier (no penalty)       | MP-8 §3    |
//! | SEQ-I4 | Aggregation is monotonic — adding actions cannot lower tier    | MP-8 §4    |
//! | SEQ-I5 | Window expiration removes actions from sequence                | MP-8 §5    |
//! | SEQ-I6 | Sequence evaluation must occur pre-execution (fail-closed)     | MP-8 §6    |
//! | SEQ-I7 | Conservative Mode blocks any sequence with composite Tier >= 2 | MP-8 §7    |

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Risk tier (mirrors governance_daemon::RiskTier for decoupling).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskTier {
    Tier0 = 0,
    Tier1 = 1,
    Tier2 = 2,
    Tier3 = 3,
    Tier4 = 4,
}

impl RiskTier {
    /// Numeric value for aggregation arithmetic.
    pub fn level(self) -> u8 {
        self as u8
    }

    /// Construct from numeric level, clamped to Tier4.
    pub fn from_level(n: u8) -> Self {
        match n {
            0 => RiskTier::Tier0,
            1 => RiskTier::Tier1,
            2 => RiskTier::Tier2,
            3 => RiskTier::Tier3,
            _ => RiskTier::Tier4,
        }
    }
}

/// A pending action in a sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    /// Unique action identifier.
    pub action_id: String,
    /// Individual risk tier of this action.
    pub tier: RiskTier,
    /// Domain category (e.g., "identity", "objective", "consent").
    pub domain: String,
    /// Unix timestamp (seconds) when the action was enqueued.
    pub enqueued_at: u64,
}

/// Sequence evaluation verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceVerdict {
    /// Sequence passes — composite tier is within allowed bounds.
    Allow {
        composite_tier: RiskTier,
        action_count: usize,
    },
    /// Sequence requires elevated consent — composite tier exceeds individual tiers.
    RequiresElevatedConsent {
        composite_tier: RiskTier,
        max_individual_tier: RiskTier,
        escalation_reason: String,
    },
    /// Sequence blocked — Conservative Mode active and composite Tier >= 2.
    BlockedConservativeMode {
        composite_tier: RiskTier,
    },
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Aggregation policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationPolicy {
    /// Rolling window duration in seconds. Actions older than this are expired.
    pub window_secs: u64,
    /// Number of same-domain actions that triggers +1 tier escalation.
    pub domain_concentration_threshold: usize,
    /// Total number of actions in window that triggers +1 tier escalation.
    pub volume_escalation_threshold: usize,
}

impl Default for AggregationPolicy {
    fn default() -> Self {
        Self {
            window_secs: 3600,                   // 1 hour
            domain_concentration_threshold: 3,    // 3 same-domain actions → escalate
            volume_escalation_threshold: 5,       // 5 total actions → escalate
        }
    }
}

// ---------------------------------------------------------------------------
// Evaluator
// ---------------------------------------------------------------------------

/// Sequence evaluator — maintains a window of pending actions and computes
/// the composite tier for the sequence.
#[derive(Debug, Clone)]
pub struct SequenceEvaluator {
    actions: Vec<PendingAction>,
    policy: AggregationPolicy,
}

impl SequenceEvaluator {
    /// Create a new evaluator with the given policy.
    pub fn new(policy: AggregationPolicy) -> Self {
        Self {
            actions: Vec::new(),
            policy,
        }
    }

    /// Create with default policy.
    pub fn with_defaults() -> Self {
        Self::new(AggregationPolicy::default())
    }

    /// Expire actions outside the rolling window.
    fn expire(&mut self, now: u64) {
        let cutoff = now.saturating_sub(self.policy.window_secs);
        self.actions.retain(|a| a.enqueued_at >= cutoff);
    }

    /// Add a pending action to the sequence.
    pub fn enqueue(&mut self, action: PendingAction) {
        self.expire(action.enqueued_at);
        self.actions.push(action);
    }

    /// Remove a completed action from the sequence.
    pub fn dequeue(&mut self, action_id: &str) {
        self.actions.retain(|a| a.action_id != action_id);
    }

    /// Clear all pending actions.
    pub fn clear(&mut self) {
        self.actions.clear();
    }

    /// Number of actions currently in the window.
    pub fn pending_count(&self) -> usize {
        self.actions.len()
    }

    /// Compute the composite tier for the current sequence.
    ///
    /// SEQ-I1: composite >= max(individual)
    /// SEQ-I2: empty = Tier0
    /// SEQ-I3: single action = its tier
    pub fn composite_tier(&self) -> RiskTier {
        if self.actions.is_empty() {
            return RiskTier::Tier0; // SEQ-I2
        }

        // Base: max individual tier (SEQ-I1)
        let max_individual = self.actions.iter()
            .map(|a| a.tier)
            .max()
            .unwrap_or(RiskTier::Tier0);

        if self.actions.len() == 1 {
            return max_individual; // SEQ-I3
        }

        let mut penalty: u8 = 0;

        // Volume escalation: too many actions in window
        if self.actions.len() >= self.policy.volume_escalation_threshold {
            penalty += 1;
        }

        // Domain concentration: same domain repeated
        let mut domain_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for action in &self.actions {
            *domain_counts.entry(&action.domain).or_insert(0) += 1;
        }
        let max_domain_count = domain_counts.values().copied().max().unwrap_or(0);
        if max_domain_count >= self.policy.domain_concentration_threshold {
            penalty += 1;
        }

        // Composite = max(individual) + penalty, clamped to Tier4
        let composite_level = max_individual.level().saturating_add(penalty).min(4);
        RiskTier::from_level(composite_level)
    }

    /// Evaluate the sequence and produce a verdict.
    ///
    /// SEQ-I6: must be called before execution (caller's responsibility).
    /// SEQ-I7: Conservative Mode check.
    pub fn evaluate(&self, conservative_mode: bool) -> SequenceVerdict {
        let composite = self.composite_tier();
        let max_individual = self.actions.iter()
            .map(|a| a.tier)
            .max()
            .unwrap_or(RiskTier::Tier0);

        // SEQ-I7: Conservative Mode blocks Tier >= 2
        if conservative_mode && composite >= RiskTier::Tier2 {
            return SequenceVerdict::BlockedConservativeMode {
                composite_tier: composite,
            };
        }

        // Did aggregation escalate beyond individual max?
        if composite > max_individual {
            let mut reasons = Vec::new();

            if self.actions.len() >= self.policy.volume_escalation_threshold {
                reasons.push(format!(
                    "volume: {} actions in window (threshold: {})",
                    self.actions.len(),
                    self.policy.volume_escalation_threshold
                ));
            }

            let mut domain_counts: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();
            for action in &self.actions {
                *domain_counts.entry(&action.domain).or_insert(0) += 1;
            }
            for (domain, count) in &domain_counts {
                if *count >= self.policy.domain_concentration_threshold {
                    reasons.push(format!(
                        "domain concentration: '{}' repeated {} times (threshold: {})",
                        domain, count, self.policy.domain_concentration_threshold
                    ));
                }
            }

            return SequenceVerdict::RequiresElevatedConsent {
                composite_tier: composite,
                max_individual_tier: max_individual,
                escalation_reason: reasons.join("; "),
            };
        }

        SequenceVerdict::Allow {
            composite_tier: composite,
            action_count: self.actions.len(),
        }
    }

    /// Evaluate the sequence *including* a hypothetical new action,
    /// without actually adding it. This allows pre-flight check.
    pub fn evaluate_with(
        &self,
        action: &PendingAction,
        conservative_mode: bool,
    ) -> SequenceVerdict {
        let mut clone = self.clone();
        clone.enqueue(action.clone());
        clone.evaluate(conservative_mode)
    }

    /// Access the current policy.
    pub fn policy(&self) -> &AggregationPolicy {
        &self.policy
    }

    /// Access pending actions (read-only).
    pub fn pending_actions(&self) -> &[PendingAction] {
        &self.actions
    }
}

// ===========================================================================
// Tests — SEQ-T01 through SEQ-T12
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn action(id: &str, tier: RiskTier, domain: &str, ts: u64) -> PendingAction {
        PendingAction {
            action_id: id.into(),
            tier,
            domain: domain.into(),
            enqueued_at: ts,
        }
    }

    // SEQ-T01: Empty sequence = Tier0 (SEQ-I2)
    #[test]
    fn seq_t01_empty_is_tier0() {
        let eval = SequenceEvaluator::with_defaults();
        assert_eq!(eval.composite_tier(), RiskTier::Tier0);
        assert_eq!(
            eval.evaluate(false),
            SequenceVerdict::Allow {
                composite_tier: RiskTier::Tier0,
                action_count: 0,
            }
        );
    }

    // SEQ-T02: Single action = its tier (SEQ-I3)
    #[test]
    fn seq_t02_single_action_no_penalty() {
        let mut eval = SequenceEvaluator::with_defaults();
        eval.enqueue(action("a1", RiskTier::Tier1, "identity", 1000));
        assert_eq!(eval.composite_tier(), RiskTier::Tier1);
    }

    // SEQ-T03: Max individual tier preserved (SEQ-I1)
    #[test]
    fn seq_t03_max_individual_preserved() {
        let mut eval = SequenceEvaluator::with_defaults();
        eval.enqueue(action("a1", RiskTier::Tier0, "identity", 1000));
        eval.enqueue(action("a2", RiskTier::Tier2, "consent", 1001));
        assert!(eval.composite_tier() >= RiskTier::Tier2);
    }

    // SEQ-T04: Volume escalation triggers tier increase
    #[test]
    fn seq_t04_volume_escalation() {
        let policy = AggregationPolicy {
            volume_escalation_threshold: 3,
            domain_concentration_threshold: 10, // high to isolate volume
            ..Default::default()
        };
        let mut eval = SequenceEvaluator::new(policy);
        // 3 Tier0 actions in different domains
        eval.enqueue(action("a1", RiskTier::Tier0, "identity", 1000));
        eval.enqueue(action("a2", RiskTier::Tier0, "consent", 1001));
        eval.enqueue(action("a3", RiskTier::Tier0, "objective", 1002));
        // Composite should be Tier0 + 1 penalty = Tier1
        assert_eq!(eval.composite_tier(), RiskTier::Tier1);
    }

    // SEQ-T05: Domain concentration triggers tier increase
    #[test]
    fn seq_t05_domain_concentration() {
        let policy = AggregationPolicy {
            volume_escalation_threshold: 100, // high to isolate domain
            domain_concentration_threshold: 3,
            ..Default::default()
        };
        let mut eval = SequenceEvaluator::new(policy);
        // 3 same-domain Tier0 actions
        eval.enqueue(action("a1", RiskTier::Tier0, "identity", 1000));
        eval.enqueue(action("a2", RiskTier::Tier0, "identity", 1001));
        eval.enqueue(action("a3", RiskTier::Tier0, "identity", 1002));
        // Composite should be Tier0 + 1 penalty = Tier1
        assert_eq!(eval.composite_tier(), RiskTier::Tier1);
    }

    // SEQ-T06: Both escalations stack
    #[test]
    fn seq_t06_dual_escalation() {
        let policy = AggregationPolicy {
            volume_escalation_threshold: 3,
            domain_concentration_threshold: 3,
            window_secs: 3600,
        };
        let mut eval = SequenceEvaluator::new(policy);
        // 3 same-domain actions (both volume and domain threshold met)
        eval.enqueue(action("a1", RiskTier::Tier0, "identity", 1000));
        eval.enqueue(action("a2", RiskTier::Tier0, "identity", 1001));
        eval.enqueue(action("a3", RiskTier::Tier0, "identity", 1002));
        // Composite should be Tier0 + 2 penalty = Tier2
        assert_eq!(eval.composite_tier(), RiskTier::Tier2);
    }

    // SEQ-T07: Monotonicity — adding actions cannot lower tier (SEQ-I4)
    #[test]
    fn seq_t07_monotonic() {
        let mut eval = SequenceEvaluator::with_defaults();
        eval.enqueue(action("a1", RiskTier::Tier2, "consent", 1000));
        let tier_after_1 = eval.composite_tier();

        eval.enqueue(action("a2", RiskTier::Tier0, "identity", 1001));
        let tier_after_2 = eval.composite_tier();

        assert!(tier_after_2 >= tier_after_1, "adding action must not lower tier");
    }

    // SEQ-T08: Window expiration removes old actions (SEQ-I5)
    #[test]
    fn seq_t08_window_expiration() {
        let policy = AggregationPolicy {
            window_secs: 100,
            ..Default::default()
        };
        let mut eval = SequenceEvaluator::new(policy);
        eval.enqueue(action("old", RiskTier::Tier2, "identity", 1000));
        // New action well beyond window
        eval.enqueue(action("new", RiskTier::Tier0, "consent", 2000));
        // Old action should have expired
        assert_eq!(eval.pending_count(), 1);
        assert_eq!(eval.composite_tier(), RiskTier::Tier0);
    }

    // SEQ-T09: Conservative Mode blocks Tier >= 2 (SEQ-I7)
    #[test]
    fn seq_t09_conservative_blocks() {
        let mut eval = SequenceEvaluator::with_defaults();
        eval.enqueue(action("a1", RiskTier::Tier2, "consent", 1000));
        let verdict = eval.evaluate(true);
        assert!(matches!(
            verdict,
            SequenceVerdict::BlockedConservativeMode { .. }
        ));
    }

    // SEQ-T10: Conservative Mode allows Tier < 2
    #[test]
    fn seq_t10_conservative_allows_low_tier() {
        let mut eval = SequenceEvaluator::with_defaults();
        eval.enqueue(action("a1", RiskTier::Tier1, "identity", 1000));
        let verdict = eval.evaluate(true);
        assert!(matches!(verdict, SequenceVerdict::Allow { .. }));
    }

    // SEQ-T11: evaluate_with does not mutate state
    #[test]
    fn seq_t11_evaluate_with_no_mutation() {
        let mut eval = SequenceEvaluator::with_defaults();
        eval.enqueue(action("a1", RiskTier::Tier0, "identity", 1000));
        let count_before = eval.pending_count();

        let hypothetical = action("a2", RiskTier::Tier3, "consent", 1001);
        let _verdict = eval.evaluate_with(&hypothetical, false);

        assert_eq!(eval.pending_count(), count_before, "evaluate_with must not mutate");
    }

    // SEQ-T12: Tier clamped to Tier4 (no overflow)
    #[test]
    fn seq_t12_tier_clamped() {
        let policy = AggregationPolicy {
            volume_escalation_threshold: 2,
            domain_concentration_threshold: 2,
            window_secs: 3600,
        };
        let mut eval = SequenceEvaluator::new(policy);
        // Tier3 base + 2 penalties = 5, but clamped to Tier4
        eval.enqueue(action("a1", RiskTier::Tier3, "identity", 1000));
        eval.enqueue(action("a2", RiskTier::Tier3, "identity", 1001));
        assert_eq!(eval.composite_tier(), RiskTier::Tier4);
    }
}
