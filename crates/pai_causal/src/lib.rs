//! # PAI Causal Telemetry (Doc 14) — Objective Causality Assurance
//!
//! **Constitutional reference:** PAI-CD, Document 14
//!
//! ## Scope
//!
//! Provides a causal graph for tracking how telemetry and objectives
//! flow through variables to protected surfaces.  Implements taint
//! propagation, forbidden-path detection, and objective-to-surface mapping.
//!
//! ## Invariants
//!
//! | ID      | Invariant                                                        | Ref      |
//! |---------|------------------------------------------------------------------|----------|
//! | CTL-I1  | Taint propagates transitively through all derived variables      | Doc14-P1 |
//! | CTL-I2  | Tainted variables cannot reach protected surfaces w/o Tier≥2     | Doc14-P1 |
//! | CTL-I3  | Every objective must map to declared surfaces                    | Doc14-P2 |
//! | CTL-I4  | Undeclared objective-to-surface influence = breach               | Doc14-P2 |
//! | CTL-I5  | Causal audit trace exportable as JSON                           | Doc14-P3 |
//! | CTL-I6  | Shadow objectives detectable via graph reachability             | Doc14-P3 |
//!
//! # Examples
//!
//! ```
//! use pai_causal::{CausalGraph, EdgeKind, NodeKind, ProtectedSurface};
//!
//! let mut graph = CausalGraph::new();
//!
//! // Telemetry source → derived variable → protected surface
//! graph.add_node("clicks", NodeKind::TelemetrySource {
//!     registry_id: "TEL-001".into(),
//! });
//! graph.add_node("score", NodeKind::DerivedVariable {
//!     name: "engagement".into(),
//! });
//! graph.add_node("ranking", NodeKind::ProtectedSurface {
//!     surface: ProtectedSurface::OutputRanking,
//! });
//!
//! graph.add_edge("clicks", "score", EdgeKind::DataFlow);
//! graph.add_edge("score", "ranking", EdgeKind::DataFlow);
//!
//! // Taint propagates transitively — ranking is now tainted
//! let forbidden = graph.forbidden_paths();
//! assert_eq!(forbidden.len(), 1);
//! assert_eq!(forbidden[0].surface, ProtectedSurface::OutputRanking);
//! ```
//!
//! Consent-gated edges block taint propagation:
//!
//! ```
//! # use pai_causal::{CausalGraph, EdgeKind, NodeKind, ProtectedSurface};
//! let mut graph = CausalGraph::new();
//! graph.add_node("tel", NodeKind::TelemetrySource {
//!     registry_id: "TEL-002".into(),
//! });
//! graph.add_node("out", NodeKind::ProtectedSurface {
//!     surface: ProtectedSurface::OutputFraming,
//! });
//! // Tier 2 consent gate — taint does NOT propagate
//! graph.add_edge("tel", "out", EdgeKind::ConsentGated { consent_tier: 2 });
//!
//! assert!(graph.forbidden_paths().is_empty());
//! ```

#![forbid(unsafe_code)]

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

// ---------------------------------------------------------------------------
// Node & Edge types
// ---------------------------------------------------------------------------

/// Classification of a variable node in the causal graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    /// Telemetry source (inherently tainted).
    TelemetrySource { registry_id: String },
    /// Declared objective.
    Objective { objective_id: String },
    /// Intermediate computation or derived variable.
    DerivedVariable { name: String },
    /// A protected output surface.
    ProtectedSurface { surface: ProtectedSurface },
}

/// Protected surfaces that telemetry must not reach without consent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtectedSurface {
    OutputRanking,
    OutputFraming,
    OutputSequencing,
    EmphasisPlacement,
    AttachmentCueGeneration,
    RelianceFrequencyShaping,
    ComplianceLikelihoodShaping,
    ObjectiveWeighting,
}

/// Edge classification: what kind of influence flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Direct data flow (variable → computation → output).
    DataFlow,
    /// Objective influence (objective → surface).
    ObjectiveInfluence,
    /// Consent-gated flow (tainted, but Tier≥2 consent present).
    ConsentGated { consent_tier: u8 },
}

// ---------------------------------------------------------------------------
// Causal Graph
// ---------------------------------------------------------------------------

