//! Example: Causal telemetry graph — taint propagation & forbidden paths (Doc 14)
//!
//! Demonstrates building a causal graph that tracks how telemetry
//! flows through variables to protected surfaces, detecting violations.
//!
//! Run: cargo run -p pai_examples --bin causal_graph

use pai_causal::{CausalGraph, EdgeKind, NodeKind, ProtectedSurface};

fn main() {
    println!("=== PAI-CD Causal Telemetry Graph Example ===\n");

    let mut graph = CausalGraph::new();

    // Add telemetry sources (inherently tainted)
    graph.add_node("tel_clicks", NodeKind::TelemetrySource {
        registry_id: "TEL-CLICK-001".into(),
    });
    graph.add_node("tel_dwell", NodeKind::TelemetrySource {
        registry_id: "TEL-DWELL-002".into(),
    });

    // Add derived variables
    graph.add_node("engagement_score", NodeKind::DerivedVariable {
        name: "engagement_score".into(),
    });
    graph.add_node("relevance_score", NodeKind::DerivedVariable {
        name: "relevance_score".into(),
    });

    // Add declared objectives
    graph.add_node("obj_accuracy", NodeKind::Objective {
        objective_id: "OBJ-ACCURACY".into(),
    });

    // Add protected surfaces
    graph.add_node("ranking", NodeKind::ProtectedSurface {
        surface: ProtectedSurface::OutputRanking,
    });
    graph.add_node("framing", NodeKind::ProtectedSurface {
        surface: ProtectedSurface::OutputFraming,
    });

    // Data flow edges
    graph.add_edge("tel_clicks", "engagement_score", EdgeKind::DataFlow);
    graph.add_edge("tel_dwell", "engagement_score", EdgeKind::DataFlow);
    graph.add_edge("engagement_score", "ranking", EdgeKind::DataFlow); // FORBIDDEN!

    // Consent-gated telemetry use (allowed path)
    graph.add_edge("tel_clicks", "relevance_score", EdgeKind::ConsentGated { consent_tier: 2 });
    graph.add_edge("relevance_score", "framing", EdgeKind::DataFlow);

    // Declared objective influence (legitimate)
    graph.add_edge("obj_accuracy", "ranking", EdgeKind::ObjectiveInfluence);

    println!("Graph: {} nodes, {} edges\n", graph.node_count(), graph.edge_count());

    // Analyze taint propagation
    let tainted = graph.tainted_nodes();
    println!("Tainted nodes: {} (via transitive propagation)", tainted.len());

    // Detect forbidden paths
    let forbidden = graph.forbidden_paths();
    println!("Forbidden paths: {}", forbidden.len());
    for fp in &forbidden {
        println!("  VIOLATION: {:?} reached by telemetry sources {:?}",
            fp.surface, fp.source_ids);
    }

    // Check objective influence mapping
    let obj_surfaces = graph.objective_surfaces("obj_accuracy");
    println!("\nObjective OBJ-ACCURACY influences: {:?}", obj_surfaces);

    // Export audit trace
    let trace = graph.to_audit_trace();
    println!("\n--- Audit Trace (JSON) ---");
    println!("{}", serde_json::to_string_pretty(&trace).unwrap());
}
