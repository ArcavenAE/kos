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

*Session-003 note:* Known representational limits identified (finding-017):
(1) Negative space — the graph cannot represent what was never considered;
the absence of a node is not a node. (2) Meta-process patterns — the graph
holds instances of process events but not patterns about how the graph-building
process works. (3) Edges flatten epistemological character — "derives from
evidence" and "derives from conviction" use the same edge type. These are
declared limits, not bugs. All models have boundaries; these are KOS's.

**B2: The Process Cycle**
A single fractal cycle scales from solo to multi-team without changing shape:
Orient → Question → Probe → Harvest → Promote. Probes are timeboxed. Dead ends
go to graveyard. Bedrock requires evidence, not consensus. The cycle is the
unit, not the sprint or the phase.

**B3: The Storage Model**
A content-addressed immutable fact store with cryptographic integrity and
distributed sync. Every fact is a datom: (entity, attribute, value,
transaction-time). History is the primary record. Current state is a view
over it. The substrate never resolves semantic conflicts — it detects
divergence and surfaces it to the layer that understands the semantics.

*Session-003 revision:* "Git as substrate" graveyarded as a long-term
assumption (grv-git-as-sufficient-substrate). Git remains valid at current
scale but its file presentation layer is a projection — the same kind of
lossy flattening that B5 identifies in documents. The required properties
are content-addressing, immutability, cryptographic integrity, and distributed
sync. Git has these but also imposes single-repo boundaries, file-shaped
content, and human-paced workflows that will become limiting. The recursive
projection pattern: documents flatten the spec graph, git repos flatten the
knowledge topology, the file layer flattens the content-addressed DAG.
Each layer is a human interface that loses structure agents could use directly.

**B4: The Correspondence Layer**
Typed, versioned, bidirectional links between spec nodes and code nodes.
Not optional infrastructure — structurally necessary. Code-derived and
doc-derived nodes produce different signal classes (finding-016); without
the correspondence layer, the graph has two disconnected halves that each
see different problems. The correspondence layer is where the lenses unify.
Relationship types: implements | tests | explores | violates | extends | obsoletes.
Confidence: declared | inferred | stale. Drift has three signatures: spec ahead
of code, code ahead of spec, symmetric divergence. Each requires different response.

**B5: The Shadow Principle**
Existing SDD documents (PRDs, architecture docs, ADRs, epics) are valid but lossy
projections of the graph — a 2D shadow of an N-dimensional object. Code is also
a lossy projection, cast from a different angle. The graph can always generate
the documents; the documents cannot recover the graph. Code carries signal that
documents miss (technology migration paths, philosophy-implementation gaps,
aspirational claims); documents carry signal that code misses (phase boundaries,
cross-document contradictions about unimplemented features). Neither projection
is sufficient. Signal appears at the seams between ALL projections — document
to document, code to code, and code to document. Signal classification (error,
evolution, drift) requires human judgment and declared supersession. The graph
makes signal countable and connected; it does not automatically distinguish
bugs from intended change.

*Session-002 validation (12 projects):* Tested against three personal projects
(ThreeDoors, penny-orc, project-alpha) and nine external open-source projects
(Backstage, Crossplane, Kubernetes KEPs, Rust RFCs, Go Proposals, Python PEPs,
BMAD enterprise, OpenHands, AutoGen). All twelve produced signal at document
seams. Signal types stabilized across the sample: errors (hard contradictions),
evolution (intentional supersession), drift (stale references), and silent
abandonment (missing graveyard entries). See findings 001-013.

*Session-003 extension:* Code-only bootstrap of project-alpha (finding-016) showed
code is also a lossy projection — missed 9 of 11 doc-based issues. The shadow
principle is symmetric: documents and code are both shadows of the graph.

**B7: The Process Quality Gradient**
KOS adds the most value where existing documentation governance is weakest.
Projects with disciplined RFC/KEP/PEP processes (Kubernetes, Go, Python)
already handle most seams through explicit supersession and review. The graph
finds residual cross-feature gaps but the yield is lower. Projects with loose
governance (solo BMAD, ad-hoc docs) have the most to gain — hard contradictions,
phase boundary erosion, and orphaned concepts are common and undetected. This
is not a limitation — it is a targeting principle. KOS should be applied where
existing processes leave the most signal undetected, not where processes are
already strong.

