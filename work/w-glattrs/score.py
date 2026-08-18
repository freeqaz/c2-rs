#!/usr/bin/env python3
"""score.py — GRID-C. THE DECODE, SCORED AGAINST REAL c2, ON ALL 7,667 EDGES.

`docs/rungs/README.md`: *"a predicate is priced against the oracle, on the
population it will apply to, or it is not priced."*  `fnbyte-exact Δ = 0` is
evidence about reach and never about correctness.

Input:
  * `GC-EDGE <tu> <caller> <callee> <arm>` lines from the scan's stderr —
    `w-fence2` GRID-W's observable, per IL call edge to a callee this TU
    defines: `kept` if the reference caller's `.text` COMDAT carries a `REL24`
    naming the callee, `inlined` if it does not.  This is real `c2.dll`'s own
    verdict and nothing here is derived from the port.
  * the dumped `.gl` per TU.

Scored under BOTH `.gl` framings, because they reach very different
populations: the incumbent `gl_offset_framed` (what `gl_function_attrs` walks)
and #2783's `gl_offset_framed_relaxed` (what `w-sizebracket` measured its 309
on).  The DECODE is the same in both; only the record population differs.

The soundness test, stated as the consumers state it: `splice`/`comdat` read
`FN_FLAG_INLINABLE` CLEAR as *"c2 keeps this call"*.  So

    bit 6 clear  =>  the edge must be `kept`

and a `noinline` edge that c2 INLINED is a counterexample — the decode wrong
about c2 in the direction that emits bytes.

Also scored: the rival escape widths, on the same edges, so "the shipped width
is right" is a comparison and not an assertion.
"""

import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import glrec  # noqa: E402


def safe(src):
    return "".join(c if c.isalnum() else "_" for c in src) + ".gl"


_CACHE = {}


def decode_all(gldir, framing, width=3):
    """{tu -> {name -> (size, form, attr)}} at a given escape width."""
    key = (gldir, framing.__name__, width)
    if key in _CACHE:
        return _CACHE[key]
    out = {}
    for fn in os.listdir(gldir):
        if not fn.endswith(".gl"):
            continue
        gl = open(os.path.join(gldir, fn), "rb").read()
        m = {}
        for v, r in glrec.walk_w(gl, framing, width):
            if v == "ok":
                m.setdefault(r["name"], (r["size"], r["form"], r["attr"]))
        out[fn] = m
    _CACHE[key] = out
    return out


def main(argv):
    err, gldir = argv[1], argv[2]
    edges = []
    for line in open(err, errors="replace"):
        if not line.startswith("GC-EDGE\t"):
            continue
        _, tu, caller, callee, arm = line.rstrip("\n").split("\t")
        edges.append((safe(tu), caller, callee, arm))
    print(f"edges {len(edges)}  arms {collections.Counter(a for *_, a in edges)}")

    for fname, framing in (("incumbent", glrec.framed_incumbent),
                           ("relaxed", glrec.framed_relaxed)):
        dec = decode_all(gldir, framing)
        print(f"\n================ framing: {fname}")
        by_form = collections.Counter()
        cross = collections.Counter()
        viol = []
        for tu, caller, callee, arm in edges:
            r = dec.get(tu, {}).get(callee)
            form = r[1] if r else "absent"
            by_form[(form, arm)] += 1
            if r:
                bit = "inlinable" if (r[2] or 0) & 0x40 else "noinline"
                cross[(form, bit, arm)] += 1
                if bit == "noinline" and arm == "inlined":
                    viol.append((tu, caller, callee, r))
        arms = ["kept", "inlined", "unknown"]
        print(f"  {'form':>10} " + "".join(f"{a:>9}" for a in arms) + f"{'total':>9}")
        for f in sorted({k[0] for k in by_form}):
            row = [by_form[(f, a)] for a in arms]
            print(f"  {f:>10} " + "".join(f"{v:>9}" for v in row) + f"{sum(row):>9}")
        print(f"\n  {'form':>10} {'bit':>10} " + "".join(f"{a:>9}" for a in arms))
        for f in sorted({k[0] for k in cross}):
            for b in ("inlinable", "noinline"):
                row = [cross[(f, b, a)] for a in arms]
                if any(row):
                    print(f"  {f:>10} {b:>10} " + "".join(f"{v:>9}" for v in row))
        n_esc = sum(by_form[("escape", a)] for a in arms)
        print(f"\n  ESCAPED-SIZE EDGES: {n_esc}")
        print(f"  COUNTEREXAMPLES to `bit 6 clear => c2 kept`: {len(viol)}")
        for v in viol[:10]:
            print(f"     {v}")

        # The rival widths, on the escaped edges only, scored the same way.
        print("\n  RIVAL ESCAPE WIDTHS on the same edges "
              "(ATTR in the 10-byte vocabulary the direct records establish):")
        vocab = set()
        for m in decode_all(gldir, framing).values():
            for (_, form, attr) in m.values():
                if form == "direct" and attr is not None:
                    vocab.add(attr)
        for w in (1, 2, 3, 5):
            d = decode_all(gldir, framing, w)
            good = tot = 0
            for tu, _, callee, _ in edges:
                r = d.get(tu, {}).get(callee)
                if r and r[1] == "escape":
                    tot += 1
                    good += r[2] in vocab
            print(f"    width {w}: {good:>6} / {tot:<6} in the vocabulary")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
