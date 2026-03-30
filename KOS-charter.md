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
(ThreeDoors, penny-orc, axiathon) and nine external open-source projects
(Backstage, Crossplane, Kubernetes KEPs, Rust RFCs, Go Proposals, Python PEPs,
BMAD enterprise, OpenHands, AutoGen). All twelve produced signal at document
seams. Signal types stabilized across the sample: errors (hard contradictions),
evolution (intentional supersession), drift (stale references), and silent
abandonment (missing graveyard entries). See findings 001-013.

*Session-003 extension:* Code-only bootstrap of axiathon (finding-016) showed
code is also a lossy projection — missed 9 of 11 doc-based issues. The shadow
principle is symmetric: documents and code are both shadows of the graph.

**B7: The Review Gap Gradient**
KOS adds the most value where cross-document consistency review is absent.
The strongest predictor of signal yield is not governance quality or artifact
visibility — it is whether anyone checks whether the documents agree *with
each other*. Projects with disciplined RFC/KEP/PEP processes (Kubernetes, Go,
Python) have implicit cross-document review through community feedback and
explicit supersession. Projects with no review process (solo projects, ad-hoc
docs) have the highest signal yield. This is the targeting principle: KOS
should be applied where cross-document consistency checking doesn't exist.

*Session-002 evidence:* Kubernetes KEPs produced 2 issues (both minor gaps).
ThreeDoors produced 10 issues including hard contradictions. Axiathon's
documents passed their own validation ("ALL GAPS RESOLVED") while containing
6 contradictions — the self-validation checked coverage, not consistency.
The gradient is consistent across the twelve-project sample. See findings
001-013.

