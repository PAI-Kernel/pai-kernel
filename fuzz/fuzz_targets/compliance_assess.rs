// Fuzz target · pai_compliance_id · CertificationEvidence deserialize + assess
//
// Adversarial input: arbitrary bytes → serde_json deserialize CertificationEvidence
// → run assess_certification. Exercises supply-chain compliance evidence parsing
// AND tier assessment logic.
//
// Adopter scenario: adopter loads CertificationEvidence from upstream registry
// OR third-party certifier · system evaluates compliance tier. Malformed evidence
// could crash parser OR confuse tier assignment downstream.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_compliance_id::{assess_certification, CertificationEvidence};

fuzz_target!(|data: &[u8]| {
    if let Ok(evidence) = serde_json::from_slice::<CertificationEvidence>(data) {
        let _tier = assess_certification(&evidence);
    }
});
