# Beads as prior art for kos substrate and lifecycle evolution

*Idea — pre-hypothesis brainstorming. Generative, uncommitted.*

## The observation

Beads (gastownhall/beads, 20k stars, Go) is a "memory upgrade for
coding agents" — an issue/work tracker built on Dolt (version-controlled
SQL). It solves different problems than kos (work tracking vs knowledge
tracking) but has already answered several questions kos is still asking.

## What beads has that kos should examine

### 1. Substrate: Dolt (version-controlled SQL)
Beads chose a database with git semantics — cell-level 3-way merge,
branch/commit history, native sync via push/pull. This is the answer
to kos's substrate hypothesis (finding-036): "what is the minimum
viable non-file substrate?" Dolt demonstrates that version-controlled
structured storage is viable for this class of tool. kos's five
required properties (first-class relationships, preserved reasoning
chains, on-demand projections, context surviving session breaks,
concurrent transaction semantics) — Dolt provides all five.

Not saying kos should use Dolt. Saying Dolt proves the substrate
class is viable and someone shipped it.

### 2. Compaction / semantic memory decay
Old closed issues are AI-summarized and shrunk: 70% reduction at
30 days, 95% at 90 days. The insight: sustainable long-term knowledge
requires deliberate, graceful information loss. Not deletion — lossy
compression that preserves conclusions while shedding evidence.

kos has graveyard (ruled-out ideas preserved) but no mechanism for
"this finding's conclusions are absorbed into bedrock, the full
evidence trail is no longer needed at full fidelity." Finding-041
(charter inflation at 51KB) is the first symptom. The graph will
grow. Without compaction, every kos graph eventually hits context
ceilings.

### 3. discovered-from edge type
"While working on X, I discovered Y needs doing." This is the pattern
that generates work faster than it can be tracked. Every kos probe
generates child questions — that lineage is implicit today (mentioned
in content, maybe a derives edge). Making it first-class would let
the graph answer: "what questions emerged from this probe?" and trace
the generative chain.

### 4. Gate/await primitives
Wait for CI, PR merge, human approval, timer. Structural waits, not
just "blocked." kos probes get blocked on external events regularly
(session-010: waiting for Linux tester). No way to encode this in
the graph today.

### 5. Ready-work computation (bd ready)
Shows only unblocked work with resolved dependencies. kos orient shows
the graph state but doesn't compute actionability. "Which frontier
questions have their prerequisite findings resolved and no external
blockers?" — kos can't answer this mechanically.

### 6. Doctor cross-referencing git
bd doctor checks whether work happened in git that wasn't recorded in
the tracker. That's drift detection across the code-knowledge boundary
— exactly kos's correspondence layer, already implemented by beads
for the work domain.

### 7. Spike type = kos probe
Required fields: Goal ("What question does this spike answer?") and
Findings ("What was learned?"). Structurally identical to probe/finding.
Beads just doesn't have confidence tiers on top.

### 8. Pinned beads = sticky context
Persistent context markers that stay open indefinitely and aren't work
items. Maps to kos bedrock but explicit: "always surface this to agents."
kos orient could have a "pinned" concept — nodes that always appear
regardless of target filtering.

## The meta-observation

kos tracks what we KNOW. Beads tracks what we need to DO. But kos's
frontier questions ARE work items — they need probing, they have
dependencies, they can be blocked, they generate child questions.
The boundary between knowledge tracking and work tracking is blurrier
than kos's non-goal ("not a project management system") suggests.

Beads solved the work side with rich graph edges, compaction, gates,
and ready-work computation. kos solved the knowledge side with
confidence tiers, typed nodes, and the probe/finding cycle. Neither
system has what the other has.

## What to probe

1. Can kos add a `discovered-from` or `spawned-by` edge type without
   becoming a task tracker? (Probably yes — it's knowledge lineage.)
2. What would compaction look like for kos? Summarize old findings?
   Collapse resolved question chains? Archive fully-promoted bedrock
   evidence trails?
3. Should kos orient compute readiness (unblocked frontier questions)?
4. Is Dolt (or a Dolt-like substrate) worth evaluating for the
   substrate question? Or is the lesson "version-controlled structured
   storage" at a more abstract level?
5. Can beads' gate/await model be adapted for kos probes that are
   waiting on external events?

## Tensions

- kos is explicitly "not a project management system." How much of
  beads' work-tracking model can kos absorb without becoming one?
- Compaction requires judgment about what to keep. Who decides? The
  human? The LLM? An algorithm? kos's append-only graveyard is
  deliberately judgment-free — everything is kept.
- Dolt is a Go library. kos is Rust. Integration would require FFI
  or a sidecar process. The substrate question may be better answered
  by a Rust-native alternative (sled, redb, sqlite + custom VCS).
