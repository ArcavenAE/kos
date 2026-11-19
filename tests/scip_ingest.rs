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
        self.ingest_with(repo, commit, at, idx, log, checkout, None)
    }

    /// As `ingest`, and with `go_list` written to a file and passed as
    /// `--golist` when it is given.
    #[allow(clippy::too_many_arguments)]
    fn ingest_with(
        &self,
        repo: &str,
        commit: &str,
        at: i64,
        idx: &Index,
        log: &str,
        checkout: &Path,
        go_list: Option<&str>,
    ) -> Out {
        let idx_path = self.dir.path().join(format!("{repo}-{at}.scip"));
        scip::write_message_to_file(&idx_path, idx.clone()).unwrap();
        let log_path = self.dir.path().join(format!("{repo}-{at}.log"));
        fs::write(&log_path, log).unwrap();
        let mut args = vec![
            "facts".to_string(),
            "ingest-scip".to_string(),
            "--db".to_string(),
            s(&self.db()),
            "--at".to_string(),
            at.to_string(),
            "--repo".to_string(),
            repo.to_string(),
            "--commit".to_string(),
            commit.to_string(),
            "--index".to_string(),
            s(&idx_path),
            "--log".to_string(),
            s(&log_path),
            "--checkout".to_string(),
            s(checkout),
        ];
        if let Some(text) = go_list {
            let path = self.dir.path().join(format!("{repo}-{at}.golist.json"));
            fs::write(&path, text).unwrap();
            args.push("--golist".to_string());
            args.push(s(&path));
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        kos(&refs)
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
#[ignore = "red until the review follow-ups land (aae-orc-yto7v); run with --ignored"]
fn the_filter_line_counts_facts_as_well_as_documents() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let mut idx = rust_index("kos", "0.1.0", true, true);
    idx.documents.push(
        index(
            "rust-analyzer",
            "1.97.1",
            vec![doc("../cache/z.rs").def(&rs("kos", "0.1.0", "cache/Z#"))],
        )
        .documents[0]
            .clone(),
    );
    let out = w.ingest("kos", "c1", 1, &idx, HEALTHY_RUST_LOG, &co);
    accepted(&out);
    assert!(
        out.stdout
            .contains("out_of_root_documents=1 out_of_root_facts=1 local_symbols=3 local_facts=3"),
        "the line counts the facts each filter dropped; stdout: {}",
        out.stdout
    );
}

#[test]
#[ignore = "red until the review follow-ups land (aae-orc-yto7v); run with --ignored"]
fn a_dotdot_segment_leaves_the_root() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let mut idx = rust_index("kos", "0.1.0", true, true);
    idx.documents.push(
        index(
            "rust-analyzer",
            "1.97.1",
            vec![doc("src/../../cache/z.rs").def(&rs("kos", "0.1.0", "cache/Z#"))],
        )
        .documents[0]
            .clone(),
    );
    let out = w.ingest("kos", "c1", 1, &idx, HEALTHY_RUST_LOG, &co);
    accepted(&out);
    assert_eq!(
        w.code_facts_at(1),
        rust_expected("kos", true, true),
        "a path with a .. segment adds no fact"
    );
    assert!(
        out.stdout.contains("out_of_root_documents=1"),
        "stdout: {}",
        out.stdout
    );
}

#[test]
#[ignore = "red until the review follow-ups land (aae-orc-yto7v); run with --ignored"]
fn a_repo_name_with_a_delimiter_is_refused() {
    for bad in ["a:b", "a@b"] {
        let w = World::new();
        let co = w.rust_checkout("kos", &[]);
        let out = w.ingest(
            bad,
            "c1",
            1,
            &rust_index("kos", "0.1.0", true, true),
            HEALTHY_RUST_LOG,
            &co,
        );
        assert_ne!(out.code, 0, "{bad:?} is refused");
        assert!(
            out.stderr.contains("--repo") && out.stderr.contains(bad),
            "the refusal names the flag and the value; stderr: {}",
            out.stderr
        );
        assert!(!w.db().exists(), "a refused repo name creates no store");
    }
}

