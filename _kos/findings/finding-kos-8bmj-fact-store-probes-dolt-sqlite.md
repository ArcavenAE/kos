# Finding: fact store Probes A (Dolt) and B (SQLite), measured on marvel (aae-orc-isie)

Status: draft, revision 2 (after review by the corporate research seat)
Date: 2026-10-21
Host: kinu (Apple M3 Max, 128 GB RAM, macOS 26.5.2)
bd: aae-orc-isie
Question node: `question-kos-fact-store-substrate`, sub-questions A to D
Briefs: `_kos/probes/brief-fact-store-probe-a-dolt.md`,
`_kos/probes/brief-fact-store-probe-b-sqlite.md` (both at kos 41fccf5)
Tools: scip-go 0.2.7 (go1.26.5), Dolt 2.2.3, SQLite 3.51.0 (CLI and the
Python 3.9 loader) and 3.53.4 (Python 3.14, harvest and round trip), rustc
1.97.1 with sqlx 0.9.0, tokio 1.53.3

## Headline

Both stores pass every criterion at marvel's scale, with wide margins. Each
commit loads from SCIP bytes to committed rows in 1.4 to 3.0 seconds, against
a 60-second bar. The diff counts match an outside set difference exactly on
both stores. A 100-fact harvest leaves the bulk behind on both. The criterion 4
traversal runs in Rust over indexed point lookups; the slowest of 100 seeded
symbols took 32 ms on SQLite and 71 ms on Dolt, against a 1-second bar.

Probe B also meets its fifth criterion, the build cost: from the first line of
schema to the first passing load took 86 seconds of a 4-hour timebox. Under
Probe B's own framing, that leaves Dolt to earn its server with the
harvest-by-branch step, and the SQLite harvest kept the bulk out as well.
What the measurements say Dolt adds is listed under "What separates them". The
choice between the two is not made here.

The falsifier does not fire on either store. The traversal never pulled the log
into memory: its peak resident set was 4.0 MB against Dolt and 7.9 MB against
SQLite.

A design written blind from the Probe A brief, by a second seat, ran as
written and also passes criterion 4 on the brief's seeds (129 ms). On an extra
worst-case seed set outside the brief, Dolt stays under 1 s only with point
lookups over a covering index (646 ms). With the blind design's batched IN
queries or its non-covering index, it takes 2.8 to 8.3 s. See "The blind Dolt
run".

## Inputs

| input | commit | how | sha256 (16) | facts |
|---|---|---|---|---:|
| 10-16 marvel index | ac67c078 | scip-go 0.2.7, built 2026-10-16 (the `marvel-warm1` run behind fact-count.md) | 612ecd9342df82d8 | 43,263 |
| fresh marvel index | d0616f98 (marvel main, 2026-10-21T00:56Z) | scip-go 0.2.7, built 2026-10-21 under the run conditions below, 19 s | 1544aec3ad5d6909 | 47,572 |

The 10-16 index was still on kinu, so it was not rebuilt. Decoded with the
published `scipdec.py` and `count.py`, it reproduces fact-count.md's tracked
row exactly: 650 documents, 9,796 DEFINES, 33,176 REFERENCES, 312 IMPLEMENTS,
43,284 in total. The fresh index gives 721 documents and 47,593 by the same
count.

Run conditions for the fresh index and for every store step:
- a scratch HOME and TMPDIR, built with `env -i`;
- no MARVEL_SOCKET (checked inside: absent);
- `GOPROXY=off GOFLAGS=-mod=readonly GOTOOLCHAIN=local`;
- `sandbox-exec` denying the network except localhost. Control: a connection to
  140.82.112.3:443 was refused inside with `PermissionError(1, 'Operation not
  permitted')` and connected outside;
