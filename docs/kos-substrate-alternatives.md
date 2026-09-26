# KOS Substrate Alternatives — Options More Ideal Than TerminusDB

> Follow-up to the TerminusDB suitability assessment. That report scored
> TerminusDB best-in-class on the five substrate properties but failing on
> three axes: no in-process embedding (Docker/server daemon), single-steward
> health (DFRNT), and WOQL/Prolog skill asymmetry for LLM agents. "More
> ideal" therefore means: fixes those three axes while conceding as little
> as possible of P1–P5 and the git-semantics fit. All health claims verified
> July 2026 against primary sources.

---

## Verdict

**Two candidates beat TerminusDB outright for KOS; two more are
probe-watch; the rest are disqualified.**

1. **Dolt** — fixes all three failure axes at once (best-in-class health,
   SQL skill symmetry, single static brew-installable binary — no Docker,
   no JVM) while *keeping* real git semantics: true branch/merge/diff,
   cell-level conflict detection, `AS OF` time travel, cherry-pick as
   selective harvest. And it is already inside KOS's fence: Embedded Dolt
   became the default backend of Beads in April 2026 [@dolthub2026beads] —
   the task tracker KOS's platform already runs.
2. **SQLite + append-only fact log** (Datomic-shaped, built thin) — the
   only candidate that is fully in-process from Rust with bedrock health;
   you build branching/asof/harvest in application space, which at KOS's
   scale (hundreds of nodes) is a bounded, ownable amount of code.
3. *Probe-watch:* **DoltLite** (embedded SQLite-compatible Dolt, March
   2026 — architecturally the ideal endpoint, but four months old and
   experimental) and **Turso** (Rust SQLite rewrite — MVCC multi-writer,
   built-in MCP mode, but branching is cloud-only and the engine is beta).

---

## What changed since the TerminusDB report

Four landscape facts, all 2025–2026, all verified:

**Kuzu is dead.** Archived without warning October 10, 2025; repo
read-only; docs pulled; "Kuzu is working on something new"
[@biggo2025kuzu; @register2025kuzu]. Community forks ("bighorn,"
"Ladybug") exist but inherit an on-disk format that was never stabilized
[@biggo2025kuzu]. Health-disqualified — a sharper cautionary tale than
TerminusDB itself.

**Cozo is dormant.** Last release v0.7 (November 2023); no releases since;
the database-of-databases entry accessed June 2026 still lists v0.7 as
current [@dbdb2026cozo]. The single-author risk flagged in the prior
report materialized. Demoted from probe candidate to design reference
(its Datalog time-travel model remains instructive).

**Dolt went from strong to dominant.** Dolt 2.0 shipped May 11, 2026
(storage-stable, adaptive storage, version-controlled vector indexes)
[@dolthub2026dolt2; @infoq2026dolt2]; Doltgres 1.0 lands August 6, 2026
[@dolthub2026doltgres]; DoltHub now explicitly markets Dolt as "the
database for AI agents": *every agent gets its own branch, writes are
isolated until you review and merge, rollback is surgical* — which is
KOS's P5 + human-pawl + harvest model restated as a product pitch
[@dolthub2026agents]. A Dolt MCP server exists and was extended to
Doltgres in July 2026 [@dolthub2026blog].

**An embedded Dolt lane opened.** DoltLite launched March 25, 2026: a
free, open-source, drop-in SQLite replacement (C library, WASM target)
with Dolt version control as SQL functions — `dolt_add`, `dolt_commit`,
`dolt_log` in-process [@dolthub2026doltlite]. Separately, Embedded Dolt
(the Go library form) became Beads' default backend, "restoring the
simple single-player experience without an external server"
[@dolthub2026beads]. The embedding objection that disqualified
TerminusDB is being solved in the Dolt ecosystem from two directions.

---

## Tier 1, candidate 1: Dolt

**P1 — first-class edges: PARTIAL→FIT.** Edges as rows in an
`edges(from, to, type, provenance…)` table: independently queryable,
diffable, versioned per-cell, carrying their own columns. Not
graph-native typing — schema discipline replaces it — but every edge has
commit-granularity history, which YAML-in-git never gave.

**P2 — preserved reasoning chains: PARTIAL→FIT via modeling.** Dolt
versions rows, not datoms. Model the graph as an append-only EAV fact
table (never UPDATE, only INSERT with op add/retract) and Dolt's commit
graph supplies transaction context on top: `dolt_log`, `dolt_diff`,
`dolt_history_facts` reconstruct any derivation. History is structurally
the primary record — Prolly-tree content addressing with structural
sharing [@infoq2026dolt2].

