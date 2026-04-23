//! Example: Anti-evasion proxy correlation audit (MP-1)
//!
//! Demonstrates detecting whether proxy metrics correlate with
//! prohibited optimization targets at p < 0.05.
//!
//! Run: cargo run -p pai_examples --bin evasion_audit

use pai_evasion::{
    run_evasion_audit, ProhibitedTarget, ProxyCorrelationTest,
};

fn main() {
    println!("=== PAI-CD Anti-Evasion Audit Example ===\n");

    // Test 1: Session duration correlated with engagement (suspicious)
    let suspicious_proxy = ProxyCorrelationTest {
        proxy_name: "avg_session_duration_ms".into(),
        target: ProhibitedTarget::Engagement,
        proxy_values: (0..50).map(|i| (i as f64) * 10.0 + 100.0).collect(),
        target_values: (0..50).map(|i| (i as f64) * 15.0 + 50.0).collect(),
    };

    // Test 2: Random metric (benign)
    let benign_proxy = ProxyCorrelationTest {
        proxy_name: "response_latency_ms".into(),
        target: ProhibitedTarget::Retention,
        proxy_values: vec![120.0, 85.0, 200.0, 95.0, 150.0, 110.0, 180.0, 90.0],
        target_values: vec![0.5, 0.8, 0.3, 0.9, 0.4, 0.7, 0.2, 0.6],
    };

    let declared_objectives = vec![
        "OBJ-RELEVANCE".into(),
        "OBJ-SAFETY".into(),
        "OBJ-ACCURACY".into(),
    ];

    // Simulate detecting an undeclared objective
    let detected_objectives = vec![
        "OBJ-RELEVANCE".into(),
        "OBJ-SAFETY".into(),
        "OBJ-ACCURACY".into(),
        "OBJ-ENGAGEMENT-PROXY".into(), // undeclared!
    ];

    let report = run_evasion_audit(
        &[suspicious_proxy, benign_proxy],
        &declared_objectives,
        &detected_objectives,
        0.05,
    );

    println!("Correlation tests: {}", report.correlation_results.len());
    for r in &report.correlation_results {
        println!(
            "  {} vs {:?}: r={:.4}, p={:.6} → {}",
            r.proxy_name,
            r.target,
            r.pearson_r.unwrap_or(0.0),
            r.p_value,
            if r.significant { "⚠ SIGNIFICANT" } else { "OK" }
        );
    }

    println!("\nViolations: {}", report.violations.len());
    for v in &report.violations {
        println!("  ⚠ {} correlates with {:?} (p={:.6})", v.proxy_name, v.target, v.p_value);
    }

    println!("\nUndeclared objectives: {:?}", report.undeclared_objectives);
    println!("\nAudit result: {}", if report.pass { "PASS" } else { "FAIL" });

    println!("\n{}", serde_json::to_string_pretty(&report).unwrap());
}
