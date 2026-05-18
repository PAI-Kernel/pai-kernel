// Fuzz target · pai_classify::RecActionClassifier::classify
//
// Adversarial input: arbitrary bytes decoded into typed inputs (domain ·
// bias signals · author/conservative flags) → call classify(). Exercises
// the pre-execution classification logic (RAB-I1..RAB-I7) which gates
// every output before delivery.
//
// Adopter scenario: classification sits on hot path · arbitrary input
// combinations could panic OR enter invalid state. Particularly probes:
//   - BiasSignals f64 edge cases (NaN · ±Inf · subnormal · ±0.0)
//   - DomainType::Consequential(variant) exhaustive coverage
//   - Conservative-mode + Author-requested interaction (RAB-I3 + RAB-I6)
//
// Class of bugs not coverable by serde JSON fuzz pattern · pai_classify
// has no Deserialize types · structured fuzz via byte-decoded input.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_classify::{
    BiasSignals, ConsequentialDomain, DomainType, RecActionClassifier,
};

// Decode an f64 from 8 little-endian bytes · transmuted bit pattern
// (lets fuzzer explore NaN · Inf · subnormal naturally · BiasSignals
// stores raw f64 without validation).
fn f64_from_bytes(bytes: &[u8]) -> f64 {
    let raw = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    f64::from_bits(raw)
}

fn domain_from(b: u8) -> Option<DomainType> {
    match b % 11 {
        0 => Some(DomainType::Consequential(ConsequentialDomain::Legal)),
        1 => Some(DomainType::Consequential(ConsequentialDomain::Financial)),
        2 => Some(DomainType::Consequential(ConsequentialDomain::Medical)),
        3 => Some(DomainType::Consequential(ConsequentialDomain::Safety)),
        4 => Some(DomainType::Consequential(ConsequentialDomain::Employment)),
        5 => Some(DomainType::Consequential(ConsequentialDomain::Education)),
        6 => Some(DomainType::Consequential(ConsequentialDomain::Housing)),
        7 => Some(DomainType::Consequential(ConsequentialDomain::Insurance)),
        8 => Some(DomainType::Consequential(ConsequentialDomain::GovernmentServices)),
        9 => Some(DomainType::NonConsequential),
        _ => None,
    }
}

fuzz_target!(|data: &[u8]| {
    // Layout: byte 0 = domain selector · byte 1 = flags · bytes 2..42 = 5×f64.
    if data.len() < 42 {
        return;
    }

    let domain = domain_from(data[0]);

    let flags = data[1];
    let bias_present = flags & 0x01 != 0;
    let author_requested = flags & 0x02 != 0;
    let conservative = flags & 0x04 != 0;

    let bias = if bias_present {
        Some(BiasSignals {
            ordering_asymmetry: f64_from_bytes(&data[2..10]),
            framing_asymmetry: f64_from_bytes(&data[10..18]),
            sequencing_manipulation: f64_from_bytes(&data[18..26]),
            emotional_tone_skew: f64_from_bytes(&data[26..34]),
            emphasis_distortion: f64_from_bytes(&data[34..42]),
        })
    } else {
        None
    };

    let _verdict = RecActionClassifier::classify(
        domain.as_ref(),
        bias.as_ref(),
        author_requested,
        conservative,
    );
});
