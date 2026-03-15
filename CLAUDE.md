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

### Harvest Verification (before starting the next cycle)
When a session runs multiple probe cycles, verify harvest is complete
before starting the next cycle. The gap between cycles is where drift
accumulates — forward-facing artifacts (findings, charter, schema) get
updated, backward-facing ones (stale questions, missing briefs) get
skipped.

Check:
- Frontier questions touched by this probe: updated with result?
- Answered questions: annotated or moved?
- Brief exists for this probe? (Write retroactively if skipped.)
- Charter next-action still reflects reality?
- Schema version matches committed changes?

This is a prompt, not a gate. Missing one is drift, not failure.
The checklist exists because session-002 ran four cycles and
accumulated harvest gaps at each transition. See finding-014.

### Commit Convention
[action]: [node-ids affected] — [one line description]

Actions: harvest | promote | graveyard | probe | finding | schema | charter
Examples:
  harvest: question-mvg-bootstrap — first probe complete, see finding-001
  promote: elem-node-schema — validated under session-001 use
  graveyard: grv-git-semantic — ruled out, see graveyard entry
