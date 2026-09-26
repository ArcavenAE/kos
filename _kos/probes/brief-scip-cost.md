# Brief: SCIP indexing cost across the fleet (aae-orc-msqx, Q1 + Q5)

Status: locked (written before measurement; probe now complete)
Date: 2026-08-16
Host: kinu (Apple M3 Max, 16 cores, 128 GB RAM, macOS 26.5.2)

## Question

Q1: what does it cost, in wall time and peak memory, to build a
compiler-grade SCIP index for each workspace in the fleet?

Q5: at what cadence can indexing run (per-commit, per-push, on-demand)
so that the index a reader consults is not meaningfully stale?

The briefing that motivated the ticket asserted a hypothesis I am here
to test rather than assume: "compiler-grade indexes are near-free for
this stack." The measurement either confirms it, refutes it, or
confirms it with conditions.

## Decision criterion

The numbers are read against one question: is per-commit indexing
affordable? If it is not, the ladder below names the next rung down,
and the finding states which rung the measured cost actually buys.

## Premise checks (step 1, executed before anything else)

The ticket flagged that `rust-analyzer` here is a cargo-installed
binary rather than a rustup component, leaving scip emission
unverified. That premise is wrong in an interesting way, and the
correction is a result in itself.

| Check | Expected per ticket | Actual |
|---|---|---|
| `~/.cargo/bin/rust-analyzer` | cargo-installed binary | symlink to `rustup`; a rustup proxy shim, not a real binary |
| `rust-analyzer --version` | prints a version | `error: infinite recursion detected` |
| rust-analyzer component | unknown | was NOT installed; available and installable |
| scip subcommand | unverified | present after install, with `--output` |
| scip-go | installable from `github.com/sourcegraph/scip-go` | module moved; that path fails to resolve |

Detail on the recursion, because it will bite the next person: the
`rust-analyzer` on PATH is a mise shim, which delegates to
`~/.cargo/bin/rust-analyzer`, which is a rustup proxy, which finds no
`rust-analyzer` component for the active toolchain and falls back to
the mise shim. The loop terminates only because rustup counts hops.
Nothing was broken about the install; the component simply had never
been added.

Resolutions applied:

- `rustup component add rust-analyzer` on the active toolchain
  (1.97.1). Yields `rust-analyzer 1.97.1 (8bab26f4 2026-07-14)` at
  `~/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rust-analyzer`.
  The `scip` subcommand is present, taking a path argument plus
  `--output`, `--config-path`, `--exclude-vendored-libraries`, and
  `--num-threads`.
- scip-go's module path is now `github.com/scip-code/scip-go`. The old
  `sourcegraph` path fails with a module-declaration conflict, since
  v0.2.7 declares itself under the new name. Installed from the new
  path: scip-go 0.2.7.

I invoke rust-analyzer by its absolute toolchain path in every
measurement, not through the shim, so the numbers describe the indexer
rather than the shim chain.

Versions of record:

- rust-analyzer 1.97.1 (8bab26f4 2026-07-14), rustup component
- scip-go 0.2.7, built with go1.26.5 darwin/arm64
- cloc 2.x from Homebrew (LOC counts)
- rustc toolchains present: 1.95.0, 1.96.0, 1.97.1 (active), stable,
  nightly

## Workspace inventory (step 2)

The ticket notes the scope list exists nowhere. I built it from
`repos.yaml` intersected with what is cloned and what actually carries
a `Cargo.toml` or `go.mod` at its root.

`repos.yaml` language labels are unreliable and I did not trust them.
kos and beadle are both labeled `markdown` there but are Rust
codebases; the disk decides.

### In scope, Rust (11 roots)

| Repo | Rust LOC | Files |
|---|---|---|
| akey | 40,214 | 109 |
| sidestep | 26,005 | 25 |
| stave | 17,750 | 43 |
| bloomctl | 14,657 | 26 |
| forestage | 8,484 | 28 |
| kos | 6,856 | 22 |
| BetterDials | 4,338 | 6 |
| beadle | 3,861 | 15 |
| tmux-cmc | 1,385 | 14 |
| flyloft | 610 | 9 |
| curtain | 3 | 1 |

### In scope, Go (6 roots)

| Repo | Go LOC | Files |
|---|---|---|
| ThreeDoors | 159,229 | 735 |
| marvel | 31,243 | 139 |
| sideshow | 18,533 | 96 |
| ai | 330 | 2 |
| ourbot | 237 | 2 |
| switchboard | 96 | 2 |

LOC counts are cloc `code` lines for the named language only, with
`target/` and `vendor/` excluded. They count what the fleet wrote, not
what it depends on; an indexer also walks dependencies, so LOC predicts
index cost only loosely. I expect the generated-client repos (sidestep,
bloomctl, stave) to show high LOC in few files and the dependency-heavy
repos to cost more than their LOC suggests.

### Measured but held apart