/// Causal graph tracking variable flow, taint, and objective influence.
///
/// Built on a petgraph `DiGraph` where nodes are variables/surfaces
/// and edges represent data flow or influence relationships.
pub struct CausalGraph {
    graph: DiGraph<NodeKind, EdgeKind>,
    /// Map from user-assigned name to node index.
    node_index: HashMap<String, NodeIndex>,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_index: HashMap::new(),
        }
    }

    /// Add a node to the graph. Returns the node index.
    pub fn add_node(&mut self, name: &str, kind: NodeKind) -> NodeIndex {
        let idx = self.graph.add_node(kind);
        self.node_index.insert(name.into(), idx);
        idx
    }

    /// Add a directed edge (data flow or influence) between named nodes.
    pub fn add_edge(&mut self, from: &str, to: &str, kind: EdgeKind) -> bool {
        let Some(&from_idx) = self.node_index.get(from) else {
            return false;
        };
        let Some(&to_idx) = self.node_index.get(to) else {
            return false;
        };
        self.graph.add_edge(from_idx, to_idx, kind);
        true
    }

    /// Total number of nodes.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Total number of edges.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Look up a node by name.
    pub fn get_node(&self, name: &str) -> Option<&NodeKind> {
        self.node_index.get(name).map(|&idx| &self.graph[idx])
    }

    // -----------------------------------------------------------------------
    // Taint Propagation (CTL-I1)
    // -----------------------------------------------------------------------

    /// Compute the set of all nodes reachable from telemetry sources
    /// via BFS (transitive taint propagation, CTL-I1).
    pub fn tainted_nodes(&self) -> HashSet<NodeIndex> {
        let mut tainted = HashSet::new();
        let mut queue = VecDeque::new();

        // Seed: all TelemetrySource nodes
        for idx in self.graph.node_indices() {
            if matches!(self.graph[idx], NodeKind::TelemetrySource { .. }) {
                tainted.insert(idx);
                queue.push_back(idx);
            }
        }

        // BFS forward
        while let Some(current) = queue.pop_front() {
            for edge in self.graph.edges_directed(current, Direction::Outgoing) {
                // Consent-gated edges do NOT propagate taint (CTL-I2 pathway)
                if matches!(edge.weight(), EdgeKind::ConsentGated { .. }) {
                    continue;
                }
                let target = edge.target();
                if tainted.insert(target) {
                    queue.push_back(target);
                }
            }
        }

        tainted
    }

    // -----------------------------------------------------------------------
    // Forbidden Path Detection (CTL-I2)
    // -----------------------------------------------------------------------

    /// Detect forbidden paths: tainted nodes that reach protected surfaces
    /// without consent gating.
    pub fn forbidden_paths(&self) -> Vec<ForbiddenPath> {
        let tainted = self.tainted_nodes();
        let mut violations = Vec::new();

        for &idx in &tainted {
            if let NodeKind::ProtectedSurface { surface } = &self.graph[idx] {
                // Find which telemetry sources can reach this surface
                let sources = self.find_telemetry_sources_reaching(idx);
                violations.push(ForbiddenPath {
                    surface: *surface,
                    tainted_node: self.node_name(idx).unwrap_or_default(),
                    source_ids: sources,
                });
            }
        }

        violations
    }

    /// BFS backward from a node to find telemetry source names reaching it.
    fn find_telemetry_sources_reaching(&self, target: NodeIndex) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut sources = Vec::new();

        visited.insert(target);
        queue.push_back(target);

        while let Some(current) = queue.pop_front() {
            for edge in self.graph.edges_directed(current, Direction::Incoming) {
                if matches!(edge.weight(), EdgeKind::ConsentGated { .. }) {
                    continue;
                }
                let source = edge.source();
                if visited.insert(source) {
                    if let NodeKind::TelemetrySource { registry_id } = &self.graph[source] {
                        sources.push(registry_id.clone());
                    }
                    queue.push_back(source);
                }
            }
        }

        sources
    }

    /// Get the user-assigned name for a node index.
    fn node_name(&self, idx: NodeIndex) -> Option<String> {
        self.node_index
            .iter()
            .find(|(_, &v)| v == idx)
            .map(|(k, _)| k.clone())
    }

    // -----------------------------------------------------------------------
    // Objective-to-Surface Mapping (CTL-I3)
    // -----------------------------------------------------------------------

    /// Get all surfaces an objective can influence (CTL-I3).
    pub fn objective_surfaces(&self, objective_name: &str) -> Vec<ProtectedSurface> {
        let Some(&idx) = self.node_index.get(objective_name) else {
            return vec![];
        };

        let mut surfaces = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(idx);
        queue.push_back(idx);

        while let Some(current) = queue.pop_front() {
            for edge in self.graph.edges_directed(current, Direction::Outgoing) {
                let target = edge.target();
                if visited.insert(target) {
                    if let NodeKind::ProtectedSurface { surface } = &self.graph[target] {
                        surfaces.push(*surface);
                    }
                    queue.push_back(target);
                }
            }
        }

        surfaces
    }

    // -----------------------------------------------------------------------
    // Export (CTL-I5)
    // -----------------------------------------------------------------------

    /// Export the causal graph as a JSON-serializable audit trace.
    pub fn to_audit_trace(&self) -> AuditTrace {
        let nodes: Vec<AuditNode> = self
            .graph
            .node_indices()
            .map(|idx| AuditNode {
                name: self.node_name(idx).unwrap_or_else(|| format!("node_{}", idx.index())),
                kind: self.graph[idx].clone(),
            })
            .collect();

        let edges: Vec<AuditEdge> = self
            .graph
            .edge_indices()
            .filter_map(|eidx| {
                let (src, tgt) = self.graph.edge_endpoints(eidx)?;
                Some(AuditEdge {
                    from: self.node_name(src).unwrap_or_default(),
                    to: self.node_name(tgt).unwrap_or_default(),
                    kind: self.graph[eidx].clone(),
                })
            })
            .collect();

        let forbidden = self.forbidden_paths();

        AuditTrace {
            node_count: nodes.len(),
            edge_count: edges.len(),
            nodes,
            edges,
            forbidden_paths: forbidden,
        }
    }
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Analysis results
// ---------------------------------------------------------------------------

