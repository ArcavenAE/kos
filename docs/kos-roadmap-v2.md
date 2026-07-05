# KOS Assessment & Roadmap v2 — Primary-Source Edition

> Supersedes `kos-roadmap.md` (v1). v1 was built from README, a truncated
> charter fetch, and secondhand reports. v2 is built from full inspection of
> the uploaded repo (`kos-main.zip`): all 260 graph artifacts, schema, Rust
> source, CI, hooks. Every claim below carries a path. Nodes were weighted
> over prose per instruction; both were tested against the code.

---

## Part 1 — What inspection verified (claims that held)

**V1. The signal-yield claim is better grounded than v1 credited.**
`_kos/findings/finding-023-signal-to-noise-four-projects.yaml` documents real
methodology: owner ground-truth classification on ThreeDoors (40%
consequential / 30% low / 30% noise) and independent-discovery
cross-referencing on OpenClaw (graph findings matched community-discovered
issues #23314 and #4940 — convergent validation), Atmos, and GitLab. This is
not hand-waving. What's still missing is a *control*: no naive
LLM-over-documents baseline has ever been run, so KOS's marginal value over
the obvious alternative remains unmeasured. (v1 Issue 11 stands, premise
corrected.)

**V2. The graveyard is real and mostly disciplined.** 9 entries in
`_kos/nodes/graveyard/`, five with full approach/ruling/reopener structure.
`grv-kos-as-task-tracker` shows genuine scope self-pruning.

**V3. The CLI is substantial, not vaporware.** 16 modules in `src/`:
orient, validate, graph, drift, bridge, seed, compact, reflect, doctor, init,
charter (with `--write`), plus node-creation helpers (idea/question/finding/
probe). The charter renderer reported in-conversation exists at
`src/charter.rs` with the regeneration warning banner already written.

**V4. Process linkage exists.** The charter tracks completed frontier items
with strikethrough + finding references (F-items → finding-029/030/031/032).
The session-002 harvest checklist survived into `CLAUDE.md:81-109`. Commit
vocabulary (harvest/promote/graveyard/probe) is documented.

**V5. Findings are evidence-shaped.** Sampled findings contain success-signal
scoring against briefs, honest PARTIAL results, and misses enumerated
alongside hits (finding-016 lists 9 of 11 missed issues explicitly).

---

## Part 2 — What inspection falsified or downgraded

**F1. The knowledge graph has ZERO automated validation. Anywhere.**
- `.github/workflows/ci.yml:12,21` — CI **paths-ignores `_kos/**`** entirely.
- `lefthook.yml` — pre-commit runs cargo fmt/clippy/deny; pre-push runs
  cargo test. No hook touches nodes.
- `kos validate` exists (`src/validate.rs`) but nothing invokes it
  automatically.

The project's own bedrock finding — *"instruction-based enforcement fails;
infrastructure enforcement works"* (`elem-gate-enforcement`, README §"What
we've learned" item 4) — is **inverted in its own repo: the code is gated,
the knowledge is not.** This is the single sharpest result of the deep dive.

**F2. Validation has a scope hole, and the hole contains live schema drift.**
`src/validate.rs:23` loads only `nodes/**`. Probe-produced node sets in
`_kos/probes/brief-*-nodes/` (15 files across 3 probe dirs) are never parsed.
Those files contain **15 edges of type `observation`** — a type absent from
both `schema/node.schema.yaml` and the Rust `EdgeType` enum
(`src/model.rs:131-165`). Serde would hard-reject them if ever loaded. The
exact silent-drift class the renderer just exposed in the orc repo (11 nodes)
exists in kos itself, undetected. The instrument does not point at itself.

**F3. Edge-type monoculture: the typed-edges thesis is ~92% unexercised.**
Across all artifacts: `derives` ×414, `supersedes` ×10, `contradicts` ×10,
`observation` ×5 in nodes-adjacent files (undefined), `implements` ×4,
`partially_resolves` ×2, and **zero** uses of `supports`, `instantiates`,
`discovered_from`. The founding thesis — typed relationships make corruption
audible — rests on a graph where 92% of relationships use the weakest,
most generic type. Either the taxonomy is wrong-sized or the discipline
never formed. This is testable (see Issue 17).

**F4. Referential integrity is soft and already broken.** Edge targets
reference IDs that don't exist: `val-uncertainty-first` (renamed to
`val-uncertainty-structural`), `elem-collaboration-model`, `finding-016`,
`td-phase1-validated`, `ax-arch-validation`, `ax-claude-md` — ~6 real
dangling targets. `validate.rs:158-162` emits only a *warning* ("may be a
finding or probe") because `known_ids` is built solely from `nodes/`, so it
cannot distinguish a legitimate cross-directory reference from a broken one.
Founding-node provenance chains are severed and nothing fails.

**F5. Type/confidence conflation regressed inside the graveyard.** 4 of 9
graveyard files carry `type: element` with `confidence: graveyard` and lack
approach/reopener fields (`grv-flat-file-context-scaling`,
`grv-human-as-integration-bus`, `grv-reactive-query-model`,
`grv-single-workflow-type`). Two node models coexist; the schema's own
graveyard structure is honored ~55%.

**F6. The predictor layer is near-empty in this repo.** Exactly **1** brief
carries `predicted_confidence` (0.6) and **3** findings carry
`surprise_magnitude` (low, medium, medium). The platform-Claude's report of
"5+ briefs / 4+ findings" is not substantiated here (possibly distributed in
repos not provided — unverifiable). Additionally **8 of 43 findings (19%)
have `probe: null`** — post-hoc findings with no pre-registered hypothesis.
Pre-registration is aspirational, not practiced.

**F7. The epistemically load-bearing code is untested.** 12 unit tests total,
confined to `process.rs` and `updater.rs`. Zero tests for `validate.rs`,
`model.rs` (parsing/enum enforcement), `drift.rs`, or `charter.rs`. The
self-updater is better tested than the knowledge validator.

**F8. Documentation drift in the flagship's own front matter.**
README claims a `placeholder/` node directory (`README.md:239`) — absent.
`sessions/` contains only `session-001.md` despite ~20 sessions of history;
the session log lives inline in the charter (the pattern the orc analysis
condemned). Charter is 889 lines and growing.

**F9. Frontier staleness with no sunset mechanism.** 25 of 26 frontier nodes
were created ≥3 months ago (8 from the founding date 2026-03-15); 1 created
this month. No audit cadence exists in the protocol.

---

## Part 3 — Process-in-use, critically

The cycle (orient → ideate → question → probe → harvest → promote) is
genuinely practiced — briefs exist, findings score against success signals,
completions link back. Three structural critiques survive contact with the
evidence:

1. **All enforcement is voluntary.** Every discipline mechanism (checklist,
   validate, commit vocabulary) is instruction, not infrastructure. F1/F2
   are the direct consequence. The project *discovered* this principle
   empirically (ThreeDoors incident convergence) and has not applied it
   to itself.

2. **Probe granularity is elastic post-hoc.** `brief-mvg-bootstrap` is
   credited as the probe for 13 findings — the entire 12-project sweep was
   retrofitted under one umbrella brief. Combined with 19% probe-null
   findings, "probes are timeboxed with declared success signals" describes
   the ideal, not the median practice.

3. **The graph's connective tissue is thinner than its node count.** 260
   artifacts connected almost entirely by `derives` means traversal answers
   "what came from what" but rarely "what conflicts with what" or "what
   replaced what" — the queries that justified typed edges in the first
   place. The 10 `contradicts` edges are where nearly all the demonstrated
   signal value lives (ThreeDoors incident convergence). That ratio is the
   thesis's own signal-to-noise problem.

**Net:** The concept survives inspection. The practice is a partially
disciplined prototype whose central claims are (a) genuinely evidenced at
the signal-detection layer, (b) unenforced at the integrity layer, and
(c) untested at the typed-edge and prediction layers.

---

## Part 4 — Roadmap v2 (diff against v1)

**Unchanged:** Issues 2, 3, 5, 6, 7, 8, 9, 13, 14, 15, 16 stand as written
in v1. **Revised:** 1, 4, 10, 11 below. **New:** 17, 18, 19.

---

### Issue 1 (REVISED): Repair schema drift in BOTH repos' graphs
**Labels:** `bug`, `schema`, `M0`

Scope expands. In kos itself: 15 `observation` edges in
`_kos/probes/brief-*-nodes/`; 4 graveyard nodes with `type: element` and
missing approach/reopener fields; ~6 dangling edge targets including renamed
founding nodes (`val-uncertainty-first`→`val-uncertainty-structural`). Plus
the 11 orc-repo nodes from the renderer report.

**AC:** `kos validate` (post-Issue-4 scope) passes clean on both graphs;
each repair class recorded in one finding; rename map committed so
provenance chains resolve.

---

### Issue 4 (REVISED): Close the enforcement inversion
**Labels:** `infrastructure`, `schema`, `M1` — **promoted to highest
priority in M1; cheapest highest-leverage fix in the entire roadmap**

Three concrete changes:
(a) `validate.rs` scope: load `nodes/**`, `findings/**`, `probes/**`
(including `brief-*-nodes/` subdirs); build `known_ids` from all of them;
dangling target = **error**, not warning (allow explicit `external:` prefix
for cross-repo refs).
(b) Remove `_kos/**` from CI `paths-ignore`; add a CI job running
`kos validate --merged`.
(c) Add `kos validate` to `lefthook.yml` pre-commit.

**AC:** A node with an undefined edge type, or a dangling target, cannot be
committed or merged. The `observation` files fail until Issue 1 fixes them —
that failure firing is the acceptance test.

---

### Issue 10 (REVISED): Calibration pipeline — corrected premise
**Labels:** `science`, `predictor`, `M3`

v1 assumed calibration data was "in flight." Ground truth: n=1 brief,
n=3 findings in this repo. There is no backfill — pre-registration cannot
be retrofitted honestly.

**Deliverables:** `predicted_confidence` required (schema + validate) on all
*new* briefs; `surprise_magnitude` required on all *new* findings;
`probe: null` findings permitted only with an explicit `post_hoc: true` flag
and rationale. `kos calibration` report ships when n≥10 pairs exist.
**AC:** First report at n≥10; probe-null rate tracked as a process metric
(current baseline: 19%).

---

### Issue 11 (REVISED): Controlled baseline — cite what exists
**Labels:** `science`, `validation`, `M3`

finding-023's convergent-discovery method (community issues independently
confirming graph findings) is real external validation and should be
foregrounded in README. The remaining gap is the counterfactual: same
documents, same time budget, capable LLM, **no graph** — does it find the
same seams?

