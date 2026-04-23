//! # PAI Drift Correlation Engine (Component #4)
//!
//! Per PHASE1-TZ-001 Section 7 / P0-1 Section 7.4.
//!
//! Cross-dimension drift correlation across **7 monitored dimensions**:
//!
//! | Dimension                       | Source                          | Weight |
//! |---------------------------------|---------------------------------|--------|
//! | Identity changes                | Consent / Delegation mutations  | 1.0    |
//! | Objective adjustments           | Objective Registry mutations    | 1.5    |
//! | Personalization modifications   | Identity Layer changes          | 1.0    |
//! | Classification rule updates     | Tier reassignments              | 2.0    |
//! | Upgrade events                  | Governance logic changes        | 2.0    |
//! | Upgrade frequency               | Count per rolling window        | 1.0    |
//! | Governance modifications        | Enforcement scope changes       | 2.5    |
//!
//! ## Architecture
//!
//! - **Time-based rolling window**: events older than `window_secs` are
//!   expired on every `record()` call.
//! - Per-dimension score = sum of deltas within the window.
//! - **Composite score** = `sum(weight_i * dim_score_i)`.
//! - When `composite >= threshold`, `record()` returns `true` and
//!   [`DriftReport::breached`] is set — the caller MUST activate
//!   Conservative Mode.
//! - **Thresholds are immutable** once the engine is constructed
//!   (absent Amendment — DFT-I5).
//!
//! ## Invariants
//!
//! - DFT-I1: Dimension scores are monotonically non-decreasing within a
//!   window until events expire.
//! - DFT-I2: Window eviction is time-based (old events expire).
//! - DFT-I3: Composite score is deterministic for the same event sequence.
//! - DFT-I4: Threshold breach always signals Conservative Mode.
//! - DFT-I5: Thresholds are immutable after construction.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Number of monitored drift dimensions.
pub const DIMENSION_COUNT: usize = 7;

// ── Dimension ──────────────────────────────────────────────────────────

/// The 7 drift dimensions per PHASE1-TZ-001 Section 7.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(usize)]
pub enum DriftDimension {
    /// Consent / Delegation mutations. Weight: 1.0.
    IdentityChanges = 0,
    /// Objective Registry mutations. Weight: 1.5.
    ObjectiveAdjustments = 1,
    /// Identity Layer changes. Weight: 1.0.
    PersonalizationMods = 2,
    /// Tier reassignments. Weight: 2.0.
    ClassificationRuleUpdates = 3,
    /// Governance logic changes. Weight: 2.0.
    UpgradeEvents = 4,
    /// Count per rolling window. Weight: 1.0.
    UpgradeFrequency = 5,
    /// Enforcement scope changes. Weight: 2.5.
    GovernanceModifications = 6,
}

impl DriftDimension {
    /// All 7 dimensions in index order.
    pub const ALL: [DriftDimension; DIMENSION_COUNT] = [
        DriftDimension::IdentityChanges,
        DriftDimension::ObjectiveAdjustments,
        DriftDimension::PersonalizationMods,
        DriftDimension::ClassificationRuleUpdates,
        DriftDimension::UpgradeEvents,
        DriftDimension::UpgradeFrequency,
        DriftDimension::GovernanceModifications,
    ];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::IdentityChanges => "identity_changes",
            Self::ObjectiveAdjustments => "objective_adjustments",
            Self::PersonalizationMods => "personalization_mods",
            Self::ClassificationRuleUpdates => "classification_rule_updates",
            Self::UpgradeEvents => "upgrade_events",
            Self::UpgradeFrequency => "upgrade_frequency",
            Self::GovernanceModifications => "governance_modifications",
        }
    }
}

// ── Thresholds (immutable — DFT-I5) ───────────────────────────────────

/// Drift thresholds. Immutable once constructed (absent Amendment).
///
/// There is **no setter** — the only way to establish thresholds is
/// through [`DriftThresholds::new`]. This enforces DFT-I5.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftThresholds {
    /// Composite score that triggers Conservative Mode.
    composite: f64,
    /// Rolling window duration in seconds.
    window_secs: u64,
    /// Per-dimension weights (index = `DriftDimension as usize`).
    weights: [f64; DIMENSION_COUNT],
}

