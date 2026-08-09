#!/usr/bin/env python3
"""w-fltret — the lane's own rows, re-derived from a scan rather than quoted.

Usage: rows.py SCAN.jsonl
"""
import sys
sys.path.insert(0, "work/w-fltret")
import importlib.util

spec = importlib.util.spec_from_file_location("keys", "work/w-fltret/keys.py")
# keys.py runs main() on import, so read it and take only `load`.
src = open("work/w-fltret/keys.py").read().replace("\nmain()\n", "\n")
ns = {}
exec(compile(src, "keys.py", "exec"), ns)
load = ns["load"]

a = load(sys.argv[1])

print("census in-class %d / %d" % (a["inclass"], a["tot"]))
print("emitted in-class %d / %d" % (a["emit_in"], a["emit_tot"]))
fam_b = sum(v for k, v in a["fn"].items() if k.startswith("expr-call-in-expr"))
fam_e = sum(v for k, v in a["em"].items() if k.startswith("expr-call-in-expr"))
print("family expr-call-in-expr: bodies %d emitted %d" % (fam_b, fam_e))

print("\n-- R2's population, by the census keys w-callprice §7 names")
r2 = 0
r2b = 0
for k in sorted(a["fn"]):
    if k.endswith("-type-real-whole") and k.startswith("expr-call-in-expr"):
        e = a["em"].get(k, 0)
        r2 += e
        r2b += a["fn"][k]
        print("  %6d emitted %8d bodies %4d TUs  %s" % (e, a["fn"][k], len(a["tus"][k]), k))
print("  R2 TOTAL: %d emitted / %d bodies" % (r2, r2b))

print("\n-- every `-type-real` key in the family (the wider FP row)")
tr = tre = 0
for k in sorted(a["fn"]):
    if "type-real" in k and k.startswith("expr-call-in-expr"):
        tr += a["fn"][k]
        tre += a["em"].get(k, 0)
print("  %d keys, %d bodies, %d emitted" % (
    len([k for k in a["fn"] if "type-real" in k and k.startswith("expr-call-in-expr")]), tr, tre))

print("\n-- the FP fences, as census keys")
for k in sorted(a["fn"]):
    if k.startswith("call-ret-fp") or k.startswith("result-type"):
        print("  %6d emitted %8d bodies %4d TUs  %s"
              % (a["em"].get(k, 0), a["fn"][k], len(a["tus"][k]), k))

print("\n-- the `prod` tags this lane moves")
for k in sorted(a["prod"]):
    if ("void-body-does-not-end" in k or "returned-body-does-not-end" in k
            or "call-ret-fp" in k or "result-type" in k):
        print("  %8d  %s" % (a["prod"][k], k))

print("\n-- `recv-load-whole` and `chained-*-whole`, the int siblings")
for k in ("expr-call-in-expr-recv-load-whole", "expr-call-in-expr-chained-whole"):
    if k in a["fn"]:
        print("  %6d emitted %8d bodies %4d TUs  %s"
              % (a["em"].get(k, 0), a["fn"][k], len(a["tus"][k]), k))