- writes denied outside scratch and `/dev`. Control: a new file at a fixed path
  outside scratch and outside any home, `/private/tmp/fsp-outside-probe`, was
  refused with `PermissionError(1, 'Operation not permitted')` and does not
  exist afterwards; a scratch write succeeded. The profile is `bin/run.sb`.
  The control follows the operator's Desk ruling cond4-1 (2026-10-20): the
  blocked write targets a fixed path outside scratch and outside any home,
  never the real `$HOME`, as amended into the orchestrator's run-conditions
  rule. The review asked for a write to the real home, quoting the rule's
  earlier text; that was not done, for this reason.

The Dolt server ran on 127.0.0.1:42498, a random free port. The fleet's bd
server (port 3307) and its data directory were not touched. The scratch server
was stopped by its pid, and `kill -0` confirmed it was gone.

## Fact model as loaded

From fact-count.md, tracked documents only, with local symbols left out:

| a | e | v |
|---|---|---|
| DEFINES | document | symbol |
| REFERENCES | document | symbol (one per distinct pair) |
| IMPLEMENTS | symbol | symbol |

Two departures from fact-count.md's arithmetic, both small:
- **Facts are distinct triples.** fact-count.md counts DEFINES occurrences
  (9,796); as a set of (document, DEFINES, symbol) facts there are 21 fewer, so
  ac67c078 loads 43,263 facts, not 43,284.
- **Own-module version.** scip-go writes `--module-version` into every marvel
  symbol, so without a change no marvel symbol would survive from one commit to
  the next. The loader replaces the marvel module's own version token with `.`.
  Dependency versions are kept, since a dependency bump is a real change of
  fact. This normalization was not in either brief: it was added after the
  briefs were locked, and criterion 2's numbers depend on it. Without it, the
  same two commits give 34,536 added and 30,227 removed (DEFINES 10,714 and
  9,775; REFERENCES 23,469 and 20,140; IMPLEMENTS 353 and 312), because every
  marvel symbol changes its version token.

Both schemas are `facts(e, a, v, tx)`, but tx means different things:
- **SQLite:** one full snapshot per tx (tx 1 and 2, 90,835 rows), with the four
  covering indexes EAVT, AEVT, AVET and VAET. The diff is a set query over tx.
- **Dolt:** the table holds the fact set at the latest commit, with primary key
  (e, a, v) and one secondary index (a, v, e). tx is the marvel commit at which
  the fact was first asserted, and unchanged facts keep it. So the row diff
  between the two Dolt commits is exactly the added and removed facts, with no
  "modified" rows.

## Results

| criterion | bar | Probe B, SQLite | Probe A, Dolt |
|---|---|---|---|
| 1 load, per commit | under 60 s | 1.44 s and 1.87 s: PASS | 2.02 s and 3.03 s: PASS |
| 2 diff counts equal an outside set difference | equal | 4,862 added and 553 removed, by EXCEPT over tx; the outside set difference gives the same, and the added sets are equal as sets: PASS | the same counts by AS OF with EXCEPT and by `dolt_diff()`, with no modified rows; the added set equals the outside set: PASS |
| 3 100-fact harvest, YAML out, bulk left behind | bulk not carried | a separate 24 KB file against the 76.5 MB store; YAML round trip equal: PASS | a branch off the schema-only commit, pushed to a fresh file remote: 24 KB against 2.8 MB for main; YAML round trip equal: PASS |
| 4 traversal in Rust, 100 seeded symbols | slowest under 1 s, results equal to the plain join | slowest 32.4, 18.2 and 16.9 ms over three runs; 100 of 100 equal: PASS | slowest 57.4, 69.3 and 71.4 ms over three runs; 100 of 100 equal: PASS |
| 5 build cost, Probe B only | inside 4 h | 86 s, from 01:00:17Z to 01:01:43Z: PASS | not a criterion |

The criterion 5 clock: the start was written to a file at 01:00:17Z, before
any probe file existed (its mtime is the same second). The extractor
`facts.py` was saved at 01:01:14Z and the loader `load_sqlite.py` at
01:01:26Z. The store file and the pass marker both carry 01:01:43Z.

The diff by attribute was 998 DEFINES, 3,793 REFERENCES and 71 IMPLEMENTS
added, and 59, 464 and 30 removed, identical on both stores and outside.

