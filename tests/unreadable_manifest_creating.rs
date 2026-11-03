//! The creating commands stop when the manifest that governs the cwd did not
//! load (kos#168).
//!
//! #166 made `kos validate` fail on a `kos.yaml` that exists but does not
//! parse. The creating commands resolve their graph through the same
//! discovery, so in a repo with such a manifest they fell through to the
//! parent orchestrator graph: `kos id` minted an id with the parent's prefix
//! and `kos idea` and `kos finding` wrote into the parent's `_kos/`.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// An orchestrator graph, a repo with a malformed manifest (`broken`), and a
/// repo with a healthy one (`ok`).
fn fixture(root: &Path) {
    // The markers `Workspace::discover` reads as an orchestrator root. Without
    // them it stops at the first `_kos/kos.yaml` it meets and never sees the
    // parent graph.
    write(&root.join("charter.md"), "# charter\n");
    fs::create_dir_all(root.join("kos")).unwrap();
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: parentorc\nscope: orchestrator\nschema_version: '0.3'\nincludes:\n- path: ok/_kos\n- path: broken/_kos\n",
    );
    fs::create_dir_all(root.join("_kos/findings")).unwrap();
    fs::create_dir_all(root.join("_kos/ideas")).unwrap();
    write(
        &root.join("ok/_kos/kos.yaml"),
        "graph_id: okrepo\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &root.join("broken/_kos/kos.yaml"),
        "graph_id: [unclosed\nscope: repo\n",
    );
    fs::create_dir_all(root.join("broken/src")).unwrap();
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
        stderr.contains("broken/_kos/kos.yaml"),
        "stderr should name the unreadable manifest: {stderr}"
    );
    assert!(
        stderr.contains("hint:"),
        "stderr should say how to fix it: {stderr}"
    );
}

fn entries(dir: &Path) -> usize {
    fs::read_dir(dir).map_or(0, Iterator::count)
}

#[test]
fn id_refuses_and_mints_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let out = kos(&tmp.path().join("broken"), &["id", "finding", "x"]);

    refused(&out);
    assert!(
        out.stdout.is_empty(),
        "no id should be printed: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn idea_refuses_and_writes_nothing_to_the_parent() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    refused(&kos(&tmp.path().join("broken"), &["idea", "an-idea"]));

    assert_eq!(entries(&tmp.path().join("_kos/ideas")), 0);
}

#[test]
fn finding_refuses_and_writes_nothing_to_the_parent() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    refused(&kos(
        &tmp.path().join("broken"),
        &["finding", "a-slug", "a title"],
    ));

    assert_eq!(entries(&tmp.path().join("_kos/findings")), 0);
}

#[test]
fn question_refuses_and_writes_nothing_to_the_parent() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    refused(&kos(
        &tmp.path().join("broken"),
        &["question", "a-slug", "a title"],
    ));

    assert!(!tmp.path().join("_kos/nodes").exists());
}

#[test]
fn probe_refuses_and_writes_nothing_to_the_parent() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    refused(&kos(
        &tmp.path().join("broken"),
        &["probe", "a-slug", "a title"],
    ));

    assert!(!tmp.path().join("_kos/probes").exists());
}

#[test]
fn promote_and_compact_refuse_too() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    write(
        &tmp.path().join("_kos/nodes/frontier/question-a.yaml"),
        "id: question-a\ntype: question\nconfidence: frontier\ntitle: \"a\"\ncontent: \"a long enough body of text\"\n",
    );
    fs::create_dir_all(tmp.path().join("_kos/nodes/bedrock")).unwrap();
    let summary = tmp.path().join("summary.txt");
    write(&summary, "short\n");

    refused(&kos(
        &tmp.path().join("broken"),
        &["promote", "question-a", "bedrock"],
    ));
    refused(&kos(
        &tmp.path().join("broken"),
        &[
            "compact",
            "--node",
            "question-a",
            "--summary",
            summary.to_str().unwrap(),
        ],
    ));

    assert!(
        tmp.path()
            .join("_kos/nodes/frontier/question-a.yaml")
            .is_file()
    );
    assert!(!tmp.path().join("_kos/snapshots").exists());
}

#[test]
fn refuses_from_a_subdirectory_of_the_broken_repo() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    refused(&kos(&tmp.path().join("broken/src"), &["idea", "an-idea"]));

    assert_eq!(entries(&tmp.path().join("_kos/ideas")), 0);
}

#[test]
fn a_healthy_repo_still_creates_in_its_own_graph() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let out = kos(&tmp.path().join("ok"), &["idea", "an-idea"]);

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(tmp.path().join("ok/_kos/ideas/an-idea.md").is_file());
    assert_eq!(entries(&tmp.path().join("_kos/ideas")), 0);
}
