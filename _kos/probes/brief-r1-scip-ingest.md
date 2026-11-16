# Brief: probe R1, SCIP facts ingested into the kos fact store (aae-orc-6fy1)

Status: DRAFT, written before any code or measurement. It is locked when its
kos PR merges after review. Nothing below changes after that without a dated
note, except the two slots marked FILLED AT RUN, which a dated note fills before
the first step.
Date: 2026-11-15.
Question node: question-code-graph-correspondence, sub-question B, the
decision-forcing half that finding-kos-ndz7 left open.
Follows: finding-kos-wre7 (live LSP did not serve W6, W7 or W8),
finding-kos-ndz7 (indexing cost, and the silent-degradation trap),
finding-kos-8bmj and finding-kos-qttt (the store probes), and Desk kdec-7
(ruled 2026-11-14: the fact store is SQLite valid-time, built in aae-orc-xxe3n).

## Question

Indexing is affordable (finding-kos-ndz7), and a store has been chosen
(kdec-7). The question that decides whether an ingest is worth building is
still open: once compiler-emitted SCIP facts sit in the kos fact store beside
kos's own knowledge, do they serve the workloads live LSP failed (W6 history,
W7 one query across repos, W8 why), and do they measurably improve freshness or
provenance (briefing 4.2)? "kos has a code graph" is not a result. A served
workload, or a measured improvement on one of those two, is.

Ranking, the third 4.2 problem, is not tested. Its host was forestage, which is
retired, and no host is named (question-code-graph-correspondence, source
calibration). This probe does not pick one.

## What this probe is not

- Not the production ingest. If the probe proves out, the build is a separate
  ticket with its own brief.
- Not an answer to sub-question D of question-kos-fact-store-substrate (code
  facts in a separate subgraph, joined by a bridge). The probe fixes one
  design, one store with two attribute namespaces (`code/` and `kos/`), so it
  can run at all. The bridge design is not run, and D stays open.
- Not a schema change. Node-level provenance is schema v0.4 (aae-orc-e689),
  which is not built. Provenance here lives on store rows only, described
  below, and nothing is written to `_kos/` nodes.

## Arms

One arm. The store is the xxe3n SQLite valid-time store: one row per fact
interval, `valid_to` null while the fact is live, versioning done by kos.

- **Store:** aae-orc-xxe3n PR 1 at its head, the sha FILLED AT RUN. If PR 1 is
  not open when this brief merges, the run waits for it. If PR 1 changes its
  store API before the run, the probe is based on the new head, and a dated
  note records both shas.
- **Ingest:** a Rust program on a probe branch based on that head. It reads
  SCIP through the `scip` crate's Protobuf bindings and writes facts through
  the store's own API, never through raw SQL. Its head sha is recorded in the
  finding. The branch is not merged from this ticket.

Facts, one per SCIP occurrence or relationship:

- `code/defines (document, symbol)` for an occurrence with the definition role;
- `code/references (document, symbol)` for any other occurrence of a symbol;
- `code/implements (symbol, symbol)` from a relationship marked
  `is_implementation`;
- `code/depends_on (repo, package)` for each external package that a symbol
  occurrence names.

Knowledge facts: kos's own nodes and findings at the same commit, as kos
already parses them, loaded as `kos/` facts: node id, type, confidence, the
edges, and each file path or crate name a node's text names.

Every row carries its provenance in the store: repo, commit sha, indexer name
and version, the sha256 of the SCIP index it came from, and a source grade of
`compiler`. The two requirements that aae-orc-pwnh carried into e689 hold in
this probe as written:

1. A fact missing from a later index ends its interval (`valid_to` is set). It
   is stale, never graveyard.
2. Source grade sets no confidence tier. No `code/` fact is bedrock, frontier
   or graveyard.

## Inputs

