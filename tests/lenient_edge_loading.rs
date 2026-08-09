//! Integration tests for lenient edge loading (aae-orc-znzh,
//! brief-lenient-edge-loading, question-edge-vocabulary-gap sub-question c).
//!
//! Regression under test: an edge type outside the schema vocabulary used to
//! fail the whole-document parse, so every read path skipped the entire NODE
//! and warned only on stderr. marvel ran two months with 14 of 25 nodes
//! invisible to orient; switchboard lost all 7 of its bedrock elements.
//!
//! The contract is tolerant reader, strict writer: readers load the node and
//! preserve the unknown type verbatim, `kos validate` fails on it.

use std::fs;
use std::path::Path;

use kos::model::{EdgeType, Node};
use kos::validate;

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// A graph with one clean node and one carrying an invented edge type.
/// `related` is the real-world case: 50 instances across the fleet.
fn fixture(root: &Path) {
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: fixture\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &root.join("_kos/nodes/bedrock/elem-clean.yaml"),
        "id: elem-clean\ntype: element\nconfidence: bedrock\ntitle: \"t\"\ncontent: \"c\"\n",
    );
    write(
        &root.join("_kos/nodes/bedrock/elem-invented.yaml"),
        "id: elem-invented\ntype: element\nconfidence: bedrock\ntitle: \"t\"\ncontent: \"c\"\n\
         edges:\n  - target: elem-clean\n    type: related\n",
    );
}

#[test]
fn node_with_unknown_edge_type_still_loads() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let summary = validate::run(&tmp.path().join("_kos")).unwrap();

    // The whole point: the node is counted, not dropped. Before the fix this
    // was 1 node + 1 parse error, and the node was invisible to every reader.
    assert_eq!(summary.total, 2, "both nodes must be visible to the loader");
    assert_eq!(
        summary.parse_errors, 0,
        "an unknown edge type is not a parse error"
    );
}

#[test]
fn validate_fails_the_node_and_names_the_offending_value() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let summary = validate::run(&tmp.path().join("_kos")).unwrap();

    assert_eq!(summary.failed, 1, "the invented edge type must fail");
    assert_eq!(summary.passed, 1, "the clean node must still pass");
    assert!(
        !summary.clean(),
        "a graph with an unknown edge type is not clean, so exit is nonzero"
    );
}

#[test]
fn unknown_edge_type_preserves_the_author_string_verbatim() {
    // Verbatim preservation is what lets a later vocabulary migration read
    // what the author actually wrote instead of a lossy normalization.
    let yaml = "id: elem-x\ntype: element\nconfidence: bedrock\ntitle: \"t\"\ncontent: \"c\"\n\
                edges:\n  - target: elem-y\n    type: leaves-open\n";
    let node: Node = serde_yaml::from_str(yaml).unwrap();

    assert_eq!(node.edges.len(), 1);
    assert_eq!(node.edges[0].edge_type.unknown_value(), Some("leaves-open"));
    assert_eq!(node.edges[0].edge_type.to_string(), "leaves-open");
}

#[test]
fn unknown_edge_type_is_never_blocking() {
    // An edge we cannot interpret must not silently evict its source node
    // from the ready queue. Marvel manufactured two permanent evictions this
    // way with a legal-but-wrong retype (aae-orc-1bte); an uninterpretable
    // edge must not be able to do it at all.
    let unknown = EdgeType::from("some-invented-thing".to_string());
    assert!(!unknown.is_blocking());

    assert!(EdgeType::from("derives".to_string()).is_blocking());
    assert!(EdgeType::from("implements".to_string()).is_blocking());
    assert!(!EdgeType::from("supports".to_string()).is_blocking());
}

#[test]
fn every_legal_edge_type_still_round_trips() {
    // Guard against the lenient path swallowing a legal type into Unknown,
    // which would silently disable blocking and drift propagation.
    for name in kos::model::LEGAL_EDGE_TYPES {
        let parsed = EdgeType::from(name.to_string());
        assert_eq!(
            parsed.unknown_value(),
            None,
            "{name} must parse as a known variant"
        );
        assert_eq!(
            parsed.to_string(),
            name,
            "{name} must render back as itself"
        );
    }
}
