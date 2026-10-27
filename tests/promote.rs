//! `kos promote <node-id> <tier>`, end to end (kos#147, the next stage of
//! idea cli-maturity-for-external-use). The command moves a node between
//! confidence tiers, rewrites its `confidence:`, runs the `kos validate` node
//! checks on the moved node before anything is written, and prints the commit
//! line from the kos commit conventions. A refused move changes no file.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

const ANCHOR: &str =
    "id: elem-anchor\ntype: element\nconfidence: bedrock\ntitle: anchor\ncontent: a body\n";

const QUESTION: &str = "id: question-x\ntype: question\nconfidence: frontier\ntitle: \"x\"\n\
     content: |\n  a body that mentions confidence: frontier in prose\n\
     edges:\n  - target: elem-anchor\n    type: derives\n";

/// A standalone graph with an anchor in bedrock and a question in frontier.
fn base(root: &Path) {
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: kos\nscope: repo\nschema_version: '0.3'\n",
    );
    write(&root.join("_kos/nodes/bedrock/elem-anchor.yaml"), ANCHOR);
    write(&root.join("_kos/nodes/frontier/question-x.yaml"), QUESTION);
}

fn node(root: &Path, tier: &str, id: &str) -> PathBuf {
    root.join("_kos/nodes")
        .join(tier)
        .join(format!("{id}.yaml"))
}

fn promote(root: &Path, id: &str, tier: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .args(["promote", id, tier, "--dir", root.to_str().unwrap()])
        .output()
        .expect("run kos promote")
}

fn text(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Every file under `_kos/`, with its contents, to show a refusal wrote nothing.
fn snapshot(root: &Path) -> Vec<(PathBuf, String)> {
    let mut v: Vec<(PathBuf, String)> = walkdir::WalkDir::new(root.join("_kos"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .map(|e| {
            (
                e.path().to_path_buf(),
                fs::read_to_string(e.path()).unwrap(),
            )
        })
        .collect();
    v.sort();
    v
}

fn assert_refused(root: &Path, id: &str, tier: &str, needle: &str) {
    let before = snapshot(root);
    let out = promote(root, id, tier);
    let all = text(&out);
    assert!(!out.status.success(), "must be refused:\n{all}");
    assert!(all.contains(needle), "refusal names '{needle}':\n{all}");
    assert_eq!(before, snapshot(root), "a refused move writes nothing");
}

#[test]
fn frontier_to_bedrock_moves_the_file_and_rewrites_confidence() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());

    let out = promote(tmp.path(), "question-x", "bedrock");
    let all = text(&out);
    assert!(out.status.success(), "{all}");

    assert!(!node(tmp.path(), "frontier", "question-x").exists());
    let moved = fs::read_to_string(node(tmp.path(), "bedrock", "question-x")).unwrap();
    // Only the top-level field changes; the same words in the body stay.
    assert_eq!(
        moved,
        QUESTION.replacen("confidence: frontier\n", "confidence: bedrock\n", 1)
    );
    assert!(all.contains("frontier -> bedrock"), "{all}");
    assert!(
        all.contains("promote: question-x \u{2014} "),
        "prints the kos commit line:\n{all}"
    );

    let v = Command::new(env!("CARGO_BIN_EXE_kos"))
        .arg("validate")
        .current_dir(tmp.path())
        .output()
        .expect("run kos validate");
    let stdout = String::from_utf8_lossy(&v.stdout);
    assert!(v.status.success(), "the moved graph validates:\n{stdout}");
    assert!(stdout.contains("0 failed"), "{stdout}");
}

#[test]
fn placeholder_to_frontier_is_an_upward_move() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    write(
        &node(tmp.path(), "placeholder", "elem-unknown"),
        "id: elem-unknown\ntype: element\nconfidence: placeholder\ntitle: u\ncontent: c\n",
    );
    let out = promote(tmp.path(), "elem-unknown", "frontier");
    assert!(out.status.success(), "{}", text(&out));
    assert!(node(tmp.path(), "frontier", "elem-unknown").exists());
    assert!(!node(tmp.path(), "placeholder", "elem-unknown").exists());
}

#[test]
fn a_missing_edge_target_warns_and_still_moves() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    write(
        &node(tmp.path(), "frontier", "question-y"),
        "id: question-y\ntype: question\nconfidence: frontier\ntitle: y\ncontent: c\n\
         edges:\n  - target: finding-elsewhere\n    type: derives\n",
    );
    let out = promote(tmp.path(), "question-y", "bedrock");
    let all = text(&out);
    assert!(out.status.success(), "{all}");
    assert!(
        all.contains("finding-elsewhere"),
        "the warning is shown:\n{all}"
    );
    assert!(node(tmp.path(), "bedrock", "question-y").exists());
}

#[test]
fn the_same_tier_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    assert_refused(tmp.path(), "question-x", "frontier", "already");
}

#[test]
fn a_demotion_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    assert_refused(tmp.path(), "elem-anchor", "frontier", "demot");
    assert_refused(tmp.path(), "question-x", "placeholder", "demot");
}

#[test]
fn a_move_out_of_graveyard_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    write(
        &node(tmp.path(), "graveyard", "grv-old"),
        "id: grv-old\ntype: element\nconfidence: graveyard\ntitle: old\ncontent: c\n",
    );
    for tier in ["frontier", "bedrock", "placeholder"] {
        assert_refused(tmp.path(), "grv-old", tier, "out of the graveyard");
    }
}

#[test]
fn a_move_into_graveyard_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    assert_refused(tmp.path(), "question-x", "graveyard", "not a promotion");
    assert_refused(tmp.path(), "elem-anchor", "graveyard", "not a promotion");
}

#[test]
fn an_unknown_id_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    assert_refused(tmp.path(), "question-nope", "bedrock", "question-nope");
}

#[test]
fn an_existing_file_at_the_target_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    // A stale copy already sits in bedrock: two files claim one id.
    write(&node(tmp.path(), "bedrock", "question-x"), QUESTION);
    assert_refused(tmp.path(), "question-x", "bedrock", "question-x");
}

#[test]
fn a_node_that_fails_validation_is_not_moved() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    write(
        &node(tmp.path(), "frontier", "question-bad"),
        "id: question-bad\ntype: question\nconfidence: frontier\ntitle: b\ncontent: c\n\
         edges:\n  - target: elem-anchor\n    type: leaves-open\n",
    );
    assert_refused(tmp.path(), "question-bad", "bedrock", "leaves-open");
}

#[test]
fn a_tier_that_does_not_exist_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    assert_refused(tmp.path(), "question-x", "granite", "granite");
}
