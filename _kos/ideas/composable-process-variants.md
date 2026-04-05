# Composable Process Variants for kos

*Idea — pre-hypothesis brainstorming. Generative, uncommitted.*

## The observation

The lifecycle composition brief (§E) describes three composition
mechanisms: variants (inherit + override), overlays (attach to any
workflow), and chains (sequence workflows).

kos's process cycle could use the same mechanisms:

## Variants (inherit base, override depth)

```
base-probe:
  steps: [question, brief, investigate, finding, harvest]
  ceremony: full (hypothesis, timebox, success_signal, edges)

feature-probe (inherits base-probe):
  steps: [question, investigate, finding, harvest]
  ceremony: medium (inline hypothesis, FATs as success_signal)
  overrides:
    brief: optional (inline in question node if small)
    finding: abbreviated (commit message + node update)

fix-probe (inherits base-probe):
  steps: [investigate, harvest]
  ceremony: light (commit with kos action vocabulary)
  overrides:
    question: implicit (the bug/issue IS the question)
    brief: skip
    finding: commit message

idea (no probe):
  steps: [brainstorm]
  ceremony: none (markdown file, no structure required)
```

## Overlays (attach additional checks)

- Security overlay: add threat model check to any architecture probe
- Compliance overlay: add regulatory review to any requirement harvest
- Performance overlay: add benchmark to any implementation probe
- Distribution overlay: add release verification to any harvest
  (this is B12 — the harvest checklist that includes distribution
  verification)

## Chains (sequence process cycles)

- Research chain: orient → ideate → question → probe → finding
  → NEW question (derived) → probe → finding → harvest
- Bootstrap chain: orient → bulk-read sources → extract nodes →
  validate → harvest (the seed=sync flow)
- Audit chain: orient → drift → compare sources → findings →
  harvest (the kos-on-existing-project flow)

## What this would look like in practice

Instead of one probe brief template, kos would have:
- `kos probe --scale project` → full brief with all fields
- `kos probe --scale feature` → abbreviated brief
- `kos probe --scale fix` → no brief, just commit convention
- Or: kos infers scale from the question node's scope/complexity

The process subcommands (kos idea, kos question, kos probe, kos
harvest) would scaffold the appropriate ceremony level.

## Tensions

- Is this over-engineering the process? The current uniform ceremony
  works — it's just sometimes heavy for small work. Is that friction
  worth building tooling to eliminate?
- Variants add complexity to the tool. Is the complexity earned?
- Could this be solved without tooling? Just document "for small
  changes, skip the brief" in CLAUDE.md?
