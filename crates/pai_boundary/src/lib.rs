//! # PAI System Boundary Declaration (MP-5)
//!
//! **Constitutional reference:** PAI-CD §MP-5, Full-Stack Binding Clause
//!
//! ## Scope
//!
//! Declares the full deployed system boundary — all components that
//! generate, transform, rank, or present outputs to the Author.
//! Compliance cannot be demonstrated by auditing inference model alone.
//!
//! ## Invariants
//!
//! | ID      | Invariant                                                         | Ref    |
//! |---------|-------------------------------------------------------------------|--------|
//! | BND-I1  | System boundary must be explicitly declared                       | MP-5   |
//! | BND-I2  | All 7 component categories must be enumerated                     | MP-5   |
//! | BND-I3  | Undeclared components are presumed non-compliant                  | MP-5   |
//! | BND-I4  | Any component violation = full system non-compliance              | MP-5   |
//! | BND-I5  | Boundary declaration is integrity-hashed                          | MP-5   |
//! | BND-I6  | Boundary declaration included in compliance statement             | MP-5   |
//!
//! # Examples
//!
//! ```
//! use pai_boundary::{BoundaryDeclaration, ComponentCategory, ComponentDeclaration};
//!
//! let mut boundary = BoundaryDeclaration::new();
//!
//! // Declare all 7 mandatory categories
//! for (i, cat) in ComponentCategory::all().iter().enumerate() {
//!     boundary.declare_component(ComponentDeclaration {
//!         component_id: format!("COMP-{}", i),
//!         category: *cat,
//!         name: format!("{:?}", cat),
//!         version: "1.0".into(),
//!         provider: "Acme Corp".into(),
//!         audited: true,
//!         compliance_notes: String::new(),
//!     });
//! }
//!
//! assert!(boundary.is_complete());
//! let validation = boundary.validate();
//! assert!(validation.fully_audited);
//! assert!(validation.missing_categories.is_empty());
//! ```

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Component Categories (MP-5 §1-7)
// ---------------------------------------------------------------------------

/// System component category per MP-5 full-stack binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentCategory {
    /// (1) The inference model (all providers).
    InferenceModel,
    /// (2) Orchestration layer (routing, chaining, agent coordination).
    Orchestration,
    /// (3) Ranking and recommendation layer.
    RankingRecommendation,
    /// (4) User interface layer (all surfaces presenting outputs).
    UserInterface,
    /// (5) Memory and personalization layer.
    MemoryPersonalization,
    /// (6) Analytics and telemetry layer.
    AnalyticsTelemetry,
    /// (7) Any component that generates, transforms, ranks, or presents outputs.
    OutputTransformer,
}

impl ComponentCategory {
    /// All 7 mandatory categories.
    pub fn all() -> [ComponentCategory; 7] {
        [
            ComponentCategory::InferenceModel,
            ComponentCategory::Orchestration,
            ComponentCategory::RankingRecommendation,
            ComponentCategory::UserInterface,
            ComponentCategory::MemoryPersonalization,
            ComponentCategory::AnalyticsTelemetry,
            ComponentCategory::OutputTransformer,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ComponentCategory::InferenceModel => "Inference Model",
            ComponentCategory::Orchestration => "Orchestration Layer",
            ComponentCategory::RankingRecommendation => "Ranking & Recommendation",
            ComponentCategory::UserInterface => "User Interface",
            ComponentCategory::MemoryPersonalization => "Memory & Personalization",
            ComponentCategory::AnalyticsTelemetry => "Analytics & Telemetry",
            ComponentCategory::OutputTransformer => "Output Transformer",
        }
    }
}

// ---------------------------------------------------------------------------
// Boundary Declaration
// ---------------------------------------------------------------------------

/// A declared system component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDeclaration {
    /// Unique component identifier.
    pub component_id: String,
    /// Category this component belongs to.
    pub category: ComponentCategory,
    /// Human-readable name/description.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Provider or maintainer.
    pub provider: String,
    /// Whether this component has been compliance-audited.
    pub audited: bool,
    /// Notes on compliance status.
    pub compliance_notes: String,
}

/// Full system boundary declaration (BND-I1, BND-I2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryDeclaration {
    /// All declared components.
    components: Vec<ComponentDeclaration>,
    /// Integrity hash of the declaration.
    declaration_hash: Option<String>,
}

