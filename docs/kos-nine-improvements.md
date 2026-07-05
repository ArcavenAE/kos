# KOS — Nine Improvements Beyond the Current Vision

> Constraint honored: none of these appear in the conversation's design
> history, the repo/zip, or roadmap v1/v2. Each is specified to probe depth:
> concept, why it's underserved, mechanism, first probe, and the risk that
> would graveyard it.

---

## 1. Source Reliability Ledger (Admiralty grading for provenance)

**Concept.** Intelligence tradecraft solved epistemic provenance a century
ago: rate the *source* (A–F reliability) separately from the *information*
(1–6 credibility). KOS records who created a node but never accumulates a
track record. A ledger derives per-source reliability from history: how
often a source's nodes reach bedrock vs. get superseded/graveyarded, and
(once Issue 10 runs) their calibration error.

**Why underserved.** Provenance is currently identity, not trust. An agent
that confabulated three findings and a human with a 90% bedrock-survival
rate produce nodes of identical standing. Every retrieval treats them
equally.

**Mechanism.** `provenance.created_by` becomes a foreign key into
`_kos/sources/<id>.yaml` (reliability grade, computed stats, manual
override + rationale). Queries gain `--min-reliability`. Ripple propagation
weights by source grade. Grades recompute at harvest.

**First probe.** Backfill grades from the existing 43 findings' survival
history; check whether grade predicts which frontier nodes later promoted.

**Graveyard risk.** With few distinct sources (one human, few agent
versions), grades may not discriminate. n may be too small until the
platform has more operators.

---

## 2. Canary Nodes — adversarial self-testing of the epistemics

**Concept.** Mutation testing for knowledge. Periodically inject known-false
but plausible nodes (a fabricated bedrock claim, a subtly wrong dependency,
a stale-but-confident finding) under sealed registration. Measure
time-to-detection and detection channel (drift? contradiction? human?
never?).

**Why underserved.** Every validation to date measures whether KOS finds
signal in *others'* corpora. Nothing measures the graph's immune response
to corruption in itself — the exact failure mode (silent agent corruption)
that motivated the typed-edge design in session-001.

**Mechanism.** `kos canary plant` writes a node from a template library and
a sealed record to `_kos/.canaries/` (gitignored, hash-committed so planting
is provable but contents hidden). `kos canary reveal` scores outstanding
canaries. Detection SLO becomes a health metric.

**First probe.** Plant 3 canaries across confidence tiers; run 5 normal
sessions; reveal. Hypothesis: contradiction-bearing canaries get caught,
orphan-plausible ones survive indefinitely.

**Graveyard risk.** Operators who know canaries exist change behavior
(Hawthorne effect). Mitigate with random long delays and low base rate.

---

## 3. Confidence Half-Life — temporal decay of bedrock

**Concept.** Bedrock is currently immortal; real knowledge stales at
domain-dependent rates. Give confidence a half-life: each node carries
`verified_at` and a decay class (physics-slow: values, non-goals;
market-fast: tool comparisons, version facts). Past its re-verification
horizon, bedrock degrades to `bedrock-stale` — still authoritative, but
flagged in every retrieval and queued for cheap re-verification.

**Why underserved.** Drift detection is *event*-driven (content hash of
dependencies changed). Nothing handles *erosion* — the world changing while
the graph's inputs sit untouched. `elem-context-ceiling`'s measurements,
the standards-landscape evaluation, the tool graveyards (skillshare,
speckit): all decay-class-fast, all treated as permanent.

**Mechanism.** Schema: `decay_class: durable | stable | volatile` with
default horizons (∞ / 12mo / 3mo). `kos stale` lists past-horizon nodes;
re-verification is a micro-probe (confirm → reset clock; fail → demote with
finding).

**First probe.** Classify the 23 bedrock nodes; count how many are already
past a reasonable horizon. Prediction: ≥4 volatile-class items verified
only at creation.

**Graveyard risk.** Horizon-setting is arbitrary until calibration data
exists; may generate re-verification busywork. Start with volatile class
only.

---

## 4. Traversable Reasoning — Toulmin fields on nodes

