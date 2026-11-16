# Probe R1 run report: measured facts (aae-orc-6fy1)

Status: DRAFT measured-facts file written by the runner. It states what was measured and where the evidence is. It carries no readings of W6 or W8 and no recommendation: those are the architect's. Paths below are relative to the probe directory, written as <probe>.

## Pins and inputs

- Store base: kos main 071f1252e5f9ccf23610427bbd492e39b6885d22 (xxe3n PR 1 merged as 5d87f18f04d7ae1eed6440d76b9af31df1614f17, byte-identical store source per the architect).
- kos window: 50 first-parent commits ending at fe1702a87346f9833fe727af66da7774d9e092b5 (brief merge, 2026-11-16T00:32:29Z); first d2ddbf598685a5add12a41028996d0e59038debc; 829c392 is the 4th, oldest first. List sha256 5a6016523fa520d2071f56f343cf7a2bf012fff54d21e4ac32524500cd35dd2f (<probe>/kos-commits.txt).
- marvel window: 50 first-parent commits ending at 5eb450df8361df36a9971998a640283e29e2291e (the head at brief merge; main has since moved to 6992204557c441da6e7da5d4ff647a3fc7be4bd1). List sha256 193420c15bd695c74b0204de05bf0900163331d2b280a5ae332b33bf14f2d98d. The brief's W7 text quotes 627a51c5; it is reported beside, with the same 18 non-test os.UserHomeDir sites.
- Clones: scratch clones made from the host's local checkouts with git clone --no-local, plus a fetch of the local remote-tracking objects; no network for them.
- Ground truth (written before the ingest ran): W7 marvel <probe>/gt/w7-marvel-5eb450df.txt sha256 25e9171a40bebba2415ac12b6ad050fdfba4016b5331d8080af65a6d69d0155c, 18 sites in 14 non-test files; W7 kos <probe>/gt/w7-kos-fe1702a8.txt sha256 4ee669a574a942e6e90fa316ae3ad965ae7453a75bb5907d7fed4e94ba5ae51a, 1 site in 1 file (src/orient.rs:1325); expected-documents lists per commit in <probe>/expect/.
- W8 premise check: serde_yaml is declared in Cargo.toml at 829c392^ and not at 829c392 (git grep).

## Tools and deviations from the brief's text

