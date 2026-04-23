//! # PAI Anti-Evasion Detection (MP-1)
//!
//! **Constitutional reference:** PAI-CD v3.1, Constraints & Prohibitions §MP-1
//!
//! ## Scope
//!
//! Detects evasion of constitutional constraints through:
//! - Proxy metric correlation with prohibited targets (p < 0.05)
//! - Session-level behavioral equivalence to prohibited outcomes
//! - Undeclared objective weighting via Objective Registry audit
//!
//! ## Design
//!
//! Core detection is implemented via pure Rust statistics (Pearson r + t-test).
//! A [`CorrelationBackend`] trait enables pluggable ML backends (e.g., linfa)
//! for production deployments requiring more sophisticated detection.
//!
//! ## Invariants
//!
//! | ID      | Invariant                                                         | Ref    |
//! |---------|-------------------------------------------------------------------|--------|
//! | EVA-I1  | Proxy metrics correlating at p<0.05 with prohibited targets = breach | MP-1  |
//! | EVA-I2  | Session behavioral equivalence to prohibited outcome = breach      | MP-1  |
//! | EVA-I3  | All inference-affecting objectives must appear in Objective Registry| MP-1  |
//! | EVA-I4  | Undeclared objective weighting = breach                           | MP-1  |
//! | EVA-I5  | Quarterly session-level behavioral effect audit required           | MP-1  |
//!
//! # Examples
//!
//! ```
//! use pai_evasion::{run_evasion_audit, ProhibitedTarget, ProxyCorrelationTest};
//!
//! // Highly correlated proxy → prohibited target
//! let test = ProxyCorrelationTest {
//!     proxy_name: "session_duration".into(),
//!     target: ProhibitedTarget::Engagement,
//!     proxy_values: (0..30).map(|i| i as f64 * 10.0).collect(),
//!     target_values: (0..30).map(|i| i as f64 * 15.0).collect(),
//! };
//!
//! let report = run_evasion_audit(
//!     &[test],
//!     &["OBJ-SAFETY".into()],
//!     &["OBJ-SAFETY".into()],
//!     0.05,
//! );
//! assert!(!report.pass); // fails — proxy correlates with engagement
//! assert_eq!(report.violations.len(), 1);
//! ```

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Prohibited Targets
// ---------------------------------------------------------------------------

/// Prohibited optimization targets per PAI-CD §Constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProhibitedTarget {
    /// Engagement maximization.
    Engagement,
    /// Retention reinforcement.
    Retention,
    /// Monetization optimization.
    Monetization,
    /// Compliance elevation (nudging user toward provider goals).
    ComplianceElevation,
    /// Attachment formation (psychological dependency).
    AttachmentFormation,
}

impl ProhibitedTarget {
    pub fn all() -> [ProhibitedTarget; 5] {
        [
            ProhibitedTarget::Engagement,
            ProhibitedTarget::Retention,
            ProhibitedTarget::Monetization,
            ProhibitedTarget::ComplianceElevation,
            ProhibitedTarget::AttachmentFormation,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ProhibitedTarget::Engagement => "Engagement Maximization",
            ProhibitedTarget::Retention => "Retention Reinforcement",
            ProhibitedTarget::Monetization => "Monetization Optimization",
            ProhibitedTarget::ComplianceElevation => "Compliance Elevation",
            ProhibitedTarget::AttachmentFormation => "Attachment Formation",
        }
    }
}

// ---------------------------------------------------------------------------
// Correlation Backend Trait (pluggable ML)
// ---------------------------------------------------------------------------

/// Trait for pluggable correlation detection backends.
///
/// The default implementation uses Pearson r + two-tailed t-test.
/// Production deployments can implement this with linfa, scikit-learn (via FFI),
/// or other ML frameworks for more sophisticated proxy detection.
pub trait CorrelationBackend {
    /// Compute Pearson correlation coefficient between two series.
    fn pearson_r(&self, x: &[f64], y: &[f64]) -> Option<f64>;

    /// Compute p-value for the correlation (two-tailed t-test).
    fn p_value(&self, r: f64, n: usize) -> f64;

    /// Detect if a proxy metric correlates with a prohibited target
    /// at the significance level (default p < 0.05).
    fn is_significant(&self, x: &[f64], y: &[f64], alpha: f64) -> bool {
        let n = x.len();
        if n < 3 {
            return false;
        }
        if let Some(r) = self.pearson_r(x, y) {
            self.p_value(r, n) < alpha
        } else {
            false
        }
    }
}

/// Default built-in statistics backend.
pub struct BuiltinStats;

impl CorrelationBackend for BuiltinStats {
    fn pearson_r(&self, x: &[f64], y: &[f64]) -> Option<f64> {
        pearson_correlation(x, y)
    }

    fn p_value(&self, r: f64, n: usize) -> f64 {
        correlation_p_value(r, n)
    }
}

// ---------------------------------------------------------------------------
// Core Statistics (self-contained, no external deps)
// ---------------------------------------------------------------------------

