//! Round trips on the YAML shapes kos reads and writes, pinned when the YAML
//! crate moved from serde_yaml to serde-saphyr (RUSTSEC-2024-0449). A change of
//! parser must not change what a node, a manifest or a finding fallback means.

use std::fs;
use std::path::Path;

use kos::findings::{load_finding_file, load_manifest};
use kos::model::{Confidence, GraphScope, NodeType};
use kos::{init, validate};

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn init_writes_a_manifest_that_loads_back_the_same() {
    let tmp = tempfile::tempdir().unwrap();
    init::run(
        tmp.path(),
        &GraphScope::Repo,
        Some("roundtrip".into()),
        false,
    )
    .unwrap();

    let manifest = load_manifest(&tmp.path().join("_kos")).expect("written manifest must parse");
    assert_eq!(manifest.graph_id, "roundtrip");
    assert_eq!(manifest.scope, GraphScope::Repo);
    assert_eq!(manifest.schema_version, "0.3");
    assert!(manifest.includes.is_empty());
    assert!(
        manifest.findings.is_none(),
        "None must stay absent, not null"
    );

    let text = fs::read_to_string(tmp.path().join("_kos/kos.yaml")).unwrap();
    assert!(
        !text.contains("findings"),
        "skip_serializing_if must hold: {text}"
    );
}

#[test]
fn manifest_with_includes_and_a_findings_policy_parses() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        &tmp.path().join("_kos/kos.yaml"),
        "graph_id: orc\nscope: orchestrator\ndescription: \"a: b # not a comment\"\n\
         schema_version: '0.3'\nincludes:\n  - path: sub/_kos\n    graph_id: sub\n\
         findings:\n  numbered_through: 211\n",
    );
    let m = load_manifest(&tmp.path().join("_kos")).expect("manifest parses");
    assert_eq!(m.scope, GraphScope::Orchestrator);
    assert_eq!(m.description.as_deref(), Some("a: b # not a comment"));
    assert_eq!(m.includes.len(), 1);
    assert_eq!(m.findings.unwrap().numbered_through, Some(211));
}

#[test]
fn node_with_block_scalars_flow_lists_and_quoted_values_loads() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        &tmp.path().join("_kos/kos.yaml"),
        "graph_id: fixture\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &tmp.path().join("_kos/nodes/frontier/q-shape.yaml"),
        "id: q-shape\ntype: question\nconfidence: frontier\n\
         title: \"Is 'yes' a string? 1.10 vs 1.1\"\n\
         content: |\n  line one\n  line two: with a colon\n\
         tags: [alpha, \"beta gamma\"]\n\
         edges:\n  - {target: q-shape, type: derives}\n\
         notes: >\n  folded\n  text\n",
    );
    let summary = validate::run(&tmp.path().join("_kos")).unwrap();
    assert_eq!(summary.total, 1);
    assert_eq!(summary.parse_errors, 0);
}

#[test]
fn malformed_yaml_is_a_parse_error_not_a_dropped_node() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        &tmp.path().join("_kos/kos.yaml"),
        "graph_id: fixture\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &tmp.path().join("_kos/nodes/bedrock/bad.yaml"),
        "id: bad\ntitle: [unclosed\n",
    );
    let summary = validate::run(&tmp.path().join("_kos")).unwrap();
    assert_eq!(summary.parse_errors, 1);
}

#[test]
fn finding_with_a_prose_block_keeps_the_block_as_searchable_content_in_order() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("finding-zz-1-shape.yaml");
    write(
        &path,
        "id: finding-zz-1-shape\ntitle: Shape\nfinding:\n  zebra: last in the alphabet\n  alpha: first in the alphabet\n  count: 3\n",
    );
    let node = load_finding_file(&path).expect("fallback must load the finding");
    assert_eq!(node.id, "finding-zz-1-shape");
    assert_eq!(node.title, "Shape");
    assert_eq!(node.node_type, NodeType::Finding);
    assert_eq!(node.confidence, Confidence::Frontier);
    let z = node.content.find("zebra").expect("zebra in content");
    let a = node.content.find("alpha").expect("alpha in content");
    assert!(z < a, "source order is kept: {}", node.content);
    assert!(node.content.contains("count"), "{}", node.content);
}
