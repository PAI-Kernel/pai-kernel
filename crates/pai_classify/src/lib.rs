//! # PAI-Kernel RecActionBoundary (HAC Component #6)
//!
//! **Constitutional reference:** Consent & Capability Model §P1 (Pre-Execution Classification),
//! Glossary (Recommendation, Informational Output, Structural Bias),
//! Constraints & Prohibitions §MP-2 (Consequential Domain Classification),
//! R1 §MP-2 (Structural Bias Thresholds)
//!
//! ## Invariants
//!
//! | ID     | Invariant                                                                     | PAI-CD ref        |
//! |--------|-------------------------------------------------------------------------------|-------------------|
//! | RAB-I1 | Every output MUST be classified before delivery                               | Consent Model §P1 |
//! | RAB-I2 | Structural bias in consequential domain → Recommendation                      | Glossary          |
//! | RAB-I3 | Recommendation requires Tier ≥2 consent unless Author-requested               | Consent Model §P2 |
//! | RAB-I4 | Informational output MUST NOT contain structural bias in consequential domain | Glossary          |
//! | RAB-I5 | Misclassification constitutes breach                                          | Consent Model §P1 |
//! | RAB-I6 | In Conservative Mode, Recommendation generation blocked                       | Governance §P3    |
//! | RAB-I7 | Classification MUST occur pre-execution, never post-hoc                       | Decision Log §P1  |
//!
//! ## Classification logic (normative)
//!
//! ```text
//! IF domain = Consequential AND bias_signals.any_present() THEN
//!     classification := Recommendation
//! ELSE
//!     classification := Informational
//! END
//!
//! IF classification = Recommendation THEN
//!     IF conservative_mode THEN → BlockedConservativeMode
//!     ELSE IF author_explicitly_requested THEN → PassRecommendation(Tier0)
//!     ELSE → PassRecommendation(Tier2)   -- consent required
//! ELSE
//!     → PassInformational
//! END
//! ```
//!
//! ## Failure modes
//!
//! - **Classification gaming:** Independent classifier required in production.
//! - **Domain misclassification:** ConsequentialDomain enum is exhaustive; adding requires code change.
//! - **Conservative Mode race (TOCTOU):** Caller must re-check if latency > threshold.

#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Risk tier classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskTier {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
    Tier4,
}

/// Output classification result (exhaustive — no "Unknown").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputClassification {
    Informational,
    Recommendation,
}

/// Domain classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainType {
    Consequential(ConsequentialDomain),
    NonConsequential,
}

/// Consequential domains per PAI-CD.
/// Exhaustive — new variants require Amendment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsequentialDomain {
    Legal,
    Financial,
    Medical,
    Safety,
    Employment,
    Education,
    Housing,
    Insurance,
    GovernmentServices,
}

/// Structural bias signals detected in output (MP-3: quantitative thresholds).
///
/// Each field is a f64 score in `[0.0, 1.0]` where 0.0 = no bias detected
/// and 1.0 = maximum measurable bias.  A signal is "present" when its
/// score exceeds [`BiasSignals::PRESENCE_THRESHOLD`] (default 0.05).
///
/// **Breaking change (v3.1):** fields changed from `bool` to `f64`.
#[derive(Clone, Debug, PartialEq)]
pub struct BiasSignals {
    pub ordering_asymmetry: f64,
    pub framing_asymmetry: f64,
    pub sequencing_manipulation: f64,
    pub emotional_tone_skew: f64,
    pub emphasis_distortion: f64,
}

impl BiasSignals {
    /// Score threshold above which a signal is considered "present".
    /// Below this value the signal is treated as noise / not actionable.
    pub const PRESENCE_THRESHOLD: f64 = 0.05;

    /// Construct with all signals at zero (no bias detected).
    pub fn none() -> Self {
        Self {
            ordering_asymmetry: 0.0,
            framing_asymmetry: 0.0,
            sequencing_manipulation: 0.0,
            emotional_tone_skew: 0.0,
            emphasis_distortion: 0.0,
        }
    }

    /// Returns `true` if any signal exceeds `PRESENCE_THRESHOLD`.
    pub fn any_present(&self) -> bool {
        self.ordering_asymmetry > Self::PRESENCE_THRESHOLD
            || self.framing_asymmetry > Self::PRESENCE_THRESHOLD
            || self.sequencing_manipulation > Self::PRESENCE_THRESHOLD
            || self.emotional_tone_skew > Self::PRESENCE_THRESHOLD
            || self.emphasis_distortion > Self::PRESENCE_THRESHOLD
    }

