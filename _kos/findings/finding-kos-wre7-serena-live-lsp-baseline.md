# Finding: Serena as the measured live-LSP baseline

**bd:** aae-orc-5lbu
**Question node:** `question-code-graph-correspondence`, sub-question A (does live LSP suffice, making an ingest optional?)
**Probe brief:** `_kos/probes/brief-serena-baseline.md` (written and locked before install)
**Date:** 2026-08-16, 15:30 to 15:39 CDT
**Serena:** `oraios/serena` at commit `19f1c33`, run via
`uvx --from git+https://github.com/oraios/serena`, installed in 19 seconds
**Language servers:** rust-analyzer 1.97.1 (8bab26f4), gopls v0.20.0
**Repos:** kos (Rust, 8,681 LOC, single crate), marvel (Go, 43,325 LOC,
multi-package)

---

## Verdict

**Serena alone is insufficient**, by the criterion the ticket set. Three of the
eight named workloads it cannot serve, and the reason in each case is
structural rather than a matter of configuration or version:

- **W3 session-preamble orientation.** Not served.
- **W6 prior-session history and decisions.** Not served.
- **W7 one query across two repos.** Not served, with an explicit error.

The registered prediction was **0.65 that Serena alone proves insufficient.
The prediction is a HIT.** The evidence carrying it is W7, which fails with a
hard error rather than a judgment call: `Cannot extract symbols from file
/Users/.../kos/src/model.rs. Active language servers: ['go']`. One language
server set is bound to one active project. A fleet question spanning kos and
marvel cannot be expressed as one query.

**Honesty discipline 1 (the prediction is a HIT, and it is
CONSTRUCTION-INFLATED).** I log the hit, and I mark it. The workload list I
wrote in the brief deliberately named W6, W7, and W8 as not-served probes, with
the reasoning stated at the time: they exist to convert an architectural fact
(a language server is a snapshot index of one working tree) into named unserved
workloads. So "at least one of the eight fails" was near-certain before I
measured anything, because I built the list to contain guaranteed failures.
The 0.65 was calibrated against an instrument I designed to trip. That does not
make the hit false, and it does not make Serena's limits imaginary. It means
the number should not be read as a well-calibrated forecast that came in. I
record the hit so the scoreboard stays complete, and I annotate it inflated so
the calibration record stays honest. This is anti-HARKing applied to my own
scoring: the prediction was not wrong, but crediting it as a clean forecast
would quietly reward writing predictions against rigged lists.

That verdict should not be read as Serena performing badly. On the four pure
symbol-resolution workloads it was **correct, fast when warm, and measurably
better than the ripgrep an agent would otherwise run**. The split matters more
than the headline, and it is the substance of the recommendation below.

## Scoreboard

| ID | Workload | Repo | Result |
|---|---|---|---|
| W1 | Who implements this interface | marvel | **Served**, beat my ground truth |
| W2 | Blast radius of a symbol change | kos | **Served**, beat grep on precision |
| W3 | What belongs in the session preamble | kos | **Not served** |
| W4 | Call path across package boundaries | marvel | **Served**, with a noise caveat |
| W5 | Callers of a symbol / is it dead | kos | **Served** |
| W6 | What did the last session decide | kos | **Not served** |
| W7 | One query across two repos | both | **Not served** (hard error) |
| W8 | Why is it this way (provenance) | kos | **Partially served**, by grep, not by symbols |

Four served, one partial, three not served.

## Where Serena beat the comparator

The brief set ripgrep as the comparator, on the grounds that a language server
has to beat the grep an agent would otherwise run, not beat nothing. It did,
twice, in ways worth recording because they are the argument for the pattern.

**W1 found an implementation my own ground truth missed.** I established by
hand that `runtime.Adapter` had six implementations, using
`rg "func \(\w+ \*?\w+\) Prepare\(" internal/runtime/`. Serena returned seven:
the six production adapters plus `feedlessAdapter` in
`internal/session/projection_test.go`. I verified it; the test double genuinely
implements the interface (`Name`, `Prepare`, `ProjectionFor`). My grep missed
it because I guessed the directory, and the type system did not. This is the
failure mode grep has that no amount of care fully removes: the query is over
text, and the answer lives in the type graph.

