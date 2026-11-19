//! `kos facts ingest-scip`, red tests (aae-orc-yto7v, brief
//! `_kos/probes/brief-scip-ingest-build.md`, SI-1 to SI-12).
//!
//! Every test runs the kos binary against a SCIP index it builds in a temp
//! dir with the `scip` crate, so no toolchain, network or live indexer is
//! needed, and a missing verb fails on an assertion about the binary's exit
//! status and output instead of failing to compile.
//!
//! The command these tests are built to:
//!
//! ```text
//! kos facts ingest-scip --db <store> --at <n> --repo <name> --commit <sha> \
//!     --index <file.scip> --log <indexer.log> --checkout <dir>
//! ```
//!
//! - Facts: `code/defines` and `code/references` (entity `<repo>:<path>`,
//!   value the symbol), `code/implements` (entity symbol, value symbol),
//!   `code/depends_on` (entity repo, value package), and `kos/names-dep`
//!   (entity a node id, value a crate name).
//! - Provenance: entity `load/<repo>@<at>`, attributes `prov/repo`,
//!   `prov/commit`, `prov/indexer`, `prov/indexer_version`,
//!   `prov/index_sha256`, `prov/grade`.
//! - A refusal exits nonzero, loads nothing and prints
//!   `GAP <repo>@<commit>: <reason>` on stderr.
//! - A load prints `filtered: out_of_root_documents=<n> local_symbols=<n>`.
//! - The indexer log's last line is `exit: <n>` for scip-go (the exit status
//!   is an input of SI-8).
//! - Crate names come from `.kos/crate-names.txt` in the checkout and are
//!   matched against the text of `_kos/nodes/*/<id>.yaml`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use kos::facts::{Fact, FactStore, Version};
use scip::types::{
    Document, Index, Metadata, Occurrence, Relationship, SymbolInformation, ToolInfo,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

// ---------------------------------------------------------------- running kos

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

fn s(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}

type Triple = (String, String, String);

fn t(e: &str, a: &str, v: &str) -> Triple {
    (e.to_string(), a.to_string(), v.to_string())
}

// ------------------------------------------------------------ building indexes

const DEFINITION: i32 = 1;

struct Doc {
    path: String,
    defs: Vec<String>,
    refs: Vec<String>,
    implements: Vec<(String, String)>,
}

fn doc(path: &str) -> Doc {
    Doc {
        path: path.to_string(),
        defs: Vec::new(),
        refs: Vec::new(),
        implements: Vec::new(),
    }
}

impl Doc {
    fn def(mut self, symbol: &str) -> Self {
        self.defs.push(symbol.to_string());
        self
    }
    fn reference(mut self, symbol: &str) -> Self {
        self.refs.push(symbol.to_string());
        self
    }
    fn implements(mut self, symbol: &str, target: &str) -> Self {
        self.implements
            .push((symbol.to_string(), target.to_string()));
        self
    }
}

fn index(tool: &str, tool_version: &str, docs: Vec<Doc>) -> Index {
    let mut info = ToolInfo::new();
    info.name = tool.to_string();
    info.version = tool_version.to_string();
    let mut meta = Metadata::new();
    meta.tool_info = Some(info).into();
    let mut idx = Index::new();
    idx.metadata = Some(meta).into();
    for d in docs {
        let mut out = Document::new();
        out.relative_path = d.path;
        for (i, symbol) in d.defs.iter().enumerate() {
            out.occurrences.push(occurrence(symbol, DEFINITION, i));
        }
        for (i, symbol) in d.refs.iter().enumerate() {
            out.occurrences.push(occurrence(symbol, 0, 100 + i));
        }
        for (symbol, target) in d.implements {
            let mut info = SymbolInformation::new();
            info.symbol = symbol;
            let mut rel = Relationship::new();
            rel.symbol = target;
            rel.is_implementation = true;
            info.relationships.push(rel);
            out.symbols.push(info);
        }
        idx.documents.push(out);
    }
    idx
}

fn occurrence(symbol: &str, roles: i32, line: usize) -> Occurrence {
    let mut o = Occurrence::new();
    o.symbol = symbol.to_string();
    o.symbol_roles = roles;
    let line = i32::try_from(line).unwrap();
    o.range = vec![line, 0, 3];
    o
}

fn rs(package: &str, version: &str, descriptor: &str) -> String {
    format!("rust-analyzer cargo {package} {version} {descriptor}")
}

fn go(module: &str, version: &str, descriptor: &str) -> String {
    format!("scip-go gomod {module} {version} {descriptor}")
}

const STD_SYMBOL_DESC: &str = "option/Option#";
const SERDE_SYMBOL_DESC: &str = "Serialize#";

/// A two-document Rust index for `repo` with its own crate at `own_version`.
/// It carries one of each fact shape, one external symbol from `std`, one from
/// `serde`, and three local symbols (two occurrences and one relationship).
fn rust_index(repo: &str, own_version: &str, with_std: bool, with_serde: bool) -> Index {
    let target_ty = rs(repo, own_version, "a/Foo#");
    let imp = rs(repo, own_version, "b/Impl#");
    let bar = rs(repo, own_version, "b/bar().");
    let mut a = doc("src/a.rs")
        .def(&target_ty)
        .def("local 1")
        .reference("local 2");
    if with_std {
        a = a.reference(&rs("std", "1.97.1", STD_SYMBOL_DESC));
    }
    if with_serde {
        a = a.reference(&rs("serde", "1.0.0", SERDE_SYMBOL_DESC));
    }
    let b = doc("src/b.rs")
        .def(&imp)
        .def(&bar)
        .implements(&imp, &target_ty)
        .implements("local 3", &target_ty);
    index("rust-analyzer", "1.97.1", vec![a, b])
}

/// What `rust_index(repo, ..)` must load as `code/` facts. The own crate's
/// version is `.` in every symbol (SI-3).
fn rust_expected(repo: &str, with_std: bool, with_serde: bool) -> BTreeSet<Triple> {
    let target_ty = rs(repo, ".", "a/Foo#");
    let imp = rs(repo, ".", "b/Impl#");
    let bar = rs(repo, ".", "b/bar().");
    let da = format!("{repo}:src/a.rs");
    let db = format!("{repo}:src/b.rs");
    let mut want = BTreeSet::new();
    want.insert(t(&da, "code/defines", &target_ty));
    want.insert(t(&db, "code/defines", &imp));
    want.insert(t(&db, "code/defines", &bar));
    want.insert(t(&imp, "code/implements", &target_ty));
    if with_std {
        want.insert(t(
            &da,
            "code/references",
            &rs("std", "1.97.1", STD_SYMBOL_DESC),
        ));
        want.insert(t(repo, "code/depends_on", "std"));
    }
    if with_serde {
        want.insert(t(
            &da,
            "code/references",
            &rs("serde", "1.0.0", SERDE_SYMBOL_DESC),
        ));
        want.insert(t(repo, "code/depends_on", "serde"));
    }
    want
}

const HEALTHY_RUST_LOG: &str = "rust-analyzer 1.97.1\nindexed 2 files\n";
const DEGRADED_RUST_LOG: &str = "rust-analyzer 1.97.1\nERROR can't load standard library, try installing rust-src\nindexed 2 files\n";

/// A Go index for module `github.com/x/<repo>`; `with_pkg` leaves out the
/// document for `pkg/a.go`.
fn go_index(repo: &str, with_pkg: bool) -> Index {
    let module = format!("github.com/x/{repo}");
    let main_sym = go(&module, "v1.2.3", &format!("`{module}`/main()."));
    let run_sym = go(&module, "v1.2.3", &format!("`{module}/pkg`/Run()."));
    let mut docs = vec![doc("main.go").def(&main_sym).reference(&run_sym)];
    if with_pkg {
        docs.push(doc("pkg/a.go").def(&run_sym));
    }
    index("scip-go", "0.2.7", docs)
}

const GO_LOG_OK: &str = "scip-go 0.2.7\nexit: 0\n";
const GO_LOG_FAILED: &str = "scip-go 0.2.7\nexit: 2\n";

// ------------------------------------------------------------------ the world

struct World {
    dir: TempDir,
}

impl World {
    fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
        }
    }

    fn db(&self) -> PathBuf {
        self.dir.path().join("facts.db")
    }

    /// A git checkout of a Rust crate with tracked `src/a.rs` and `src/b.rs`,
    /// plus any extra tracked files.
    fn rust_checkout(&self, repo: &str, extra: &[(&str, &str)]) -> PathBuf {
        let mut files = vec![
            ("Cargo.toml", "[package]\nname = \"x\"\n"),
            ("src/a.rs", "pub struct Foo;\n"),
            ("src/b.rs", "pub struct Impl;\n"),
        ];
        files.extend_from_slice(extra);
        self.checkout(repo, &files)
    }

    /// A git checkout of a Go module with two tracked source files and one
    /// tracked test file.
    fn go_checkout(&self, repo: &str) -> PathBuf {
        self.checkout(
            repo,
            &[
                ("go.mod", "module x\n"),
                ("main.go", "package main\n"),
                ("pkg/a.go", "package pkg\n"),
                ("pkg/a_test.go", "package pkg\n"),
            ],
        )
    }

    fn checkout(&self, repo: &str, files: &[(&str, &str)]) -> PathBuf {
        let root = self.dir.path().join(format!("checkout-{repo}"));
        for (path, text) in files {
            let full = root.join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, text).unwrap();
        }
        let git = |args: &[&str]| {
            let status = Command::new("git")
                .current_dir(&root)
                .args(args)
                .status()
                .unwrap();
            assert!(status.success(), "git {args:?} failed");
        };
        git(&["init", "-q"]);
        git(&["add", "."]);
        root
    }

    /// Write the index and the log, run the ingest, and return the result.
    fn ingest(
        &self,
        repo: &str,
        commit: &str,
        at: i64,
        idx: &Index,
        log: &str,
        checkout: &Path,
    ) -> Out {
        let idx_path = self.dir.path().join(format!("{repo}-{at}.scip"));
        scip::write_message_to_file(&idx_path, idx.clone()).unwrap();
        let log_path = self.dir.path().join(format!("{repo}-{at}.log"));
        fs::write(&log_path, log).unwrap();
        kos(&[
            "facts",
            "ingest-scip",
            "--db",
            &s(&self.db()),
            "--at",
            &at.to_string(),
            "--repo",
            repo,
            "--commit",
            commit,
            "--index",
            &s(&idx_path),
            "--log",
            &s(&log_path),
            "--checkout",
            &s(checkout),
        ])
    }

    fn index_sha(&self, repo: &str, at: i64) -> String {
        let bytes = fs::read(self.dir.path().join(format!("{repo}-{at}.scip"))).unwrap();
        format!("{:x}", Sha256::digest(&bytes))
    }

    fn facts_at(&self, db: &Path, at: i64) -> BTreeSet<Triple> {
        let out = kos(&[
            "facts",
            "as-of",
            "--db",
            &s(db),
            "--at",
            &at.to_string(),
            "--json",
        ]);
        assert_eq!(out.code, 0, "as-of failed: {}", out.stderr);
        out.stdout
            .lines()
            .map(|l| {
                let v: Value = serde_json::from_str(l).unwrap();
                t(
                    v["e"].as_str().unwrap(),
                    v["a"].as_str().unwrap(),
                    v["v"].as_str().unwrap(),
                )
            })
            .collect()
    }

    fn code_facts_at(&self, at: i64) -> BTreeSet<Triple> {
        self.facts_at(&self.db(), at)
            .into_iter()
            .filter(|(_, a, _)| a.starts_with("code/"))
            .collect()
    }

    /// `(op, fact)` for the `code/` facts that changed from `from` to `to`.
    fn code_diff(&self, from: i64, to: i64) -> BTreeSet<(String, Triple)> {
        self.diff(&self.db(), from, to)
            .into_iter()
            .filter(|(_, (_, a, _))| a.starts_with("code/"))
            .collect()
    }

    fn diff(&self, db: &Path, from: i64, to: i64) -> BTreeSet<(String, Triple)> {
        let out = kos(&[
            "facts",
            "diff",
            "--db",
            &s(db),
            "--from",
            &from.to_string(),
            "--to",
            &to.to_string(),
            "--json",
        ]);
        assert_eq!(out.code, 0, "diff failed: {}", out.stderr);
        out.stdout
            .lines()
            .map(|l| {
                let v: Value = serde_json::from_str(l).unwrap();
                (
                    v["op"].as_str().unwrap().to_string(),
                    t(
                        v["e"].as_str().unwrap(),
                        v["a"].as_str().unwrap(),
                        v["v"].as_str().unwrap(),
                    ),
                )
            })
            .collect()
    }

    fn export(&self, db: &Path) -> String {
        let out = kos(&["facts", "export", "--db", &s(db)]);
        assert_eq!(out.code, 0, "export failed: {}", out.stderr);
        out.stdout
    }

    /// Facts of one entity at `at`, as `attribute -> value`.
    fn entity_at(&self, entity: &str, at: i64) -> Vec<(String, String)> {
        let out = kos(&[
            "facts",
            "as-of",
            "--db",
            &s(&self.db()),
            "--at",
            &at.to_string(),
            "--entity",
            entity,
            "--json",
        ]);
        assert_eq!(out.code, 0, "as-of failed: {}", out.stderr);
        out.stdout
            .lines()
            .map(|l| {
                let v: Value = serde_json::from_str(l).unwrap();
                (
                    v["a"].as_str().unwrap().to_string(),
                    v["v"].as_str().unwrap().to_string(),
                )
            })
            .collect()
    }
}

