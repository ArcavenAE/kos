# KOS Use Assessment — Operator Burden, Onboarding, Detection, Coexistence

> Assessment of how KOS is actually operated: the unguided cycle within a
> typical Claude session, onboarding of new repos, detection of
> not-onboarded and not-working states, and coexistence with parallel
> systems (bd/beads, multiclaude, gastown-class orchestrators,
> vsdd-factory pipelines, BMAD). Goal: recommendations that make KOS more
> reliable, more automatic, and less dependent on expert knowledge —
> within the SOUL constraints (no auto-summarization, no silent rewriting,
> human remains the pawl on promotion, uncertainty stays explicit).

---

## Part 1 — The operator's current burden

Reconstructed from CLAUDE.md, the CLI surface, and observed session
history, one full cycle requires roughly **fifteen distinct expert
judgments**, all voluntary, all invisible when skipped:

1. **Session start:** know to read the charter or run `kos orient`; know
   which directory/topology to launch from (finding-037 F4 — undefined
   for subrepos and standalone repos).
2. **Cycle discipline:** begin with a question; write a brief *before*
   probing; set `predicted_confidence` at brief time (1 of 25 briefs did).
3. **During work:** judge when an insight becomes a node vs. charter
   prose vs. nothing; pick among 8 edge types; pick a confidence tier;
   follow the id-naming grammar.
4. **Harvest:** remember the checklist; write the finding with
   `surprise_magnitude`; update affected nodes; move files between tiers;
   regenerate the charter; use the commit vocabulary.
5. **Periodically:** graveyard audits, staleness review, validation —
   none scheduled, none prompted.
6. **Repair:** notice drift/breakage unaided — nothing fires; CI ignores
   `_kos/**`.

## Part 2 — The fragility diagnosis

The evidence says skipping is the norm: 19% of findings have no brief;
92% of edges default to `derives`; the predictor fields sat unused; the
harvest checklist was added *because* steps were skipped, and steps were
skipped after; the charter re-inflated through three compressions; 11 orc
nodes and 15 kos probe-nodes drifted silently.

The pattern is named in the project's own bedrock —
**`elem-gate-enforcement`: instruction-based enforcement fails;
infrastructure enforcement works.** KOS today is an instruction-based
system whose own author's compliance is visibly partial. For any second
operator it is an expert instrument with no guardrails: cost of correct
use is high, cost of incorrect use is invisible, benefit is deferred —
Grudin's asymmetry expressed as UX.

A second fragility: **the cycle has no clock.** Orient → Ideate →
Question → Probe → Harvest → Promote is a loop with no mechanism that
knows where in the loop you are. Open briefs, unharvested findings, and
unmoved tiers exist in the files, but nothing reads the state back as
obligation. Every transition depends on the human remembering the next
verb.

## Part 3 — Recommendations: from recorder to partner

The inversion in one line: **today the operator drives the process and
the tool records; the fix is the tool drives the process and the operator
makes the decisions.** All items below are prompting, defaulting, and
machine-paid checking — never automated judgment.

### R1. Give the cycle a state machine — `kos status` as the spine
The workspace already encodes cycle state implicitly (briefs without
findings, findings without promotions, dirty dependents, stale frontier,
fired triggers). One command reads it and answers *"where am I and what
is the next verb?"* Nothing enforced; everything surfaced. `kos status`
replaces "read the charter" as the session entry ritual; orient becomes
what status recommends when context is cold. Fifteen memorized duties
collapse into one habit.

### R2. Hooks fire the rituals; humans perform them
Session-start hook → `kos status`. Pre-commit → validate plus soft
prompts ("finding lacks `surprise_magnitude` — proceed? [y/n]").
Post-harvest → charter render. Every-Nth-session → graveyard-audit and
staleness prompts. The human never remembers a cadence again; the human
only answers prompts. Bainbridge-safe: automate *reminding*, never
*judging*.

### R3. Make node creation cheaper than prose
The charter inflates because a paragraph costs 10 seconds and a node
costs 3 minutes. `kos capture`: one interactive command that takes a
sentence, proposes id/type/tier/edges as defaults the human confirms or
overrides, writes a valid node. LLM-assisted drafting is within spirit if
every field is human-confirmed (proposal ≠ promotion). Default-with-
override replaces blank-page-with-schema.

### R4. Fail visibly at the moment of authorship
All current failure is silent and deferred. Invert: validation errors at
commit time; dangling edges and unknown edge types rejected; `kos
finding` asks "which brief? [none → marks post_hoc: true]". The tool
teaches the schema at the point of use.

### R5. One golden path per workflow scale
`grv-single-workflow-type` established that one ceremony doesn't fit all
scales. `kos cycle start --scale=probe|feature|fix` sets which steps are
expected: a bug-scale fix isn't nagged about briefs; a project-scale
probe is. The expectation profile lives in the state machine, not the
operator's head.

### R6. Let the graph open the session
Reopener triggers (v3 Issue 24) plus `kos status` mean graveyard entries,
fired triggers, and dirty nodes present themselves at session start —
the lessons-learned retrieval-at-need failure, operationalized in
reverse. The operator stops being the system's scheduler and becomes its
judge — the only role the human-as-pawl model ever required.

---

## Part 4 — Onboarding new projects and repos

### Current burden
The operator must know: that `kos init` exists and where the binary
lives; which of four topologies applies (in-kos, orc-subrepo, standalone,
orc-without-kos — the fourth never designed); that init yields an empty
graph and stub charter; that seeding is their unnamed job; that seed =
sync-maximally-behind (bedrock insight, prose only — no command
implements it); and that docs and code are complementary lenses
(finding-016) requiring both passes.

