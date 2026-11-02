//! Commands that create content refuse a bare `_kos/` (kos#163).
//!
//! A `_kos/` with no `kos.yaml` is not a graph. `kos validate` already refuses
//! it, because walking up would report a parent graph's result as this repo's.
//! `kos id` and the other creating commands resolved past it to the
//! orchestrator graph, so the id (or file) landed under the wrong graph with
//! exit 0.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// An orchestrator graph, one repo graph with a manifest (`ok`), one repo whose
/// `_kos/` has no manifest (`bare`), and one repo with no `_kos/` at all
/// (`plain`).
fn fixture(root: &Path) {
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: parentorc\nscope: orchestrator\nschema_version: '0.3'\nincludes:\n- path: ok/_kos\n",
    );
    write(
        &root.join("ok/_kos/kos.yaml"),
        "graph_id: okrepo\nscope: repo\nschema_version: '0.3'\n",
    );
    fs::create_dir_all(root.join("bare/_kos/findings")).unwrap();
    fs::create_dir_all(root.join("bare/src")).unwrap();
    fs::create_dir_all(root.join("plain")).unwrap();
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
    assert!(
        out.stdout.is_empty(),
        "nothing should print on stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn id_refuses_a_bare_kos_dir() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    refused(&kos(&tmp.path().join("bare"), &["id", "finding", "x"]));
}

#[test]
fn id_refuses_from_a_subdirectory_of_a_bare_repo() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    refused(&kos(&tmp.path().join("bare/src"), &["id", "finding", "x"]));
}

#[test]
fn creating_commands_refuse_and_write_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    let bare = tmp.path().join("bare");

    for args in [
        vec!["idea", "x"],
        vec!["question", "x", "t"],
        vec!["finding", "x", "t"],
        vec!["probe", "x", "t"],
    ] {
        refused(&kos(&bare, &args));
    }

    for dir in ["ideas", "nodes", "findings", "probes"] {
        let created = fs::read_dir(tmp.path().join("_kos").join(dir))
            .map(Iterator::count)
            .unwrap_or(0);
        assert_eq!(created, 0, "parent graph gained files under _kos/{dir}");
    }
    assert_eq!(
        fs::read_dir(bare.join("_kos/findings")).unwrap().count(),
        0,
        "the bare repo gained a finding"
    );
}

#[test]
fn id_still_mints_in_a_repo_with_a_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    let out = kos(&tmp.path().join("ok"), &["id", "finding", "x"]);
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("finding-okrepo-"),
        "got {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn id_in_a_repo_with_no_kos_dir_still_inherits_the_parent_graph() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    let out = kos(&tmp.path().join("plain"), &["id", "finding", "x"]);
    assert!(out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("finding-parentorc-"),
        "got {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