- **forestage** is retired as a live target (vision.md; the deep
  adapter is frozen as reference). I measure it because it is cloned
  and was named in the briefing, but I exclude it from the fleet
  recurring-cost total, which should describe live workspaces.
- **curtain** is design-only (3 LOC, one file). Kept deliberately as
  the floor case: it prices the fixed overhead of invoking the indexer
  at all, which is the number that decides whether per-commit indexing
  is viable for small repos.
- **switchboard** at 96 LOC is the canon skeleton, not the proven
  -blue prototype. The prototype lives under a tier-3 path the fleet
  recipes must not read.

### Excluded, with reasons

- `run/`, `forks/`, `contrib/`: tier-2 and tier-3 trees that fleet
  recipes must not read (CLAUDE.md). Not fleet workspaces.
- Content and markdown repos (spectacle, callbook, usher, bmad-extras,
  bmad-extras-private, multiclaude-enhancements, betterdials-site,
  homebrew-tap, sideshow-packs): no compiled source, no indexer.
- `ftc`: Python; neither indexer under test covers it.
- `critic`: pre-code, language undecided.
- `director`: not initialized as a git repo.
- `people-profile`: present on disk, absent from `repos.yaml`,
  markdown only.
- Nothing was cloned for this probe.

## Measurement protocol (step 3)

One repo at a time, strictly sequential, never parallel, so that peak
RSS is attributable to a single indexer run.

- Wrapper: `/usr/bin/time -l`, which on macOS reports wall time and
  maximum resident set size.
- Rust: `rust-analyzer scip . --output <scratchpad>/<repo>.scip`, run
  from the repo root, using the absolute toolchain binary.
- Go: `scip-go index --output <scratchpad>/<repo>.scip`, run from the
  module root.
- Every index is written into the scratchpad by the indexer's own
  `--output` flag, so no `index.scip` is ever created inside a repo. I
  verify `git status --short` in each repo after its runs.
- Two runs per repo: cold, then warm, back to back.
- Index size recorded per repo.
- Each measurement timestamped.

### What cold and warm actually mean here

Worth stating precisely, because the naive reading ("cold compiles,
warm reuses") is only half right and the distinction drives the
cadence answer.

For Rust, a first run may compile build scripts and proc macros into
`target/`, and populates the OS page cache. A second run reuses both.
What it does not reuse is rust-analyzer's own analysis: the `scip`
command does not persist a cross-run analysis cache. So the warm number
is the floor for repeated indexing, not a near-zero incremental cost,
and warm is the number the cadence question must be priced against.

For Go, the equivalent warm benefit is the build cache plus page cache.

Neither indexer is incremental with respect to a diff. Both reindex the
whole workspace. This matters more than any single measurement: if
per-commit indexing is affordable, it is affordable at full-reindex
cost, and the warm column is the honest recurring price.

### Load caveat

A second agent (serena-baseline) is driving live rust-analyzer LSP
sessions on this machine during my window. At T0 the load average was
7.07 with no LSP process yet resident.

The bias direction is one-way and I state it explicitly: wall time on a
loaded machine overstates cost. That biases against the near-free
hypothesis, so a confirmation survives the contamination while a
refutation would need a solo rerun before anyone relies on it. Peak RSS
is far less sensitive to competing load than wall time. I snapshot load
before the first run and after the last, and mark any borderline number
for solo rerun.

## The Q5 cadence ladder

The measured warm cost is read against these rungs. Each rung buys a
different freshness story, and the honest version of each is written
out so the finding can name one rather than gesture at a range.

| Rung | Trigger | Staleness the reader faces | Honest claim |
|---|---|---|---|
| Per-commit | every commit, blocking or via hook | zero to one commit | the index describes the tree you are looking at |
| Per-push | pre-push hook or CI on push | zero to one branch's worth of local commits | the index describes what is shared; local WIP is invisible |
| Nightly | scheduled | up to 24 hours, and unbounded across a quiet weekend | the index describes yesterday; anything renamed today misleads |
| On-demand | explicit invocation | whatever the reader accepts, and they know it | no staleness claim at all, which is the one honest story a stale index can tell |

The failure mode that decides between rungs is not slowness, it is a
confidently wrong answer. A stale symbol index does not fail loudly; it
returns a definition that has since moved and looks exactly like a
fresh one. That asymmetry is why the ladder is scored by what each rung
can honestly claim rather than by cost alone.

Cost thresholds I will apply when reading the numbers:

- Per-commit is viable if the warm run for a typical repo stays inside
  a few seconds, since it sits in a hook a human waits on.
- Per-push tolerates tens of seconds.
- Anything reaching minutes for a routine workspace pushes the fleet to
  nightly or on-demand for that workspace, and the answer may be
  per-repo rather than fleet-wide.

## Deliverables

- This brief.
- `finding-aae-orc-msqx-scip-indexing-cost.md`: the numbers table, the Q1 verdict
  against the decision criterion, the Q5 cadence answer, an explicit
  ruling on the near-free hypothesis, and any repo that failed to index
  with its error verbatim.
