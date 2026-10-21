import os, socket, sys
print("MARVEL_SOCKET in env:", "MARVEL_SOCKET" in os.environ, "HOME:", os.environ.get("HOME"))
s = socket.socket(); s.settimeout(3)
try:
    s.connect(("140.82.112.3", 443)); print("net: CONNECTED")
except Exception as e:
    print("net:", repr(e))
for p in ("/private/tmp/fsp-outside-probe", sys.argv[1]):
    try:
        open(p, "w").close(); print("write ok:", p)
    except Exception as e:
        print("write refused:", p, repr(e))
