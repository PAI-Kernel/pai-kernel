# denylist.rego — PAI-CD P0-1 9 (Growth Signal Denylist)
# Reject context containing growth-signal keys.
package pai.operational.denylist

default allow := true

# Deny if any key in context matches the denylist
allow := false if {
    count(violations) > 0
}

violations[key] if {
    key := input.context_keys[_]
    data.denylist_patterns[_] == key
}

breach := true if {
    count(violations) > 0
}

breach_class := "GROWTH.SIGNAL.INJECTION" if {
    breach
}
