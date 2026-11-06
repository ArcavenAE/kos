---
id: finding-kos-qe45
type: finding
confidence: frontier
title: "Cross-graph blocking edges never resolve, and the docs promise a resolution order nothing implements"
created_at: 2026-11-06
finding:
  probe: null
  result: partial
  evidence: mixed
---

# finding-kos-qe45: cross-graph blocking edges never resolve, and the docs promise a resolution order nothing implements

**Bead.** aae-orc-ti6zx, feeding aae-orc-nyvk (a ruling the operator is asked for). The live specimens come from aae-orc-1bte.

**Status.** MEASURED (code at kos origin/main 12cd440; node parse at kos 12cd440 and marvel ba0ea65). Mixed by finding-kos-to4v's rubric: it measured, and it named no brief with success signals.

**Provenance.** Researched by the research-supervisor role at the operator's request. The chair re-checked `src/orient.rs:170-178` and `schema/manifest.schema.yaml:74-76` at origin/main.

## Summary

1. At kos origin/main 12cd440, `kos orient --ready` treats a blocking edge as satisfied only when its target is a local bedrock or graveyard node or a local finding id. Cross-graph ids, probe files and absent ids stay blockers forever [measured].
2. Live specimens exist in both public graphs. marvel has 21 blocking cross-graph edges on 16 nodes, and 9 of its 45 frontier questions are blocked by them, 6 by cross-graph targets alone. kos has 1 question with 4 cross-graph derives edges, and 2 questions blocked on a probe file that exists locally. Neither graph has an absent-id blocker.
3. Spec and code disagree. `schema/manifest.schema.yaml:76` promises "current graph first, then orchestrator, then includes in order", and no read path implements it. `kos doctor` skips `::` and `brief-` targets, so it reports edges as resolving that orient holds as permanent blocks.
4. No option alone covers the specimens:
   - relocation (c) cannot absorb edges to platform values, which stay at the orchestrator under the subject test;
   - forbidding (b) as a hard error breaks 25 public edges at once;
   - resolution (a) needs the finding-132 and finding-133 primitives first.
5. Recommendation, as input to the nyvk ruling: make the docs truthful first, then add nyvk's "indeterminate" status and a WARN-level support default (b), keep relocating (c), and defer (a). Valid until 2026-11-20.

## Question

aae-orc-ti6zx, feeding aae-orc-nyvk: is local-only blocker resolution in `kos orient` a defect to fix, or semantics to codify? The bead lists three options: (a) build cross-graph resolution, (b) forbid cross-graph blocking edges, (c) relocate. nyvk adds a middle option, non-local blockers legal but reported as indeterminate, treated here as (d).

## Method

- Fetched kos (origin/main 12cd440) and marvel (origin/main ba0ea65), and read the code with `git show` at origin/main.
- Parsed every `_kos/nodes/**.yaml` at origin/main with a read-only script that applies orient's rule:
  - derives, implements and `depends_on` count as blocking;
  - a target resolves if it is a local bedrock or graveyard node or a finding key (file stem, `finding-NNN` prefix, or frontmatter id).
