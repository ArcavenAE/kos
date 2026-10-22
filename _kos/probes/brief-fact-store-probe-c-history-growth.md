# Brief: fact store Probe C, growth with history (aae-orc-isie)

Status: DRAFT, written before any measurement. Lock it before the probe runs:
it is locked when its kos PR merges after review, and nothing below changes
after that without a dated note.
Date: 2026-10-21. Revised 2026-10-22 for review 5608546539 on kos#143, before
the lock and before any measurement.
Question node: question-kos-fact-store-substrate, sub-questions A to C
Follows: finding-kos-8bmj (Probes A and B), section "Limits", which names this
test as the one that would separate the stores.

## Question

Probes A and B loaded two commits, and both stores passed every criterion with
wide margins. Two commits cannot tell the stores apart, because history is
where their layouts differ: Dolt shares structure across commits and keeps
only the latest rows plus first-seen tx, while an SQLite fact log either
repeats a snapshot per commit or keeps one row per fact interval. So: as
marvel's history grows from 2 to 200 commits, does each layout keep its load,
diff and traversal inside the bars, and how far apart do they end up?

## Arms

Three arms, each with its physical design fixed here so the result measures
the engine and layout, not a design choice made during the run (the blind run
in finding-kos-8bmj showed Dolt's hot tail moving about 13 times on query shape alone):

- **S-vt, SQLite valid-time.** `facts(e, a, v, valid_from, valid_to)`, one row
  per fact interval, `valid_to` null while live. Four covering indexes: EAVT,
  AEVT, AVET and VAET, each carrying `valid_from` and `valid_to`.
- **S-snap, SQLite snapshot.** One full fact set per tx, as Probe B ran it,
  with the same four indexes. This arm is expected to fail criterion G1; it is
  here to put a number on the snapshot layout's cost, not as a candidate. It
  is stopped once its store passes 2 GB: G1 is then FAIL with the commit count
  reached, and its G2 to G5 are not scored.
- **D, Dolt.** One Dolt commit per marvel commit. Primary key (e, a, v), one
  secondary index (a, v, e), `tx` as the first-seen marvel commit, as Probe A
  ran it.

Column types and settings, fixed: in both SQLite arms `e`, `a` and `v` are
TEXT and the tx columns INTEGER, with `PRAGMA journal_mode=WAL` and
`PRAGMA synchronous=NORMAL`; in D, `e`, `a` and `v` are VARCHAR(255) as in
Probe A, and a value longer than that is a load error for that commit, counted
under G0. A fact removed and later re-added is a new interval in S-vt and a
delete then insert in D; both are expected, and no criterion reads tx.

Every arm traverses in Rust by indexed point lookups, one per frontier element
(the Probe A and B traversal), never by batched IN lists.

## Inputs

- **Commits:** the 200 first-parent commits of marvel main ending at d0616f98,
  oldest first. The list is written to a file and its sha256 recorded before
  any index is built.
- **Clone:** the scratch clone of marvel is made from the run host's existing
  local checkout (`git clone --no-local` from its path), never from the network.
- **Indexes:** one scip-go index per commit, from the scratch clone, under the
  run conditions below. A commit whose index fails to build is a GAP, recorded
  with its error line, and the next commit's diff is taken against the last
  commit that did build.
- **Extraction:** the published `scipdec.py` and `facts.py` from
  `code-graph-measurements-2026-10-21/bin/`, including the own-module version
  normalization, which is part of this brief from the start.
- **Traversal build:** the Rust traversal from Probes A and B, built with
  `cargo build --offline`, or the existing build reused, its sha256 recorded.
- **Draws:** every random draw uses Python's `random.Random(20261022)`, one
  fresh generator per draw, with `.sample` over a list sorted by commit order
  or by symbol string. Draws are taken only from commits that indexed (GAP
  commits are excluded), in this order: first the 20 G4 as-of commits, then
  the 50 G3 pairs, then the seeds.
- **Seeds:** the seed pool is the DEFINES symbols that have at least one
  tracked reference at every one of the 20 G4 as-of commits and at d0616f98.
  100 seeds are drawn from that pool, so no seed is empty at any commit it is
  traversed at (finding-kos-8bmj found 34 of 100 empty seeds at one commit,
  and marvel's tracked `.go` files grow from 425 to 730 over this window). If
  the pool holds fewer than 100, all of it is used and the count is reported.
  The empty-seed count is reported for every as-of commit; any nonzero count
  is a defect in the draw, not a result. The hot set is the 100
  most-referenced DEFINES symbols at d0616f98, as before.

## Pre-check, before the timebox starts

Under the run conditions below, `go list -m all` runs offline at each of the
200 commits. The count that fail is reported first. If more than 10 fail, G0
cannot pass on this host's module cache, and the probe stops there and reports
that count, before the two days are spent.

## Decision criteria

Each criterion has a bar every arm must meet. Results are reported per arm as
PASS or FAIL with the number.

- **G0, coverage.** At least 190 of the 200 commits index and load. An arm
  below that reports its GAP count, and no other criterion is scored for it.
- **G1, size.** Store size on disk after all commits load: under 2 GB. It is
  measured once, after the last commit loads and before G3 to G5 run, as
  allocated size: `du -sk` of the arm's store directory, holding every file
  the store writes. Before it: for D, `CALL dolt_gc()` and then the server
  stopped; for the SQLite arms, `PRAGMA wal_checkpoint(TRUNCATE)` and then the
  connection closed, with any `-wal` and `-shm` file counted. The apparent
  size (`du -sk -A` on macOS, `du -sk --apparent-size` on Linux) is reported
  beside it, not scored.
- **G2, incremental load.** Loading one commit onto the existing store, from
  fact file to committed rows: p95 under 10 s, and the median of the last 20
  commits no more than 2 times the median of the first 20.
- **G3, diff at any pair.** 50 commit pairs (i, j), i < j: each pair's added
  and removed counts equal a set difference on (e, a, v), tx ignored, computed
  outside the store from the two fact files. p95 diff time under 5 s.
- **G4, historical traversal.** The three-hop traversal of Probe A criterion 4,
  run as of the 20 drawn commits, from the 100 seeds. Correctness: at every
  commit, each of the three hop sets equals, as a set, the plain join on that
  commit's fact file. Time: three runs; each run's number is its slowest
  traversal over all seeds and commits; the scored number is the median of the
  three, under 1 s, with all three reported. The hot set is run as of the same
  20 commits and reported, not scored.
- **G5, history tax on the latest commit.** The seeds, traversed at d0616f98
  after all 200 commits load, against the same seeds traversed at d0616f98 in
  a store holding only that commit. Each side is three runs, each run's number
  is its slowest traversal, and each side's number is the median of its three.
  Bar: the 200-commit number is no more than 2 times the one-commit number. All
  six runs are reported.

## What counts as separating

Only the two candidate arms, S-vt and D, are compared. S-snap's numbers are
reported, and it never counts as the passing or the failing arm.

- **Coverage.** If one candidate fails G0 and the other passes, they separate
  on coverage, and the passing arm's G1 to G5 are still scored and reported.
  If both fail G0, the probe reports no separation, with each arm's GAP causes.
- **A bar.** Otherwise they separate if one passes a criterion G1 to G5 that
  the other fails.
- **A ratio.** Otherwise they separate if, on G1, G4 or G5, one arm's scored
  number is at least 3 times the other's: for G1 the allocated size, for G4
  the median of the three slowest values, for G5 the ratio of its two medians.

If none holds, the probe reports that history to 200 commits does not
separate them, and names the next scale to try. The probe reports; it does not
choose the store. The choice stays with the operator.

## The three MDE killers, answered before the probe runs (kos roadmap v3, M4)

- **Tool lock-in.** Answer to test: each candidate arm's full history exports to
  JSONL and re-imports into a fresh store whose per-commit fact counts and
  pair diffs (the G3 pairs) are equal to the source's. Exit cost: the export
  and import times at 200 commits, measured.
- **Round-trip breakage.** Answer to test: the same 100-fact harvest as Probes
  A and B, taken as of d0616f98 from the 200-commit store, goes to YAML and
  back with the same facts and attributes. Any loss is a fail.
- **Skill asymmetry.** Answer to test: every query is plain SQL except Dolt's
  `AS OF` and `dolt_diff()`. Record every engine-specific verb, and any query
  that had to be rewritten for speed after its first run.

## Run conditions

- A scratch HOME and TMPDIR built with `env -i`; no MARVEL_SOCKET.
- `GOPROXY=off GOFLAGS=-mod=readonly GOTOOLCHAIN=local` for every index build
  and for the pre-check. A commit whose module set is not already in the cache
  fails to build, and is a GAP under G0, never a fetch.
- `sandbox-exec` with this profile, in this order, the port denies last so
  they override the localhost allow; the bd Dolt server and message broker
  ports on the fleet hosts are among them:

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
  (deny network-outbound (remote ip "localhost:47999"))
  ```

  47999 is a scratch control port, not a live service. A run host whose live
  services listen on other localhost ports adds a deny for each before the
  lock, and the report names them. The localhost allow is `network*`, not
  `network-outbound`: with an outbound-only allow and no other outbound rule
  between it and the denies, the end denies are skipped and a denied port
  connects (measured on two fleet hosts, 2026-10-22). The 2026-10-21
  `run.sb` has that shape and is not reused as is.
- Controls, each under the run's own profile, each result in the report:
  - a connection to a non-local address, refused by the profile with
    "Operation not permitted";
  - a scratch listener on 47999 and one on 47998, both started inside the
    sandbox: the connection to 47999 is refused by the profile with
    "Operation not permitted", and the one to 47998 succeeds;
  - a write to a fixed path under `/private/tmp` outside `PROBE_DIR`, refused
    by the profile, and a write inside `PROBE_DIR`, which succeeds.
  No control connects to a live port. If any control gives the wrong result,
  above all if the connection to 47999 succeeds, the run stops before any
  probe step and reports the control.
- The Dolt server binds 127.0.0.1 by name (`dolt sql-server --host 127.0.0.1`)
  on a random localhost port that is none of the denied ports. It is stopped
  by its pid with `kill -TERM` and checked gone with `kill -0`.
- Tools already installed on the run host, versions recorded. A host without
  scip-go, Dolt or a Rust toolchain does not run this probe until the
  operator's package-fetch ruling covers the install, with pinned versions.
- Disk: about 8 GB free before the run starts (the 200 indexes plus S-snap up
  to its 2 GB stop), checked and reported.

## Timebox

Two working days after the pre-check passes. Indexing 200 commits is expected
to take about an hour, at the 19 s per commit measured for d0616f98.

## Prediction (registered)

The research-supervisor's, registered before any run:
- 0.8 that S-vt and D both pass G0 to G5;
- 0.7 that D's G1 allocated size is at least 3 times smaller than S-vt's, which
  would separate them on size;
- 0.6 that S-vt's G4 number is lower than D's, as in the two-commit runs;
- 0.9 that S-snap fails G1.

## Falsifier

If S-vt and D both fail G2, or both fail G4, at 200 commits, the
versioned-store-below shape does not hold with history at marvel's scale, and
the flat-file ceiling argument needs restating against a different store
family.

## Outputs

- A finding in kos `_kos/findings/`, with the id minted by
  `kos id finding <slug>`.
- A table of G0 to G5 per arm, PASS or FAIL with the number, and the
  separation result against the rule above.
- The pre-check count, the empty-seed count per as-of commit, every run behind
  the G4 and G5 medians, and both sizes for G1.
- The MDE answers as measured, and the GAP list from G0.
- Evidence under `_kos/probes/code-graph-measurements-<run date>/`, same
  pattern as 2026-10-21, including the run's `run.sb`.
