//! `kos facts refs | impls | dependents` and the symbol matcher under them
//! (aae-orc I-853). Each CLI test builds a store with the library in a temp dir,
//! runs the binary against it, and reads the exit status and both streams.

use std::path::{Path, PathBuf};
use std::process::Command;

use kos::facts::{Fact, FactStore, Version};
use serde_json::Value;

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

const TRAV: &str = "rust-analyzer cargo kos . facts/FactStore#traverse().";
const STORE: &str = "rust-analyzer cargo kos . facts/FactStore#";
const MARVEL_STORE: &str = "rust-analyzer cargo marvel . store/FactStore#";
const TRAIT: &str = "rust-analyzer cargo kos . facts/Reader#";
const MEM: &str = "rust-analyzer cargo kos . facts/MemReader#";
const DISK: &str = "rust-analyzer cargo kos . facts/DiskReader#";
const UNREFERENCED: &str = "rust-analyzer cargo kos . facts/Unused#";
/// A document path with a tab and a newline in it.
const AWKWARD_DOC: &str = "kos:src/we\tird\nname.rs";

/// A code graph in two versions. At version 1, three kos documents reference
/// `TRAV`; at version 2 one of them stops and another starts, and `DISK` stops
/// implementing `TRAIT`. A marvel document at the same path `src/a.rs` refers
/// to marvel's own `FactStore`.
fn code_store(dir: &Path) -> PathBuf {
    let db = dir.join("code.db");
    let mut store = FactStore::open(&db).unwrap();
    store
        .assert_facts(
            v(1),
            &[
                f("kos:src/a.rs", "code/defines", STORE),
                f("kos:src/a.rs", "code/defines", TRAV),
                f("kos:src/a.rs", "code/references", TRAV),
                f("kos:src/a.rs", "code/references", STORE),
                f("kos:src/b.rs", "code/references", TRAV),
                f(AWKWARD_DOC, "code/references", TRAV),
                f("kos:src/r.rs", "code/defines", TRAIT),
                f("kos:src/r.rs", "code/defines", MEM),
                f("kos:src/r.rs", "code/defines", DISK),
                f("kos:src/z.rs", "code/defines", UNREFERENCED),
                f(MEM, "code/implements", TRAIT),
                f(DISK, "code/implements", TRAIT),
                f("marvel:src/a.rs", "code/defines", MARVEL_STORE),
                f("marvel:src/a.rs", "code/references", MARVEL_STORE),
                f("kos", "code/depends_on", "cargo serde 1.0"),
                f("kos", "code/depends_on", "cargo thiserror 2.0"),
                f("marvel", "code/depends_on", "cargo serde 1.0"),
            ],
        )
        .unwrap();
    store
        .apply(
            v(2),
            &[
                f("kos:src/b.rs", "code/references", TRAV),
                f(DISK, "code/implements", TRAIT),
            ],
            &[f("kos:src/c.rs", "code/references", TRAV)],
        )
        .unwrap();
    db
}

fn json_lines(text: &str) -> Vec<Value> {
    text.lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("not JSON: {l:?}: {e}")))
        .collect()
}

// 1
#[test]
fn refs_prints_the_referencing_documents_sorted_with_a_header_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&[
        "facts",
        "refs",
        "--db",
        &s(&db),
        "--at",
        "1",
        "FactStore#traverse().",
    ]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "kos:src/a.rs\nkos:src/b.rs\nkos:src/we\\tird\\nname.rs\n"
    );
    assert_eq!(
        out.stderr.trim_end(),
        format!("symbol {TRAV} at version 1: 3 results")
    );
}

// 2
#[test]
fn refs_json_is_one_object_per_document_and_a_trailer_with_the_resolution() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&[
        "facts",
        "refs",
        "--db",
        &s(&db),
        "--at",
        "1",
        "--json",
        "FactStore#traverse().",
    ]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    let lines = json_lines(&out.stdout);
    assert_eq!(lines.len(), 4, "three documents and a trailer: {lines:?}");
    assert_eq!(lines[0]["document"], "kos:src/a.rs");
    assert_eq!(lines[2]["document"], AWKWARD_DOC);
    assert_eq!(lines[3]["query"], "FactStore#traverse().");
    assert_eq!(lines[3]["symbol"], TRAV);
    assert_eq!(lines[3]["at"], 1);
    assert_eq!(lines[3]["count"], 3);
    assert_eq!(lines[3]["truncated"], false);
}

