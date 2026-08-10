#!/usr/bin/env python3
"""w-seclayout — enumerate the 380 from THIS lane's own scan, and print the
per-TU fields §11.4 item 8 says to read (`gate_cause`, not `fn_names`).

  pick380.py <base.jsonl> [--keys]
"""
import json
import sys


def rows(path):
    for line in open(path):
        d = json.loads(line)
        if d.get("record") == "provenance":
            continue
        yield d


def main():
    path = sys.argv[1]
    if "--keys" in sys.argv:
        ks = set()
        for d in rows(path):
            ks |= set(d.get("bind_checks", {}))
        for k in sorted(ks):
            print(k)
        return

    precise = []
    gate = []
    for d in rows(path):
        b = d.get("bind_checks", {})
        if b.get("selbind-emit-subset-scan-precise-tus"):
            precise.append(d)
        if b.get("selbind-emit-subset-gate-tus"):
            gate.append(d)
    gate_srcs = {d["src"] for d in gate}
    target = [d for d in precise if d["src"] not in gate_srcs]
    print(f"precise {len(precise)}  gate {len(gate)}  precise-not-gate {len(target)}")

    causes = {}
    also = {}
    for d in target:
        causes[d["gate_cause"]] = causes.get(d["gate_cause"], 0) + 1
        for c in d.get("gate_causes") or []:
            also[c] = also.get(c, 0) + 1
    print("first cause:", sorted(causes.items(), key=lambda kv: -kv[1]))
    print("also carry :", sorted(also.items(), key=lambda kv: -kv[1]))

    # Can ANY selective binding bind them?  records vs segments, from the
    # `selective_bind` quad (records, segments, unclaimed_mangled,
    # unclaimed_inline_fit).
    eq = short = 0
    for d in target:
        sb = d.get("selective_bind")
        if not sb:
            continue
        if sb[0] == sb[1]:
            eq += 1
        else:
            short += 1
    print(f"selective_bind records == segments: {eq}   records < segments: {short}")

    with open("work/w-seclayout/target380.txt", "w") as f:
        for d in sorted(target, key=lambda d: d["src"]):
            f.write(d["src"] + "\n")
    print("-> work/w-seclayout/target380.txt")


main()
