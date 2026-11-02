//! `kos promote` and `kos compact` refuse a bare `_kos/` (kos#167).
//!
//! A `_kos/` with no `kos.yaml` is not a graph. #163 and #165 made the
//! creating commands refuse it. `promote` and `compact` each kept a private
//! copy of the resolver, so run from a bare repo they moved or rewrote the
//! parent graph's nodes with exit 0.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn node(id: &str, body: &str) -> String {
    format!(
        "id: {id}\ntype: question\nconfidence: frontier\ntitle: \"{id}\"\ncontent: \"{body}\"\n"
    )
}

/// An orchestrator graph holding two frontier nodes, a repo graph with its
/// own manifest and node (`ok`), and a repo whose `_kos/` has no manifest
/// (`bare`).
fn fixture(root: &Path) {
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: parentorc\nscope: orchestrator\nschema_version: '0.3'\nincludes:\n- path: ok/_kos\n",
    );
    write(
        &root.join("_kos/nodes/frontier/question-a.yaml"),
        &node("question-a", "a body"),
    );
    write(
        &root.join("_kos/nodes/frontier/question-b.yaml"),
        &node(
            "question-b",
            "a much longer body of text that is larger than the summary so compaction proceeds",
        ),
    );
    write(
        &root.join("ok/_kos/kos.yaml"),
        "graph_id: okrepo\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &root.join("ok/_kos/nodes/frontier/question-c.yaml"),
        &node("question-c", "c body"),
    );
    // promote moves a file into an existing tier directory.
    fs::create_dir_all(root.join("_kos/nodes/bedrock")).unwrap();
    fs::create_dir_all(root.join("ok/_kos/nodes/bedrock")).unwrap();
    fs::create_dir_all(root.join("bare/_kos")).unwrap();
    fs::create_dir_all(root.join("bare/src")).unwrap();
    write(&root.join("summary.txt"), "short summary\n");
}

fn kos(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run kos")
}

fn refused(out: &Output) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "expected a refusal, got success");
    assert!(
        stderr.contains("has no kos.yaml manifest"),
        "stderr should name the missing manifest: {stderr}"
    );
}

#[test]
fn promote_refuses_a_bare_kos_dir_and_moves_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    refused(&kos(
        &tmp.path().join("bare"),
        &["promote", "question-a", "bedrock"],
    ));

    assert!(
        tmp.path()
            .join("_kos/nodes/frontier/question-a.yaml")
            .is_file()
    );
    assert!(
        !tmp.path()
            .join("_kos/nodes/bedrock/question-a.yaml")
            .exists()
    );
}

#[test]
fn promote_refuses_from_a_subdirectory_of_a_bare_repo() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    refused(&kos(
        &tmp.path().join("bare/src"),
        &["promote", "question-a", "bedrock"],
    ));

    assert!(
        tmp.path()
            .join("_kos/nodes/frontier/question-a.yaml")
            .is_file()
    );
}

#[test]
fn compact_refuses_a_bare_kos_dir_and_rewrites_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    let before =
        fs::read_to_string(tmp.path().join("_kos/nodes/frontier/question-b.yaml")).unwrap();
    let summary = tmp.path().join("summary.txt");

    refused(&kos(
        &tmp.path().join("bare"),
        &[
            "compact",
            "--node",
            "question-b",
            "--summary",
            summary.to_str().unwrap(),
        ],
    ));

    let after = fs::read_to_string(tmp.path().join("_kos/nodes/frontier/question-b.yaml")).unwrap();
    assert_eq!(before, after, "the parent's node must be untouched");
    assert!(!tmp.path().join("_kos/snapshots").exists());
}

#[test]
fn compact_listing_refuses_a_bare_kos_dir() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    refused(&kos(&tmp.path().join("bare"), &["compact"]));
}

#[test]
fn promote_still_works_in_a_repo_with_its_own_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let out = kos(
        &tmp.path().join("ok"),
        &["promote", "question-c", "bedrock"],
    );

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        tmp.path()
            .join("ok/_kos/nodes/bedrock/question-c.yaml")
            .is_file()
    );
    assert!(
        tmp.path()
            .join("_kos/nodes/frontier/question-a.yaml")
            .is_file()
    );
}

#[test]
fn compact_still_works_in_a_repo_with_its_own_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    write(
        &tmp.path().join("ok/_kos/nodes/frontier/question-d.yaml"),
        &node(
            "question-d",
            "another much longer body of text that is larger than the summary so compaction proceeds",
        ),
    );
    let summary = tmp.path().join("summary.txt");

    let out = kos(
        &tmp.path().join("ok"),
        &[
            "compact",
            "--node",
            "question-d",
            "--summary",
            summary.to_str().unwrap(),
        ],
    );

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        tmp.path()
            .join("ok/_kos/snapshots/question-d.yaml")
            .is_file()
    );
}
