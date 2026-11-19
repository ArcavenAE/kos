# SI-12 acceptance re-run: the SCIP ingest on the probe's two windows

Status: measured facts, written by the builder. It carries no reading of W6 or W8 beyond the numbers and no recommendation.

The build matches the probe on C0, C1, C2 and W7, and differs from the probe's numbers in four places, each with its cause below. The run found one defect in the build, a Rust check that refused a healthy kos index; the same defect was found in review, and the fix is in the chain below.

## Pins and inputs

- Build under test: the top of the chain, kos `8a02a3dcc7e8ee9c5a001f70ac99c7c28c3ec94a` (the review follow-ups, on the `--golist` PR). Release binary sha256 `b0cfda9da4c81c72ef35895accaaee84ab5492d60c94129efd4c59fd97a0d24d`, built with `cargo build --release --locked --offline`.
- The run was done twice. The first run used an earlier build (the `--golist` PR plus a stand-alone Rust root-crate fix, binary sha256 `8be9adcd...`) and indexed all 100 commits. The second run reuses those 100 index files and logs (the indexers do not depend on the ingest; their sha256 values are in the idx tsv files) and loads them into a fresh store with the top build. Both stores have the same sha256, `61417b7444324591e68e3ca2fcfcd3b5e81f1c163a7781b7ffbac0b2f743393b`, so the chain's later changes (the member-aware Rust rule, the transactional load, the unreadable-node refusal, the follow-ups) change nothing on these inputs. Every result below was computed on the second store.
- kos window: 50 first-parent commits ending at `fe1702a87346f9833fe727af66da7774d9e092b5`; list sha256 `5a6016523fa520d2071f56f343cf7a2bf012fff54d21e4ac32524500cd35dd2f`, equal to the probe's. 829c392 is the 4th.
- marvel window: 50 first-parent commits ending at `5eb450df8361df36a9971998a640283e29e2291e`; list sha256 `193420c15bd695c74b0204de05bf0900163331d2b280a5ae332b33bf14f2d98d`, equal to the probe's.
- Clones: `git clone --no-local` of the orc checkouts into a scratch directory, one detached checkout per commit; the orc checkouts were not touched.
- Crate-name lists: the probe's `deps-union-kos.txt` (18 names) and `deps-union-marvel.txt` (29), placed as `.kos/crate-names.txt` in each clone, untracked.
- One store for both repos; version n loads kos's n-th and marvel's n-th commit. Store 94 MB. For marvel the recipe's `go list -json ./...` output, with the `exit: 0` line the recipe appends, is passed as `--golist` at every commit.

## Indexers and how they differ from the probe

| | probe | this run |
|---|---|---|
| rust-analyzer | 1.96.0 on `stable` | 1.97.1 (8bab26f4), run by its absolute path in the 1.97.1 toolchain; the `stable` toolchain is now rustc 1.98.1 and has rust-src, but no rust-analyzer component |
| scip-go | v0.2.7 | 0.2.7 |

Nothing was installed. The version difference is a cause for the numbers in "Where the numbers differ".

## Run conditions

- `env -i` with a scratch HOME and a short scratch TMPDIR; no MARVEL_*, DIRECTOR_* or NATS_* variables; `CARGO_HOME` and `CARGO_TARGET_DIR` in scratch with `CARGO_NET_OFFLINE=true`; `GOCACHE` and `GOMODCACHE` in scratch with `GOPROXY=off`, `GOFLAGS=-mod=mod`, `GOTOOLCHAIN=local`.
- Every index, `go list`, ingest and query ran under `sandbox-exec` with the probe's profile form: `(deny network*)`, the localhost allow, bind and inbound allows, `(deny file-write*)` with writes allowed under the run directory and `/dev`, then one `(deny network-outbound (remote ip "localhost:N"))` line per live listener (36 read from the host, plus 3307, 4222, 4242, 7422, 7442 and the control port 47999). The profile that ran is 50 lines, sha256 `da907e893761ec99340bf4f8d7d24bbfcd425400d07ced9c9444f1aa7a5769ab`. It is not committed, because it lists the host's live ports.
- Controls, run before the first measured commit, under the run's own profile:
  - c1: a connect to 192.0.2.1:80 inside is refused with `PermissionError: [Errno 1] Operation not permitted`; the same connect outside times out (it gets past the sandbox and fails for a network reason).
  - c2: the run's own listener on denied port 47999, started outside, is refused inside with `Operation not permitted` and connects from outside.
  - c3: an undenied scratch port connects inside.
  - c4: a write to a fixed path under `/private/tmp` that is outside the run directory, is refused inside with `Operation not permitted`, and the file does not exist afterwards.
  - c5: a write inside the run directory succeeds.