#[test]
#[ignore = "red until the review follow-ups land (aae-orc-yto7v); run with --ignored"]
fn an_empty_indexer_version_is_refused() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let mut idx = rust_index("kos", "0.1.0", true, true);
    idx.metadata
        .mut_or_insert_default()
        .tool_info
        .mut_or_insert_default()
        .version = String::new();
    let out = w.ingest("kos", "c1", 1, &idx, HEALTHY_RUST_LOG, &co);
    refused(&out, "version");
    assert!(!w.db().exists(), "a refusal creates no store");
}

#[test]
#[ignore = "red until the review follow-ups land (aae-orc-yto7v); run with --ignored"]
fn a_go_list_file_with_a_rust_index_is_refused() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let listed = format!("{}exit: 0\n", go_list(&[("", &["main.go"])]));
    let out = w.ingest_with(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
        Some(&listed),
    );
    refused(&out, "--golist");
}

#[test]
#[ignore = "red until the review follow-ups land (aae-orc-yto7v); run with --ignored"]
fn a_go_list_file_without_its_exit_line_is_refused() {
    let w = World::new();
    let co = w.go_checkout("kos");
    let listed = go_list(&[("", &["main.go"]), ("/pkg", &["a.go"])]);
    let out = w.ingest_with(
        "kos",
        "c1",
        1,
        &go_index("kos", true),
        GO_LOG_OK,
        &co,
        Some(&listed),
    );
    refused(&out, "exit");
}

#[test]
#[ignore = "red until the review follow-ups land (aae-orc-yto7v); run with --ignored"]
fn a_go_list_that_exited_nonzero_is_refused() {
    let w = World::new();
    let co = w.go_checkout("kos");
    let listed = format!(
        "{}exit: 1\n",
        go_list(&[("", &["main.go"]), ("/pkg", &["a.go"])])
    );
    let out = w.ingest_with(
        "kos",
        "c1",
        1,
        &go_index("kos", true),
        GO_LOG_OK,
        &co,
        Some(&listed),
    );
    refused(&out, "exit: 1");
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
fn a_failing_write_leaves_the_store_unchanged() {
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

    // Make the second load's last write, its load entity, fail after the
    // facts it ends and asserts have been written.
    let conn = rusqlite::Connection::open(w.db()).unwrap();
    conn.execute_batch(
        "CREATE TRIGGER fail_on_load BEFORE INSERT ON facts WHEN NEW.e = 'load/kos@2'
         BEGIN SELECT RAISE(ABORT, 'forced failure'); END;",
    )
    .unwrap();
    drop(conn);

    let out = w.ingest(
        "kos",
        "c2",
        2,
        &rust_index("kos", "0.1.0", true, false),
        HEALTHY_RUST_LOG,
        &co,
    );
    assert_ne!(
        out.code, 0,
        "the forced failure surfaces; stdout: {}",
        out.stdout
    );
    assert_eq!(
        w.export(&w.db()),
        before,
        "the facts the load ended were not kept when its last write failed"
    );
}

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
    // Intact at version 1, mutant at version 2. Only an ended interval shows
    // the deletion, so a store that only adds facts does not catch it.
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let bar = rs("kos", "0.1.0", "b/bar().");
    let mut mutant = rust_index("kos", "0.1.0", true, true);
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
    let bar_facts: BTreeSet<Triple> = expected
        .iter()
        .filter(|(e, _, v)| e.contains("b/bar().") || v.contains("b/bar()."))
        .cloned()
        .collect();
    assert!(!bar_facts.is_empty(), "the fixture has facts naming bar");
    let without_bar: BTreeSet<Triple> = expected.difference(&bar_facts).cloned().collect();
    assert_eq!(
        w.code_facts_at(1),
        expected,
        "the state check passes at the intact version"
    );
    assert_eq!(
        w.code_facts_at(2),
        without_bar,
        "the state check at the mutated version lacks exactly bar's facts"
    );
    let removed: BTreeSet<(String, Triple)> = bar_facts
        .iter()
        .map(|f| ("removed".to_string(), f.clone()))
        .collect();
    assert_eq!(
        w.code_diff(1, 2),
        removed,
        "the diff is exactly the removal of bar's facts"
    );

    // The other direction: mutant at version 1, intact at version 2.
    let back = World::new();
    let co = back.rust_checkout("kos", &[]);
    accepted(&back.ingest("kos", "c1", 1, &mutant, HEALTHY_RUST_LOG, &co));
    accepted(&back.ingest(
        "kos",
        "c2",
        2,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    assert_eq!(back.code_facts_at(1), without_bar);
    assert_eq!(back.code_facts_at(2), expected);
    let added: BTreeSet<(String, Triple)> = bar_facts
        .iter()
        .map(|f| ("added".to_string(), f.clone()))
        .collect();
    assert_eq!(
        back.code_diff(1, 2),
        added,
        "the diff is exactly bar's facts added"
    );
}

// ------------------------------------------------------------------------ SI-8

#[test]
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
fn a_nested_non_member_crate_is_not_required() {
    let w = World::new();
    // kos tracks probe crates like this one, and rust-analyzer loads only the
    // root workspace, so a healthy index has no document for it.
    let co = w.rust_checkout(
        "kos",
        &[
            (
                "_kos/probes/m/trav/Cargo.toml",
                "[package]\nname = \"trav\"\n",
            ),
            ("_kos/probes/m/trav/src/main.rs", "fn main() {}\n"),
        ],
    );
    let out = w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    );
    accepted(&out);
    assert!(!out.stderr.contains("GAP"), "stderr: {}", out.stderr);
}

#[test]
fn std_symbols_are_required_even_when_the_log_is_healthy() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[]);
    let out = w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", false, true),
        HEALTHY_RUST_LOG,
        &co,
    );
    refused(&out, "std");
}

