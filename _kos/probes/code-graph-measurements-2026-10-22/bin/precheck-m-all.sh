cd "$PROBE_DIR/marvel" || exit 9
n=0; f=0
while read -r c; do
  n=$((n+1))
  git checkout -q --detach "$c" 2>"$PROBE_DIR/work/co.err" || { echo "$c CHECKOUT-FAIL $(head -1 "$PROBE_DIR/work/co.err")"; f=$((f+1)); continue; }
  if out=$(go list -m all 2>&1 >/dev/null); then echo "$c ok"; else f=$((f+1)); echo "$c FAIL $(printf '%s' "$out" | head -1)"; fi
done < "$PROBE_DIR/work/commits.txt"
echo "total=$n fail=$f"
