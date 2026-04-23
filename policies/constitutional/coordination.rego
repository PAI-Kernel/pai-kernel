# coordination.rego — PAI-CD Package P Coordination Governance (C0-5, C0-6)
# Validates coordination actions against Adjacent-Only, Anti-Capture,
# and Emergency Bounded invariants.
#
# Ref: Amendment Package P (C0-1..C0-8), DL-2026-04-040
# Ref: CoordinationModel.tla verified by TLC (DL-2026-04-072)
package pai.constitutional.coordination

# ---- C0-6: Adjacent-Only Coordination ----
# Coordination occurs exclusively between neighboring layers.
# Non-adjacent communication requires active emergency exception.

default adjacent_only := false

adjacent_only if {
    abs_diff(input.source_layer, input.destination_layer) == 1
}

adjacent_only if {
    input.emergency == true
}

deny_adjacent contains msg if {
    abs_diff(input.source_layer, input.destination_layer) != 1
    not input.emergency
    msg := sprintf("C0-6 violation: non-adjacent coordination from layer %d to layer %d without emergency exception", [input.source_layer, input.destination_layer])
}

deny_adjacent contains msg if {
    input.source_layer == input.destination_layer
    msg := "C0-6 violation: coordination source and destination must differ"
}

# ---- C0-5: Anti-Capture ----
# No single entity may control more than 20% of coordination resources
# at any single layer.

default anti_capture := true

anti_capture := false if {
    some entity in input.entity_resources
    entity.percentage > 20
}

deny_capture contains msg if {
    some entity in input.entity_resources
    entity.percentage > 20
    msg := sprintf("C0-5 violation: entity '%s' controls %d%% of resources at layer %d (max 20%%)", [entity.id, entity.percentage, entity.layer])
}

# ---- C0-5: Emergency Bounded ----
# Emergency exceptions are time-bounded (maximum 90 days)
# and may not be cited as precedent for permanent restructuring.

default emergency_bounded := true

emergency_bounded := false if {
    input.emergency == true
    input.emergency_duration_days > 90
}

deny_emergency contains msg if {
    input.emergency == true
    input.emergency_duration_days > 90
    msg := sprintf("C0-5 violation: emergency exception exceeds 90-day bound (%d days)", [input.emergency_duration_days])
}

deny_emergency contains msg if {
    input.emergency == true
    input.emergency_precedent == true
    msg := "C0-5 violation: emergency exception cited as precedent for permanent restructuring"
}

# ---- Helper: absolute difference ----

abs_diff(a, b) := a - b if {
    a >= b
}

abs_diff(a, b) := b - a if {
    b > a
}
