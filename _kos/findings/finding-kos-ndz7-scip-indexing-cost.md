# Finding: SCIP indexing cost across the fleet (aae-orc-msqx, Q1 + Q5)

Status: final
Date: 2026-08-16
Host: kinu (Apple M3 Max, 16 cores, 128 GB RAM, macOS 26.5.2)
bd: aae-orc-msqx
Question node: `question-code-graph-correspondence`, sub-question B (is compiler-grade indexing affordable at our cadence?)
Brief: `_kos/probes/brief-scip-cost.md`
Indexers: rust-analyzer 1.97.1 (8bab26f4 2026-07-14), scip-go 0.2.7 (go1.26.5)

## Scope of this finding: the cost half, not the decision-forcing half

This finding measures what a compiler-grade SCIP index COSTS to build, in wall
time and peak memory, at each cadence. That is Q1 and Q5, and it is the cost
half of sub-question B. It does NOT answer the decision-forcing half: whether
the raw SCIP facts, once built, can ADDRESS the workloads that live LSP failed
in the sibling probe (finding-aae-orc-5lbu): W6 (prior-session history), W7
(one query across the whole fleet), and W8 (typed, ranked, provenanced answers
to "why"). Cheap-to-produce and answers-the-question are different claims. This
finding settles the first. The second is a separate probe, still open, and it
is the one that decides whether an ingest is worth building rather than merely
affordable to build. Read the affordability verdict below with that boundary in
view: I measured that the index is cheap for Go and memory-bound for Rust, not
that its contents close the gap 5lbu found.

## Headline

Indexing is cheap in wall time and expensive in memory, and the two
languages are not in the same cost class. A warm reindex of every live
workspace in the fleet takes 44.31 seconds and produces 60.83 MB of
index. The same sweep from cold takes 294.24 seconds.

Go is near-free: every Go workspace indexes warm in under one second,
including the 159,000-line one, at under 130 MB peak RSS. Rust is not.
Every Rust run costs between 0.55 and 2.2 GB of peak RSS regardless of
how small the crate is, and warm times run 1.72 to 8.46 seconds.

The hypothesis under test survives for Go and does not survive
unconditionally for Rust. Detail in the ruling section.

The most consequential result is not a timing number. It is that
rust-analyzer indexed one workspace without the standard library,
exited 0, and wrote a plausible-looking index. Details under
"Silent degradation".

## Premise checks (executed first)

The ticket's stated premise was wrong, in a way worth recording.

| Check | Ticket expected | Actual |
|---|---|---|
| `~/.cargo/bin/rust-analyzer` | cargo-installed binary | symlink to `rustup`; a proxy, not a binary |
| `rust-analyzer --version` | a version string | `error: infinite recursion detected` |
| rust-analyzer component | unknown | not installed; available |
| `scip` subcommand | unverified | present after install, with `--output` |
| scip-go install path | `github.com/sourcegraph/scip-go` | moved; that path no longer resolves |

The recursion is a three-hop loop: the `rust-analyzer` on PATH is a
mise shim, which delegates to `~/.cargo/bin/rust-analyzer`, which is a
rustup proxy, which finds no rust-analyzer component for the active
toolchain and falls back to the mise shim. Nothing was broken; the
component had simply never been added.

Both blockers were cleared:

- `rustup component add rust-analyzer` on toolchain 1.97.1. The `scip`
  subcommand exists and accepts `--output`, `--config-path`,
  `--exclude-vendored-libraries`, `--num-threads`.
- scip-go now lives at `github.com/scip-code/scip-go`. Installing from
  the old path fails, because v0.2.7 declares the new module name. It
  installs cleanly from the new path.

I invoked rust-analyzer by absolute toolchain path in every
measurement, never through the shim.

## The numbers

Sequential runs, one workspace at a time, `/usr/bin/time -l`. Warm is
the second consecutive run. All numbers below were executed, not
estimated.

### Rust (rust-analyzer scip)

