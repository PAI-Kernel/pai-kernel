// Fuzz target · pai_witness::WitnessEntry deserialize
//
// Adversarial input: arbitrary bytes → serde_json deserialize attempt.
// Exercises the WitnessEntry structure parsing path · which adopters use
// when importing witness chain logs from other PAI-Kernel deployments.
//
// Goal: discover panics, infinite loops, OR memory issues in WitnessEntry
// deserialization · particularly around hash chain parsing, enum
// (DecisionClass · Initiator · ImpactScope · ReversibilityStatus) tag
// validation, and StructuredRationale handling.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_witness::WitnessEntry;

fuzz_target!(|data: &[u8]| {
    // Try к deserialize а WitnessEntry from arbitrary fuzzer-provided bytes.
    // Result ignored · only checking что this path doesn't panic.
    let _: Result<WitnessEntry, _> = serde_json::from_slice(data);
});
