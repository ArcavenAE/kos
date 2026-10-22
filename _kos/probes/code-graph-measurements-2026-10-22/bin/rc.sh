#!/bin/bash
# run one command under Probe C's run conditions (cond4-1, run.sb)
# As run, this file had the scratch root, PATH and GOMODCACHE written in as
# literals; this copy names them as variables and is otherwise the same.
set -euo pipefail
S="${PROBE_DIR:?set PROBE_DIR to the probe scratch directory}"
cd "$S/work"
exec env -i HOME="$S/home" PATH="$PATH" LANG="${LANG:-en_US.UTF-8}" TMPDIR="$S/tmp" PROBE_DIR="$S" \
  GOPROXY=off GOFLAGS=-mod=readonly GOTOOLCHAIN=local GOCACHE="$S/gocache" GOMODCACHE="$(go env GOMODCACHE)" GOTELEMETRY=off \
  sandbox-exec -f "$S/run.sb" -D PROBE_DIR="$S" "$@"