| Repo | LOC | Cold s | Warm s | Peak RSS MB | Index MB |
|---|---:|---:|---:|---:|---:|
| akey | 40,214 | 28.29 | 8.46 | 1,776 | 7.67 |
| sidestep | 26,005 | 27.60 | 6.49 | 1,766 | 7.61 |
| stave | 17,750 | 12.11 | 4.66 | 1,596 | 2.20 |
| bloomctl | 14,657 | 8.02 | 5.43 | 1,673 | 3.57 |
| forestage (retired) | 8,484 | 14.68 | 5.18 | 2,161 | 1.66 |
| kos | 6,856 | 3.34 | 3.49 | 1,295 | 1.27 |
| BetterDials | 4,338 | 4.63 | 3.96 | 1,243 | 0.91 |
| beadle | 3,861 | 141.77 | 2.46 | 824 | 0.33 |
| tmux-cmc | 1,385 | 4.54 | 2.42 | 673 | 0.28 |
| flyloft | 610 | 8.25 | 2.60 | 941 | 0.14 |
| curtain | 3 | 4.60 | 1.72 | 556 | 0.00 |

### Go (scip-go index)

| Repo | LOC | Cold s | Warm s | Peak RSS MB | Index MB |
|---|---:|---:|---:|---:|---:|
| ThreeDoors | 159,229 | 28.99 | 0.95 | 679 | 27.90 |
| marvel | 31,243 | 0.47 | 0.45 | 122 | 5.86 |
| sideshow | 18,533 | 0.77 | 0.33 | 110 | 2.97 |
| ai | 330 | 15.03 | 0.35 | 745 | 0.05 |
| ourbot | 237 | 1.64 | 0.31 | 156 | 0.04 |
| switchboard | 96 | 4.19 | 0.23 | 222 | 0.02 |

### Totals

Live workspaces only, excluding retired forestage.

| Set | Cold | Warm | Index |
|---|---:|---:|---:|
| Rust (10 repos) | 243.15 s | 41.69 s | 23.99 MB |
| Go (6 repos) | 51.09 s | 2.62 s | 36.84 MB |
| **Fleet (16 repos)** | **294.24 s** | **44.31 s** | **60.83 MB** |

Extremes: slowest cold is beadle at 141.77 s, slowest warm is akey at
8.46 s, fastest warm is switchboard at 0.23 s. Highest peak RSS is
forestage at 2.16 GB.

## Reading the numbers

**LOC does not predict cost, in either direction.** beadle at 3,861
lines cost 141.77 s cold, five times akey's 28.29 s at ten times the
size. ThreeDoors at 159,229 lines indexes warm in 0.95 s while
BetterDials at 4,338 lines takes 3.96 s in Rust. What cold cost tracks
is dependency graph population, not source volume, and the two are
unrelated. Anyone budgeting from line counts will be wrong per repo.

**The Rust floor is high and flat.** curtain is three lines of Rust in
one file. It costs 1.72 s and 556 MB. That is the price of starting
rust-analyzer, loading a sysroot, and running cargo metadata, and no
repo pays less. Roughly the first 1.7 seconds and half a gigabyte of
every Rust index is fixed overhead. For the seven Rust repos under
15,000 lines, fixed overhead is the majority of the cost.

**Memory, not time, is the binding constraint for Rust.** The ten live
Rust workspaces sum to 41.69 s warm, which looks trivially
parallelizable until the RSS column is read: running them concurrently
would demand roughly 12 GB of resident memory. This machine has 128 GB
and would absorb it. A 16 GB laptop or a standard CI runner would not.
The sequential 41.69 s is therefore the honest fleet number, not a
figure to divide by core count.

**Go's cold numbers are dependency fetching, not indexing.** `ai` is
330 lines and cost 15.03 s cold at 745 MB, higher peak RSS than
ThreeDoors' entire 159,000-line index. That is module download and
compilation on a cold build cache. Its warm run is 0.35 s at 43 MB.
Cold Go cost measures the state of the build cache and nothing about
the code.

## Silent degradation (the load-bearing caveat)

While measuring beadle I noticed a 141.77 s cold run followed by a
0.77 s warm run with *lower* peak RSS than the three-line crate. The
logs explained it:

```
ERROR can't load standard library, try installing `rust-src`
  sysroot_path=/Users/michael.pursifull/.rustup/toolchains/stable-aarch64-apple-darwin
```

rust-analyzer indexed beadle without resolving the Rust standard
library. It exited 0. It wrote a 346,690-byte index that looks
structurally fine. Nothing in the exit status or the output file
signals that every reference into std is unresolved.

After `rustup component add rust-src --toolchain stable`, beadle's warm
run moved from 0.77 s to 2.46 s and from 297 MB to 864 MB. The
three-fold increase in both is the measure of how much work was being
skipped. The 0.77 s figure was not a fast index; it was a partial one.

beadle pins `channel = "stable"` in `rust-toolchain.toml`, and rust-src
was installed on 1.97.1 but not on stable. kos and tmux-cmc pin stable
too and did not emit the error, so I am not confident of the full
mechanism and am not going to invent one. The reproducible facts are
the error, the toolchain it named, and the before/after cost change.

