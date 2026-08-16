# Exploration Brief: Serena as the measured live-LSP baseline

**bd:** aae-orc-5lbu
**Status:** locked, written before any tool was installed or measured
**Date:** 2026-08-16
**Repos under test:** `kos` (Rust, 8,681 LOC, single crate, 19 source files),
`marvel` (Go, 43,325 LOC, ~13 exported interfaces, multi-package)

---

## Why this document exists first

The ticket's blocking gap is that no artifact enumerates the workloads a code
baseline has to satisfy. A measurement taken against an unwritten workload list
cannot fail, because any result can be narrated as success afterward. So the
list is written first, with the expected output and the served/not-served line
fixed before I know what Serena does.

The decision criterion the ticket sets is that "insufficient" means a **named
workload Serena cannot serve**, not a general impression. That criterion is only
meaningful if the names exist in advance. They are W1 through W8 below.

## What counts as a workload here

A workload is a question an agent session in this fleet actually asks, in the
form it actually asks it. I derived the candidates from three sources: the
examples the ticket itself names, the session-protocol steps in `CLAUDE.md`
(orient, probe, harvest), and the recurring shapes in the session log (blast
radius before a refactor, cross-repo convention checks, "why is this like this"
archaeology).

Two premise corrections found while grounding the list, both of which change
the workloads:

- **kos has zero traits.** The ticket's example workload "who implements this
  trait on a Rust repo" has no instance in kos. It is a struct, enum, and
  free-function codebase. The implementers question moves to marvel, where
  `runtime.Adapter` has six implementations, and kos gets an enum-variant
  blast-radius workload instead. Testing the trait question against kos would
  have measured an empty set and scored it as a pass.
- **kos is a single crate with a flat `src/`.** Cross-module resolution in kos
  is intra-crate, which is the easy case for any LSP. Marvel's multi-package Go
  layout is the harder one, so the cross-boundary workloads are sited there.

## The scoring rule

Each workload is scored **served**, **partially served**, or **not served**
against its own stated expected output, not against a general sense of
usefulness.

- **Served** means the tool produced the expected output shape, correct and
  complete against ground truth I established independently with ripgrep and
  by reading the source, within the timebox.
- **Partially served** means it produced part of the answer, or the right
  answer with material gaps I had to close by hand, or the right answer only
  after I supplied information the tool should have found.
- **Not served** means it could not answer, answered wrongly, or the question
  is outside the shape of question it can take.

The comparator throughout is **ripgrep**, not nothing. The interesting question
is not "does a language server beat no tool", which is settled, but "does it
beat the grep an agent would otherwise run, by enough to justify the process,
the memory, and the warm-up". A workload that ripgrep answers in one command is
not evidence for a language server even when the language server also answers
it.

---

## The workloads

### W1. Who implements this interface?

- **Question:** "What are all the implementations of `runtime.Adapter`, and
  where is each one registered?"
- **Repo:** marvel (Go)
- **Expected output:** a list of six concrete types (`Forestage`, `Claude`,
  `Codex`, `OpenCode`, `Simulator`, `Generic`), each with its file and the
  location of its `Prepare` method, plus the registration site in
  `NewRegistry` at `internal/runtime/adapter.go`.
- **Ground truth:** established independently. Six `Prepare` methods across
  `codex.go`, `opencode.go`, `forestage.go`, `claude.go`, `simulator.go`,
  `generic.go`; registry constructor registers all six with `Generic` also set
  as fallback.
- **Served:** all six, no false positives, with the fallback relationship
  visible or at least not contradicted.
- **Not served:** misses an implementation, includes a non-implementer, or
  cannot express the query.
- **Comparator note:** `rg "func \(\w+ \*?\w+\) Prepare\("` returns all six in
  one command. For this to count as a win for the language server it has to
  add something grep did not: the fallback wiring, or correctness under a
  method set grep's regex would miss (embedded types, pointer vs value
  receivers).
- **Timebox:** 5 minutes.

