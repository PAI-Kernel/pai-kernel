# coordination_test.rego — Tests for C0-5, C0-6 coordination policies
package pai.constitutional.coordination_test

import data.pai.constitutional.coordination

# ============================================================
# C0-6: Adjacent-Only Coordination Tests
# ============================================================

test_adjacent_layers_allowed if {
    coordination.adjacent_only with input as {
        "source_layer": 0,
        "destination_layer": 1,
        "emergency": false,
    }
}

test_adjacent_layers_reverse_allowed if {
    coordination.adjacent_only with input as {
        "source_layer": 2,
        "destination_layer": 1,
        "emergency": false,
    }
}

test_non_adjacent_without_emergency_denied if {
    not coordination.adjacent_only with input as {
        "source_layer": 0,
        "destination_layer": 2,
        "emergency": false,
    }
}

test_non_adjacent_with_emergency_allowed if {
    coordination.adjacent_only with input as {
        "source_layer": 0,
        "destination_layer": 3,
        "emergency": true,
    }
}

test_non_adjacent_deny_message if {
    result := coordination.deny_adjacent with input as {
        "source_layer": 0,
        "destination_layer": 5,
        "emergency": false,
    }
    count(result) > 0
}

test_same_layer_denied if {
    result := coordination.deny_adjacent with input as {
        "source_layer": 1,
        "destination_layer": 1,
        "emergency": false,
    }
    count(result) > 0
}

# ============================================================
# C0-5: Anti-Capture Tests
# ============================================================

test_under_capture_threshold_allowed if {
    coordination.anti_capture with input as {
        "entity_resources": [
            {"id": "entity_a", "percentage": 15, "layer": 0},
            {"id": "entity_b", "percentage": 10, "layer": 0},
        ],
    }
}

test_at_capture_threshold_allowed if {
    coordination.anti_capture with input as {
        "entity_resources": [
            {"id": "entity_a", "percentage": 20, "layer": 0},
        ],
    }
}

test_over_capture_threshold_denied if {
    not coordination.anti_capture with input as {
        "entity_resources": [
            {"id": "entity_a", "percentage": 21, "layer": 0},
        ],
    }
}

test_capture_deny_message if {
    result := coordination.deny_capture with input as {
        "entity_resources": [
            {"id": "megacorp", "percentage": 35, "layer": 2},
        ],
    }
    count(result) > 0
}

test_multiple_entities_one_over if {
    not coordination.anti_capture with input as {
        "entity_resources": [
            {"id": "entity_a", "percentage": 15, "layer": 0},
            {"id": "entity_b", "percentage": 25, "layer": 0},
        ],
    }
}

# ============================================================
# C0-5: Emergency Bounded Tests
# ============================================================

test_emergency_within_bound_allowed if {
    coordination.emergency_bounded with input as {
        "emergency": true,
        "emergency_duration_days": 30,
    }
}

test_emergency_at_bound_allowed if {
    coordination.emergency_bounded with input as {
        "emergency": true,
        "emergency_duration_days": 90,
    }
}

test_emergency_exceeds_bound_denied if {
    not coordination.emergency_bounded with input as {
        "emergency": true,
        "emergency_duration_days": 91,
    }
}

test_emergency_precedent_denied if {
    result := coordination.deny_emergency with input as {
        "emergency": true,
        "emergency_duration_days": 30,
        "emergency_precedent": true,
    }
    count(result) > 0
}

test_no_emergency_allowed if {
    coordination.emergency_bounded with input as {
        "emergency": false,
    }
}
