//! Example: Compliance identity declaration (G-1) & certification (H-2)
//!
//! Demonstrates declaring compliance level and assessing certification
//! tier with anti-capture threshold enforcement.
//!
//! Run: cargo run -p pai_examples --bin compliance_identity

use pai_compliance_id::{
    assess_certification, check_anti_capture, CertificationEvidence, ComplianceCapabilities,
    ComplianceIdentity, ComplianceLevel,
};

fn main() {
    println!("=== PAI-CD Compliance Identity & Certification Example ===\n");

    // G-1: Declare compliance level
    println!("--- G-1: Compliance Identity ---\n");

    let full_caps = ComplianceCapabilities {
        core_invariants: true,
        sdk_runtime: true,
        governance_daemon: true,
        formal_verification: true,
        test_coverage: true,
    };

    let identity = ComplianceIdentity::declare(
        ComplianceLevel::PaiGoverned,
        "3.1",
        Some("1.3.0"),
        full_caps,
    );
    println!("Declared level: {:?}", identity.level);
    println!("Valid declaration: {}\n", identity.declaration_valid);

    // Try to overclaim
    let limited_caps = ComplianceCapabilities {
        core_invariants: true,
        sdk_runtime: true,
        governance_daemon: false, // no daemon!
        formal_verification: false,
        test_coverage: true,
    };

    let overclaim = ComplianceIdentity::declare(
        ComplianceLevel::PaiGoverned, // claims governed without daemon
        "3.1",
        Some("1.3.0"),
        limited_caps,
    );
    println!("Overclaim (PaiGoverned without daemon):");
    println!("  Valid: {} (correctly rejected)\n", overclaim.declaration_valid);

    // H-2: Certification tiers
    println!("--- H-2: Certification Assessment ---\n");

    let tier_a_evidence = CertificationEvidence {
        checklist_completed: true,
        test_pass_rate: Some(0.98),
        formal_proof_coverage: Some(0.45),
        certifier: "Independent Verifier A".into(),
    };
    let tier = assess_certification(&tier_a_evidence);
    println!("Evidence: tests={:.0}%, formal={:.0}%",
        tier_a_evidence.test_pass_rate.unwrap() * 100.0,
        tier_a_evidence.formal_proof_coverage.unwrap() * 100.0);
    println!("Tier: {}\n", tier.label());

    let tier_b_evidence = CertificationEvidence {
        checklist_completed: true,
        test_pass_rate: Some(0.92),
        formal_proof_coverage: None,
        certifier: "Runtime Verifier B".into(),
    };
    let tier = assess_certification(&tier_b_evidence);
    println!("Evidence: tests={:.0}%, no formal proofs",
        tier_b_evidence.test_pass_rate.unwrap() * 100.0);
    println!("Tier: {}\n", tier.label());

    // Anti-capture check
    println!("--- H-2: Anti-Capture Check ---\n");
    let certifier_counts = vec![
        ("Verifier Alpha".into(), 25),
        ("Verifier Beta".into(), 5),
        ("Verifier Gamma".into(), 10),
        ("Verifier Delta".into(), 8),
    ];
    let total = 100;
    let warnings = check_anti_capture(&certifier_counts, total);
    println!("Total certified systems: {}", total);
    println!("Threshold: 20% ({})", (total as f64 * 0.20).ceil() as usize);
    for (name, count) in &certifier_counts {
        println!("  {}: {} systems ({:.0}%)", name, count, *count as f64 / total as f64 * 100.0);
    }
    println!("\nWarnings: {}", warnings.len());
    for w in &warnings {
        println!("  {}", w);
    }
}
