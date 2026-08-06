#!/usr/bin/env python3
"""two_readers.py — reconcile the emit-predicate channel's `.in` reader with
`crates/c2-il`'s own, **on the same 850 TUs**.

The lane's headline compares `work/w-mark/instream.py` against w-tag02's
measured grammar and finds them identical.  That leaves a second question the
brief's premise depends on: does the reader that SHIPS see the same stream?

  * instrument A — `work/w-emitp2/scan2.py`'s strict pass (this lane).
  * instrument B — `crates/c2-il/tests/in_init_probe.rs`, the production
    reader's own cursor, over a symlink farm of the same cache entries.

Neither is the other's witness: A is python written from `GRAMMAR.md`, B is the
shipping Rust.  Every disagreement is printed with a COUNT and the TUs are
ranked, because a ratio with no denominator is not a measurement.

    usage: two_readers.py <scan2.jsonl> <instrument2.txt>
"""
import json
import sys


def main():
    rows = {}
    for l in open(sys.argv[1]):
        r = json.loads(l)
        if r.get("status") == "ok":
            rows[r["src"].replace("/", "__")] = r

    b = {}
    for l in open(sys.argv[2]):
        if "\trecords=" not in l:
            continue
        cell, rest = l.rstrip("\n").split("\t", 1)
        d = {}
        for tok in rest.split(" ["):
            pass
        head, _, tail = rest.partition(" [")
        for kv in head.split():
            k, _, v = kv.partition("=")
            d[k] = int(v)
        for kv in tail.rstrip("]").split():
            k, _, v = kv.partition("=")
            d["res:" + k] = int(v)
        b[cell] = d

    common = sorted(set(rows) & set(b))
    print("TUs: channel %d ; crate probe %d ; common %d"
          % (len(rows), len(b), len(common)))

    tot = {"a_rec": 0, "b_rec": 0, "a_elem": 0, "b_elem": 0,
           "a_e02": 0, "b_e02": 0, "b_acc": 0, "b_res": 0,
           "b_rec_sym": 0, "t_rec": 0, "t_elem": 0, "t_e02": 0,
           "t_res": 0, "t_rec_sym": 0}
    resid = {}
    worse = []
    agree = {"rec": 0, "e02": 0, "recsym": 0}
    for c in common:
        a = rows[c]
        st = a["in"]["st"]
        stc = a["in"]["stc"]
        tot["a_rec"] += a["in"]["rec_s"]
        tot["a_elem"] += st["elem"]
        tot["a_e02"] += st["e02"]
        tot["b_rec"] += b[c]["records"]
        tot["b_elem"] += b[c]["elements"]
        tot["b_e02"] += b[c]["symrefs"]
        tot["b_res"] += b[c]["residue"]
        tot["b_rec_sym"] += b[c]["records_with_symrefs"]
        tot["t_rec"] += stc["c_rec"]
        tot["t_elem"] += stc["c_elem"]
        tot["t_e02"] += stc["c_e02"]
        tot["t_res"] += stc["c_refused"]
        tot["t_rec_sym"] += stc["c_rec_with_sym"]
        agree["rec"] += 1 if stc["c_rec"] == b[c]["records"] else 0
        agree["e02"] += 1 if stc["c_e02"] == b[c]["symrefs"] else 0
        agree["recsym"] += (1 if stc["c_rec_with_sym"]
                            == b[c]["records_with_symrefs"] else 0)
        for k, v in b[c].items():
            if k.startswith("res:"):
                resid[k] = resid.get(k, 0) + v
        worse.append((st["e02"] - b[c]["symrefs"], c, st["e02"],
                      b[c]["symrefs"]))

    def ratio(x, y):
        return (x / y) if y else float("inf")

    print()
    print("== THE SAME 850 STREAMS, TWO READERS ==")
    print("  %-34s %12s %12s %8s" % ("", "channel (A)", "crate (B)", "B/A"))
    for lbl, ka, kb in (("`.in` records", "a_rec", "b_rec"),
                        ("elements (ARITY)", "a_elem", "b_elem"),
                        ("tag-02 symbol addresses", "a_e02", "b_e02")):
        print("  %-34s %12d %12d %8.4f"
              % (lbl, tot[ka], tot[kb], ratio(tot[kb], tot[ka])))
    print("  %-34s %12s %12d" % ("records the crate REFUSES", "-", tot["b_res"]))
    print("  %-34s %12s %12d"
          % ("records carrying a symbol address", "-", tot["b_rec_sym"]))
    print()
    print("  crate residue by reason: %s"
          % {k[4:]: v for k, v in sorted(resid.items()) if v})

    print()
    print("== GRADING THE TRANSCRIPTION — `strictin.parse_records_crate` (T) "
          "against the crate's own cursor (B) ==")
    print("  %-34s %12s %12s %8s" % ("", "transcript T", "crate B", "T/B"))
    for lbl, kt, kb in (("records", "t_rec", "b_rec"),
                        ("elements", "t_elem", "b_elem"),
                        ("tag-02 symbol addresses", "t_e02", "b_e02"),
                        ("records carrying one", "t_rec_sym", "b_rec_sym"),
                        ("residue records", "t_res", "b_res")):
        print("  %-34s %12d %12d %8.4f"
              % (lbl, tot[kt], tot[kb], ratio(tot[kt], tot[kb])))
    print("  TUs where T and B agree EXACTLY: records %d/%d ; symbol "
          "addresses %d/%d ; records-carrying-one %d/%d"
          % (agree["rec"], len(common), agree["e02"], len(common),
             agree["recsym"], len(common)))

    worse.sort(reverse=True)
    print()
    print("== THE 12 TUs WHERE THE CRATE SEES FEWEST OF THE CHANNEL'S "
          "SYMBOL ADDRESSES ==")
    print("  %-52s %9s %9s %9s" % ("TU", "channel", "crate", "unseen"))
    for d, c, x, y in worse[:12]:
        print("  %-52s %9d %9d %9d" % (c.replace("__", "/")[:52], x, y, d))
    n_gap = sum(1 for d, _c, _x, _y in worse if d > 0)
    n_eq = sum(1 for d, _c, _x, _y in worse if d == 0)
    n_neg = sum(1 for d, _c, _x, _y in worse if d < 0)
    print("  TUs where the crate sees FEWER: %d ; equal: %d ; MORE: %d"
          % (n_gap, n_eq, n_neg))


if __name__ == "__main__":
    main()
