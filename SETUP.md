# KOS Phase 2 Setup Guide
# Moving from Claude Chat to a persistent repo

---

## What You Have Now

Three files from session-001:

- `KOS-charter.md`      — re-introduction document (paste at start of each session)
- `schema/node.schema.yaml`  — node schema v0.1
- `nodes/seed.yaml`     — charter recast as typed nodes

These are the founding artifacts. Phase 2 makes them persistent,
version-controlled, and accessible to Claude Code.

---

## Step 1: Create the Repo

```bash
mkdir kos
cd kos
git init
git branch -M main
```

Create on GitHub (or Gitea, wherever you work):
```bash
gh repo create kos --private
git remote add origin git@github.com:[you]/kos.git
```

---

## Step 2: Directory Structure

```
kos/
├── README.md                  # One paragraph. What this is.
├── KOS-charter.md             # Re-introduction doc. Update each session.
├── schema/
│   └── node.schema.yaml       # Node schema. FRONTIER until validated.
├── nodes/
│   ├── bedrock/               # Committed nodes. Stable.
│   ├── frontier/              # Active hypotheses. In flux.
│   ├── graveyard/             # Closed. Append-only. Never delete.
│   └── placeholder/           # Known unknowns. Honest markers.
├── sessions/
│   └── session-001.md         # Full conversation reconstruction.
├── probes/                    # Exploration briefs, one per probe.
└── findings/                  # Harvested findings, one per probe.
```

Moving a node file between confidence directories IS the confidence change.
Git history of that move IS the promotion/demotion record.

---

## Step 3: Seed the Repo

Copy files from this session:

```bash
# From your downloads
cp KOS-charter.md kos/
cp node.schema.yaml kos/schema/
cp KOS-session-001.md kos/sessions/session-001.md

# Split seed.yaml into individual node files
# One file per node, named by id
# Place in appropriate confidence directory

# Bedrock nodes:
cp [id].yaml kos/nodes/bedrock/

# Frontier nodes (questions):
cp [id].yaml kos/nodes/frontier/

# Graveyard nodes:
cp [id].yaml kos/nodes/graveyard/
```

Or keep seed.yaml whole for now — split into individual files
when you have tooling to do it automatically.

---

## Step 4: Initial Commit

```bash
cd kos
git add .
git commit -m "session-001: founding charter, schema v0.1, seed nodes

Establishes:
- Charter Kernel (bedrock)
- Node schema v0.1 (frontier — validate under use)
- 6 bedrock elements
- 3 graveyard entries
- 5 open questions

Next: validate schema under real use, begin question-mvg-bootstrap probe"
```

Commit messages are harvest records. They should reference node ids
and spec cycle actions. This is the beginning of the provenance chain.

---

## Step 5: Claude Code Setup

Install Claude Code if not already:
```bash
npm install -g @anthropic/claude-code
```

In the repo root, create a CLAUDE.md:

```markdown
# KOS — Knowledge Operating System

## What This Is
A graph-based knowledge accumulation system for designed systems.
See KOS-charter.md for full context and re-introduction.

## How to Work Here

### Re-introduction
Read KOS-charter.md before any spec work. It contains:
- Current bedrock (what's committed)
- Current frontier (what's under exploration)
- Current graveyard (what's been ruled out)
- Open questions (the current work queue)

### Node Files
Nodes live in /nodes/[confidence]/[id].yaml
Schema is in /schema/node.schema.yaml
One node per file. Filename = node id.

### Confidence Changes
Moving a file between confidence directories IS the promotion.
Always accompany a move with a commit message explaining the evidence.

### Session Protocol
1. Read KOS-charter.md
2. Identify the highest-value open question
3. Write an Exploration Brief in /probes/
4. Do the probe work
5. Write a finding in /findings/
6. Harvest: update affected nodes, move files if confidence changed
7. Update KOS-charter.md if bedrock changed significantly

### Commit Convention
[action]: [node-ids affected] — [one line description]

Actions: harvest | promote | graveyard | probe | finding | schema | charter
Examples:
  harvest: question-mvg-bootstrap — first probe complete, see finding-001
  promote: elem-node-schema — validated under session-001 use
  graveyard: grv-git-semantic — ruled out, see graveyard entry
```

---

## Step 6: How Claude Code Sessions Work

Claude Code reads the repo. The repo IS the context.
You don't paste the charter — Claude Code reads it from the file.

Start each Claude Code session:
```bash
cd kos
claude
```

Claude Code will have access to all node files, the schema, the charter,
and the full git history. The repo solves the continuity problem.

For heavy reasoning sessions (schema design, synthesis, architecture):
use Claude Chat with the charter pasted as re-introduction.

For repo work (writing nodes, harvesting findings, moving files):
use Claude Code directly in the repo.

---

## The Next Probe

When the repo is set up, the first probe is:

**question-mvg-bootstrap**: What is the minimum viable graph?

Exploration Brief should specify:
- Hypothesis: a flat YAML node store with git as substrate,
  no additional tooling, is sufficient to demonstrate the
  checksum property against one existing SDD document
- Excluded scope: ripple engine, agent contracts, correspondence
  layer — those come later
- Success signal: we can ingest one existing document, cast it
  as inferred nodes, and identify at least one inconsistency
  that is invisible in the document but visible in the graph
- Timebox: one session

That probe tells us whether the schema holds and whether the
checksum property is real before we invest in infrastructure.

---

## What Comes After

Only after the bootstrap probe validates the schema:

1. Simple CLI tooling: validate node files against schema
2. Dependency graph rendering: visualize the node graph
3. Drift detection: compare node hashes across git history
4. Document ingestion: parse existing docs into inferred nodes
5. Correspondence layer: link spec nodes to code nodes
6. Agent contracts: typed agent registry
7. Ripple engine: automated dirty propagation

Each of these is a probe with a finding. None of them get built
until the previous one validates.

---

## The Rule

Build nothing you haven't proven you need.
Prove nothing you haven't questioned first.
Question nothing without a timebox.
The spec is the product.
