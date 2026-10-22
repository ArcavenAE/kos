#!/bin/bash
# arms one at a time, so no G2 timing shares the machine with another load
R=/private/tmp/pc22/bin/rc.sh
$R python3 -I /private/tmp/pc22/bin/load_vt.py /private/tmp/pc22/run2/db/vt/vt.sqlite /private/tmp/pc22/work/index.jsonl /private/tmp/pc22/facts /private/tmp/pc22/run2/work/load-vt.jsonl; echo "vt rc=$?"
$R python3 -I /private/tmp/pc22/bin/load_d.py /private/tmp/pc22/run2/db/d /private/tmp/pc22/work/index.jsonl /private/tmp/pc22/facts /private/tmp/pc22/run2/work/dload /private/tmp/pc22/run2/work/load-d.jsonl; echo "d rc=$?"
$R python3 -I /private/tmp/pc22/bin/load_snap.py /private/tmp/pc22/run2/db/snap /private/tmp/pc22/work/index.jsonl /private/tmp/pc22/facts /private/tmp/pc22/run2/work/load-snap.jsonl; echo "snap rc=$?"
echo done