#[test]
fn a_tracked_root_source_without_a_document_is_refused() {
    let w = World::new();
    let co = w.rust_checkout("kos", &[("src/c.rs", "pub struct C;\n")]);
    let out = w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    );
    refused(&out, "src/c.rs");
}

#[test]
fn a_workspace_members_sources_are_required() {
    let w = World::new();
    let co = w.rust_checkout(
        "kos",
        &[
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\n    \"crates/a\",\n    \"tools/*\",\n]\n",
            ),
            ("crates/a/src/lib.rs", "pub struct A;\n"),
            ("tools/t1/src/main.rs", "fn main() {}\n"),
            ("crates/other/src/lib.rs", "pub struct O;\n"),
        ],
    );
    let out = w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    );
    // crates/a and tools/t1 are members; crates/other is not.
    assert_ne!(out.code, 0, "members without documents are refused");
    assert!(
        out.stderr.contains("crates/a/src/lib.rs") && out.stderr.contains("tools/t1/src/main.rs"),
        "both members are named; stderr: {}",
        out.stderr
    );
    assert!(
        !out.stderr.contains("crates/other"),
        "a crate that is not a member is not required; stderr: {}",
        out.stderr
    );
}

#[test]
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
fn go_index_missing_a_document_is_refused() {
    let w = World::new();
    let co = w.go_checkout("kos");
    let out = w.ingest("kos", "c1", 1, &go_index("kos", false), GO_LOG_OK, &co);
    refused(&out, "pkg/a.go");
}

/// `go list -json ./...` output for a module whose root is `/work/kos`: one
/// object per package, with the files the package builds.
fn go_list(packages: &[(&str, &[&str])]) -> String {
    packages
        .iter()
        .map(|(dir, files)| {
            let files: Vec<String> = files.iter().map(|f| format!("{f:?}")).collect();
            format!(
                "{{\"Dir\": \"/work/kos{dir}\", \"ImportPath\": \"github.com/x/kos{dir}\", \"GoFiles\": [{}], \"Module\": {{\"Dir\": \"/work/kos\"}}}}\n",
                files.join(", ")
            )
        })
        .collect()
}

fn go_checkout_with_ignored_file(w: &World) -> PathBuf {
    w.checkout(
        "kos",
        &[
            ("go.mod", "module x\n"),
            ("main.go", "package main\n"),
            ("pkg/a.go", "package pkg\n"),
            ("tools.go", "//go:build ignore\npackage main\n"),
        ],
    )
}

