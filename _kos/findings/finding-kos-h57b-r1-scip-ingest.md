# Finding: probe R1, SCIP facts in the kos fact store (aae-orc-6fy1)

Status: draft, revision 2. The MDE section waits for its run (brief note 5).
Date: 2026-11-16
Host: mokuzai (Apple silicon, macOS), by the director's host ruling
bd: aae-orc-6fy1
Question node: `question-code-graph-correspondence`, sub-question B, the
decision-forcing half that finding-kos-ndz7 left open
Brief: `_kos/probes/brief-r1-scip-ingest.md`, locked at kos#185, with dated
notes 1 to 5
Follows: finding-kos-wre7 (live LSP), finding-kos-ndz7 (indexing cost),
finding-kos-8bmj and finding-kos-qttt (the store probes)
Runner: a kos builder seat (arcaven team). Author: the arcaven architect seat,
which wrote the brief.
Evidence: `_kos/probes/code-graph-measurements-2026-11-16/` (the run report,
sha256 872b572e, and its per-commit tables)

## Headline

Compiler facts in the kos fact store answer the cross-repo question that live
LSP could not answer at all, and every row they return carries its
provenance. W7 is
served in one query, with document-level recall and precision of 1.000 in both
repos. The store tracks change exactly: at all 50 versions it equals the fact
files, and every consecutive difference equals the set difference.

The history and "why" questions come back partially served. The code side of
each is right: the 77 facts added and 213 removed at 829c392, and the interval
in which `serde_yaml` was a dependency of kos, ending at that commit. But no
kos record names the decision. That is the outcome the brief predicted, and
the gap it names is in what kos records, not in the code graph: nothing links
a commit to the decision behind it.

By the brief's rule this recommends a production ingest ticket: C0 to C2 pass,
W7 is served, and both W6 and W8 are at least partially served. The probe
recommends; the operator rules.

## Run

Pins (note 1):
- the store at kos 071f1252, whose `src/facts` is xxe3n PR 1's (5d87f18f);
- kos at fe1702a8 and marvel at 5eb450df, the main heads when the brief
  merged, each with its 50 first-parent commits.

Indexers: rust-analyzer 1.96.0 on the `stable` toolchain with rust-src, and
scip-go v0.2.7. Both were installed under a pinned fetch by director's ruling,
which note 1 records as a deviation. The `scip` crate and its dependency
closure were fetched into the probe's own `CARGO_HOME` (note 3, note 4).

The run conditions are those of the brief: `env -i`, a scratch HOME and
TMPDIR, Go offline, and the Probe C sandbox profile (sha256 a50acb5e) with its
five controls passing before each step. The store file is c0207ac0 and the
ingest binary af030637. Full shas are in the run report.

## Silent degradation (finding-kos-ndz7)

The check works, and it has a working negative control.

- **The first control did not reproduce the trap.** With rust-src present,
  `RUST_SRC_PATH` pointed at an empty directory changed nothing: the index was
  byte-identical to the positive control. The run stopped, as the brief
  requires, and note 2 chose a replacement in writing.
- **The replacement reproduces finding-kos-ndz7's mechanism.** It runs
  rust-analyzer against an installed toolchain that has no rust-src. The
  indexer logged "can't load standard library", exited 0, and wrote an index
  with 0 std symbols (the positive control had 26). The ingest refused it on
  two grounds and loaded nothing.
- **Go control.** One source file removed from the clone: scip-go exited 0,
  and the ingest refused the index for the missing document.

One thing seen and not made a refusal (note 2): every index logs an offline
`cargo metadata` warning for std's own dependencies. kos's references into std
still resolve, so the run report only counts it.

## Results

| criterion | result | number |
|---|---|---|
| C0 coverage | PASS | kos 50 of 50, marvel 50 of 50 |
| C1 freshness | PASS | 50 of 50 versions equal the fact files; consecutive diffs equal the set differences; load p95 0.04 s, max 1.56 s |
| C2 provenance | PASS | every returned row resolves to a complete load entity: W6 290 of 290, W7 18 of 18, both W8 ends |
| W7 one query, two repos | served | recall and precision 1.000: kos 1 of 1 file, marvel 14 of 14 files |
| W6 a past change | partially served | changes at 829c392 correct (77 added, 213 removed); no decision record |
| W8 why | partially served | interval correct (live at version 1, not from version 4), provenance at both ends; no decision record |

