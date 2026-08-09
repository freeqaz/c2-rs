#!/usr/bin/env python3
"""w-fltret — RE-DERIVE the commissioned population at THIS lane's base.

w-callprice §7 recommends R2 at "544 emitted over 9 constructs":
`recv-load-then-type-real-whole` 439/5 and `chained-then-type-real-whole` 105/4.
Inherited prices have been wrong six times this week, so nothing here is
copied: every number is taken out of a scan of this lane's own base.

    pop.py SCAN.jsonl [--names NAMED_SCAN.jsonl]

Without `--names` the emitted/body columns come out of the ordinary census keys.
With it, the scan was taken with the scratch instrument that appends the emitted
symbol's mangled name to the key, which is the only way to get the CONSTRUCTS
column (w-callprice §2.1's pattern, re-aimed and much smaller).

The script ASSERTS its own totals against the family total taken independently
from the same file, rather than printing them and hoping (w-callprice P2).
"""
import json
import sys
from collections import Counter, defaultdict

FAMILY = "expr-call-in-expr"
TARGETS = [
    "expr-call-in-expr-recv-load-then-type-real-whole",
    "expr-call-in-expr-chained-then-type-real-whole",
]


def load(path):
    fn, em, em_tus = Counter(), Counter(), defaultdict(set)
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        src = r.get("src")
        for k, v in (r.get("fn_blockers") or {}).items():
            fn[k] += v
        for k, v in (r.get("emit_blockers") or {}).items():
            em[k] += v
            em_tus[base_key(k)].add(src)
    return fn, em, em_tus


def base_key(k):
    return k.split("|", 1)[0]


def main():
    path = sys.argv[1]
    fn, em, em_tus = load(path)

    tot_fn, tot_em = sum(fn.values()), sum(em.values())
    fam_fn = sum(v for k, v in fn.items() if base_key(k).startswith(FAMILY))
    fam_em = sum(v for k, v in em.items() if base_key(k).startswith(FAMILY))
    print(f"WHOLE BLOCKED CENSUS   bodies {tot_fn}   emitted {tot_em}")
    print(f"FAMILY {FAMILY}   bodies {fam_fn}   emitted {fam_em}")
    print(f"  share of the blocked EMITTED column: {100*fam_em/tot_em:.2f} %")

    named = "|" in "".join(list(em)[:50])
    if not named:
        print("\n=== THE TWO TARGET KEYS (no constructs column in this scan) ===")
        for k in TARGETS:
            print(f"  {em.get(k,0):6d} emitted  {fn.get(k,0):8d} bodies  "
                  f"{len(em_tus.get(k,())):5d} TUs   {k}")
        return

    # --- the constructs column ------------------------------------------------
    per_key_em = Counter()
    per_key_names = defaultdict(Counter)
    per_key_name_tus = defaultdict(lambda: defaultdict(set))
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        for k, v in (r.get("emit_blockers") or {}).items():
            bk, _, nm = k.partition("|")
            per_key_em[bk] += v
            per_key_names[bk][nm] += v
            per_key_name_tus[bk][nm].add(r.get("src"))

    fam_em2 = sum(v for k, v in per_key_em.items() if k.startswith(FAMILY))
    assert fam_em2 == fam_em, f"ASSERT FAILED: {fam_em2} != {fam_em}"
    print(f"  ASSERTED: the de-shattered key preserves the emitted column exactly "
          f"({fam_em2}).")

    print("\n=== THE COMMISSIONED POPULATION, RE-DERIVED ===")
    print(f"{'emitted':>8s} {'cons':>5s} {'TUs':>6s} {'bodies':>8s} {'em/1k':>7s}  key")
    tot_e = tot_c = 0
    for k in TARGETS:
        e = per_key_em.get(k, 0)
        c = len(per_key_names.get(k, ()))
        b = fn.get(k, 0)
        t = len(em_tus.get(k, ()))
        tot_e += e
        tot_c += c
        print(f"{e:8d} {c:5d} {t:6d} {b:8d} {1000*e/b if b else 0:7.1f}  {k}")
    print(f"{tot_e:8d} {tot_c:5d} {'':6s} {'':8s} {'':7s}  TOTAL "
          f"(w-callprice §7-R2 published 544 / 9)")

    for k in TARGETS:
        print(f"\n--- the names on {k} ---")
        for nm, v in per_key_names.get(k, Counter()).most_common(20):
            tus = len(per_key_name_tus[k][nm])
            flag = "  <= emitted == TUs" if tus == v else ""
            print(f"  {v:6d} emitted  {tus:6d} TUs  {nm}{flag}")

    # The `prod` cross the commission also names.
    print("\n=== the whole family, top 25 by EMITTED, with the constructs column ===")
    print(f"{'#':>3s} {'emitted':>8s} {'cons':>5s} {'bodies':>9s} {'em/1k':>7s}  key")
    fe = {k: v for k, v in per_key_em.items() if k.startswith(FAMILY)}
    for i, (k, v) in enumerate(sorted(fe.items(), key=lambda kv: -kv[1])[:25]):
        b = fn.get(k, 0)
        print(f"{i+1:3d} {v:8d} {len(per_key_names[k]):5d} {b:9d} "
              f"{1000*v/b if b else 0:7.1f}  {k}")


main()