Two operational consequences:

1. **A zero exit code does not mean a complete index.** Any pipeline
   running rust-analyzer scip must grep its log for `can't load
   standard library` and fail on it. Exit status will not catch this.
2. **rust-src is a prerequisite on every pinned toolchain**, not just
   the default one.

I re-ran curtain, flyloft, and beadle after the fix; their post-fix
warm numbers are the ones in the table. For tmux-cmc I confirmed the
index was byte-identical before and after, so rust-src changed
resolution fidelity without changing output size or cost there.

## Other index-quality observations

These bear on the word "compiler-grade" and are worth recording even
though they did not change any timing.

- **rust-analyzer reports its own bug on 8 of 11 Rust workspaces.** The
  message is "Encountered duplicate scip symbols, indicating an
  internal rust-analyzer bug ... information about these symbols
  presented by downstream tools may be incorrect." It appears for akey,
  beadle, bloomctl, forestage, kos, sidestep, stave, tmux-cmc. Not for
  BetterDials, curtain, flyloft.
- **Unnamed definitions are common.** "Encountered enclosing definition
  with no name" appears 539 times in akey, 17 in forestage, 9 in
  tmux-cmc, 7 in BetterDials, and a handful elsewhere.
- **scip-go output is not byte-reproducible.** Two consecutive warm
  ThreeDoors runs produced identical sizes (29,259,655 bytes) but
  differing bytes. The cold run produced 28,796,028 bytes, 1.6 percent
  smaller, while both runs reported identical coverage of 77/77
  packages with no errors. This matters only if someone plans to
  content-address or diff indexes; it is not a correctness failure I
  can demonstrate.

No workspace failed to index. All 34 runs exited 0.

## Q1 verdict: is per-commit indexing affordable?

**For Go, yes, without qualification.** Every Go workspace indexes warm
in 0.23 to 0.95 seconds at under 130 MB (excluding cold-cache
artifacts). That is below the threshold where a human notices a commit
hook. Per-commit Go indexing is affordable today.

**For Rust, per-commit is affordable only per-repo and only warm, and
it is not the cadence I would set fleet-wide.** Warm cost is 1.72 to
8.46 seconds for a single workspace. Against the brief's criterion (a
few seconds, in a hook a human waits on) the small and mid repos pass
and the two largest do not: akey at 8.46 s and sidestep at 6.49 s are
past the point where a commit hook becomes an interruption. The memory
profile reinforces this, since a 1.8 GB indexer firing during an active
cargo build competes for exactly the resource that build needs.

**Fleet-wide per-commit is not affordable and is also not the right
shape.** A commit touches one repo, so the 44.31 s fleet number is a
sweep cost, not a commit cost. It belongs to the nightly or on-demand
rungs.

## Q5 verdict: what cadence keeps staleness bounded?

Split by language, because one answer does not fit both.

| Scope | Cadence | Measured basis | Freshness honestly claimable |
|---|---|---|---|
| Go, per repo | **per-commit** | 0.23 to 0.95 s warm | the index describes the tree you are looking at |
| Rust, small and mid (under ~15k LOC) | **per-commit acceptable, per-push safer** | 1.72 to 5.43 s warm | as above, at the cost of a noticeable pause |
| Rust, large (akey, sidestep) | **per-push** | 6.49 to 8.46 s warm | describes what is shared; local work in progress is invisible |
| Whole fleet sweep | **nightly or on-demand** | 44.31 s warm, 294.24 s cold | describes the last sweep, and callers must be told when it ran |

The recommendation I would defend: **per-push as the fleet default,
with per-commit available per-repo for Go and for whoever wants it on a
small Rust crate.** Per-push costs at most 8.46 seconds on the worst
workspace, runs off the interactive path, and its freshness story is
honest and easy to state. Per-commit buys a marginal freshness
improvement for the two largest Rust repos at a cost users will feel on
every commit.

### The CI caveat that changes the arithmetic

Everything above prices the warm column. A CI runner is cold by
construction. Cold fleet cost is 294.24 seconds, and beadle alone was
141.77 seconds cold.

So "per-push in CI" does not cost the per-push numbers unless the
`target/` directory and Go build cache are persisted between runs.
Without caching, per-push indexing on a Rust repo can cost minutes, and
the cadence collapses to nightly on economics alone. **Whether the
cadence ladder is affordable is decided by cache persistence, not by
indexer speed.** That is the single most actionable consequence of
these numbers, and I would settle the caching question before
committing to any rung.