### W2. What breaks if I change this symbol?

- **Question:** "I want to add a variant to the `Confidence` enum. What has to
  change?"
- **Repo:** kos (Rust)
- **Expected output:** the definition site in `src/model.rs`, and every site
  that matches on the enum exhaustively, which is where a new variant produces
  a compile error. Ranked or at least separated: match sites matter, mentions
  in strings and comments do not.
- **Ground truth:** 51 textual occurrences across 8 files (`orient.rs` 11,
  `model.rs` 11, `graph.rs` 10, `reflect.rs` 7, `charter.rs` 5, `init.rs` 2,
  `doctor.rs` 2, `bridge.rs` 2). The subset that is an exhaustive `match` is
  the answer; the rest is noise.
- **Served:** returns the reference set and distinguishes the match sites from
  incidental mentions, so the blast radius is smaller than the grep count.
- **Partially served:** returns all 51 references undifferentiated. That is
  grep parity with extra machinery.
- **Not served:** cannot enumerate references, or misses files.
- **Timebox:** 10 minutes.

### W3. What should be in the session preamble?

- **Question:** "I am starting a session on kos. What is this codebase, what
  are its major components, and what should I have loaded before I touch it?"
- **Repo:** kos
- **Expected output:** an orientation summary naming the CLI verbs and the
  module structure, at a level a session preamble could carry.
- **Served:** produces an orientation an agent could act on without opening the
  repo, and does so from code structure rather than from re-reading the README
  I could have read myself.
- **Partially served:** produces a file listing or a symbol dump that still
  needs a human to turn into orientation.
- **Not served:** no answer, or an answer that is just the README returned to
  me.
- **Test note:** this is the workload where I expect the honest answer to be
  that the value comes from the repo's own authored documents (`charter.md`,
  `CLAUDE.md`), not from the symbol graph. If so, that is the finding: symbol
  tooling does not produce orientation, and the F25 backdrop discipline is
  doing that work already.
- **Timebox:** 10 minutes.

### W4. Where does this actually happen, across package boundaries?

- **Question:** "Where does a marvel session's process environment actually get
  constructed, from the manifest field to the spawned process?"
- **Repo:** marvel (Go, crosses `internal/session`, `internal/runtime`, and the
  tmux driver)
- **Expected output:** a call path, or enough of one to follow: the field on
  the launch context, the adapter method that reads it, and the point where it
  becomes the environment handed to tmux.
- **Served:** the path is traceable through the tool's own symbol navigation
  (definition, references, callers) without me falling back to reading files
  in order.
- **Partially served:** individual hops resolve but I have to assemble the path
  by hand.
- **Not served:** callers cannot be resolved across packages.
- **Why this one:** this is the shape that grep is genuinely bad at, because
  the interesting edges are calls, not names. If a language server wins
  anywhere, it wins here. Treat a loss here as strong evidence.
- **Timebox:** 15 minutes.

### W5. Is this symbol dead?

- **Question:** "Is any of kos's public surface unreferenced, and specifically,
  what calls `graph::run`?"
- **Repo:** kos
- **Expected output:** the caller set for a named public function, and an
  answer distinguishing "called from the CLI dispatch" from "called nowhere".
- **Served:** correct caller set, including the dispatch site.
- **Not served:** cannot answer, or reports references that are definitions and
  imports rather than calls.
- **Timebox:** 10 minutes.

### W6. What did the last session touch, and what did it conclude?

- **Question:** "The previous session changed the compaction path. What did it
  change and what did it decide about it?"
- **Repo:** kos
- **Expected output:** the changed symbols and the reasoning recorded for the
  change.
- **Served:** any answer grounded in something other than the current file
  contents.
- **Not served:** the tool has no representation of time, prior sessions, or
  decisions, and can only describe the code as it stands now.
- **Why this one:** this is a deliberate failure probe. I expect not-served,
  and the point is to state precisely what is absent so the finding names it
  rather than gesturing at it. A language server is a snapshot index of one
  working tree; it has no history and no cross-session persistence by
  construction. This workload exists to convert that architectural fact into a
  named unserved workload, which is what the decision criterion requires.