    /// Returns `true` if any signal exceeds the given `threshold`.
    pub fn any_above(&self, threshold: f64) -> bool {
        self.ordering_asymmetry > threshold
            || self.framing_asymmetry > threshold
            || self.sequencing_manipulation > threshold
            || self.emotional_tone_skew > threshold
            || self.emphasis_distortion > threshold
    }

    /// Maximum signal score across all dimensions.
    pub fn max_score(&self) -> f64 {
        self.ordering_asymmetry
            .max(self.framing_asymmetry)
            .max(self.sequencing_manipulation)
            .max(self.emotional_tone_skew)
            .max(self.emphasis_distortion)
    }

    /// Aggregate score (mean of all dimensions).
    pub fn aggregate_score(&self) -> f64 {
        (self.ordering_asymmetry
            + self.framing_asymmetry
            + self.sequencing_manipulation
            + self.emotional_tone_skew
            + self.emphasis_distortion)
            / 5.0
    }
}

/// Consent record stub for boundary checking.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsentRecord {
    pub tier: RiskTier,
}

/// Boundary check result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundaryVerdict {
    /// Output is Informational — may proceed without Tier 2 consent.
    PassInformational,
    /// Output is Recommendation — requires Tier ≥2 consent.
    PassRecommendation { consent_required: RiskTier },
    /// Output blocked — Recommendation in Conservative Mode.
    BlockedConservativeMode,
    /// Classification incomplete — execution MUST NOT proceed.
    BlockedIncompleteClassification { missing: Vec<&'static str> },
}

// ---------------------------------------------------------------------------
// Classifier
// ---------------------------------------------------------------------------

/// Pre-execution output classifier.
///
/// Stateless — classification is a pure function of inputs.
/// This enforces RAB-I7 (pre-execution) by design: the caller
/// receives a verdict and must act on it before delivery.
pub struct RecActionClassifier;

impl RecActionClassifier {
    /// Classify output and determine whether it may proceed.
    ///
    /// This MUST be called BEFORE output delivery (RAB-I7).
    pub fn classify(
        domain: Option<&DomainType>,
        bias_signals: Option<&BiasSignals>,
        author_explicitly_requested: bool,
        conservative_mode: bool,
    ) -> BoundaryVerdict {
        // RAB-I7: incomplete inputs → block
        let mut missing = Vec::new();
        if domain.is_none() {
            missing.push("domain");
        }
        if bias_signals.is_none() {
            missing.push("bias_signals");
        }
        if !missing.is_empty() {
            return BoundaryVerdict::BlockedIncompleteClassification { missing };
        }

        let domain = domain.unwrap();
        let bias_signals = bias_signals.unwrap();

        // Core classification logic per spec §6.4
        let is_consequential = matches!(domain, DomainType::Consequential(_));
        let classification = if is_consequential && bias_signals.any_present() {
            OutputClassification::Recommendation
        } else {
            OutputClassification::Informational
        };

        match classification {
            OutputClassification::Recommendation => {
                if conservative_mode {
                    // RAB-I6
                    BoundaryVerdict::BlockedConservativeMode
                } else if author_explicitly_requested {
                    // Author sovereignty — Tier0 sufficient
                    BoundaryVerdict::PassRecommendation {
                        consent_required: RiskTier::Tier0,
                    }
                } else {
                    // RAB-I3: requires Tier ≥2 consent
                    BoundaryVerdict::PassRecommendation {
                        consent_required: RiskTier::Tier2,
                    }
                }
            }
            OutputClassification::Informational => {
                // RAB-I4 verified: no bias in consequential domain
                BoundaryVerdict::PassInformational
            }
        }
    }
}

