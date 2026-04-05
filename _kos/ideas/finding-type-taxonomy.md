# Finding Type Taxonomy for kos

*Idea — pre-hypothesis brainstorming. Generative, uncommitted.*

## The observation

The lifecycle tier definitions introduce structured delivery findings
with a type classification:

```yaml
type: gap | conflict | question | improvement
source: story/phase/person
affected_spec: tier/artifact
description: ...
proposed_action: ...
urgency: blocking | non-blocking
```

kos findings currently have no type beyond "finding." All findings
look the same regardless of what they discovered: a contradiction, a
gap, a new question, or a proposed improvement.

## What this might become

kos findings could adopt a similar taxonomy:

| Finding type | What it captures | Example |
|---|---|---|
| contradiction | Two sources disagree | PRD says REST, code uses WebSocket |
| gap | Something expected is missing | Architecture has no error handling section |
| question | Investigation surfaced a new unknown | Does the parser handle nested structures? |
| improvement | Something works but could be better | Config loading could use env overlays |
| confirmation | Hypothesis validated | The spike lifecycle maps to kos probes |
| redirect | Probe went wrong direction | TypeScript was wrong choice for aclaude |

This maps to kos's existing signal classification (error, evolution,
drift from schema v0.3) but at a higher level — signal type tells you
what kind of change the node represents; finding type tells you what
kind of discovery the finding represents.

## The SCR (Spec Correction Request) pattern

The lifecycle also introduces SCRs — upward-flowing requests to change
specifications. When a delivery finding affects a higher tier, it
becomes an SCR rather than being resolved locally.

In kos terms: when a finding in a subrepo graph affects the orchestrator
charter, it's an SCR. Currently this is handled informally ("update
charter.md if bedrock changed" in the harvest checklist). A typed
finding with affected_spec metadata would formalize the routing.

## Urgency classification

blocking vs non-blocking matters for process flow:
- Blocking findings require acknowledgment within a timeframe
- Non-blocking findings accumulate for periodic review
- Unacknowledged blocking findings should auto-escalate

kos has no urgency concept today. Nodes have confidence but not
priority or urgency. This might be a property of findings specifically,
not all nodes.

## Tension

Is this over-typing? kos currently works with untyped findings and
the content carries the semantics. Adding type/urgency/affected_spec
metadata makes the schema heavier. Is the routing value worth the
ceremony cost? Would the process subcommands (`kos harvest`) make
the ceremony cheap enough?
