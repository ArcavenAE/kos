//! Minted finding ids, end to end (aae-orc-wmna4, opaque-finding-ids design
//! plan item 2). Two clones of one base each run `kos finding` with the same
//! slug and title and no coordination; the ids must differ and the union must
//! validate.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// A standalone graph with one node and one numbered finding.
fn base(root: &Path) {
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: kos\nscope: repo\nschema_version: '0.3'\n",
    );
    write(
        &root.join("_kos/nodes/bedrock/elem-anchor.yaml"),
        "id: elem-anchor\ntype: element\nconfidence: bedrock\ntitle: anchor\ncontent: a body\n",
    );
    write(
        &root.join("_kos/findings/finding-001-old.md"),
        "# old\n\n**Date:** 2026-01-01\n\nbody\n",
    );
}

fn kos(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .args(args)
        .output()
        .expect("run kos")
}

fn findings(root: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(root.join("_kos/findings"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    v.sort();
    v
}

fn new_finding(root: &Path) -> PathBuf {
    let made: Vec<PathBuf> = findings(root)
        .into_iter()
        .filter(|p| !p.ends_with("finding-001-old.md"))
        .collect();
    assert_eq!(
        made.len(),
        1,
        "exactly one new finding in {}",
        root.display()
    );
    made[0].clone()
}

#[test]
fn two_clones_mint_distinct_ids_and_the_union_validates() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a");
    let b = tmp.path().join("b");
    base(&a);
    base(&b);

    for clone in [&a, &b] {
        let out = kos(&[
            "finding",
            "same-slug",
            "Same title",
            "--dir",
            clone.to_str().unwrap(),
        ]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let fa = new_finding(&a);
    let fb = new_finding(&b);
    let na = fa.file_name().unwrap().to_str().unwrap().to_string();
    let nb = fb.file_name().unwrap().to_str().unwrap().to_string();
    for n in [&na, &nb] {
        assert!(
            n.starts_with("finding-kos-"),
            "minted with the owning prefix: {n}"
        );
        assert!(
            n.ends_with("-same-slug.md"),
            "markdown by default, slug kept: {n}"
        );
    }
    assert_ne!(na, nb, "two uncoordinated mints must differ");

    // The union: the base plus both new files, as a merge of the two clones.
    fs::copy(&fb, a.join("_kos/findings").join(&nb)).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_kos"))
        .arg("validate")
        .current_dir(&a)
        .output()
        .expect("run kos validate");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "union must validate:\n{stdout}");
    assert!(
        stdout.contains("3 findings: 0 duplicate-id failures, 0 warnings"),
        "{stdout}"
    );
}

#[test]
fn kos_id_finding_prints_the_id_and_filename_and_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    let before = findings(tmp.path());

    let out = kos(&[
        "id",
        "finding",
        "some-topic",
        "--dir",
        tmp.path().to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "id then filename:\n{stdout}");
    assert!(lines[0].starts_with("finding-kos-"), "{stdout}");
    assert_eq!(lines[1], format!("{}-some-topic.md", lines[0]));
    assert_eq!(findings(tmp.path()), before, "kos id writes nothing");
}

#[test]
fn kos_finding_yaml_keeps_the_yaml_shape() {
    let tmp = tempfile::tempdir().unwrap();
    base(tmp.path());
    let out = kos(&[
        "finding",
        "as-yaml",
        "A title",
        "--yaml",
        "--dir",
        tmp.path().to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let made = new_finding(tmp.path());
    let name = made.file_name().unwrap().to_str().unwrap();
    assert!(
        name.starts_with("finding-kos-") && name.ends_with("-as-yaml.yaml"),
        "{name}"
    );
}
