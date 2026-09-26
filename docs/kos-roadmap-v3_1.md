# KOS Roadmap v3.1 — Operability Fold

> Delta document. v3 (literature-hardened) remains the base; v3.1 folds
> the use assessment's fourteen recommendations (R1–R14,
> `kos-use-assessment.md`) into the milestone structure. Unchanged v3
> issues are not restated. The standing rules gain a fourth.
>
> Placement logic: every R-item is machine-paid or confirm-only, so all
> pass the Grudin rule by construction. They slot as (a) extensions to
> existing issues, (b) three new issues, (c) one new sub-milestone.

---

## Standing rules (rule 4 added)

1. Each issue closes with a commit.
2. No new author-side capture requirement ships before MR demonstrates
   retrieval value (Grudin rule).
3. Process metrics are diagnostic, never gates (Goodhart rule).
4. **The tool drives the process; the operator makes the decisions
   (Bainbridge rule).** Automation of reminding, scheduling, checking,
   and proposing is encouraged; automation of judging (promotion,
   meaning, dissent) is prohibited. Every automated proposal is
   confirm-or-override.

---

## Fold map — R-items into the v3 structure

### Into M1 (Enforcement Inversion)

**Issue 4 extended (validate gate) ← R4.**
Fail-at-authorship absorbed: unknown edge types and dangling targets
reject at commit (already in v3); *add* creation-time interrogation —
`kos finding` prompts "which brief? [none → post_hoc: true]",
`kos probe` prompts for `predicted_confidence`. The schema is taught at
the point of use, not memorized.

**Issue 18 extended (instrument tests) ← R1 dependency.**
Status-machine state derivation (open briefs, unharvested findings,
unmoved tiers) gets test fixtures alongside validate/model/drift/charter.

**NEW Issue 29: `kos status` — the cycle state machine (R1, R6).**
Reads workspace state and answers "where am I; what's the next verb":
open loops, obligations, fired reopener triggers, dirty dependents,
stale frontier. Nothing enforced, everything surfaced. Replaces
"read the charter" as the session entry ritual in CLAUDE.md; `orient` is
what status recommends on cold context. R6 rides on it once Issue 24's
triggers exist: the graph opens the session.
*AC: one command; session protocol updated; a seeded fixture workspace
with three known open loops yields exactly those three obligations.*

**NEW Issue 30: Ritual hooks (R2, R5).**
Session-start hook → status; post-harvest → charter render;
every-Nth-session → graveyard-audit and staleness prompts (feeds v3
Issue 14's cadence); pre-commit soft prompts for missing predictor
fields. R5's scale profiles: `kos cycle start --scale=probe|feature|fix`
sets the expectation profile status checks against — fix-scale work is
never nagged about briefs.
*AC: hooks installed via lefthook/CLAUDE.md; cadence prompts fire on a
fixture; scale profile suppresses/raises expectations correctly.*

### Into MR (Retrieval-First)

**Issue 20 extended (telemetry) ← R10 input.**
Reads.jsonl additionally timestamps per-repo graph-touch, enabling the
decay signature (graph frozen while git moves).

**Issue 23 extended (MCP server) ← R13.**
Adds the write-side complement: `_kos/inbox/` for agent-emitted candidate
observations, triaged by the human at harvest. Agents query; agents
propose; agents never write nodes. Proposal ≠ promotion. The inbox is
also the landing zone for multi-topic-source extraction (universal-bridge
problem) — unplaced observations go to triage, not to loss.
*AC addition: a foreign agent files an observation via MCP; it appears in
inbox; `kos status` surfaces it; promotion requires human action.*

**NEW Issue 31: `kos capture` — cheap node creation (R3).**
One interactive command: sentence in → proposed id/type/tier/edges as
confirm-or-override defaults → valid node out. LLM-assisted drafting
permitted; every field human-confirmed. Attacks the charter-inflation
vacuum at its source: closes the 10-second-prose vs. 3-minute-node gap.
*Cost posture: author-light, confirm-only — MR-eligible under rule 2
because it reduces capture cost rather than adding capture duty.*
*AC: capture-to-committed-valid-node under 30 seconds on a fixture;
telemetry (Issue 20) later tests whether captured nodes are ever read.*

### NEW sub-milestone MO (Onboarding & Fleet) — after MR core, before M3

Rationale: adopt/census depend on capture (31), status (29), and the
two-lens findings; they precede the science block because Issues 11/13
(baseline, external operator) need cheap onboarding to run at all.

**NEW Issue 32: `kos adopt <path>` (R7) + onboarding levels (R8).**
One verb replacing init+manual-seed: inventory (docs, code, git,
CLAUDE.md, .claude/, BMAD, ADRs) → lens-classified seed plan → 
confirm-or-override node proposals via the capture path. Implements
seed = sync-maximally-behind as a command instead of prose bedrock.
Levels land in `_kos/kos.yaml`: `tracked | seeded | active`; doctor and
status scale expectations to level; the fourth topology
(orc-without-kos) gets a decision, not a shrug.
*AC: adopt on one real BMAD-born repo yields a reviewed seed of ≥10
nodes with zero hand-written YAML; finding-016's both-lenses requirement
enforced by the plan prompt; level recorded and respected by doctor.*

**NEW Issue 33: Fleet census + vital signs (R9, R10) + coexistence
registry (R12, R14).**
`kos graphs --census`: per-repo rows — onboarded?, level, last graph
touch, open loops, validate status, drift count, decay signature.
Not-onboarded repos are rows, not absences. Vital signs are liveness
only (Goodhart rule): schema-valid %, dangling edges, unaddressed fired
reopeners, charter/graph render-diff, graph-frozen-while-git-moves.
`_kos/coexistence.yaml` declares system boundaries (tasks→bd,
sessions→multiclaude, pipeline-state→factory, specs→BMAD/spectacle) and
seam-scan inputs; doctor flags boundary violations as a signal class;
cross-system contradictions enter seam detection as first-class input
(R14). R11 (observe, never migrate) is recorded as a design value node,
not an issue — it constrains all adapters.
*AC: census runs from orc session-start hook; one deliberately decayed
fixture repo is flagged by signature alone; one boundary violation
(task-shaped frontier node) is caught by doctor.*

### Into M3 (Science) — dependency updates only

Issue 13 (external operator): now runs against Issue 32's adopt +
Issue 23's MCP endpoint — the probe cost drops from "coach a stranger
through expert ceremony" to "hand them two commands," which is itself
the measurement.
Issue 26 (canaries): census (33) becomes the reveal surface — an
undetected canary is a vital-signs miss, sharpening what the health
metric means.

---

## Revised sequencing (v3 base, deltas bolded)

```
n:     Issue 4(+R4 prompts) — gate on
n+1:   Issues 1, 19
n+2:   Issues 2, 3
n+3:   Issues 18(+state fixtures), 5, 20(+touch-telemetry)
n+4:   **Issue 29 (status)**, Issue 21
n+5:   **Issues 30 (hooks), 31 (capture)**, Issue 22
n+6:   Issue 23(+**inbox**), Issue 24, Issue 14 (now hook-prompted)
n+7:   **Issue 32 (adopt+levels)** — first real BMAD repo onboarded
n+8:   **Issue 33 (census+coexistence)**, Issue 25 (retrodiction)
then:  10, 11, 17, 26 · 12, 13(cheapened) · 27 · 15 gated · 16, 28
```

Net effect on the operator, stated once: fifteen memorized duties become
one habit (`kos status`), two confirm-flows (`capture`, `adopt`), and a
set of prompts that arrive when due. The judgments — promotion, meaning,
dissent — remain exactly where the SOUL put them.