## Ruling on the asserted hypothesis

The briefing asserted: "compiler-grade indexes are near-free for this
stack."

**Confirmed for Go. Confirmed with conditions for Rust, and the
conditions are load-bearing enough that I would not repeat the phrase
without them.**

What holds it up:

- Go is near-free by any reading: 2.62 s and 130 MB peak to index six
  workspaces including a 159,000-line one.
- Warm Rust is cheap in time: 41.69 s for ten workspaces, 4.2 s
  average.
- Total artifact size is small: 60.83 MB for the live fleet.

What the phrase hides:

- **Near-free only when warm.** Cold costs 5.8x more for Rust and 19x
  more for Go. Cold is the CI default.
- **Not free in memory for Rust.** 0.55 to 2.16 GB per run, unrelated
  to repo size. This is the constraint that decides whether indexing
  can run concurrently or alongside a build, and time-only framing
  hides it entirely.
- **Not free at the floor.** 1.72 s and 556 MB to index three lines. On
  a fleet of many small crates, per-repo overhead dominates.
- **"Compiler-grade" needs an asterisk.** 8 of 11 Rust workspaces emit
  rust-analyzer's own duplicate-symbol bug warning, akey emits 539
  unnamed-definition errors, and one workspace silently indexed without
  the standard library while exiting 0.

The honest restatement: *warm indexing is cheap in time and moderate in
memory for Rust, and genuinely near-free for Go; cold indexing is not
near-free for either, and index completeness needs verifying rather
than assuming.*

## Blockers and anomalies

1. `rust-analyzer` unusable as installed (infinite shim recursion,
   component missing). Resolved by `rustup component add
   rust-analyzer`.
2. scip-go module path relocated to `github.com/scip-code/scip-go`.
   The old path fails with a module-declaration conflict.
3. rust-src absent on the stable toolchain, producing a silently
   degraded index with a zero exit code. Resolved by `rustup component
   add rust-src --toolchain stable`. This is the finding I would act on
   first.
4. rust-analyzer duplicate-symbol bug on 8 of 11 Rust workspaces.
   Upstream issue, not actionable here, but it constrains what
   downstream tools can trust.
5. scip-go output not byte-reproducible across runs.
6. No workspace failed to index; all 34 runs exited 0.

## Measurement hygiene

- No file was written into any repo working tree. Both indexers were
  given `--output` paths in the scratchpad, so no `index.scip` was ever
  created inside the fleet. Verified: `find` for `*.scip` across the
  fleet returns nothing.
- `git status --short` after the sweep is clean for 15 of 17 repos. Two
  carry pre-existing modifications that are not mine and that I did not
  touch: stave has a modified `_bmad-output/.../.memlog.md`, marvel has
  untracked `.DS_Store` files. kos briefly showed an untracked
  `.serena/` belonging to the concurrent serena-baseline agent; it was
  gone by the final sweep.
- Compilation artifacts under gitignored `target/` directories were
  written by cold runs, as expected and permitted.

### Load contamination

The concurrent serena-baseline agent ran throughout. Load average went
from 7.07 at T0 (15:30) to 19.50 at T1 (15:41). No rust-analyzer or
gopls process was resident at T0.

The bias is one-directional: **wall time on a loaded machine overstates
cost**, which biases against the near-free hypothesis. Since the
verdict confirms affordability, the contamination does not threaten it;
a cleaner machine would only make these numbers better. Peak RSS is
substantially less sensitive to competing load than wall time, so the
memory conclusions (which carry the Rust argument) are the more solid
half of this finding.

Numbers I would rerun solo before relying on them, because they are
borderline against the per-commit threshold and were taken during the
rising-load window: **akey warm (8.46 s) and sidestep warm (6.49 s)**.
Both sit right at the boundary between per-commit and per-push, and
that boundary is the decision this finding drives.

## Suggested follow-ups

1. Add rust-src to every pinned toolchain and add a log grep for
   `can't load standard library` to any indexing pipeline. Cheapest and
   highest-value item here.
2. Settle the CI cache-persistence question before choosing a rung; it
   dominates indexer speed.
3. Rerun akey and sidestep warm on an idle machine to firm up the
   per-commit boundary.
4. Neither indexer is incremental with respect to a diff; both reindex
   the whole workspace. If per-commit Rust indexing is wanted later,
   incrementality is the lever, not faster hardware.