*Session-004 revision:* "Governance gap" renamed to "review gap" after
twelve-project executive mechanism test (finding-022). The original framing
suggested the predictor was governance quality. It is not — Kubernetes has
excellent governance but low signal because its repo contains polished
records, not the working process. The actual predictor is cross-document
consistency review: does anyone ask "do these documents agree?" Internal
review (axiathon's self-validation) is insufficient. The review must be
cross-document. Note: some repos (Kubernetes, Rust RFCs, Go Proposals)
are meta-projects — the repo contains decisions, not the deliberation
that produced them. Much of the thinking happens in meetings, mailing
lists, and Slack channels that the repo never sees. Low signal yield in
these cases reflects low visibility, not high quality.

**B6: The Cognitive Architecture**
The system is metabolically complementary to LLM inference:
- Graph = long-term memory: typed, structured, confidence-mapped, persistent
- Inference = working memory + reasoning: generative, present-tense, amnesiac
- Correspondence layer = proprioception: the system's sense of its own body
- Ripple engine = attention: notices change, directs resources
- Graveyard = wisdom: accumulated shape of what was tried and where it led
- The executive = the cycle running between graph and inference
- Human = pattern recognition on undeclared structure

Neither inference nor graph alone produces directed understanding. Together,
running the orient-question-probe-harvest-promote loop continuously, they produce
something that behaves like a mind applied to a problem over time.

*Session-003 revision (finding-019):* The graph captures declared structure.
It does not capture the structure of ideas themselves. Pattern recognition
across undeclared conceptual relationships — seeing that findings 015-017
and B3/B5 were all instances of the same recursive projection pattern —
required the human. The agent had all the pieces and didn't connect them.
This is not a temporary limitation. The human's role is permanently the
pattern recognizer operating on the graph's blind spot: conceptual
relationships that exist between nodes but haven't been typed and declared.
Agents execute within declared structure. The human expands the declared
structure by seeing what it doesn't yet contain.

*Session-004 revision (finding-022):* The executive is now specified as an
event-driven priority queue with three input channels: (1) ripple channel
(automatic dirty-flag propagation after harvest), (2) question channel
(open questions ranked by dependency depth × staleness × review gap ×
tractability × adjacent evidence), (3) human channel (highest authority —
meta-epistemological judgment about the investigation itself, not just
the subject). The human channel is not a gate — it is the input that
reasons about the reasoning. "Broaden before deepening" and the recursive
projection pattern are both meta-epistemological: judgments about how to
investigate, not what to investigate. The priority function cannot reach
this level. See finding-022.

---

## Frontier

*Actively open — under exploration, not yet resolved.*

**F1: The dimensional question**
Documents are a projection of the graph. How many dimensions are lost in the
projection is not established. The point is systematic lossiness, not the count.

**F2: The executive mechanism**
The executive loop is now designed but not implemented. It is an event-driven
priority queue with three input channels (ripple, questions, human), convergence
detection across four types (path, cross-source, cross-lens, pattern), and
three work types (discovery, maintenance, synthesis). Validated against twelve
projects (finding-022). Remaining open: implementation, threshold calibration,
and the parallel-vs-sequential document distinction.

*Session-004 update:* Designed and tested across full twelve-project sample.
The mechanism holds — convergence detection fired correctly on 8/12 projects
(0 false positives, 0 false negatives), priority function produced reasonable
rankings for 12/12, maturity model matched 12/12. Three work types identified:
discovery (new structure), maintenance (propagate staleness), synthesis
(consolidate sequential documents that have drifted apart). The priority
function uses review_gap (cross-document consistency checking) not governance
quality or visibility — see B7 revision. See findings 020 (placeholder),
021, 022.

*Session-005 note:* Implementation deprioritized behind tooling probes (F6-F9).
The executive design is validated but building it requires a working graph with
tooling — which doesn't exist yet. Build the substrate first.

**F3: Agent type signatures and contracts**
Described in principle. Not specified: the schema for agent declarations,
the enforcement mechanism, how the precondition/postcondition system works
in practice.

*Session-005 note:* ThreeDoors's agent evaluation framework and label authority
matrix (finding-028) are empirical implementations of agent contracts at one-repo
scale. When this frontier reopens, ThreeDoors is prior art.

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

*Session-003 update:* Code-only bootstrap tested against axiathon (finding-016).
Result: PARTIAL. Code+git produces valid nodes and real signal, but the signal
class is different from document-based probing. Code finds: technology migration
paths, philosophy-implementation gaps, aspirational claims gaps, planning weight
ratios. Code misses: phase boundary erosion, cross-document contradictions about
unimplemented features. Key insight: code-only and doc-only are complementary
lenses, not substitutes. A complete graph needs both sources. Of the three
proposed mechanisms: git conventions showed limited value (too sparse), structural
pattern matching showed high value (Cargo.toml analysis, stub detection), embedding
similarity remains untested.

*Session-005 note:* The RD-to-node bridge probe (brief-rd-bridge, session-008)
is the first automated bootstrap test — extracting findings from prose briefs.

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

*Session-005 note:* `kos orient` (brief-kos-orient, session-006) is the first
tooling answer to F5 — mechanical context surfacing across repos.

*Session-006 update:* `kos orient` built and validated (finding-029). Works
across 8 targets in 3-11ms. The tool's value tracks graph coverage — probed
repos (kos 18, axiathon 13, ThreeDoors 10) get rich output; unprobed repos
(switchboard 0, aclaude 1) get charter items only. Knowledge distribution
across the ecosystem is uneven. External repos work via --workspace flag /
KOS_WORKSPACE env var. Remaining F5 question sharpened: when does the
charter stop being sufficient? orient partially answers — the charter is
sufficient but the graph is sparse where probes haven't run.

**F6: Implementation pivot — from theory to tooling**
Session-005 reviewed the full kos corpus and the aae-orc ecosystem. Central
finding (027): kos's process works through RD vocabulary (ADR-006) but lacks
mechanical connection. The project has spent 4 sessions establishing that it's
right without building anything usable. G3 (waterfall front-loading) predicts
diminishing returns from more theory probes. The evidence base (16 projects,
26 findings, validated schema, designed ripple and executive) is sufficient
to begin building.

Build order, each a probe with a finding:
1. ~~`kos orient` — cross-repo orientation CLI (session-006, brief-kos-orient)~~
   **Complete (session-006).** Finding-029. All 6 success signals met. YAML-in-git
   reads work at current scale (3-11ms). Knowledge distribution uneven across
   ecosystem. External workspace support via --workspace / KOS_WORKSPACE.
   Opt-in usage logging (JSONL) working.
2. `kos validate` + `kos graph` — schema tooling (session-007, brief-schema-tooling)
3. `kos bridge` — RD-to-node extraction (session-008, brief-rd-bridge)
4. `kos drift` — simplest possible ripple (session-009, brief TBD)

**Language: Rust.** kos is a correctness tool — it detects silent corruption
in typed structures. Rust's type system mirrors the kos schema: node types,
edge types, confidence states, and signal classifications become enums with
exhaustive `match`. A forgotten case is a compile error, not a silent bug.
This is the same structural guarantee kos provides to spec documents —
correctness enforced by the system, not by convention.

The aae-orc ecosystem leans Go (marvel, switchboard, ThreeDoors), but kos's
nature differs from those tools. Marvel manages processes; switchboard relays
sessions; kos enforces typed structure. Paying the Rust startup cost now
sidesteps a probable Go-to-Rust refactor after significant investment. Prior
art: axiathon (mature Rust codebase, test patterns, CI/CD), jira-cli (Rust,
Homebrew tap, Apple code signing). Build infrastructure for Rust binaries with
Homebrew and macOS packaging already exists in the portfolio and can be adapted.

Long-term benefits: serde_yaml gives typed deserialization (schema validation
partially free at parse time), petgraph provides typed graph traversal for
drift/ripple, exhaustive pattern matching over edge types prevents the exact
class of silent bugs kos detects in other systems' documents.

Each build tests the YAML-in-git substrate empirically. Where it breaks,
those failures ARE the answer to question-knowledge-layer-requirements.

Evidence: finding-027 (implementation pivot), finding-028 (ThreeDoors patterns).

**F7: Where does YAML-in-git break?**
Revised from the original question-knowledge-layer-requirements (session-005).
The knowledge layer is YAML nodes in git, read by humans and agents via file
I/O. Known pressure points: cross-repo orientation (manual), finding retrieval
(grep works at 76, won't at 200+), edge traversal (no tooling), ripple
propagation (designed, not implemented). The answer comes from building: each
tool in F6 tests a pressure point. Failure modes discovered during building
compose into the requirements list.

*Session-006 data point (finding-029):* `kos orient` reads ~100 artifacts
in 3-11ms. No indexing needed. String matching for relevance filtering is
adequate but coarse (F4: RD brief matching too broad). The substrate is
not the bottleneck at this scale. Remaining pressure points: schema
validation (session-007), edge traversal (session-007 graph), drift
detection (session-009).

See: question-knowledge-layer-requirements (revised), finding-027, finding-029.

**F8: ThreeDoors as empirical kos evidence**
ThreeDoors (2k commits, dark factory, 3 incident reports) has independently
implemented kos patterns: JSONL retrospector (primitive ripple), L0-L4
provenance (primitive confidence), agent failure modes that match kos signal
classes (enforcement gaps, silent abandonment, review gap at agent speed).
These are practical solutions at one-repo scale, not validated principles.
They inform implementation choices — particularly the JSONL output format for
kos tooling.

See: finding-028.

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

**G5: Git as sufficient long-term substrate**
Using git repos as the primary KOS substrate at scale. Ruled out because:
git repos are lossy projections of the knowledge topology — the same
recursive pattern as documents being lossy projections of the spec graph.
The file presentation layer, single-repo boundaries, and human-paced
workflows will become limiting. The required properties (content-addressing,
immutability, cryptographic integrity, distributed sync) are substrate-
agnostic. Git remains valid at current scale. See grv-git-as-sufficient-
substrate for full rationale and reopener.

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
   axiathon issues detectable conditional on decomposition quality. Key insight:
   ripple effectiveness is bounded by probe decomposition quality. See
   finding-018.
3. ~~What does the executive loop look like as software — scheduler, event
   system, agent orchestration?~~ **Partially answered (session-004).**
   The executive is an event-driven priority queue with three input channels
   (ripple, questions, human) and convergence detection that bridges
   maintenance to discovery. Tested against all twelve projects from the
   session-002 sample (findings 020-022). The mechanism holds across the
   full governance gradient. Three work types: discovery, maintenance,
   synthesis (new — consolidating drifted sequential documents). Key
   revision: "governance gap" is actually "review gap" — the predictor
   of signal yield is cross-document consistency review, not governance
   quality. Remaining open: implementation as software, threshold
   calibration, and whether convergence detection produces false
   escalations at scale (tested only on probe data, not live graphs).
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
   on that documents handle naturally?~~ **Partially answered (sessions 003-004).**
   Narrow probe: "what can't be a node?" (finding-017). Two structural limits
   found: negative space (can't represent what was never considered) and
   meta-process patterns (can't capture patterns about the knowing process).
   One pervasive flattening: edges lose epistemological character.
   *Session-004 update (signal-to-noise):* Tested against four projects
   (ThreeDoors with owner ground truth, OpenClaw, Atmos, GitLab). 31 issues,
   45% consequential, 32% noticed/low consequence, 23% noise. The graph's
   value is not finding MORE than humans — it's finding EARLIER. ThreeDoors
   #6 (mood-tracking broken dependency chain) would have been caught before
   an ineffective feature was built. OpenClaw and GitLab issues were
   independently discovered by community through pain — the graph finds
   them structurally before users are affected. Four new signal classes:
   AI configuration seam (multiple AI tool configs contradicting each other),
   analysis-vs-operations divergence, enforcement gaps, authority surface
   fragmentation. The graph misses behavioral gaps that require running
   the software (CLI changes, removed features, edge-case behaviors).
   Remaining angle: redundancy (is there a project type where code and docs
   produce the same signal?). See findings 023-026.
6. ~~What relationship types beyond depends_on does the schema need?~~
   **Answered (session-002).** Four typed edges: derives, implements,
   contradicts, supersedes. Added in schema v0.2. See finding-004.
7. ~~How should the schema handle temporal status vs. epistemic confidence?~~
   **Deferred (session-002).** Temporal state does not need its own field.
   External temporal inconsistencies are contradicts edges. KOS lifecycle
   is derived from graph topology. See finding-004. Reopener documented.
8. At what scale and complexity does the YAML-in-git approach break, and
   what's the minimum additional infrastructure needed? **Partially answered
   (session-006).** Orient reads ~100 artifacts in 3-11ms — no infrastructure
   needed at this scale. Relevance filtering is coarse (string matching) but
   adequate. Usage logging tracks duration_ms to detect degradation
   empirically as the corpus grows. Three pressure points remain untested:
   schema validation (session-007), edge traversal (session-007), drift
   detection (session-009). See finding-029.

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
- **GitHub SpecKit** — SDD toolkit (constitution → spec → plan → tasks); closest to KOS's
  spec-as-product philosophy but linear pipeline, not graph-structured

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
*Established: session-001, updated session-006*

*Session-006 built `kos orient` — the first running kos code. Rust CLI,
all 6 success signals met, 3-11ms per query across 8 targets. YAML-in-git
substrate works at current scale. Knowledge distribution across the
ecosystem is uneven — probed repos get rich output, unprobed repos get
charter items only. External workspace support added (--workspace /
KOS_WORKSPACE). Opt-in usage logging (JSONL) operational. Finding-029
produced.*

*Key session-006 observations:*
- *spike-ocsf-rs1 uses kos process (hypotheses, questions, findings F1-F19,
  charter discipline) but non-standard format — orient can't read it.
  Future: local artifact scanning mode.*
- *RD brief relevance filtering is too broad (string matching body text).
  Refinement: weight title/question/frontier over body.*
- *Knowledge coverage gap is now visible and measurable — orient makes it
  obvious which repos kos has examined and which it hasn't.*

*Session roadmap:*
- *~~Session-006: `kos orient` — COMPLETE. Finding-029.~~*
- *Session-007: Build `kos validate` + `kos graph` — schema validator
  and graph renderer. Probe: brief-schema-tooling. Tests schema v0.3
  mechanically for the first time.*
- *Session-008: Build `kos bridge` — extract RD findings into queryable
  format. Probe: brief-rd-bridge. Tests whether RD briefs compose with
  kos findings into a unified index.*
- *Session-009: Build `kos drift` — simplest ripple (hash, walk derives,
  flag dirty). Probe brief TBD. Tests finding-018's design empirically.*

*Deprioritized: Q3 implementation (executive needs substrate first), Q5
redundancy angle (diminishing returns), threshold calibration (needs live
graph), more probing on the 20-project lineup (evidence base sufficient).*

*Charter priority encoding still under observation (question-charter-priority-
encoding).*
