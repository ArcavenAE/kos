#!/usr/bin/env python3
"""Minimal MCP stdio client to drive Serena programmatically with per-call timing.

Usage: mcpdrive.py <project_path> <script.json>

script.json is a list of {"tool": name, "args": {...}, "label": "W1"} entries.
Emits JSON lines with timing to stdout, server stderr to <out>.stderr.
"""
import json
import subprocess
import sys
import time
import threading
import os

SERENA = [
    "/opt/homebrew/bin/uvx", "--from", "git+https://github.com/oraios/serena",
    "serena", "start-mcp-server", "--transport", "stdio",
    "--enable-web-dashboard", "false", "--enable-gui-log-window", "false",
    "--open-web-dashboard", "false", "--log-level", "INFO",
]


class MCP:
    def __init__(self, project, stderr_path):
        cmd = SERENA + ["--project", project]
        self.errf = open(stderr_path, "wb")
        self.t_spawn = time.time()
        self.p = subprocess.Popen(
            cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=self.errf, text=True, bufsize=1,
        )
        self.id = 0

    def _send(self, obj):
        self.p.stdin.write(json.dumps(obj) + "\n")
        self.p.stdin.flush()

    def _read_until_id(self, want, timeout=600):
        deadline = time.time() + timeout
        while time.time() < deadline:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError("server closed stdout")
            line = line.strip()
            if not line:
                continue
            try:
                msg = json.loads(line)
            except json.JSONDecodeError:
                continue
            if msg.get("id") == want:
                return msg
        raise TimeoutError(f"no response to id {want} in {timeout}s")

    def request(self, method, params=None, timeout=600):
        self.id += 1
        i = self.id
        self._send({"jsonrpc": "2.0", "id": i, "method": method,
                    "params": params or {}})
        return self._read_until_id(i, timeout)

    def notify(self, method, params=None):
        self._send({"jsonrpc": "2.0", "method": method, "params": params or {}})

    def initialize(self):
        r = self.request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "mcpdrive", "version": "0.1"},
        })
        self.notify("notifications/initialized")
        return r

    def call(self, tool, args, timeout=600):
        return self.request("tools/call",
                            {"name": tool, "arguments": args}, timeout)

    def close(self):
        try:
            self.p.stdin.close()
        except Exception:
            pass
        try:
            self.p.wait(timeout=15)
        except Exception:
            self.p.kill()
        self.errf.close()


def main():
    project = sys.argv[1]
    script = json.load(open(sys.argv[2]))
    outbase = sys.argv[3]

    m = MCP(project, outbase + ".stderr")
    results = []

    t0 = time.time()
    m.initialize()
    t_init = time.time() - t0
    results.append({"label": "__initialize__", "seconds": round(t_init, 3)})
    print(json.dumps(results[-1]), flush=True)

    for step in script:
        label = step.get("label", step["tool"])
        t = time.time()
        err = None
        try:
            r = m.call(step["tool"], step.get("args", {}),
                       step.get("timeout", 600))
        except Exception as e:
            r = None
            err = f"{type(e).__name__}: {e}"
        dt = time.time() - t
        text = ""
        if r and "result" in r:
            for c in r["result"].get("content", []):
                if c.get("type") == "text":
                    text += c["text"]
            if r["result"].get("isError"):
                err = err or "isError"
        elif r and "error" in r:
            err = json.dumps(r["error"])[:400]
        rec = {
            "label": label, "tool": step["tool"], "args": step.get("args", {}),
            "seconds": round(dt, 3), "error": err,
            "chars": len(text), "text": text,
            "ts": time.strftime("%H:%M:%S"),
        }
        results.append(rec)
        print(json.dumps({k: v for k, v in rec.items() if k != "text"}),
              flush=True)

    m.close()
    with open(outbase + ".json", "w") as f:
        json.dump(results, f, indent=2)


if __name__ == "__main__":
    main()