impl DriftThresholds {
    /// Construct thresholds with the spec-default weights
    /// (1.0, 1.5, 1.0, 2.0, 2.0, 1.0, 2.5).
    pub fn new(composite: f64, window_secs: u64) -> Self {
        Self {
            composite,
            window_secs,
            weights: [1.0, 1.5, 1.0, 2.0, 2.0, 1.0, 2.5],
        }
    }

    /// Construct with custom per-dimension weights.
    pub fn with_weights(composite: f64, window_secs: u64, weights: [f64; DIMENSION_COUNT]) -> Self {
        Self {
            composite,
            window_secs,
            weights,
        }
    }

    pub fn composite(&self) -> f64 {
        self.composite
    }

    pub fn window_secs(&self) -> u64 {
        self.window_secs
    }

    pub fn weights(&self) -> &[f64; DIMENSION_COUNT] {
        &self.weights
    }
}

// ── Internal event ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct DriftEvent {
    dimension: DriftDimension,
    delta: f64,
    timestamp: u64,
}

// ── Accumulator ────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct DriftAccumulator {
    events: Vec<DriftEvent>,
}

impl DriftAccumulator {
    /// Drop events older than `cutoff`.
    fn expire(&mut self, cutoff: u64) {
        self.events.retain(|e| e.timestamp >= cutoff);
    }

    fn push(&mut self, event: DriftEvent) {
        self.events.push(event);
    }

    /// Sum of deltas per dimension for events currently in the window.
    fn dimension_totals(&self) -> [f64; DIMENSION_COUNT] {
        let mut totals = [0.0f64; DIMENSION_COUNT];
        for e in &self.events {
            totals[e.dimension as usize] += e.delta;
        }
        totals
    }

    fn event_count(&self) -> usize {
        self.events.len()
    }
}

// ── Report ─────────────────────────────────────────────────────────────

/// Snapshot of the engine's current drift state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    /// Per-dimension accumulated scores within the rolling window.
    pub dimension_scores: [f64; DIMENSION_COUNT],
    /// Weighted composite score.
    pub composite_score: f64,
    /// Whether the composite score meets or exceeds the threshold.
    pub breached: bool,
    /// The configured composite threshold.
    pub threshold: f64,
    /// The configured rolling window duration.
    pub window_secs: u64,
    /// Number of events currently in the window.
    pub event_count: usize,
    /// Whether Conservative Mode should be activated.
    pub conservative_mode_required: bool,
}

// ── Engine ─────────────────────────────────────────────────────────────

/// Cross-dimension drift correlation engine.
///
/// ```text
/// record(dimension, delta, now) → bool (breached?)
/// report()                      → DriftReport
/// is_breached()                 → bool
/// ```
pub struct DriftEngine {
    thresholds: DriftThresholds,
    accumulator: DriftAccumulator,
}

impl DriftEngine {
    /// Create a new engine. Thresholds are **frozen** at construction
    /// and cannot be modified (DFT-I5).
    pub fn new(thresholds: DriftThresholds) -> Self {
        Self {
            thresholds,
            accumulator: DriftAccumulator::default(),
        }
    }

    /// Record a drift-contributing event.
    ///
    /// 1. Expires events outside the rolling window.
    /// 2. Appends the new event.
    /// 3. Recomputes the composite score.
    /// 4. Returns `true` if the threshold is now breached
    ///    (Conservative Mode signal).
    pub fn record(&mut self, dimension: DriftDimension, delta: f64, now: u64) -> bool {
        // Expire old events
        let cutoff = now.saturating_sub(self.thresholds.window_secs);
        self.accumulator.expire(cutoff);

        // Record
        self.accumulator.push(DriftEvent {
            dimension,
            delta,
            timestamp: now,
        });

        // Check breach
        self.is_breached()
    }

    /// Current drift report.
    pub fn report(&self) -> DriftReport {
        let dimension_scores = self.accumulator.dimension_totals();
        let composite = self.weighted_composite(&dimension_scores);
        let breached = composite >= self.thresholds.composite;
        DriftReport {
            dimension_scores,
            composite_score: composite,
            breached,
            threshold: self.thresholds.composite,
            window_secs: self.thresholds.window_secs,
            event_count: self.accumulator.event_count(),
            conservative_mode_required: breached,
        }
    }

