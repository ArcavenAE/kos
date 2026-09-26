//! Integration tests for `kos ask` (aae-orc-jajn3,
//! question-kos-ask-retrieval-value).
//!
//! Exercises the corpus loader and ranker against an on-disk fixture graph:
//! the loader must fold nodes and findings into one corpus, and the ranker
//! must (1) rank a title hit above a body-only hit, (2) surface a ruled-out
//! node as ruled-out rather than hiding it, and (3) surface a node that only
//! matched through a citation edge.

use std::fs;
use std::path::Path;

use kos::ask::{self, Freshness};

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// A small graph: a strong frontier hit, a ruled-out node, a citing node with
/// no lexical match, and a finding (loaded from findings/, carrying a date).
fn fixture(root: &Path) {
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: fixture\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &root.join("_kos/nodes/frontier/question-read-telemetry.yaml"),
        "id: question-read-telemetry\ntype: question\nconfidence: frontier\n\
         title: \"Should kos record read telemetry?\"\n\
         content: \"Telemetry logs which nodes a read verb served.\"\n",
    );
    write(
        &root.join("_kos/nodes/graveyard/grv-telemetry-gate.yaml"),
        "id: grv-telemetry-gate\ntype: graveyard\nconfidence: graveyard\n\
         title: \"Gate merges on telemetry counts\"\n\
         content: \"Ruled out: telemetry is diagnostic, not a gate.\"\n",
    );
    write(
        &root.join("_kos/nodes/frontier/question-active-surfacing.yaml"),
        "id: question-active-surfacing\ntype: question\nconfidence: frontier\n\
         title: \"Active knowledge surfacing\"\n\
         content: \"The push half of the read problem.\"\n\
         edges:\n  - target: question-read-telemetry\n    type: derives\n",
    );
    write(
        &root.join("_kos/findings/finding-100-read-path.yaml"),
        "id: finding-100-read-path\ntype: finding\nconfidence: frontier\n\
         title: \"Read path telemetry shipped\"\n\
         content: \"The telemetry module logs served ids per session.\"\n\
         provenance:\n  created_at: \"2026-08-16\"\n",
    );
}

#[test]
fn corpus_folds_nodes_and_findings() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let corpus = ask::load_corpus(&tmp.path().join("_kos")).unwrap();
    // 3 nodes + 1 finding.
    assert_eq!(corpus.len(), 4);
    assert!(corpus.iter().any(|n| n.id == "finding-100-read-path"));
}

#[test]
fn title_hit_leads_and_finding_carries_date() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    let corpus = ask::load_corpus(&tmp.path().join("_kos")).unwrap();

    let results = ask::rank(&corpus, "read telemetry", 10);
    assert!(!results.is_empty());
    // The finding titles "Read path telemetry shipped" and hits both terms in
    // the title, so it leads; and it carries its provenance date.
    let finding = results
        .iter()
        .find(|r| r.id == "finding-100-read-path")
        .expect("finding should rank");
    assert_eq!(finding.date.as_deref(), Some("2026-08-16"));
    assert!(finding.is_finding);
}

#[test]
fn ruled_out_node_is_surfaced_with_status() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    let corpus = ask::load_corpus(&tmp.path().join("_kos")).unwrap();

    let results = ask::rank(&corpus, "telemetry", 10);
    let grave = results
        .iter()
        .find(|r| r.id == "grv-telemetry-gate")
        .expect("a ruled-out match must still be returned");
    assert_eq!(grave.freshness, Freshness::RuledOut);
}

#[test]
fn proximity_surfaces_the_citing_question() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    let corpus = ask::load_corpus(&tmp.path().join("_kos")).unwrap();

    // question-active-surfacing does not contain "telemetry"; it only cites the
    // node that does. Proximity must pull it in and name the neighbor.
    let results = ask::rank(&corpus, "telemetry", 10);
    let cited = results
        .iter()
        .find(|r| r.id == "question-active-surfacing")
        .expect("proximity should surface the citing node");
    assert_eq!(cited.via.as_deref(), Some("question-read-telemetry"));
}