impl BoundaryDeclaration {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            declaration_hash: None,
        }
    }

    /// Add a component to the boundary declaration.
    pub fn declare_component(&mut self, component: ComponentDeclaration) {
        self.declaration_hash = None; // invalidate
        self.components.push(component);
    }

    /// Get all declared components.
    pub fn components(&self) -> &[ComponentDeclaration] {
        &self.components
    }

    /// Number of declared components.
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Check which of the 7 mandatory categories are covered (BND-I2).
    pub fn covered_categories(&self) -> Vec<ComponentCategory> {
        let mut seen = std::collections::HashSet::new();
        for c in &self.components {
            seen.insert(c.category);
        }
        seen.into_iter().collect()
    }

    /// Check which mandatory categories are missing.
    pub fn missing_categories(&self) -> Vec<ComponentCategory> {
        let covered: std::collections::HashSet<_> = self.components.iter().map(|c| c.category).collect();
        ComponentCategory::all()
            .into_iter()
            .filter(|cat| !covered.contains(cat))
            .collect()
    }

    /// Whether all 7 mandatory categories are declared (BND-I2).
    pub fn is_complete(&self) -> bool {
        self.missing_categories().is_empty()
    }

    /// Validate the boundary declaration.
    pub fn validate(&self) -> BoundaryValidation {
        let missing = self.missing_categories();
        let unaudited: Vec<String> = self
            .components
            .iter()
            .filter(|c| !c.audited)
            .map(|c| c.component_id.clone())
            .collect();

        BoundaryValidation {
            complete: missing.is_empty(),
            fully_audited: unaudited.is_empty() && missing.is_empty(),
            missing_categories: missing,
            unaudited_components: unaudited,
        }
    }

    /// Finalize the declaration with an integrity hash (BND-I5).
    pub fn finalize(&mut self) -> String {
        let serialized = serde_json::to_string(&self.components).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let hash = hex::encode(hasher.finalize());
        self.declaration_hash = Some(hash.clone());
        hash
    }

    /// Verify integrity hash.
    pub fn verify_integrity(&self) -> bool {
        let Some(ref stored) = self.declaration_hash else {
            return false;
        };
        let serialized = serde_json::to_string(&self.components).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        hex::encode(hasher.finalize()) == *stored
    }
}

impl Default for BoundaryDeclaration {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of boundary validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryValidation {
    /// All 7 categories declared.
    pub complete: bool,
    /// Which categories are missing.
    pub missing_categories: Vec<ComponentCategory>,
    /// Components declared but not audited.
    pub unaudited_components: Vec<String>,
    /// Complete + all audited.
    pub fully_audited: bool,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_component(id: &str, cat: ComponentCategory) -> ComponentDeclaration {
        ComponentDeclaration {
            component_id: id.into(),
            category: cat,
            name: format!("Test {}", id),
            version: "1.0.0".into(),
            provider: "PAI-Kernel".into(),
            audited: true,
            compliance_notes: String::new(),
        }
    }

    fn full_boundary() -> BoundaryDeclaration {
        let mut bd = BoundaryDeclaration::new();
        for (i, cat) in ComponentCategory::all().iter().enumerate() {
            bd.declare_component(make_component(&format!("C{}", i), *cat));
        }
        bd
    }

    // BND-T01: Empty boundary is incomplete
    #[test]
    fn bnd_t01_empty_incomplete() {
        let bd = BoundaryDeclaration::new();
        assert!(!bd.is_complete());
        assert_eq!(bd.missing_categories().len(), 7);
    }

    // BND-T02: Full boundary with all 7 categories is complete
    #[test]
    fn bnd_t02_full_boundary_complete() {
        let bd = full_boundary();
        assert!(bd.is_complete());
        assert!(bd.missing_categories().is_empty());
    }

    // BND-T03: Missing one category detected
    #[test]
    fn bnd_t03_missing_category() {
        let mut bd = BoundaryDeclaration::new();
        // Add 6 of 7 categories (skip UserInterface)
        for cat in &[
            ComponentCategory::InferenceModel,
            ComponentCategory::Orchestration,
            ComponentCategory::RankingRecommendation,
            ComponentCategory::MemoryPersonalization,
            ComponentCategory::AnalyticsTelemetry,
            ComponentCategory::OutputTransformer,
        ] {
            bd.declare_component(make_component("x", *cat));
        }
        assert!(!bd.is_complete());
        let missing = bd.missing_categories();
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&ComponentCategory::UserInterface));
    }

    // BND-T04: Validation reports unaudited components
    #[test]
    fn bnd_t04_unaudited_flagged() {
        let mut bd = full_boundary();
        bd.components[0].audited = false;
        let val = bd.validate();
        assert!(val.complete);
        assert!(!val.fully_audited);
        assert_eq!(val.unaudited_components.len(), 1);
    }

    // BND-T05: Finalize and verify integrity
    #[test]
    fn bnd_t05_integrity() {
        let mut bd = full_boundary();
        bd.finalize();
        assert!(bd.verify_integrity());
    }

    // BND-T06: Tampering invalidates integrity
    #[test]
    fn bnd_t06_tamper_detected() {
        let mut bd = full_boundary();
        bd.finalize();
        bd.components.push(make_component("tampered", ComponentCategory::InferenceModel));
        // Hash was invalidated by mutation through declare_component... but we pushed directly
        // so declaration_hash remains the old one
        assert!(!bd.verify_integrity());
    }

    // BND-T07: All 7 categories accessible
    #[test]
    fn bnd_t07_all_categories() {
        let all = ComponentCategory::all();
        assert_eq!(all.len(), 7);
        for cat in &all {
            assert!(!cat.label().is_empty());
        }
    }

    // BND-T08: Serialization roundtrip
    #[test]
    fn bnd_t08_serde() {
        let mut bd = full_boundary();
        bd.finalize();
        let json = serde_json::to_string(&bd).unwrap();
        let parsed: BoundaryDeclaration = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.component_count(), 7);
        assert!(parsed.verify_integrity());
    }

    // BND-T09: Multiple components per category allowed
    #[test]
    fn bnd_t09_multiple_per_category() {
        let mut bd = full_boundary();
        bd.declare_component(make_component("extra", ComponentCategory::InferenceModel));
        assert_eq!(bd.component_count(), 8);
        assert!(bd.is_complete());
    }

    // BND-T10: Fully audited = complete + all audited
    #[test]
    fn bnd_t10_fully_audited() {
        let bd = full_boundary();
        let val = bd.validate();
        assert!(val.fully_audited);
    }
}