fn accepted(out: &Out) {
    assert_eq!(
        out.code, 0,
        "ingest-scip should exit 0; stderr: {}",
        out.stderr
    );
}

fn refused(out: &Out, reason_word: &str) {
    assert!(
        out.stderr.contains("GAP kos@c1:") && out.stderr.contains(reason_word),
        "a refusal prints `GAP kos@c1: <reason>` naming {reason_word:?}; exit {}, stderr: {}",
        out.code,
        out.stderr
    );
    assert_ne!(out.code, 0, "a refusal exits nonzero");
}

// ------------------------------------------------------------------------ SI-1

#[test]
fn ingest_reads_a_fixture_index() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let out = w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    );
    accepted(&out);
    let facts = w.code_facts_at(1);
    assert!(
        facts.contains(&t(
            "kos:src/a.rs",
            "code/defines",
            &rs("kos", ".", "a/Foo#")
        )),
        "src/a.rs defines Foo; got {facts:?}"
    );
    assert!(
        facts.contains(&t(
            "kos:src/b.rs",
            "code/defines",
            &rs("kos", ".", "b/bar().")
        )),
        "src/b.rs defines bar; got {facts:?}"
    );
}

// ------------------------------------------------------------------------ SI-2

#[test]
fn ingest_emits_the_four_shapes() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    assert_eq!(w.code_facts_at(1), rust_expected("kos", true, true));
}