**P3 — projections on demand: FIT.** SQL views and queries; charter,
subgraph bundles, drift reports are all `SELECT`s. Best possible LLM
agent fluency — SQL is the one query language with saturated training
data.

**P4 — asof/time-travel: FIT.** `AS OF <commit|timestamp>` queries are
native; `kos asof` for blinded retrodiction maps directly
[@dbdb2026dolt].

**P5 — concurrent multi-agent: FIT, and productized.** Branch-per-agent
isolation with cell-level conflict detection at merge is DoltHub's own
recommended agent pattern [@dolthub2026agents; @dolthub2026conflicts].

**Git-semantics/harvest: FIT — the best available.** True branch, diff,
merge, rebase; `dolt_cherry_pick()` and cross-branch
`INSERT INTO main SELECT … AS OF branch` are *literal selective harvest*,
the operation KOS defined and no other candidate ships.

**Operational:** single static Go binary, Homebrew-installable, fully
offline, no Docker, no JVM. From Rust: sidecar `dolt sql-server` over
MySQL wire (`mysql_async`) or shelling to the CLI; or the Go embedded
library if any KOS component tolerates Go (Beads proves the pattern in
this exact ecosystem [@dolthub2026beads]). Sidecar-penalized but the
penalty is one supervised child process of a boring binary — not a
container.

**Health:** exceptional. Funded company, eight-year track record,
storage-format stability promise since 1.0, weekly releases
[@dolthub2026dolt2; @dolthub2026doltgres]. Apache 2.0.

**MDE checklist:** lock-in LOW (MySQL-dialect SQL, `dolt dump`, open
format); round-trip MODERATE (migrate, don't mirror, same as
TerminusDB); skill asymmetry LOW — the decisive win.

**Build cost on top:** EAV schema + edge tables + a thin `kos` shim
mapping cycle verbs to Dolt verbs (probe branch = `dolt branch`,
harvest = cherry-pick/select-into, promote = merge to main). Weeks, not
months; no query engine, no history engine, no conflict engine to write.

## Tier 1, candidate 2: SQLite + append-only fact log

The boring build: `rusqlite` in-process; `facts(e, a, v, tx, op)`
append-only; `tx(id, parent, branch, author, ts)` giving a transaction
DAG; current state as a view; `asof` as a tx-filter; branches as DAG
metadata; harvest as fact-copy with new tx. Mozilla's abandoned Mentat
is the direct design reference for Datomic-on-SQLite in Rust.

**P1 FIT** (edges are facts). **P2 FIT** (this *is* the datom model).
**P3 FIT** (SQL). **P4 FIT** (tx-filtered views). **P5 PARTIAL** —
SQLite is single-writer; WAL + short transactions is fine at KOS scale,
and branch-per-agent contention is nil, but it is a ceiling.
**Git-semantics PARTIAL** — you implement branch/diff/harvest yourself
(diff = set difference on fact tables — genuinely small).

