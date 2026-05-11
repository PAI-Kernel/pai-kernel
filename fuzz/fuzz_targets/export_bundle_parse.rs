// Fuzz target · pai_export::ExportBundle deserialize + verify_bundle_integrity
//
// Adversarial input: arbitrary bytes → serde_json deserialize attempt.
// If JSON parses successfully into ExportBundle, run verify_bundle_integrity
// which exercises constitutional invariant checks on the parsed structure.
//
// Goal: discover panics, infinite loops, OR memory issues in the
// deserialization + integrity-verification path для adopter-provided bundles
// (i.e. someone attempting к import а compromised OR malformed PAI-Kernel
// governance bundle).

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_export::{verify_bundle_integrity, ExportBundle};

fuzz_target!(|data: &[u8]| {
    // Try к deserialize a bundle from arbitrary fuzzer-provided bytes.
    if let Ok(bundle) = serde_json::from_slice::<ExportBundle>(data) {
        // If valid JSON deserialization, run integrity verification.
        // Result ignored · only checking that this path doesn't panic.
        let _ = verify_bundle_integrity(&bundle);
    }
});
