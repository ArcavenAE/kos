# ADR-009: Read Surface as a Single Query Engine With Three Faces

## Status

Accepted

## Date

2026-08-16

## Context

kos has a working write path and almost no read path. The adopted vision
names this Gap 0 and calls Query the weakest verb of the operating loop.
Two candidate homes for the read surface were live at once: a kos-side
MCP server, and flyloft as the retrieval substrate fronting kos. bd
`aae-orc-di23` asked which one to build, with the criterion "soonest
surface, federation not foreclosed."

Two constraints bound the answer.

First, flyloft is a skeleton: types only, every verb `todo!()`. Waiting
for it means shipping no read surface at all.

Second, the Track B sequencing in bd `aae-orc-b0kp` (round 3, item 2)
puts the M0 schema repair before any MCP surface. The argument is that a
server over a corrupted graph "serves lies with a nice API"; finding-060
measured roughly 24 percent undeclared-type edges while validation was
green, so the corruption is real and invisible to the current checks.

## Considered Options

1. One query engine, exposed through successive faces (chosen).
2. Two permanent parallel servers: a kos MCP and a flyloft MCP as
   co-equal read surfaces.
3. MCP first, before any CLI verb.

## Decision Outcome

**One query engine, three faces over time.**

- **Now:** the `kos ask` CLI verb (bd `aae-orc-jajn3`), lexical plus
  graph-proximity as the interim ranking. The verb is the contract.
- **Next:** a kos-side read-only MCP Phase 0 wrapping the same engine,
  after the M0 schema repair, per `aae-orc-b0kp` round 3 item 2.
- **Later:** flyloft-mcp federation when flyloft is real, fronting the
  same engine rather than replacing it.

This satisfies di23's criterion directly: the CLI verb is the soonest
surface available, and nothing in it forecloses federation, because the
engine is the asset and the faces are thin.

### Why option 2 was rejected

Two permanent co-equal servers means two ranking implementations and two
provenance stories over the same graph. When the same question returns
different answers with different confidence framing depending on which
server was asked, neither answer can be trusted, and the failure is
quiet. Maintenance doubles for a benefit that federation over one engine
already provides.

### Why option 3 was rejected

Per `aae-orc-b0kp`: M0 before MCP. An MCP surface is the point where the
graph stops being read by people who can notice it is wrong and starts
being consumed by agents that cannot. Shipping the server first inverts
the order in which the corruption would surface.

### Positive Consequences

- A usable read surface lands without waiting on flyloft or on M0.
- One ranking and provenance implementation to test, measure, and
  correct.
- Federation stays open; flyloft inherits an engine with measured
  behavior instead of a specification.
- Read telemetry (bd `aae-orc-2qlx0`) measures one engine's circulation,
  so the numbers stay comparable across faces.

### Negative Consequences

- The CLI verb sits outside the agent task loop, so early usage
  measurement under-counts in-task retrieval. This is held open, not
  waved away, as `question-read-surface-sequencing`.
- kos carries a query engine it may eventually hand to flyloft. The
  standing guard from the read-path idea applies: the verb must not grow
  into flyloft-inside-kos.

## Links

- `kos/docs/proposals/read-path-first-steps.md` (the plan this decision
  serves)
- bd `aae-orc-di23` (the question), `aae-orc-b0kp` (Track B sequencing),
  `aae-orc-jajn3` (`kos ask` Phase 0), `aae-orc-2qlx0` (read telemetry)
- `_kos/ideas/kos-read-path-gap.md` (direction 2: the verb is the
  contract, the substrate swaps)
- `kos/_kos/nodes/frontier/question-read-surface-sequencing.yaml`
- vision.md, Gap 0 and the operating loop
- finding-060 (undeclared-type edges under green validation)
