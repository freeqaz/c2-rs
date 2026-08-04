#!/usr/bin/env python3
"""w-bss census: .data / .bss across the dc3 878-TU workload's real c2 objs.
Emits one JSON line per TU to sections.jsonl. Tooling only."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj, chdec, SEL, SC

import os
# repo root = four levels up from this file (.claude/worktrees/<lane>/work/w-bss/census)
W = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OBJS = W + "/work/w-bss/census/objs"
FILES = W + "/work/dc3-workload/files.txt"


def classify(name):
    if name.startswith("??_R0"):
        return "??_R0"
    for k in "1234":
        if name.startswith("??_R" + k):
            return "??_R" + k
    if name.startswith("??_7"):
        return "??_7"
    if name.startswith("??_8"):
        return "??_8"
    if name.startswith("$SG"):
        return "$SG"
    if name.startswith("?"):
        return "?...@@3..." if "@@3" in name else "?other-decorated"
    if name.startswith("_") or name[0].isalpha():
        return "undecorated"
    return "other"


def xbld_tag(o, s):
    d = o.secdata(s)
    return d[:2].decode("latin1", "replace") if len(d) >= 2 else "??"


out = open(W + "/work/w-bss/census/sections.jsonl", "w")
n = 0
for src in open(FILES).read().split():
    p = os.path.join(OBJS, src.replace("/", "_") + ".obj")
    if not os.path.exists(p):
        continue
    o = Obj(open(p, "rb").read())
    # section symbols keyed by section index (sc==3, naux==1, name==sec name)
    secsym = {}
    for sy in o.syms:
        if sy["naux"] == 1 and sy["sec"] > 0 and sy["sc"] == 3 and sy["val"] == 0:
            for s in o.secs:
                if s["idx"] == sy["sec"] and s["name"] == sy["name"]:
                    a = sy["aux"][0]
                    import struct
                    ln, nrel, nln, cks, num, sel = struct.unpack_from("<IHHIHB", a, 0)
                    secsym.setdefault(sy["sec"], []).append(
                        dict(len=ln, nrel=nrel, cks=cks, num=num, sel=sel))
    rec = dict(src=src, nsec=o.nsec, order=[], data=[], bss=[])
    for s in o.secs:
        nm = s["name"]
        tag = nm
        if nm == ".XBLD$W":
            tag = ".XBLD$W:" + xbld_tag(o, s)
        rec["order"].append(tag)
    for s in o.secs:
        if s["name"] not in (".data", ".bss"):
            continue
        syms = []
        for sy in o.syms:
            if sy["sec"] == s["idx"] and sy["naux"] == 0:
                syms.append(dict(n=sy["name"], v=sy["val"], sc=sy["sc"],
                                 typ=sy["typ"], cls=classify(sy["name"])))
        ss = secsym.get(s["idx"], [])
        e = dict(idx=s["idx"], size=s["size"], vsz=s["vsz"], ptr=s["ptr"],
                 nrel=s["nrel"], ch=s["ch"], chdec=chdec(s["ch"]),
                 comdat=bool(s["ch"] & 0x1000),
                 secsym=ss, syms=syms)
        rec["data" if s["name"] == ".data" else "bss"].append(e)
    out.write(json.dumps(rec) + "\n")
    n += 1
out.close()
print("wrote", n, "records")
