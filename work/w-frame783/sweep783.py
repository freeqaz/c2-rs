#!/usr/bin/env python3
"""w-frame783 — the framing question over a whole sample of workload captures.

For every captured TU: the GATE framing's record count, the RELAXED (#2783)
framing's record count, and — the soundness question — how many of the framed
`80 <LE32>` offsets are NOT `.ex` `4F 1F` split points, at each width.

A framed offset that is not a split point is the only way this relaxation could
turn a refusal into wrong bytes: `Bindings::per_record` would bind a name to a
body that does not begin there. So that column is the whole safety case and it
is counted here rather than asserted.

Also printed: what the relaxation does to the INCUMBENT 1:1 contract
(`records == segments`), which is the regression risk on the 23 matches.

    sweep783.py <capdir> [base.jsonl]
"""
import sys, os, glob, struct, json
from frame783 import gate_framed, wide_framed, scan, ex_splits


def analyse(d):
    g = glob.glob(os.path.join(d, "*.gl"))
    e = glob.glob(os.path.join(d, "*.ex"))
    if not g or not e:
        return None
    gl = open(g[0], "rb").read()
    ex = open(e[0], "rb").read()
    segs = set(ex_splits(ex))
    out = {"segments": len(segs), "gl_len": len(gl), "ex_len": len(ex)}
    for label, pred in (("gate", gate_framed), ("wide", wide_framed)):
        hits = scan(gl, pred)
        offs = [v for _, v, _ in hits]
        out[label] = len(hits)
        out[label + "_notsplit"] = sum(1 for v in offs if v not in segs)
        out[label + "_dup"] = len(offs) - len(set(offs))
        out[label + "_1to1"] = (len(hits) == len(segs) and len(set(offs)) == len(offs)
                                and all(v in segs for v in offs))
    return out


def main():
    capdir = sys.argv[1]
    jsonl = sys.argv[2] if len(sys.argv) > 2 else "work/w-frame783/base.jsonl"
    rows = {}
    for l in open(jsonl):
        if '"record"' in l[:14]:
            continue
        r = json.loads(l)
        rows[r["src"]] = r

    res = []
    for d in sorted(glob.glob(os.path.join(capdir, "*"))):
        done = os.path.join(d, ".done")
        if not os.path.isfile(done):
            continue
        src = open(done).read().strip()
        a = analyse(d)
        if a is None:
            continue
        a["src"] = src
        a["class"] = rows.get(src, {}).get("class")
        a["gate_cause"] = rows.get(src, {}).get("gate_cause")
        res.append(a)

    n = len(res)
    print(f"TUs analysed: {n}")
    fp_gate = sum(r["gate_notsplit"] for r in res)
    fp_wide = sum(r["wide_notsplit"] for r in res)
    tu_fp_gate = sum(1 for r in res if r["gate_notsplit"])
    tu_fp_wide = sum(1 for r in res if r["wide_notsplit"])
    print(f"framed records:  GATE {sum(r['gate'] for r in res):8d}   "
          f"WIDE {sum(r['wide'] for r in res):8d}")
    print(f"offsets that are NOT an .ex 4F 1F split point:")
    print(f"    GATE {fp_gate} over {tu_fp_gate} TUs")
    print(f"    WIDE {fp_wide} over {tu_fp_wide} TUs")
    print(f"duplicate offsets:  GATE {sum(r['gate_dup'] for r in res)}   "
          f"WIDE {sum(r['wide_dup'] for r in res)}")
    g1 = [r for r in res if r["gate_1to1"]]
    w1 = [r for r in res if r["wide_1to1"]]
    print(f"1:1 with the segments (the incumbent per_record contract):"
          f"  GATE {len(g1)}   WIDE {len(w1)}")
    lost = [r["src"] for r in res if r["gate_1to1"] and not r["wide_1to1"]]
    gained = [r["src"] for r in res if r["wide_1to1"] and not r["gate_1to1"]]
    print(f"    1:1 LOST by relaxing: {len(lost)}")
    for s in lost:
        r = next(x for x in res if x["src"] == s)
        print(f"      {s}  class={r['class']}  segs={r['segments']} "
              f"gate={r['gate']} wide={r['wide']}")
    print(f"    1:1 GAINED by relaxing: {len(gained)}")
    for s in gained:
        print(f"      {s}")
    # the matches specifically
    mm = [r for r in res if r["class"] == "match"]
    print(f"\nof the {len(mm)} MATCHING TUs in the sample: "
          f"gate 1:1 {sum(1 for r in mm if r['gate_1to1'])}, "
          f"wide 1:1 {sum(1 for r in mm if r['wide_1to1'])}, "
          f"wide false-positive offsets {sum(r['wide_notsplit'] for r in mm)}")
    for r in sorted(mm, key=lambda r: r["src"]):
        flag = "" if r["wide_1to1"] else "   <-- WIDE BREAKS 1:1"
        print(f"    {r['src']:70s} segs {r['segments']:5d} gate {r['gate']:5d} "
              f"wide {r['wide']:5d}{flag}")
    json.dump(res, open(os.path.join(os.path.dirname(capdir) or ".",
                                     "sweep783.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
