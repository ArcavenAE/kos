#!/usr/bin/env bash
# Measure SCIP index cost for one repo. Sequential use only.
# usage: measure.sh <rust|go> <repo-name>
set -uo pipefail

SP=/private/tmp/claude-501/-Users-michael-pursifull-work-aae-orc/b9280faf-90c6-4ad2-a353-f711ff60f773/scratchpad/scip-cost
FLEET=/Users/michael.pursifull/work/aae-orc
RA=/Users/michael.pursifull/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rust-analyzer
SCIPGO=/Users/michael.pursifull/.local/share/mise/installs/go/1.26.5/bin/scip-go

lang=$1; repo=$2
out="$SP/idx/$repo.scip"
mkdir -p "$SP/idx" "$SP/logs"

run_once() {
  local phase=$1
  local log="$SP/logs/${repo}.${phase}.log"
  local t0 t1
  t0=$(date "+%Y-%m-%dT%H:%M:%S%z")
  if [ "$lang" = rust ]; then
    ( cd "$FLEET/$repo" && /usr/bin/time -l "$RA" scip . --output "$out" ) >"$log" 2>&1
  else
    ( cd "$FLEET/$repo" && /usr/bin/time -l "$SCIPGO" index --output "$out" ) >"$log" 2>&1
  fi
  local rc=$?
  t1=$(date "+%Y-%m-%dT%H:%M:%S%z")
  local real rss size
  real=$(grep -E '^\s*[0-9.]+ real' "$log" | tail -1 | awk '{print $1}')
  rss=$(grep -E 'maximum resident set size' "$log" | tail -1 | awk '{print $1}')
  size=$(stat -f %z "$out" 2>/dev/null || echo 0)
  echo "$repo|$lang|$phase|rc=$rc|real=${real:-NA}|rss_bytes=${rss:-NA}|scip_bytes=$size|start=$t0|end=$t1" \
    | tee -a "$SP/results.txt"
}

run_once cold
run_once warm

# repo cleanliness check
( cd "$FLEET/$repo" && echo "GITSTATUS $repo: [$(git status --short | head -5 | tr '\n' ';')]" ) \
  | tee -a "$SP/results.txt"