- **Repos:** kos and marvel. Each one's 50 first-parent commits of main, ending
  at the main head when this brief merges (FILLED AT RUN), oldest first. Each
  commit list is written to a file and its sha256 recorded before any index is
  built. kos's window includes 829c392, the commit that replaced `serde_yaml`
  with `serde-saphyr` (the 42nd first-parent commit counting back from kos
  main at 87743bb).
- **Clones:** scratch clones made from the run host's existing local checkouts
  (`git clone --no-local`), never from the network.
- **Indexers:** rust-analyzer `scip`, invoked by its absolute toolchain path,
  never through a shim (finding-kos-ndz7 premise checks); scip-go from
  `github.com/scip-code/scip-go`. Both already installed, versions recorded.
- **Ground truth:** written to a file, with its sha256 recorded, before the
  ingest runs. It comes from `git grep` and `git log` at the pinned commits, as
  each workload below states, never from the store. Pre-run premise check:
  each workload's ground truth is non-empty in every repo it names. An empty
  one stops the run before the ingest, and is reported with the command that
  produced it, because a workload with nothing to find fails for a reason that
  has nothing to do with the store.

## Silent degradation check, built into the ingest (finding-kos-ndz7)

rust-analyzer indexed a workspace without the standard library, exited 0, and
wrote an index that looked fine. An ingest that trusts the exit status loads a
partial graph and reports it as whole. So the ingest refuses an index unless
all of these hold, and a refusal exits nonzero, loads nothing for that commit,
and records the commit as a GAP with the reason:

- **Rust:** the indexer log has no `can't load standard library` line; the
  index names at least one external symbol from the `std` crate; and the index
  has a document for every tracked `.rs` file under the crate's `src/`.
- **Go:** scip-go exited 0, and the index has a document for every tracked
  non-test `.go` file in a package the module builds.

Controls, run before the first measured commit, each result in the report:

- **Negative, Rust.** kos indexed at its pinned head with `RUST_SRC_PATH` set
  to an empty scratch directory. The control first shows that rust-analyzer
  reproduces the trap (the log line is present and the exit status is 0), and
  then that the ingest refuses the index with a nonzero exit and loads no rows.
  If `RUST_SRC_PATH` does not reproduce the trap on the run host, the run
  stops and reports that. Another way to remove the standard library is
  chosen in a dated note, never during the run.
- **Positive, Rust.** The same commit indexed normally passes, and its row
  count is reported.
- **Go.** One marvel commit indexed with a package's files removed from the
  scratch clone must be refused for the missing documents.

A check that passes its negative control is the only kind that counts. Any
control with the wrong result stops the run before measurement.

## Workloads

Each workload is answered by one store query, written before the run and quoted
in the finding. "Served" and "not served" use finding-kos-wre7's definitions.

- **W7, one query across two repos.** "Where does each of kos and marvel
  resolve a per-user directory?" The query asks for references to the external
  symbols `dirs_next::data_dir` (kos) and `os.UserHomeDir` (marvel) at each
  repo's pinned head. kos never reads the home directory itself: at 87743bb
  its one per-user lookup is `dirs_next::data_dir` (src/orient.rs:1327), which
  resolves under the home directory, so it stands in for home there. marvel at
  627a51c5 has 18 non-test `os.UserHomeDir` calls. Ground truth: the non-test
  call sites from `git grep` at the same commits. Served means one query
  returns sites in both repos, with recall of at least 0.9 against the ground
  truth; precision is reported. This replaces finding-kos-wre7's logging
  question, because kos's `Cargo.toml` at 87743bb names no logging crate.
