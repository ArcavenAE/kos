# The Eight Lifecycle Problems as kos's Roadmap

*Idea — pre-hypothesis brainstorming. Generative, uncommitted.*

## The observation

The lifecycle composition brief identifies eight structural problems
with BMAD's linear pipeline. Each one maps to a kos capability that
either exists, is frontier, or is unbuilt. Reading the eight problems
as a roadmap for kos:

### 1. Lifecycle stops at "shipped" → kos IS the feedback loop
The LEARN phase (ingest → assess → ripple) maps to Orient → Drift →
Harvest. kos already does this for its own graph. The bridge framework
extends it to other projects.
**Status:** Partially built (orient, drift exist). Bridge is idea-stage.

### 2. Only works at one scale → fractal process ceremony
The process needs variants at product/feature/fix/idea scale.
**Status:** Question (question-fractal-process-ceremony). Unbuilt.

### 3a. Execution shape is hub-and-spoke → cross-repo knowledge
Parallel work tracks diverge silently. kos's cross-repo graph (F10)
addresses this — nodes in different repos with edges between them.
**Status:** Partially built (distributed graph, _kos/ convention).
The edge traversal across repos is not automated.

### 3b. Human is integration bus → gathering layer
The human carries context between sessions. kos orient partially
automates this. The gathering-vs-interpreting question asks whether
orient should go further.
**Status:** Question (question-orient-gathering-vs-interpreting).

### 3c. Single workflow type → composable process variants
Different activities need different structures. The process needs
stepped (planning), phased (implementation), procedural (review).
**Status:** Idea (composable-process-variants.md). Unbuilt.

### 4. No post-1.0 answers → multiple input channels (seed=sync)
Bug reports, feature requests, dependency changes, tech debt, user
feedback, direction changes — all need lifecycle entry points. The
seed=sync insight says these are all the same operation (graph update)
at different distances from current state.
**Status:** Idea (universal-bridge-intermediary.md). The seed=sync
insight is bedrock-worthy but not yet promoted.

### 5. Implementation drift invisible → delivery finding channel
Code diverges from spec silently. kos drift detects content changes
but not code-to-graph divergence. The bridge framework would close
this gap.
**Status:** Question (question-delivery-finding-channel). Unbuilt.

### 6. Changes ripple untracked → ripple engine
When a node changes, dependents need reassessment. kos drift does
basic content-hash ripple. The lifecycle document adds blast zone
classification (delivered/in-progress/future) and propagation
taxonomy (extend/integrate/deprecate/target).
**Status:** kos drift exists but is basic. Propagation taxonomy is
bedrock (elem-drift-propagation-taxonomy) but not implemented.

### 7. Process not composable → variants, overlays, chains
Improvements to base don't benefit variants. kos process changes
(new ceremony, new fields) don't automatically apply to reduced-
ceremony variants because no inheritance exists.
**Status:** Idea (composable-process-variants.md). Unbuilt.

### 8. Context doesn't fit → knowledge graph as retrieval
Flat-file loading hits ceiling. The graph IS the retrieval layer.
**Status:** Graveyarded as scaling strategy (grv-flat-file-context-
scaling). kos orient is the first retrieval mechanism. The gathering-
vs-interpreting question asks whether it should be the primary one.

## The roadmap reading

If you order these by what kos already has:
1. **Built:** Orient, drift, distributed graph, confidence tiers
2. **Question-stage:** Gathering layer, fractal ceremony, gates, delivery finding
3. **Idea-stage:** Bridge framework, composable variants, process subcommands
4. **Unbuilt:** Cross-projection integrity, automated code-graph link, blast zone classification

The lifecycle document is a 10,000-word requirements specification
for what kos needs to become. It just wasn't written for kos.
