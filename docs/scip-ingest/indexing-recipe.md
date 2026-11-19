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
GOCACHE="$SCRATCH/gocache" go list -json ./... > "$SCRATCH/golist.json"
```

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
