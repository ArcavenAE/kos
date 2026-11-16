//! `kos facts`, end to end (aae-orc-xxe3n, PR 3). Each test builds a store
//! with the library, runs the binary against it in a temp dir, and reads the
//! exit status and the two output streams.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use kos::facts::{Fact, FactStore, Version};

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn kos(args: &[&str]) -> Out {
    let out = Command::new(env!("CARGO_BIN_EXE_kos"))
        .args(args)
        .env_remove("KOS_WORKSPACE")
        .output()
        .unwrap();
    Out {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8(out.stdout).unwrap(),
        stderr: String::from_utf8(out.stderr).unwrap(),
    }
}

fn v(n: i64) -> Version {
    Version::new(n)
}

fn f(e: &str, a: &str, val: &str) -> Fact {
    Fact::new(e, a, val)
}

fn s(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}

/// A store file holding two facts at version 1, one with a value that carries
/// a tab, a newline and a backslash.
const AWKWARD: &str = "tab\there\nnew\\slash";

fn awkward_store(dir: &Path) -> PathBuf {
    let db = dir.join("facts.db");
    let mut store = FactStore::open(&db).unwrap();
    store
        .assert_facts(
            v(1),
            &[
                f("b.go", "REFERENCES", AWKWARD),
                f("a.go", "DEFINES", "plain"),
            ],
        )
        .unwrap();
    db
}

/// A store with a change between versions 1 and 2: two facts removed, two
/// added, written out of order so the sort shows.
fn changing_store(dir: &Path) -> PathBuf {
    let db = dir.join("change.db");
    let mut store = FactStore::open(&db).unwrap();
    store
        .assert_facts(
            v(1),
            &[f("keep", "r", "1"), f("y", "r", "1"), f("x", "r", "1")],
        )
        .unwrap();
    store
        .retract_facts(v(2), &[f("y", "r", "1"), f("x", "r", "1")])
        .unwrap();
    store
        .assert_facts(v(2), &[f("z", "r", "2"), f("w", "r", "1")])
        .unwrap();
    db
}

fn harvest_file(dir: &Path, name: &str, with_source: bool) -> PathBuf {
    let path = dir.join(name);
    let source = if with_source {
        "source:\n  repo: demo/repo\n  commit: abc1234\n"
    } else {
        ""
    };
    fs::write(
        &path,
        format!("kind: harvested-facts\n{source}facts:\n- e: a\n  a: r\n  v: \"1\"\n- e: b\n  a: r\n  v: \"2\"\n"),
    )
    .unwrap();
    path
}

#[test]
fn load_asserts_a_harvest_file_at_the_given_version_and_prints_one_line() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("facts.db");
    let file = harvest_file(dir.path(), "h.yaml", true);
    let out = kos(&["facts", "load", "--db", &s(&db), "--at", "7", &s(&file)]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_eq!(
        out.stdout.lines().count(),
        1,
        "one line, got {:?}",
        out.stdout
    );
    assert!(
        out.stdout.contains('2'),
        "the count 2 is named: {:?}",
        out.stdout
    );
    assert_eq!(out.stderr, "");
    let store = FactStore::open(&db).unwrap();
    assert_eq!(
        store.facts_as_of(v(7)).unwrap(),
        vec![f("a", "r", "1"), f("b", "r", "2")]
    );
    assert_eq!(store.facts_as_of(v(6)).unwrap(), Vec::<Fact>::new());
}

#[test]
fn load_refuses_a_harvest_file_without_a_source_and_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("facts.db");
    let file = harvest_file(dir.path(), "h.yaml", false);
    let out = kos(&["facts", "load", "--db", &s(&db), "--at", "1", &s(&file)]);
    assert_eq!(out.code, 1, "stdout: {}", out.stdout);
    assert!(
        out.stderr.contains("source"),
        "stderr names the source: {:?}",
        out.stderr
    );
    let store = FactStore::open(&db).unwrap();
    assert_eq!(store.latest_version().unwrap(), None);
}