- Control: ran the installed `kos orient --ready --json` (alpha c4f93c2) on local checkouts. Those checkouts lag origin/main (marvel's `_kos` by 21 files), so they corroborate rather than measure.

## Results

**Mechanism [measured]**
- `src/orient.rs:107-114`: `run_ready` loads one graph, `workspace.nearest_graph(cwd)`.
- `src/orient.rs:125-139`: the resolution sets are local bedrock ids, local graveyard ids and local finding ids (`load_finding_ids`, 256-271). Probes are never loaded for readiness.
- `src/orient.rs:170-178`: `is_resolved` is a plain set lookup. It does no `::` parsing and makes no distinction between unreachable, unknown and open. Any miss becomes `ReadyStatus::Blocked` (183-187).
- `src/model.rs:204-206`: `is_blocking` is derives or implements. `src/model.rs:48-59`: `all_edges()` turns every `depends_on` entry into a Derives edge, so `depends_on` is affected as well.
- By kind: a cross-graph target is always blocked. A probe file is always blocked, even when `_kos/probes/<id>.yaml` exists. An absent id is blocked, which is defensible. A local finding resolves, so nyvk's wording on findings is out of date.
- `src/doctor.rs:215-222` skips `::` targets and passes `finding-` and `brief-` targets. So doctor's "all edge targets resolve" can print over a graph that orient holds blocked. In `--merged` mode it still skips them, although the comment at 216 says "non-merged" [measured from code; doctor was not run].
- `src/validate.rs:520-527` warns on any target not found in `nodes/`, `::` included, with the hint "may be a finding or probe".
- `schema/manifest.schema.yaml:74-76` documents `::` and a resolution order that nothing implements. `schema/node.schema.yaml:123-131` defines blocking with no word on graph boundaries.
- Related kos graph state:
  - `_kos/nodes/bedrock/elem-kos-artifact-id-scheme.yaml` (an operator ruling of 2026-09-16) makes `graph::slug` the canonical cross-graph reference and says validate should accept the syntax. So (b) can forbid blocking types across a boundary, never the syntax.
  - `docs/design/opaque-finding-ids.md:316` (item 3) plans a `graph::`-aware citation resolver. `src/findings.rs` has no `graph::` parsing yet [measured by grep].
  - kos findings 131, 132 and 133 live in kos `_kos/findings/`. finding-133's requirement 2, that absence of access is not absence of node, is nyvk's option 3 stated as a hard requirement.

**Specimens [measured, origin/main]**

| graph | nodes | cross-graph blocking edges (nodes) | frontier questions blocked by them | probe-file blockers | absent |
|---|---|---|---|---|---|
| kos | 94 | 4 (1) | 1 | 2 (2 questions) | 0 |
| marvel | 66 | 21 (16) | 9 (6 by cross-graph targets alone) | 0 | 0 |

The orchestrator graph holds more; it was not counted here. Examples:
1. `marvel/_kos/nodes/frontier/question-gateway-external-api.yaml`: one `derives` to `aae-orc::elem-marvel-control-plane`, its only blocker. This is the open 1bte specimen, still unrepaired.
2. `marvel/_kos/nodes/frontier/question-permission-model.yaml`: `implements` to `aae-orc::elem-component-independence` and `derives` to `aae-orc::val-user-sovereignty`. Both targets are platform values the subject test keeps at the orchestrator.
3. `kos/_kos/nodes/frontier/question-fractal-gates.yaml`: `derives` to `brief-orient-standalone`, which exists at `kos/_kos/probes/brief-orient-standalone.yaml` and which orient never consults.

[inferred] Author intent is mixed:
- `marvel::question-multi-host -> aae-orc::question-multi-host` points at a likely open question, so the block may be intended.
- `derives -> val-*` edges read as provenance ("grounded in"). Conflating derives-as-provenance with derives-as-dependency turns them into false blocks.
- `kos/_kos/nodes/frontier/question-code-graph-correspondence.yaml` already uses `supports` for its cross-graph edge, which is the support default applied by hand.

## Options

**(a) Build cross-graph resolution.**
- What changes: orient, validate and doctor parse `graph::id` and look it up through the workspace graph list (`workspace.rs` already discovers graphs) or a registry. The 1bte targets would resolve.
- Cost: the highest. It needs finding-133's split between unreachable and absent, a rule for `own-graph::id` (legal under the id-scheme ruling, and a false block today), and ideally finding-132's forwarding; without that, relocations turn resolved edges back into dangling ones.
- Independence: holds only with a three-way outcome (resolved, blocked, unreachable), so that a marvel checkout alone never hangs or errors.
- What breaks: no data. Readiness becomes a function of which checkouts are present.

**(b) Forbid cross-graph blocking edges.**
- What changes: schema text and a validate check. The lowest cost.
- As a validate ERROR it fails 25 public edges on the next run, a sudden gate wherever validate runs in CI. A WARN that suggests the support default fits the fleet's rule that a vital sign reports and does not gate.
- It overrides authors who meant a real dependency (the multi-host case).
- Independence: good, since each graph is judged alone.

**(c) Relocate.**
- What changes: no code. Nodes move by the subject test, and their edges become bare local ids.
- Cost: manual. Without finding-132 forwarding, every move needs an edge sweep.
- Limit: edges to platform values (`val-user-sovereignty`, `elem-component-independence`, `val-gradual-elaboration`; 5 of marvel's 12 frontier-question edges) cannot relocate, because their subject is the composition. (c) alone leaves those questions blocked.

**(d) Indeterminate status (nyvk option 3).**
- What changes: orient only. A non-local blocking target puts the question in a third bucket, neither ready nor blocked, and lists the unresolved non-local ids.
- It stops evicting questions silently without falsely reporting a true block as ready.
- It matches finding-133's requirement 2 now, and it becomes (a)'s "unreachable" case later, so it is forward-compatible.

## Recommendation (input to the nyvk ruling, not a decision)

An ordered combination of (d), (b) as a WARN, and (c), with (a) deferred:
1. **Make the docs truthful.** `node.schema.yaml` states that blocking resolves against the local graph, that a cross-graph or probe target cannot be discharged locally, and that the support default applies. `manifest.schema.yaml:76` either drops the unimplemented resolution order or marks it as not yet implemented. The bead requires this statement in the schema doc whatever the ruling.
2. **Add the indeterminate status.** Orient puts `::` targets in it, and treats `own-graph::id` as local.
3. **Warn, never error.** Validate WARNs on derives or implements across a graph boundary or to a probe, and suggests `supports`. Doctor stops skipping these targets silently.
4. **Keep relocating** by the subject test, retargeting edges as nodes move.
5. **Revisit (a)** once finding-132 forwarding and the `graph::` resolver of opaque-finding-ids item 3 exist. (d)'s bucket is where (a) plugs in.

First step: a kos PR that does item 1 only, plus characterization tests that pin today's behavior, with no behavior change. It is decision-neutral, so it stays correct under any ruling.

## Sell-by

Valid until 2026-11-20, or until nyvk is ruled, whichever comes first. The research-supervisor seat re-checks it then, against kos origin/main and the marvel specimen counts.

## Tests (test-architect lens, red-first)

1. A frontier question whose only blocker is `other::x`, with no other graph present, gives indeterminate, not Blocked. Red today.
2. In the same fixture, the question appears in both the human output and the JSONL output, with `"status":"indeterminate"` and the non-local ids listed, so it is never silently absent.
3. A question with a local open blocker plus `other::x` stays Blocked; the local block dominates.
4. A `depends_on: [other::x]` entry behaves exactly like the derives edge (guards `model.rs:48-59`).
5. Inside graph `g`, `g::bedrock-node` resolves as local. Red today.
6. A bare local id that is absent stays Blocked, and doctor counts it as broken (pins absent-node semantics).
7. A blocking edge to a `brief-*` that exists in `_kos/probes/`: validate WARNs, and orient's outcome is pinned to whatever the ruling says.
8. Validate on a derives or implements edge with a `::` target: WARN naming `supports`, exit 0. Positive control: a `supports` edge with a `::` target produces no warning.
9. A consistency property: every target orient lists as a blocker is never counted as resolving by doctor. Red today, from `doctor.rs:215-222`.
10. Later, for (a): a reachable sibling graph with a bedrock target gives ready, and with a frontier target gives blocked. An unreachable graph gives indeterminate. A reachable graph that lacks the id gives absent, distinct from unreachable.

## Not read

- The orchestrator graph and its relocation index.
- The other fleet graphs.
- finding-074 and finding-131 in full.
- kos `docs/decisions`.
- Whether any CI runs `kos validate` on marvel or kos.
- `kos doctor` was not run; its behavior here comes from reading the code.
- The brief status fields in `_kos/probes`.
