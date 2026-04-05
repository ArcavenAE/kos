# kos as the Lifecycle's Knowledge Layer

*Idea — pre-hypothesis brainstorming. Generative, uncommitted.*

## The observation

The lifecycle composition brief (penny-orc) and kos describe the same
system from different projections:

| Lifecycle Composition | kos |
|---|---|
| LEARN phase (ingest, assess, ripple) | Orient → Drift → Harvest |
| Delivery finding channel (upward flow) | Finding node + edges |
| Change propagation / blast radius | Ripple engine |
| Fractal lifecycle (product/feature/spike) | Process cycle at different scales |
| Context-aware project intelligence | Knowledge graph as retrieval layer |
| Gates (declarative quality checkpoints) | Success signals in probe briefs |
| Workflow variants (inherit + override) | Process ceremony at different depths |
| Multiple input channels (bug/feature/spike/external) | Seed = sync (all inputs are graph updates) |

The lifecycle document describes workflows, agents, gates, and feedback
loops. kos describes the knowledge substrate those workflows operate on.
They're not separate systems — they're different views of the same thing.

## What this might mean

If kos IS the knowledge layer for the lifecycle:
- The universal bridge framework is how kos connects to lifecycle
  workflows (bmad, gitspec, spectacle are all workflow output formats)
- kos orient is the LEARN phase's "ingest & integrate" step
- kos drift is the LEARN phase's "assess drift & impact" step
- kos harvest is the LEARN phase's "ripple changes if adopted" step
- kos's confidence tiers map to lifecycle maturity:
  - bedrock = shipped/established (delivered stories, accepted architecture)
  - frontier = in-progress (active development, open questions)
  - graveyard = ruled out (abandoned approaches, failed spikes)
  - placeholder = scaffolding (not yet real, stub only)

If the lifecycle composition engine is ever built (penny-orc Phase 1-5),
kos could be its persistence and retrieval layer. The engine orchestrates
workflows; kos stores what was learned, tracks what changed, and surfaces
what's relevant.

## The spike lifecycle is already kos

The lifecycle document's spike lifecycle (§D) maps almost perfectly to
kos's probe cycle:

| Spike Lifecycle | kos Process |
|---|---|
| Define (charter, hypothesis, timebox) | Question node + Probe brief |
| Secondary research (literature, prior art) | Orient + RAG references |
| Primary research (prototypes, benchmarks) | Probe work |
| Analyze (comparison, trade-off matrix) | Finding node |
| Present (review, accept/reject/modify) | Human review + confidence assignment |
| Fold back (update parent specs) | Harvest (update nodes, move confidence) |

The spike repo separation principle also maps: spike research artifacts
belong in the spike repo (or _kos/probes/), not in the parent project
context. kos already does this — probe work products live in _kos/probes/
subdirectories, findings live in _kos/findings/, and the graph (nodes)
is what persists.

## Tensions

- Is this mapping too neat? Are we seeing patterns because we want to,
  or because they're structural?
- The lifecycle document is about workflows with agents. kos is about
  knowledge with humans. The overlap might be at the abstract level
  only, not at the implementation level.
- If kos IS the lifecycle's knowledge layer, should kos evolve toward
  lifecycle support (gates, workflow awareness, agent integration)?
  Or should it stay focused on knowledge (nodes, edges, confidence,
  drift) and let the lifecycle engine be a separate tool?
- The lifecycle document was written for penny-orc's context (BMAD,
  Pennyfarthing, BikeLane). kos is in aae-orc. Are the contexts
  compatible or are we conflating two different problem spaces?
