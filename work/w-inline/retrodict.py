#!/usr/bin/env python3
"""retrodict.py — does INLINE-P retrodict the 4,711?

Lane w-inline measurement tooling. **Read-only with respect to `crates/`.**

`work/w-fnbyte/differ_taxonomy.txt` reports 4,711 emitted functions whose port
bytes are wrong, in 61 signatures, and traces the largest family to *"c2 inlines
a callee defined in the same TU and the port's IL-level call recognizers do not
model that decision."* This file asks the question that claim implies:

> For a differing caller `F`, take the same-TU callees `F` actually has — read
> off the `/Ob0` obj, where nothing is inlined — and ask `INLINE-P` whether c2
> would inline each of them. Then check against the `/O1` obj, where a surviving
> REL24 says it did not.

Inputs:
  * `c2rs gap --jsonl`'s own `fnbyte-differs-fn|<shape>|w…|<first>|<symbol>`
    keys — the differ list, per TU, by name, from the standing instrument.
  * the `/O1` and `/Ob0` objs of the sampled TUs (`build_objs*.sh`).

Two numbers are printed and they answer different questions:

  1. **PER CALLER** — of the differing callers this sample can resolve, for how
     many does `INLINE-P` get *every* same-TU callee right? This is the number
     the port would need, because a recognizer decides one caller at a time.
  2. **PER PAIR** — the (caller, callee) accuracy, which is the same currency
     `grade_pair.py` reports and is comparable to it.

**FAMILY A is scored separately**, by the taxonomy's own definition: the
reference body is one word and that word is `4e800020` (`blr`).

Usage:
    retrodict.py --jsonl PATH --a DIR --b DIR --index PATH
"""

import collections
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scan_obj import (  # noqa: E402
    UNBOUNDED, read_obj, annotate_params, is_leaf, n_max, caller_kind,
)

BLR = 0x4E800020


def differs_by_tu(path):
    out = collections.defaultdict(dict)
    for line in open(path, encoding="utf-8"):
        d = json.loads(line)
        for k in (d.get("emit") or {}):
            if not k.startswith("fnbyte-differs-fn|"):
                continue
            sh, words, firstw, name = k[len("fnbyte-differs-fn|"):].split("|", 3)
            pw, rw, eq = words[1:].split("/")
            out[d.get("src", "?")][name] = {
                "shape": sh, "ref_words": int(rw), "first": firstw,
                "family_a": firstw.endswith("ref=4e800020") and int(rw) == 1,
            }
    return out


def main(argv):
    jsonl = argv[argv.index("--jsonl") + 1]
    da = argv[argv.index("--a") + 1]
    db = argv[argv.index("--b") + 1]
    idx = argv[argv.index("--index") + 1]
    diffs = differs_by_tu(jsonl)

    per_caller = collections.Counter()
    per_pair = collections.Counter()
    fam_a_pair = collections.Counter()
    no_callee = 0
    no_callee_fam_a = 0
    fam_a_total = 0
    not_in_obj = 0
    tus_seen = 0
    miss_examples = []

    for line in open(idx):
        n, src = line.rstrip("\n").split("\t")
        if src not in diffs:
            continue
        pa, pb = os.path.join(da, n + ".obj"), os.path.join(db, n + ".obj")
        if not (os.path.exists(pa) and os.path.exists(pb)):
            continue
        tus_seen += 1
        a = read_obj(pa)
        b = read_obj(pb)
        annotate_params(a)
        # Which same-TU functions does the caller still hold a REL24 to at /O1?
        for fname, info in diffs[src].items():
            if info["family_a"]:
                fam_a_total += 1
            if fname not in b or fname not in a:
                not_in_obj += 1
                continue
            # The callee set the SOURCE gives this caller: /Ob0's REL24s to
            # functions this TU also defines. Self-recursion excluded — the
            # incumbent has no row for it (§6.19.10).
            callees = [t for t in b[fname].rel24
                       if t in a and t != fname and caller_kind(t) == "ordinary"]
            if not callees:
                # NOT AN INLINE DECISION. `/Ob0` restores every inline
                # expansion (ctl_ob0.py's p4 control), so a differing caller
                # with no same-TU REL24 even there never had a call to inline.
                # ctl_ob0.py's p2/p8 pin the mechanism: a call to a callee whose
                # SOURCE body is empty is dropped by the front end, at /Ob0 too.
                no_callee += 1
                if info["family_a"]:
                    no_callee_fam_a += 1
                continue
            survivors = set(a[fname].rel24)
            all_ok = True
            for g in dict.fromkeys(callees):
                gf = a[g]
                nm = n_max(gf, is_leaf(gf, a), drop_leaf_term=True)
                sites = sum(1 for t in b[fname].rel24 if t == g)
                predicted = "INLINED" if nm >= sites else "DECLINED"
                observed = "DECLINED" if g in survivors else "INLINED"
                ok = predicted == observed
                per_pair["HIT" if ok else "MISS"] += 1
                if info["family_a"]:
                    fam_a_pair["HIT" if ok else "MISS"] += 1
                if not ok:
                    all_ok = False
                    if len(miss_examples) < 12:
                        miss_examples.append(
                            (src.split("/")[-1], fname[:60], g[:60],
                             gf.size, predicted, observed))
            per_caller["HIT" if all_ok else "MISS"] += 1

    def pct(c):
        t = c["HIT"] + c["MISS"]
        return f"{c['HIT']}/{t} = {c['HIT'] / t:.4f}" if t else "n=0"

    print(f"TUs with differs in this sample: {tus_seen}")
    print(f"differing callers with NO same-TU callee even at /Ob0 "
          f"(NOT an inline decision): {no_callee}")
    print(f"   ... of which FAMILY A: {no_callee_fam_a} of {fam_a_total} family-A callers")
    print(f"differing callers skipped — name not in both objs:     {not_in_obj}")
    print()
    print(f"PER CALLER (every callee right): {pct(per_caller)}")
    print(f"PER PAIR:                        {pct(per_pair)}")
    print(f"PER PAIR, FAMILY A only:         {pct(fam_a_pair)}")
    if miss_examples:
        print("\nmisses (up to 12):")
        for m in miss_examples:
            print(f"  {m[0]:28s} {m[1]:60s} -> {m[2]:60s} s={m[3]:4d} "
                  f"pred={m[4]} obs={m[5]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
