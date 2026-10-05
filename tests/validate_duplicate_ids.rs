//! `kos validate --only duplicate-ids` (aae-orc-dq328, opaque-finding-ids
//! design plan item 7). The duplicate-finding-id check becomes a required CI
//! check, so it must run alone with its own exit status: a PR that adds a
//! duplicate id fails it, and a PR with only warnings (or only node failures,
//! which stay advisory) passes it.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn kos(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run kos")
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A clean standalone graph: one valid node, one numbered finding.
fn clean(root: &Path) {
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: fixture\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &root.join("_kos/nodes/bedrock/elem-anchor.yaml"),
        "id: elem-anchor\ntype: element\nconfidence: bedrock\ntitle: anchor\ncontent: a body\n",
    );
    write(
        &root.join("_kos/findings/finding-001-first.md"),
        "# first\n\nbody\n",
    );
}

#[test]
fn a_duplicate_finding_number_fails_the_only_check() {
    let tmp = tempfile::tempdir().unwrap();
    clean(tmp.path());
    write(
        &tmp.path().join("_kos/findings/finding-001-second.md"),
        "# second\n\nbody\n",
    );
    let out = kos(tmp.path(), &["validate", "--only", "duplicate-ids"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(
        text(&out).contains("duplicate finding number 'finding-001'"),
        "{}",
        text(&out)
    );
}

#[test]
fn a_warnings_only_graph_passes_the_only_check() {
    let tmp = tempfile::tempdir().unwrap();
    clean(tmp.path());
    // A node edge to a target that does not exist is a warning.
    write(
        &tmp.path().join("_kos/nodes/bedrock/elem-edgy.yaml"),
        "id: elem-edgy\ntype: element\nconfidence: bedrock\ntitle: t\ncontent: c\nedges:\n- target: nowhere-at-all\n  type: supports\n",
    );
    // A finding whose frontmatter id differs from its filename is drift, a warning.
    write(
        &tmp.path().join("_kos/findings/finding-002-drift.md"),
        "---\nid: finding-003-drift\n---\n# drift\n\nbody\n",
    );
    let out = kos(tmp.path(), &["validate", "--only", "duplicate-ids"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert!(
        !text(&out).contains("WARN"),
        "the only run reports no warnings:\n{}",
        text(&out)
    );
}

#[test]
fn a_node_failure_stays_advisory_for_the_only_check() {
    let tmp = tempfile::tempdir().unwrap();
    clean(tmp.path());
    // A frontier node filed under bedrock/ is a node failure, not a duplicate id.
    write(
        &tmp.path().join("_kos/nodes/bedrock/elem-misfiled.yaml"),
        "id: elem-misfiled\ntype: element\nconfidence: frontier\ntitle: t\ncontent: c\n",
    );
    let full = kos(tmp.path(), &["validate"]);
    assert_eq!(
        full.status.code(),
        Some(1),
        "plain validate still fails:\n{}",
        text(&full)
    );
    let only = kos(tmp.path(), &["validate", "--only", "duplicate-ids"]);
    assert_eq!(only.status.code(), Some(0), "{}", text(&only));
    assert!(
        !text(&only).contains("elem-misfiled"),
        "node results are not printed:\n{}",
        text(&only)
    );
}

#[test]
fn the_only_check_reports_the_findings_count_line() {
    let tmp = tempfile::tempdir().unwrap();
    clean(tmp.path());
    let out = kos(tmp.path(), &["validate", "--only", "duplicate-ids"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert!(
        text(&out).contains("1 findings: 0 duplicate-id failures"),
        "{}",
        text(&out)
    );
}

#[test]
fn an_unknown_section_is_refused_not_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    clean(tmp.path());
    let out = kos(tmp.path(), &["validate", "--only", "everything"]);
    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    assert!(
        text(&out).contains("possible values: duplicate-ids"),
        "{}",
        text(&out)
    );
}

#[test]
fn merged_runs_the_only_check_on_every_graph() {
    let tmp = tempfile::tempdir().unwrap();
    clean(tmp.path());
    write(
        &tmp.path().join("_kos/kos.yaml"),
        "graph_id: fixture\nscope: orchestrator\nschema_version: '0.3'\nincludes:\n- path: sub/_kos\n",
    );
    write(
        &tmp.path().join("sub/_kos/kos.yaml"),
        "graph_id: sub\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &tmp.path().join("sub/_kos/nodes/bedrock/elem-a.yaml"),
        "id: elem-a\ntype: element\nconfidence: bedrock\ntitle: t\ncontent: c\n",
    );
    write(
        &tmp.path().join("sub/_kos/findings/finding-001-a.md"),
        "# a\n\nbody\n",
    );
    write(
        &tmp.path().join("sub/_kos/findings/finding-001-b.md"),
        "# b\n\nbody\n",
    );
    let out = kos(
        tmp.path(),
        &["validate", "--merged", "--only", "duplicate-ids"],
    );
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
}

/// An orchestrator root graph that includes `n` subrepo graphs, each clean.
fn orc_with_subs(root: &Path, n: usize) {
    let includes: String = (1..=n).map(|i| format!("- path: sub{i}/_kos\n")).collect();
    write(
        &root.join("_kos/kos.yaml"),
        &format!(
            "graph_id: fixture\nscope: orchestrator\nschema_version: '0.3'\nincludes:\n{includes}"
        ),
    );
    write(
        &root.join("_kos/findings/finding-001-root.md"),
        "# root\n\nbody\n",
    );
    for i in 1..=n {
        write(
            &root.join(format!("sub{i}/_kos/kos.yaml")),
            &format!("graph_id: sub{i}\nscope: repo\nschema_version: '0.3'\n"),
        );
        write(
            &root.join(format!("sub{i}/_kos/findings/finding-001-a.md")),
            "# a\n\nbody\n",
        );
    }
}

#[test]
fn merged_catches_a_duplicate_in_the_root_graph_alone() {
    // A merged run that skipped the first graph would hide the orchestrator's
    // own duplicates (the orc is the graph that collides most).
    let tmp = tempfile::tempdir().unwrap();
    orc_with_subs(tmp.path(), 2);
    write(
        &tmp.path().join("_kos/findings/finding-001-root-again.md"),
        "# again\n\nbody\n",
    );
    let out = kos(
        tmp.path(),
        &["validate", "--merged", "--only", "duplicate-ids"],
    );
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
}

#[test]
fn merged_catches_a_duplicate_in_the_last_graph_alone() {
    let tmp = tempfile::tempdir().unwrap();
    orc_with_subs(tmp.path(), 2);
    write(
        &tmp.path().join("sub2/_kos/findings/finding-001-b.md"),
        "# b\n\nbody\n",
    );
    let out = kos(
        tmp.path(),
        &["validate", "--merged", "--only", "duplicate-ids"],
    );
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
}

#[test]
fn merged_only_summary_does_not_pretend_a_node_pass_ran() {
    let tmp = tempfile::tempdir().unwrap();
    orc_with_subs(tmp.path(), 1);
    let out = kos(
        tmp.path(),
        &["validate", "--merged", "--only", "duplicate-ids"],
    );
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert!(!text(&out).contains("0 nodes"), "{}", text(&out));
    assert!(
        text(&out).contains("all graphs: 2 findings, 0 duplicate-id failures"),
        "{}",
        text(&out)
    );
}
