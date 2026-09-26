> **SUPERSEDED — historical primary source.** This is roadmap v1
> (unversioned original). Revised by [kos-roadmap-v2.md](kos-roadmap-v2.md)
> (items 1, 4, 10, 11 revised; 17–19 added), reassessed by
> [kos-roadmap-v3.md](kos-roadmap-v3.md), amended by
> [kos-roadmap-v3_1.md](kos-roadmap-v3_1.md). Authoritative for original
> intent only; do NOT work from this document. Current scope lives in
> v3/v3.1 and the GitHub issues (kos#51–80).

# KOS: Assessment and Recovery Roadmap

> Produced from the founding conversation (sessions 001–present) plus review of
> github.com/ArcavenAE/kos. Each numbered issue below is written for direct
> import as a GitHub issue. A `gh` script is at the end.
> Note: 4 issues already exist on the repo — reconcile before importing.

---

## Part 1 — Assessment

### The concept as it developed

The conversation produced, in order: spec-as-product with epistemic confidence
tiers (bedrock/frontier/graveyard); a fractal cycle (orient → ideate → question
→ probe → harvest → promote); graph-over-documents with typed nodes, typed
edges, and provenance; the shadow principle (documents and code as lossy
projections of the graph); the correspondence layer (typed spec↔code links);
the predictor layer (prediction-error as learning signal, the missing fourth
element of the cognitive architecture); GraphRAG as the query interface; the
substrate hypothesis (YAML-in-git at a structural ceiling); and finally the
honest split — two missions were conflated from the start:

1. **Knowledge accumulation and projection** for human-AI teams. Tractable.
   Evidenced. This is what KOS actually is.
2. **LLM continuity** ("pearls on a string"). A model-level research problem.
   Infrastructure alone cannot deliver it. KOS is a precondition for that
   research, not the research itself.

### The implementation — what's earned

- Empirical validation across 16–21 projects; ~45% consequential signal rate,
  honestly quantified with a noise breakdown.
- A stabilized signal taxonomy (contradiction, gap, silent abandonment, drift,
  AI-config seam, enforcement gap).
- The graveyard: append-only negative knowledge with finding/ruling/reopener
  structure. KOS's best feature and its most original contribution.
- Predictor fields (`predicted_confidence`, `surprise_magnitude`) live in
  briefs and findings — calibration data is accumulating.
- Real downstream value: ThreeDoors bootstrap filed actionable issues;
  finding-019/020 drove a cross-repo refactor; BetterDials tested the method
  in a foreign domain.
- The charter renderer (just shipped) exposed 11 silently-broken nodes on its
  first run — infrastructure catching what prose could not. This is the thesis
  demonstrated on the project itself.

### The implementation — where it deviated

Root deviation: **the charter became a writable working document instead of a
regenerated projection.** The regeneration mechanism specified in session-001
was never built, so the charter absorbed every piece of content that needed to
be reliably present at session start. Everything else follows from this:

1. **No query layer.** The charter is a cache for queries that don't exist
   (5-whys root cause, confirmed independently by three analysis voices).
2. **Soft constraints unenforced.** KOS's own platform finding — "instruction-
   based enforcement fails; infrastructure enforcement works" — was never
   applied to KOS itself. Result: 11 schema-drift nodes, charter re-inflation
   through three compression cycles, cycle-boundary harvest gaps.
3. **Analysis outpaces artifacts.** The party-of-agents episode produced 30+
   proposals and zero commits. Insight production has no forcing function
   converting it to action.
4. **Single-operator validation.** Every test to date has the author as
   operator. No control condition exists for the 45% signal claim (naive
   LLM-over-docs baseline never run). Severity classification is single-rater.
5. **Calibration data uncollected as science.** The predictor fields exist but
   no calibration analysis has ever been run on them.
6. **Distributed evidence is illegible.** 227 nodes across 9 repos with no
   cross-repo traversal; an outside reader cannot reconstruct project state
   from artifacts alone.

### The recovery principle

Every fix below converts a discipline problem into an infrastructure
guarantee, or converts a claim into a measurement. Nothing below is a process
exhortation. The project has proven it cannot hold soft constraints; it has
also proven infrastructure catches what prose hides. Build accordingly.

---

## Part 2 — Roadmap

Milestones are sequential. Issues within a milestone can run in parallel
unless a dependency is stated. Every issue closes with a commit, not a
document.

---

### Milestone 0 — Restore the architecture (charter as projection)

#### Issue 1: Fix the 11 schema-drift nodes

**Labels:** `bug`, `schema`, `M0`

The `kos charter render` first run skipped 11 frontier nodes that fail
deserialization: 7 use undefined edge types (`relates` ×4, `related` ×2,
`depends` ×2, `resolves-partially` ×1), 2 have `provenance` as a YAML sequence
instead of a struct.

**Deliverables:** Corrected node files; decision recorded per undefined edge
type (map to existing type, or add to schema with definition); re-run renderer.

**Acceptance:** `kos validate` passes on all graphs; rendered charter includes
all frontier nodes; a finding node records which violations existed and how
each was resolved.

---

#### Issue 2: Classify charter-only content

**Labels:** `charter`, `M0`
**Depends on:** Issue 1

The diff between rendered output (247 lines) and hand-maintained charter
(~1100 lines) is the inventory of content with no node home: preamble
(problem statement, values, non-goals), operating posture, "how this document
is used," session log, research notes pointer, plus prose bodies on F-items.

**Deliverables:** Every charter-only span classified as exactly one of:
(a) becomes a node, (b) becomes static preamble in the renderer,
(c) discarded with one-line rationale in the commit message.

**Acceptance:** Zero unclassified content; nodes created for (a); renderer
preamble updated for (b); classification recorded as a finding.

---

#### Issue 3: Cut over to rendered charter; forbid hand-edits

**Labels:** `charter`, `infrastructure`, `M0`
**Depends on:** Issue 2

**Deliverables:** `kos charter render --write` becomes the only sanctioned
write path to charter.md. Pre-commit hook (lefthook) compares charter.md
against fresh render output; mismatch fails the commit with "edit nodes, not
the projection."

**Acceptance:** Hand-editing charter.md is structurally blocked; harvest
protocol in CLAUDE.md updated to end with the render step; one full cycle
completed under the new regime.

---

### Milestone 1 — Infrastructure-enforced discipline

#### Issue 4: Schema validation as a hard pre-commit gate

**Labels:** `infrastructure`, `schema`, `M1`

The 11-node drift class must become structurally impossible, in every graph,
not just kos's.

**Deliverables:** `kos validate` wired into lefthook pre-commit across all
9 graphs; edge-type whitelist enforced; provenance shape enforced;
unknown-field warnings.

**Acceptance:** A deliberately malformed node cannot be committed anywhere on
the platform.

---

#### Issue 5: Context budget accounting and eager-import removal

**Labels:** `infrastructure`, `context`, `M1`

Sessions pay an unmeasured, unconditional context tax. The orc CLAUDE.md
eagerly @-imports subrepo CLAUDE.mds that Claude Code's per-cwd resolution
already handles.

**Deliverables:** (a) `scripts/claudemd-budget.sh` — token estimate per
@-import, total, flags over threshold. (b) Remove subrepo CLAUDE.md @-imports
from orc. (c) Log budget for 5 sessions before any further pruning — measure,
then cut.

**Acceptance:** Budget report exists in-repo; eager imports removed; before/
after token cost recorded as a finding.

---

#### Issue 6: Artifact-close rule for the cycle

**Labels:** `process`, `M1`

Harvest currently accepts prose findings without committed artifacts —
analysis-only sessions close cycles they shouldn't.

**Deliverables:** CLAUDE.md session protocol amended: a cycle closes only on
a commit touching code, schema, or node files. Two consecutive analysis-only
sessions trigger a mandatory graveyard review of the current line of work.
Where checkable (commit present in session window), enforce via the harvest
checklist script.

**Acceptance:** Rule live in protocol; first enforcement instance (pass or
triggered review) recorded.

---

### Milestone 2 — Query layer MVP (kill the cache)

#### Issue 7: Define the subgraph context format

**Labels:** `design`, `query`, `M2`

Spec before engine. The format is what every query command emits and what
agents consume: node id, type, confidence, summary, typed edges with targets
and confidence, provenance pointer, staleness flag.

**Deliverables:** `docs/subgraph-context-format.md` with schema and 3 worked
examples hand-produced from the ThreeDoors graph; validated by using one
example to answer a real question in-session.

**Acceptance:** Format doc merged; the hand-produced examples demonstrably
improved an answer vs. charter-only context (record as finding).

---

#### Issue 8: Core query commands

**Labels:** `cli`, `query`, `M2`
**Depends on:** Issue 7

**Deliverables:** `kos frontier --active`, `kos bedrock [--tag <area>]`,
`kos recent --since <n>d`, each emitting the subgraph context format
(plus `--human` for prose).

**Acceptance:** Each command answers its question from the graph in <1s on the
227-node platform; output validates against the format.

---

#### Issue 9: Task-conditional orientation

**Labels:** `cli`, `query`, `M2`
**Depends on:** Issue 8

**Deliverables:** `kos orient --for=<task>` — scope-tagged retrieval returning
the relevant bundle (bedrock in scope, active frontier in scope, recent
findings touching scope). Frontier nodes gain a `scope:` tag as needed.

**Acceptance:** A session in one subrepo orients via the command instead of
loading the full charter; token cost vs. charter load measured and recorded.
This is the empirical test of the "charter is a cache" hypothesis — the
finding stands regardless of which way it goes.

---

### Milestone 3 — Scientific rigor

#### Issue 10: Calibration pipeline and scheduled review

**Labels:** `science`, `predictor`, `M3`

Prediction fields exist; the science doesn't. Pre-registration discipline:
`predicted_confidence` is written before the probe and never revised —
enforce via the validate gate (Issue 4: reject edits to that field on
existing briefs).

**Deliverables:** `predicted_confidence` required on all new briefs;
`surprise_magnitude` required on all new findings; `kos calibration` —
tabulates prediction vs. outcome, flags systematic over/under-confidence.
Review every 5 sessions, results committed as findings.

**Acceptance:** First calibration report produced from the ≥9 existing
brief/finding pairs plus new ones; one concrete adjustment made based on it.

---

#### Issue 11: Controlled baseline for the signal claim

**Labels:** `science`, `validation`, `M3`

The 45%-consequential claim has no control condition. Measure KOS's marginal
value over the obvious alternative.

**Deliverables:** On 2 previously-probed projects, run a naive baseline —
LLM over the raw documents, no graph, same time budget — and score its
findings with the same severity rubric. Compare yield, precision, and overlap
against the KOS results.

**Acceptance:** Comparison committed as a finding. If the baseline matches
KOS, that is a bedrock-challenging result and gets treated as one. README
claims updated to cite the controlled result.

---

#### Issue 12: Inter-rater reliability on severity classification

**Labels:** `science`, `validation`, `M3`

Consequential/low-impact/noise is currently single-rater (the author).

**Deliverables:** Blind second rating of a 30-finding sample (second human or
independent agent session with no access to original ratings); agreement
statistic computed; rubric tightened where disagreement clusters.

**Acceptance:** Agreement measured and published; rubric v2 committed if
agreement < 0.7.

---

#### Issue 13: External-operator probe

**Labels:** `science`, `validation`, `M3`
**Depends on:** Issues 3, 8

The strongest untested claim: artifacts alone can orient a stranger.

**Deliverables:** One person who has never been in a session runs `kos init`
+ `kos orient` on their own repo, and separately reads the kos repo cold and
writes what the project is, what it produced, and what's next. Both recorded
verbatim.

**Acceptance:** The gap between their account and actual state is committed
as a finding — that gap is the measured communication debt. No coaching
during the probe.

---

#### Issue 14: Scheduled graveyard audit

**Labels:** `process`, `M3`

Self-pruning happens ad hoc; it must be load-bearing.

**Deliverables:** Every 3rd session includes a graveyard audit: propose 3
candidates (assumptions, scope items, or stale frontier questions — sunset
rule: frontier with no probe activity in 10 sessions is automatically a
candidate), accept or reject each with explicit reasoning, commit accepted
entries. F1 (Director Design, frontier since session-001) is the first test
case.

**Acceptance:** First audit completed and committed; cadence recorded in
CLAUDE.md protocol.

---

### Milestone 4 — Substrate research track (gated on M0–M2)

#### Issue 15: Substrate property probes

**Labels:** `research`, `substrate`, `M4`

The substrate hypothesis names five properties files cannot provide:
first-class relationships, preserved reasoning chains, on-demand projection,
session-surviving context, concurrent transaction semantics. Do not choose a
substrate; test one property at a time.

**Deliverables:** One probe per property, each with a brief
(`predicted_confidence` required), tested against the ThreeDoors 104-node
graph. Candidates: SQLite+graph layer, Dolt, XTDB/Datomic-class store,
git-DAG-direct (no file presentation layer). Timebox per probe: one session.

**Acceptance:** Each probe produces a finding or a graveyard entry. No
substrate migration begins until ≥3 properties have empirical results.

---

### Milestone 5 — Mission clarity

#### Issue 16: Split the missions; graveyard the conflation

**Labels:** `charter`, `mission`, `M5`

**Deliverables:** (a) Graveyard entry: `grv-continuity-via-infrastructure` —
ruled out; LLM continuity requires model-level mechanisms (persistent-state
inference, graph-native training); reopener: when such mechanisms exist, KOS
is the substrate they operate on. (b) Mission restated in charter preamble:
KOS is a knowledge accumulation and projection system for human-AI teams.
(c) Continuity tracked as a named external research dependency, not an
implied KOS property.

**Acceptance:** Graveyard entry committed; rendered charter reflects the
restated mission; README updated to match.

---

## Part 3 — Sequencing and forcing function

```
Session n:    Issues 1, 2          (schema fix, classification)
Session n+1:  Issue 3              (cutover — architecture restored)
Session n+2:  Issues 4, 5          (gates, budget)
Session n+3:  Issues 6, 7          (artifact rule, format spec)
Session n+4:  Issue 8              (query commands)
Session n+5:  Issues 9, 14         (task orient, first graveyard audit)
Session n+6:  Issue 10             (first calibration report)
Then:         11, 12, 13 as capacity allows; 15 gated on 0–2; 16 anytime
```

Standing rule for the whole roadmap: **each issue closes with a commit.**
Findings about issues do not close issues. If a session produces only
analysis, the cycle did not close.

---

## Part 4 — Import script

Requires `gh` authenticated to ArcavenAE/kos. Bodies reference this file —
commit it to the repo first (suggested path: `docs/roadmap-2026-07.md`).

```bash
#!/usr/bin/env bash
set -euo pipefail
R="ArcavenAE/kos"
D="docs/roadmap-2026-07.md"

gh label create M0 --repo $R --color 0e8a16 --force
gh label create M1 --repo $R --color 0e8a16 --force
gh label create M2 --repo $R --color 1d76db --force
gh label create M3 --repo $R --color 5319e7 --force
gh label create M4 --repo $R --color b60205 --force
gh label create M5 --repo $R --color fbca04 --force
gh label create science --repo $R --color 5319e7 --force

i() { gh issue create --repo $R --title "$1" --label "$2" --body "$3

Full spec: $D (Issue $4)"; }

i "Fix 11 schema-drift nodes (undefined edge types, provenance shape)" "bug,schema,M0" \
  "7 nodes use undefined edge types (relates, related, depends, resolves-partially); 2 have provenance as sequence. Invisible until the renderer parsed them. AC: kos validate passes everywhere; finding records each resolution." 1
i "Classify charter-only content: node / preamble / discard" "M0" \
  "Diff rendered charter (247 lines) vs hand-maintained (~1100). Every span gets exactly one classification. AC: zero unclassified; finding committed." 2
i "Cut over to rendered charter; block hand-edits via pre-commit" "M0" \
  "charter render --write is the only write path. Lefthook compares charter.md to fresh render; mismatch fails commit. AC: hand-editing structurally blocked; one cycle completed under new regime." 3
i "kos validate as hard pre-commit gate across all 9 graphs" "M1,schema" \
  "Edge-type whitelist, provenance shape, unknown-field warnings. AC: malformed node cannot be committed anywhere." 4
i "Context budget accounting; remove eager subrepo CLAUDE.md imports" "M1" \
  "Token estimate per @-import; remove orc's eager subrepo imports (per-cwd resolution handles it). Measure 5 sessions before further cuts. AC: before/after cost recorded as finding." 5
i "Artifact-close rule: cycles close on commits, not prose" "M1" \
  "Cycle closes only on commit touching code/schema/nodes. Two analysis-only sessions trigger graveyard review. AC: rule live; first enforcement recorded." 6
i "Define subgraph context format (spec before engine)" "M2" \
  "docs/subgraph-context-format.md + 3 hand-built examples from ThreeDoors graph. AC: one example demonstrably improves an in-session answer; recorded as finding." 7
i "Query commands: kos frontier / bedrock / recent" "M2" \
  "Each emits subgraph context format; --human for prose. AC: <1s on 227-node platform; output validates." 8
i "kos orient --for=<task> — task-conditional context bundle" "M2" \
  "Scope-tagged retrieval replaces full-charter load for subrepo sessions. AC: token cost vs charter load measured — empirical test of the charter-as-cache hypothesis." 9
i "Calibration pipeline: enforce prediction fields, ship kos calibration" "M3,science" \
  "predicted_confidence required pre-probe and immutable; surprise_magnitude required at harvest; report every 5 sessions. AC: first report from existing pairs; one adjustment made from it." 10
i "Controlled baseline for the 45% signal claim" "M3,science" \
  "Naive LLM-over-docs on 2 previously-probed projects, same rubric, same time budget. AC: comparison committed; README cites controlled result. If baseline matches KOS, treat as bedrock-challenging." 11
i "Inter-rater reliability on severity classification" "M3,science" \
  "Blind second rating of 30-finding sample; agreement statistic; rubric v2 if <0.7. AC: agreement published." 12
i "External-operator probe: stranger runs init/orient uncoached" "M3,science" \
  "One outside person, their repo, no coaching; plus cold-read of the kos repo. AC: gap between their account and actual state committed as the measured communication debt." 13
i "Scheduled graveyard audit every 3 sessions" "M3" \
  "3 candidates per audit, accept/reject with reasoning. Sunset rule: frontier with no probe activity in 10 sessions is auto-candidate. F1 is the first test case. AC: first audit committed; cadence in protocol." 14
i "Substrate property probes (one property per probe)" "M4,research" \
  "Five properties, tested one at a time against ThreeDoors graph. Candidates: SQLite+graph, Dolt, XTDB-class, git-DAG-direct. AC: finding or graveyard per probe; no migration until 3+ properties have results." 15
i "Split the missions: graveyard continuity-via-infrastructure" "M5" \
  "grv-continuity-via-infrastructure with reopener (model-level mechanisms). Mission restated: knowledge accumulation + projection for human-AI teams. AC: graveyard committed; charter and README updated." 16
```

---

*End of roadmap. The measure of this document is how fast it stops being read
and starts being closed.*
