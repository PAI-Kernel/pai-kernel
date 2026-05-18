// Fuzz target · pai_pii::scan_text
//
// Adversarial input: arbitrary bytes converted to UTF-8 lossy string →
// PII detection scan. Exercises regex-based pattern matching · catches
// regex DoS (ReDoS) patterns · panics on edge-case Unicode · OR
// algorithmic complexity blow-ups.
//
// Adopter scenario: end-user submits arbitrary text to their PAI instance ·
// PII detection scans it before sending downstream to AI provider. Malicious
// input that crashes OR hangs the detector = denial-of-service / governance
// bypass attack vector.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_pii::scan_text;

fuzz_target!(|data: &[u8]| {
    // Convert arbitrary bytes to UTF-8 string (lossy · replaces invalid with U+FFFD).
    let text = String::from_utf8_lossy(data);
    let _findings = scan_text(&text);
});