// ------------------------------------------------------------------------ SI-3

#[test]
fn own_version_bump_changes_no_fact() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    accepted(&w.ingest(
        "kos",
        "c2",
        2,
        &rust_index("kos", "0.1.1", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    let changed = w.code_diff(1, 2);
    assert!(
        changed.is_empty(),
        "an own-version bump must change no code/ fact; changed: {changed:?}"
    );
}

// ------------------------------------------------------------------------ SI-4

#[test]
fn out_of_root_documents_are_skipped_and_counted() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let mut idx = rust_index("kos", "0.1.0", true, true);
    let stray = index(
        "rust-analyzer",
        "1.97.1",
        vec![doc("../cache/z.rs").def(&rs("kos", "0.1.0", "cache/Z#"))],
    );
    idx.documents.push(stray.documents[0].clone());
    let out = w.ingest("kos", "c1", 1, &idx, HEALTHY_RUST_LOG, &co);
    accepted(&out);
    assert_eq!(
        w.code_facts_at(1),
        rust_expected("kos", true, true),
        "the stray document adds no fact"
    );
    assert!(
        out.stdout.contains("out_of_root_documents=1"),
        "the load reports the dropped document; stdout: {}",
        out.stdout
    );
}

#[test]
fn same_path_in_two_repos_is_two_entities() {
    let w = World::new();
    let kos_co = w.rust_checkout("kos", &[]);
    let marvel_co = w.rust_checkout("marvel", &[]);
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &kos_co,
    ));
    accepted(&w.ingest(
        "marvel",
        "m1",
        2,
        &rust_index("marvel", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &marvel_co,
    ));
    let facts = w.code_facts_at(2);
    let entities: BTreeSet<&str> = facts
        .iter()
        .filter(|(_, a, _)| a == "code/defines")
        .map(|(e, _, _)| e.as_str())
        .collect();
    assert!(
        entities.contains("kos:src/a.rs"),
        "kos:src/a.rs; got {entities:?}"
    );
    assert!(
        entities.contains("marvel:src/a.rs"),
        "marvel:src/a.rs; got {entities:?}"
    );
}

