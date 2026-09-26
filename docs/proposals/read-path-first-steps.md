# Proposal: kos Read Path, First Steps

Status: proposed (2026-08-16). A probe, not a commitment to outcomes.
Scope: kos read features. Audience: a future kos session executing from
this document.

## The goal

Turn kos from a warehouse into a library. Make Query a working verb of
the operating loop.

kos has a working write path and almost no read path. What exists today
is `kos orient` (a bulk per-cwd dump with substring filtering, `--json`,
and `--ready`) plus hygiene reads (validate, drift, doctor, reflect,
graph, compact, charter render). There is no `kos ask`, no read
telemetry, no MCP surface, and no `kos status`. Nothing anywhere records
whether a node has ever been consulted.

Three of the project's own documents diagnose this with one voice:

- `vision.md` (adopted 2026-08-01) names the operating loop as Write /
  Query / Generate / Judge, calls Query "Gap 0, the weakest verb,"
  marks kos "USABLE; read path missing," and gives the fix order
  cheapest first: read-telemetry loglines, then `kos ask` (lexical plus
  graph-proximity interim, flyloft as eventual substrate), then
  "findable or it isn't harvested" as a harvest gate. Gap 0 gates the
  operating loop.
- `_kos/ideas/kos-read-path-gap.md` (2026-07-15): "A knowledge system
  with writes and no reads is a warehouse, not a library." This is the
  PULL half; F19 active surfacing is the PUSH half.
- `kos/docs/kos-roadmap-v3.md` and `v3_1.md` (2026-07-04) promote the
  MR (Retrieval-First) milestone above capture discipline, because
  every prior knowledge system died of the Grudin asymmetry: capture
  cost and retrieval benefit fall on different people. Standing rules
  carried forward here: no new author-side capture before retrieval
  value is demonstrated; metrics stay diagnostic and never gate;
  automate proposing, never judging.

## Five proof obligations

In order.

