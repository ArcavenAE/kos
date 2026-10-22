# Brief: fact store Probe C, growth with history (aae-orc-isie)

Status: DRAFT, written before any measurement. Lock it before the probe runs:
it is locked when its kos PR merges after review, and nothing below changes
after that without a dated note.
Date: 2026-10-21
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
  here to put a number on the snapshot layout's cost, not as a candidate.
- **D, Dolt.** One Dolt commit per marvel commit. Primary key (e, a, v), one
  secondary index (a, v, e), `tx` as the first-seen marvel commit, as Probe A
  ran it.

Every arm traverses in Rust by indexed point lookups, one per frontier element
(the Probe A and B traversal), never by batched IN lists.

## Inputs

- **Commits:** the 200 first-parent commits of marvel main ending at d0616f98,
  oldest first. The list is written to a file and its sha256 recorded before
  any index is built.
- **Indexes:** one scip-go index per commit, from a scratch clone, under the
  run conditions below. A commit whose index fails to build is a GAP, recorded
  with its error line, and the next commit's diff is taken against the last
  commit that did build.
- **Extraction:** the published `scipdec.py` and `facts.py` from
  `code-graph-measurements-2026-10-21/bin/`, including the own-module version
  normalization, which is part of this brief from the start.
- **Seeds:** 100 symbols drawn with seed 20261022 from the DEFINES symbols at
  d0616f98 that have at least one tracked reference, so no seed is empty
  (finding-kos-8bmj found 34 of 100 empty seeds in Probes A and B). The hot set
  is the 100 most-referenced DEFINES symbols at d0616f98, as before.

## Decision criteria

Each criterion has a bar every arm must meet. Results are reported per arm as
PASS or FAIL with the number.

- **G0, coverage.** At least 190 of the 200 commits index and load. Below that,
  the probe reports the GAP count and no other criterion is scored.
- **G1, size.** Store bytes on disk after all commits load: under 2 GB.
- **G2, incremental load.** Loading one commit onto the existing store, from
  fact file to committed rows: p95 under 10 s, and the median of the last 20
  commits no more than 2 times the median of the first 20.
- **G3, diff at any pair.** 50 commit pairs (i, j), i < j, drawn with seed
  20261022: each pair's added and removed counts equal a set difference
  computed outside the store from the two fact files. p95 diff time under 5 s.
- **G4, historical traversal.** The three-hop traversal of Probe A criterion 4,
  run as of 20 commits drawn with seed 20261022, from the 100 seeds: every
  result set equals the plain join on that commit's fact file, and the slowest
  run finishes under 1 s. The hot set is run as of the same 20 commits and
  reported, not scored.
- **G5, history tax on the latest commit.** The criterion seeds, traversed at
  d0616f98 after all 200 commits load: slowest no more than 2 times the slowest
  of the same seeds traversed at d0616f98 in a store holding only that commit.

## What counts as separating

The stores separate on this probe when either holds:
- one arm passes a criterion that another candidate arm (S-vt or D) fails; or
- on G1, G4 or G5, one candidate arm's number is at least 3 times the other's.

If neither holds, the probe reports that history to 200 commits does not
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
- `GOPROXY=off GOFLAGS=-mod=readonly GOTOOLCHAIN=local` for every index build.
  A commit whose module set is not already in the cache fails to build, and is a
  GAP under G0, never a fetch.
- `sandbox-exec` denying the network except localhost, with the
  refused-connection control, and a sandbox write-deny proof: a blocked write
  outside scratch refused by the profile, and a write inside scratch that
  succeeds.
- The Dolt server on a random localhost port, stopped by its pid with
  `kill -TERM` and checked with `kill -0`.
- Tools already installed on the run host, versions recorded. A host without
  scip-go, Dolt or a Rust toolchain does not run this probe until the
  operator's package-fetch ruling covers the install, with pinned versions.

## Timebox

Two working days, of which indexing 200 commits is expected to take about an
hour at the 19 s per commit measured for d0616f98.

## Prediction (registered)

The research-supervisor's, registered before any run:
- 0.8 that S-vt and D both pass G0 to G5;
- 0.7 that D's store is at least 3 times smaller than S-vt's on G1, which
  would separate them on size;
- 0.6 that S-vt's G4 slowest is lower than D's, as in the two-commit runs;
- 0.9 that S-snap fails G1.

## Falsifier

If S-vt and D both fail G2 or G4 at 200 commits, the versioned-store-below
shape does not hold with history at marvel's scale, and the flat-file ceiling
argument needs restating against a different store family.

## Outputs

- A finding in kos `_kos/findings/`, with the id minted by
  `kos id finding <slug>`.
- A table of G0 to G5 per arm, PASS or FAIL with the number, and the
  separation result against the rule above.
- The MDE answers as measured, and the GAP list from G0.
- Evidence under `_kos/probes/code-graph-measurements-<run date>/`, same
  pattern as 2026-10-21.
