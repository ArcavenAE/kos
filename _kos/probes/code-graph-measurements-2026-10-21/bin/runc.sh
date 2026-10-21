#!/bin/bash
# run a command under the probe's run conditions (cond4-1)
set -euo pipefail
P="${PROBE_DIR:?set PROBE_DIR to the probe scratch directory}"
GOMODCACHE="${GOMODCACHE:-$HOME/go/pkg/mod}"
exec env -i HOME=$P/home PATH="$PATH" LANG="${LANG:-en_US.UTF-8}" TMPDIR=$P/tmp \
  GOPROXY=off GOFLAGS=-mod=readonly GOTOOLCHAIN=local GOCACHE=$P/gocache GOMODCACHE="$GOMODCACHE" \
  sandbox-exec -D PROBE_DIR="$P" -f "$(dirname "$0")/run.sb" "$@"
