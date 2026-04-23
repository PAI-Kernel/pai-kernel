# drift.rego — PAI-CD Constitutional Core I6, P0-1 7.4
# Evaluates drift composite against immutable threshold.
package pai.constitutional.drift

default breached := false

breached if {
    input.composite_score >= input.threshold
}

action := "enter_conservative" if {
    breached
}

action := "none" if {
    not breached
}
