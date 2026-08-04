#!/usr/bin/env python3
import os, sys, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj, chdec, SEL
import os
# repo root = four levels up from this file (.claude/worktrees/<lane>/work/w-bss/census)
W = os.environ.get("C2RS_LANE_ROOT") or os.path.abspath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
OBJS = W + "/work/w-bss/census/objs"
SC = {2: "EXTERNAL", 3: "STATIC", 105: "SECTION", 103: "FILE"}

def show(src, maxsec=60):
    o = Obj(open(os.path.join(OBJS, src.replace("/", "_") + ".obj"), "rb").read())
    print("\n" + "=" * 78)
    print("TU %s   NumberOfSections=%d NumberOfSymbols=%d" % (src, o.nsec, o.nsym))
    ssym = {}
    byidx = {s["idx"]: s for s in o.secs}
    for sy in o.syms:
        if sy["naux"] == 1 and sy["sc"] == 3 and sy["val"] == 0 and sy["sec"] > 0:
            s = byidx.get(sy["sec"])
            if s and s["name"] == sy["name"]:
                ssym[sy["sec"]] = struct.unpack_from("<IHHIHB", sy["aux"][0], 0)
    order = []
    for s in o.secs:
        n = s["name"]
        if n == ".XBLD$W":
            n += ":" + o.secdata(s)[:2].decode("latin1", "replace")
        order.append(n)
    print("section order (%d):" % len(order))
    if len(order) <= maxsec:
        print("  " + " ".join("%d:%s" % (i + 1, n) for i, n in enumerate(order)))
    else:
        print("  " + " ".join("%d:%s" % (i + 1, n) for i, n in enumerate(order[:maxsec])) + " ...")
    for s in o.secs:
        if s["name"] not in (".data", ".bss"):
            continue
        ln, nrel, nln, cks, num, sel = ssym[s["idx"]]
        print("  [%d] %-5s SizeOfRawData=%-7d VirtualSize=%d PointerToRawData=0x%x "
              "NumberOfRelocations=%d" % (s["idx"], s["name"], s["size"], s["vsz"],
                                          s["ptr"], s["nrel"]))
        print("        Characteristics=0x%08x  %s" % (s["ch"], chdec(s["ch"])))
        print("        aux: Length=%d CheckSum=0x%08x Number=%d Selection=%d(%s)"
              % (ln, cks, num, sel, SEL.get(sel, "-")))
        for sy in o.syms:
            if sy["sec"] == s["idx"] and sy["naux"] == 0:
                print("        sym Value=0x%-5x sc=%d(%s) Type=0x%04x  %s"
                      % (sy["val"], sy["sc"], SC.get(sy["sc"], "?"), sy["typ"], sy["name"]))

for a in sys.argv[1:]:
    show(a)