#[test]
fn go_file_set_comes_from_go_list() {
    let w = World::new();
    let co = go_checkout_with_ignored_file(&w);
    let idx = go_index("kos", true);

    // Without the list, the path approximation requires a document for the
    // build-ignored tools.go and refuses.
    let out = w.ingest("kos", "c1", 1, &idx, GO_LOG_OK, &co);
    refused(&out, "tools.go");

    // With go list's own file set, tools.go is not a file the module builds.
    let listed = go_list(&[("", &["main.go"]), ("/pkg", &["a.go"])]);
    let out = w.ingest_with("kos", "c1", 1, &idx, GO_LOG_OK, &co, Some(&listed));
    accepted(&out);
    assert!(!out.stderr.contains("GAP"), "stderr: {}", out.stderr);
}

#[test]
fn a_file_go_list_names_must_have_a_document() {
    let w = World::new();
    let co = go_checkout_with_ignored_file(&w);
    // The list names pkg/a.go, and the index has no document for it.
    let listed = go_list(&[("", &["main.go"]), ("/pkg", &["a.go"])]);
    let out = w.ingest_with(
        "kos",
        "c1",
        1,
        &go_index("kos", false),
        GO_LOG_OK,
        &co,
        Some(&listed),
    );
    refused(&out, "pkg/a.go");
}

#[test]
fn an_unreadable_go_list_is_refused() {
    let w = World::new();
    let co = w.go_checkout("kos");
    let out = w.ingest_with(
        "kos",
        "c1",
        1,
        &go_index("kos", true),
        GO_LOG_OK,
        &co,
        Some("this is not json"),
    );
    refused(&out, "go list");
    assert!(!w.db().exists(), "a refusal creates no store");
}

#[test]
fn go_index_with_nonzero_exit_is_refused() {
    let w = World::new();
    let co = w.go_checkout("kos");
    let out = w.ingest("kos", "c1", 1, &go_index("kos", true), GO_LOG_FAILED, &co);
    refused(&out, "exit");
}

#[test]
fn healthy_go_index_is_accepted() {
    let w = World::new();
    let co = w.go_checkout("kos");
    let out = w.ingest("kos", "c1", 1, &go_index("kos", true), GO_LOG_OK, &co);
    accepted(&out);
    assert!(
        !out.stderr.contains("GAP"),
        "no GAP on a healthy Go index; stderr: {}",
        out.stderr
    );
    let facts = w.code_facts_at(1);
    assert!(
        facts
            .iter()
            .any(|(e, a, _)| e == "kos:main.go" && a == "code/defines"),
        "main.go defines a symbol; got {facts:?}"
    );
}

#[test]
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
    // kos loads again and drops the serde facts; marvel's facts must be
    // untouched, and kos's dropped facts must really end (the positive control).
    accepted(&w.ingest(
        "kos",
        "c2",
        3,
        &rust_index("kos", "0.1.0", true, false),
        HEALTHY_RUST_LOG,
        &kos_co,
    ));

    let at3 = w.code_facts_at(3);
    for fact in rust_expected("marvel", true, true) {
        assert!(
            at3.contains(&fact),
            "marvel's fact {fact:?} was lost when kos loaded again"
        );
    }
    let dropped: BTreeSet<Triple> = rust_expected("kos", true, true)
        .difference(&rust_expected("kos", true, false))
        .cloned()
        .collect();
    assert!(!dropped.is_empty(), "the second kos load drops facts");
    for fact in &dropped {
        assert!(
            !at3.contains(fact),
            "kos's dropped fact {fact:?} still holds at version 3"
        );
    }
    let removals: BTreeSet<(String, Triple)> = dropped
        .iter()
        .map(|f| ("removed".to_string(), f.clone()))
        .collect();
    assert_eq!(
        w.code_diff(2, 3),
        removals,
        "the only change from 2 to 3 is the removal of kos's dropped facts"
    );
}

// ----------------------------------------------------------------------- SI-10

#[test]
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

    // Code facts enter a store only through JSONL: a harvest file that carries
    // one is refused, names JSONL, and creates no store.
    let yaml = w.dir.path().join("code-facts.yaml");
    fs::write(
        &yaml,
        "kind: harvested-facts\nsource:\n  repo: kos\n  commit: c1\nfacts:\n- e: kos:src/a.rs\n  a: code/defines\n  v: some symbol\n",
    )
    .unwrap();
    let yaml_db = w.dir.path().join("from-yaml.db");
    let out = kos(&[
        "facts",
        "load",
        "--db",
        &s(&yaml_db),
        "--at",
        "1",
        &s(&yaml),
    ]);
    assert_ne!(out.code, 0, "a harvest file with a code/ fact is refused");
    assert!(
        out.stderr.contains("JSONL"),
        "the refusal names JSONL; stderr: {}",
        out.stderr
    );
    assert!(!yaml_db.exists(), "a refused file creates no store");
}