// 3
#[test]
fn impls_returns_the_implementers_of_the_symbol() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&["facts", "impls", "--db", &s(&db), "--at", "1", "Reader#"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_eq!(out.stdout, format!("{DISK}\n{MEM}\n"));
}

// 4
#[test]
fn dependents_returns_the_repos_that_depend_on_the_package() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&["facts", "dependents", "--db", &s(&db), "cargo serde 1.0"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_eq!(out.stdout, "kos\nmarvel\n");
    let json = kos(&[
        "facts",
        "dependents",
        "--db",
        &s(&db),
        "--json",
        "cargo serde 1.0",
    ]);
    let lines = json_lines(&json.stdout);
    assert_eq!(lines[0]["repo"], "kos");
    assert_eq!(lines[1]["repo"], "marvel");
}

// 5
#[test]
fn as_of_reads_the_facts_of_that_version() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let at1 = kos(&["facts", "refs", "--db", &s(&db), "--at", "1", TRAV]);
    let at2 = kos(&["facts", "refs", "--db", &s(&db), "--at", "2", TRAV]);
    assert_eq!(at1.code, 0, "stderr: {}", at1.stderr);
    assert_eq!(at2.code, 0, "stderr: {}", at2.stderr);
    assert!(at1.stdout.contains("kos:src/b.rs") && !at1.stdout.contains("kos:src/c.rs"));
    assert!(at2.stdout.contains("kos:src/c.rs") && !at2.stdout.contains("kos:src/b.rs"));
    let impls2 = kos(&["facts", "impls", "--db", &s(&db), "--at", "2", TRAIT]);
    assert_eq!(impls2.stdout, format!("{MEM}\n"));
}

// 6
#[test]
fn without_at_the_verb_reads_the_latest_version() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&["facts", "refs", "--db", &s(&db), "--json", TRAV]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    let lines = json_lines(&out.stdout);
    assert_eq!(lines.last().unwrap()["at"], 2);
    assert_eq!(lines[1]["document"], "kos:src/c.rs");
}

// 7: the '/' boundary
#[test]
fn a_descriptor_suffix_resolves_to_the_one_symbol_it_ends() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let store = FactStore::open(&db).unwrap();
    let got = store
        .symbols_matching(
            v(2),
            &["code/defines", "code/references", "code/implements"],
            "FactStore#traverse().",
        )
        .unwrap();
    assert_eq!(got, vec![TRAV.to_string()]);
}

// 8
#[test]
fn the_full_symbol_resolves_to_itself() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let store = FactStore::open(&db).unwrap();
    let got = store
        .symbols_matching(v(2), &["code/defines", "code/references"], STORE)
        .unwrap();
    assert_eq!(got, vec![STORE.to_string()]);
}

// 9: the space boundary, and no match in mid-word
#[test]
fn a_suffix_that_does_not_start_at_a_boundary_matches_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let store = FactStore::open(&db).unwrap();
    let attrs = ["code/defines", "code/references", "code/implements"];
    let got = store
        .symbols_matching(v(2), &attrs, "actStore#traverse().")
        .unwrap();
    assert_eq!(got, Vec::<String>::new(), "mid-word text is not a match");
    let at_space = store
        .symbols_matching(v(2), &attrs, "kos . facts/FactStore#traverse().")
        .unwrap();
    assert_eq!(at_space, vec![TRAV.to_string()], "a space is a boundary");
}

// 10
#[test]
fn an_ambiguous_query_lists_the_candidates_and_prints_nothing_on_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&["facts", "refs", "--db", &s(&db), "FactStore#"]);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout, "");
    assert!(out.stderr.contains("matches 2 symbols"), "{:?}", out.stderr);
    assert!(out.stderr.contains(STORE) && out.stderr.contains(MARVEL_STORE));
    assert!(out.stderr.contains("hint:"), "{:?}", out.stderr);
}

