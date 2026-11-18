# Brief: build the SCIP ingest into the kos fact store (I-484)

Status: DRAFT. A build brief, not a probe. It is locked when its kos PR merges
after review; after that it changes only by dated note.
Date: 2026-11-18.
Ruling: the operator, Desk card r1-ingest, option 1, 2026-11-17 about 21:57Z,
relayed by director: "build the SCIP ingest in kos as its own ticket and
brief, with the probe's lessons as requirements." Option 1 as carded also
files a separate flat ticket for the commit-to-decision link (see "Out of
scope").
Author: the arcaven architect seat. Builder: as the supervisor routes it.
Question node: `question-code-graph-correspondence`, sub-question B.
Follows: `finding-kos-h57b-r1-scip-ingest.md` (probe R1, brief
`brief-r1-scip-ingest.md` with dated notes 1 to 5), finding-kos-ndz7 (indexing
cost and the silent-degradation trap), finding-kos-wre7 (live LSP), Desk kdec-7
(the SQLite valid-time store, aae-orc-xxe3n).

## Why build it

Probe R1 showed that compiler facts in the fact store answer the cross-repo
question live LSP could not answer at all: W7 was one query over kos and
marvel, with document recall and precision of 1.000, and every returned row
traced to the index and commit it came from. Coverage, freshness and
provenance passed at all 100 commits (h57b, Results). This brief turns that
one-off probe into a kos feature, and carries forward what the probe learned
the hard way.

## Starting point

The build starts from finding-kos-h57b and its run report
(`_kos/probes/code-graph-measurements-2026-11-16/r1-run-report.md`, sha256
8f836c9e), not from the probe's code. The probe's ingest was one file,
`src/bin/r1_ingest.rs`, on a local branch `probe/r1-ingest` that was never
pushed and is not on any remote. The one probe source committed is the MDE
test program, `r1_mde.rs`, beside the run report. The store API it targets is
on main: `FactStore` in `src/facts/mod.rs` (`assert_facts`, `retract_facts`,
`facts_as_of`, `diff`) and `export_jsonl` and `import_jsonl` in
`src/facts/exchange.rs`.

## Scope

In:
- reading a SCIP index file that rust-analyzer or scip-go already wrote, for
  one repo at one commit, and loading its facts into a `FactStore`;
- the refusal that stops a degraded index from loading;
- provenance for every loaded fact;
- several repos in one store;
- moving code facts between stores.

Out of scope:
- **The commit-to-decision link.** W6 and W8 were only partly served because
  no kos record links a commit or PR to the decision behind it (h57b, "W6 and
  W8: what is missing"). That gap is on the knowledge side. It is its own flat
  ticket, filed beside this brief's ticket with no edge between them.
- Ranking, which still has no host since forestage was retired (h57b, last
  section).
- Sub-question D of `question-kos-fact-store-substrate` (code facts in a
  separate subgraph). This build keeps the probe's layout, one store with
  `code/` and `kos/` attribute namespaces, and D stays open.
- Node-level provenance in the schema (v0.4, aae-orc-e689).

## Requirements

Every requirement is PROVISIONAL until review rules it, with its source named.
Source classes: **lesson** (h57b's "For a production ingest"), **result** (a
measured outcome or MDE test in h57b), **method** (the R1 brief or its dated
notes, which the probe ran and the finding reports), **new** (this brief adds
it, with the reason given). Each names the red test that would fail without
it. Tests use small fixture indexes committed under `tests/fixtures/scip/`,
never a live toolchain, so they run offline in CI.

**SI-1. Read SCIP through the `scip` crate and write through the store API.**
PROVISIONAL, lesson. The ingest decodes the index with the `scip` crate's
Protobuf bindings and writes only through `FactStore`, never raw SQL. Adding
the crate is a new dependency: its lockfile entries and a `cargo deny check
advisories` result go in the PR body (the probe fetched it under a ruled
deviation, brief note 3).
Red test: `ingest_reads_a_fixture_index`. A two-document Rust fixture yields
the expected `code/` facts. It fails today because no ingest exists.

**SI-2. Four fact shapes.** PROVISIONAL, method (brief, "Arms").
- `code/defines (document, symbol)` for an occurrence with the definition role.
- `code/references (document, symbol)` for any other occurrence.
- `code/implements (symbol, symbol)` from a relationship marked `is_implementation`.
- `code/depends_on (repo, package)` for each external package a symbol names.
Red test: `ingest_emits_the_four_shapes`, one fixture that has one of each.

**SI-3. A repo's own version is not part of a symbol's identity.**
PROVISIONAL, method (note 3). The indexed repo's own module or crate version
inside a symbol is replaced by `.`, as `facts.py` does. Without it, scip-go's
own-module version moves with every commit and every own fact would end and
restart at each load.
Red test: `own_version_bump_changes_no_fact`. The same fixture under two own
versions, loaded at versions 1 and 2, gives an empty `diff`.

**SI-4. Document and symbol filters.** PROVISIONAL, method and lesson.
- Every document entity is prefixed with its repo (`kos:src/orient.rs`), so the
  same path in two repos stays two entities (note 3).
- Documents whose path leaves the repo root (`../`) are skipped. In the probe
  these came from a Go build cache beside the clone: 67 to 68 documents and
  about 6,500 facts per marvel commit (h57b).
- Local symbols are skipped for occurrences and for `code/implements`.
- The load reports how many documents and facts each filter dropped, so a
  filter that starts dropping a lot is visible.
Red tests: `out_of_root_documents_are_skipped_and_counted`,
`same_path_in_two_repos_is_two_entities`, `local_symbols_are_skipped`.

**SI-5. The Go build cache stays outside the indexed tree.** PROVISIONAL,
lesson. The documented indexing recipe sets `GOCACHE` outside the clone. SI-4's
filter is the second line of defence, not the first.
Check: the recipe in the PR's docs names it. There is no unit test, since the
ingest does not run the indexer (open item O1).

**SI-6. Provenance through a load entity.** PROVISIONAL, method (note 3) and
result (C2 passed, 290 of 290 rows for W6, 18 of 18 for W7, both W8 ends).
Each load writes `load/<repo>@<n>`, asserted at that version and never
retracted. It carries six values: repo, commit, indexer, indexer version,
index sha256, and source grade `compiler`. A `code/` row's provenance is its
repo's load entity at the row's start version, and at its end version for an
ended row. The store has no provenance column, and putting provenance in `v`
would change every fact's identity at every commit.
Red test: `every_code_row_resolves_to_a_complete_load_entity`. After two
loads, every `code/` row at both versions resolves to a load entity with all
six values.

**SI-7. Intervals end; no tier.** PROVISIONAL, method (the two aae-orc-pwnh
requirements) and result (C1 passed at 50 of 50 versions, with a mutant
caught).
- A fact missing from a later index ends its interval. It is stale, never
  graveyard.
- Source grade sets no confidence tier; no `code/` fact is bedrock, frontier or
  graveyard.
Red tests:
- `missing_fact_ends_its_interval`: load 2 drops one fact, its row gets
  `valid_to`, and `diff(1,2)` equals the set difference.
- `deleted_line_mutant_is_caught`: a fixture copy with one fact line deleted
  fails the state check at that version and the diff check at the next.

**SI-8. Refuse a degraded index.** PROVISIONAL, lesson and result
(finding-kos-ndz7; h57b, "Silent degradation"). The ingest refuses an index
unless all of these hold.
- Rust: the indexer log has no `can't load standard library` line; the index
  names at least one external symbol from `std`; and it has a document for
  every tracked `.rs` file under the crate's `src/`.
- Go: the index has a document for every tracked non-test `.go` file in a
  package the module builds.

A refusal exits nonzero, loads nothing for that commit, and records the
commit as a GAP with the reason. So the ingest's inputs are the index, the
indexer's log, and the repo checkout at that commit.
Red tests (negative controls, kept in the suite):
- `rust_index_without_std_is_refused`: fixture with 0 std symbols plus the log
  line, from the probe's working method (note 2: a toolchain with no rust-src;
  an empty `RUST_SRC_PATH` did not reproduce the trap);
- `go_index_missing_a_document_is_refused`;
- `refusal_loads_nothing`: after a refusal, the store version count and rows
  are unchanged.

A positive control, `healthy_rust_index_is_accepted`, guards against a check
that refuses everything.

**SI-9. Several repos in one store.** PROVISIONAL, method (note 3). One store
file holds several repos, so a cross-repo question is one query. Each repo's
load asserts and retracts only that repo's facts, against that repo's own
previous load.
Red test: `loading_repo_a_never_retracts_repo_b`.

**SI-10. Code facts move as JSONL, not harvest YAML.** PROVISIONAL, result
(MDE round-trip FAILED as written; lock-in PASSED, 88,997 interval rows, 0
mismatches). Harvest YAML carries a set of facts at one version and drops four
of six provenance values and every interval (h57b, "Round-trip breakage";
`src/facts/exchange.rs:11-12`). So code facts leave and enter a store only
through `export_jsonl` and `import_jsonl`. A harvest that would include `code/`
or `load/` attributes is refused, and the refusal names JSONL.
Red tests:
- `harvest_refuses_code_attributes`;
- `jsonl_round_trip_keeps_code_facts_intervals_and_load_entities`: state, diff
  and count equal at every version.

**SI-11. A defined crate-name list.** PROVISIONAL, lesson (h57b: a window
union matched names that were dependencies only at another commit; per-commit
manifests dropped mentions once a dependency left, which moved W8 from 3 facts
to 0). Where a `kos/` fact records a crate name that a node's text names, the
names come from one checked-in list per repo, versioned with the repo. The
list is not derived from the window or from each commit's manifest.
Red test: `crate_names_come_from_the_list`. A name in the text but not in the
list makes no fact; a listed name makes one at every version.

**SI-12. Acceptance re-run.** PROVISIONAL, new. It shows the build matches the
probe on real indexes, not only on fixtures. Before the PR leaves draft, the
builder re-runs the probe's two windows (kos fe1702a8 and marvel 5eb450df, 50
first-parent commits each), under the probe's run conditions, and reports:
- C0 at 50 of 50 per repo;
- C1 state and diff equal at all versions;
- C2 complete provenance;
- W7 against the committed ground truth (`gt/w7-*.txt`), with recall and
  precision of 1.000.

A number that differs from h57b is reported with its cause, not tuned away.
Check: a run report beside the PR, as the probe's was.

## Open items for review

These are not settled here. Each is a question for the reviewer or, where it
changes scope, a ruling.
- **O1.** Does kos only read index files (this brief's default), or also run
  the indexers? Running them pulls toolchain discovery into kos, which is the
  surface where finding-kos-ndz7's trap lives.
- **O2.** Who keeps each repo's crate-name list (SI-11) current?
- **O3.** What a store version means in production. The probe loaded the n-th
  commit of each repo together. A real feed loads repos at different times, so
  a version may need to be per load rather than per window position.
- **O4.** For SI-10, should a code fact in a harvest be refused or only warned
  about? This brief proposes refusal, because a silent provenance loss is the
  failure the MDE test found.
- **O5.** The CLI verb and where the store file lives.

## Tickets

- **This build.** One flat ticket, labels `aae-orc,kos`, pointing at this brief.
- **The commit-to-decision link.** One flat ticket, labels `aae-orc,kos`, with
  no edge to the build.

Neither has a parent.

## Dated notes

None yet.
