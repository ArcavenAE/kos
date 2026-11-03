//! `kos reflect` resolves its graph like the other commands (kos#167, #168).
//!
//! It kept a private copy of the resolver that skipped the two refusals, so in
//! a repo whose `_kos/` has no manifest, or whose manifest does not parse, it
//! reported on the parent orchestrator graph and attributed the result to the
//! wrong repo.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// An orchestrator (with the markers `Workspace::discover` reads), a repo with
/// a healthy manifest (`ok`), a bare `_kos/` (`bare`) and a repo whose
/// manifest does not parse (`broken`).
fn fixture(root: &Path) {
    write(&root.join("charter.md"), "# charter\n");
    fs::create_dir_all(root.join("kos")).unwrap();
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: parentorc\nscope: orchestrator\nschema_version: '0.3'\nincludes:\n- path: ok/_kos\n- path: broken/_kos\n",
    );
    fs::create_dir_all(root.join("_kos/nodes/frontier")).unwrap();
    write(
        &root.join("ok/_kos/kos.yaml"),
        "graph_id: okrepo\nscope: repo\nschema_version: '0.3'\n",
    );
    fs::create_dir_all(root.join("ok/_kos/nodes/frontier")).unwrap();
    fs::create_dir_all(root.join("bare/_kos")).unwrap();
    write(
        &root.join("broken/_kos/kos.yaml"),
        "graph_id: [unclosed\nscope: repo\n",
    );
}

fn kos(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run kos")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn reflect_refuses_a_bare_kos_dir() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let out = kos(&tmp.path().join("bare"), &["reflect"]);

    assert!(!out.status.success(), "expected a refusal, got success");
    assert!(
        stderr(&out).contains("has no kos.yaml manifest"),
        "stderr: {}",
        stderr(&out)
    );
}

#[test]
fn reflect_refuses_a_manifest_that_does_not_parse() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let out = kos(&tmp.path().join("broken"), &["reflect"]);

    assert!(!out.status.success(), "expected a refusal, got success");
    assert!(
        stderr(&out).contains("broken/_kos/kos.yaml"),
        "stderr: {}",
        stderr(&out)
    );
}

#[test]
fn reflect_does_not_refuse_a_repo_with_a_healthy_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());

    let out = kos(&tmp.path().join("ok"), &["reflect"]);

    let err = stderr(&out);
    assert!(out.status.success(), "stderr: {err}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("=== kos reflect ===") && stdout.contains("Range:"),
        "stdout: {stdout}"
    );
    assert!(!err.contains("has no kos.yaml manifest"), "stderr: {err}");
    assert!(!err.contains("manifest error"), "stderr: {err}");
}
