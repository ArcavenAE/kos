#!/bin/bash
# G4 runs for one arm. Usage: g4.sh <arm> <url> <seeds> <modefile> <outdir> <runs>
# modefile lines: "<i> <commit> <mode>"
set -euo pipefail
arm=$1 url=$2 seeds=$3 modes=$4 od=$5 runs=$6
mkdir -p "$od"
for r in $(seq 1 "$runs"); do
  while read -r i c m; do
    line=$("$PROBE_DIR/target/release/trav" "$url" "$seeds" "$od/$arm-r$r-${c:0:8}.jsonl" "$m")
    echo "{\"arm\":\"$arm\",\"run\":$r,\"i\":$i,\"commit\":\"$c\",\"summary\":\"$line\"}"
  done < "$modes"
done