#[test]
fn as_of_prints_sorted_tab_separated_lines_with_escaped_fields() {
    let dir = tempfile::tempdir().unwrap();
    let db = awkward_store(dir.path());
    let out = kos(&["facts", "as-of", "--db", &s(&db), "--at", "1"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    // The value's tab, newline and backslash print as \t, \n and \\.
    let want = "a.go\tDEFINES\tplain\nb.go\tREFERENCES\ttab\\there\\nnew\\\\slash\n";
    assert_eq!(out.stdout, want);
}

#[test]
fn as_of_json_is_jsonl_and_a_value_with_a_tab_and_a_newline_round_trips_exactly() {
    let dir = tempfile::tempdir().unwrap();
    let db = awkward_store(dir.path());
    let out = kos(&["facts", "as-of", "--db", &s(&db), "--at", "1", "--json"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    let rows: Vec<serde_json::Value> = out
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(rows.len(), 2, "one object per line, got {:?}", out.stdout);
    assert_eq!(rows[0]["e"], "a.go");
    assert_eq!(rows[1]["e"], "b.go");
    assert_eq!(rows[1]["v"], AWKWARD, "the value survives byte for byte");
    let keys: Vec<&String> = rows[1].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["e", "a", "v"], "exactly the three fields");
}

#[test]
fn as_of_filters_by_entity_and_attribute() {
    let dir = tempfile::tempdir().unwrap();
    let db = awkward_store(dir.path());
    let only_entity = kos(&[
        "facts",
        "as-of",
        "--db",
        &s(&db),
        "--at",
        "1",
        "--entity",
        "a.go",
    ]);
    assert_eq!(only_entity.stdout, "a.go\tDEFINES\tplain\n");
    let only_attr = kos(&[
        "facts",
        "as-of",
        "--db",
        &s(&db),
        "--at",
        "1",
        "--attribute",
        "REFERENCES",
    ]);
    assert!(
        only_attr.stdout.starts_with("b.go\tREFERENCES\t"),
        "{:?}",
        only_attr.stdout
    );
    assert_eq!(only_attr.stdout.lines().count(), 1);
    let neither = kos(&[
        "facts",
        "as-of",
        "--db",
        &s(&db),
        "--at",
        "1",
        "--entity",
        "a.go",
        "--attribute",
        "REFERENCES",
    ]);
    assert_eq!(neither.code, 0);
    assert_eq!(neither.stdout, "");
}

#[test]
fn diff_prints_removed_then_added_each_group_sorted_with_a_tab_after_the_sign() {
    let dir = tempfile::tempdir().unwrap();
    let db = changing_store(dir.path());
    let out = kos(&["facts", "diff", "--db", &s(&db), "--from", "1", "--to", "2"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    let want = "-\tx\tr\t1\n-\ty\tr\t1\n+\tw\tr\t1\n+\tz\tr\t2\n";
    assert_eq!(out.stdout, want);
}

#[test]
fn diff_json_is_jsonl_with_an_op_field_removed_first_each_group_sorted() {
    let dir = tempfile::tempdir().unwrap();
    let db = changing_store(dir.path());
    let out = kos(&[
        "facts",
        "diff",
        "--db",
        &s(&db),
        "--from",
        "1",
        "--to",
        "2",
        "--json",
    ]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    let rows: Vec<serde_json::Value> = out
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let got: Vec<(String, String)> = rows
        .iter()
        .map(|r| {
            (
                r["op"].as_str().unwrap_or("").to_string(),
                r["e"].as_str().unwrap_or("").to_string(),
            )
        })
        .collect();
    let want: Vec<(String, String)> = [
        ("removed", "x"),
        ("removed", "y"),
        ("added", "w"),
        ("added", "z"),
    ]
    .iter()
    .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
    .collect();
    assert_eq!(got, want);
    let keys: Vec<&String> = rows[0].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["op", "e", "a", "v"]);
}

#[test]
fn export_to_stdout_is_the_library_export() {
    let dir = tempfile::tempdir().unwrap();
    let db = changing_store(dir.path());
    let mut want = Vec::new();
    FactStore::open(&db)
        .unwrap()
        .export_jsonl(&mut want)
        .unwrap();
    let out = kos(&["facts", "export", "--db", &s(&db)]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_eq!(out.stdout, String::from_utf8(want).unwrap());
}

#[test]
fn export_out_writes_a_new_file_and_refuses_an_existing_one() {
    let dir = tempfile::tempdir().unwrap();
    let db = changing_store(dir.path());
    let target = dir.path().join("export.jsonl");
    let first = kos(&["facts", "export", "--db", &s(&db), "--out", &s(&target)]);
    assert_eq!(first.code, 0, "stderr: {}", first.stderr);
    assert_eq!(first.stdout, "", "nothing on stdout when --out is given");
    assert!(target.exists(), "--out creates the file");
    let written = fs::read_to_string(&target).unwrap();
    assert!(
        written.starts_with(r#"{"kos_facts_export":1"#),
        "{written:?}"
    );
    // A second export to the same name must not overwrite it.
    fs::write(&target, "keep me\n").unwrap();
    let second = kos(&["facts", "export", "--db", &s(&db), "--out", &s(&target)]);
    assert_eq!(second.code, 1);
    assert!(
        second.stderr.contains("export.jsonl"),
        "names the file: {:?}",
        second.stderr
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), "keep me\n");
}

#[test]
fn import_replays_a_jsonl_export_into_a_new_store_and_prints_one_line() {
    let dir = tempfile::tempdir().unwrap();
    let source = changing_store(dir.path());
    let mut text = Vec::new();
    FactStore::open(&source)
        .unwrap()
        .export_jsonl(&mut text)
        .unwrap();
    let file = dir.path().join("in.jsonl");
    fs::write(&file, &text).unwrap();
    let db = dir.path().join("copy.db");
    let out = kos(&["facts", "import", "--db", &s(&db), &s(&file)]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_eq!(
        out.stdout.lines().count(),
        1,
        "one line, got {:?}",
        out.stdout
    );
    assert!(
        out.stdout.contains('5'),
        "the row count 5 is named (keep, x and y from version 1, then z and w): {:?}",
        out.stdout
    );
    assert_eq!(out.stderr, "");
    let copy = FactStore::open(&db).unwrap();
    let orig = FactStore::open(&source).unwrap();
    for n in 0..=3 {
        assert_eq!(
            copy.facts_as_of(v(n)).unwrap(),
            orig.facts_as_of(v(n)).unwrap()
        );
    }
}

#[test]
fn import_into_a_store_that_holds_facts_exits_1_with_the_hint() {
    let dir = tempfile::tempdir().unwrap();
    let source = changing_store(dir.path());
    let mut text = Vec::new();
    FactStore::open(&source)
        .unwrap()
        .export_jsonl(&mut text)
        .unwrap();
    let file = dir.path().join("in.jsonl");
    fs::write(&file, &text).unwrap();
    let full = awkward_store(dir.path());
    let out = kos(&["facts", "import", "--db", &s(&full), &s(&file)]);
    assert_eq!(out.code, 1);
    assert!(
        out.stderr.contains("empty store"),
        "hint in stderr: {:?}",
        out.stderr
    );
    assert_eq!(out.stdout, "");
}

#[test]
fn load_refuses_a_jsonl_file_and_import_refuses_a_yaml_file_each_with_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("facts.db");
    let jsonl = dir.path().join("x.jsonl");
    fs::write(&jsonl, "{}\n").unwrap();
    let load = kos(&["facts", "load", "--db", &s(&db), "--at", "1", &s(&jsonl)]);
    assert_eq!(load.code, 1);
    assert!(
        load.stderr.contains("kos facts import"),
        "load hint names import: {:?}",
        load.stderr
    );
    let yaml = harvest_file(dir.path(), "h.yaml", true);
    let import = kos(&["facts", "import", "--db", &s(&db), &s(&yaml)]);
    assert_eq!(import.code, 1);
    assert!(
        import.stderr.contains("kos facts load"),
        "import hint names load: {:?}",
        import.stderr
    );
    assert!(!db.exists(), "a refused file must not create the store");
}

#[test]
fn at_is_the_version_flag_and_version_is_not_a_facts_flag_and_db_is_required() {
    let dir = tempfile::tempdir().unwrap();
    let db = awkward_store(dir.path());
    let ok = kos(&["facts", "as-of", "--db", &s(&db), "--at", "1"]);
    assert_eq!(ok.code, 0, "--at is accepted, stderr: {}", ok.stderr);
    assert!(
        ok.stdout.contains("a.go"),
        "and it reads the version: {:?}",
        ok.stdout
    );
    let wrong = kos(&["facts", "as-of", "--db", &s(&db), "--version", "1"]);
    assert_eq!(wrong.code, 2, "--version is a usage error");
    assert_eq!(wrong.stdout, "", "and it must not print kos's own version");
    let wrong_load = kos(&["facts", "load", "--db", &s(&db), "--version", "1", "x.yaml"]);
    assert_eq!(wrong_load.code, 2);
    let no_db = kos(&["facts", "as-of", "--at", "1"]);
    assert_eq!(
        no_db.code, 2,
        "--db is required, there is no default location"
    );
    assert!(no_db.stderr.contains("--db"), "{:?}", no_db.stderr);
}
