# conservative.rego — PAI-CD Constitutional Core, Conservative Mode
# When Conservative Mode is active, all Tier >= 2 actions are suspended.
package pai.constitutional.conservative

default allow := true

# Block when conservative mode is active AND tier >= 2
allow := false if {
    input.conservative_mode == true
    input.required_tier >= 2
}

deny contains msg if {
    input.conservative_mode == true
    input.required_tier >= 2
    msg := "Conservative Mode active: Tier >= 2 actions suspended"
}
