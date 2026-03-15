# Knowledge Operating System (KOS)
## Project Charter — Re-Introduction Document

> This document is a re-introduction artifact. It captures the current state of
> understanding of a project developed in a single session, sufficient to restore
> context for a collaborator (human or agent) who was present but does not persist.
> It is written in the format the project itself proposes.

---

## The Problem Statement

Every complex system built by humans suffers the same failure: the understanding
that produced it doesn't survive it. Code ships. The reasoning behind it —
failed approaches, load-bearing assumptions, tradeoffs — lives in a few heads
and decays. Documents written before building capture intent but not discovery.
Documents written after are archaeology.

This is not a documentation problem. It is a knowledge accumulation problem.
It compounds when AI agents enter the loop — agents operate on outputs with no
access to the process that produced them.

What's missing is a system that treats **accumulated understanding as the primary
product**.

---

## Design Values

Three principles resolved every design tradeoff in this session:

1. **The spec is the product.** Code is evidence. Understanding is the deliverable.
2. **Uncertainty is structural, not rhetorical.** Confidence must be declared, not performed.
3. **The relationship between understanding and execution must be typed and traversable,
   or it will silently corrupt.**

---

## Non-Goals

- Replacing existing SDD processes (this runs alongside them)
- A better document format
- A project management system
- Solving the problem permanently (this is a moment in time, useful for a time)

---

## Bedrock

*What was established with enough evidence and reasoning to treat as stable.*

**B1: The Spec Graph Model**
Every unit of understanding is a typed node with: identity, type, declared
dependencies, confidence classification, and provenance. Confidence has exactly
three states: BEDROCK, FRONTIER, GRAVEYARD. Nothing is deleted. The graveyard
is append-only and permanent.

*Session-002 note:* Three-state confidence validated as sufficient for epistemic
classification but may conflate epistemic state with temporal state (e.g.,
"completed" is neither frontier nor graveyard). Under examination — see
question-temporal-state. The node model itself was validated across three
project decompositions (43 nodes) with no missing fields or insufficient types.

**B2: The Process Cycle**
A single fractal cycle scales from solo to multi-team without changing shape:
Orient → Question → Probe → Harvest → Promote. Probes are timeboxed. Dead ends
go to graveyard. Bedrock requires evidence, not consensus. The cycle is the
unit, not the sprint or the phase.

**B3: The Storage Model**
Git as substrate (content-addressed, distributed, cryptographically sound) with
a Datomic-style immutable fact store above it. Every fact is a datom:
(entity, attribute, value, transaction-time). History is the primary record.
Current state is a view over it. Git never resolves semantic conflicts — it
detects divergence and surfaces it to the layer that understands the semantics.

**B4: The Correspondence Layer**
Typed, versioned, bidirectional links between spec nodes and code nodes.
Relationship types: implements | tests | explores | violates | extends | obsoletes.
Confidence: declared | inferred | stale. Drift has three signatures: spec ahead
of code, code ahead of spec, symmetric divergence. Each requires different response.

**B5: The Shadow Principle**
Existing SDD documents (PRDs, architecture docs, ADRs, epics) are valid but lossy
projections of the graph — a 2D shadow of an N-dimensional object. The graph
can always generate the documents. The documents cannot recover the graph.
Running in parallel, the graph acts as checksum and diagnostic on the shadow
system. Errors invisible in documents become structurally apparent in the graph —
measurable, not interpretive.

*Session-002 validation:* Tested against three real projects (ThreeDoors,
penny-orc, axiathon). All three had individually-coherent documents hiding
inter-document inconsistencies at seams. Axiathon's documents had passed their
own internal validation ("ALL GAPS RESOLVED") while containing 6 contradictions
and systematic phase boundary erosion. See findings 001-003.

**B6: The Cognitive Architecture**
The system is metabolically complementary to LLM inference:
- Graph = long-term memory: typed, structured, confidence-mapped, persistent
- Inference = working memory + reasoning: generative, present-tense, amnesiac
- Correspondence layer = proprioception: the system's sense of its own body
- Ripple engine = attention: notices change, directs resources
- Graveyard = wisdom: accumulated shape of what was tried and where it led
- The executive = the cycle running between graph and inference

Neither inference nor graph alone produces directed understanding. Together,
running the orient-question-probe-harvest-promote loop continuously, they produce
something that behaves like a mind applied to a problem over time.

---

## Frontier

*Actively open — under exploration, not yet resolved.*

**F1: The dimensional question**
Documents are a projection of the graph. How many dimensions are lost in the
projection is not established. The point is systematic lossiness, not the count.

**F2: The executive mechanism**
The executive loop is described functionally but not architecturally implemented.
What runs it, how it allocates attention, how it maintains coherence of purpose
across time — open.

**F3: Agent type signatures and contracts**
Described in principle. Not specified: the schema for agent declarations,
the enforcement mechanism, how the precondition/postcondition system works
in practice.

**F4: Bootstrap from existing artifacts**
Three mechanisms proposed (git message conventions, embedding similarity,
structural pattern matching) but not implemented or validated. The transition
from low-confidence inferred nodes to declared bedrock needs a practical
workflow.

