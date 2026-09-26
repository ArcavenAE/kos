# kos as Universal Knowledge Intermediary

*Idea — pre-hypothesis brainstorming. Generative, uncommitted.*

## The observation

The spectacle bridge (F4) is one instance of a general pattern: kos
needs bidirectional bridges to *every* format where project knowledge
lives. Not just spectacle — bmad, gitspec, code, git history, PRs,
issues, ADRs, meeting notes, Jira tickets, Slack threads, Obsidian
vaults, PDFs, email.

kos is always the superset. It learns from everything it touches.

## Three operations per bridge

Every format bridge has three modes:

1. **Project out** — generate/update documents in the target format
   from kos graph state. kos → spectacle template, kos → bmad docs,
   kos → gitspec.

2. **Read in / seed** — extract knowledge from existing documents into
   kos nodes. Existing bmad docs → kos, existing gitspec → kos,
   codebase + git log → kos.

3. **Sync / true-up** — assess drift between kos graph and the source
   material. What changed in code that kos doesn't know? What's in
   the spec that the code contradicts? This is the "node-ification"
   of information encoded elsewhere.

## The continuum insight

Seeding is not a separate process from steady-state use. It's the
same sync operation where kos happens to be very far behind.

- kos never used in project → seed is full brownfield onboarding
- kos slightly behind → sync catches up recent changes
- kos current → sync confirms or detects drift

The difference is degree, not kind. A project where kos was never
used is just one where the graph is empty and the code/docs evolved
without it. The sync process is the same — it just has more to
discover.

## Bridge combinatorics

kos as intermediary enables format-to-format projection:

```
bmad (existing) → kos → clean bmad (exported)
bmad (existing) → kos → IEEE spectacle docs (generated)
gitspec (existing) → kos → bmad (updated)
code + git → kos → spectacle SRS (generated)
random ADRs + diagrams → kos → coherent bmad or spectacle
```

Always: `source → kos → target`. kos is the canonical graph.
The source and target can be the same format (clean up existing
bmad) or different (translate bmad to spectacle).

## Source diversity

Where documents live varies wildly:

**Common cases:**
- Documents in the same repo as code
- Documents in a separate repo/folder from code
- Documents in an -orc repo, overlapping multiple projects
- No documents at all — just code, git history, PRs, issues,
  scattered ADRs, diagrams, or nothing

**The "no docs" case is critical.** Many projects have no spec
documents. Their knowledge lives in code structure, commit
messages, PR descriptions, issue threads, and tribal knowledge.
A brownfield onboarding might start from: "it's basically tmux +
k8s but in Rust/Go, runs REPLs instead of containers, orchestrates
agents with their own definitions, has a supervisor pattern, a
message queue, remote terminal access, a Slackbot, monitors GH
issues..." — a verbal description that references other known
systems. kos needs to work with this.

**Less common (at first):**
- Jira, Linear, Word docs, Google Docs, PDF product briefs
- Gantt charts, email threads, Slack conversations
- Meeting notes, transcripts, Obsidian vaults

## Multi-topic sources

Human sources are messy. A meeting transcript discusses twelve
topics. A Slack thread weaves between three projects. An email
chain covers a bug, a feature request, and vacation plans.

kos needs to:
- Point at a source that is NOT dedicated to one topic
- Extract the subset relevant to a specific project/graph
- Ignore or park knowledge about other projects that has no
  home in the current graph

This means kos must handle the case where it encounters
information about Project B while indexing a source for
Project A. Options: discard it, tag it as out-of-scope,
route it to Project B's graph if accessible, or surface
it as an unplaced observation.

## What this implies for kos

1. **Bridge as a first-class concept.** Not just spectacle-bridge
   but a bridge framework. Each format adapter implements read-in,
   project-out, and sync.

2. **Source registry.** kos needs to know where to look — a manifest
   of sources (repos, folders, URLs, API endpoints) and what format
   each is in.

3. **Incremental sync.** Full re-scan is expensive. kos should track
   what it's seen (last commit hash, last modified date, last sync
   timestamp) and process deltas.

4. **Confidence from provenance.** Knowledge extracted from a formal
   SRS has different confidence than knowledge inferred from a commit
   message. The bridge should tag provenance so the graph knows how
   much to trust each node.

5. **The graph is always richer than any single projection.** A
   spectacle SRS shows requirements. A bmad PRD shows product intent.
   kos holds both plus the edges between them plus the evidence trail.
   No single projection captures everything.

## Relationship to existing ideas

- **DAG vs RAG** — external sources (PDFs, papers, meeting transcripts)
  are RAG-side until kos internalizes their relevant content as nodes.
  The bridge is what crosses that boundary.

- **Content-addressed store** — cross-repo federation becomes essential
  when kos graphs span multiple repos and sources. The bridge needs
  to reference facts that live elsewhere.

- **Prediction engine (F19)** — active knowledge surfacing could use
  bridge sync as a signal: "code changed in ways that contradict
  three kos nodes" is exactly the kind of thing the prediction
  engine should surface.

## Tensions

- How much format-specific knowledge does kos need? Each bridge is
  a parser, a mapper, and a renderer. That's substantial per format.
- Is the LLM the bridge? An LLM can read any format and extract
  structured knowledge. kos might not need format-specific parsers
  if it can prompt an LLM with "read this bmad PRD and extract
  nodes in kos schema." The LLM IS the universal parser.
- Scale: a full Slack workspace or Jira project is enormous. What
  scopes the intake?
- Identity: when the same concept appears in bmad, in code, and in
  a Jira ticket, how does kos recognize it's the same thing?