**W2 was smaller and more precise than the grep.** Ripgrep found 51 textual
occurrences of `Confidence` across 8 files. Serena returned 42 across 6. Every
omission was correct on inspection:

- `src/init.rs` (2): both inside a markdown template string (`### Confidence Changes`)
- `src/bridge.rs` (2): both in doc comments
- `src/reflect.rs` (1): a string literal, `section("Confidence changes", ...)`
- `src/model.rs` (3): the enum definition and two `impl` headers, which are
  definitions rather than references

So grep's answer carried nine false positives into a blast-radius estimate, and
the false positives were exactly the kind that look real in a diff review.
Serena also attributed every remaining reference to its containing symbol
(`confidence_score`, `print_dot`, `load_tier`, `check_graph`), which grep cannot
produce at all and which is the actual unit of a refactor.

**W4 resolved the cross-package hop.** The one production caller of
`Adapter.Prepare` is `planLaunch` in `internal/session/manager.go`, and Serena
found it. The caveat is signal-to-noise: that single production caller arrived
alongside roughly 31 test functions across four test files, with no ranking
between them. The containing-symbol attribution is what makes the result usable,
because `Test*` reads differently from `planLaunch` at a glance. A consumer that
only got file and line would be worse off than a careful grep.

## The shape of the failure

The ticket said instrumenting the failure is the probe's real value. Five
measured properties, each stated as what it is rather than as a complaint.

### 1. Warm-up is real, and it is paid per session, per repo

| Repo | initialize | first symbol query (cold) | same query (warm) | ratio |
|---|---|---|---|---|
| kos (Rust) | 2.1 to 2.4 s | **6.28 to 7.22 s** (4 cold runs) | 0.25 to 0.30 s | ~23 to 28x |
| marvel (Go) | 2.1 to 2.5 s | **2.14 to 2.74 s** (3 cold runs) | 0.12 to 0.15 s | ~15 to 18x |

The counter-intuitive result is that the 5x larger repo is the cheaper one.
Cold latency and memory track the dependency graph a language server must
expand, not first-party line count.

### 2. The Rust index does not survive a restart; the Go one partly does

I ran each project twice in separate server processes. For kos, run 2's cold
query took 7.22 s against run 1's 6.91 s: no improvement, so rust-analyzer
rebuilt its picture from scratch. For marvel, gopls held cold latency flat
(~2.1 s) and its peak RSS dropped from 600 MB to 212 MB on the second run,
consistent with its on-disk cache doing real work.

Practical consequence: for Rust, every new session pays the full warm-up again.
This fleet's documented norm is 3 to 5 concurrent sessions.

### 3. Memory is dominated by the language server, and it is large

Sampled at 0.5 s intervals against **only the descendants of the Serena process
I spawned**, so a concurrent rust-analyzer on the same machine could not be
mistaken for mine.

| Process | kos peak RSS | marvel peak RSS |
|---|---|---|
| rust-analyzer | **1,204 MB** / 1,173 MB (two runs) | n/a |
| gopls | n/a | 601 MB (cold cache) / 212 MB (warm cache) |
| rust-analyzer-proc-macro-srv | 40 MB | n/a |
| Serena (python) | 113 MB | 128 MB |
| uv wrapper | 53 MB | 53 MB |
| **Total** | **1,381 to 1,412 MB** | 420 to 808 MB |

1.2 GB of rust-analyzer to answer questions about 8,681 lines of Rust. The
Python and uv layers together are about 166 to 181 MB, which is the portion a
native client could recover; the rest belongs to the language server and would
survive any rewrite.

An earlier uncontrolled sample caught gopls at 1,700 MB during marvel's
first-ever activation with no cache present. That number is **contaminated and
should be rerun solo before relying on it**, but it is directionally consistent
with the 601 MB cold-cache figure.

*Extrapolation, labelled as such:* at 3 to 5 concurrent sessions each holding a
warm rust-analyzer, language servers alone would want roughly 3.6 to 6 GB
resident. I did not measure concurrent sessions; this is arithmetic on the
single-session figure and should be tested before it is planned against.

### 4. Answers carry location, not provenance

Every result is a file path, a line range, and a containing symbol. Nothing
carries a commit, a date, an author, or a reason. That is correct behavior for
a language server, and it is the precise gap: a location tells you where the
code is now, not when it got that way, who decided it, or what was ruled out.

