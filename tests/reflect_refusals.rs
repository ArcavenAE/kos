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

/// Run git in `dir` with an isolated identity, every commit signed by a
/// throwaway ssh key beside `dir`, and the repository-local variables dropped
/// so a GIT_DIR inherited from a hook cannot redirect the commit.
fn git(dir: &Path, key: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=kos-test",
            "-c",
            "user.email=kos-test@example.com",
            "-c",
            "init.defaultBranch=main",
            "-c",
            "gpg.format=ssh",
            "-c",
            "gpg.ssh.program=ssh-keygen",
            "-c",
            "commit.gpgsign=true",
            "-c",
        ])
        .arg(format!("user.signingkey={}", key.display()))
        .args(args)
        .status()
        .expect("git runnable");
    assert!(status.success(), "git {args:?} failed");
}

#[test]
fn reflect_in_a_healthy_repo_reports_that_repos_own_graph() {
    let tmp = tempfile::tempdir().unwrap();
    fixture(tmp.path());
    let ok = tmp.path().join("ok");

    // A node only the parent graph has, and a node only `ok/` has, committed
    // so the report has something to show for `ok/` and nothing for the parent.
    write(
        &tmp.path().join("_kos/nodes/frontier/question-parent.yaml"),
        "id: question-parent\ntype: question\nconfidence: frontier\ntitle: \"parent node\"\ncontent: \"p\"\n",
    );
    write(
        &ok.join("_kos/nodes/frontier/question-ok.yaml"),
        "id: question-ok\ntype: question\nconfidence: frontier\ntitle: \"ok node\"\ncontent: \"o\"\n",
    );
    let key = tmp.path().join("signing-key");
    let keygen = Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "kos-test", "-f"])
        .arg(&key)
        .status()
        .expect("ssh-keygen runnable");
    assert!(keygen.success(), "ssh-keygen failed");
    git(&ok, &key, &["init", "-q"]);
    git(&ok, &key, &["add", "-A"]);
    git(&ok, &key, &["commit", "-q", "-m", "chore: seed ok"]);

    let out = kos(&ok, &["reflect"]);

    let err = stderr(&out);
    assert!(out.status.success(), "stderr: {err}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("=== kos reflect ===") && stdout.contains("Range:"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("question-ok"),
        "the report should list ok/'s own node under Confidence changes: {stdout}"
    );
    assert!(
        !stdout.contains("question-parent"),
        "the report must not carry the parent graph's node: {stdout}"
    );
    assert!(!err.contains("has no kos.yaml manifest"), "stderr: {err}");
    assert!(!err.contains("manifest error"), "stderr: {err}");
}
