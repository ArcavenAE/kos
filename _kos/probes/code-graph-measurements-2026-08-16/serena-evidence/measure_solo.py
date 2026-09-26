#!/usr/bin/env python3
"""Attributed measurement: sample RSS of ONLY the descendants of the Serena
process we spawn, so a concurrent rust-analyzer on the same machine cannot
contaminate the number. Also measures cold-start twice to test persistence
across a server restart.

Usage: measure_solo.py <project> <label>
"""
import json
import subprocess
import sys
import threading
import time
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mcpdrive import MCP


def descendants(root):
    """All descendant pids of root, via repeated pgrep -P."""
    out, frontier = set(), [root]
    while frontier:
        p = frontier.pop()
        try:
            kids = subprocess.run(["pgrep", "-P", str(p)],
                                  capture_output=True, text=True).stdout.split()
        except Exception:
            kids = []
        for k in kids:
            k = int(k)
            if k not in out:
                out.add(k)
                frontier.append(k)
    return out


def rss_of(pids):
    if not pids:
        return {}
    try:
        r = subprocess.run(["ps", "-o", "pid=,rss=,comm=", "-p",
                            ",".join(str(p) for p in pids)],
                           capture_output=True, text=True).stdout
    except Exception:
        return {}
    d = {}
    for line in r.strip().splitlines():
        parts = line.split(None, 2)
        if len(parts) >= 3:
            d[int(parts[0])] = (int(parts[1]), parts[2])
    return d


class Sampler(threading.Thread):
    def __init__(self, root, interval=0.5):
        super().__init__(daemon=True)
        self.root, self.interval, self.stop = root, interval, False
        self.peak = {}      # pid -> (peak_kb, comm)
        self.samples = 0

    def run(self):
        while not self.stop:
            pids = descendants(self.root) | {self.root}
            for pid, (kb, comm) in rss_of(pids).items():
                base = os.path.basename(comm)
                prev = self.peak.get(pid, (0, base))[0]
                if kb > prev:
                    self.peak[pid] = (kb, base)
            self.samples += 1
            time.sleep(self.interval)


QUERIES = {
    "/Users/michael.pursifull/work/aae-orc/kos": [
        ("cold", "find_referencing_symbols",
         {"name_path": "Confidence", "relative_path": "src/model.rs"}),
        ("warm", "find_referencing_symbols",
         {"name_path": "Confidence", "relative_path": "src/model.rs"}),
    ],
    "/Users/michael.pursifull/work/aae-orc/marvel": [
        ("cold", "find_implementations",
         {"name_path": "Adapter", "relative_path": "internal/runtime/adapter.go"}),
        ("warm", "find_implementations",
         {"name_path": "Adapter", "relative_path": "internal/runtime/adapter.go"}),
    ],
}


def one_run(project, tag):
    m = MCP(project, f"solo_{tag}.stderr")
    s = Sampler(m.p.pid)
    s.start()
    t0 = time.time()
    m.initialize()
    rec = {"tag": tag, "project": project,
           "initialize_s": round(time.time() - t0, 3), "queries": []}
    for label, tool, args in QUERIES[project]:
        t = time.time()
        try:
            m.call(tool, args)
            err = None
        except Exception as e:
            err = str(e)
        rec["queries"].append({"label": label, "seconds": round(time.time() - t, 3),
                               "error": err})
    # let the server settle so peak memory is captured after indexing
    time.sleep(6)
    s.stop = True
    s.join(timeout=3)
    rec["rss_peak_mb"] = {f"{c}:{p}": round(kb / 1024, 1)
                          for p, (kb, c) in sorted(s.peak.items(),
                                                   key=lambda x: -x[1][0])}
    rec["total_peak_mb"] = round(sum(kb for kb, _ in s.peak.values()) / 1024, 1)
    rec["samples"] = s.samples
    m.close()
    return rec


if __name__ == "__main__":
    project, label = sys.argv[1], sys.argv[2]
    out = []
    # run 1: genuine cold. run 2: after restart, tests cache persistence.
    for i in (1, 2):
        r = one_run(project, f"{label}-run{i}")
        out.append(r)
        print(json.dumps(r, indent=2), flush=True)
        time.sleep(2)
    json.dump(out, open(f"solo_{label}.json", "w"), indent=2)
