# consent.rego — PAI-CD Consent Model P2
# Silence MUST NOT constitute consent (WIT-I1, DEL-I1).
package pai.constitutional.consent

default allow := false

# Explicit consent with matching tier permits the action.
allow if {
    input.consent_record.explicit == true
    input.consent_record.tier >= input.required_tier
    not input.consent_record.revoked
}

# Deny reasons (collected as a set).
deny contains msg if {
    not input.consent_record
    msg := "no consent record provided"
}

deny contains msg if {
    input.consent_record
    input.consent_record.explicit != true
    msg := "silence does not constitute consent (P2)"
}

deny contains msg if {
    input.consent_record
    input.consent_record.revoked == true
    msg := "consent has been revoked"
}

deny contains msg if {
    input.consent_record
    input.consent_record.tier < input.required_tier
    msg := "consent tier insufficient"
}