**AC:** Baseline run on ThreeDoors + one other previously-probed project
using the finding-023 rubric; overlap/precision/unique-yield reported;
README's yield claims cite both finding-023 and the controlled result.

---

### Issue 17 (NEW): Test the typed-edge thesis or right-size the taxonomy
**Labels:** `science`, `schema`, `M3`

92% `derives`, three types never used. Two-part probe:
(a) Retype a 50-edge sample where a more specific type applies; measure what
`contradicts`/`supersedes`/`implements` traversal answers that `derives`
cannot (use the ThreeDoors incident-convergence query as the benchmark —
it is the project's best demonstrated typed-edge win).
(b) Sunset decision on `supports`/`instantiates`/`discovered_from`: earn
first use within the probe or move to a schema graveyard note.

**AC:** Finding quantifies typed-edge marginal value; schema v0.4 either
defends or prunes the unused types. This is a direct empirical test of a
founding bedrock claim (`val-typed-traversable`) — treat a negative result
as bedrock-challenging.

---

### Issue 18 (NEW): Test the instrument
**Labels:** `code-quality`, `M1`

Zero tests on validate/model/drift/charter. Minimum set: EdgeType enum
rejection of unknown types; dangling-target detection (post-Issue-4
semantics); graveyard-node structural requirements; charter render
idempotence (render → render produces identical output); drift hash
stability.

**AC:** ≥20 tests across those four modules; CI green; a deliberately
malformed fixture node fails the suite.

---

### Issue 19 (NEW): Truth pass on self-description
**Labels:** `docs`, `M0`

README claims `placeholder/` (absent); `sessions/` implies history it
doesn't hold; charter embeds the session log the project's own analysis
condemned.

**AC:** README structure matches `ls`; session log extracted to
`sessions/` or `_kos/session-log.md` with charter pointer (this also
pre-implements part of Issue 2); every quantitative README claim cites a
finding ID.

---

### Revised sequencing

```
Session n:    Issue 4 (enforcement)  ← firing failures become the worklist
Session n+1:  Issues 1, 19           (repairs + truth pass, guided by the gate)
Session n+2:  Issues 2, 3            (charter classification + cutover)
Session n+3:  Issues 18, 5           (instrument tests, context budget)
Session n+4:  Issues 6, 7            (artifact rule, subgraph format)
Session n+5:  Issues 8, 9, 14        (queries, task-orient, first graveyard audit)
Then:         10, 11, 17 (science block) · 12, 13 · 15 gated on M0–M2 · 16 anytime
```

Ordering change from v1: **enforcement before repair.** Turning the gate on
first makes the drift enumerate itself — the failures are the repair
checklist, and the gate proves itself by catching them.

---

## Part 5 — Import script (delta)

Prereq: v1 issues 2,3,5–9,13–16 created from the v1 script. This creates the
revised/new set. Commit this file as `docs/roadmap-v2-2026-07.md` first.

```bash
#!/usr/bin/env bash
set -euo pipefail
R="ArcavenAE/kos"; D="docs/roadmap-v2-2026-07.md"
i(){ gh issue create --repo $R --title "$1" --label "$2" --body "$3

Evidence & full spec: $D (Issue $4)"; }

i "Repair schema drift in kos + orc graphs (observation edges, graveyard type conflation, dangling founding refs)" "bug,schema,M0" \
"kos repo: 15 undefined 'observation' edges in _kos/probes/brief-*-nodes/; 4 graveyard nodes typed 'element' missing approach/reopener; ~6 dangling targets incl. val-uncertainty-first rename. Plus 11 orc nodes. AC: validate clean on both; rename map committed." 1
i "Close the enforcement inversion: validate scope + CI + pre-commit" "infrastructure,schema,M1" \
"CI paths-ignores _kos/**; lefthook runs cargo only; validate.rs loads nodes/ only and only WARNS on dangling targets. Fix all three: full-scope validation, dangling=error (external: prefix escape), CI job, pre-commit hook. AC: the existing observation-edge files fail the gate — that firing is the acceptance test." 4
i "Calibration pipeline (corrected premise: n=1 brief, n=3 findings today)" "science,predictor,M3" \
"No backfill possible. Require predicted_confidence on new briefs, surprise_magnitude on new findings, post_hoc:true flag for probe-null findings (current rate 19%). Ship kos calibration at n>=10 pairs." 10
i "Controlled baseline: naive LLM-over-docs vs graph (finding-023 rubric)" "science,validation,M3" \
"finding-023's convergent-discovery validation is real — cite it in README. Remaining gap: counterfactual with no graph, same docs, same budget. AC: overlap/precision/unique-yield on 2 projects; README cites both." 11
i "Test the typed-edge thesis or right-size the taxonomy (92% derives)" "science,schema,M3" \
"414/450 edges are 'derives'; supports/instantiates/discovered_from have zero uses. Retype 50-edge sample, benchmark traversal value against the ThreeDoors incident-convergence query; sunset-or-defend unused types in schema v0.4. Negative result is bedrock-challenging for val-typed-traversable." 17
i "Test the instrument: validate/model/drift/charter have zero tests" "code-quality,M1" \
"12 tests exist, all in process.rs/updater.rs — the self-updater is better tested than the knowledge validator. AC: >=20 tests incl. unknown-edge rejection, dangling detection, graveyard structure, charter render idempotence; malformed fixture fails suite." 18
i "Truth pass: README/sessions/charter self-description" "docs,M0" \
"README claims placeholder/ dir (absent); sessions/ holds only session-001 vs ~20 sessions of history; session log embedded in charter. AC: structure matches ls; log extracted; every quantitative README claim cites a finding ID." 19
```

---

*v2 verdict in one line: the signal-detection layer is real and externally
corroborated; the integrity layer is unenforced and already leaking; the
typed-edge and prediction layers are the founding claims still awaiting
their first honest test. Turn on the gate, then let the failures write the
worklist.*
