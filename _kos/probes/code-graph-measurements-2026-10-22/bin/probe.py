import socket, sys, threading
ports = [int(p) for p in sys.argv[1:]]
srvs = []
for p in ports:
    s = socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("127.0.0.1", p)); s.listen(4); srvs.append(s)
    threading.Thread(target=lambda s=s: [s.accept() for _ in range(4)], daemon=True).start()
for p in ports:
    c = socket.socket(); c.settimeout(2)
    try:
        c.connect(("127.0.0.1", p)); print(p, "connected")
    except OSError as e:
        print(p, "refused:", e.strerror)
    finally:
        c.close()