**Concept.** Nodes store conclusions; reasoning lives in prose and dies
there. Add optional structured argumentation: `claim` (the title),
`grounds` (edge refs to evidence), `warrant` (why grounds support claim,
one sentence), `rebuttal_conditions` (what observation would defeat this).
The reasoning becomes queryable: "show me every bedrock whose warrant
depends on single-operator observation."

**Why underserved.** The substrate hypothesis names "preserved reasoning
chains" as a property files can't provide — and defers it to a future
substrate. This captures 60% of that property now, at schema level, no
migration. It also operationalizes reopeners: `rebuttal_conditions` are
machine-checkable reopener triggers, where today's graveyard reopeners are
prose.

**Mechanism.** Schema v0.4 optional block; validate warns when bedrock
lacks `rebuttal_conditions` (unfalsifiable bedrock is the smell). `kos why
<node>` renders the argument tree by traversing grounds recursively.

**First probe.** Retrofit the 3 founding values + 5 newest bedrock; attempt
`kos why val-typed-traversable`. Does the tree bottom out in evidence or in
assertion? Either answer is a finding.

**Graveyard risk.** Ceremony cost — if authors skip it, it becomes another
soft constraint. Gate only on bedrock promotion, nowhere else.

---

## 5. Retrodiction Engine — `kos asof` and historical seam replay

**Concept.** The strongest KOS claims are counterfactuals: "the graph would
have caught #6 before implementation" (finding-023). Test them. Git makes
time travel free: `kos asof <date>` reconstructs the graph at a historical
commit; replay seam detection against the documents as they existed then;
check whether known-later incidents were predictable at that state.

**Why underserved.** Roadmap Issue 11 tests the present (baseline vs.
graph, today). Nothing tests the *temporal* claim — early detection — which
is the actual product pitch. Retrodiction converts the anecdote into a
measured rate: of N incidents with known discovery dates, KOS-at-T-minus-30
flags K.

**Mechanism.** `kos asof <ref> -- <subcommand>` (checkout to temp worktree,
run, discard). A retrodiction harness pairs ThreeDoors incident dates with
pre-incident document states.

**First probe.** The four ThreeDoors incidents. Prediction registered
before running: ≥2 of 4 detectable at T-minus-one-sprint. This is the
single highest-credibility experiment available to the project.

**Graveyard risk.** Hindsight leakage — the person building the historical
graph knows the incidents. Mitigate: bootstrap the historical state from
documents only, by an operator blinded to which incidents are being tested.

---

## 6. Dissent Nodes — minority reports at promotion time

**Concept.** The graph records outcomes; it cannot hold "B overruled A."
Graveyard is *ruled out*; dissent is *unresolved but overridden* — a live
alternative that lost the promotion decision without being falsified. A
`dissent` node attaches to a promotion, preserving the losing argument, its
author, and its trigger condition for reconsideration.

**Why underserved.** Historiography self-corrects through minority reports;
KOS currently launders disagreement into false consensus at every promote
step. The human-as-pawl model makes this worse: when the human overrules an
agent (or vice versa), the override is invisible. finding-019 says humans
see what the graph can't — dissent nodes record when the graph's own
operators saw differently from each other.

**Mechanism.** `kos promote --with-dissent <file>`; dissent nodes carry
`overridden_by`, `trigger` (condition for revisit), and edge
`dissents_from → <promoted node>`. `kos stale`/audit surfaces dissents
whose triggers have fired.

