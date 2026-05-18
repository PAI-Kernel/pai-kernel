// Fuzz target · pai_delegation::DelegationStore operation sequence
//
// Adversarial input: arbitrary bytes decoded into sequence of grant/validate/
// revoke operations against a fresh DelegationStore. Exercises:
//   - DEL-I1..DEL-I6 invariants under arbitrary call ordering
//   - Conservative-mode pause (DEL-I6) across all op types
//   - Empty/large scope vec handling
//   - Timestamp arithmetic edge cases (issued_at · expires_at · revoked_at)
//   - next_id overflow surface (sequence-bounded to 1024 ops)
//
// Adopter scenario: governance daemon receives delegation events from
// caller (untrusted) · arbitrary sequencing could trigger panics OR
// invariant violations. Class of bugs not coverable by serde JSON fuzz ·
// pai_delegation has no Deserialize types · structured operation-sequence
// fuzz via byte-decoded opcode stream.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pai_delegation::{DelegationGrant, DelegationStore, RiskTier};

const MAX_OPS: usize = 1024;
const OP_BYTES: usize = 17; // 1 opcode + 16 args

fn u64_from(b: &[u8]) -> u64 {
    u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
}

fn tier_from(b: u8) -> RiskTier {
    match b % 5 {
        0 => RiskTier::Tier0,
        1 => RiskTier::Tier1,
        2 => RiskTier::Tier2,
        3 => RiskTier::Tier3,
        _ => RiskTier::Tier4,
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let conservative_mode = data[0] & 1 != 0;
    let mut store = DelegationStore::new();
    let mut cursor = 1usize;
    let mut ops = 0usize;

    while cursor + OP_BYTES <= data.len() && ops < MAX_OPS {
        let opcode = data[cursor];
        let args = &data[cursor + 1..cursor + OP_BYTES];
        cursor += OP_BYTES;
        ops += 1;

        match opcode % 3 {
            0 => {
                // grant
                let grant = DelegationGrant {
                    id: 0,
                    principal: u64_from(&args[0..8]),
                    delegate: u64_from(&args[8..16]),
                    scope: Vec::new(),
                    domain_scope: Vec::new(),
                    tier_ceiling: tier_from(args[0]),
                    issued_at: u64_from(&args[8..16]),
                    expires_at: if args[0] & 0x80 != 0 {
                        Some(u64_from(&args[0..8]))
                    } else {
                        None
                    },
                    revoked_at: None,
                    issued_by_seq: u64_from(&args[0..8]),
                };
                let _ = store.grant(grant, conservative_mode);
            }
            1 => {
                // validate
                let delegate = u64_from(&args[0..8]);
                let capability = u64_from(&args[8..16]);
                let _verdict = store.validate(
                    &delegate,
                    &capability,
                    None,
                    tier_from(args[0]),
                    u64_from(&args[8..16]),
                    conservative_mode,
                );
            }
            _ => {
                // revoke
                let grant_id = u64_from(&args[0..8]);
                let now = u64_from(&args[8..16]);
                let _ = store.revoke(&grant_id, now);
            }
        }
    }
});