*Session-002 update:* Manual bootstrap validated — human+agent decomposition of
existing documents into inferred nodes works and produces actionable findings.
Automated mechanisms (git conventions, embedding, pattern matching) remain
untested. The manual workflow is: read documents, identify claims, cast as typed
nodes with depends_on edges, look for contradictions and gaps at seams.

**F5: The collaboration model**
Human provides continuity, direction, judgment, and graveyard maintenance.
Agent provides inference, synthesis, and reasoning within sessions. The
practical tooling for handing context across sessions is unresolved.

*Session-002 update:* Context handoff demonstrated via CLAUDE.md + charter + git
history. Agent restored context without paste-the-charter workflow. Human
judgment shaped probe direction (broad before deep, skepticism about the ten).
Division of labor functioned as designed. Tooling question narrowing: the
repo-as-context-handoff pattern works for Claude Code; the original concern
about cross-session continuity may be solved for this tool specifically.

---

## Graveyard

*Tried, ruled out, permanently recorded.*

**G1: Documents as the primary artifact**
The entire existing SDD ecosystem — PRDs, architecture docs, ADRs — as source
of truth rather than projection. Ruled out because: no dependency modeling,
no confidence classification, no provenance, silent corruption undetectable,
cannot be checksummed against reality. Documents survive as rendering format only.

**G2: Git as the semantic layer**
Using Git's merge and diff semantics for spec conflict resolution. Ruled out
because: Git resolves by text proximity with no concept of type constraints.
A semantically invalid merge with no textual conflict passes silently. Git is
retained as substrate only — content-addressed storage and distribution.

**G3: Waterfall front-loading**
Comprehensive upfront specification before building. Ruled out because: assumes
knowledge is front-loaded, execution is back-loaded. That assumption is broken.
Code is now cheap enough to be a learning instrument. The spec should evolve
from probe findings, not precede them.

**G4: Sequential document pipeline**
Brief → PRD → Architecture → Epics → Stories as a linear flow. Ruled out
because: creates coordination artifacts that serve project managers, not
engineering comprehension. Produces false confidence in early stages and
expensive late-stage discovery of wrong assumptions.

---

## Open Questions

*The most important things we don't know yet.*

1. ~~What is the minimum viable implementation that demonstrates the checksum
   property against an existing SDD process?~~ **Partially answered (session-002).**
   The checksum property works — three projects, three findings, consistent
   categories (contradictions, gaps, structural chaos). Remaining gaps: the
   schema needs edge types beyond depends_on and temporal state handling before
   the graph can fully express what it finds. See findings 001-003.
2. How does the ripple engine surface conflicts to humans without creating
   noise that gets ignored?
3. What does the executive loop look like as software — scheduler, event
   system, agent orchestration?
4. Can the graph bootstrap from a codebase with no existing spec, using only
   code structure and git history?
5. Where does this break — what class of problems does the graph model fail
   on that documents handle naturally? *Session-002 data point:* penny-orc
   showed a case the graph handles awkwardly — a project whose analysis
   contradicts its own practice. The graph can show it but doesn't make it
   automatic. Intent-execution gaps may be a class of problem that requires
   judgment, not structure.
6. What relationship types beyond depends_on does the schema need? See
   question-edge-types. The correspondence layer (B4) already has typed links
   for spec-to-code — do spec-to-spec relationships need the same treatment?
7. How should the schema handle temporal status (planned, in-progress,
   completed, validated) vs. epistemic confidence (bedrock, frontier,
   graveyard)? See question-temporal-state.

---

## Known Prior Art

*Scattered implementations of components — none with the integrated philosophy.*

- **Datomic** — immutable fact accumulation; the right storage model
- **Sourcegraph** — code graph traversal; missing the spec layer
- **IBM DOORS / Jama Connect** — formal requirements tracing; waterfall-shaped, hostile to use
- **Structurizr** — architecture as code with some drift detection
- **Wardley Mapping** — uncertainty-explicit planning; independent arrival at similar ideas
- **Shape Up (Basecamp)** — appetite-bounded, uncertainty-first; closest process analog
- **ADRs (Nygard)** — closest to graveyard entries; widely adopted, rarely done well
- **GraphRAG / LlamaIndex** — graph-structured LLM retrieval; infrastructure without spec semantics

*The integration and the philosophy that connects them is the missing piece.*

---

## Collaboration Model

This project is being developed in collaboration between a human (carrying
continuity, direction, judgment, and graveyard maintenance) and an AI
collaborator (providing inference, synthesis, and reasoning within sessions).

The division of labor is not accidental — it mirrors the architecture being designed.
The human is the graph. The AI is the inference. The session is the executive loop.

The AI does not persist between sessions. This document exists to restore context.
It should be presented at the start of each new session with: "We are building this.
Here is where we are."

---

*Document status: CURRENT*
*Established: session-001, updated session-002*
*Next action: probe question-edge-types and question-temporal-state (schema
revision), informed by three-project data set. Broad evidence gathered;
ready for targeted depth on schema design.*
