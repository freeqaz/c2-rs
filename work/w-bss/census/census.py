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


# ---------------------------------------------------------------- provenance
# ONE copy of the stamper, in work/w-bss2/, imported by path.  A second copy
# would be a checker that can disagree with itself, which is worse than none.
sys.path.insert(0, os.path.join(W, "work", "w-bss2"))
import prov, paths  # noqa: E402

CORPUS = paths.DC3
# The objs this aggregates were compiled EARLIER, by one.sh.  If the caller
# (scripts/regen_census.sh) snapshotted the corpus before that phase, use its
# snapshot -- only then does the stamp cover the compiles.  Taking one here
# instead is recorded as begin_scope="aggregate", which states in the artefact
# that drift during the compile phase was invisible to it.  A stamp that
# silently narrows its own scope would be worse than no stamp.
_bp = os.environ.get("C2RS_PROV_BEGIN")
if _bp and os.path.exists(_bp):
    BEGIN, SCOPE = prov.begin_read(_bp), "run"
else:
    BEGIN, SCOPE = prov.begin(CORPUS), "aggregate"

SECTIONS_PATH = W + "/work/w-bss/census/sections.jsonl"
out = open(SECTIONS_PATH, "w")
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

# `sections.jsonl` is COMMITTED (force-added: regenerating it needs the real
# toolchain, the dc3 tree, and ~102 MB of intermediate objs).  So its sidecar is
# committed too, and `committed=True` strips every absolute path and refuses to
# write one -- CLAUDE.md forbids machine paths in the history.  The corpus is
# still pinned, by `path_sha256` over the resolved path.
# `path_rel` is taken against the MAIN repo, not the lane root: from a worktree
# (`<main>/.claude/worktrees/<lane>`) the corpus is four levels up, which
# `prov.path_rel` correctly refuses to encode, and the sidecar would then carry
# no readable path at all. Against MAIN it is the documented `../dc3-decomp`.
_p = prov.stamp("census.py", SECTIONS_PATH, BEGIN, paths.MAIN,
                inputs=dict(flags_sha256=prov.sha256_file(
                                W + "/work/dc3-workload/flags.txt"),
                            files_sha256=prov.sha256_file(FILES)),
                allow_dirty=os.environ.get("C2RS_PROV_ALLOW_DIRTY") == "1",
                allow_move=os.environ.get("C2RS_PROV_ALLOW_MOVE") == "1",
                begin_scope=SCOPE, records=n)
print("provenance ->", prov.write(SECTIONS_PATH, _p, committed=True))
print(" ", prov.describe(_p), "scope=" + SCOPE)
