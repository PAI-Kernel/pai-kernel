// Fuzz target · pai_provenance · ProvenanceMetadata deserialize
//
// Adversarial input: arbitrary bytes → serde_json deserialize ProvenanceMetadata.
// Exercises supply-chain artifact provenance parsing (Doc 11 · Supply Chain
// and Artifact Provenance).
//
// Adopter scenario: adopter imports ProvenanceMetadata from upstream artifact
// registry OR signed bundle. Malformed provenance could crash parser ·
// confuse trust assessment downstream · OR (worst case) enable supply-chain
// confusion attack.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_provenance::ProvenanceMetadata;

fuzz_target!(|data: &[u8]| {
    let _: Result<ProvenanceMetadata, _> = serde_json::from_slice(data);
});