*Evidence:* Kubernetes KEPs produced 2 issues (both minor gaps). ThreeDoors
produced 10 issues including hard contradictions. Project Alpha's documents passed
their own validation ("ALL GAPS RESOLVED") while containing 6 contradictions
and systematic phase boundary erosion. The gradient is consistent across the
twelve-project sample. See findings 001-013.

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

*Session-003 update:* Code-only bootstrap tested against project-alpha (finding-016).
Result: PARTIAL. Code+git produces valid nodes and real signal, but the signal
class is different from document-based probing. Code finds: technology migration
paths, philosophy-implementation gaps, aspirational claims gaps, planning weight
ratios. Code misses: phase boundary erosion, cross-document contradictions about
unimplemented features. Key insight: code-only and doc-only are complementary
lenses, not substitutes. A complete graph needs both sources. Of the three
proposed mechanisms: git conventions showed limited value (too sparse), structural
pattern matching showed high value (Cargo.toml analysis, stub detection), embedding
similarity remains untested.

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

*Session-003 update:* Clean cold-start validated (finding-015). New agent
instance (Opus 4.6, 1M context) restored full project context from repo alone.
Correctly prioritized open questions without human correction. Conditions:
model-dependent, scale-dependent (repo still small), tested for orientation
not deep continuation. Remaining F5 question: when does the charter stop being
sufficient as the compression artifact?

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
2. ~~How does the ripple engine surface conflicts to humans without creating
   noise that gets ignored?~~ **Answered (session-003).** Ripple is maintenance,
   not discovery. It propagates staleness along existing edges using dirty-flag
   propagation with typed attenuation: contradicts edges always propagate,
   derives edges attenuate by distance, supersedes edges stop propagation
   (firebreak). Noise is false staleness, not false conflicts — managed by
   distance attenuation, content hashing, and supersession. Cross-lens
   propagation via correspondence confidence levels. Retrospective test: 10/11
   project-alpha issues detectable conditional on decomposition quality. Key insight:
   ripple effectiveness is bounded by probe decomposition quality. See
   finding-018.
3. What does the executive loop look like as software — scheduler, event
   system, agent orchestration? *Session-003 sharpening:* The executive loop
   must coordinate probes (discovery) and ripple (maintenance). The bridge:
   convergent dirty flags from ripple trigger new probes. This is the most
   important open design question.
4. ~~Can the graph bootstrap from a codebase with no existing spec, using only
   code structure and git history?~~ **Partially answered (session-003).**
   Yes — code-only bootstrap produces valid nodes and real signal, but a
   different class of signal than document-based probing. Code finds technology
   migration paths, philosophy-implementation gaps, aspirational claims.
   Code misses phase boundary erosion and cross-document contradictions about
   unimplemented features. The two are complementary, not substitutes.
   Remaining gap: automated extraction (structural pattern matching showed
   promise, embedding untested). See finding-016.
5. ~~Where does this break — what class of problems does the graph model fail
   on that documents handle naturally?~~ **Partially answered (session-003).**
   Narrow probe: "what can't be a node?" (finding-017). Two structural limits
   found: negative space (can't represent what was never considered) and
   meta-process patterns (can't capture patterns about the knowing process).
   One pervasive flattening: edges lose epistemological character. Remaining
   angles untested: signal-to-noise ratio (do the issues the graph finds
   matter?) and redundancy (is there a project type where code and docs
   produce the same signal?). *Session-002 data point:* penny-orc showed
   intent-execution gaps require judgment, not structure — consistent with
   the edge-flattening finding.
6. ~~What relationship types beyond depends_on does the schema need?~~
   **Answered (session-002).** Four typed edges: derives, implements,
   contradicts, supersedes. Added in schema v0.2. See finding-004.
7. ~~How should the schema handle temporal status vs. epistemic confidence?~~
   **Deferred (session-002).** Temporal state does not need its own field.
   External temporal inconsistencies are contradicts edges. KOS lifecycle
   is derived from graph topology. See finding-004. Reopener documented.

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
*Established: session-001, updated session-003*
*Next action: Session-003 answered or partially answered Q2, Q4, Q5. Q2 answered:
ripple is maintenance not discovery (finding-018). Q4 partial: complementary
lenses (finding-016). Q5 partial: two structural limits, one flattening
(finding-017). Cold-start validated (finding-015). Remaining: Q3 (executive
mechanism — now sharpened as coordinator between probes and ripple), untested
Q5 angles (signal-to-noise, redundancy). Charter priority encoding under
observation (question-charter-priority-encoding).*