- Outside the sandbox, only: the Go module fetch (`go mod download` at each of marvel's 50 commits, 33 distinct module versions, into a scratch GOMODCACHE, list in `gomods.txt`), the clones, the checkouts and the release build. Rust crates were not fetched: the host's cargo registry (2.1 GB, crate caches only) was copied into the scratch `CARGO_HOME`, and every kos commit indexed offline from it.
- Disk: 348 GB free before the run (the brief needs 5).
- cargo deny at the build: `cargo deny check advisories licenses bans` ok.

## Silent-degradation controls

These ran with the first build. The top build was not given the two negative real-index controls again; its unit tests cover the same refusal paths (a degraded log, a missing std symbol, a missing document), and the top build loaded the 100 good indexes without a GAP.

- Rust positive: kos at the pin, exit 0, no `can't load standard library` line, index 2,440,295 bytes. The ingest accepted it (5,596 facts at the first load of the control store, run with the first build).
- Rust negative: the same commit indexed under `RUSTUP_TOOLCHAIN=1.95.0` (no rust-src). rust-analyzer logged `ERROR can't load standard library, try installing rust-src`, exited 0, and wrote an index (983,624 bytes). The ingest refused it: `GAP ...: the indexer log has can't load standard library; the index names no external symbol from std`, and no store file was created.
- Go positive: marvel at the pin, exit 0, 43,978,978 bytes, 68 out-of-root documents skipped; accepted with the file set from `go list`.
- Go negative: `internal/backoff/backoff.go` moved aside in the scratch clone (and moved back afterwards), scip-go exit 0, index 43,968,215 bytes. With `go list` output from the intact tree the ingest refused it: `GAP ...: the index has no document for internal/backoff/backoff.go, which go list says the module builds`, and no store was created.

## Results

| criterion | result | number |
|---|---|---|
| C0 coverage | PASS | kos 50 of 50, marvel 50 of 50; no GAP; no index exit other than 0; no trap line |
| C1 freshness | PASS | 50 of 50 versions: the store's code/ state equals the independent expected set, and every consecutive diff equals the set difference; load time max 3.39 s, p95 1.02 s, median 0.74 s over 100 loads (bar: under 10 s) |
| C2 provenance | PASS | 100 of 100 load entities carry all six prov/ values and are never retracted; the recorded commit equals the window commit at all 100; every one of 87,449 code/ intervals resolves to a complete load entity at its start version; W6's 178 changed rows 178 of 178; W7's 18 rows 18 of 18 |
| W7 one query, two repos | served | recall 1.000 and precision 1.000: kos 1 of 1 file, marvel 14 of 14 non-test files (the 18 call sites are reported, not scored); 3 marvel test files were also returned and are outside the ground truth by its definition |
| W6 a past change | partially served | kos 829c392 (version 4): 146 code/ facts added and 32 removed, equal to the independent set difference; no kos fact names 829c392 or #135 |
| W8 why | partially served | `code/depends_on (kos, serde_yaml)` live from version 1, ended at version 4 (829c392); provenance at both ends names rust-analyzer 1.97.1 and the commit; no `kos/names-dep` fact names serde_yaml or serde-saphyr |

How C1 was checked: `oracle.py` decodes each of the 100 index files with the probe's own decoder (`scipdec.py`) and builds the expected `code/defines`, `code/references`, `code/implements` and `code/depends_on` sets with the repo prefix and the own-version replacement, without calling the ingest. It compares them with the intervals of `kos facts export`. A mutant, one `code/defines` interval (asserted at version 1 and live to the end) dropped from a copy of the export, turned 50 of 50 states red and version 1's diff red. The oracle re-implements the ingest's own-package and `depends_on` rules, so it checks the ingest against a second implementation and not against the brief's text alone.

## Where the numbers differ from h57b

- **W6 counts**: 146 added and 32 removed here; the probe had 77 added and 213 removed. Both runs equal their own independent set difference, so the change tracking agrees with the files. The counts differ because the indexer differs (1.97.1 against 1.96.0), which changes the symbols emitted for the same source; I did not run 1.96.0 to confirm it.
- **rust-analyzer `dlmalloc` warning**: present in 50 of 50 kos logs in the probe, in 0 of 50 here. I take the indexer version to be the cause, unconfirmed.
- **Index sizes and fact counts**: kos index 1,810,190 bytes at version 1 and 2,440,295 at the pin (the probe's pin index was 1,552,519); marvel 42,123,479 to 43,978,975 bytes. Load line at version 1: kos 4,303 facts, marvel 77,949. Same cause, unconfirmed.
- **`kos/` knowledge facts**: 2 `kos/names-dep` facts here (1 kos, 1 marvel); the probe's kos file had 9 at version 50. Cause, established: the build reads `_kos/nodes/*/<id>.yaml` only, which is the reading I named in the SI-11 PR. The probe's `kos/` facts also came from the findings (`_kos/findings`), and the text that names serde_yaml is in findings 029, 030 and wre7, none of them nodes. So W8's record is not found by this reading. Whether to read findings too is a question for review; I have not changed it.

## One defect the run found

At the first kos load the ingest refused a healthy index: `GAP ... the index has no document for tracked file _kos/probes/code-graph-measurements-2026-10-21/trav/src/main.rs` (and three more `.rs` files under probe crates). The Rust check required a document for every tracked `.rs` file under any `src/` directory; kos tracks probe projects with their own `src/`. The brief's text is "under the crate's `src/`". The architect's review of the third group found the same thing independently. The fix, with red tests first, reads the root crate's `src/` and the `src/` of workspace members the root `Cargo.toml` lists, and is merged up the chain.

## What the chain's filtered line showed

The `filtered:` line now carries fact counts. At every marvel commit it reported `out_of_root_documents` 67 to 68 and `out_of_root_facts` 6,516 to 6,801, the same ranges the probe measured for the Go build cache's stray documents (67 to 68 documents, 6,516 to 6,801 facts). At every kos commit both were 0. `local_facts` ran from 4,316 at the first kos load to 5,826 at the last.

## Not run

- A rerun with rust-analyzer 1.96.0, since nothing may be installed.
- The probe's MDE tests (they ran at kos#192 in the probe).
- A counterfactual for the `dlmalloc` warning.

## Files beside this report

Under `si12/`: `idx-kos.tsv` and `idx-marvel.tsv` (per commit: exit, seconds, bytes, index sha256, trap lines, dlmalloc warnings), `loads2.log` (one line per load, with its time and filtered counts), `gomods.txt` (the Go module versions fetched), `oracle.py` (the C1 oracle; it uses the probe's `scipdec.py` from `code-graph-measurements-2026-10-20`). The index files, the store and the sandbox profile are not committed: the indexes are 2 to 44 MB each, and the profile lists the host's live ports.