### R7. `kos adopt <path>` — one verb replacing init + manual seed
Runs init, inventories the repo (docs, code, git history, CLAUDE.md,
.claude/, BMAD dirs, ADRs), classifies findings against the two-lens
model, and *proposes* a seed plan: "47 documents, 3 doc systems detected,
~15 candidate nodes from docs, ~8 from code structure. Proceed
lens-by-lens? [docs/code/both/skip]." Every proposed node is
confirm-or-override — the `kos capture` pattern scaled. This is the
universal-bridge idea's phase 1, scoped to ingestion only. Human judges;
tool drives.

### R8. Declared onboarding levels
Not every repo deserves a full graph. `_kos/kos.yaml` gains
`level: tracked | seeded | active` — drift-watched only / graph
populated, no active cycle / full process. Doctor and status scale
expectations to level. This kills the current binary where a repo is
either fully ceremonied or invisible — why switchboard/spectacle/curtain
sit at 17 nodes, zero findings, and no one can say whether that's fine.

---

## Part 5 — Detecting the not-onboarded and the not-working

Today nothing detects either state: a repo without `_kos/` is outside the
universe; a rotting graph looks identical to a healthy quiet one.

### R9. Workspace census — `kos graphs --census`
Orc-level walk of all repos; per-repo row: onboarded? level? last graph
touch? open-loop count? validate status? drift count? One-screen fleet
view, run by the orc session-start hook. Not-onboarded repos appear as
*rows, not absences* — the difference between "no data" and
"known-untracked" is the entire detection game.

### R10. Vital signs, not virtue metrics
Per-repo health = machine-checkable liveness only (Goodhart rule):
schema-valid %, dangling edges, fired-but-unaddressed reopeners,
charter/graph divergence (renderer diff ≠ 0), and the decay signature —
**days-since-last-graph-touch relative to git activity.** Code moving
while the graph is frozen = decay; both quiet = dormant, fine. Detectable
from git alone at zero author cost — the Reflexion pattern applied to
onboarding itself.

---

## Part 6 — Coexistence with parallel systems

The failure mode to avoid is the one already lived: knowledge duplicated
into whichever system is guaranteed-loaded, then diverging (F10's
design-questions.md; charter inflation).

### R11. KOS observes; it never demands migration
Per the shadow principle, other systems' artifacts are projections KOS
*reads* — never formats it forces. Beads/convoys, multiclaude worktrees,
factory pipeline state, BMAD docs keep their native homes. The bridge
framework's adapter split (fetch is format-specific; understanding is
LLM-scoped-by-graph) plugs into `kos adopt`'s inventory step. Beads is
the proven template: `grv-kos-as-task-tracker` drew the boundary (kos =
knowledge, bd = work) and it holds *because* neither claims the other's
domain.

### R12. Boundary registry, not integration code
Small `_kos/coexistence.yaml`: which system owns which artifact class
(tasks→bd, sessions→multiclaude, pipeline-state→factory,
specs→BMAD/spectacle) and which are seam-scan *inputs*. Doctor flags
boundary violations — a task-shaped frontier node, a spec claim living
only in kos — the same way it flags schema drift. Cheap, declarative,
makes the division of labor auditable instead of tribal.

### R13. Agent-loop delivery; observation inbox
Per-agent kos ceremony in multi-agent setups is dead on arrival
(concurrent soft constraints — the ThreeDoors incident lesson). The MCP
surface is the coexistence answer: foreign agents *query* the graph
mid-task and *emit candidate observations* to `_kos/inbox/`, triaged by
the human at harvest. Agents never write nodes directly; they feed the
funnel. Proposal ≠ promotion; the pawl is preserved. The inbox also
answers the multi-topic-source problem: unplaced observations land there
instead of being lost or misfiled.

### R14. Cross-system drift is just seam detection
BMAD PRD vs. kos bedrock disagreeing; factory config contradicting an
architecture node — cross-*system* seams, same signal class as
cross-document seams. With census scan-inputs declared, every parallel
system becomes another shadow to triangulate against: coexistence
*strengthens* the core function rather than competing with it.

---

## Summary table

| # | Recommendation | Burden removed | Cost posture |
| --- | --- | --- | --- |
| R1 | `kos status` state machine | 15 memorized duties → 1 habit | machine-paid |
| R2 | Hooks fire rituals | all cadence memory | machine-paid |
| R3 | `kos capture` | node-vs-prose cost gap | author-light, confirm-only |
| R4 | Fail at authorship | schema memorization | machine-paid |
| R5 | Scale-profiled cycles | ceremony mismatch | machine-paid |
| R6 | Graph opens session | operator-as-scheduler | machine-paid |
| R7 | `kos adopt` | init+seed expertise | author-light, confirm-only |
| R8 | Onboarding levels | all-or-nothing ceremony | declarative |
| R9 | Fleet census | detection of absence | machine-paid |
| R10 | Vital signs | detection of decay | machine-paid |
| R11 | Observe, never migrate | duplication pressure | zero |
| R12 | Boundary registry | tribal division of labor | declarative |
| R13 | MCP + inbox | per-agent ceremony | machine-paid |
| R14 | Cross-system seams | integration code | zero |

Nothing above touches the judgments the SOUL reserves for the human:
promotion, meaning, dissent. Everything above is the tool assuming the
scheduling, checking, and proposing that the operator currently performs
from memory.
