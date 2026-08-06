#!/usr/bin/env python3
"""blindspot.py — WHY the shipping `.in` reader loses 24 % of the initializer
reference graph, decomposed so the next lane has a target and not an adjective.

For every record the sequential parser frames, this asks what
`crates/c2-il/src/func/ininit.rs` would do with it and, when the answer is
"nothing", **which element byte is responsible** and **how many tag-02 symbol
addresses go with it**.

    usage: blindspot.py <cacheidx.tsv> [jobs]

stdlib only.  Reads no c2 output.
"""
import collections
import json
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import strictin as si  # noqa: E402

SYM = si.SYM


def base_in(entry):
    for n in os.listdir(entry):
        if n.startswith("_CL_") and n.endswith("in"):
            return os.path.join(entry, n)
    return None


def blame(el):
    """The FIRST element the crate's `read_elements` would refuse, as a key."""
    for k, a, w in el:
        if k == SYM:
            continue
        if k == 0x03:
            return "element-tag-03 (inline bytes / string)"
        if k == 0x08:
            return "element-tag-08 (zero fill)"
        if k != 0x01:
            return "element-tag-%02x" % k
        if a == 0x05:
            return "scalar type 05 (floating point)"
        if a not in si.CRATE_TYPES:
            return "scalar type %02x" % a
        if w not in si.CRATE_WIDTHS:
            return "scalar width %d" % w
    return "NONE"


def one(row):
    src, entry = row[0], row[1]
    p = base_in(entry)
    if p is None:
        return {"src": src, "status": "NOIN"}
    _clean, recs, _st = si.parse_ex(open(p, "rb").read())
    why = collections.Counter()
    lost = collections.Counter()
    firsts = collections.Counter()
    tot = {"rec": 0, "e02": 0, "kept": 0, "kept_e02": 0}
    for _t, _f, _o, el in recs:
        tot["rec"] += 1
        n02 = sum(1 for e in el if e[0] == SYM)
        tot["e02"] += n02
        ok, first, _w = si._crate_verdict(el)
        if ok:
            tot["kept"] += 1
            tot["kept_e02"] += n02
            continue
        firsts["first=%s" % ("02" if first == SYM else
                             ("%02x" % first if first is not None else "-"))] += 1
        k = blame(el)
        why[k] += 1
        lost[k] += n02
    return {"src": src, "status": "ok", "tot": tot, "why": dict(why),
            "lost": dict(lost), "firsts": dict(firsts)}


def main():
    idxp = sys.argv[1]
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 8
    rows = [l.rstrip("\n").split("\t") for l in open(idxp)]
    why = collections.Counter()
    lost = collections.Counter()
    firsts = collections.Counter()
    tot = collections.Counter()
    tus = collections.Counter()
    with cf.ProcessPoolExecutor(max_workers=jobs) as ex:
        for r in ex.map(one, rows, chunksize=8):
            if r.get("status") != "ok":
                print("  %s %s" % (r["status"], r["src"]))
                continue
            for k, v in r["tot"].items():
                tot[k] += v
            for k, v in r["why"].items():
                why[k] += v
                tus[k] += 1
            for k, v in r["lost"].items():
                lost[k] += v
            for k, v in r["firsts"].items():
                firsts[k] += v

    print("records framed %d ; the crate keeps %d (%.4f)"
          % (tot["rec"], tot["kept"], tot["kept"] / tot["rec"]))
    print("tag-02 symbol addresses %d ; the crate keeps %d (%.4f) ; LOSES %d"
          % (tot["e02"], tot["kept_e02"], tot["kept_e02"] / tot["e02"],
             tot["e02"] - tot["kept_e02"]))
    print()
    print("== WHY A RECORD IS LOST — the FIRST element `read_elements` refuses ==")
    print("  %-42s %10s %10s %8s" % ("blame", "records", "sym addrs", "TUs"))
    for k, v in why.most_common():
        print("  %-42s %10d %10d %8d" % (k, v, lost.get(k, 0), tus[k]))
    print()
    print("== THE LOST RECORDS BY THEIR FIRST ELEMENT ==")
    for k, v in firsts.most_common():
        print("  %-42s %10d" % (k, v))


if __name__ == "__main__":
    main()
