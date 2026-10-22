cd "$PROBE_DIR/marvel" || exit 9
one() { git checkout -q --detach "$1"
  if go list -deps -test ./... >/dev/null 2>"$PROBE_DIR/work/d.err" && ! grep -v 'writing stat cache' "$PROBE_DIR/work/d.err" | grep -q .; then echo "$1 ok"; return 0
  else echo "$1 FAIL $(grep -v 'writing stat cache' "$PROBE_DIR/work/d.err" | head -1)"; return 1; fi; }
echo "== positive control"; one d0616f9841bcb4bd78479d64ded42151b8ab0816 || { echo "CONTROL FAILED"; exit 3; }
echo "== full pass"; n=0; f=0
while read -r c; do n=$((n+1)); one "$c" || f=$((f+1)); done < "$PROBE_DIR/work/commits.txt"
echo "total=$n fail=$f"; if [ "$f" -gt 10 ]; then echo STOP; exit 4; fi; echo PASS
