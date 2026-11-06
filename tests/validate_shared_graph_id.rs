//! Two roots that share a graph id are a note, not a failure (aae-orc-gtv04).
//!
//! `validate --merged` prints one `note: manifest ...` line per shared id and
//! exits 0 when nothing else is wrong. Adding a failure after the note, or
//! dropping the note, must fail this file.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const ORC: &str = "graph_id: orc\nscope: orchestrator\nschema_version: '0.3'\n";

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn repo_graph(id: &str) -> String {
    format!("graph_id: {id}\nscope: repo\nschema_version: '0.3'\n")
}

/// An orchestrator root holding a primary checkout `alpha` and a linked
/// worktree `a-wt` that both claim graph id `x`, and a lone graph `solo`.
fn workspace(share: bool) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(&r.join("charter.md"), "charter\n");
    fs::create_dir_all(r.join("kos")).unwrap();
    write(&r.join("_kos/kos.yaml"), ORC);
    write(&r.join("alpha/_kos/kos.yaml"), &repo_graph("x"));
    fs::create_dir_all(r.join("alpha/.git")).unwrap();
    write(
        &r.join("a-wt/_kos/kos.yaml"),
        &repo_graph(if share { "x" } else { "y" }),
    );
    write(&r.join("a-wt/.git"), "gitdir: elsewhere\n");
    tmp
}

fn validate_merged(cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .args(["validate", "--merged"])
        .current_dir(cwd)
        .env("HOME", cwd)
        .env_remove("KOS_WORKSPACE")
        .output()
        .expect("run kos")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn a_shared_id_is_one_note_and_the_run_still_exits_zero() {
    let tmp = workspace(true);
    let out = validate_merged(tmp.path());
    let err = stderr(&out);

    assert!(
        out.status.success(),
        "a shared id must not fail validate; stderr: {err}"
    );
    let notes: Vec<&str> = err
        .lines()
        .filter(|l| l.starts_with("note: manifest "))
        .collect();
    assert_eq!(notes.len(), 1, "one note per shared id; stderr: {err}");
    assert!(
        notes[0].contains("'x'"),
        "the note names the id: {}",
        notes[0]
    );
    assert!(
        notes[0].contains("alpha"),
        "the primary checkout is the root it uses: {}",
        notes[0]
    );
    assert!(
        !err.contains("error:"),
        "a shared id adds no error line; stderr: {err}"
    );
}

#[test]
fn with_no_shared_id_there_is_no_note() {
    let tmp = workspace(false);
    let out = validate_merged(tmp.path());
    let err = stderr(&out);

    assert!(out.status.success(), "stderr: {err}");
    assert!(
        !err.contains("note: manifest"),
        "no note without a shared id; stderr: {err}"
    );
}
