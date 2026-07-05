//! Integration tests for validate graph scoping (aae-orc-z67m / kos#54).
//!
//! Regression under test: `kos validate` previously resolved to the kos
//! repo's own graph from every cwd, so subrepo graphs were never
//! validated and fleet "0 failed" was coverage illusion (finding-060).

use std::fs;
use std::path::Path;

use kos::validate;
use kos::workspace::Workspace;

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn node_yaml(id: &str, confidence: &str) -> String {
    format!("id: {id}\ntype: element\nconfidence: {confidence}\ntitle: \"t\"\ncontent: \"c\"\n")
}

/// Build a fixture workspace: standalone-style root graph plus one
/// subrepo graph, with different node counts so scoping is observable.
fn fixture(root: &Path) {
    // Orchestrator-shaped root: _kos with 1 valid node
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: fixture-orc\nscope: orchestrator\nschema_version: '0.3'\nincludes:\n- path: sub/_kos\n",
    );
    write(
        &root.join("_kos/nodes/bedrock/elem-one.yaml"),
        &node_yaml("elem-one", "bedrock"),
    );

    // Subrepo graph: 2 valid nodes + 1 misfiled node (frontier file in bedrock/)
    write(
        &root.join("sub/_kos/kos.yaml"),
        "graph_id: fixture-sub\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &root.join("sub/_kos/nodes/frontier/question-a.yaml"),
        &node_yaml("question-a", "frontier"),
    );
    write(
        &root.join("sub/_kos/nodes/frontier/question-b.yaml"),
        &node_yaml("question-b", "frontier"),
    );
    write(
        &root.join("sub/_kos/nodes/bedrock/elem-misfiled.yaml"),
        &node_yaml("elem-misfiled", "frontier"),
    );
}

#[test]
fn nearest_graph_resolves_subrepo_graph_from_subrepo_cwd() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let ws = Workspace::from_explicit(tmp.path()).unwrap();
    assert_eq!(ws.graphs.len(), 2, "orc graph + included subrepo graph");

    let nearest = ws.nearest_graph(&tmp.path().join("sub")).unwrap();
    assert_eq!(nearest.graph_id, "fixture-sub");

    let at_root = ws.nearest_graph(tmp.path()).unwrap();
    assert_eq!(at_root.graph_id, "fixture-orc");
}

#[test]
fn validate_summaries_differ_per_graph() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let ws = Workspace::from_explicit(tmp.path()).unwrap();

    let orc = ws
        .graphs
        .iter()
        .find(|g| g.graph_id == "fixture-orc")
        .unwrap();
    let sub = ws
        .graphs
        .iter()
        .find(|g| g.graph_id == "fixture-sub")
        .unwrap();

    let orc_summary = validate::run(&orc.path).unwrap();
    assert_eq!(orc_summary.total, 1);
    assert_eq!(orc_summary.failed, 0);
    assert!(orc_summary.clean());

    let sub_summary = validate::run(&sub.path).unwrap();
    assert_eq!(sub_summary.total, 3);
    assert_eq!(
        sub_summary.failed, 1,
        "misfiled node (frontier confidence in bedrock/) must fail"
    );
    assert!(!sub_summary.clean());

    // The regression: identical summaries from every scope meant the
    // subrepo graph was never read. These must differ.
    assert_ne!(orc_summary.total, sub_summary.total);
}

#[test]
fn summary_merge_accumulates_across_graphs() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let ws = Workspace::from_explicit(tmp.path()).unwrap();
    let mut combined = validate::Summary::default();
    for g in &ws.graphs {
        combined.merge(&validate::run(&g.path).unwrap());
    }
    assert_eq!(combined.total, 4);
    assert_eq!(combined.failed, 1);
    assert!(
        !combined.clean(),
        "one failing graph must fail the merged run"
    );
}
