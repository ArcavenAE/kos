cd "$PROBE_DIR/work" || exit 9
echo "== ports"; python3 -I probe.py 47997 47998 47999
echo "== public"; python3 -I -c '
import socket
c=socket.socket(); c.settimeout(5)
try: c.connect(("140.82.112.3",443)); print("public connected")
except OSError as e: print("public refused:", e.strerror)'
echo "== outside write"; ( echo x > /private/tmp/pc22-outside-probe ) 2>&1; echo "rc=$?"
echo "== inside write"; ( echo x > "$PROBE_DIR/work/inside-probe" ) 2>&1; echo "rc=$?"; cat "$PROBE_DIR/work/inside-probe"
echo "== env"; env | sort | grep -E '^(HOME|TMPDIR|MARVEL_SOCKET|PROBE_DIR)=' ; pwd