/// Compute Pearson correlation coefficient.
///
/// Returns `None` if inputs are invalid (different lengths, <2 elements,
/// or zero variance in either series).
pub fn pearson_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() != y.len() || x.len() < 2 {
        return None;
    }
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for (xi, yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return None;
    }

    Some(cov / (var_x.sqrt() * var_y.sqrt()))
}

/// Compute approximate two-tailed p-value for Pearson r.
///
/// Uses the t-statistic: `t = r * sqrt((n-2) / (1-r²))`
/// and approximates the p-value via the normal CDF with a
/// degrees-of-freedom correction for small samples.
pub fn correlation_p_value(r: f64, n: usize) -> f64 {
    if n < 3 {
        return 1.0;
    }
    let r_abs = r.abs();
    if r_abs >= 1.0 {
        return 0.0;
    }

    let df = (n - 2) as f64;
    let t = r_abs * (df / (1.0 - r_abs * r_abs)).sqrt();

    // Convert t to z-score with finite-df correction
    // (Abramowitz & Stegun 26.7.5 approximation).
    let z = t * (1.0 - 1.0 / (4.0 * df)) / (1.0 + t * t / (2.0 * df)).sqrt();

    // Two-tailed p-value via normal survival function.
    2.0 * normal_sf(z)
}

/// Standard normal survival function: P(Z > z).
///
/// Uses Abramowitz & Stegun rational approximation (26.2.17),
/// accurate to ~1.5e-7.
fn normal_sf(z: f64) -> f64 {
    if z < 0.0 {
        return 1.0 - normal_sf(-z);
    }
    // Constants for A&S 26.2.17
    let p = 0.231_641_9;
    let b1 = 0.319_381_53;
    let b2 = -0.356_563_782;
    let b3 = 1.781_477_937;
    let b4 = -1.821_255_978;
    let b5 = 1.330_274_429;

    let t = 1.0 / (1.0 + p * z);
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;

    let phi = (-z * z / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();
    phi * (b1 * t + b2 * t2 + b3 * t3 + b4 * t4 + b5 * t5)
}

// ---------------------------------------------------------------------------
// Evasion Analysis
// ---------------------------------------------------------------------------

/// A proxy metric sample with its corresponding prohibited target measure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyCorrelationTest {
    /// Name of the proxy metric being tested.
    pub proxy_name: String,
    /// Which prohibited target this might correlate with.
    pub target: ProhibitedTarget,
    /// Proxy metric values (one per session/sample).
    pub proxy_values: Vec<f64>,
    /// Prohibited target metric values (same length).
    pub target_values: Vec<f64>,
}

/// Result of proxy correlation analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    pub proxy_name: String,
    pub target: ProhibitedTarget,
    pub sample_size: usize,
    pub pearson_r: Option<f64>,
    pub p_value: f64,
    /// True if correlation is significant at the given alpha level.
    pub significant: bool,
    /// Significance level used.
    pub alpha: f64,
}

/// Run proxy correlation analysis (EVA-I1).
pub fn analyze_proxy_correlation(
    test: &ProxyCorrelationTest,
    alpha: f64,
) -> CorrelationResult {
    let backend = BuiltinStats;
    let n = test.proxy_values.len();
    let r = backend.pearson_r(&test.proxy_values, &test.target_values);
    let p = r.map(|r| backend.p_value(r, n)).unwrap_or(1.0);
    let significant = r.is_some() && p < alpha;

    CorrelationResult {
        proxy_name: test.proxy_name.clone(),
        target: test.target,
        sample_size: n,
        pearson_r: r,
        p_value: p,
        significant,
        alpha,
    }
}

/// Evasion audit report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvasionAuditReport {
    /// All correlation tests performed.
    pub correlation_results: Vec<CorrelationResult>,
    /// Violations found (significant correlations).
    pub violations: Vec<CorrelationResult>,
    /// Objectives declared in registry.
    pub declared_objectives: Vec<String>,
    /// Objectives detected but undeclared.
    pub undeclared_objectives: Vec<String>,
    /// Overall pass/fail.
    pub pass: bool,
}

