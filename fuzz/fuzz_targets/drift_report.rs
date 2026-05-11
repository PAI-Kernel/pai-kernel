// Fuzz target · pai_drift::DriftReport deserialize
//
// Adversarial input: arbitrary bytes → serde_json deserialize DriftReport.
// Exercises drift-engine report parsing (DFT-I1..DFT-I5) · the structured
// artifact adopters surface к external monitoring (Grafana / Prometheus /
// audit log replay).
//
// Adopter scenario: drift reports are deserialized from upstream
// pai-kernel deployment logs · third-party monitoring tools · OR test
// fixtures. Malformed input could crash parser · confuse breach-detection
// downstream (DFT-I4: threshold breach signal integrity).
//
// Probes:
//   - DriftDimension enum tag validation (7 variants per PHASE1-TZ-001 §7.2)
//   - f64 score edge cases (NaN propagation в composite score)
//   - Weight × score multiplication overflow paths
//   - Window-eviction timestamp arithmetic

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_drift::DriftReport;

fuzz_target!(|data: &[u8]| {
    let _: Result<DriftReport, _> = serde_json::from_slice(data);
});