- The brief says both indexers are already installed. On this host they were not. Under the fetch ruling: rustup component add rust-analyzer rust-src --toolchain stable (nothing else from rustup); go install github.com/scip-code/scip-go/cmd/scip-go@v0.2.7 into a GOPATH inside the probe directory.
- rustup toolchain: stable-aarch64-apple-darwin (kos's rust-toolchain.toml reads channel = "stable"); rustc 1.96.0 (ac68faa20 2026-05-25); rust-analyzer 1.96.0 (ac68faa2 2026-05-25), run by its absolute toolchain path, never a shim. Installed components on stable after the add: cargo, clippy, rust-analyzer, rust-docs, rust-src, rust-std, rustc, rustfmt.
- scip-go 0.2.7, module github.com/scip-code/scip-go v0.2.7 h1:gcHFnhoMCdp3C/0xvCn9QFuTsF/W3JSR5cLVf7uG+Us=; dependency github.com/scip-code/scip/bindings/go/scip v0.7.1 h1:x2QVP2nO09TFUGVrNJ+8XakD45KyXn/tSalzAELcB3Q=; built with go1.27.0; binary sha256 4f0de56b2153be89f34825869b879beac86e2e2cdb9f9db639e1700d0b1775fe.
- Crate fetch (director's ruling): the scip crate and its closure into the probe's own CARGO_HOME: scip 0.10.0 checksum d7f651fbd0f98742a47b58623a7bcfd8fad14455912689e4867a1f43ee7e02c6; protobuf 3.7.2 checksum d65a1d4ddae7d8b5de68153b48f6aa3bba8cb002b243dbdbc55a5afbc98f99f4; protobuf-support 3.7.2 checksum 3e36c2f31e0a47f9280fb347ef5e461ffcd2c52dd520d8e216b52f93b0b0d7d6. Cargo.lock = base lock (218 packages) plus those three (221), sha256 0a210921e06d959fb104161a92eb5fac2faa0d31e7e4538e8a723f60ffbbca15; protobuf added as a direct dependency (no new crate).
- cargo deny 0.19.9, check advisories, offline, --disable-fetch, repo deny.toml: advisories ok, rc 0 (<probe>/deny-advisories.log). The advisory database was a copy of the host's existing one, head 02e2517f457a359fc6d3440ea4560c8ea188fee8, committed 2026-11-15T10:59:21+01:00; no database was fetched.
- The ingest is a probe branch (probe/r1-ingest, one new file src/bin/r1_ingest.rs, local, not merged), built with cargo build --locked --offline.
- Disk: about 235 GB free on the data volume at the start (brief needs 5).

## Run conditions

- Every index, check, load and query ran under sandbox-exec with the brief's Probe C profile literally (<probe>/run.sb, sha256 a50acb5e4ffd490677efb3eda1264e52ede8f932a5296601b3ca1a25b45ffc63, 289 lines: the brief's rules, then a deny for each live localhost port on the run host at the time, then 47999 last), env -i, HOME and TMPDIR inside the probe directory, GOPROXY=off GOFLAGS=-mod=readonly GOTOOLCHAIN=local for Go, CARGO_NET_OFFLINE=true for Rust, CARGO_HOME the probe's copy of the crate registry.
- Controls (repeated before each negative control): 192.0.2.1:80 refused with "Operation not permitted"; scratch listener 47999 started inside the sandbox refused inside with "Operation not permitted" and reachable from outside; scratch listener 47998 reachable inside; a write to a fixed path outside the probe directory refused and the file absent; a write inside the probe directory succeeded.
- The fetches ran outside the sandbox and nothing else did.
- Go list for the expected-documents lists ran in the sandbox; one write to the real Go module cache stat file was refused by the sandbox ("operation not permitted") and did not stop the list.

## Silent-degradation controls (finding-kos-ndz7)

- Rust positive: kos fe1702a8, exit 0, index 1552519 bytes, sha256 ff546670263ce8663a9326f25e9048888ecb5e3603a07733bf4f5a3806e2e43d, no "can't load standard library" line, 26 std external symbols (strings count); the ingest's own check: ok documents=40.
- Rust negative, first method: RUST_SRC_PATH set to an empty directory. It did NOT reproduce the trap: exit 0, no log line, the index byte-identical to the positive control. Run stopped, reported, resumed under the architect's dated note 2.
- Rust negative, Method A (numbered toolchain 1.96.0 with no rust-src, shown by rustup component list --installed, rustc 1.96.0; stable's rust-analyzer binary): log line present ("ERROR can't load standard library, try installing `rust-src` sysroot_path=<rustup>/toolchains/1.96.0-aarch64-apple-darwin"), exit 0, index 1150561 bytes, sha256 2b3b97bd0e7d1aa5a23b7c2e373d83f5edbd2971e3968f6caa8aec9a139d1dc0, 0 std symbols. The ingest check refuses it: "REFUSED: the indexer log has \"can't load standard library\"" and "REFUSED: the index names no external symbol from the std crate", rc 3, nothing loaded.
- Go negative: marvel 5eb450df with internal/backoff/backoff.go removed in the scratch clone (restored afterwards): scip-go exit 0, index 43968233 bytes (positive 43978994); the ingest check: "REFUSED: 1 tracked source files have no document, first: internal/backoff/backoff.go", rc 3.
- Go positive: ok documents=1286 (372 expected files from go list).
- The rust-analyzer WARN seen in every kos index log (50 of 50) and in none of the 50 Go logs: "error: no matching package named `dlmalloc` found / location searched: crates.io index / required by package `std v0.0.0 (<rustup>/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/std)` / note: offline mode (via `--offline`) can sometimes cause surprising resolution failures", introduced by "WARN `cargo metadata` failed and returning succeeded result with `--no-deps`". Present, not shown to change the symbols the probe uses, not tested (dlmalloc was not fetched).

## Indexing and checks (C0)

- kos: 50 of 50 commits exit 0, no trap line, check ok at all 50; index time 151 s in total, 4 to 6 s per commit warm. Log <probe>/idx/kos.tsv (per commit: exit, seconds, bytes, index sha256, trap lines, dlmalloc flag).
- marvel: 50 of 50 commits exit 0, check ok at all 50; index time 213 s in total, max 10 s per commit; the first, cold index of the pin took 23 s. Log <probe>/idx/marvel.tsv.
- C0: kos 50/50, marvel 50/50. GAPs: none.
- Per-commit check and fact counts: <probe>/prep-kos.tsv and <probe>/prep-marvel.tsv (columns include dropped documents, dropped facts, dropped implements, code facts, kos facts, read by kos's parser, refused by kos's parser, read by fallback, lost).

## Facts and the ../ filter

- Symbols: the indexed repo's own module or crate version is replaced by "." (note 3); document entities are prefixed with their repo ("kos:src/orient.rs"); local symbols are skipped for occurrences and for code/implements (the latter differs from the published extractor; code/implements dropped by it: 0 at every commit in both repos).
- Documents whose path starts with "../" are skipped, as the published extractor does. Cause: the Go build cache sat beside the clone, so scip-go indexed the test mains it generated. kos: 0 dropped. marvel: 67 to 68 documents per commit, 6,516 (version 1) to 6,801 (version 50) facts per commit, 3,353 documents and 332,733 facts over the 50 commits (dropped_facts = distinct (role, symbol) pairs the dropped documents would have produced).
- kos/ knowledge facts are read by kos's own parsers (kos::model::Node through kos::yaml, kos::findings::load_finding_file): refused by the parser at every one of the 100 commits: 0; read by the fallback: 0; lost: 0. kos 144 to 152 nodes and findings read per commit, marvel 154 to 157 (corrected: an earlier draft of this line said 144 to 156 and 154 to 156, which was wrong; the per-commit tables prep-kos.tsv and prep-marvel.tsv, column read_by_kos_parser, give these ranges).
- The names-dep vocabulary (crate or module names a node's text names): v2 = the dependencies declared at the commit being read; v3 = the union of dependency names across the 50-commit window (kos 18, marvel 29; <probe>/gt/deps-union-*.txt). v3's bias: a name that is a dependency only at another commit of the window still matches.

## Stores and binaries

- v1 (defect run, ../ documents included): <probe>/store-v1-gocache-included.db sha256 a7099bfd6d65531c82b7ee64b191616c262fd7aece0e51160bf22595c3cef6cc, loader binary sha256 10bc19315c032ed0296d26b9a77b7e52a9fd42a4e2316a5a9597b36ea659dbed, log <probe>/loads-v1-gocache-included.log. Not scored.
- v2 (../ filtered; names-dep vocabulary v2): <probe>/store-v2-gocache-filtered-deps-at-commit.db sha256 f5c11d4a8168f0aa486c8e5353e38c07ed35d0ebf6ca5c95c3130480c46540fc, loader binary sha256 68392eb88a34f77e05b324abc6b1146b1ad16c959e06b85d14a070cd2eead8d2 (<probe>/r1_ingest-loader-68392eb8), log <probe>/loads-v2.log. Not scored; the finding reports it beside v3.
- v3 (scored: ../ filtered, vocabulary v3): <probe>/store.db sha256 c0207ac041b86b487bba4df4e65617bffc5bcdb6f97f804aad991bb88395ebb0, binary sha256 af030637a5c9beacede2250670f65ca53b51d55a06970963e268ca2eb29dae6a; copy for the MDE tests <probe>/store-v3-mde-copy.db sha256 c0207ac041b86b487bba4df4e65617bffc5bcdb6f97f804aad991bb88395ebb0; log <probe>/loads.log.
- Shape: one store file for both repos; version n loads kos's n-th and marvel's n-th commit; provenance as load entities "load/<repo>@<n>" with six prov/ attributes, asserted at that version and kept.

## C1, load time (v3)

- 100 loads, 0 errors. Time from fact file to committed rows: max 1.56 s, p95 0.04 s; the bar is under 10 s per commit and repo: PASS.
- Equality: for versions 1 to 50 the store's code/ and kos/ facts equal the union of the two repos' fact files and the diff between consecutive versions equals the files' set difference: 50 of 50 equal, 0 mismatches (<probe>/c1.log). Checker control: one fact line (line 5) deleted from a copy of marvel's version-30 fact file, nothing else changed, gives mismatches at versions 30 (state and diff) and 31 (diff only) and exit 1 (c1-mutant.log).

## C2, provenance (v3)

- W7's 18 returned rows each resolve by their first live version to a load entity of their repo with all six values: true (<probe>/w7.log). W8's two ends resolve likewise. W6, every row: each of the 290 changed code/ facts at version 4 resolves to a complete load entity (all six values) of its repo: the 77 added facts by their start version (4), the 213 removed facts by their first live version and by the version that ended them (4): 77 of 77, 213 of 213 and 213 of 213, all_resolved=true. A query added after the scored runs (read-only, same store, store sha256 unchanged after); the binary that ran it has sha256 7ea390833cd213724fd91fa2f4cb726dfccb6ca4904ff96698105f75a8ea4f71 (source difference from the scored binary af030637...: the added check only).
- "No code/ fact carries a confidence tier": the store has no tier field; it holds by absence.

## Workloads (v3, measured; readings are the architect's)

- W7, denominators fixed before the run: marvel 14 non-test files (18 sites) at 5eb450df, kos 1 file (1 site) at fe1702a8; bar recall 0.9 and sites returned in both repos by one read. Measured at document level (note 4): kos found 1 of 1, recall 1.000, precision 1.000; marvel found 14 of 14, recall 1.000, precision 1.000; missed none, extra none; one read returned both repos; 18 rows returned, the marvel test-file documents filtered by name. The kos symbol is "rust-analyzer cargo dirs-next 2.0.0 data_dir()."; the marvel symbol carries the Go toolchain version: "scip-go gomod github.com/golang/go/src go1.26.0 os/UserHomeDir()." (constant over the run: one toolchain indexed every commit).
- W6 (kos 829c392, version 4): code/ facts changed for kos at version 4: 77 added, 213 removed, equal to the set difference of kos's fact files at commits 3 and 4 (as sets, both directions). kos/ facts naming 829c392: 0 at version 4, 1 at version 50 (kos:node:question-kos-fact-store-substrate); naming #135: 0 at versions 4 and 50.
- W8 (kos, serde_yaml): code/depends_on (kos, serde_yaml) live from version 1, not live from version 4 (the version of 829c392). Provenance at version 1: commit d2ddbf598685a5add12a41028996d0e59038debc, rust-analyzer, 1.96.0 (ac68faa2 2026-05-25), index sha256 4598ac004822d880a6cf7cfa7180f6a9ef86304f6951699f0b5d59c3c8a38363, grade compiler; at version 4: commit 829c39224b894e796a9898aa3a90b54b2bd41b97, same indexer, index sha256 a8927bf6a3ccae96b9ca4a234684ad378aaaf9786d58e4a3808fdebe11f5ce5f, grade compiler. kos/ facts naming serde-yaml at version 50 (v3): 3 (finding-029-kos-orient-probe, finding-030-schema-tooling-probe, finding-kos-wre7-serena-live-lsp-baseline); naming serde-saphyr: 0. Under v2 the first count was 0.

## Not run

- The MDE tests now ran; see "MDE tests" below.
- A counterfactual for the rust-analyzer dlmalloc WARN.
- The brief's timebox and the registered predictions are not scored here.

Report path: <probe>/report/r1-run-report.md.

## Profile network lines, quoted

```
(version 1)
(allow default)
(deny network*)
(allow network* (remote ip "localhost:*"))
(allow network-bind (local ip "localhost:*"))
(allow network-inbound (local ip "localhost:*"))
(deny file-write*)
(allow file-write* (subpath (param "PROBE_DIR")) (subpath "/dev"))
(deny network-outbound (remote ip "localhost:3307"))
(deny network-outbound (remote ip "localhost:4222"))
(deny network-outbound (remote ip "localhost:4242"))
(deny network-outbound (remote ip "localhost:7422"))
(deny network-outbound (remote ip "localhost:7442"))
(deny network-outbound (remote ip "localhost:<port>"))   one line per live localhost listener port on the host at the time, 276 lines
(deny network-outbound (remote ip "localhost:47999"))
```

The 289 lines are in <probe>/run.sb (sha256 a50acb5e4ffd490677efb3eda1264e52ede8f932a5296601b3ca1a25b45ffc63). The live-port denies were read from the host at the run; the control port 47999 is a scratch port, not a live service.

## Queries, quoted (plain reads through the store's API, no SQL)

- W7: one read, `FactStore::facts_as_of(version 50)`, filtered in the client to attribute code/references with value in {the kos symbol, the marvel symbol}; documents grouped by repo prefix, marvel documents ending _test.go removed from the scored set; compared to the ground-truth file lists. No query was rewritten after its first run.
- W6: `FactStore::diff(version 3, version 4)` filtered to code/ facts of the repo (kos entities: "kos", "kos:...", "rust-analyzer ..."), compared to the set difference of the kos fact files of commits 3 and 4; and `FactStore::entities_with("kos/names-ref", "829c392" | "#135", version)` at versions 4 and 50.
- W8: `FactStore::values_of("kos", "code/depends_on", version)` at every version 1 to 50 for serde_yaml; `FactStore::values_of("load/kos@<n>", "prov/<k>", n)` for the six prov/ attributes at the interval's ends; `FactStore::entities_with("kos/names-dep", "serde-yaml" | "serde-saphyr", version 50)`.
- C1: `facts_as_of(n)` and `diff(n-1, n)` for n = 1 to 50 against the files.

## v2 and v3 counts

- v2 versus v3 differ only in the kos/ names-dep facts. At version 50: kos's whole fact file     4888 lines in v2 and     4891 in v3; marvel's    82923 and    82923; kos/ names-dep facts in kos's file 6 (v2) and 9 (v3); in marvel's file 1 (v2) and 1 (v3).
- W8 serde-yaml naming count at version 50: 0 in v2 (the vocabulary of the commit read), 3 in v3.
- W6 and W7 numbers are the same in v2 and v3 (the code facts are identical); C1 was run on v3 only.

## Dropped counts per repo (v3 fact files)

- kos: dropped ../ documents 0 and dropped facts 0 at every commit; dropped code/implements 0.
- marvel: dropped ../ documents 67 to 68 per commit (3,353 in all); dropped facts 6516 to 6801 per commit (332,733 in all); dropped code/implements 0 at every commit.
- dlmalloc WARN: present in 50 of 50 kos index logs, 0 of 50 marvel logs.

## Files beside this report

- idx-kos.tsv, idx-marvel.tsv: per commit index exit, seconds, bytes, index sha256, trap lines, dlmalloc flag.
- prep-kos.tsv, prep-marvel.tsv: per commit check, dlmalloc, dropped documents, dropped facts, dropped implements, code facts, kos facts, read by kos's parser, refused, fallback, lost (v3 fact files).
- c1.log: C1 equality per version 1 to 50 (v3).
- deps-union-kos.txt, deps-union-marvel.txt: the v3 window-wide dependency vocabulary.
- loads.log stays in the probe directory (per-load timings, 100 lines).

## MDE tests (run after note 5 was approved)

Setup. kos at main 73a55684a1552107bbaa282bddc67f6d866b270b (kos#192), from a separate clone made with git clone --no-local from the host's local checkout and checked out detached; the probe's own code is one untracked file, src/bin/r1_mde.rs (a copy is beside this report; sha256 175e13f94a3a332f23a1582b4c3254bb99c1af55adbf049b4625ee8c7d105a5f), built with cargo build --locked --offline, outside the sandbox, with the probe's CARGO_HOME. Probe base for the scored store: kos 071f1252e5f9ccf23610427bbd492e39b6885d22. The store file is a copy of the scored v3 store: <probe>/store-v3-mde-copy.db sha256 c0207ac041b86b487bba4df4e65617bffc5bcdb6f97f804aad991bb88395ebb0, checked equal to the original before the run and after it (the original was never opened by these runs). MDE binary sha256 b527a3ce839e179f6d4fc1445d11032f9dd3004033a41cbbf75b7f563da3bf0a (an earlier build, which counted a fact's first-ever interval instead of the one live at the head, was replaced after the first round-trip run; its output was not used). The crate-name vocabulary is the scored v3 list, unchanged. The copy opened as it is: SCHEMA_VERSION is still 1.
Run conditions: every test ran under the sandbox profile run.sb (sha256 a50acb5e4ffd490677efb3eda1264e52ede8f932a5296601b3ca1a25b45ffc63, network lines quoted above), env -i with HOME and TMPDIR in the probe directory, no MARVEL_ variables; five controls run first, in this order, results as expected: 192.0.2.1:80 refused inside with "Operation not permitted"; a fresh scratch listener on 47999, started for this run, refused inside with "Operation not permitted" and reachable from outside; a scratch listener on 47998 reachable inside; a write to a fixed path outside the probe directory refused ("Operation not permitted") and absent; a write inside the probe directory succeeded. No unix-socket allows were needed. Offline: the tests open local files only.

### Tool lock-in
Criterion (brief): all facts, intervals and provenance export to JSONL and re-import into a fresh store whose per-commit fact counts and C1 pair differences equal the source's; export and import times measured.
Measured: export of the whole history: a header line plus 88,997 rows (88,998 lines, 17,467,708 bytes, sha256 a9b5cd7b3f108f8b1ac5df53dad77f1daa243893768f6e1c36430b6fc7304e5f); import into a fresh store: 88,997 rows, latest version 50 in both. For versions 1 to 50, facts_as_of equals the source's as a set at every version (state mismatches 0), the diff between consecutive versions equals the source's, added and removed (diff mismatches 0), and the per-version counts equal (count mismatches 0); this compares all attributes, which includes the load/<repo>@<n> provenance facts (the six prov/ attributes), not only code/ and kos/. Intervals: the rows carry valid_from and valid_to; the state and diff equalities at every version hold only if the intervals are equal. Export 6.54 s, import 2.53 s on this host (one run each). Log: mde-lockin.log. Result against the criterion: PASS.

### Round-trip breakage
Criterion (brief): 100 code/ facts drawn as of each repo's head (Python random.Random(20261115).sample over the facts sorted by entity, attribute and value) go to YAML and back with the same facts, intervals and provenance. Any loss is a FAIL.
Drawn: version 50, code/ facts of the repo by entity prefix; kos 3,875 facts, marvel 81,412; sample files picks-kos.tsv and picks-marvel.tsv beside this report. Each pick went through FactStore::harvest at the start version of the interval it holds at the head (13 harvest files for kos, 4 for marvel, grouped by start version; the source map carried repo, commit and version), then Harvest::parse, then apply_harvest into a fresh in-memory store at the same start version.
Measured: facts equal after the YAML round trip: 100 of 100 for kos, 100 of 100 for marvel. Start version of the interval after apply equal to the source's: 100 of 100 and 100 of 100; end (open) equal: 100 and 100. What the YAML file carries of the six provenance values: repo and commit (and the version, given as source, which the format treats as provenance only); not in the file: indexer, indexer-version, index-sha256, grade (four of six). The harvest format carries no interval: the start version above was supplied to apply_harvest by the caller from the grouping, not read from the file. Earlier closed intervals: 6 of the 100 kos picks (none of marvel's) were live earlier, absent between, and live again; the harvest carries the current interval only, so those 6 earlier intervals do not come back.
Result against the criterion, read literally ("Any loss is a FAIL"): FAIL for the YAML path as it is, because of that loss: four of the six provenance values, the interval itself (carried by the caller, not the file) and six earlier closed intervals. The JSONL path carries all of them (lock-in above). Whether the harvest format is meant to be the exchange path for provenance and history is the architect's reading; the facts themselves round trip exactly. Logs: mde-roundtrip-kos.log, mde-roundtrip-marvel.log.

### Skill asymmetry
Criterion (brief): every query is plain SQL through the store's API; any query rewritten for speed after its first run is recorded with both forms.
Measured: the probe's queries are calls to the store's Rust API (facts_as_of, diff, values_of, entities_with, latest_version, and the exchange functions above), not SQL text; no SQL was written by the probe. None was rewritten for speed. Queries changed after their first run, for other reasons, recorded here: (a) W6, before and after: the first form compared the store's whole version-4 diff, which holds both repos' changes, with kos's file difference; the second form keeps only kos's code/ facts by entity prefix (kos, kos:..., rust-analyzer ...). (b) W6's naming query gained a version argument so the check could be read at version 4 and at version 50; the query is the same call. (c) W8's names-dep vocabulary: v2 (dependencies at the commit read) replaced by v3 (window-wide union), by the architect's rulings. (d) The C2 every-row check over W6 was added after the scored runs. No query was changed after seeing its result in order to change the result's bar.
Result: PASS, with the recorded changes.

### Not run
- A counterfactual for the rust-analyzer dlmalloc WARN.

## Files added beside this report for the MDE tests
- r1_mde.rs, mde-lockin.log, mde-roundtrip-kos.log, mde-roundtrip-marvel.log, picks-kos.tsv, picks-marvel.tsv.

## More files beside this report (second staging)
- w7.log: the W7 run's output (recall, precision, rows, provenance line).
- c1-mutant.log: the C1 checker control, the run with one fact line deleted at version 30.
- run.sb: the sandbox profile the report quotes (276 deny lines for the live localhost ports of the run host, then the rules; sha256 a50acb5e4ffd490677efb3eda1264e52ede8f932a5296601b3ca1a25b45ffc63).
- deny-advisories.log: the cargo deny advisories result ("advisories ok").
- gt/w7-kos-fe1702a8.txt, gt/w7-marvel-5eb450df.txt: the W7 ground-truth file lists; gt/expect-docs-kos-fe1702a8.txt, gt/expect-docs-marvel-5eb450df.txt: the expected-documents lists for the indexer checks; gt/golist.err: the one line the Go listing wrote to stderr, with the module-cache path replaced by <gomodcache>.
- Left out: the per-commit expected-documents directory (expect/, about 800 KB); the report is the only record of its totals.