// 11
#[test]
fn an_unknown_symbol_is_refused_with_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&["facts", "refs", "--db", &s(&db), "NoSuchThing#"]);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout, "");
    assert!(
        out.stderr
            .contains("no symbol matches \"NoSuchThing#\" at version 2"),
        "{:?}",
        out.stderr
    );
    assert!(out.stderr.contains("hint:"), "{:?}", out.stderr);
}

// 12
#[test]
fn a_missing_store_is_refused_and_none_is_created() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("absent.db");
    for verb in ["refs", "impls", "dependents"] {
        let out = kos(&["facts", verb, "--db", &s(&db), "anything"]);
        assert_eq!(out.code, 1, "{verb}: stderr {}", out.stderr);
        assert!(
            out.stderr.contains("no fact store at"),
            "{verb}: {:?}",
            out.stderr
        );
        assert_eq!(out.stdout, "", "{verb} printed on stdout");
        assert!(!db.exists(), "{verb} created a store");
    }
}

// 13
#[test]
fn an_as_of_before_the_first_version_says_the_store_holds_no_facts_then() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&["facts", "refs", "--db", &s(&db), "--at", "0", TRAV]);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout, "");
    assert!(
        out.stderr
            .contains("holds no facts at version 0 (versions 1 to 2)"),
        "{:?}",
        out.stderr
    );
}

// 14
#[test]
fn a_symbol_nobody_references_is_an_empty_answer_and_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&["facts", "refs", "--db", &s(&db), "--json", "Unused#"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    let lines = json_lines(&out.stdout);
    assert_eq!(lines.len(), 1, "only the trailer: {lines:?}");
    assert_eq!(lines[0]["symbol"], UNREFERENCED);
    assert_eq!(lines[0]["count"], 0);
}

// 15
#[test]
fn limit_truncates_and_says_so_in_both_forms() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let text = kos(&[
        "facts",
        "refs",
        "--db",
        &s(&db),
        "--at",
        "1",
        "--limit",
        "2",
        TRAV,
    ]);
    assert_eq!(text.code, 0, "stderr: {}", text.stderr);
    assert_eq!(text.stdout, "kos:src/a.rs\nkos:src/b.rs\n");
    assert!(text.stderr.contains("showing 2 of 3"), "{:?}", text.stderr);
    let json = kos(&[
        "facts",
        "refs",
        "--db",
        &s(&db),
        "--at",
        "1",
        "--limit",
        "2",
        "--json",
        TRAV,
    ]);
    let lines = json_lines(&json.stdout);
    assert_eq!(lines.len(), 3, "two documents and a trailer: {lines:?}");
    assert_eq!(lines[2]["truncated"], true);
    assert_eq!(lines[2]["count"], 3);
}

// 16
#[test]
fn text_escapes_a_tab_and_a_newline_and_json_keeps_them_exact() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let text = kos(&["facts", "refs", "--db", &s(&db), "--at", "1", TRAV]);
    assert!(
        text.stdout.contains("kos:src/we\\tird\\nname.rs\n"),
        "{:?}",
        text.stdout
    );
    assert!(!text.stdout.contains('\t'));
    let json = kos(&[
        "facts",
        "refs",
        "--db",
        &s(&db),
        "--at",
        "1",
        "--json",
        TRAV,
    ]);
    assert_eq!(json_lines(&json.stdout)[2]["document"], AWKWARD_DOC);
}

// 17
#[test]
fn one_repos_document_is_not_mixed_with_another_repos_at_the_same_path() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    // kos:src/a.rs and marvel:src/a.rs are one path in two repos, and each
    // references its own repo's FactStore.
    let marvel = kos(&["facts", "refs", "--db", &s(&db), "store/FactStore#"]);
    assert_eq!(marvel.code, 0, "stderr: {}", marvel.stderr);
    assert_eq!(marvel.stdout, "marvel:src/a.rs\n");
    let own = kos(&["facts", "refs", "--db", &s(&db), "facts/FactStore#"]);
    assert_eq!(own.code, 0, "stderr: {}", own.stderr);
    assert_eq!(own.stdout, "kos:src/a.rs\n");
}