    /// Whether the current composite score meets or exceeds the threshold.
    pub fn is_breached(&self) -> bool {
        let totals = self.accumulator.dimension_totals();
        self.weighted_composite(&totals) >= self.thresholds.composite
    }

    /// Read-only access to the immutable thresholds.
    pub fn thresholds(&self) -> &DriftThresholds {
        &self.thresholds
    }

    /// Number of events currently in the rolling window.
    pub fn event_count(&self) -> usize {
        self.accumulator.event_count()
    }

    // ── Internal ────────────────────────────────────────────

    fn weighted_composite(&self, dim_scores: &[f64; DIMENSION_COUNT]) -> f64 {
        let mut sum = 0.0f64;
        for (i, &score) in dim_scores.iter().enumerate() {
            sum += self.thresholds.weights[i] * score;
        }
        sum
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Spec-default thresholds: composite=10.0, 30-day window (in seconds).
    fn default_thresholds() -> DriftThresholds {
        DriftThresholds::new(10.0, 30 * 86400)
    }

    // ── DFT-T01: Single dimension below threshold → not breached ───
    #[test]
    fn dft_t01_single_dimension_below_threshold() {
        let mut engine = DriftEngine::new(default_thresholds());

        // Record a small delta in one dimension.
        // Weight for IdentityChanges = 1.0, delta = 0.5
        // Composite = 1.0 * 0.5 = 0.5 < 10.0
        let breached = engine.record(DriftDimension::IdentityChanges, 0.5, 1000);
        assert!(!breached, "single small delta must not breach");
        assert!(!engine.is_breached());

        let report = engine.report();
        assert!(!report.breached);
        assert!(!report.conservative_mode_required);
        assert!(
            report.composite_score < 10.0,
            "composite {:.2} must be below threshold 10.0",
            report.composite_score
        );
        assert_eq!(report.event_count, 1);
    }

    // ── DFT-T02: Single dimension at threshold → breached ──────────
    #[test]
    fn dft_t02_single_dimension_at_threshold() {
        let mut engine = DriftEngine::new(default_thresholds());

        // GovernanceModifications weight = 2.5
        // delta = 4.0 → composite = 2.5 * 4.0 = 10.0 → exactly at threshold
        let breached = engine.record(DriftDimension::GovernanceModifications, 4.0, 1000);
        assert!(breached, "composite == threshold must be breached");
        assert!(engine.is_breached());

        let report = engine.report();
        assert!(report.breached);
        assert!(report.conservative_mode_required);
        assert!(
            (report.composite_score - 10.0).abs() < f64::EPSILON,
            "composite {:.4} should be exactly 10.0",
            report.composite_score
        );
    }

    // ── DFT-T03: Multiple sub-threshold changes accumulate → breach
    #[test]
    fn dft_t03_sub_threshold_accumulation() {
        let mut engine = DriftEngine::new(default_thresholds());
        let now = 1000;

        // Each individual record is sub-threshold, but they accumulate.
        // IdentityChanges (w=1.0):      3 * 1.0 = 3.0   → weighted 3.0
        // ObjectiveAdjustments (w=1.5):  2 * 1.0 = 2.0   → weighted 3.0
        // GovernanceModifications (w=2.5): 1 * 1.0 = 1.0  → weighted 2.5
        // Total composite = 3.0 + 3.0 + 2.5 = 8.5 < 10.0 → not yet

        for i in 0..3 {
            let b = engine.record(DriftDimension::IdentityChanges, 1.0, now + i);
            assert!(!b, "step {} should not breach", i);
        }
        for i in 0..2 {
            let b = engine.record(DriftDimension::ObjectiveAdjustments, 1.0, now + 10 + i);
            assert!(!b, "obj step {} should not breach", i);
        }
        let b = engine.record(DriftDimension::GovernanceModifications, 1.0, now + 20);
        assert!(!b, "gov step should not breach yet (composite=8.5)");

        // One more GovernanceModifications delta pushes over:
        // Existing 2.5 + new 2.5*1.0 = 5.0; total = 3.0 + 3.0 + 5.0 = 11.0
        let b = engine.record(DriftDimension::GovernanceModifications, 1.0, now + 21);
        assert!(b, "accumulated changes must breach threshold");
        assert!(engine.is_breached());
    }

    // ── DFT-T04: Rolling window: old events expire → score decreases
    #[test]
    fn dft_t04_rolling_window_expiry() {
        // Short window: 100 seconds
        let thresholds = DriftThresholds::new(10.0, 100);
        let mut engine = DriftEngine::new(thresholds);

        // Record events at t=0..4 that breach threshold.
        // GovernanceModifications (w=2.5) * delta 5.0 = 12.5 → breached
        let b = engine.record(DriftDimension::GovernanceModifications, 5.0, 0);
        assert!(b, "should be breached immediately");

        // Advance time past the window (t=200, window=100 → cutoff=100).
        // Old event at t=0 expires.
        let b = engine.record(DriftDimension::IdentityChanges, 0.1, 200);
        assert!(
            !b,
            "after old events expire, score should drop below threshold"
        );

        let report = engine.report();
        // Only the t=200 event remains: 1.0 * 0.1 = 0.1
        assert!(
            report.composite_score < 10.0,
            "composite {:.2} should be well below threshold after expiry",
            report.composite_score
        );
        assert_eq!(report.event_count, 1, "only the new event should remain");
    }

    // ── DFT-T05: Threshold immutable: attempt to change → rejected ─
    #[test]
    fn dft_t05_threshold_immutability() {
        let thresholds = DriftThresholds::new(10.0, 30 * 86400);
        let engine = DriftEngine::new(thresholds);

        // DriftThresholds has NO public setter methods.
        // DriftEngine has NO method to replace or modify thresholds.
        // The only way to read them is through the getter:
        assert!(
            (engine.thresholds().composite() - 10.0).abs() < f64::EPSILON,
            "threshold must remain at constructed value"
        );
        assert_eq!(engine.thresholds().window_secs(), 30 * 86400);

        // Verify the spec weights are correctly set:
        let w = engine.thresholds().weights();
        let expected = [1.0, 1.5, 1.0, 2.0, 2.0, 1.0, 2.5];
        for (i, (&got, &exp)) in w.iter().zip(expected.iter()).enumerate() {
            assert!(
                (got - exp).abs() < f64::EPSILON,
                "weight[{}] = {:.1}, expected {:.1}",
                i,
                got,
                exp
            );
        }

        // Compile-time guarantee: there is no `set_threshold`,
        // `set_composite`, `set_window_secs`, or `set_weights` method.
        // This test documents the invariant.
    }

    // ── DFT-T06: Breach triggers Conservative Mode signal ──────────
    #[test]
    fn dft_t06_breach_triggers_conservative_mode() {
        let mut engine = DriftEngine::new(default_thresholds());

        // Phase 1: below threshold — no Conservative Mode signal
        engine.record(DriftDimension::UpgradeEvents, 1.0, 100);
        let report = engine.report();
        assert!(!report.conservative_mode_required);

        // Phase 2: push past threshold
        // UpgradeEvents weight=2.0, already have delta=1.0 → weighted=2.0
        // ClassificationRuleUpdates weight=2.0
        // GovernanceModifications weight=2.5
        // Need composite >= 10.0
        engine.record(DriftDimension::ClassificationRuleUpdates, 2.0, 200);
        // composite = 2.0*1.0 + 2.0*2.0 = 2.0 + 4.0 = 6.0
        engine.record(DriftDimension::GovernanceModifications, 2.0, 300);
        // composite = 2.0 + 4.0 + 2.5*2.0 = 2.0 + 4.0 + 5.0 = 11.0

        let report = engine.report();
        assert!(
            report.breached,
            "composite {:.1} must exceed threshold {:.1}",
            report.composite_score,
            report.threshold
        );
        assert!(
            report.conservative_mode_required,
            "breached drift MUST signal Conservative Mode"
        );

        // Verify `record()` itself returns the breach signal
        let signal = engine.record(DriftDimension::IdentityChanges, 0.01, 301);
        assert!(
            signal,
            "record() must return true while breached (Conservative Mode signal)"
        );
    }
}