### 5. Persistence exists, but it is authored notes without provenance

I expected to report that Serena has no memory. That would have been wrong, so
I tested it: I wrote a memory in one server process, killed it, started a new
one, and read the memory back. It persisted.

The honest statement is more specific. Serena's memories are markdown files at
`.serena/memories/*.md`, and:

- the content is whatever an agent chose to type, not anything derived from the
  code;
- the file I wrote came back byte-identical with **no metadata added at all**:
  no date, no author, no commit, no link to the code it describes;
- they are per project, so kos memories are invisible from marvel;
- nothing detects staleness when the code moves underneath a note;
- `.serena/.gitignore` excludes only `/cache` and `/project.local.yml`, so
  memories and `project.yml` are **designed to be committed into the repo**.

That last point is a direct collision with this fleet's F25 discipline, where
props stay verbatim with provenance and backdrops are authored and marked as
such. Serena memories would land in the repo as unmarked, unprovenanced,
undated prose beside kos nodes that carry all three.

### 6. It writes into the working tree

Activating a project created `.serena/` in the repo root (config, a 344 KB
symbol cache, and the memories directory), showing up as `?? .serena/` in
`git status`. I removed it from both repos; kos is fully clean and marvel is
back to the two pre-existing `.DS_Store` files it had before I started. Worth
naming because any adoption has to decide whether that directory is committed,
ignored fleet-wide, or relocated.

## On W3 and W8, which are subtler than a pass or fail

**W3 (orientation).** The `onboarding` tool does not produce orientation. It
returns 2,023 characters of *instructions to the model*, telling it to go read
the project and write memory files itself, under generic headings:
`tech_stack`, `suggested_commands`, `conventions`, `task_completion`. So the
orientation is authored by the model, not derived by the tool, and the
categories are generic project scaffolding rather than what this fleet's
preamble actually needs (current bedrock, open frontier questions, what is
already ruled out). The brief predicted this outcome and the prediction held:
symbol tooling does not produce orientation, and the authored backdrops of F25
are already doing that work.

**W8 (why).** This is the most interesting result. Serena *did* surface the
rationale for the deprecated `serde_yaml` dependency, including
`_kos/probes/brief-kos-orient.yaml`, `_kos/findings/finding-030`,
`finding-029`, `KOS-charter.md`, and `docs/product-brief.md`. But it did so
through `search_for_pattern`, which is grep, not through any symbol tool. The
symbol tools cannot address those files at all: they are YAML and markdown, and
no language server indexes them.

The failure is therefore not that the knowledge is absent. It is on disk, and it
is greppable. The failure is that **the tool has no notion that a finding is a
different kind of object from a call site**. `finding-030`'s recorded reason sat
at roughly position three in a flat list of twenty files, ranked no differently
from `src/validate.rs:141`, which is just a parse call. There is no typing, no
ranking, and no provenance, so the answer to "why" is present in the output and
indistinguishable from the noise around it.

**Honesty discipline 2 (W8 is the strongest read-path signal here, and it does
not authorise read-path work).** W8 is the single most pointed piece of evidence
in this probe for building a typed, ranked, provenance-carrying read path: the
right finding was in the output and the tool could not tell it apart from a
parse call. I want to be exact about what that licenses. It is one workload out
of eight. A single workload names a gap; it does not name a program. If read-path
work proceeds, it proceeds as its own registered question with its own probe and
its own timebox, scored against its own pre-registered criterion, not as a
conclusion carried out of this finding on W8's back. I am recording the gap
plainly and declining to promote it, because the cheapest way to smuggle an
unearned mandate into the graph is to let the most vivid workload in a probe
stand in for a decision the probe was not scoped to make.

## The third outcome, addressed directly

The ticket requires me to say plainly whether live LSP is the right permanent
pattern, and to conclude "the pattern won, implement an LSP client natively in
Rust" rather than "keep Serena" if it is. Splitting the verdict is the honest
answer:

**For symbol-resolution workloads, the pattern won.** W1, W2, W4, and W5 were
served, and two of them beat a careful grep on correctness, not just on
convenience. That is not a marginal result and it should not be discarded
because three other workloads failed. The conclusion for this half is the one
the ticket names: **implement an LSP client natively in Rust**, and specifically
not "adopt Serena". Serena is a Python MCP server with a uv bootstrap and its
own dependency tree; adopting the pattern is a different decision from adopting
the vendor of the pattern.

