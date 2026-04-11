# KOS — Knowledge Operating System

> Note: The "aclaude" project has been renamed to "forestage."
> Historical references in findings may use the old name.

## What This Is
A graph-based knowledge accumulation system for designed systems.
Rust CLI tool + knowledge graph in YAML over git.
See KOS-charter.md for full context and re-introduction.
See docs/product-brief.md for the CLI tool overview.

@.claude/rules/rust.md
@.claude/rules/git-commits.md
@.claude/rules/bash.md

## Build / Run / Test

Requires: Rust 1.85+ (Edition 2024), `just`, nightly rustfmt.

```sh
just build          # cargo build
just test           # cargo test
just check          # fmt + clippy + deny (pre-commit mirror)
just ci             # full CI mirror (fmt, clippy, build, deny, test, doc-test)
just fmt            # cargo +nightly fmt --all
just run orient     # run orient subcommand
```

## How to Work Here

### Re-introduction
Read KOS-charter.md before any spec work. It contains:
- Current bedrock (what's committed)
- Current frontier (what's under exploration)
- Current graveyard (what's been ruled out)
- Open questions (the current work queue)

### Node Files
Nodes live in _kos/nodes/[confidence]/[id].yaml
Schema is in /schema/node.schema.yaml
One node per file. Filename = node id.

### Confidence Changes
Moving a file between confidence directories IS the promotion.
Always accompany a move with a commit message explaining the evidence.

### Probe and Finding Files
Exploration briefs live in _kos/probes/[brief-slug].yaml
Probe work products (decomposed nodes, test artifacts) live in _kos/probes/[probe-slug]-nodes/
Findings live in _kos/findings/finding-NNN-[slug].yaml (numbered sequentially)
These are separate from _kos/nodes/ — probe artifacts are evidence, not graph nodes.

### Ideas (pre-hypothesis brainstorming)
Ideas live in _kos/ideas/ as markdown files. They are generative, possibly
contradictory, and carry no commitment — the ideation stage between Orient
and Question. Ideas are part of the graph's memory (the DAG): internal
knowledge that springs into existence. Contrast with external research
(RAG): referenced, queried, reranked — external information we cite but
don't own.

When an idea crystallizes into something testable, extract it into a
frontier question node and a probe brief. The idea file gets a note
pointing to what it became.

### Session Protocol
1. Read KOS-charter.md
2. Identify the highest-value open question — or capture new ideas in _kos/ideas/
3. Write an Exploration Brief in _kos/probes/
4. Do the probe work
5. Write a finding in _kos/findings/
6. Harvest: update affected nodes, move files if confidence changed
7. Update KOS-charter.md if bedrock changed significantly

### Harvest Verification (before starting the next cycle)
# Arose: session-002, cycle boundary drift
# Limitation: manual prompt only. Automated detection of skipped
# backward-facing artifacts is a tooling concern, not a CLAUDE.md concern.
# Expected evolution: checklist items will be revised as signal/noise
# ratio becomes clearer across sessions.

When a session runs multiple probe cycles, verify harvest is complete
before starting the next cycle. The gap between cycles is where drift
accumulates — forward-facing artifacts (findings, charter, schema) get
updated, backward-facing ones (stale questions, missing briefs) get
skipped.

- [ ] Finding node written and committed
- [ ] Schema changes committed if implied by finding
- [ ] Charter updated if bedrock changed
- [ ] Frontier questions updated (closed, opened, or revised)
- [ ] Exploration briefs marked complete or carried forward

This is a prompt, not a gate. Missing one is drift, not failure.

### Commit Convention
[action]: [node-ids affected] — [one line description]

Actions: harvest | promote | graveyard | probe | finding | schema | charter
Examples:
  harvest: question-mvg-bootstrap — first probe complete, see finding-001
  promote: elem-node-schema — validated under session-001 use
  graveyard: grv-git-semantic — ruled out, see graveyard entry
