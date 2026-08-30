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

// ── aae-orc-5z4p / kos: a bare _kos/ must not silently validate a parent ──

/// A cwd whose `_kos/` exists but carries no `kos.yaml` must be REFUSED, not
/// silently attributed to the walked-up parent graph. Regression: callbook and
/// betterdials-site each have a `_kos/` with no manifest, and `kos validate`
/// there returned the aae-orc graph's clean result as if it were theirs.
#[test]
fn run_nearest_refuses_bare_kos_directory() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    // A subrepo-shaped dir with a _kos/ that has content but no manifest.
    write(
        &tmp.path().join("bare/_kos/findings/finding-001-orphan.md"),
        "# orphan finding\n\nbody\n",
    );

    let ws = Workspace::from_explicit(tmp.path()).unwrap();

    // Absent the guard, the walk-up would resolve `bare/` to the orchestrator
    // graph — the misattribution this test protects against.
    assert_eq!(
        ws.nearest_graph(&tmp.path().join("bare")).unwrap().graph_id,
        "fixture-orc",
        "precondition: nearest_graph would misattribute the parent graph"
    );

    match validate::run_nearest(&ws, &tmp.path().join("bare")).unwrap() {
        validate::ScopedValidation::BareKosDir(path) => {
            assert!(
                path.ends_with("bare/_kos"),
                "must name the offending directory, got {}",
                path.display()
            );
        }
        validate::ScopedValidation::Validated(s) => {
            panic!("bare _kos/ silently validated a parent graph: {s:?}");
        }
    }
}

/// The refusal must fire from a SUBDIRECTORY of a bare-_kos repo too — the
/// axis a cwd-only check missed. `kos validate` in `callbook/src/` walks up
/// past `callbook/_kos` to the orchestrator graph without this.
#[test]
fn run_nearest_refuses_bare_kos_from_subdirectory() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    write(
        &tmp.path().join("bare/_kos/findings/finding-001-orphan.md"),
        "# orphan finding\n\nbody\n",
    );
    // A nested working directory with no _kos/ of its own.
    std::fs::create_dir_all(tmp.path().join("bare/src/deep")).unwrap();

    let ws = Workspace::from_explicit(tmp.path()).unwrap();
    match validate::run_nearest(&ws, &tmp.path().join("bare/src/deep")).unwrap() {
        validate::ScopedValidation::BareKosDir(path) => {
            assert!(path.ends_with("bare/_kos"), "got {}", path.display());
        }
        validate::ScopedValidation::Validated(s) => {
            panic!("bare _kos/ misattributed from a subdirectory: {s:?}");
        }
    }
}

/// A cwd with a real graph (manifest present) validates that graph as before.
#[test]
fn run_nearest_validates_real_graph() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let ws = Workspace::from_explicit(tmp.path()).unwrap();
    match validate::run_nearest(&ws, &tmp.path().join("sub")).unwrap() {
        validate::ScopedValidation::Validated(s) => {
            assert_eq!(s.total, 3, "the sub graph has three nodes");
        }
        validate::ScopedValidation::BareKosDir(p) => {
            panic!("a real graph was refused as bare: {}", p.display());
        }
    }
}