**The harvest.** Both probes took the same 100 facts, drawn with seed 20261021
from the d0616f98 fact set sorted by (e, a, v): 27 DEFINES, 71 REFERENCES and
2 IMPLEMENTS. Both YAML files load back to the same facts and attributes. The
YAML shape is this probe's (`kind: harvested-facts`, a source block, a fact
list); kos has no fact schema yet, so the round trip proves semantic equality
of this shape, not conformance to a kos node.

**The traversal.** `trav/src/main.rs` is one Rust binary for both
stores, using sqlx over SQLite or the MySQL wire protocol. The hops' set logic
runs in Rust, and the store answers only indexed point lookups, one per
frontier element: 6,859 lookups for the 100 seeds. The plain join outside the
store (`join_check.py`) was built from the extracted fact file, not from either
store. The first runs compared hop 3 as a set and hops 1 and 2 by size. After
review, the traversal was changed to emit all three hop sets and re-run once
on each store: 100 of 100 equal at every hop as sets, slowest 59 ms on SQLite
and 68 ms on Dolt.

**Seeds.** The 100 seeds came from the distinct DEFINES symbols at d0616f98
(10,043 of them), seed 20261021 (`seeds-d0616f98.txt`, sha256 b6ad49c43470c1b0).
34 of the 100 have no tracked reference, so hop 1 is empty and the result is
the empty set. This makes the criterion easier than it looks. A future brief
could draw its seeds from symbols with at least one reference.

So I added a run that is not a criterion: the 100 most-referenced DEFINES
symbols (`seeds-hot.txt`, at most 226 referencing documents, 126,339 lookups).
The slowest was 268 ms on SQLite and 646 ms on Dolt, with 100 of 100 equal to
the plain join. Both stay under the bar, and Dolt's margin is the narrower one.

| run | p50 ms | p95 ms | max ms |
|---|---:|---:|---:|
| SQLite, criterion seeds (run 1) | 1.9 | 18.2 | 32.4 |
| Dolt, criterion seeds (run 1) | 3.6 | 42.9 | 57.4 |
| SQLite, hot seeds | 45.0 | 103.2 | 268.3 |
| Dolt, hot seeds | 155.2 | 381.1 | 646.3 |

## The three MDE killers, as measured

- **Tool lock-in.**
  - SQLite: the whole log (both tx) exported to JSONL (15.6 MB, 0.43 s) and
    re-imported into a fresh file (0.16 s). The per-tx counts were 43,263 and
    47,572, the same as the source, and the sets were equal.
  - Dolt: both commits exported AS OF to JSONL (17.6 MB) and re-imported into a
    fresh Dolt repository as two commits, in about 3 s. Each commit's ordered
    export is byte-identical to the source's, and the fresh repository's
    `dolt_diff` gives the same 4,862 added and 553 removed.
  - Exit cost, measured: one dump of under 1 s for SQLite, and about 3 s each
    way for Dolt at this scale.
- **Round-trip breakage.** None. Both harvests went to YAML and back with the
  same facts and attributes. This is to this probe's YAML shape; see the
  harvest note above.
- **Skill asymmetry.**
  - SQLite used plain SQL only: EXCEPT, GROUP BY and indexed lookups. It needed
    no recursive CTE, because the traversal lives in Rust.
  - Dolt-specific verbs used:
    - in SQL: `AS OF`, `dolt_diff()`, `dolt_log`, `DOLT_BRANCH`, `DOLT_ADD`,
      `DOLT_COMMIT`, `DOLT_REMOTE` and `DOLT_PUSH`;
    - on the CLI: `dolt init`, `dolt table import -r`, `dolt add`,
      `dolt commit` and `dolt sql-server`.
  - Two Dolt details the brief did not anticipate:
    - The database name is the directory name, `dolt-a`, not `dolt_a`.
    - `dolt commit` has no `-q`.
  - The brief's "cherry-picked to a branch" is not `dolt cherry-pick`, which
    applies whole commits. The 100 rows went to the harvest branch by
    `INSERT ... SELECT ... FROM facts AS OF 'main'`.
  - **Tested blind.** Whether a fresh session writes these queries from the
    brief alone: see "The blind Dolt run" below. It did, and every query ran
    as written. The cost showed in physical design, not in the Dolt verbs: on
    the hot seeds the blind design took 8,271 ms, against 646 ms here. A future
    brief should name the covering indexes and the lookup shape.