#[test]
fn local_symbols_are_skipped() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let out = w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    );
    accepted(&out);
    for (e, a, v) in w.code_facts_at(1) {
        assert!(
            !e.starts_with("local ") && !v.starts_with("local "),
            "a local symbol reached the store: {e} {a} {v}"
        );
    }
    assert!(
        out.stdout.contains("local_symbols=3"),
        "the load counts the three local items it dropped; stdout: {}",
        out.stdout
    );
}

// ------------------------------------------------------------------------ SI-6

#[test]
fn every_code_row_resolves_to_a_complete_load_entity() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    accepted(&w.ingest(
        "kos",
        "c2",
        2,
        &rust_index("kos", "0.1.0", true, false),
        HEALTHY_RUST_LOG,
        &co,
    ));
    let sha = [w.index_sha("kos", 1), w.index_sha("kos", 2)];
    let commit = ["c1", "c2"];

    let rows: Vec<Value> = w
        .export(&w.db())
        .lines()
        .skip(1)
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|r: &Value| r["a"].as_str().unwrap().starts_with("code/"))
        .collect();
    assert!(!rows.is_empty(), "no code/ rows were loaded");

    let mut checked = BTreeSet::new();
    for row in &rows {
        let mut versions = vec![row["valid_from"].as_i64().unwrap()];
        if let Some(end) = row["valid_to"].as_i64() {
            versions.push(end);
        }
        checked.extend(versions);
    }
    for n in checked {
        let load = w.entity_at(&format!("load/kos@{n}"), n);
        let get = |k: &str| {
            load.iter()
                .find(|(a, _)| a == k)
                .unwrap_or_else(|| panic!("load/kos@{n} lacks {k}; has {load:?}"))
                .1
                .clone()
        };
        let i = usize::try_from(n - 1).unwrap();
        assert_eq!(get("prov/repo"), "kos");
        assert_eq!(get("prov/commit"), commit[i]);
        assert_eq!(get("prov/indexer"), "rust-analyzer");
        assert_eq!(get("prov/indexer_version"), "1.97.1");
        assert_eq!(get("prov/index_sha256"), sha[i]);
        assert_eq!(get("prov/grade"), "compiler");
        assert_eq!(load.len(), 6, "exactly six provenance values; got {load:?}");
    }
}

