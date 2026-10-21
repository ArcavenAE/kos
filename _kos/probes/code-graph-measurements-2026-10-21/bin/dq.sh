#!/bin/bash
# one query against the scratch Dolt server, under run conditions
set -euo pipefail
P="${PROBE_DIR:?set PROBE_DIR to the probe scratch directory}"
cd "$P/tmp"
exec "$(dirname "$0")/runc.sh" "$(command -v dolt)" --host 127.0.0.1 --port 42498 --no-tls --user root --password "" --use-db "${DB:-dolt-a}" sql "$@"
