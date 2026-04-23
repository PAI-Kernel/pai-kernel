# tier_gates.rego — PAI-CD Consent and Capability Model
# Enforces risk tier requirements per action type.
package pai.operational.tier_gates

default allow := false

# Tier 0 and Tier 1 are always allowed (no consent required)
allow if {
    input.required_tier <= 1
}

# Tier 2+ requires explicit active consent
allow if {
    input.required_tier >= 2
    input.has_consent == true
    not input.conservative_mode
}

# Tier 4 additionally requires dual confirmation
allow if {
    input.required_tier == 4
    input.has_dual_confirm == true
    not input.conservative_mode
}

deny[msg] if {
    input.required_tier >= 2
    not input.has_consent
    msg := "Tier >= 2 requires explicit consent"
}

deny[msg] if {
    input.required_tier >= 2
    input.conservative_mode == true
    msg := "Conservative Mode blocks Tier >= 2"
}
