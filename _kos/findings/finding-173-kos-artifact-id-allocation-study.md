# finding-173: bd's id allocation pinned to source, and what it implies for kos artifact ids

**Date:** 2026-09-16
**Status:** RULED 2026-09-16 (see the ruling section below). The study's
recommendation was accepted for node ids and overruled for finding numbers.
Implementation is separate work, tracked in bd; no allocator change has shipped.
**Scope:** kos-tool id design. The study finding-131 asked for ("exactly how bd
generates the suffix") and the weighing of the candidates on the table for
question-artifact-id-allocation. A finding **for kos**, filed here by the
subject test.
**Brief:** `_kos/probes/brief-artifact-id-allocation.md`. **Question node:**
`question-artifact-id-allocation`.
**Method:** a read-only study subagent confirmed each element of the bd
algorithm the architect seat had already verified against the live db, pinning
it to file and line references at `forks/beads` `0c5fa422d` (bd 1.1.2 era),
rather than re-deriving it; the research-supervisor seat spot-checked four of
the references by hand (hash input line, adaptive rule and defaults, retry
loop, kos allocator). Comparators were fetched, not remembered; sources not
fetched are named as not cited. Live counts are from `bd sql` against the kinu
server on 2026-09-16.
**Number allocation note:** this finding took 173 by read-max-plus-one across
the orc and kos graphs and the #335 branch at 05:50 UTC, the very mechanism
under study. If it collides, the first-committed keeps the number and the later
file renumbers with a note.


## Ruling (operator, 2026-09-16, relayed by the director seat)

The decision of record for both collision classes:

- **Node ids: ACCEPTED.** The namespace prefix `graph::slug` is the canonical
  cross-graph reference. A bare slug stays legal inside its own graph. This
  ratifies what four subrepo graphs already do unmanaged; `kos validate`
  learns the syntax and stops warning on it.
- **Finding numbers: allocation-at-merge (`aae-orc-vxaa`) REJECTED. Opaque
  ids ADOPTED NOW**, bd-style (`prefix-{short hash}`), not deferred. The
  study's reason for deferring (readability spent per finding) was weighed
  and overruled; the property that decided it is that an opaque id minted at
  authoring removes both the race and the fan-out case with no merge-time
  step that a session can skip or that depends on CI the orc does not have.
- The allocator change (namespace-prefix resolution for nodes; opaque id
  minting for findings) is implementation, captured as its own bd tickets
  with this ruling as the decision of record: `aae-orc-qvpr8` (graph::slug
  resolver and validate) and `aae-orc-wmna4` (opaque finding ids minted at
  authoring). `aae-orc-vxaa` closed as rejected; `aae-orc-hf58k` reopened
  with the false-premise note; `aae-orc-ul2h` and `aae-orc-qso2f` annotated. The recommendation in section
  5 below is kept verbatim as the study's position; where it differs from
  the ruling, the ruling stands.

## Summary

- The relayed bd algorithm is confirmed in every element. The live db matches the code's thresholds to the unit: 164 `aae-orc` ids at length 3, 820 at length 4, 155 at length 5; the computed boundaries are 164 and 984.
- Two details the relay glossed: the hash consumes only 2 to 5 leading bytes chosen by target length, and the retry loop walks every length from base to 8 (up to 60 attempts), not one growth step.
- bd's hash gives uniqueness within one prefix, checked against a serialized store. Cross-project uniqueness comes from the prefix alone.
- `engdocs/COLLISION_MATH.md` drifts on the table start (4 vs 3), its threshold table contradicts its own formula, and it understates the retry count.
- Recommendation: two collision classes, two fixes. Ratify the namespace prefix (already deployed unmanaged) for node ids across graphs; move finding-number allocation to merge (aae-orc-vxaa) with the slug as citation key. Defer opaque ids.

## 1. bd's algorithm, pinned

Two implementations exist (legacy `issueops`, newer `domain`); `adaptive.go:6-7` says "Both copies must agree", and they do. Refs give the `domain` path first.

| Element | Verified value | Code ref (0c5fa422d) | Quote |
|---|---|---|---|
| Hash input, separator | `title\|description\|creator\|UnixNano\|nonce`; timestamp in nanoseconds | `internal/idgen/hash.go:58` | `fmt.Sprintf("%s\|%s\|%s\|%d\|%d", title, description, creator, timestamp.UnixNano(), nonce)` |
| Hash | sha256 | `hash.go:61` | `hash := sha256.Sum256([]byte(content))` |
| Bytes consumed | leading 2,3,4,4,5,5 bytes for L=3..8 | `hash.go:63-80` | `case 3: numBytes = 2 // 2 bytes = 16 bits ≈ 3.09 base36 chars` |
| Alphabet, base | base36 `0-9a-z`, big-int conversion | `hash.go:12,16-31` | `const base36Alphabet = "0123456789abcdefghijklmnopqrstuvwxyz"` |
| Truncation | left-pad `0` to L; if longer keep the least significant L digits (mod 36^L) | `hash.go:39-47` | `// Truncate to exact length if needed (keep least significant digits)` |
| Id shape | `prefix-suffix` | `hash.go:84` | `fmt.Sprintf("%s-%s", prefix, shortHash)` |
| Adaptive rule | first L in [Min,Max] with `1 - exp(-n^2/(2*36^L)) <= 0.25`, else Max | `internal/storage/domain/adaptive.go:25-36` (dup `issueops/helpers.go:381-391`) | `exponent := -float64(numIssues*numIssues) / (2.0 * totalPossibilities)` |
| Defaults | 0.25, Min 3, Max 8 | `adaptive.go:15-21` | `MaxCollisionProbability: 0.25, MinLength: 3, MaxLength: 8` |
| Overrides | config keys `max_collision_prob`, `min_hash_length`, `max_hash_length` | `domain/db/config.go:222-249` | `r.GetConfig(ctx, "max_collision_prob")` |
| Per-prefix count n | `COUNT(*)` of ids with the prefix, child ids (`.` in suffix) excluded, all statuses, counted before insert | `domain/db/issue.go:464-482` (dup `helpers.go:316-330`) | `WHERE id LIKE CONCAT(?, '-%%') AND INSTR(SUBSTRING(id, LENGTH(?) + 2), '.') = 0` |
| Retry and grow | for L=base..Max, nonce 0..9: mint, `Exists`, first free wins; error after all | `domain/issue.go:1351-1364` (dup `helpers.go:196-213`) | `for length := baseLength; length <= cfg.MaxLength; length++ { for nonce := 0; nonce < 10; nonce++ {` |
| Timestamp stability | `CreatedAt` set once before minting | `domain/issue.go:680-685` | `// Set CreatedAt before the mint path: GenerateHashID hashes timestamp` |
| Counter mode (not in relay) | opt-in `issue_id_mode=counter` gives `prefix-N` from an `issue_counter` table inside the write | `domain/issue.go:1320-1335` | `if mode == "counter" { n, err := u.issueRepo.NextCounterID(ctx, prefix)` |

Corrections to the relay: "leading sha256 bytes" means 2 to 5 bytes chosen by L; "grow after ten misses" continues through L=8 (60 attempts from base 3). All else matches.

Boundaries from the code's formula (n = issues already present): L=3 to n=163, L=4 to 983, L=5 to 5,898, L=6 to 35,389, L=7 to 212,339. Live check via `bd sql` (child ids excluded): 164 at L=3, 820 at L=4 (164+820=984), 155 at L=5. Exact.

Hedge, not verified against maintainer intent: 36^L overstates the space at L=8 (5 bytes = 2^40 ≈ 1.1e12 < 2.8e12), and at L=3, 16 bits map mod 46656, slightly non-uniform. Neither matters at fleet scale.

## 2. Doc drift (`engdocs/COLLISION_MATH.md`)

- **Table start.** Tables begin at `4-char` (lines 19, 42); "Default Thresholds" opens `0-500 | 4 chars` (line 61). Code: `MinLength: 3`; 164 live ids are 3 chars.
- **Threshold table contradicts its own formula.** Doc: `501-1,500 | 5 chars` (line 62). By its own formula at 25%, 4 chars hold to n=983; the doc's own row `1,000 | 25.75%` shows the crossing near 1,000, not 500.
- **Formula and threshold agree.** `P(collision) ≈ 1 - e^(-n²/2N)`, `N = 36^length` (lines 10, 15); "exceeds **25%**" (line 55).
- **Retry count.** Doc: base, base+1, base+2, "**Total: 30 attempts**" (lines 102-106). Code walks base through 8. The example showing nonce 0 and 1 both giving `bd-a3f2` (lines 109-110) is implausible since the nonce is hashed.
- Absent from the doc: counter mode, child-id exclusion, the per-length byte table.

## 3. Comparators

**git** (git-scm.com; the WebFetch summary of git-config misreported "minimum length of 7", so the raw page was pulled with curl and is quoted). `core.abbrev`: "If unspecified or set to "auto", an appropriate value is computed based on the approximate number of packed objects in your repository, which hopefully is enough for abbreviated object names to stay unique for some time. ... The minimum length is 4." `git rev-parse --short[=<length>]`: "Same as --verify but shortens the object name to a unique prefix with at least length characters. The minimum length is 4, the default is the effective value of the core.abbrev configuration variable."

git keeps two layers that never trade off inside one string: the full object name is the opaque canonical id; humans use refs (branches, tags) and abbreviations computed at display time from the current object count, never stored. That is the difference from bd, which fixes the short id at write time against a store. Both scale length with population; only git can lengthen retroactively.

**nanoid** (README fetched): "returns an ID with 21 characters (to have a collision probability similar to UUID v4) ... Don't forget to check the safety of your ID size in our ID collision probability calculator." Same birthday arithmetic as bd, offered as a sizing tool rather than an adaptive rule.

**ULID** (spec README fetched): "Canonically encoded as a 26 character string", "Uses Crockford's base32", "Lexicographically sortable", 128 bits. Uniqueness by randomness plus timestamp; no store, no readability.

## 4. What this implies for kos

kos today: node ids are slugs, unique per graph by filename; finding numbers are `max+1` over the local `_kos/findings/` (`kos/src/process.rs:283-300`). Both allocate from a local view with no serialized store. bd's design rests on that store (the `Exists` check and per-prefix `COUNT`); without it, bd's hash still separates fan-out arms (nanosecond `created_at` plus nonce differ) but its collision check degrades to whatever the arm's base commit shows.

| Option | Parallel race | Fan-out determinism | Cross-graph collision | Renumber breaks citations | Cost |
|---|---|---|---|---|---|
| (a) vxaa: slug-only authoring, number at merge | fixed (allocation moves to the serialization point) | fixed | untouched | fixed if slug is the citation key | a merge-time step that must exist and run; unnumbered files pre-merge |
| (b) bd-style `graph-{hash}` | fixed | fixed (arms hash differently) | fixed by the prefix, not the hash | fixed | readable handle lost in edges, prose, filenames, rendered charters; existence check stale under fan-out, so ul2h's detector still needed |
| (c1) namespace prefix `graph::slug` | untouched | untouched | fixed for exact-id clashes; unambiguous cross-graph edges | n/a | near zero; already deployed unmanaged (marvel, forestage, curtain, switchboard edges); validate warns today |
| (c2) two-layer opaque id + slug alias | fixed | fixed | fixed | fixed | highest: new field on ~2,000 nodes, a resolver, a minting pass |
| (c3) hybrid `slug-{shortid}` | fixed for the id | fixed | reduced without namespace | fixed | longer filenames; renames still rename; two strings kept in sync |

What bd's measured design says:

- bd separates the namespacing job (prefix) from the intra-namespace job (hash plus store check). finding-131's 13 cross-graph collisions are a prefix problem; bd would solve them with `marvel-` versus `aae-orc-`, never with the suffix. That is (c1).
- bd's rule puts a 100 to 200 node graph at L=3 or 4. Three or four opaque chars per node is the readability price of (b) or (c3), the same price finding-131 already calls "difficult for humans" at bd's scale.
- bd hashes nanosecond `created_at` plus a nonce, so fan-out arms from one base commit mint different ids without coordination. A sequential counter has the opposite property. (a) does not fix this by hashing; it makes the counter's input the merge order, serialized by construction.
- bd ships a sequential mode (`issue_id_mode=counter`) and makes it safe by allocating inside the write transaction. kos has no write transaction except merge. Number-at-merge is counter mode with merge as the transaction.
- bd never re-mints when length grows; 3, 4, and 5 char suffixes coexist live. No option here requires migrating existing findings or slugs.

## 5. Recommendation and tradeoffs

For the operator, not a decision: **adopt (c1) and (a) together; defer (b), (c2), (c3).**

(c1) ratifies what the graphs already do: `graph::slug` is the canonical cross-graph reference, a bare slug stays legal inside its graph, `kos validate` learns the syntax and stops warning. (a) removes the finding-number race and the fan-out failure at their source, with the slug as citation key so renumbering never breaks a reference again. ul2h's detector (kos PR #101) stays as backstop and needs wiring (aae-orc-bs28, advisory).

Tradeoffs. (c1) fixes exact-id clashes only; finding-131's larger mass (same concept under different ids) is a content problem no id scheme touches. (a) depends on a step that runs at merge; aae-orc has no `.github/` and no lefthook (per ul2h notes), so until then the step is a person running a `kos` verb, and a finding never merged never gets a number. Neither gives kos an opaque canonical id; if graphs later federate into one store, (c2) may return, and bd's mixed-length coexistence suggests that migration can be additive. (c3) is the fallback if within-graph slug collisions under fan-out are ever measured; today they are not.

## 6. Recommended ticket and finding updates

**aae-orc-ul2h (detect-first).** Add: (1) wire `kos validate` into a run on aae-orc PRs (bs28, advisory), since the detector fires nowhere automatically; (2) a fleet-scope check (`kos validate --fleet` or an `aq` recipe over `repos.yaml`) reporting exact-id collisions across graphs, making the finding-131 sweep repeatable; (3) the "allocation guidance" item should read "author slug-only; do not allocate a number" once vxaa ships, and carry the fan-out caveat until then. bd shows why detection is the right first half: even its hash needs a store-backed existence check.

**aae-orc-vxaa (number-at-merge).** The study answers its open questions: number in the filename only, slug as canonical citation key (bd's id is a handle; `created_at` is hashed, never shown); no migration of existing findings (bd's live 3/4/5-char coexistence is the precedent); a never-merged finding needs no number; `kos validate` should WARN, not FAIL, on a numbered finding off the default branch; subrepo graphs keep independent numbering (bd counts per prefix; the graph is the prefix); the allocator reads `max+1` from the merged tree, the ordering bd gets from `COUNT` inside the insert. Add one question the study raises: who runs the allocator (merger-run `kos` verb versus CI commit), since aae-orc has no CI today.

**aae-orc-qso2f (relocations).** With (c1) ratified, a relocation keeps the slug and changes the namespace; the forwarding record is `aae-orc::slug -> marvel::slug`, and orc edges naming the bare slug are rewritten to namespaced form. No opaque id is needed to execute the 38 rows; the id dependency (hf58k, closed) is met by ratifying (c1), leaving the move primitive (0ges9) as the blocker. Under (c2) relocations become trivial but wait on minting ids for every node.

**finding-131, the "study: exactly how bd generates the suffix" paragraph.** Can now say: base36 of the leading 2 to 5 bytes of sha256 over `title|description|creator|created_at_ns|nonce`, at the smallest length in 3..8 whose birthday-bound collision probability over the per-prefix count stays at or under 0.25, retried with ten nonces per length and grown through 8 against a store-backed existence check (`internal/idgen/hash.go`, `internal/storage/domain/adaptive.go`, `internal/storage/domain/issue.go:1351-1364` at 0c5fa422d). The prefix does all cross-project namespacing; the hash never does. bd also offers a counter mode allocated inside the write transaction. The design transfers to kos only where kos has a serialization point, and kos's only one is merge. Note the doc drift so a future reader trusts the code.

## 7. Sources fetched and code refs

Fetched, full page text via curl after a WebFetch summary misreported one figure: `https://git-scm.com/docs/git-config` (core.abbrev), `https://git-scm.com/docs/git-rev-parse` (--short, --disambiguate). Fetched, README only: `https://raw.githubusercontent.com/ai/nanoid/main/README.md`, `https://raw.githubusercontent.com/ulid/spec/master/README.md`. Not fetched, not cited: k8s, Sqids, Nix, UUIDv7.

Code at `forks/beads` `0c5fa422d`: `internal/idgen/hash.go`; `internal/storage/domain/adaptive.go`; `internal/storage/domain/issue.go:672-700, 1309-1364`; `internal/storage/domain/db/issue.go:464-482`; `internal/storage/domain/db/config.go:222-249`; `internal/storage/issueops/helpers.go:170-213, 316-391`; `engdocs/COLLISION_MATH.md`. kos at `be57ad5`: `src/process.rs:283-300`. Live data: `bd sql` against the kinu server, 2026-09-16.
