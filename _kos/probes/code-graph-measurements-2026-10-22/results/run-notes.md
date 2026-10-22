# Probe C run notes (kinu)

Seat: arcaven-research-supervisor-g24-0. Request: arcaven-supervisor 01M5GS67AV,
under director ruling 01M5GS5KZ9. Brief: kos da21d201,
`_kos/probes/brief-fact-store-probe-c-history-growth.md`, sha256
13da98e250d3b33a22bb07ed5c07621467e5ec433083af34fb475731e8fe9db3, locked.
Run started 2026-10-22T03:07:00Z.

## Notes from review 5608782453, recorded at run start

1. (2026-10-22, dated note) The brief's revision line (:182 area) says "before
   the lock". It probably means before the run: every edit it names was made
   before any measurement. Read it that way; nothing in the brief changed after
   the lock.
2. The disk budget (about 8 GB) counts the 200 indexes and S-snap's 2 GB stop.
   It leaves out the fact files and the S-vt and D stores. The free-space check
   reports all five.
3. The seed pool size is reported, whether or not it reaches 100.
4. If G0 is decided by D-only VARCHAR(255) load failures, the report names it
   as such: a D schema limit, not an indexing GAP.
5. The hot-set empty counts are given per as-of commit, beside the seed counts.
6. The G3 pairs are sampled from the list of indexed commits (GAP commits
   excluded), sorted by commit order, as the brief's Draws line says; the
   report names that list and its sha256.

## Profile deviation (2026-10-22T03:12Z, dated note)

The brief says a run host whose live services listen on other localhost ports
adds a deny for each. kinu listens on more than the five named ports
(`lsof -iTCP -sTCP:LISTEN` at 03:10Z, 27 ports). `run.sb` keeps the brief's
profile verbatim and adds, between the 3307 deny and the final 47999 deny, a
deny for each listener seen, plus 7422 (named in the brief, not listening).
Every addition only narrows what the run can reach; no probe step dials any
of them. One extra control port, 47997, is denied FIRST in the block, so the
controls show the first and the last end deny both hold. Listener ports:
1884 4222 4242 5000 6463 7000 7442 7422 8222 8242 11434 26000 28186 28196 28198 32222 36722 41343 42050 48969 51612 54992 55206 55612 57321 60260 64260. Ephemeral listeners that appear after 03:10Z are not denied; the
Dolt server's random port is chosen outside this set.

## Pre-check change (2026-10-22T03:14Z, dated note)

Director ruling 01M5GSHQA6 (B, conditional on the brief's reviewer), relayed
by arcaven-supervisor 01M5GSJ0F7; the brief's author and reviewer agreed with four
conditions (01M5GSJ239); GO from arcaven-supervisor 01M5GSJVC4, 2026-10-22.

Original result, kept: `go list -m all` offline failed at 59 of 200 commits
(precheck.txt). All 59 miss the same two modules, golang.org/x/sync@v0.24.0
and golang.org/x/tools@v0.51.0, which are in the module graph but loaded by
no package (precheck-missing-modules.txt). Under the brief as written that is
a stop.

Change: the pre-check is `go list -deps -test ./...`, the package graph a
build and test load, under the same offline flags (GOPROXY=off
GOFLAGS=-mod=readonly GOTOOLCHAIN=local) and with the same stop at more than
10 failures. d0616f98 is among the 59. An exploratory pass of the new form
resolved at all 200 (precheck-buildgraph.txt, fail=0). Under this note it runs
again in order: first a positive control at d0616f98 alone, which must pass,
then the full 200. The timebox starts when that full pass passes.

Why G0 is unchanged: the pre-check only predicts G0, and the 59 get no
allowance. G0 itself still counts
every commit whose scip-go index fails to build, or whose facts fail to load,
under GOPROXY=off, with its error line. A commit that needed either module
would show up there as a GAP.

## Load restart (2026-10-22T03:25Z, dated note)

The three arm loaders were first started together at 03:22:35Z. That would
have put every G2 timing on a machine shared with two other loads, so they
were stopped by pid (kill -TERM, then kill -0) after 5 to 62 commits, and
every arm was loaded again from empty, one arm at a time, into fresh store
directories under `run2/`, starting 03:25:11Z. Only the `run2/` loads are
scored. The first partial stores were left in place: a guard refused their
removal, and they are not read. Order: S-vt, then D, then S-snap.

## Run notes after the loads (2026-10-22T03:55Z, dated note)

- G4 hot set, D: run 1 only (20 commits, hop sets checked). Runs 2 and 3 were
  stopped by pid after run 1, about 30 min each; the hot set is reported,
  not scored. S-vt's hot set ran all 3.
- G5 one-commit D store: the first load failed ("dolt table rc=1") because the
  sql-server held the repo. Loaded again with the server stopped; the failed
  attempt's log is kept as load-d1.failed-server-held.jsonl.
- S-snap stop: the 2 GB stop read `du -sk` during the load, which counts the
  WAL. It stopped at commit 47 at 2,100,012 KB. After the checkpoint, the
  allocated size is 1,294,400 KB at 47 commits.
- mde_d.py: the first run stopped on `KeyError: 'rows'`, because dolt prints
  `{}` for an empty result. The reader was changed to treat that as no rows,
  and the run repeated. The failed run had written nothing to the re-import repo.
- rc.sh changes into the work dir, so one `dolt status` check ran outside the
  repo; that check was discarded. No measurement used it.