- **W6, what did a past change touch and decide?** Fixed at the lock as the
  kos commit 829c392. The query returns the `code/` facts whose interval
  starts or ends at that commit, plus any `kos/` fact naming the commit or its
  PR (#135). Served: the changed facts equal the set difference of the two
  commits' fact files, computed outside the store, and a decision record is
  returned. Partially served: the changed facts are right, but no record comes
  back.
- **W8, why is it this way?** "When did `serde_yaml` leave kos, and was that
  decided?" The query returns the `code/depends_on (kos, serde_yaml)` interval
  with its provenance, and any `kos/` fact naming either crate. Served: the
  interval ends at 829c392, the row's provenance names the indexer and the
  commit, and a record of the decision comes back. Partially served: the
  interval and provenance come back without a record.

## Decision criteria

Results are reported as PASS or FAIL, with the number.

- **C0, coverage.** At least 45 of each repo's 50 commits index, pass the
  silent-degradation check and load. If a repo falls below that, its GAP
  reasons are reported, and its workloads are scored only at commits that
  loaded.
- **C1, freshness.** Loading commit i+1 onto the store holding commit i
  closes exactly the intervals of the facts that left and opens exactly those
  that arrived. This is checked against the set difference of the two fact
  files for every consecutive pair that loaded, with any mismatch a FAIL. Load
  time from fact file to committed rows, p95 under 10 s per commit and per
  repo. Index time is reported, not scored, since finding-kos-ndz7 measured it.
- **C2, provenance.** Every row that W6, W7 or W8 returns carries its repo,
  commit, indexer and version, index sha256 and source grade: 100 percent of
  rows, or FAIL. No `code/` fact carries a confidence tier: 0 rows, or FAIL.
- **C3, the workloads.** W6, W7 and W8, each scored served, partially served
  or not served, beside finding-kos-wre7's result for the same workload.

## What the probe reports

It recommends a production ingest ticket only if all of these hold:

- C0, C1 and C2 pass;
- W7 is served;
- at least one of W6 and W8 is at least partially served.

Otherwise it reports which criterion failed and what that says about R1. The
probe recommends and does not choose: the operator rules.

## The three MDE killers, answered before the probe runs (kos roadmap v3, M4)

- **Tool lock-in.** Test: all facts, intervals and provenance export to JSONL
  and re-import into a fresh store whose per-commit fact counts and C1 pair
  differences equal the source's. Exit cost: the export and import times,
  measured.
- **Round-trip breakage.** Test: 100 `code/` facts drawn as of each repo's head
  (Python `random.Random(20261115)`, `.sample` over the facts sorted by
  entity, attribute and value) go to YAML and back with the same facts,
  intervals and provenance. Any loss is a FAIL.
- **Skill asymmetry.** Every query is plain SQL through the store's API. Any
  query rewritten for speed after its first run is recorded with both forms.

## Run conditions

- A scratch HOME and TMPDIR made with `env -i`; no MARVEL_SOCKET.
- Go: `GOPROXY=off GOFLAGS=-mod=readonly GOTOOLCHAIN=local`. A commit whose
  modules are not in the local cache does not index, and is a GAP, never a
  fetch.
- Rust:
  - `CARGO_NET_OFFLINE=true`, with `CARGO_TARGET_DIR` inside the probe
    directory;
  - reads of the run host's Rust toolchains and crate caches only, never its
    configuration or credentials;
  - the ingest built with `cargo build --offline`, its binary's sha256
    recorded.
- `sandbox-exec` with the Probe C profile from
  `brief-fact-store-probe-c-history-growth.md`, unchanged except that the
  Dolt line does not apply. It is quoted literally in the report, with every
  live localhost port on the run host denied. It runs with Probe C's three
  controls (a non-local connection refused, the denied and undenied scratch
  ports, and the write outside the probe directory refused). No control dials
  a live port or a public host.
- Disk: 5 GB free before the run, checked and reported.

## Timebox

Three working days, starting when the PR 1 head is recorded. Indexing is
expected to take about an hour: 100 commits at the warm costs in
finding-kos-ndz7, with kos cold builds slower.

## Prediction (registered)

The arcaven architect seat's, registered before any code is written:

- 0.9 that the negative Rust control reproduces the trap and the ingest refuses
  it;
- 0.8 that W7 is served;
- 0.6 that W6 is partially served: the changed facts are right, and no
  decision record names 829c392 or #135 in `_kos/`;
- 0.5 that W8 is partially served, for the same reason;
- 0.7 that C0 to C2 all pass.

If W6 and W8 come back partially served for that reason, the finding should
say so plainly. The missing piece is then the link from a commit to a
decision, which is knowledge kos does not yet record, and is not a gap in the
code graph.

## Dated notes

Each note is written after the lock and before the step it governs. The text
above is unchanged.

**Note 1, 2026-11-16: the FILLED AT RUN slots.** Runner: a kos builder seat
(arcaven team), on mokuzai, by the director's host ruling.

- Store: the probe branch is based on kos main at 071f1252. That commit
  contains xxe3n PR 1 (kos#188, merged as 5d87f18f), and
  `git diff --stat 5d87f18f 071f1252 -- src/facts tests/facts_store.rs Cargo.toml`
  is empty, so the store is PR 1's.
- kos pin: fe1702a8, the main head when this brief merged (kos#185,
  2026-11-16T00:32:29Z). 829c392 is the 47th first-parent commit counting back
  from it, inside the 50.
- marvel pin: 5eb450df, the main head at the same moment (committed 00:26:49Z;
  the next first-parent commit is at 00:38:06Z). W7 scores against it, and
  its non-test `os.UserHomeDir` count is 18, the same as at 627a51c5.
- Deviation: "both already installed" did not hold on this host. rust-analyzer
  and rust-src were added to the `stable` toolchain that kos's
  `rust-toolchain.toml` resolves to (rustc 1.96.0), and scip-go v0.2.7 was
  installed from `github.com/scip-code/scip-go`. That is a pinned fetch under
  the director's ruling, run outside the sandbox, and each version string is
  recorded in the report.

**Note 2, 2026-11-16: the negative Rust control is replaced.** The run
stopped as the brief requires, because `RUST_SRC_PATH` set to an empty
scratch directory did not reproduce the trap:

- With rust-src installed on the active toolchain, rust-analyzer 1.96.0 loaded
  std from the toolchain's own `lib/rustlib/src`.
- The index was byte for byte the positive control's (sha256 ff546670...e43d,
  1,552,519 bytes).
- There was no "can't load standard library" line, and std symbols were
  present.

The replacement reproduces the mechanism that finding-kos-ndz7 observed: the
toolchain in use has no rust-src.

- **Method A.** Run the same rust-analyzer binary (stable's, by absolute path)
  on the same commit with `RUSTUP_TOOLCHAIN` set to an installed numbered
  toolchain that has no rust-src. Before the run, record
  `rustup component list --installed --toolchain <that toolchain>` showing no
  rust-src, and record the toolchain's rustc version.
  - Nothing is installed, removed or written outside the probe directory, and
    the real toolchains are only read.
  - The control passes as the brief states: the log line is present, the exit
    status is 0, and the ingest then refuses the index with a nonzero exit and
    loads no rows.
- **Method B, only if A does not reproduce the trap.** Use a copy of the
  stable toolchain inside the probe directory with its `lib/rustlib/src`
  removed, selected through `RUSTUP_TOOLCHAIN` as a path, under the same pass
  condition.
- If neither reproduces it, the run stops again and reports. The brief's
  check then has no working negative control on this host, and per the brief
  it does not count.

**Note 2, continued: the offline std metadata warning.** Both control logs
carry a warning that `cargo metadata` for the std workspace failed offline
("no matching package named `dlmalloc` found ... required by package `std`")
and fell back to `--no-deps`.

- This is not a refusal condition. It concerns std's own dependencies, not
  whether kos's references into std resolve. Those do: the positive control
  names std symbols and has no "can't load standard library" line, which is
  what the silent-degradation check tests.
- The ingest records, per index, whether this warning appeared, and the report
  gives the count. If the run shows that an index with the warning lacks std
  symbols that an index without it has, the finding says so and names the
  warning as a candidate refusal condition for the production ingest.
