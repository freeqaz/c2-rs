#!/usr/bin/env python3
"""cmp.py — the Rust tag-0x10 alias table against w-emitp's Python, TU by TU.

The lane's whole verification.  Aggregate counts agreeing is weak; this compares
the 850 per-TU tables **name for name**, and reports the first disagreements
rather than a status.

    usage: cmp.py <cacheidx.tsv> <rust_alias.jsonl> [jobs]

stdlib only.  Reads no c2 output.
"""
import json
import os
import sys
import concurrent.futures as cf

MAIN = os.environ["C2RS_LANEROOT"]
for _p in (os.path.join(MAIN, "work", "w-emitp"),
           os.path.join(MAIN, "work", "emitpred", "pipeline"),
           os.path.join(MAIN, "work", "w-roots"),
           os.path.join(MAIN, "work", "w-refs"),
           os.path.join(MAIN, "work", "w-mark"),
           os.path.join(MAIN, "work", "w-skip"),
           os.path.join(MAIN, "work", "w-db")):
    sys.path.insert(0, os.path.abspath(_p))
import alias as al   # noqa: E402


def base_of(entry):
    for n in os.listdir(entry):
        if n.startswith("_CL_") and n.endswith("gl"):
            return n[:-2]
    return None


def one(row):
    src, entry = row
    base = base_of(entry)
    glb = open(os.path.join(entry, base + "gl"), "rb").read()
    a0, _t, s0 = al.scan(glb, shift=0)
    _am, _t, sm = al.scan(glb, shift=-1)
    _ap, _t, sp = al.scan(glb, shift=+1)
    shape = sum(1 for k, v in a0.items()
                if k.startswith("??_E") and v.startswith("??_G")
                and k[4:] == v[4:])
    return src, {
        "tag10": s0["tag10"], "bound": s0["bound"], "shape": shape,
        "head_fail": s0["head_fail"], "rt_fail": s0["rt_fail"],
        "unbound_target": s0["unbound_target"], "self": s0["self_alias"],
        "dup": s0["dup"],
        "bound_m1": sm["bound"], "bound_p1": sp["bound"],
    }, a0


def main():
    idx, rustf = sys.argv[1], sys.argv[2]
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 6
    rows = []
    for line in open(idx):
        f = line.rstrip("\n").split("\t")
        if len(f) >= 2:
            rows.append((f[0], f[1]))

    rust = {}
    for line in open(rustf):
        o = json.loads(line)
        rust[o["src"]] = o

    tot_py = {}
    agree = 0
    disagree = []
    ndiff_pairs = 0
    with cf.ProcessPoolExecutor(max_workers=jobs) as ex:
        for src, st, table in ex.map(one, rows, chunksize=8):
            for k, v in st.items():
                tot_py[k] = tot_py.get(k, 0) + v
            r = rust.get(src)
            if r is None:
                disagree.append((src, "MISSING FROM THE RUST DUMP"))
                continue
            bad = [k for k in ("tag10", "bound", "shape", "head_fail",
                               "rt_fail", "unbound_target", "dup",
                               "bound_m1", "bound_p1")
                   if r[k] != st[k]]
            if st["self"] != r["self"]:
                bad.append("self")
            if table != r["pairs"]:
                only_py = set(table.items()) - set(r["pairs"].items())
                only_rs = set(r["pairs"].items()) - set(table.items())
                ndiff_pairs += len(only_py) + len(only_rs)
                bad.append("PAIRS py-only %d rs-only %d" %
                           (len(only_py), len(only_rs)))
                if len(disagree) < 5:
                    disagree.append((src, sorted(only_py)[:3],
                                     sorted(only_rs)[:3]))
            if bad:
                if len(disagree) < 10:
                    disagree.append((src, bad))
            else:
                agree += 1

    print("python totals   %s" % json.dumps(tot_py, sort_keys=True))
    print("TUs agreeing name-for-name AND on every count: %d of %d"
          % (agree, len(rows)))
    print("disagreeing pair entries (either direction): %d" % ndiff_pairs)
    for d in disagree[:10]:
        print("  DISAGREE %s" % (d,))
    print("VERDICT: %s" % ("AGREE" if agree == len(rows) else "DISAGREE"))


if __name__ == "__main__":
    main()