// ------------------------------------------------------------------------ SI-7

#[test]
fn missing_fact_ends_its_interval() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    accepted(&w.ingest(
        "kos",
        "c2",
        2,
        &rust_index("kos", "0.1.0", true, false),
        HEALTHY_RUST_LOG,
        &co,
    ));

    let serde_ref = t(
        "kos:src/a.rs",
        "code/references",
        &rs("serde", "1.0.0", SERDE_SYMBOL_DESC),
    );
    let serde_dep = t("kos", "code/depends_on", "serde");
    let before = w.code_facts_at(1);
    let after = w.code_facts_at(2);
    let dropped: BTreeSet<Triple> = before.difference(&after).cloned().collect();
    assert_eq!(dropped, BTreeSet::from([serde_ref.clone(), serde_dep]));

    let diff = w.code_diff(1, 2);
    let want: BTreeSet<(String, Triple)> = dropped
        .iter()
        .map(|f| ("removed".to_string(), f.clone()))
        .collect();
    assert_eq!(diff, want, "diff(1,2) equals the set difference");

    let row = w
        .export(&w.db())
        .lines()
        .skip(1)
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|r| r["e"] == serde_ref.0 && r["a"] == serde_ref.1 && r["v"] == serde_ref.2)
        .expect("the dropped fact keeps a row");
    assert_eq!(
        row["valid_to"].as_i64(),
        Some(2),
        "the dropped fact's interval ends at 2"
    );
}