- **Timebox:** 5 minutes.

### W7. Where else in the fleet do we do this?

- **Question:** "kos and marvel both write structured logs. Do they agree on a
  convention, and where is each one's implementation?"
- **Repos:** kos and marvel together, in one query
- **Expected output:** implementation sites in both repos, returned from a
  single question.
- **Served:** one query spans both repos.
- **Partially served:** answerable only by running the query twice against two
  separately activated projects, with me joining the results.
- **Not served:** cannot address a second repo at all.
- **Why this one:** the fleet is 25-plus repos and the recurring session
  question is a convention check across them. A language server's unit is the
  workspace, and two languages means two servers. I expect partial at best.
  The measurement to record is the cost of the join: how many activations, how
  much warm-up paid twice, and whether any of it persists.
- **Timebox:** 10 minutes.

### W8. Why is it this way?

- **Question:** "kos depends on `serde_yaml`, which is deprecated and archived.
  Why, and was that a decision or an accident?"
- **Repo:** kos
- **Expected output:** the rationale, wherever it lives.
- **Ground truth:** there is a `TODO` comment in `Cargo.toml` naming the
  deprecation and the intent to migrate. Whether more exists in `_kos/` or in
  git history is part of what the workload tests.
- **Served:** surfaces the rationale, including anything outside the source
  files.
- **Partially served:** surfaces the comment only, with no path to the decision
  record.
- **Not served:** no answer.
- **Why this one:** the second deliberate failure probe, aimed at provenance.
  Symbol tooling answers "what is this" and "where is it used". It does not
  answer "why", because why is not in the symbol graph. This fleet keeps why in
  `_kos/findings` and node YAML, which is exactly the material a kos ingest
  would carry and a language server cannot.
- **Timebox:** 10 minutes.

---

## Instrumentation, run alongside every workload

The ticket says the shape of the failure is the probe's real value, so these
are measured whether or not the workloads pass.

1. **Cold versus warm latency.** First query after activation against the same
   query repeated once the server is warm. Recorded per repo, since Rust and Go
   servers warm differently.
2. **Peak RSS of the spawned language server** on the kos workspace, by `ps`
   sampling at a fixed interval, reported as peak and steady state.
3. **Persistence across restarts.** Whether any index or answer survives the
   process. If it does not, warm-up is paid once per session, and the session
   count in this fleet is 3 to 5 concurrent.
4. **Provenance on answers.** Whether a result carries anything a later reader
   could audit: a commit, a timestamp, a reason. A file and line is a location,
   not a provenance.
5. **History.** Whether anything survives the session, per W6.

## Contamination caveat

A second agent is running rust-analyzer SCIP indexing on this machine
concurrently with this probe. Every latency and memory number below is
timestamped and bracketed by a `vm_stat` and `uptime` snapshot. Any number
close enough to a decision boundary to change the verdict is marked **rerun
solo before relying on it**. I would rather report a contaminated number as
contaminated than spend the window fighting for the machine.

## Registered prediction

The ticket registers 0.65 that Serena alone proves insufficient. I score it in
the finding as a plain hit or miss against the criterion above, with the named
workload that carries it. Recording it here, before measuring, so the score is
not written to fit the result.

I will also record the two dissenting outcomes the ticket names, so that
neither can be quietly skipped:

- **Serena proves sufficient.** Then the ingest work is not justified now, and
  the finding says so.
- **Live LSP is the right permanent pattern.** Then the conclusion is *the
  pattern won, implement an LSP client natively in Rust*, and specifically not
  *keep Serena*. Serena is a Python MCP server with its own process and
  dependency footprint; adopting the pattern is a different decision from
  adopting the vendor of the pattern.

## Out of scope

Answer quality of the model driving the tools, prompt engineering of the
queries, and any comparison against SCIP indexing, which the concurrent probe
owns.
