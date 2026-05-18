// Fuzz target · pai_boundary::BoundaryDeclaration deserialize
//
// Adversarial input: arbitrary bytes → serde_json deserialize attempt
// into BoundaryDeclaration. Exercises system-boundary parser path that
// adopters use to declare which components are in-scope vs out-of-scope
// (MP-5 System Boundary Declaration).
//
// Adopter scenario: adopter imports BoundaryDeclaration from a governance
// configuration file OR another deployment. Malformed declaration could
// crash the parser OR confuse the boundary-enforcement logic downstream.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_boundary::BoundaryDeclaration;

fuzz_target!(|data: &[u8]| {
    // Try to deserialize a boundary declaration from arbitrary fuzzer bytes.
    // Result ignored · only checking that this path doesn't panic.
    let _: Result<BoundaryDeclaration, _> = serde_json::from_slice(data);
});