// 18: a literal backslash-t is not a tab
#[test]
fn text_escapes_a_backslash_so_a_literal_backslash_t_differs_from_a_tab() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("esc.db");
    let mut store = FactStore::open(&db).unwrap();
    store
        .assert_facts(
            v(1),
            &[
                f("kos:src/tab\there.rs", "code/references", TRAV),
                f("kos:src/lit\\there.rs", "code/references", TRAV),
            ],
        )
        .unwrap();
    drop(store);
    let text = kos(&["facts", "refs", "--db", &s(&db), TRAV]);
    assert_eq!(text.code, 0, "stderr: {}", text.stderr);
    assert_eq!(
        text.stdout, "kos:src/lit\\\\there.rs\nkos:src/tab\\there.rs\n",
        "a backslash is written \\\\ and a tab \\t, so the two are told apart"
    );
    let json = kos(&["facts", "refs", "--db", &s(&db), "--json", TRAV]);
    let lines = json_lines(&json.stdout);
    assert_eq!(lines[0]["document"], "kos:src/lit\\there.rs");
    assert_eq!(lines[1]["document"], "kos:src/tab\there.rs");
}

// 19: an unknown package, as an unknown symbol
#[test]
fn dependents_of_an_unknown_package_is_refused_with_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&["facts", "dependents", "--db", &s(&db), "cargo nosuch 9.9"]);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout, "");
    assert!(
        out.stderr
            .contains("no package matches \"cargo nosuch 9.9\" at version 2"),
        "{:?}",
        out.stderr
    );
    assert!(out.stderr.contains("hint:"), "{:?}", out.stderr);
}

/// A store of `n` symbols that each end `Dup#` after a different directory,
/// every one defined by one document.
fn dup_store(dir: &Path, n: usize) -> PathBuf {
    let db = dir.join("dup.db");
    let mut store = FactStore::open(&db).unwrap();
    let facts: Vec<Fact> = (0..n)
        .map(|i| {
            f(
                "kos:src/d.rs",
                "code/defines",
                &format!("rust-analyzer cargo kos . m{i:02}/Dup#"),
            )
        })
        .collect();
    store.assert_facts(v(1), &facts).unwrap();
    db
}

// 20: the candidate list is capped at ten
#[test]
fn an_ambiguous_query_lists_at_most_ten_candidates_and_counts_the_rest() {
    let dir = tempfile::tempdir().unwrap();
    let db = dup_store(dir.path(), 11);
    let out = kos(&["facts", "refs", "--db", &s(&db), "Dup#"]);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout, "");
    assert!(
        out.stderr.contains("matches 11 symbols"),
        "{:?}",
        out.stderr
    );
    let listed = out
        .stderr
        .lines()
        .filter(|l| l.contains("rust-analyzer cargo kos . m"))
        .count();
    assert_eq!(listed, 10, "{:?}", out.stderr);
    assert!(out.stderr.contains("and 1 more"), "{:?}", out.stderr);
}

// 21: the default --limit is 200
#[test]
fn without_limit_the_verb_shows_200_results_and_says_how_many_there_are() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("many.db");
    let mut store = FactStore::open(&db).unwrap();
    let facts: Vec<Fact> = (0..201)
        .map(|i| f(&format!("kos:src/f{i:03}.rs"), "code/references", TRAV))
        .collect();
    store.assert_facts(v(1), &facts).unwrap();
    drop(store);
    let out = kos(&["facts", "refs", "--db", &s(&db), TRAV]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_eq!(out.stdout.lines().count(), 200);
    assert!(
        out.stderr.contains("showing 200 of 201"),
        "{:?}",
        out.stderr
    );
    let all = kos(&["facts", "refs", "--db", &s(&db), "--limit", "0", TRAV]);
    assert_eq!(all.stdout.lines().count(), 201, "--limit 0 is no cap");
}

// 22: no references, text form
#[test]
fn a_symbol_nobody_references_prints_nothing_and_a_header_saying_zero_in_text() {
    let dir = tempfile::tempdir().unwrap();
    let db = code_store(dir.path());
    let out = kos(&["facts", "refs", "--db", &s(&db), "Unused#"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_eq!(out.stdout, "");
    assert_eq!(
        out.stderr.trim_end(),
        format!("symbol {UNREFERENCED} at version 2: 0 results")
    );
}