// ----------------------------------------------------------------------- SI-11

#[test]
fn crate_names_come_from_the_list() {
    let w = World::new();
    let co = w.rust_checkout(
        "kos",
        &[
            (".kos/crate-names.txt", "# crates kos names\nserde-saphyr\n"),
            (
                "_kos/nodes/frontier/q-storage.yaml",
                "id: q-storage\ntext: parsing moved to serde-saphyr and away from rand\n",
            ),
            (
                "_kos/nodes/bedrock/elem-other.yaml",
                "id: elem-other\ntext: the serde-saphyr-extras crate is not on the list\n",
            ),
        ],
    );
    let load = |commit: &str, at: i64| {
        accepted(&w.ingest(
            "kos",
            commit,
            at,
            &rust_index("kos", "0.1.0", true, true),
            HEALTHY_RUST_LOG,
            &co,
        ));
    };
    let names_at = |at: i64| -> BTreeSet<Triple> {
        w.facts_at(&w.db(), at)
            .into_iter()
            .filter(|(_, a, _)| a == "kos/names-dep")
            .collect()
    };
    let want = BTreeSet::from([t("kos:q-storage", "kos/names-dep", "serde-saphyr")]);

    load("c1", 1);
    load("c2", 2);
    for at in [1, 2] {
        assert_eq!(
            names_at(at),
            want,
            "at version {at}: a listed name that a node's text names makes a fact; an unlisted name, and a listed name inside a longer one, make none"
        );
    }

    // The list is the source: with the name gone from it, the fact ends.
    fs::write(co.join(".kos/crate-names.txt"), "# no crates yet\n").unwrap();
    load("c3", 3);
    assert_eq!(names_at(2), want, "version 2 keeps its fact");
    assert!(
        names_at(3).is_empty(),
        "a name that left the list ends its fact; got {:?}",
        names_at(3)
    );
}

#[test]
fn an_unreadable_node_file_is_refused_not_skipped() {
    let w = World::new();
    let co = w.rust_checkout(
        "kos",
        &[
            (".kos/crate-names.txt", "serde-saphyr\n"),
            (
                "_kos/nodes/frontier/q-ok.yaml",
                "id: q-ok\ntext: parsing moved to serde-saphyr\n",
            ),
        ],
    );
    let load = |commit: &str, at: i64| {
        w.ingest(
            "kos",
            commit,
            at,
            &rust_index("kos", "0.1.0", true, true),
            HEALTHY_RUST_LOG,
            &co,
        )
    };
    accepted(&load("c1", 1));
    let before = w.export(&w.db());

    // One node file that is not UTF-8: skipping it would end its names facts
    // in history on a load that exits 0.
    fs::write(
        co.join("_kos/nodes/frontier/q-bad.yaml"),
        [0xff_u8, 0xfe, 0x00, 0x41],
    )
    .unwrap();
    let out = load("c2", 2);
    assert_ne!(out.code, 0, "an unreadable node file is refused");
    assert!(
        out.stderr.contains("q-bad.yaml"),
        "the refusal names the file; stderr: {}",
        out.stderr
    );
    assert_eq!(w.export(&w.db()), before, "a refused load changes nothing");
}

#[test]
fn no_crate_list_means_no_names_facts() {
    let w = World::new();
    let co = w.rust_checkout(
        "kos",
        &[(
            "_kos/nodes/frontier/q-storage.yaml",
            "id: q-storage\ntext: parsing moved to serde-saphyr\n",
        )],
    );
    accepted(&w.ingest(
        "kos",
        "c1",
        1,
        &rust_index("kos", "0.1.0", true, true),
        HEALTHY_RUST_LOG,
        &co,
    ));
    let names: BTreeSet<Triple> = w
        .facts_at(&w.db(), 1)
        .into_iter()
        .filter(|(_, a, _)| a == "kos/names-dep")
        .collect();
    assert!(names.is_empty(), "no list, no names facts; got {names:?}");
}
