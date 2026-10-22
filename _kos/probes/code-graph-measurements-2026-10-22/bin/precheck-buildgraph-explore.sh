cd "$PROBE_DIR/marvel" || exit 9
n=0; f=0
while read -r c; do
  n=$((n+1)); git checkout -q --detach "$c"
  if go list -deps -test ./... >/dev/null 2>"$PROBE_DIR/work/d.err" && ! grep -v 'writing stat cache' "$PROBE_DIR/work/d.err" | grep -q .; then echo "$c ok"
  else f=$((f+1)); echo "$c FAIL $(grep -v 'writing stat cache' "$PROBE_DIR/work/d.err" | head -1)"; fi
  echo "$c $(go list -e -m -json all 2>/dev/null | python3 -I -c '
import json,sys
d=sys.stdin.read(); dec=json.JSONDecoder(); i=0; bad=[]
while i<len(d):
    while i<len(d) and d[i].isspace(): i+=1
    if i>=len(d): break
    o,i=dec.raw_decode(d,i)
    if o.get("Error"): bad.append(o["Path"]+"@"+str(o.get("Version")))
print("missing:"+",".join(bad))')" >> "$PROBE_DIR/work/missing.txt"
done < "$PROBE_DIR/work/commits.txt"
echo "total=$n fail=$f"