1. **Measure reads before building anything heavier.** Grudin
   discipline applied to ourselves. Telemetry records which node IDs
   are actually served, so every later read-feature decision (ranking,
   `kos ask` scoring, promotion evidence, cold-node retirement,
   flyloft's eventual role) rests on measured circulation rather than
   intuition. Ten-session baseline; pre-registered hypothesis "at least
   40 percent of nodes are never read post-creation." Ticket:
   `aae-orc-2qlx0` (collection slice), with `aae-orc-kx8f` as the
   aggregator-side seam.
2. **Ship a query verb agents and humans use mid-session.** `kos ask
   <question>`: scoped, ranked retrieval, lexical plus graph-proximity
   as the interim substrate. The verb is the contract; the substrate
   swaps later. It must not grow into flyloft-inside-kos. Ticket:
   `aae-orc-jajn3`.
3. **Keep the read surface honest.** Do not serve a corrupted graph
   (M0 schema repair before any server, per `aae-orc-b0kp`); do not
   serve untested findings as if tested (`aae-orc-37hm`); do not let
   machine-paid features smuggle in operator rituals (the `aae-orc-b0kp`
   cross-track rule); do not let confirm-or-override decay into
   rubber-stamping (`aae-orc-qao2`).
4. **Converge the surfaces, do not multiply them.** One query engine
   behind three faces over time: CLI verb now, kos-side read-only MCP
   Phase 0 next, flyloft federation later. Recorded as ADR-009;
   `aae-orc-di23` is resolved by sequencing rather than by choosing a
   single permanent home.
5. **Repair the record so the plan is traceable.** Gap 0 lives in the
   adopted vision but was absent from the adopted roadmap spine;
   `kos ask` and read telemetry had no bd tickets at all; "R1" named
   three different things; and the substrate report cited as settled in
   `aae-orc-1od3` existed but was chat-produced and never committed.
   That last item is a structural harvest gap in the collaboration
   itself: a planning conversation is a probe surface whose findings do
   not exist for repo sessions until they are committed, and repo
   sessions had started citing ghosts.

## The plan, three waves

### Wave 1: records reconciliation (orc repo and kos repo docs, one sitting, PRs)

Green-lit by operator review 2026-08-16 with three amendments, all
folded in: amend the telemetry pre-registration before any data
collection; commit the chat-produced substrate report and file a
finding on the chat-as-probe-surface harvest gap; adopt an R-prefix
convention.

- Commit the chat-produced documents so cross-session references
  resolve. `kos/docs/kos-substrate-alternatives.md` carries the
  substrate report verbatim (verdict: Dolt and a SQLite fact log beat
  TerminusDB; DoltLite and Turso are probe-watch; Kuzu is dead; Cozo is
  dormant; revised M4 Probe A/B design with pre-registered confidences
  0.75 and 0.6 and a decision rule; TerminusDB drops to tier 3).
  Confirm `kos-roadmap-v3`, `v3_1`, and `kos-use-assessment` are
  committed. Then update `aae-orc-1od3`: the report is located and
  committed; the remaining work is naming the two "five properties"
  lists distinctly (finding-036's what-files-cannot-provide versus the
  report's what-a-versioned-store-must-provide) and tracing `isie`'s
  design-field claims either to props or to an "asserted, unverified"
  tag.
- File the chat-as-probe-surface finding in the orc `_kos/findings/`.
  No new rule this wave; a pointer from the tooling-friction-adjacent
  rules only if it earns one.
- File the two missing read-path tickets: `aae-orc-jajn3` (`kos ask`
  Phase 0) and `aae-orc-2qlx0` (read-telemetry collection slice). Both
  flat, per the bd-hierarchy rule; no invented parents.
- Fix the record discontinuities. Add the Gap 0 read-path item to
  `docs/roadmap.md` Track K as K7 (summary and pointer only, per
  charter-light-touch). Disambiguate the three "R1"s with one
  clarifying line, without rewriting history documents. Note the
  crystallization in `_kos/ideas/kos-read-path-gap.md`, pointing at the
  new tickets.
- Adjudicate `aae-orc-uuob` (sequencing conflict). It likely dissolves:
  the recorded criterion is that `aae-orc-b0kp` stands unless the
  ten-session telemetry clock is genuinely on retrodiction's critical
  path, and with collection starting now and the circulation clock
  re-pinned to `kos ask`, it probably is not. Check
  `kos-roadmap-v3.md` Issue 25 for whether retrodiction's baseline
  consumes telemetry, write the adjudication into the ticket notes, and
  close if it dissolves.
- Draft ADR-009 for `aae-orc-di23`: one query engine, three faces over
  time. Record why two permanent servers is the ruled-out failure and
  why a kos-side Phase 0 does not foreclose federation.
- Write this proposal and cross-link it from every new ticket.
- Create the three kos frontier question nodes (see Epistemic status
  below).

Not worked in this wave: `aae-orc-1y3g`, `aae-orc-37hm`, and
`aae-orc-qao2` are open items with their own criteria and are only
cross-referenced here. `aae-orc-1y3g` gates the SCIP ingest, not this
wave.

### Wave 2: telemetry collection slice (kos repo)

Ticket: `aae-orc-2qlx0`.

A minimal read-telemetry module in kos: fail-open, logging the node IDs
served by `kos orient`, per-session append-only JSONL, on by default,
machine-paid, never a gate.

Spec sources: `kos-roadmap-v3.md` Issue 20 (`_kos/.telemetry/reads.jsonl`,
node IDs served, ten-session clock) and `kos-nine-improvements.md`
(tuple of node_id, command, session, timestamp; diagnostic, never a
gate). The existing `--log` precedent in `orient.rs` logs counts only,
is opt-in, and is machine-global. The gap this fills is IDs,
graph-scoped, on by default.

Shape:

- New `src/telemetry.rs` with `ReadEvent { verb, target, node_ids,
  finding_ids }`, `record_reads(graph_dir, &event)`, and `session_id()`.
- Location `<graph>/_kos/.telemetry/reads-<session>.jsonl`: the
  document's directory, `aae-orc-b0kp`'s per-session multiplicity.
  Graph-scoped rather than under `~/.local/share`, because aggregation
  is per-graph and machine-global storage reintroduces multi-writer
  contention. Add `.telemetry/` to kos's `.gitignore`, matching the
  `.drift-snapshot.json` precedent.
- Session ID from `KOS_SESSION` if set, else `<pid>-<unix-start-secs>`.
  Worst case that degenerates to one file per invocation, which is
  still contention-free; the aggregator globs.
- One JSONL line per invocation with ID arrays: `ts`, `session`, `verb`,
  `target`, `read_class`, `json_output`, `node_ids`, `finding_ids`. One
  atomic write preserves co-service context, and the per-node tuple is
  derivable at aggregation time.
- Instrument `run()` in `orient.rs` once, after the `Orientation` struct
  is populated, since both printers render everything in it (so
  "served" means "present in `Orientation`"). Instrument `run_ready()`
  as verb `orient-ready`. Do not instrument doctor (a health check, not
  consumption) or reflect (it serves just-edited nodes, which would
  contaminate the never-read-after-creation hypothesis). Slice 1 is
  orient only.
- Reads are `Orientation.{bedrock_nodes, frontier_questions,
  graveyard_nodes}` plus finding IDs. Charter items, probes, ideas, and
  `rd_*` are excluded: they are not graph nodes, and the hypothesis is
  about nodes.
- Fail-open at the call site, mirroring the existing warning pattern in
  `orient.rs`. Telemetry can never fail orient, and nothing reads it in
  any check path.
- On by default with a `KOS_NO_TELEMETRY` opt-out checked at the call
  site, keeping the function pure and parameterized for tests. IDs
  only, no content; gitignored; local.

Tests, unit-level in `telemetry.rs` with a tempfile dev-dependency:
appends valid JSONL; appends rather than truncates; distinct sessions
produce distinct files; errors on an unwritable directory (proving
fail-open is caller-side); opt-out skips the write.

Ship via kos's normal flow (main-branch repo, PR), commit vocabulary
`probe(kos):` or `feat:` per its conventions. Close `aae-orc-2qlx0` on
merge.

### Wave 3: cheap code-graph measurements (parallel, machine-paid)

Run under their existing tickets once `aae-orc-aj3c` (the arc seed)
lands, or alongside it. Neither gates Waves 1 or 2; both inform whether
the SCIP ingest is ever worth building.

- `aae-orc-5lbu`: install Serena, measure it as the live-LSP baseline
  against real session workloads (CG-R2; registered prediction 0.65
  insufficient).
- `aae-orc-msqx`: measure `rust-analyzer scip` and `scip-go` time and
  memory on the full workspace (CG-Q1). The claim that CG-R1 cannot
  land on YAML-in-git is asserted, not measured.

### Explicitly deferred

Recorded, not started:

- `kos ask` implementation itself. The next wave after telemetry lands;
  `aae-orc-jajn3` carries the design pointer.
- MCP Phase 0, after M0 schema repair per `aae-orc-b0kp` round-3 item 2.
- The telemetry aggregator and its consumers, gated on the writer model;
  `aae-orc-kx8f` is the seam.
- The SCIP-to-kos ingest (five blockers), reopener triggers,
  retrodiction, and the F19 predictor push half.

## Sequencing inheritance

The standing sequencing decision is `aae-orc-b0kp` (party consensus,
round-3 amended). It splits work into two tracks:

- **Track A**, operator-facing, serial, gated by the operator's habit
  budget.
- **Track B**, the read side, machine-paid, parallel. Nothing in Track B
  may spend Track A's budget; that cross-track rule is what keeps a
  machine-paid feature from smuggling in an operator ritual.

Track B order, inherited by this proposal unchanged:

1. `validate` as CLI plus CI.
2. M0 schema repair before any MCP surface. An MCP server over a
   corrupted graph serves lies with a nice API, and finding-060
   measured roughly 24 percent undeclared-type edges under green
   validation.
3. Telemetry collection starts now, as per-session append-only files.
   That is `aae-orc-2qlx0`, Wave 2 above.
4. MCP read-only Phase 0.
5. Reopener triggers.
6. Retrodiction.

Three decisions taken in the 2026-08-16 planning session sit on top of
that order. Scope: reconcile the record and start the cheapest
machine-paid build. Read surface: `kos ask` as a CLI verb first, with
the kos-side MCP Phase 0 later wrapping the same query engine and
flyloft fronting it eventually, which satisfies `aae-orc-di23`'s
criterion of soonest surface without foreclosing federation. Code-graph
arc: fold in only the two cheap measurements (`aae-orc-5lbu`,
`aae-orc-msqx`); the SCIP ingest stays gated.

## Pre-registration amendment

Written before any data collection, and carried in the body of
`aae-orc-2qlx0`. It exists to prevent the HARKing-adjacent re-scope
that the Kerr guard (roadmap Issue 10) was put in place for.

1. **Reads are classified at log time**, not at analysis time, into two
   classes. CONSULTATION covers targeted or filtered orient and the
   future `kos ask`. BULK-SERVE covers unfiltered orient, which marks
   essentially the whole graph as read by construction. The class is
   written into the `read_class` field on each event.
2. **The slice-1 deliverable is orient-shape statistics**, not the
   circulation hypothesis. Slice 1 tells us how orient is actually
   invoked and in what proportion the two classes occur. It does not
   test whether nodes go unread.
3. **The ten-session circulation clock starts when `kos ask` lands**,
   and it counts consultation reads only. The hypothesis under that
   clock is "at least 40 percent of nodes are never read
   post-creation." Starting the clock earlier, or counting bulk serves
   toward it, would answer a different question with the same number
   and call it the same result.

## Epistemic status

This proposal is a probe, not a commitment to outcomes. Two layers of
uncertainty are held open structurally rather than rhetorically.

**Benefit uncertainty.** Telemetry, `kos ask`, and the chosen sequencing
may not deliver the intended value. The plausible failures are a
measured-but-unconsulted instrument, an unused verb, and a surface in
the wrong place.

**Approach uncertainty.** The read-path-first approach itself
(Grudin-ordered, verb before server, measure before build) may be wrong.
The pre-registered hypotheses and the `aae-orc-b0kp` / `aae-orc-uuob`
adjudication trail are what let us find that out rather than argue it.

Three kos frontier nodes carry that uncertainty, each with a testable
success signal and a kill condition, so the graveyard can collect
honestly. Their subject is kos, so they live in kos's graph
(`kos/_kos/nodes/frontier/`, schema v0.3). Each cross-links to this
proposal, to its tickets, and to the orc-level parents
(question-active-knowledge-surfacing, F19, and the
`kos-read-path-gap.md` idea).

- **`question-read-telemetry-decision-value`**. Does measuring
  circulation actually change a later read-feature decision (ranking,
  retirement, promotion evidence), or does the instrument go unread
  like the graph it measures? Success: a named decision cites the
  telemetry within N sessions of the aggregator existing. Kill: two
  consecutive review rounds where nobody consulted it.
- **`question-kos-ask-retrieval-value`**. Does `kos ask` beat grep and
  unfiltered orient on real session questions, against the ranking,
  freshness, and provenance criterion, and does it get used?
  Consultation reads in the telemetry are the usage measurement.
  Pre-registered: if sessions keep reaching for grep after the verb
  exists, the verb failed regardless of benchmark quality. The kill
  condition is explicit in the node.
- **`question-read-surface-sequencing`**. Does verb-first-then-MCP
  hold, or does the benefit only arrive inside the agent task loop
  (the MCP-first argument)? Evidence: consultation-read share from the
  CLI versus, later, from MCP.

The orc-level chat-as-probe-surface finding stays at the orc. If that
failure recurs, it graduates to its own question node.

## Verification

- **Wave 1**: PRs merged on orc main and kos main; this proposal
  carries real ticket IDs; the frontier nodes pass `kos validate`;
  `bd show` resolves the new tickets; `aae-orc-uuob` notes carry the
  adjudication; `docs/roadmap.md` Track K names the read path; ADR-009
  appears in `decisions/index.md`.
- **Wave 2**: it builds; `kos orient kos` twice, then `ls
  _kos/.telemetry/` shows `reads-<pid>-<ts>.jsonl`, and
  `KOS_SESSION=test-a` produces `reads-test-a.jsonl`; `jq .node_ids`
  over the file, diffed against orient's printed `[id]` lines, gives
  matching sets; `time kos orient kos` before and after shows no
  measurable slowdown; `git status` confirms `.telemetry/` is ignored;
  tests green.
- **Wave 3**: measurements recorded in `aae-orc-5lbu` and
  `aae-orc-msqx` plus findings, per the arc's own criteria.

Note for whoever runs Wave 2's verification: `command -v kos` resolves
to the Homebrew binary. Built code has to be exercised with `cargo run
--` from the kos checkout.

## Conventions and pointers

**R-prefix convention**, adopted 2026-08-16 and used in all new tickets
and documents. "R1" previously named three different things, which is
how the read-path item went missing from the roadmap spine without
anyone noticing.

| Prefix | Namespace |
| --- | --- |
| `UX-R*` | use-assessment items (`kos/docs/kos-use-assessment.md`) |
| `CG-R*` | code-graph briefing items |
| `Gap-0` | the vision.md read-path item |

**Substrate**: `kos/docs/kos-substrate-alternatives.md`, committed
alongside this proposal, is the report `aae-orc-1od3` refers to. It
governs the eventual store choice under kos's read path, not the read
path's first steps; nothing in Waves 1 through 3 depends on its
outcome.

**Ticket index for this proposal.**

| Ticket | Task |
| --- | --- |
| `aae-orc-b0kp` | two-track sequencing decision (Track A / Track B) |
| `aae-orc-uuob` | sequencing conflict adjudication (Wave 1) |
| `aae-orc-di23` | single MCP surface decision, resolved by ADR-009 |
| `aae-orc-1od3` | substrate report not in graph (Wave 1) |
| `aae-orc-jajn3` | `kos ask` Phase 0 (deferred build; filed Wave 1) |
| `aae-orc-2qlx0` | read-telemetry collection slice (Wave 2) |
| `aae-orc-kx8f` | telemetry seam registration (aggregator side, deferred) |
| `aae-orc-37hm` | untested findings indistinguishable (trust side) |
| `aae-orc-qao2` | who pays to trust (trust side) |
| `aae-orc-5lbu` | Serena baseline probe (Wave 3) |
| `aae-orc-msqx` | scip cost probe (Wave 3) |
| `aae-orc-aj3c` | code-graph arc seed (Wave 3 gate) |

Labels on all new tickets, per convention: `aae-orc`, `kos`,
`source:session`.