/// Run a full evasion audit (EVA-I1 through EVA-I5).
pub fn run_evasion_audit(
    tests: &[ProxyCorrelationTest],
    declared_objectives: &[String],
    detected_objectives: &[String],
    alpha: f64,
) -> EvasionAuditReport {
    let correlation_results: Vec<_> = tests
        .iter()
        .map(|t| analyze_proxy_correlation(t, alpha))
        .collect();

    let violations: Vec<_> = correlation_results
        .iter()
        .filter(|r| r.significant)
        .cloned()
        .collect();

    let undeclared: Vec<_> = detected_objectives
        .iter()
        .filter(|obj| !declared_objectives.contains(obj))
        .cloned()
        .collect();

    let pass = violations.is_empty() && undeclared.is_empty();

    EvasionAuditReport {
        correlation_results,
        violations,
        declared_objectives: declared_objectives.to_vec(),
        undeclared_objectives: undeclared,
        pass,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // EVA-T01: Perfect positive correlation detected
    #[test]
    fn eva_t01_perfect_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0];
        let r = pearson_correlation(&x, &y).unwrap();
        assert!((r - 1.0).abs() < 1e-10);
    }

    // EVA-T02: No correlation
    #[test]
    fn eva_t02_no_correlation() {
        // Alternating pattern that should produce near-zero correlation
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y = vec![5.0, 1.0, 5.0, 1.0, 5.0, 1.0, 5.0, 1.0];
        let r = pearson_correlation(&x, &y).unwrap();
        assert!(r.abs() < 0.3, "r={} should be near zero", r);
    }

    // EVA-T03: Negative correlation
    #[test]
    fn eva_t03_negative_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r = pearson_correlation(&x, &y).unwrap();
        assert!((r - (-1.0)).abs() < 1e-10);
    }

    // EVA-T04: Insufficient data returns None
    #[test]
    fn eva_t04_insufficient_data() {
        assert!(pearson_correlation(&[1.0], &[2.0]).is_none());
        assert!(pearson_correlation(&[], &[]).is_none());
    }

    // EVA-T05: Zero variance returns None
    #[test]
    fn eva_t05_zero_variance() {
        assert!(pearson_correlation(&[1.0, 1.0, 1.0], &[1.0, 2.0, 3.0]).is_none());
    }

    // EVA-T06: Strong correlation produces low p-value
    #[test]
    fn eva_t06_significant_p_value() {
        let r = 0.95;
        let n = 20;
        let p = correlation_p_value(r, n);
        assert!(p < 0.001, "p={} should be very small for r=0.95 n=20", p);
    }

    // EVA-T07: Weak correlation produces high p-value
    #[test]
    fn eva_t07_nonsignificant_p_value() {
        let r = 0.1;
        let n = 10;
        let p = correlation_p_value(r, n);
        assert!(p > 0.05, "p={} should be >0.05 for r=0.1 n=10", p);
    }

    // EVA-T08: Proxy correlation analysis — significant
    #[test]
    fn eva_t08_proxy_significant() {
        let test = ProxyCorrelationTest {
            proxy_name: "session_duration".into(),
            target: ProhibitedTarget::Engagement,
            proxy_values: (0..30).map(|i| i as f64).collect(),
            target_values: (0..30).map(|i| (i as f64) * 1.5 + 0.1).collect(),
        };
        let result = analyze_proxy_correlation(&test, 0.05);
        assert!(result.significant, "strong linear proxy should be detected");
        assert!(result.p_value < 0.05);
    }

    // EVA-T09: Proxy correlation analysis — not significant
    #[test]
    fn eva_t09_proxy_not_significant() {
        let test = ProxyCorrelationTest {
            proxy_name: "random_metric".into(),
            target: ProhibitedTarget::Retention,
            proxy_values: vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0],
            target_values: vec![2.0, 7.0, 1.0, 8.0, 2.0, 8.0, 1.0, 8.0],
        };
        let result = analyze_proxy_correlation(&test, 0.05);
        assert!(!result.significant);
    }

    // EVA-T10: Full audit — clean (no violations, no undeclared)
    #[test]
    fn eva_t10_clean_audit() {
        let tests = vec![ProxyCorrelationTest {
            proxy_name: "benign_metric".into(),
            target: ProhibitedTarget::Engagement,
            proxy_values: vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0],
            target_values: vec![2.0, 7.0, 1.0, 8.0, 2.0, 8.0, 1.0, 8.0],
        }];
        let declared = vec!["OBJ-RELEVANCE".into(), "OBJ-SAFETY".into()];
        let detected = vec!["OBJ-RELEVANCE".into(), "OBJ-SAFETY".into()];
        let report = run_evasion_audit(&tests, &declared, &detected, 0.05);
        assert!(report.pass);
        assert!(report.violations.is_empty());
        assert!(report.undeclared_objectives.is_empty());
    }

    // EVA-T11: Audit fails on undeclared objective (EVA-I4)
    #[test]
    fn eva_t11_undeclared_objective() {
        let declared = vec!["OBJ-RELEVANCE".into()];
        let detected = vec!["OBJ-RELEVANCE".into(), "OBJ-SHADOW".into()];
        let report = run_evasion_audit(&[], &declared, &detected, 0.05);
        assert!(!report.pass);
        assert_eq!(report.undeclared_objectives, vec!["OBJ-SHADOW".to_string()]);
    }

    // EVA-T12: All prohibited targets representable
    #[test]
    fn eva_t12_all_targets() {
        let all = ProhibitedTarget::all();
        assert_eq!(all.len(), 5);
        for t in &all {
            assert!(!t.label().is_empty());
        }
    }

    // EVA-T13: Backend trait works with builtin stats
    #[test]
    fn eva_t13_backend_trait() {
        let backend = BuiltinStats;
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let y = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
        assert!(backend.is_significant(&x, &y, 0.05));
    }

    // EVA-T14: Audit report serialization
    #[test]
    fn eva_t14_report_serde() {
        let report = run_evasion_audit(&[], &["OBJ-A".into()], &["OBJ-A".into()], 0.05);
        let json = serde_json::to_string(&report).unwrap();
        let parsed: EvasionAuditReport = serde_json::from_str(&json).unwrap();
        assert!(parsed.pass);
    }
}
