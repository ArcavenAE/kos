#!/usr/bin/env bash
# create-roadmap-issues.sh — file the roadmap v3/v3.1 issues in ArcavenAE/kos.
#
# Source docs: docs/kos-roadmap-v3.md (Issues 1-28; 1-19 reconstructed from
# v3's "v2 issues stand" summaries) + docs/kos-roadmap-v3_1.md (Issues 29-33,
# extensions, standing rule 4). Sequencing per the 2026-07-04 two-track
# adoption consensus (bd aae-orc-b0kp, round-3 amended). Reconciliation map
# with existing bd tickets: bd aae-orc-4u8m notes.
#
# Idempotence: guarded by title search — re-running skips existing titles.
# bd: aae-orc-dtj0

set -euo pipefail

REPO="ArcavenAE/kos"

# ── Milestones ───────────────────────────────────────────────────────
declare -a MILESTONES=(
  "M0 Repair & Truth"
  "M1 Enforcement Inversion"
  "MR Retrieval-First"
  "MO Onboarding & Fleet"
  "M3 Science"
  "M4 Substrate"
  "M5 Mission & Positioning"
)

echo "== milestones"
existing_ms=$(gh api "repos/$REPO/milestones?state=all&per_page=100" -q '.[].title')
for m in "${MILESTONES[@]}"; do
  if grep -qxF "$m" <<<"$existing_ms"; then
    echo "exists: $m"
  else
    gh api "repos/$REPO/milestones" -X POST -f title="$m" --silent && echo "created: $m"
  fi
done

# ── Issue helper ─────────────────────────────────────────────────────
existing_titles=$(gh issue list -R "$REPO" --state all -L 200 --json title -q '.[].title')

mk() { # mk <milestone> <title> <body>
  local ms="$1" title="$2" body="$3"
  if grep -qxF "$title" <<<"$existing_titles"; then
    echo "skip (exists): $title"
    return 0
  fi
  gh issue create -R "$REPO" --title "$title" --milestone "$ms" --body "$body" | tail -1
}

FOOT_A="Track A (operator-facing, serial) per bd aae-orc-b0kp."
FOOT_B="Track B (machine-paid, parallel) per bd aae-orc-b0kp."

echo "== issues"

# ── M0 — Repair & Truth ─────────────────────────────────────────────
mk "M0 Repair & Truth" "roadmap#1: schema-drift repair + _kos/migrations/ infrastructure (schema v0.4)" \
"v2-stands per v3 (docs/kos-roadmap-v3.md M0) + v3 addendum: repair observation edges, graveyard type conflation, dangling founding refs; rename maps become \`_kos/migrations/\` with a \`kos migrate\` stub — v0.4 ships WITH its migration path.