// ===========================================================================
// Tests — RAB-T01 through RAB-T10 per HAC-COMP-TZ-001 §6.5
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a BiasSignals with one field set to a high score (0.8).
    fn bias_with(field: &str) -> BiasSignals {
        bias_with_score(field, 0.8)
    }

    fn bias_with_score(field: &str, score: f64) -> BiasSignals {
        let mut s = BiasSignals::none();
        match field {
            "ordering" => s.ordering_asymmetry = score,
            "framing" => s.framing_asymmetry = score,
            "sequencing" => s.sequencing_manipulation = score,
            "emotional" => s.emotional_tone_skew = score,
            "emphasis" => s.emphasis_distortion = score,
            _ => panic!("unknown field"),
        }
        s
    }

    // RAB-T01: No bias signals, non-consequential → PassInformational
    #[test]
    fn rab_t01_no_bias_non_consequential() {
        let v = RecActionClassifier::classify(
            Some(&DomainType::NonConsequential),
            Some(&BiasSignals::none()),
            false,
            false,
        );
        assert_eq!(v, BoundaryVerdict::PassInformational);
    }

    // RAB-T02: No bias signals, consequential → PassInformational
    #[test]
    fn rab_t02_no_bias_consequential() {
        let v = RecActionClassifier::classify(
            Some(&DomainType::Consequential(ConsequentialDomain::Financial)),
            Some(&BiasSignals::none()),
            false,
            false,
        );
        assert_eq!(v, BoundaryVerdict::PassInformational);
    }

    // RAB-T03: Bias signals, consequential → PassRecommendation(Tier2)
    #[test]
    fn rab_t03_bias_consequential_recommendation() {
        let v = RecActionClassifier::classify(
            Some(&DomainType::Consequential(ConsequentialDomain::Medical)),
            Some(&bias_with("framing")),
            false,
            false,
        );
        assert_eq!(
            v,
            BoundaryVerdict::PassRecommendation {
                consent_required: RiskTier::Tier2,
            }
        );
    }

    // RAB-T04: Bias signals, non-consequential → PassInformational
    #[test]
    fn rab_t04_bias_non_consequential_informational() {
        let v = RecActionClassifier::classify(
            Some(&DomainType::NonConsequential),
            Some(&bias_with("ordering")),
            false,
            false,
        );
        assert_eq!(v, BoundaryVerdict::PassInformational);
    }

    // RAB-T05: Recommendation + conservative mode → BlockedConservativeMode
    #[test]
    fn rab_t05_recommendation_conservative_blocked() {
        let v = RecActionClassifier::classify(
            Some(&DomainType::Consequential(ConsequentialDomain::Legal)),
            Some(&bias_with("emphasis")),
            false,
            true,
        );
        assert_eq!(v, BoundaryVerdict::BlockedConservativeMode);
    }

    // RAB-T06: Recommendation + author-requested → PassRecommendation(Tier0)
    #[test]
    fn rab_t06_author_requested_tier0() {
        let v = RecActionClassifier::classify(
            Some(&DomainType::Consequential(ConsequentialDomain::Financial)),
            Some(&bias_with("sequencing")),
            true,
            false,
        );
        assert_eq!(
            v,
            BoundaryVerdict::PassRecommendation {
                consent_required: RiskTier::Tier0,
            }
        );
    }

    // RAB-T07: Incomplete classification input → BlockedIncompleteClassification
    #[test]
    fn rab_t07_incomplete_blocked() {
        let v = RecActionClassifier::classify(None, None, false, false);
        assert!(matches!(
            v,
            BoundaryVerdict::BlockedIncompleteClassification { .. }
        ));

        // Missing only bias_signals
        let v2 = RecActionClassifier::classify(
            Some(&DomainType::NonConsequential),
            None,
            false,
            false,
        );
        match v2 {
            BoundaryVerdict::BlockedIncompleteClassification { missing } => {
                assert!(missing.contains(&"bias_signals"));
            }
            _ => panic!("expected BlockedIncompleteClassification"),
        }
    }

    // RAB-T08: Each bias signal type individually triggers Recommendation in consequential domain
    #[test]
    fn rab_t08_each_bias_signal_triggers() {
        for field in &["ordering", "framing", "sequencing", "emotional", "emphasis"] {
            let v = RecActionClassifier::classify(
                Some(&DomainType::Consequential(ConsequentialDomain::Safety)),
                Some(&bias_with(field)),
                false,
                false,
            );
            assert!(
                matches!(v, BoundaryVerdict::PassRecommendation { .. }),
                "bias signal '{}' should trigger Recommendation",
                field
            );
        }
    }

    // RAB-T09: All 9 consequential domains correctly classified
    #[test]
    fn rab_t09_all_consequential_domains() {
        let domains = [
            ConsequentialDomain::Legal,
            ConsequentialDomain::Financial,
            ConsequentialDomain::Medical,
            ConsequentialDomain::Safety,
            ConsequentialDomain::Employment,
            ConsequentialDomain::Education,
            ConsequentialDomain::Housing,
            ConsequentialDomain::Insurance,
            ConsequentialDomain::GovernmentServices,
        ];
        for d in &domains {
            // With bias → Recommendation
            let v = RecActionClassifier::classify(
                Some(&DomainType::Consequential(d.clone())),
                Some(&bias_with("framing")),
                false,
                false,
            );
            assert!(
                matches!(v, BoundaryVerdict::PassRecommendation { .. }),
                "domain {:?} with bias should be Recommendation",
                d
            );

            // Without bias → Informational
            let v2 = RecActionClassifier::classify(
                Some(&DomainType::Consequential(d.clone())),
                Some(&BiasSignals::none()),
                false,
                false,
            );
            assert_eq!(
                v2,
                BoundaryVerdict::PassInformational,
                "domain {:?} without bias should be Informational",
                d
            );
        }
    }

    // RAB-T10: Misclassification attempt (Recommendation labeled Informational)
    //          → impossible by type system
    #[test]
    fn rab_t10_misclassification_impossible() {
        // The classifier returns BoundaryVerdict, not OutputClassification.
        // There is no API that allows the caller to override the classification.
        // If bias_signals are present in a consequential domain, the only
        // possible return is PassRecommendation or BlockedConservativeMode.
        // RAB-I5 is enforced by construction.

        let v = RecActionClassifier::classify(
            Some(&DomainType::Consequential(ConsequentialDomain::Medical)),
            Some(&bias_with("emotional")),
            false,
            false,
        );
        // Cannot be PassInformational — type system + logic enforce this.
        assert!(!matches!(v, BoundaryVerdict::PassInformational));
    }

    // ===================================================================
    // MP-3 Quantitative Bias Threshold tests
    // ===================================================================

    // MP3-T01: Below-threshold score is NOT treated as present
    #[test]
    fn mp3_t01_below_threshold_not_present() {
        let s = bias_with_score("framing", 0.03); // below 0.05
        assert!(!s.any_present());
    }

    // MP3-T02: Above-threshold score IS treated as present
    #[test]
    fn mp3_t02_above_threshold_present() {
        let s = bias_with_score("framing", 0.06); // above 0.05
        assert!(s.any_present());
    }

    // MP3-T03: Exactly-at-threshold NOT present (strict >)
    #[test]
    fn mp3_t03_at_threshold_not_present() {
        let s = bias_with_score("ordering", BiasSignals::PRESENCE_THRESHOLD);
        assert!(!s.any_present());
    }

    // MP3-T04: max_score returns highest dimension
    #[test]
    fn mp3_t04_max_score() {
        let mut s = BiasSignals::none();
        s.ordering_asymmetry = 0.3;
        s.emotional_tone_skew = 0.7;
        s.emphasis_distortion = 0.5;
        assert!((s.max_score() - 0.7).abs() < f64::EPSILON);
    }

    // MP3-T05: aggregate_score computes mean
    #[test]
    fn mp3_t05_aggregate_score() {
        let mut s = BiasSignals::none();
        s.ordering_asymmetry = 0.5;
        s.framing_asymmetry = 0.5;
        // rest are 0.0 → mean = 1.0 / 5 = 0.2
        assert!((s.aggregate_score() - 0.2).abs() < f64::EPSILON);
    }

    // MP3-T06: any_above with custom threshold
    #[test]
    fn mp3_t06_any_above_custom() {
        let s = bias_with_score("sequencing", 0.4);
        assert!(!s.any_above(0.5));
        assert!(s.any_above(0.3));
    }

    // MP3-T07: Sub-threshold score in consequential domain → still Informational
    #[test]
    fn mp3_t07_subthreshold_consequential_informational() {
        let s = bias_with_score("framing", 0.02); // below threshold
        let v = RecActionClassifier::classify(
            Some(&DomainType::Consequential(ConsequentialDomain::Financial)),
            Some(&s),
            false,
            false,
        );
        assert_eq!(v, BoundaryVerdict::PassInformational);
    }

    // MP3-T08: Above-threshold score in consequential domain → Recommendation
    #[test]
    fn mp3_t08_above_threshold_consequential_recommendation() {
        let s = bias_with_score("framing", 0.10); // above threshold
        let v = RecActionClassifier::classify(
            Some(&DomainType::Consequential(ConsequentialDomain::Financial)),
            Some(&s),
            false,
            false,
        );
        assert!(matches!(v, BoundaryVerdict::PassRecommendation { .. }));
    }
}
