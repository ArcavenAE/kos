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

### Probe and Finding Files
Exploration briefs live in /probes/[brief-slug].yaml
Probe work products (decomposed nodes, test artifacts) live in /probes/[probe-slug]-nodes/
Findings live in /findings/finding-NNN-[slug].yaml (numbered sequentially)
These are separate from /nodes/ — probe artifacts are evidence, not graph nodes.

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
