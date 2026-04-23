# delegation.rego — PAI-CD Consent Model P3 (Delegation)
# Validates delegation grants against scope, expiry, and revocation.
package pai.constitutional.delegation

default allow := false

allow if {
    input.delegation
    not input.delegation.revoked
    not expired
    capability_in_scope
}

expired if {
    input.delegation.expires_at > 0
    input.delegation.expires_at < input.current_time
}

capability_in_scope if {
    input.requested_capability == input.delegation.scope[_]
}

deny contains msg if {
    not input.delegation
    msg := "no delegation grant found"
}

deny contains msg if {
    input.delegation
    input.delegation.revoked == true
    msg := "delegation has been revoked"
}

deny contains msg if {
    input.delegation
    expired
    msg := "delegation has expired"
}

deny contains msg if {
    input.delegation
    not capability_in_scope
    msg := "requested capability not in delegation scope"
}