Empirical worklist (finding-060, orc): **~128 of 531 fleet edges use undeclared types** (\`related\`=78, \`leaves-open\`=19, \`depends-on\`=9, ...). Decide legalize-vs-retype per type before sweeping.

Coordinate: bd aae-orc-e689 (schema v0.4 provenance/region/conditional_on) — v0.4 ships ONCE with both payloads. bd: aae-orc-fd15 (umbrella). Cost posture: author-neutral, system-paid. $FOOT_B Fed by roadmap#4's failure enumeration."

mk "M0 Repair & Truth" "roadmap#2: charter-content classification" \
"v2-stands per v3 M0. Classify charter content (graph-derived vs backdrop prose) ahead of cutover. Overlaps bd aae-orc-foqo (T3g charter reduction) + aae-orc-t6pn (T3d dogfood audit) — same work, different framing; coordinate. $FOOT_B"

mk "M0 Repair & Truth" "roadmap#3: charter cutover" \
"v2-stands per v3 sequencing (n+2 'charter classification, cutover'). Complete the charter-as-renderer cutover for the kos charter (orc F22 pattern; subrepo generalization is bd aae-orc-gezz). $FOOT_B"

mk "M0 Repair & Truth" "roadmap#19: README truth pass" \
"v2-stands per v3 M0. Bring README claims in line with what the code actually does. Cheap, rides along with M0. $FOOT_B"

# ── M1 — Enforcement Inversion ──────────────────────────────────────
mk "M1 Enforcement Inversion" "roadmap#4: full-scope validate gate — CLI + CI only (pre-commit severed to roadmap#30)" \
"v3 M1 + v3.1 R4 extension, AMENDED by the round-3 consensus: ship the full-scope validator + CI job now (machine-paid, author-neutral); the **pre-commit delivery surface is severed** and rides roadmap#30 behind the hook environment contract (bd aae-orc-dxmh, finding-058).

Full-scope means fixing the cwd-scoping bug: validate currently returns the identical 58-node result from every subrepo cwd — fleet coverage ~16% (bd **aae-orc-z67m**, finding-060). Its failure output enumerates the M0 worklist. Add R4 creation-time interrogation: \`kos finding\` prompts 'which brief? [none → post_hoc: true]'; \`kos probe\` prompts for predicted_confidence.

Overlaps: bd aae-orc-1pxb (orc lefthook wiring — the pre-commit half). Dangling targets = error. $FOOT_B — head of Track B."

mk "M1 Enforcement Inversion" "roadmap#18: instrument tests — validate/model/drift/charter (+ status-state fixtures)" \
"v2-stands per v3 M1 + v3.1 R1-dependency extension: >=20 tests across validate/model/drift/charter; add fixtures for status-machine state derivation (open briefs, unharvested findings, unmoved tiers) consumed by roadmap#29. The validator must be more trustworthy than the graph it judges. $FOOT_B"

mk "M1 Enforcement Inversion" "roadmap#5: context budget accounting (measure-only)" \
"v3 M1 (modified): ship the accounting; pruning waits for roadmap#20 telemetry to show what's actually read. Cutting before measuring repeats the KM pattern of optimizing unobserved behavior. Related: party finding that machine-paid cost relocates to context budget (bd aae-orc-kx8f notes). $FOOT_B"

mk "M1 Enforcement Inversion" "roadmap#6: artifact-close dashboard signal in kos doctor (not a gate)" \
"v3 M1 (softened per Goodhart/Austin): sessions closed with/without artifacts becomes a doctor dashboard signal, never a hard gate; two consecutive analysis-only sessions still trigger the graveyard review. Near-identical to bd **aae-orc-0ejy** (doctor warns when substantive commits create no node) — 0ejy is the implementation ticket; keep them linked. $FOOT_B"

# ── MR — Retrieval-First ────────────────────────────────────────────
mk "MR Retrieval-First" "roadmap#20: retrieval telemetry (per-session append-only files)" \
"v3 MR (Improvement #7 promoted) + v3.1 R10 extension (per-repo graph-touch timestamps for the decay signature), AMENDED by round-3 consensus: **collection starts immediately** with per-session append-only files keyed by session UUID — the multi-writer question is dissolved, not resolved (see orc frontier node question-kos-multi-writer-concurrency sub-A); the writer-model question gates only the aggregator/consumer.

Pre-registered hypothesis (v3): retrieval follows a power law; >=40% of nodes never read post-creation. One telemetry spine shared with flyloft cue sheets — bd **aae-orc-kx8f**. 10-session clock starts at first collection. Cost posture: author-neutral; produces the benefit ledger. $FOOT_B"

mk "MR Retrieval-First" "roadmap#21: subgraph context format + query commands" \
"v3 MR (v2 issues 7+8 merged): format spec first, three hand-built ThreeDoors examples, then frontier/bedrock/recent commands emitting it. Retrieval at the moment of need (the LLIS failure inverted).

MUST be designed with the stage-rig epic (bd aae-orc-p4t1, esp. p4t1.8 HippoRAG task-conditional cue study and p4t1.9 paint/refresh/cue commands) — MR and the stage rig describe the same retrieval layer in two vocabularies; do not build two retrieval systems. $FOOT_B"

mk "MR Retrieval-First" "roadmap#22: task-conditional orient" \
"v3 MR (v2 issue 9): the empirical test of charter-as-cache. Token cost measured against full charter load; either outcome is a finding. Depends on roadmap#21's format. $FOOT_B"

mk "MR Retrieval-First" "roadmap#23: MCP server — read-only epistemic API (+ observation inbox)" \
"v3 MR (Improvement #8 promoted; 'the delivery mechanism the entire literature says is mandatory') + v3.1 R13 extension (\`_kos/inbox/\` — agents query, agents propose, agents never write nodes; human triages at harvest).

Consensus constraints (bd aae-orc-b0kp + orc frontier question-kos-multi-writer-concurrency): localhost-first Phase 0 (Dolt-server pattern), query/access logging from day one, inbox provenance stamping + per-session rate caps, inbox content never served as graph knowledge, network exposure via switchboard is a parked user decision. Gated on roadmap#21's format AND on M0 repair — an MCP server over a graph that silently drops nodes serves lies with a nice API. $FOOT_B"

mk "MR Retrieval-First" "roadmap#24: reopener triggers — the graveyard's differentiator" \
"v3 MR, flagged as **the project's most publishable novelty**: graveyard reopeners become machine-checkable structured \`trigger\` conditions (dependency-hash change, date horizon, external-fact assertion) evaluated by \`kos stale\`; fired triggers surface in orient. Negative knowledge that re-presents itself at the moment it becomes relevant — the feature no lessons-learned system ever shipped.

Sequenced AFTER M0 repair (triggers over conflated graveyard nodes fire garbage — round-3 consensus). Design the trigger vocabulary against bd aae-orc-m2bz (T2c bidirectional verb taxonomy). Cost posture: small structured cost at graveyard time (the cheapest moment), machine-delivered benefit forever. Only surviving Toulmin fragment (\`rebuttal_conditions\` on bedrock) doubles as this trigger vocabulary. $FOOT_B"

mk "MR Retrieval-First" "roadmap#29: kos status — the cycle state machine" \
"v3.1 Issue 29 (R1, R6): reads workspace state and answers 'where am I; what is the next verb' — open loops, obligations, fired reopener triggers (once #24 exists), dirty dependents, stale frontier. Nothing enforced, everything surfaced. Replaces 'read the charter' as the session entry ritual; fifteen memorized duties collapse into one habit.

**Wizard-of-Oz first** (round-2 consensus, Maya): a shell script ships before any Rust; promote only after it survives three real cycles. WoZ prototype + probe brief: aae-orc tools/kos-status + kos/_kos/probes/brief-kos-status-woz.md. AC (v3.1): seeded fixture workspace with three known open loops yields exactly those three obligations. $FOOT_A — head of Track A."

mk "MR Retrieval-First" "roadmap#30: ritual hooks — session-start, post-harvest, Nth-session (gated on hook environment contract)" \
"v3.1 Issue 30 (R2, R5): session-start → status; post-harvest → charter render; every-Nth-session → graveyard-audit + staleness prompts; pre-commit soft prompts for missing predictor fields; \`kos cycle start --scale=probe|feature|fix\` expectation profiles (fix-scale work is never nagged about briefs).

HARD GATE (round-3 consensus): the hook execution-environment contract must exist first — bd **aae-orc-dxmh** (finding-058: lefthook loses toolchain PATH under agent shells; fail-OPEN semantics; self-diagnostic). This issue also carries roadmap#4's severed pre-commit surface. Overlaps bd aae-orc-b7vi (charter-render CI gate). $FOOT_A"

mk "MR Retrieval-First" "roadmap#31: kos capture — cheap node creation" \
"v3.1 Issue 31 (R3): one interactive command — sentence in → proposed id/type/tier/edges as confirm-or-override defaults → valid node out. Attacks charter inflation at its source (10-second prose vs 3-minute node). LLM-assisted drafting permitted; every field human-confirmed; proposal != promotion.

AC: capture-to-committed-valid-node under 30s on a fixture; roadmap#20 telemetry later tests whether captured nodes are ever read. Verification-side controls (override-rate vital sign, pawl canaries) tracked in bd **aae-orc-qao2**. Cost posture: author-light, confirm-only. $FOOT_A"

# ── MO — Onboarding & Fleet ─────────────────────────────────────────
mk "MO Onboarding & Fleet" "roadmap#32: kos adopt + onboarding levels (tracked/seeded/active)" \
"v3.1 Issue 32 (R7, R8): one verb replacing init+manual-seed — inventory (docs, code, git, CLAUDE.md, .claude/, BMAD, ADRs) → lens-classified seed plan → confirm-or-override node proposals via the capture path. Levels land in \`_kos/kos.yaml\`; doctor and status scale expectations to level.

Baseline motivation measured (finding-060): 7 fleet graphs have nodes but zero findings/briefs and nobody can say whether that's fine. AC: adopt on one real BMAD-born repo yields a reviewed seed of >=10 nodes with zero hand-written YAML; finding-016's both-lenses requirement enforced. LAST on Track A — do not onboard the fleet onto an unproven loop. $FOOT_A"

mk "MO Onboarding & Fleet" "roadmap#33: fleet census + vital signs + coexistence registry" \
"v3.1 Issue 33 (R9, R10, R12, R14): \`kos graphs --census\` per-repo rows (onboarded?, level, last graph touch, open loops, validate status, drift count, decay signature — graph-frozen-while-git-moves). Not-onboarded repos are rows, not absences. Vital signs are liveness only — **diagnostic, never gates; no kos vital sign in branch protection or required checks** (Goodhart enforcement location, round-1 Priya).

Deployment pattern (consensus): compute via launchd LaunchAgent to a cached status file; session-start reads the file, never walks 20 repos. \`_kos/coexistence.yaml\` declares system boundaries (tasks→bd, sessions→multiclaude, pipeline-state→factory, specs→BMAD/spectacle); doctor flags boundary violations; cross-system contradictions enter seam detection (R14). R11 (observe, never migrate) recorded as a design value node, not an issue. Sub-check overlap: bd aae-orc-hoi2. Baseline instrument to absorb: aae-orc scripts/kos-baseline-vitals.sh (finding-060). $FOOT_B"

# ── M3 — Science ────────────────────────────────────────────────────
mk "M3 Science" "roadmap#25: retrodiction engine — kos asof + blinded historical seam replay" \
"v3 M3, first science item ('the project's highest-credibility experiment and its best paper'): \`kos asof <ref>\` reconstructs the graph at a historical commit; replay seam detection on the four ThreeDoors incidents; operator blinded to incident selection; prediction registered before running (>=2 of 4 detectable at T-minus-one-sprint). Backtesting hygiene per Bailey et al.; the blinding protocol is not optional, it IS the experiment (Fischhoff/Dekker). Blinding design must account for one-operator-shared-agent-context reality (party round 1, Mary). Rides the MCP read surface (round-2 consensus). $FOOT_B"

mk "M3 Science" "roadmap#10: calibration pipeline — Brier scoring on predicted_confidence" \
"v2-stands per v3 M3, survey-hardened: adopt Brier scoring + forecasting apparatus wholesale; surprise_magnitude maps to calibration residuals; briefs ARE registered reports (no post-hoc hypothesis editing; post_hoc:true flag; HARKing named). Lock predictions — timestamped, immutable post-harvest. Baseline (finding-060): predicted_confidence in 1/19 kos briefs, 8/19 orc. Upstream design input: bd aae-orc-szh (predictor engine layer). $FOOT_B"

mk "M3 Science" "roadmap#11: controlled baseline — naive LLM-over-docs vs graph" \
"v2-stands per v3 M3: finding-023 rubric, two projects. Does the graph beat a naive LLM reading the documents cold? $FOOT_B"

mk "M3 Science" "roadmap#17: typed-edge thesis test — the formality tax, measured" \
"v2-stands per v3 M3, sharpened: retype 50 edges, measure traversal value AND authoring cost per typed edge. If value < cost, prune the taxonomy to derives/contradicts/supersedes and treat as confirmation of incremental-formalization literature, not defeat.

Baseline data ready (finding-060): fleet derives share 52.5% (69% of valid-typed edges); 24% of edges use undeclared types — the taxonomy already failed its authors once (Bowker/Star residual-category reading, party round 1 Maya: grow edge types bottom-up from prose vocabulary). Transcript-mining input: bd aae-orc-b6tu. $FOOT_B"

mk "M3 Science" "roadmap#26: canary nodes — epistemic chaos engineering (dead last)" \
"v3 M3: three canaries across tiers, sealed hash-committed registration, five normal sessions, reveal. The only game-resistant health metric (L5).

Sequenced DEAD LAST on both tracks (unanimous, rounds 1-3). Gates before any planting (bd **aae-orc-j53y**): failure-class taxonomy from the graveyard corpus; canaries declared in a signed OUT-OF-BAND registry (append-only history means a canary is reachable at every historical ref forever — kos asof and MCP must be canary-aware by default); written control narrative (auditor-facing). $FOOT_B"

mk "M3 Science" "roadmap#12: inter-rater reliability — Cohen's kappa on a 30-finding sample" \
"v2-stands per v3 M3. Do two raters classify the same findings the same way? $FOOT_B"

mk "M3 Science" "roadmap#13: external-operator probe" \
"v2-stands per v3 M3, cheapened by roadmap#32 (adopt) + roadmap#23 (MCP): the probe cost drops from 'coach a stranger through expert ceremony' to 'hand them two commands' — which is itself the measurement. Also the first real test of the Grudin thesis outside the solo-operator mask (party round 1, Victor: onboarding isn't a feature, it's the experiment that validates the entire thesis). $FOOT_B"

mk "M3 Science" "roadmap#14: scheduled graveyard audit — every 3 sessions, 10-session sunset rule" \
"v2-stands per v3 M3; hook-prompted once roadmap#30 lands. $FOOT_B"

# ── M4 — Substrate ──────────────────────────────────────────────────
mk "M4 Substrate" "roadmap#15: substrate probes (gated on M0-MR)" \
"v2-stands per v3 M4 + survey addition: every substrate probe brief must answer the three MDE killers (tool lock-in, round-trip breakage, skill asymmetry) before it runs. Related evidence: orc question-kos-multi-writer-concurrency sub-C (node-write concurrency threshold — at what creation rate does YAML-in-git need the bd Phase 0 treatment). $FOOT_B"

# ── M5 — Mission & Positioning ──────────────────────────────────────
mk "M5 Mission & Positioning" "roadmap#16: mission split + graveyard continuity-via-infrastructure" \
"v2-stands per v3 M5. $FOOT_B"

mk "M5 Mission & Positioning" "roadmap#27: chronicle — narrative projection of causal history" \
"v3 (Improvement #9, admitted as read-side projection): \`kos chronicle <node>\` walks provenance/edge chains chronologically, emits a structured event list; optional LLM prose pass with citation-per-sentence discipline (if citation coverage can't reach ~100%, ship the event list and skip the prose). Reuses roadmap#21's traversal — dependency, not deferral. Paired with roadmap#13 as its comprehension benchmark. Party note (Maya): session logs are already a voluntarily-paid chronicle — build on the thriving capture behavior. $FOOT_B"

mk "M5 Mission & Positioning" "roadmap#28: publishable claims register" \
"v3 M5: three paper-shaped claims tracked to evidentiary closure — (a) retrodictive seam detection (roadmap#25), (b) reopener-triggered negative knowledge (roadmap#24), (c) calibrated engineering hypotheses (roadmap#10 at n>=30). Nothing drafted until its evidence closes. Party note (Victor): the wedge is the retrodiction result; straddling five research communities is positioning in none. $FOOT_B"

echo "== done"