**First probe.** Mine the conversation history: at least two documented
splits exist (Q4-vs-Q5 prioritization; "no charter edits" vs. "query-cited
edits"). Backfill them; check whether either trigger has since fired.

**Graveyard risk.** Low volume in a one-human project. Value scales with
operator count — may be premature until the platform has more voices.

---

## 7. Knowledge Economics — retrieval telemetry and node ROI

**Concept.** Every node has a creation cost (session tokens, human minutes)
and a realized value (retrieval count, participation in consequential
findings). Nothing measures either. Instrument the read path: orient,
query, and (future) subgraph-context calls log node IDs served. Compute
cost-per-consequential-finding and identify dead weight — nodes never
retrieved after creation.

**Why underserved.** Mary's "read-utility per token" metric was proposed
for the charter only and never built. Generalized, it answers the questions
the project keeps arguing from intuition: is the graph earning its
maintenance cost? Which node types pay rent? Is 92%-derives connective
tissue ever traversed at all?

**Mechanism.** Append-only `_kos/.telemetry/reads.jsonl` (node_id, command,
session, ts); `kos economics` reports retrieval distribution, orphan-read
rate, cost proxies from provenance timestamps. Privacy-trivial: it's all
local.

**First probe.** Instrument for 10 sessions; hypothesis: retrieval follows
a power law and ≥40% of nodes are never read after their creating session.
Either result reshapes pruning and schema priorities.

**Graveyard risk.** Measuring reads changes reading (operators perform
retrieval). Accept it — the bias direction is toward *more* graph use,
which is the desired behavior anyway.

---

## 8. Epistemic API — KOS as an MCP server

**Concept.** Expose the graph read-only over Model Context Protocol:
`kos serve --mcp` offering tools (`orient`, `query_frontier`,
`subgraph_context`, `why`, `chronicle`) to *any* MCP-capable agent —
marvel workers, director, Claude Desktop, foreign tooling. KOS stops being
a repo-local CLI and becomes queryable infrastructure.

**Why underserved.** The roadmap's query layer (Issues 7–9) is
session-and-human facing: CLI invocations inside the repo. The platform's
own architecture (marvel teams, director protocol, F10 cross-repo
continuity) needs agents *elsewhere* to consult the graph mid-task — the
exact "orientation loss" failure F10 documents. An MCP surface is the
missing integration primitive, and it makes the external-operator probe
(Issue 13) trivial to run: hand someone the endpoint.

**Mechanism.** Rust MCP server module wrapping existing workspace loaders;
read-only in v1 (writes stay in the harvest path, preserving the
promotion pawl); serves the Issue-7 subgraph context format verbatim.

**First probe.** One marvel worker mid-task asks `query_frontier(scope=X)`
instead of reading a CLAUDE.md; measure whether it avoids one documented
repeated-settled-question failure (F10's concrete failure mode).

**Graveyard risk.** Premature if Issue 7's format isn't settled — gate on
it. Concurrency reads on YAML are safe; this dodges the substrate question
rather than answering it.

---

## 9. Chronicle — narrative projection of causal history

**Concept.** The graph projects documents; it should also project *story*.
`kos chronicle <node>` walks provenance and edge chains chronologically and
renders the history of an idea as readable narrative with citations: what
question spawned it, what probes tested it, what died so it could live,
who dissented, when it hardened. Durant for a codebase: the graph already
contains the civilization's records; nothing yet writes its history.

**Why underserved.** Onboarding, stakeholder communication, and the
"outside reader cannot reconstruct project state" problem (v2 assessment
F8, Issue 13) are all narrative problems. Every current projection is
structural (charter sections, mermaid graphs, YAML). Humans consolidate
understanding through story; the graph has the raw material and no
renderer. This is also the shadow principle completed: documents,
diagrams, *and narrative* as projections of one source.

**Mechanism.** Deterministic traversal (chronological topo-sort of the
node's ancestry + contradicts/supersedes/dissent events) emitting a
structured event list; optional LLM pass converts events to prose, with
every sentence citing node IDs — grounded generation, hallucination
detectable by citation check.

**First probe.** `kos chronicle elem-storage-model` — the richest lineage
in the graph (session-001 design → grv-git-semantic-layer →
grv-git-as-sufficient-substrate → substrate hypothesis). Hand the output
to a cold reader; measure comprehension against the Issue-13 baseline.

**Graveyard risk.** The LLM prose pass can smuggle unsupported claims;
the citation-per-sentence discipline is the control. If citation coverage
can't reach ~100%, ship the event list and skip the prose.

---

## Cross-cutting note

Three of these (1, 2, 5) share a theme absent from the entire project to
date: **testing the graph's epistemics, not just its yield.** Reliability
grading tests sources, canaries test detection, retrodiction tests the
early-warning claim. Together they would move KOS from "a system that
finds signal" to "a system whose trustworthiness is itself measured" —
which is the difference between an instrument and a methodology.

Suggested first adoptions if forced to pick two: **#5 Retrodiction** (the
highest-credibility experiment available, nearly free via git) and
**#8 MCP server** (converts the platform's biggest documented failure mode,
F10, into a solved integration).
