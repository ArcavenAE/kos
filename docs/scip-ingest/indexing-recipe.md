# Indexing a repo for `kos facts ingest-scip`

`kos facts ingest-scip` reads an index file that an indexer already wrote. It
does not run an indexer. This page is the recipe for producing the three inputs
it takes: the index, the indexer's log, and a checkout of the repo at the
commit that was indexed.

## Keep the Go build cache outside the clone

Set `GOCACHE` to a directory outside the repo checkout before running
`scip-go`. When the cache sat beside the clone, scip-go indexed the test mains
the build generated there, which added 67 to 68 documents and 6,516 to 6,801
facts per commit to one repo in the probe that preceded this feature. The
ingest skips any document whose path leaves the repo root (`../`) and reports
how many it skipped, but that filter is the second line of defence. The first
is this setting.

```sh
mkdir -p "$SCRATCH/gocache"
cd "$CHECKOUT"
GOCACHE="$SCRATCH/gocache" scip-go index --output "$SCRATCH/index.scip" \
  > "$SCRATCH/index.log" 2>&1
echo "exit: $?" >> "$SCRATCH/index.log"
```

The last line of a scip-go log is `exit: <n>`, written by the recipe itself, so
the ingest can see the indexer's exit status. A check that refuses a failed
run reads it.

For a Go repo, also write the file set the module builds, from the same
checkout and with the same `GOCACHE`:

```sh
GOCACHE="$SCRATCH/gocache" go list -json ./... > "$SCRATCH/golist.json" 2> "$SCRATCH/golist.err"
echo "exit: $?" >> "$SCRATCH/golist.json"
```

The last line of the list is `exit: <n>`, written by the recipe, the way the
scip-go log's is. The ingest refuses a list that does not end with `exit: 0`, so
a `go list` that stopped part way is not trusted. `--golist` is for a Go index
only; the ingest refuses it with an index from any other indexer.

The ingest requires an index document for every file in that output (`GoFiles`
and `CgoFiles`, relative to `Module.Dir`). Without it, a path rule stands in:
every tracked `.go` file that is not a test, and not under `testdata`,
`vendor`, or a directory starting with `_` or `.`. That rule wrongly requires a
document for a file a build constraint excludes.

## Rust

```sh
cd "$CHECKOUT"
rust-analyzer scip . --output "$SCRATCH/index.scip" > "$SCRATCH/index.log" 2>&1
```

Index with a toolchain that has `rust-src` installed. Without it the indexer
logs `can't load standard library`, exits 0 and writes an index with no
standard-library symbols.

## Load

```sh
kos facts ingest-scip --db facts.db --at 1 --repo kos --commit "$(git -C "$CHECKOUT" rev-parse HEAD)" \
  --index "$SCRATCH/index.scip" --log "$SCRATCH/index.log" --checkout "$CHECKOUT"
```

For Go, add `--golist "$SCRATCH/golist.json"`.

## Read

Three verbs read the loaded graph. Each takes the store with `--db`, a query as
the last argument, and an optional `--at <version>` (the latest version when
left out; a version above the latest reads the latest).

```sh
kos facts refs --db facts.db 'FactStore#traverse().'    # documents that reference it
kos facts impls --db facts.db 'Reader#'                 # symbols that implement it
kos facts dependents --db facts.db 'serde 1.0'          # repos that depend on it
```

A query is a whole symbol or package, or its end where the character before it
is a space or a `/`, so a descriptor is enough. A query that matches nothing, or
more than one name, is refused with exit 1, and the refusal lists up to ten
candidates. Results go to stdout, sorted, one per line, with a backslash, a tab
and a newline escaped; the header (`symbol <name> at version <n>: <k> results`)
goes to stderr. `--json` writes one object per result and a final trailer with
the query, the resolved name, the version, the count and whether the list was
cut short. `--limit <n>` caps the results (200 by default); `--limit 0` means no
cap.