#[test]
fn deleted_line_mutant_is_caught() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    // The mutant: one fact line (the definition of bar) is deleted from the
    // second load's index. Ending an interval is the only way the store can
    // show the deletion, so a store that never ends one does not catch it.
    let mut mutant = rust_index("kos", "0.1.0", true, true);
    let bar = rs("kos", "0.1.0", "b/bar().");
    mutant.documents[1].occurrences.retain(|o| o.symbol != bar);
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    accepted(&w.ingest("kos", "c2", 2, &mutant, HEALTHY_RUST_LOG, &co));

    let expected = rust_expected("kos", true, true);
    assert_eq!(
        w.code_facts_at(1),
        expected,
        "the state check passes at the intact version"
    );
    assert_ne!(
        w.code_facts_at(2),
        expected,
        "the state check fails at the mutated version"
    );
    let no_change: BTreeSet<(String, Triple)> = BTreeSet::new();
    let diff = w.code_diff(1, 2);
    assert_ne!(diff, no_change, "the diff check fails at the next version");
    assert_eq!(
        diff,
        BTreeSet::from([(
            "removed".to_string(),
            t("kos:src/b.rs", "code/defines", &rs("kos", ".", "b/bar()."))
        )]),
        "the diff names exactly the deleted line"
    );
}

// ------------------------------------------------------------------------ SI-8

#[test]
#[ignore = "red until SI-8 lands in G3 (aae-orc-yto7v); run with --ignored"]
fn healthy_rust_index_is_accepted() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let out = w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    );
    accepted(&out);
    assert!(
        !out.stderr.contains("GAP"),
        "no GAP on a healthy index; stderr: {}",
        out.stderr
    );
}

#[test]
#[ignore = "red until SI-8 lands in G3 (aae-orc-yto7v); run with --ignored"]
fn rust_index_without_std_is_refused() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let out = w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", false, true),
        DEGRADED_RUST_LOG,
        &co,
    );
    refused(&out, "standard library");
}

#[test]
#[ignore = "red until SI-8 lands in G3 (aae-orc-yto7v); run with --ignored"]
fn go_index_missing_a_document_is_refused() {
    let w = World::new();
    let co = w.go_checkout("kos");
    let out = w.ingest("kos", "c1", 1, &go_index("kos", false), GO_LOG_OK, &co);
    refused(&out, "pkg/a.go");
}

#[test]
#[ignore = "red until SI-8 lands in G3 (aae-orc-yto7v); run with --ignored"]
fn go_index_with_nonzero_exit_is_refused() {
    let w = World::new();
    let co = w.go_checkout("kos");
    let out = w.ingest("kos", "c1", 1, &go_index("kos", true), GO_LOG_FAILED, &co);
    refused(&out, "exit");
}

#[test]
#[ignore = "red until SI-8 lands in G3 (aae-orc-yto7v); run with --ignored"]
fn refusal_loads_nothing() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    let before = w.export(&w.db());
    let out = w.ingest(
        "kos",
        "c2",
        2,
        &rust_index("kos", "0.1.0", false, true),
        DEGRADED_RUST_LOG,
        &co,
    );
    assert!(
        out.stderr.contains("GAP kos@c2:"),
        "the refusal is a GAP; stderr: {}",
        out.stderr
    );
    assert_eq!(
        w.export(&w.db()),
        before,
        "version count and rows are unchanged by a refusal"
    );
}

// ------------------------------------------------------------------------ SI-9

#[test]
fn loading_repo_a_never_retracts_repo_b() {
    let w = World::new();
    let kos_co = w.rust_checkout("kos", &[]);
    let marvel_co = w.rust_checkout("marvel", &[]);
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &kos_co,
    ));
    accepted(&w.ingest(
        "marvel",
        "m1",
        2,
        &rust_index("marvel", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &marvel_co,
    ));
    // kos loads again and drops a fact; marvel's facts must be untouched.
    accepted(&w.ingest(
        "kos",
        "c2",
        3,
        &rust_index("kos", "0.1.0", true, false),
        HEALTHY_RUST_LOG,
        &kos_co,
    ));

    // kos's dropped facts end, and marvel's stay exactly as loaded.
    let mut expected = rust_expected("kos", true, false);
    expected.extend(rust_expected("marvel", true, true));
    assert_eq!(
        w.code_facts_at(3),
        expected,
        "kos's load ended its own dropped facts and left marvel's alone"
    );
    let removed_marvel: Vec<_> = w
        .code_diff(2, 3)
        .into_iter()
        .filter(|(_, (e, _, v))| e.starts_with("marvel") || v.contains(" marvel "))
        .collect();
    assert!(
        removed_marvel.is_empty(),
        "kos's load changed marvel's facts: {removed_marvel:?}"
    );
}