## The blind Dolt run

A second research seat, on another host, wrote a full Probe A design from the
Probe A brief alone. That seat had no Dolt, Rust or inputs. Its design file
carries sha256 f4b0e3a88547df66, written before it saw this finding or its
scripts. It was run here as written, against the same two commits, in a fresh
scratch Dolt repository under the same run conditions.

Its design:
- **Schema:** `facts(k CHAR(64) PRIMARY KEY, e VARCHAR(2048), a VARCHAR(32),
  v VARCHAR(2048), tx CHAR(40))`, where k is the sha256 of e, a and v joined
  by `\x1f`. Indexes `av(a, v)` and `ae(a, e)`.
- **tx:** carried forward from the commit where each fact first appeared.
- **Commits:** tags `c1` and `c2`.
- **Traversal:** one query per hop, with IN lists for hops 2 and 3.

| step | blind design | this finding's design |
|---|---|---|
| load (import, add, commit, tag) | about 2 s and 1 s | 2.02 s and 3.03 s, from SCIP bytes |
| `dolt_diff` by type and attribute | the same 998, 3,793 and 71 added; 59, 464 and 30 removed; no modified | identical |
| AS OF diff | NOT EXISTS on k: the same counts, both directions | EXCEPT: identical |
| the design's own note, tx as the loaded commit | confirmed: 4,862 added, 42,710 modified, 553 removed | not applicable (carried forward) |
| harvest to its own empty remote | 100 rows, branch log is init only, push 48 KB | 24 KB |
| EXPLAIN of each hop | IndexedTableAccess on av, ae, av | |
| criterion 4, the brief's seeds, slowest | 126 to 129 ms over three runs; 100 of 100 equal: PASS | 57 to 71 ms: PASS |
| lock-in, AS OF c1 to JSON and back | ordered rows byte-identical | identical |

The JSON export does not carry k, so the re-import recomputes it. The
round trip works, but that export is not self-contained.

**The hot-seed extra, crossed by schema and query shape** (all Dolt 2.2.3,
the same 100 most-referenced seeds, 100 of 100 equal to the plain join in
every cell):

| Dolt schema | query shape | slowest | under 1 s |
|---|---|---:|---|
| blind: PK k, `av(a, v)`, `ae(a, e)`, VARCHAR(2048) | batched IN | 8,271 ms | no |
| this finding: PK (e, a, v), `avet(a, v, e)`, VARCHAR(255) | batched IN | 8,228 ms | no |
| blind | point lookups | 2,818 ms | no |
| this finding | point lookups | 646 ms | yes |

So on Dolt the hot tail depends on the query shape first and the index second:
- The batched IN form costs about the same on both schemas.
- A secondary index that does not cover e, and so needs a primary-key fetch
  per row, costs about 4 times the covering one under point lookups.
- The slowest batched seed fanned out to 226, 3,841 and 373 at its three hops.
- How much of the IN time is per list was not measured per query.

SQLite with point lookups took 268 ms on the same set. None of this changes a
criterion: the brief's seeds pass under every shape tried. It does mean Dolt's
latency margin at a worst-case seed depends on physical design that the brief
does not specify.

## What separates them

Measured differences only:

| | SQLite | Dolt |
|---|---|---|
| process | none, a file | a server: 176 MB RSS idle after the runs, one port, one pid to manage |
| store size, two commits | 36.5 MB with one row per fact, valid_from and valid_to, and the four covering indexes (48,125 rows, the like-for-like layout); 76.5 MB with the full snapshot per tx actually used (90,835 rows) | 5.3 MB `.dolt` (latest rows plus first-seen tx, structural sharing), 2.8 MB pushed |
| diff | a query over tx, or over valid_from and valid_to (4,862 added and 553 removed by `valid_from = 2` and `valid_to = 2`) | native, AS OF or `dolt_diff`, with no tx bookkeeping beyond first-seen |
| harvest | a separate file, which carries no history | a branch whose ancestry is the schema commit only, pushable on its own |
| traversal latency | about 2 times faster at the tail | slower; under the bar at the hot tail only with point lookups over a covering index (see the blind run) |
| build cost | 86 s, inside the 4-hour box | not measured separately (it ran second and reused the extractor and traversal) |

The size row compares layouts as well as engines. The first draft set Dolt
against the snapshot layout only, which overstated Dolt's size edge by about
2 times. The valid-time SQLite file was built after review
(`bin/load_sqlite_vt.py`, 1.6 s). Only its size and diff counts were
measured; no criterion was re-run on it.

The prediction registered for Probe A, 0.7 that the layered answer holds (a
store below carries the bulk and kos takes only promoted facts), is
consistent with both results. Neither store's bulk travelled with a harvest.
The measurement does not separate which store should sit below kos.

## Limits

- **One repository, two commits, one host.** The scale is about 48k facts per
  commit; the fleet-wide order (10^5 to 10^6, INFERRED in fact-count.md) was
  not loaded.
- **The briefs.** Both still read "Status: DRAFT ... Lock it before the probe
  runs". They were treated as locked by their merge in kos#139 (41fccf5), as
  director and the arcaven supervisor stated on 2026-10-21.
- **Build cost is an agent session's.** The 86 seconds is one agent session's,
  and that session had read the briefs and fact-count.md before the clock
  started. The clock began before any probe file was written, and the extractor
  reuses the published `scipdec.py`. It is not a human estimate.
- **Probe A's timebox.** Probe A ran second, inside minutes of its one-day
  timebox, and reused Probe B's extractor and traversal.
- **The 10-16 build was not fully re-checked.** The 10-16 input's build
  conditions are those recorded on 10-16; its counts were re-checked here, its
  run conditions were not.
- **The hot-seed run is outside the brief.** It is an extra and changes no
  verdict.
- **What criterion 4 does not test.** Point lookups meet the brief's words,
  but the node's hypothesis is an in-memory index rebuilt on load, which was
  not measured; at about 47k facts it would fit trivially. What separates the
  stores, and what really tests the falsifier, is growth with history: many
  commits, not two. That is the next probe, for both layouts (snapshot or
  valid-time SQLite, and Dolt with first-seen tx).
- **The SQLite version split.** The loaders ran on Python 3.9 (SQLite 3.51.0);
  the harvest and round trip ran on Python 3.14 (SQLite 3.53.4).

## Evidence

For the kos PR, the evidence goes under
`_kos/probes/code-graph-measurements-2026-10-21/`, the same pattern as the
2026-10-20 directory. The research holding area copy is not durable after a
reclone. It contains:
- `bin/`: the scripts, the sandbox profile `run.sb`, and the run wrappers
  `runc.sh` and `dq.sh`. After the runs, the wrappers and the profile had the
  scratch path replaced by a `PROBE_DIR` variable; the controls were re-run
  through the edited wrapper with the same results;
- `trav/`: the Rust traversal crate and its lockfile; `trav2/`: the batched IN
  traversal written for the blind run;
- `blind/`: the blind design's schema and its 100 harvest keys;
- `results/`: the seeds, both harvest YAML files, the 100-fact pick, and the
  per-seed timings with result sets left out, including the blind and crossed
  runs.

The large outputs stay in kinu scratch: the fact files (7.0 and 7.7 MB), the
exports, the stores and the full traversal results. Their sha256 prefixes:
- SQLite export: 45d9191711b99df5
- Dolt export: 1dc6c01029b45436
- harvest YAML, SQLite: 2ea54bf1c061b645
- harvest YAML, Dolt: 2df23dc6f3615307
