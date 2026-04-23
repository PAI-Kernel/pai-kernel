//! Example: Pre-execution output classification (PAI-CD §P1)
//!
//! Demonstrates the RecActionBoundary — every output MUST be classified
//! before delivery as either Informational or Recommendation.
//!
//! Run: cargo run -p pai_examples --bin classify_output

use pai_classify::{
    BiasSignals, BoundaryVerdict, ConsequentialDomain, DomainType, RecActionClassifier,
};

fn main() {
    println!("=== PAI-CD Output Classification Example ===\n");

    // 1. No bias in non-consequential domain → Informational
    let verdict = RecActionClassifier::classify(
        Some(&DomainType::NonConsequential),
        Some(&BiasSignals::none()),
        false,
        false,
    );
    println!("1. Non-consequential, no bias:");
    println!("   Verdict: {:?}\n", verdict);

    // 2. Bias detected in financial domain → Recommendation (Tier 2 consent required)
    let bias = BiasSignals {
        framing_asymmetry: 0.72,
        ..BiasSignals::none()
    };
    println!("2. Financial domain, framing bias score = {:.2}:", bias.framing_asymmetry);
    println!("   Above threshold ({})? {}", BiasSignals::PRESENCE_THRESHOLD, bias.any_present());

    let verdict = RecActionClassifier::classify(
        Some(&DomainType::Consequential(ConsequentialDomain::Financial)),
        Some(&bias),
        false,
        false,
    );
    println!("   Verdict: {:?}\n", verdict);

    // 3. Same bias but Author explicitly requested → Tier 0 (sovereignty)
    let verdict = RecActionClassifier::classify(
        Some(&DomainType::Consequential(ConsequentialDomain::Financial)),
        Some(&bias),
        true, // author_explicitly_requested
        false,
    );
    println!("3. Same bias, Author-requested:");
    println!("   Verdict: {:?}\n", verdict);

    // 4. Conservative mode blocks recommendations entirely
    let verdict = RecActionClassifier::classify(
        Some(&DomainType::Consequential(ConsequentialDomain::Medical)),
        Some(&bias),
        false,
        true, // conservative_mode
    );
    println!("4. Conservative mode active:");
    println!("   Verdict: {:?}\n", verdict);

    // 5. Sub-threshold bias → not detected
    let subtle_bias = BiasSignals {
        ordering_asymmetry: 0.03, // below 0.05 threshold
        ..BiasSignals::none()
    };
    println!("5. Sub-threshold bias (score = {:.2}):", subtle_bias.ordering_asymmetry);
    println!("   Detected? {}", subtle_bias.any_present());
    let verdict = RecActionClassifier::classify(
        Some(&DomainType::Consequential(ConsequentialDomain::Legal)),
        Some(&subtle_bias),
        false,
        false,
    );
    println!("   Verdict: {:?}", verdict);

    match verdict {
        BoundaryVerdict::PassInformational => println!("   → Output may proceed without consent"),
        BoundaryVerdict::PassRecommendation { consent_required } => {
            println!("   → Requires {:?} consent", consent_required)
        }
        BoundaryVerdict::BlockedConservativeMode => println!("   → BLOCKED"),
        BoundaryVerdict::BlockedIncompleteClassification { missing } => {
            println!("   → BLOCKED: missing {:?}", missing)
        }
    }
}
