#!/usr/bin/env python3
"""Compact per-obj summary for lane w-bss: section order, data sections, data symbols."""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj, chdec

def summarize(path, want=("bss", "data")):
    o = Obj(open(path, "rb").read())
    print("== %s  nsec=%d nsym=%d" % (os.path.basename(path), o.nsec, o.nsym))
    print("   order: " + " ".join(s["name"] for s in o.secs))
    for s in o.secs:
        if s["name"].lstrip(".").split("$")[0] in want or s["name"].startswith(".rdata"):
            print("   [%d] %-10s size=0x%-4x ptr=0x%-5x ch=0x%08x %s"
                  % (s["idx"], s["name"], s["size"], s["ptr"], s["ch"], chdec(s["ch"])))
            for sy in o.syms:
                if sy["sec"] == s["idx"] and sy["naux"] == 0:
                    print("        %-40s Value=0x%-5x SC=%d naux=%d" % (sy["name"][:40], sy["val"], sy["sc"], sy["naux"]))
            d = o.secdata(s)
            if d:
                print("        raw: " + d.hex(" "))
    return o

def order(path, secname=".bss"):
    """Return [(offset, symname)] for the named section, ascending address."""
    o = Obj(open(path, "rb").read())
    idx = [s["idx"] for s in o.secs if s["name"] == secname]
    if not idx:
        return None
    i = idx[0]
    got = [(sy["val"], sy["name"]) for sy in o.syms if sy["sec"] == i and sy["naux"] == 0]
    got.sort()
    return got

if __name__ == "__main__":
    for p in sys.argv[1:]:
        summarize(p)
