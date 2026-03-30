# DAG vs RAG: Two Knowledge Topologies

*Idea — pre-hypothesis brainstorming. This is itself an idea, not even a
clearly stated hypothesis yet.*

## The observation

There are two kinds of knowledge in a kos-driven project, and they have
fundamentally different topologies:

**The DAG (our graph)** — internal knowledge. Ideas, questions, findings,
decisions, probes. These spring into existence inside the project. They are
authored, accumulated, promoted, graveyarded. They have confidence tiers.
They have typed edges. They ARE the knowledge graph. Ideas live here too —
they are pre-hypothesis internal generative knowledge, part of the brain's
working memory before it crystallizes into something structured.

**The RAG (external references)** — external knowledge. Papers, docs, blog
posts, API references, standards documents (IEEE 29148, etc.), prior art in
other projects. These are referenced, queried, reranked, cited. They exist
outside the project. We don't own them. We don't promote or graveyard them.
We retrieve them when relevant and cite them when they influence decisions.

## Why this distinction matters

The current kos process doesn't distinguish between these. A finding might
cite an external paper alongside internal evidence, and both look the same
in the content field. But they have different lifecycles:

- Internal knowledge evolves with the project (ideas → questions → findings → bedrock)
- External knowledge is static from our perspective (a paper doesn't change
  because we read it)

- Internal knowledge has confidence tiers (we authored it, we can assess it)
- External knowledge has relevance and authority but not kos-style confidence

- Internal knowledge is traversed by the ripple engine (changes propagate)
- External knowledge doesn't ripple — but a new external reference might
  trigger re-examination of internal knowledge that assumed something the
  external source contradicts

## What this might become

If this idea crystallizes:
- The DAG is the knowledge graph as it exists (nodes, edges, findings)
- The RAG is a retrieval layer over external references
- `_kos/ideas/` is the ideation layer — pre-DAG internal knowledge
- Ideas are part of the brain (DAG-adjacent), not the library (RAG)
- There might be a `_kos/references/` or citation layer that connects
  external knowledge to internal nodes
- The kos CLI might have a `kos cite` or `kos ref` command

## Tensions

- Is this over-engineering? The current approach (just put everything in
  node content fields) works fine at current scale.
- Does the distinction between DAG and RAG map to the B5 shadow principle?
  External docs are projections of someone else's graph. When we reference
  them, we're building cross-graph edges to graphs we don't control.
- The ideas/ directory is already the beginning of this — pre-structured
  internal knowledge that lives outside the typed graph but inside the
  project's knowledge boundary.
- This whole observation is recursive: this idea file is itself an example
  of DAG-side knowledge (internal, generative, pre-hypothesis) about the
  distinction between DAG and RAG knowledge.