// ----------------------------------------------------------------------- SI-10

#[test]
#[ignore = "red until SI-10 lands in G4 (aae-orc-yto7v); run with --ignored"]
fn harvest_refuses_code_attributes() {
    let dir = TempDir::new().unwrap();
    let mut store = FactStore::open(&dir.path().join("facts.db")).unwrap();
    let code = Fact::new(
        "kos:src/a.rs",
        "code/defines",
        "rust-analyzer cargo kos . a/Foo#",
    );
    let load = Fact::new("load/kos@1", "prov/repo", "kos");
    store
        .assert_facts(Version::new(1), &[code.clone(), load.clone()])
        .unwrap();
    let mut source = std::collections::BTreeMap::new();
    source.insert("repo".to_string(), "kos".to_string());
    source.insert("commit".to_string(), "c1".to_string());
    for pick in [code, load] {
        let result = store.harvest(Version::new(1), std::slice::from_ref(&pick), &source);
        let err = result.expect_err(&format!("a harvest of {pick:?} must be refused"));
        assert!(
            err.to_string().contains("JSONL"),
            "the refusal names JSONL; got: {err}"
        );
    }
}

#[test]
#[ignore = "red until SI-10 lands in G4 (aae-orc-yto7v); run with --ignored"]
fn jsonl_round_trip_keeps_code_facts_intervals_and_load_entities() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    accepted(&w.ingest(
        "kos",
        "c2",
        2,
        &rust_index("kos", "0.1.0", true, false),
        HEALTHY_RUST_LOG,
        &co,
    ));

    let copy = w.dir.path().join("copy.db");
    let dump = w.dir.path().join("dump.jsonl");
    let out = kos(&["facts", "export", "--db", &s(&w.db()), "--out", &s(&dump)]);
    assert_eq!(out.code, 0, "export: {}", out.stderr);
    let out = kos(&["facts", "import", "--db", &s(&copy), &s(&dump)]);
    assert_eq!(out.code, 0, "import: {}", out.stderr);

    for at in [1, 2] {
        let src = w.facts_at(&w.db(), at);
        assert!(
            src.iter().any(|(e, _, _)| e.starts_with("load/")),
            "the source store holds load entities at {at}"
        );
        assert_eq!(src, w.facts_at(&copy, at), "state equal at version {at}");
    }
    assert_eq!(w.diff(&w.db(), 1, 2), w.diff(&copy, 1, 2), "diff equal");
    assert_eq!(w.export(&w.db()), w.export(&copy), "intervals equal");
}

// ----------------------------------------------------------------------- SI-11

#[test]
#[ignore = "red until SI-11 lands in G5 (aae-orc-yto7v); run with --ignored"]
fn crate_names_come_from_the_list() {
    let w = World::new();
    let co = w.rust_checkout(
        "kos",
        &[
            (".kos/crate-names.txt", "serde-saphyr\n"),
            (
                "_kos/nodes/frontier/q-storage.yaml",
                "id: q-storage\ntext: parsing moved to serde-saphyr and away from rand\n",
            ),
        ],
    );
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    accepted(&w.ingest(
        "kos",
        "c2",
        2,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    for at in [1, 2] {
        let names: BTreeSet<Triple> = w
            .facts_at(&w.db(), at)
            .into_iter()
            .filter(|(_, a, _)| a == "kos/names-dep")
            .collect();
        assert_eq!(
            names,
            BTreeSet::from([t("q-storage", "kos/names-dep", "serde-saphyr")]),
            "at version {at}: the listed name makes a fact and the unlisted one does not"
        );
    }
}
