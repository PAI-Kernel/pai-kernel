// Fuzz target · pai_attestation::TcbManifest deserialize
//
// Adversarial input: arbitrary bytes → serde_json deserialize TcbManifest.
// Exercises Trusted Computing Base manifest parsing (Doc 10 §P1) · the
// attestation entry point that adopters / auditors use к verify framework
// integrity claims cryptographically.
//
// Adopter scenario: third party submits TCB attestation evidence как JSON ·
// pai_attestation parses + verifies. Malformed manifest could crash the
// verifier (denial-of-service against adopter governance daemon) OR enable
// confusion attacks (TCB-I4: measured state mismatch detection bypass).
//
// Probes:
//   - TcbComponent enum tag validation (9 expected variants)
//   - Hex-encoded hash fields (SHA-256 byte/char-length confusion · same
//     class as pai_witness Hash256 finding · commit e8cd83e)
//   - Nested PolicyHashes · AttestationEvidence · EnforcementState

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_attestation::TcbManifest;

fuzz_target!(|data: &[u8]| {
    let _: Result<TcbManifest, _> = serde_json::from_slice(data);
});