**Operational: the only full FIT** — in-process from the existing Rust
CLI, zero sidecar, brew ships one binary, bedrock health, SQL symmetry.
**Cost:** the versioning layer is yours to build and own (~the size of
kos's existing `charter.rs`+`drift.rs` work, at KOS scale) and yours to
maintain forever. Lock-in: none. This is the Grudin-clean option — the
machine that pays is code you own.

## Tier 2 — probe-watch

**DoltLite** [@dolthub2026doltlite]: architecturally the ideal terminal
state — SQLite embedding *plus* Dolt version control *plus* SQL. But:
four months old, ~42 stars, author-supported in a personal repo, and by
its creator's own framing "first and foremost an experiment" in fully
AI-generated code whose DoltHub support will be reassessed on usage
[@dolthub2026doltlitesupport]. Smoke-test it (same SQL surface as Probe
A below, ~30 minutes); do not depend on it.

**Turso (Rust rewrite of SQLite)**: in-process, MIT, ~23k stars, 253
contributors, beta; MVCC `BEGIN CONCURRENT` lifts the single-writer
ceiling; CDC and a built-in MCP server mode [@explainx2026turso;
@dbdb2026turso]. Branching is metadata-instant — *in Turso Cloud's
object-store architecture*, not the local embedded engine
[@turso2026what]. Reads as the succession path for the SQLite lane once
it stabilizes: adopt candidate 2's schema on SQLite today, inherit MVCC
by swapping the engine later.

## Disqualified

| Candidate | One-line reason |
| --- | --- |
| Kuzu (+ forks) | Archived 2025-10-10; unstable on-disk format; forks unproven [@biggo2025kuzu; @register2025kuzu] |
| CozoDB | Dormant since v0.7 (2023-11); single author; niche DSL [@dbdb2026cozo] |
| XTDB v2 / Datomic / Datalevin | JVM; no branching (XTDB/Datomic); disqualified on ops |
| SurrealDB | BSL license + embedded/versioning maturity unproven (per prior report; unre-verified) |
| Oxigraph | No versioning; SPARQL asymmetry; app-layer history = candidate 2 with worse skill fit |
| Automerge/CRDT class | Automatic merge is doctrinally opposed to harvest-not-merge and the human pawl |
| Irmin | Conceptually perfect mergeable store; OCaml — impractical from Rust |
| lakeFS/DVC/Oxen | File-granularity versioning; wrong layer — no graph query |

## M4 probe design (revised)

**Probe A — Dolt.** ThreeDoors-104 as EAV facts + edge table; sidecar
`dolt sql-server` from the Rust CLI via `mysql_async`. Test: (i) full
derivation-chain reconstruction for one finding via `dolt_history`/
`dolt_diff`; (ii) `AS OF` historical query for retrodiction; (iii) probe
branch → divergent finding → **cherry-pick harvest** to main without
merge; (iv) two concurrent writers on separate branches. Optional
30-minute DoltLite smoke test of (i)–(iii) on the identical SQL.
Pre-registered `predicted_confidence`: 0.75.

**Probe B — SQLite fact log.** Same four operations implemented on
`rusqlite` within a one-session build timebox. Pre-registered
`predicted_confidence`: 0.6 that all four land inside the timebox.

**Decision rule.** If B completes all four within the timebox and the
diff/harvest code stays under an agreed size budget → choose B
(in-process wins; Turso as engine succession). Otherwise → choose A
(sidecar cost buys mature branch/merge/asof machinery). If A wins and
DoltLite's smoke test passes clean → open a watch item to migrate the
sidecar to DoltLite when it exits experimental status. Either outcome
supersedes the TerminusDB probe recommendation: **TerminusDB drops to
tier 3** — its remaining unique asset (graph-native typed schema with
versioned migrations) is not worth the daemon + Prolog + steward risk
against these two.

## Gained / lost vs TerminusDB

**Gained:** health (Dolt/SQLite are the two healthiest options in the
entire field), SQL skill symmetry (maximal LLM fluency vs minimal),
no-Docker local-first ops, an ecosystem precedent already inside the
platform (Beads on Embedded Dolt), and — Dolt only — selective harvest
as a shipped primitive. **Lost:** graph-native typed schema and
schema-migration-as-commits (replaced by SQL DDL discipline plus kos's
own validate layer — which already exists), and RDF/JSON-LD standards
alignment (never a KOS requirement). **P1–P5 impact of the loss: none
that modeling discipline doesn't cover;** the losses are conveniences,
not properties.

## Risk register (top 2)

| Risk | Candidate | Severity | Note |
| --- | --- | --- | --- |
| Sidecar lifecycle management | Dolt | Medium | Child-process supervision in the CLI; failure modes are boring and testable |
| EAV modeling discipline | Dolt | Medium | Nothing stops an UPDATE; enforce append-only via triggers/validate gate |
| Go dependency drift | Dolt | Low | Single static binary pinned by brew formula |
| Build-and-own versioning layer | SQLite | Medium-High | Weeks of code, forever yours; scope-creep is the real hazard |
| Single-writer ceiling | SQLite | Low-Med at KOS scale | Mitigated by WAL + branch metadata; Turso succession path |
| DoltLite immaturity | (watch) | High if depended on | Experimental, personal-repo, AI-generated codebase |

---

## Bibliography

```bibtex
@misc{biggo2025kuzu,
  author = {{BigGo News}},
  title = {KuzuDB, the Promising Embedded Graph Database, is Suddenly Archived},
  year = {2025},
  howpublished = {\url{https://biggo.com/news/202510130126_KuzuDB-embedded-graph-database-archived}},
  note = {Archived 2025-10-10; on-disk format never stabilized}
}
@misc{register2025kuzu,
  author = {{The Register}},
  title = {KuzuDB graph database abandoned, community mulls options},
  year = {2025},
  howpublished = {\url{https://www.theregister.com/2025/10/14/kuzudb_abandoned/}}
}
@misc{dbdb2026cozo,
  author = {{Database of Databases}},
  title = {CozoDB},
  year = {2026},
  howpublished = {\url{https://dbdb.io/db/cozodb}},
  note = {Accessed 2026-06; latest documented release v0.7, 2023-11-21}
}
@misc{dolthub2026dolt2,
  author = {Sehn, Tim},
  title = {Dolt 2.0},
  year = {2026},
  howpublished = {\url{https://www.dolthub.com/blog/2026-05-11-dolt-2-dot-0/}}
}
@misc{infoq2026dolt2,
  author = {Losio, Renato},
  title = {Version Controlled SQL Database Dolt Releases 2.0 with Automatic Storage Cleanup and Compression},
  year = {2026},
  howpublished = {\url{https://www.infoq.com/news/2026/07/dolt-version-control/}}
}
@misc{dolthub2026doltgres,
  author = {Fulghum, Jason},
  title = {Doltgres 1.0 Coming August 6th},
  year = {2026},
  howpublished = {\url{https://www.dolthub.com/blog/2026-06-26-doltgres-1-0-coming-this-fall/}}
}
@misc{dolthub2026agents,
  author = {Leng, James},
  title = {Dolt, The Database for AI Agents},
  year = {2026},
  howpublished = {\url{https://www.dolthub.com/blog/2026-05-18-database-for-ai-video/}},
  note = {Branch-per-agent isolation; review-then-merge; surgical rollback}
}
@misc{dolthub2026conflicts,
  author = {Sehn, Tim},
  title = {How Users Manage Merge Conflicts in Dolt},
  year = {2026},
  howpublished = {\url{https://www.dolthub.com/blog/?tags=dolt}},
  note = {2026-05-27 entry; cell-level conflict management}
}
@misc{dolthub2026doltlite,
  author = {Sehn, Tim},
  title = {Introducing DoltLite},
  year = {2026},
  howpublished = {\url{https://www.dolthub.com/blog/2026-03-25-doltlite/}},
  note = {SQLite-compatible embedded engine with Dolt version control as SQL functions}
}
@misc{dolthub2026doltlitesupport,
  author = {Sehn, Tim},
  title = {DoltHub Adopts DoltLite},
  year = {2026},
  howpublished = {\url{https://www.dolthub.com/blog/2026-04-14-dolthub-adopts-doltlite/}},
  note = {``first and foremost an experiment''; support reassessed on usage}
}
@misc{dolthub2026beads,
  author = {Brown, Dustin},
  title = {Restoring Beads Classic},
  year = {2026},
  howpublished = {\url{https://www.dolthub.com/blog/2026-04-02-restoring-beads-classic/}},
  note = {Embedded Dolt as Beads' default backend, no external server}
}
@misc{dolthub2026blog,
  author = {{DoltHub}},
  title = {DoltHub Blog index},
  year = {2026},
  howpublished = {\url{https://www.dolthub.com/blog/}},
  note = {Dolt MCP server extended to Doltgres, July 2026}
}
@misc{dbdb2026dolt,
  author = {{Database of Databases}},
  title = {Dolt},
  year = {2026},
  howpublished = {\url{https://dbdb.io/db/dolt}},
  note = {Cell-based conflict detection; embedded and server modes}
}
@misc{dbdb2026turso,
  author = {{Database of Databases}},
  title = {Turso},
  year = {2026},
  howpublished = {\url{https://dbdb.io/db/turso}},
  note = {Limbo rewrite history; copy-on-write branching; MVCC}
}
@misc{explainx2026turso,
  author = {{ExplainX}},
  title = {Turso Database: SQLite Rewritten in Rust with MVCC, Async I/O and Vector Search},
  year = {2026},
  howpublished = {\url{https://www.explainx.ai/blog/turso-database-sqlite-rust-rewrite-guide-2026}},
  note = {Beta; 20k+ stars; 253 contributors; MCP server mode}
}
@misc{turso2026what,
  author = {{Turso}},
  title = {What is Turso?},
  year = {2026},
  howpublished = {\url{https://turso.tech/what-is-turso}},
  note = {Metadata-only branching in cloud object-store architecture}
}
```
