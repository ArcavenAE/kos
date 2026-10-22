cd "$PROBE_DIR/marvel" || exit 9
i=0
while read -r c; do
  i=$((i+1)); s=${c:0:8}
  git checkout -q --detach "$c"
  t0=$(python3 -I -c 'import time;print(time.time())')
  if scip-go --output "$PROBE_DIR/idx/$s.scip" >"$PROBE_DIR/idx/$s.log" 2>&1; then
    t1=$(python3 -I -c 'import time;print(time.time())')
    if python3 -I "$PROBE_DIR/bin/facts.py" "$PROBE_DIR/idx/$s.scip" "$PROBE_DIR/facts/$s.jsonl" 2>"$PROBE_DIR/facts/$s.err"; then
      t2=$(python3 -I -c 'import time;print(time.time())')
      echo "{\"i\":$i,\"commit\":\"$c\",\"status\":\"ok\",\"index_s\":$(python3 -I -c "print(round($t1-$t0,2))"),\"extract_s\":$(python3 -I -c "print(round($t2-$t1,2))")}"
    else echo "{\"i\":$i,\"commit\":\"$c\",\"status\":\"GAP\",\"stage\":\"extract\",\"err\":$(tail -1 "$PROBE_DIR/facts/$s.err" | python3 -I -c 'import json,sys;print(json.dumps(sys.stdin.read().strip()))')}"; fi
  else echo "{\"i\":$i,\"commit\":\"$c\",\"status\":\"GAP\",\"stage\":\"index\",\"err\":$(grep -iv '^\s*$' "$PROBE_DIR/idx/$s.log" | tail -1 | python3 -I -c 'import json,sys;print(json.dumps(sys.stdin.read().strip()))')}"; fi
done < "$PROBE_DIR/work/commits.txt"
echo '{"done":true}'