**One caveat that materially changes the sizing of that work.** A native Rust
client does **not** reduce the dominant cost. The 1.2 GB and the 7 second cold
start belong to rust-analyzer, which a native client would still spawn and still
wait for. What a rewrite recovers is roughly 166 to 181 MB of Python and uv
overhead, plus control over process lifecycle, cache location, and whether
anything is written into the working tree. Those are real, and they are not the
same as making live LSP cheap. Any plan that assumes a Rust rewrite fixes the
memory profile is assuming something this probe measured to be false.

**For the other four workloads, the pattern cannot win**, because the questions
are not about symbols. History, orientation, cross-repo scope, and provenance
are not properties of a snapshot index of one working tree, and no amount of
language-server work adds them.

## What a kos ingest must provide that live LSP cannot

Each item is tied to the workload that demonstrated it, so none of these is a
wish-list entry:

1. **Fleet-scope query in one question** (W7). One language server set per
   active project is a hard boundary, not a configuration choice. The fleet is
   25-plus repos across at least two languages.
2. **Provenance on every answer** (W6, W8): the commit, the date, the author,
   and the reason. A file and line is a location, not a provenance.
3. **Typed knowledge objects** (W8). A finding, a node, a decision record, and a
   call site must be distinguishable and rankable. Serena returned all four in
   one undifferentiated list.
4. **History and cross-session continuity** (W6). What a prior session changed
   and concluded, not only what the tree currently says.
5. **Orientation material** (W3): the authored backdrops of F25, which are
   written rather than derived, and which no symbol index produces.
6. **Non-code artifacts as first-class** (W8). The `_kos/` YAML and markdown
   corpus is where this fleet's reasoning lives, and it is invisible to every
   language server.
7. **Staleness detection.** Serena memories never learn that the code moved.
   Recorded knowledge needs a mechanism for noticing it has gone out of date.
8. **Bounded resident cost.** An ingest can answer from an artifact without
   holding 1.2 GB warm per repo per session.

Note that items 1 through 7 are all things the fleet's existing `_kos/` corpus
already contains and that Serena could physically read as text, and still could
not use as knowledge. The gap is not corpus availability. It is that live LSP
has no representation for anything that is not a symbol.

## Recommendation

Keep the live-LSP pattern for symbol workloads and build it natively in Rust,
sized with the knowledge that the language-server memory and cold-start cost
survives the rewrite. Do not adopt Serena as infrastructure: it writes into the
working tree, its memory model conflicts with the F25 provenance discipline, and
it binds one language server set to one project. Treat the two halves as
complementary rather than competing, because the probe found no workload where
an ingest would replace symbol resolution and none where symbol resolution would
replace an ingest.

## Caveats on the numbers

A second agent ran rust-analyzer SCIP indexing on this machine throughout the
window. Load average moved between 4.83 and 9.06 during the run, and free memory
between roughly 444 MB and 2.6 GB.

- **Memory figures are PID-attributed** to descendants of the Serena process I
  spawned, so the concurrent rust-analyzer could not be counted as mine. I
  consider these sound.
- **Latency figures** were taken under variable load and are the least
  trustworthy. The cold-versus-warm ratio is large enough (15x to 28x) that
  contamination does not threaten the conclusion, but the absolute cold numbers
  should be **rerun solo before being used for planning**.
- The single 1,700 MB gopls sample is **contaminated and flagged above**; I did
  not rerun it, because it was not decision-relevant next to the attributed
  601 MB figure.
- Correctness results (W1, W2, W5, W7) do not depend on machine load at all, and
  I verified each against the source by hand.

## Reproduction

Artifacts in the probe scratchpad: `mcpdrive.py` (a minimal MCP stdio client
with per-call timing), `measure_solo.py` (PID-attributed RSS sampling plus the
restart test), `kos.json` and `out_marvel.json` (full tool outputs),
`solo_kos.json` and `solo_marvel.json` (timing and memory), `rss.csv` (the
uncontrolled sampler, retained only to document the contamination).

Both repos were left as found. `git status --short` is empty for kos and shows
only marvel's two pre-existing `.DS_Store` entries.
