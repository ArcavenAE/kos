# kos CLI — Product Brief

## What this tool is

A single-binary Rust CLI for knowledge graph operations over YAML-in-git.
kos reads typed YAML nodes (with declared confidence, typed edges, and
provenance), validates them against a schema, renders the graph, surfaces
relevant knowledge across repos, and detects drift.

kos-the-tool is the mechanical arm of kos-the-system. The system is
described in KOS-charter.md. The tool makes the system's operations
possible without manual YAML reading and mental graph traversal.

## Who it serves

1. **Developers in the aae-orc ecosystem** who need to know what's relevant
   when starting work in any subrepo. Today this requires reading the
   charter, RD briefs, and multiple CLAUDE.md files. kos surfaces the
   relevant subset.

2. **Agents** that need structured context for a repo. The `--json` flag
   outputs JSONL for machine consumption, following the pattern established
   by ThreeDoors's retrospector.

3. **The kos project itself** — schema validation catches drift between
   the schema-as-documented and nodes-as-written. Graph rendering makes
   the topology visible. Drift detection implements the ripple engine
   that was designed in finding-018 but never built.

## Subcommands

### `kos orient [target]`

Surface relevant knowledge for a repo. Reads the aae-orc charter, RD
briefs, kos findings, and frontier questions. Filters by target repo
(inferred from cwd or specified). Outputs human-readable text (default)
or JSONL (`--json`).

Serves: charter F10 (cross-repo knowledge continuity), marvel F-08.
Probe: brief-kos-orient. Session-006.

### `kos validate`

Validate all nodes against the schema. Checks: required fields, confidence
values, edge targets exist, filename matches node ID, file in correct
confidence directory. Reports pass/fail per node.

First mechanical test of schema v0.3 (revised three times, never tooled).
Probe: brief-schema-tooling. Session-007.

### `kos graph`

Render the node/edge graph as mermaid (default) or dot (`--format dot`).
Nodes colored by confidence tier. Edge labels show type and signal. Makes
the 25-node graph visible as a graph for the first time.

Probe: brief-schema-tooling. Session-007.

### `kos bridge`

Extract findings from aae-orc RD briefs (sprint/rd/*.md) into structured
YAML or JSONL. Connects the RD process (ADR-006) to the kos graph
mechanically. The 48 RD findings across marvel and forestage become
queryable by `kos orient`.

Probe: brief-rd-bridge. Session-008.

### `kos drift` (planned)

Simplest possible ripple: hash node content, walk derives edges, flag
dirty consumers. Implements finding-018's design at minimum viable scope.

Probe: TBD. Session-009.

## What this tool is NOT

- **Not a graph database.** It reads YAML files. When that breaks, the
  failure modes tell us what infrastructure is actually needed
  (question-knowledge-layer-requirements).
- **Not a document generator.** specticle generates documents. kos reads
  and validates the graph that specticle will eventually project from.
- **Not a project management tool.** It doesn't track work, assign tasks,
  or manage sprints. It tracks knowledge — what's known, at what
  confidence, with what dependencies.
- **Not a replacement for SDD processes.** It runs alongside them
  (charter non-goal NG1).

## Architecture

Single Rust binary, single crate (no workspace until complexity warrants
splitting). Modules per subcommand:

```
src/
  main.rs         CLI entry point (clap derive)
  lib.rs          Public API (when added)
  error.rs        KosError enum (thiserror, non_exhaustive)
  model/          Schema types — Node, Edge, Confidence, Signal, etc.
                  serde_yaml maps YAML directly to Rust types.
                  Exhaustive match over enums prevents silent bugs.
  orient/         orient subcommand
  validate/       validate subcommand
  graph/          graph rendering (petgraph + mermaid/dot output)
  bridge/         RD brief extraction
  drift/          ripple implementation (petgraph edge traversal)
```

See `.claude/rules/rust.md` for type design, error handling, and testing
conventions.

## Path to formal documentation

After session-009 (all four subcommands built), evaluate whether an
IEEE 1016 SDD with the cli-tool profile (context, composition, interaction,
algorithm viewpoints) is warranted. The specticle templates are available.
This product brief becomes input to section 1 of the SDD; the probe briefs
become input to the algorithm viewpoint. Build first, specify after —
per G3 (waterfall front-loading is graveyarded).