C1 was checked with a mutant: one changed line at version 30 turned versions
30 and 31 red. C2 over W6's rows was checked after the scored loads, by a
binary that differs from the scored one (af030637) only by that added
read-only check (7ea39083); the store was unchanged by it.

C2's "no code fact carries a tier" holds by construction: the store has no
tier field. That says the store cannot mislabel a compiler fact as bedrock. It
is not evidence that anything was checked.

## W6 and W8: what is missing

The kos records the ingest found are real, but none is the record asked for.

- **W6.** No kos fact names 829c392 at version 4. One names it at version 50:
  `question-kos-fact-store-substrate`, which uses 829c392 as a measurement
  commit, not as the decision. No kos fact names #135.
- **W8.** Three findings name `serde_yaml`: finding-029, finding-030 and
  finding-kos-wre7. Each names it as the library in use, and none records the
  decision to replace it. No kos fact names `serde-saphyr`.

The decision was recorded, but outside the graph: in #135's body, and in the
`Cargo.toml` comment that 829c392 removed. kos has no link from a commit or a
PR to a decision node. That is a knowledge-side gap, and closing it would turn
W6 and W8 from partial to served without any change to the code graph.

## What the run changed from the brief, and why

Each change is a dated note in the brief, written before the step it governs:

- **Note 2:** the negative control was replaced, as described above.
- **Note 3:**
  - the repo's own version inside symbols becomes `.`, as `facts.py` does, so
    that a fact's identity does not move with the commit;
  - both repos share one store, so W7 is one query;
  - provenance is reached through each row's load version.
- **Note 4:**
  - a version pattern is given for each repo;
  - C1 compares exact sets with the repo prefix kept;
  - W7 is scored per file, because a reference fact has no line.
- **Note 5:** the crate-name list is fixed for the run, and the MDE tests run
  at kos#192.

Two filters follow `facts.py`, and one departs from it:
- Documents outside the repo root are skipped, as `facts.py` skips them. In
  marvel these were generated test mains in a Go build cache beside the clone.
  They were 67 to 68 documents and 6,516 to 6,801 facts per commit (332,733
  over the 50 commits, counted as the distinct pairs those documents would
  have produced), and they made about 1,000 spurious changes per commit
  before the filter. kos had none.
- Local symbols are skipped in occurrences, as in `facts.py`.
- The ingest also skips local symbols in implementation relationships, where
  `facts.py` does not. It dropped 0 facts at every commit in both repos, so
  the difference changed nothing here.

Crate names in kos facts come from one list for the whole window: 18 names for
kos and 29 for marvel. A load that used each commit's own manifest dropped the
`serde_yaml` mentions once the dependency left. The report gives both loads.
The fixed list has a known bias: a name that is a dependency only at another
commit in the window still matches.

## MDE killers (kos roadmap v3, M4)

Pending. These run at kos 73a5568, from a separate checkout, against a copy of
the scored store (note 5), once note 5 is approved.

## Prediction against result

| registered | result |
|---|---|
| 0.9 the negative Rust control reproduces the trap and the ingest refuses it | held, on the second method (note 2) |
| 0.8 W7 served | held |
| 0.6 W6 partially served, no record naming 829c392 or #135 | held. One node names 829c392 as a measurement commit, not as a decision. |
| 0.5 W8 partially served | held |
| 0.7 C0 to C2 all pass | held |

## For a production ingest

These are recommendations, not decisions:
- use the `scip` crate, as this run did;
- keep the Go build cache outside the indexed tree, as well as filtering out
  documents outside the repo root;
- define a crate-name lexicon instead of a window union;
- keep the silent-degradation refusal, with a negative control in its tests.

Ranking is still untested: it has no host since forestage was retired. So is
sub-question D of question-kos-fact-store-substrate.
