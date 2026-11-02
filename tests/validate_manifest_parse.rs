//! A `kos.yaml` that exists but does not parse must fail `kos validate`
//! with the manifest path and the parse error (kos#164).
//!
//! Regression under test: two readers turned a parse failure into `None`.
//! Graph discovery dropped the graph, so `validate` took its fallback branch
//! with no "Validating graph" header, and the findings pass read no manifest
//! at all. With nothing else wrong the run exited 0.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use kos::validate;

const MALFORMED: &str = "graph_id: fixture\nscope: repo\nincludes: [\n";
const VALID: &str = "graph_id: fixture\nscope: repo\nschema_version: '0.3'\nincludes: []\n";

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// A node whose only fault is an edge type the schema does not list.
fn bad_edge_node(root: &Path) {
    write(
        &root.join("_kos/nodes/frontier/q.yaml"),
        "id: q\ntype: question\nconfidence: frontier\ntitle: \"t\"\ncontent: \"c\"\nedges:\n  - target: finding-x\n    type: derives-from\n",
    );
}

/// Run the built `kos` in `cwd` with a scratch HOME.
fn kos(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .args(args)
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
fn malformed_manifest_with_nothing_else_wrong_fails_validate() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("_kos/kos.yaml"), MALFORMED);

    let out = kos(tmp.path(), &["validate"]);
    let err = stderr(&out);

    assert!(!out.status.success(), "must not exit 0; stderr: {err}");
    assert!(err.contains("kos.yaml"), "must name the manifest: {err}");
    assert!(err.contains("parse"), "must carry the parse error: {err}");
}

#[test]
fn malformed_manifest_is_named_even_when_a_node_also_fails() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("_kos/kos.yaml"), MALFORMED);
    bad_edge_node(tmp.path());

    let out = kos(tmp.path(), &["validate"]);
    let err = stderr(&out);

    assert!(!out.status.success(), "stderr: {err}");
    assert!(
        err.contains("kos.yaml"),
        "the failing node must not hide the manifest: {err}"
    );
}

#[test]
fn merged_validate_reports_a_malformed_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("_kos/kos.yaml"), MALFORMED);

    let out = kos(tmp.path(), &["validate", "--merged"]);
    let err = stderr(&out);

    assert!(!out.status.success(), "stderr: {err}");
    assert!(err.contains("kos.yaml"), "must name the manifest: {err}");
}

#[test]
fn duplicate_ids_only_run_reports_a_malformed_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("_kos/kos.yaml"), MALFORMED);

    let out = kos(tmp.path(), &["validate", "--only", "duplicate-ids"]);
    let err = stderr(&out);

    assert!(!out.status.success(), "stderr: {err}");
    assert!(err.contains("kos.yaml"), "must name the manifest: {err}");
}

#[test]
fn findings_pass_errors_on_a_malformed_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    let kos_root = tmp.path().join("_kos");
    write(&kos_root.join("kos.yaml"), MALFORMED);
    fs::create_dir_all(kos_root.join("findings")).unwrap();

    let full = validate::run(&kos_root);
    let dup = validate::run_duplicate_ids(&kos_root);

    for (name, got) in [("run", full), ("run_duplicate_ids", dup)] {
        let err = got.expect_err(&format!("{name} must not pass a manifest it cannot read"));
        assert!(
            err.to_string().contains("kos.yaml"),
            "{name} error must name the manifest: {err}"
        );
    }
}

#[test]
fn valid_manifest_still_prints_the_header_and_passes() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("_kos/kos.yaml"), VALID);

    let out = kos(tmp.path(), &["validate"]);
    let err = stderr(&out);

    assert!(out.status.success(), "stderr: {err}");
    assert!(err.contains("Validating graph: fixture (repo)"), "{err}");
}

#[test]
fn valid_manifest_with_a_bad_node_still_fails_on_the_node() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("_kos/kos.yaml"), VALID);
    bad_edge_node(tmp.path());

    let out = kos(tmp.path(), &["validate"]);
    let all = format!("{}{}", stderr(&out), String::from_utf8_lossy(&out.stdout));

    assert!(!out.status.success(), "{all}");
    assert!(all.contains("unknown type"), "{all}");
}

#[test]
fn an_absent_manifest_in_a_subdirectory_is_not_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    write(&tmp.path().join("_kos/kos.yaml"), VALID);
    // A sibling directory with a bare _kos/ is scanned and skipped, not
    // reported: absence is not a parse failure.
    fs::create_dir_all(tmp.path().join("other/_kos")).unwrap();

    let out = kos(tmp.path(), &["validate", "--merged"]);

    assert!(out.status.success(), "stderr: {}", stderr(&out));
}
