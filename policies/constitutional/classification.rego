# classification.rego — PAI-CD Glossary / R1 MP-2
# Structural bias in a consequential domain MUST be classified as Recommendation.
package pai.constitutional.classification

default classification := "informational"

# Bias present in a consequential domain → Recommendation
classification := "recommendation" if {
    input.domain_type == "consequential"
    input.bias_detected == true
}

# Consent tier required for recommendations
consent_required := 2 if {
    classification == "recommendation"
}

consent_required := 0 if {
    classification == "informational"
}