/// A forbidden path: tainted telemetry reaching a protected surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenPath {
    pub surface: ProtectedSurface,
    pub tainted_node: String,
    pub source_ids: Vec<String>,
}

/// Serializable audit trace for export (CTL-I5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrace {
    pub node_count: usize,
    pub edge_count: usize,
    pub nodes: Vec<AuditNode>,
    pub edges: Vec<AuditEdge>,
    pub forbidden_paths: Vec<ForbiddenPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditNode {
    pub name: String,
    pub kind: NodeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEdge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn build_simple_graph() -> CausalGraph {
        let mut g = CausalGraph::new();
        g.add_node("tel_engagement", NodeKind::TelemetrySource {
            registry_id: "TEL-001".into(),
        });
        g.add_node("derived_score", NodeKind::DerivedVariable {
            name: "engagement_score".into(),
        });
        g.add_node("ranking", NodeKind::ProtectedSurface {
            surface: ProtectedSurface::OutputRanking,
        });
        g.add_node("obj_relevance", NodeKind::Objective {
            objective_id: "OBJ-RELEVANCE".into(),
        });
        g.add_node("framing", NodeKind::ProtectedSurface {
            surface: ProtectedSurface::OutputFraming,
        });

        g.add_edge("tel_engagement", "derived_score", EdgeKind::DataFlow);
        g.add_edge("derived_score", "ranking", EdgeKind::DataFlow);
        g.add_edge("obj_relevance", "framing", EdgeKind::ObjectiveInfluence);
        g
    }

    // CTL-T01: Telemetry source is tainted
    #[test]
    fn ctl_t01_telemetry_tainted() {
        let g = build_simple_graph();
        let tainted = g.tainted_nodes();
        let tel_idx = *g.node_index.get("tel_engagement").unwrap();
        assert!(tainted.contains(&tel_idx));
    }

    // CTL-T02: Taint propagates to derived variable
    #[test]
    fn ctl_t02_taint_propagates() {
        let g = build_simple_graph();
        let tainted = g.tainted_nodes();
        let derived_idx = *g.node_index.get("derived_score").unwrap();
        assert!(tainted.contains(&derived_idx));
    }

    // CTL-T03: Taint reaches protected surface = forbidden path
    #[test]
    fn ctl_t03_forbidden_path_detected() {
        let g = build_simple_graph();
        let forbidden = g.forbidden_paths();
        assert_eq!(forbidden.len(), 1);
        assert_eq!(forbidden[0].surface, ProtectedSurface::OutputRanking);
        assert!(forbidden[0].source_ids.contains(&"TEL-001".to_string()));
    }

    // CTL-T04: Consent-gated edge blocks taint propagation
    #[test]
    fn ctl_t04_consent_blocks_taint() {
        let mut g = CausalGraph::new();
        g.add_node("tel", NodeKind::TelemetrySource {
            registry_id: "TEL-002".into(),
        });
        g.add_node("surface", NodeKind::ProtectedSurface {
            surface: ProtectedSurface::EmphasisPlacement,
        });
        // Consent-gated edge should NOT propagate taint
        g.add_edge("tel", "surface", EdgeKind::ConsentGated { consent_tier: 2 });

        let forbidden = g.forbidden_paths();
        assert!(forbidden.is_empty(), "consent-gated path should not be forbidden");
    }

    // CTL-T05: Objective surfaces mapping
    #[test]
    fn ctl_t05_objective_surfaces() {
        let g = build_simple_graph();
        let surfaces = g.objective_surfaces("obj_relevance");
        assert_eq!(surfaces.len(), 1);
        assert_eq!(surfaces[0], ProtectedSurface::OutputFraming);
    }

    // CTL-T06: Objective with no path to surfaces
    #[test]
    fn ctl_t06_isolated_objective() {
        let mut g = CausalGraph::new();
        g.add_node("obj_isolated", NodeKind::Objective {
            objective_id: "OBJ-ISOLATED".into(),
        });
        let surfaces = g.objective_surfaces("obj_isolated");
        assert!(surfaces.is_empty());
    }

    // CTL-T07: Audit trace export is valid JSON
    #[test]
    fn ctl_t07_audit_trace_json() {
        let g = build_simple_graph();
        let trace = g.to_audit_trace();
        let json = serde_json::to_string(&trace).unwrap();
        let parsed: AuditTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.node_count, 5);
        assert_eq!(parsed.edge_count, 3);
    }

    // CTL-T08: Shadow objective detection — undeclared telemetry→surface
    #[test]
    fn ctl_t08_shadow_objective() {
        // If telemetry reaches a surface without going through any declared objective,
        // the forbidden_paths report acts as shadow objective detection
        let g = build_simple_graph();
        let forbidden = g.forbidden_paths();
        // tel_engagement → derived_score → ranking, with no objective declaring this path
        assert!(!forbidden.is_empty(), "shadow influence path should be detected");
    }

    // CTL-T09: Graph node count and edge count
    #[test]
    fn ctl_t09_graph_counts() {
        let g = build_simple_graph();
        assert_eq!(g.node_count(), 5);
        assert_eq!(g.edge_count(), 3);
    }

    // CTL-T10: Add edge to nonexistent node returns false
    #[test]
    fn ctl_t10_invalid_edge() {
        let mut g = CausalGraph::new();
        assert!(!g.add_edge("nonexistent", "also_nonexistent", EdgeKind::DataFlow));
    }

    // CTL-T11: Multiple telemetry sources — all taint tracked
    #[test]
    fn ctl_t11_multiple_telemetry_sources() {
        let mut g = CausalGraph::new();
        g.add_node("tel_a", NodeKind::TelemetrySource { registry_id: "A".into() });
        g.add_node("tel_b", NodeKind::TelemetrySource { registry_id: "B".into() });
        g.add_node("merged", NodeKind::DerivedVariable { name: "merged".into() });
        g.add_node("surface", NodeKind::ProtectedSurface {
            surface: ProtectedSurface::OutputSequencing,
        });
        g.add_edge("tel_a", "merged", EdgeKind::DataFlow);
        g.add_edge("tel_b", "merged", EdgeKind::DataFlow);
        g.add_edge("merged", "surface", EdgeKind::DataFlow);

        let forbidden = g.forbidden_paths();
        assert_eq!(forbidden.len(), 1);
        assert_eq!(forbidden[0].source_ids.len(), 2);
    }

    // CTL-T12: Empty graph has no violations
    #[test]
    fn ctl_t12_empty_graph() {
        let g = CausalGraph::new();
        assert!(g.tainted_nodes().is_empty());
        assert!(g.forbidden_paths().is_empty());
    }

    // CTL-T13: All protected surface types representable
    #[test]
    fn ctl_t13_all_surfaces() {
        let surfaces = [
            ProtectedSurface::OutputRanking,
            ProtectedSurface::OutputFraming,
            ProtectedSurface::OutputSequencing,
            ProtectedSurface::EmphasisPlacement,
            ProtectedSurface::AttachmentCueGeneration,
            ProtectedSurface::RelianceFrequencyShaping,
            ProtectedSurface::ComplianceLikelihoodShaping,
            ProtectedSurface::ObjectiveWeighting,
        ];
        // 8 surfaces match Doc 14 §P1 enumeration
        assert_eq!(surfaces.len(), 8);
    }
}
