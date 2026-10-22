# Finding: fact store Probe C, growth with history to 200 commits (aae-orc-isie)

Status: draft, revision 2 (after review by the brief's author, 01M5GW8SD0)
Date: 2026-10-22
Host: kinu (Apple M3 Max, 128 GB RAM, macOS 26.5.2)
bd: aae-orc-isie
Question node: `question-kos-fact-store-substrate`, sub-questions A to C
Brief: `_kos/probes/brief-fact-store-probe-c-history-growth.md` (kos da21d20,
sha256 13da98e2...e9db3, locked)
Follows: finding-kos-8bmj (Probes A and B)
Tools: scip-go 0.2.7 (go1.26.5), Dolt 2.2.3, SQLite 3.53.4 (Python 3.14.7),
rustc 1.97.1 with sqlx 0.9.0
Evidence: `_kos/probes/code-graph-measurements-2026-10-22/`

## Headline

Both candidate stores pass every criterion, G0 to G5, at 200 commits of
marvel history. They separate on two ratios, in opposite directions. Dolt's
store is 3.4 times smaller than the SQLite valid-time store (12.0 MB against
40.6 MB). The SQLite store traverses 4.5 times faster as of a past commit
(80.5 ms against 364.9 ms, slowest of 100 seeds over 20 commits), with each
engine as deployed here: D answers each point lookup over a local sql-server
connection, while S-vt reads a file. The snapshot
layout fails on size, as expected: it passed 2 GB at commit 47.

History costs neither candidate anything measurable on the latest commit. The
200-commit store traverses d0616f98 within 5% of a store holding only that
commit, for both. The probe reports; it does not choose the store.

## Run

- 200 first-parent commits of marvel main, e84283eb (2026-10-06) to d0616f98
  (2026-10-20), oldest first. List sha256 cfbab4e7...395b.
- All 200 indexed and loaded in all three arms: no GAP and no load error. The
  fact count grows from 31,641 to 47,572.
- Indexing took 282 s in total, median 1.2 s per commit. Probe A took 19 s
  for one commit. The gap is likely the build cache, which the pre-check had
  warmed here; that was not measured.
- No value passed Dolt's VARCHAR(255): the longest `e` or `v` is 168 bytes.

### Run conditions and controls

All of them under the brief's profile (`bin/run.sb`), with a scratch HOME and
TMPDIR from `env -i`, and no MARVEL_SOCKET. Every control gave the required
result (`results/controls.txt`):

- denied scratch port 47999 refused with "Operation not permitted";
- undenied scratch port 47998 connected;
- a public address refused inside the sandbox (it connects outside);
- a write to a fixed `/private/tmp` path outside scratch refused, the file
  absent afterwards, and a write inside scratch succeeded.

`run.sb` keeps the brief's profile verbatim. It adds a deny for every other
localhost port kinu listened on at 03:10Z (27 ports), plus scratch control
port 47997, denied first in the block. So both the first and the last end deny
were shown to hold. Every addition only narrows what the run can reach. The
Dolt server bound 127.0.0.1 on port 45459, which is not a denied port. It was
stopped by pid each time, and confirmed gone with `kill -0`.

### Pre-check, under a dated note

The brief's pre-check, `go list -m all` offline, failed at 59 of 200 commits,
d0616f98 among them. That is a stop under the brief as written.

All 59 miss the same two modules: golang.org/x/sync v0.24.0 and
golang.org/x/tools v0.51.0. They are in the module graph, but no package
loads them, and Probe A indexed d0616f98 on this host under the same flags.

The brief's author agreed to replace the pre-check, under four conditions, and
director ruled it. The replacement is `go list -deps -test ./...` under the
same flags and the same stop. Its positive control at d0616f98 passed, then
the full pass gave 0 of 200 failures. G0 was unchanged: the 59 got no
allowance. The note and both results are in `results/run-notes.md` and
`results/precheck*.txt`. The timebox started at 03:14:40Z.

### Draws

`random.Random(20261022)`, one fresh generator per draw, in the brief's order
(`draws/draws.json`).

- **G4 commits:** 20 commits from the indexed list (`draws/indexed.txt`,
  sha256 2cd9ecf1...a12).
- **G3 pairs (review note 6):** 50 pairs, sampled from the list of all
  (i, j), i < j, over the indexed list, in lexicographic order of (i, j).
  That list has 19,900 pairs.
- **Seed pool (note 3):** 4,899 DEFINES symbols that have a reference at all
  20 G4 commits and at d0616f98. 100 seeds were drawn from it.
- **Empty counts (note 5):**
  - seeds: 0 at every one of the 21 as-of commits (the 20 G4 commits plus
    d0616f98);
  - hot set: 9 at the 13 earliest G4 commits, 7 at the next three, and 0 at
    the last four and at d0616f98. The hot set is drawn at d0616f98, and some
    of its symbols do not exist yet at the older commits.

## Results

| | S-vt (SQLite valid-time) | D (Dolt) | S-snap (reported only) |
|---|---|---|---|
| G0 coverage | 200/200: PASS | 200/200: PASS | 47 loaded, then stopped |
| G1 size, allocated (apparent) | 41,588 KB (41,241): PASS | 12,276 KB (12,251): PASS | FAIL at 47 commits; 1,294,400 KB after checkpoint |
| G2 load p95; median of last 20 / first 20 | 0.14 s; 0.120 / 0.077 = 1.56: PASS | 1.38 s; 1.241 / 0.980 = 1.27: PASS | not scored |
| G3 diff p95 (50 pairs equal to the outside set difference) | 674 ms (50/50): PASS | 15 ms (50/50): PASS | not scored |
| G4 median of the three slowest (runs) | 80.5 ms (80.5, 161.3, 78.9): PASS | 364.9 ms (364.9, 369.6, 358.9): PASS | not scored |
| G4 hop sets equal to the join on the commit's fact file | all three hop sets equal in 2,000/2,000 traversals (6,000 sets) | all three hop sets equal in 2,000/2,000 traversals (6,000 sets) | |
| G5 200-commit / 1-commit at d0616f98 | 80.8 / 77.1 ms = 1.05: PASS | 353.8 / 342.0 ms = 1.03: PASS | not scored |
| G5 runs, 200-commit store | 80.810, 81.475, 80.043 ms | 353.797, 353.865, 299.580 ms | |
| G5 runs, 1-commit store | 78.086, 77.089, 74.716 ms | 349.654, 342.023, 309.109 ms | |

### Notes on the table

- **G1.** Measured once, after the last commit loaded and before G3 to G5
  ran. D after `CALL dolt_gc()` with the server stopped. The SQLite arms
  after `PRAGMA wal_checkpoint(TRUNCATE)` with the connection closed, so no
  `-wal` or `-shm` file remained.
  - S-snap's stop read `du -sk` during the load, which counts the WAL. It
    stopped at commit 47 at 2,100,012 KB.
  - It grew about 44 MB a commit while loading. Even at its checkpointed size
    (1.29 GB at 47 commits), 200 commits would pass 2 GB several times over.
- **G2.** Measured from the fact file to committed rows:
  - S-vt: reads the live set from the store and writes the interval changes,
    in one long-lived connection;
  - D: the Probe A path through the dolt CLI (read the previous rows,
    `table import -r`, `add`, `commit`).
  The arms were loaded one at a time; see "Deviations".
- **G3.** S-vt: two EXCEPT queries over the valid-time predicate. D:
  `dolt_diff()` counted by `diff_type`. Each query was timed in process by
  `qtime` (`trav/src/bin/qtime.rs`).
- **G4.** A traversal counts as equal only when all three hop sets (hop 1,
  hop 2, and the result) equal the join, as sets (`bin/g4check.py`). S-vt's
  run 2 (161 ms) is a single outlier. The scored median is run 1.
- **G4 hot set (not scored):**
  - S-vt: median of the slowest 190.3 ms (333.98, 190.34, 173.75);
  - D: one run only, 702.6 ms; runs 2 and 3 were stopped to save about an
    hour;
  - all hop sets equal.
- **G5.** Both one-commit stores hold d0616f98 alone, loaded by the same
  loader, and are traversed as of that commit with the same query form. All
  three hop sets equal the join in all 100 traversals of every store.

## Separation

By the brief's rule, comparing S-vt and D only:

- **Coverage:** no separation; both pass G0.
- **A bar:** no separation; both pass G1 to G5.
- **A ratio:** they separate twice.
  - G1: S-vt's allocated size is 3.39 times D's, so D is smaller.
  - G4: D's number is 4.53 times S-vt's, so S-vt is faster.
  - G5: no separation (1.05 against 1.03).

Each store wins one ratio. That is a trade, not a ranking, and choosing
between them stays with the operator.

## The three MDE killers, as measured

| | S-vt | D |
|---|---|---|
| Lock-in: export to JSONL, re-import, per-commit counts and the 50 pair diffs equal | 200/200 and 50/50 | 200/200 and 50/50 |
| Exit cost: export, import | 0.22 s, 0.48 s (9.6 MB) | 17.8 s, 199.2 s (10.0 MB) |
| Round trip: 100 facts as of d0616f98 to YAML and back | equal | equal |
| Engine-specific verbs | `PRAGMA journal_mode`, `synchronous`, `wal_checkpoint` | `AS OF`, `dolt_diff()`, `HASHOF()`, `CALL dolt_gc()`, `dolt_log`; CLI `table import`, `add`, `commit` |

### Notes on the MDE table

- **Exit cost.** D's export is an event log built from `dolt_diff()` between
  consecutive commits. Its import replays that log as one Dolt commit per
  marvel commit through the CLI, so most of the 199 s is 200 CLI round trips.
  A bulk path may exist; it was not looked for.
- **Rewrites.** No query was rewritten for speed after its first run.

## Predictions

The research-supervisor's predictions, registered before the run:

- 0.8 that both pass G0 to G5: held.
- 0.7 that D is at least 3 times smaller on G1: held (3.39).
- 0.6 that S-vt's G4 is lower than D's: held (4.5 times).
- 0.9 that S-snap fails G1: held.

The falsifier does not fire: neither arm fails G2 or G4.

## Deviations, each with a dated note in `results/run-notes.md`

1. **Profile:** the extra port denies described under "Run conditions". The
   brief adds such denies "before the lock"; run note 1 reads that as before
   the run, which is when they were added.
2. **Pre-check:** the change above, with the brief's author agreeing and
   director ruling it.
3. **Load restart.** The three loaders were first started together, which
   would have put every G2 timing on a shared machine.
   - They were stopped by pid after 5 to 62 commits.
   - All three were reloaded from empty, one arm at a time, into fresh
     directories. Only those loads are scored.
4. **D hot set:** one run of three, as noted under the table.
5. **One-commit D store.** The first load failed because the sql-server held
   the repo. It was loaded again with the server stopped, and the failed log
   is kept.
6. **Review notes 1 to 6 from review 5608782453.** All six are recorded at
   run start. Note 2 is answered here:
   - free disk was 410,662,432 KB before indexing;
   - the 200 indexes took 4,134,228 KB, the fact files 1,225,468 KB, the S-vt
     store 41,588 KB, D 12,276 KB, and S-snap up to 2,100,012 KB.

## Limits

- **One host, one repository, warm caches.** Indexing times are not
  comparable to Probe A's cold 19 s.
- **G2 measures two different load paths.** S-vt runs in process; D goes
  through the CLI, as Probe A ran it. D's 1.4 s p95 includes process starts.
- **D runs as a sql-server.** Its traversal pays a network round trip per
  point lookup; S-vt opens a file. That is part of each engine as deployed
  here, and may explain much of the G4 gap.
- **The next test that could change the trade** would vary query shape on D
  (the 13 times swing in finding-kos-8bmj) or scale history by another order.
  Neither was run.

## Evidence

Kept in the evidence directory:

- the run profile, every script and the trav and qtime source (`bin/`, `trav/`);
- the draws with their hashes;
- every per-commit load log;
- the G1 to G5 outputs and the MDE results;
- the controls, the pre-check outputs and the run notes.

Kept out, and listed with sha256 and size in `results/outputs-kept-out.txt`:
the indexes, fact files, stores, exports, binaries and traversal set files.
The as-run `bin/rc.sh` named the scratch root, PATH and GOMODCACHE as
literals; the published copy names them as variables and is otherwise the
same.
