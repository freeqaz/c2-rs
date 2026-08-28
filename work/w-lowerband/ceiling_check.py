#!/usr/bin/env python3
"""ceiling_check.py -- RE-READ of w-sizebracket's already-measured series
(work/w-sizebracket/series.jsonl, 176 cells, committed) against the ceiling this
lane read out of the image: DAT_10c46318 = 0x10 << DAT_10c2ea98 = 0x10 << 3 = 128.

No compilation.  Read-before-probe: the cells exist, so the question "does the
.gl SIZE separate inlined from kept at 128?" is answered by re-reading them.

std only; tooling, not crates/.
"""
import json, sys

PATH = 'work/w-sizebracket/series.jsonl'
CEIL = 128        # 0x10 << k, k = DAT_10c2ea98 = 3 (raw .data, file offset 0x12dc98)


def main():
    rows = []
    seen = set()
    with open(PATH) as f:
        for ln in f:
            ln = ln.strip()
            if not ln:
                continue
            r = json.loads(ln)
            key = (r['tag'],)
            if key in seen:          # the file has duplicate tags; de-dup
                continue
            seen.add(key)
            rows.append(r)
    print(f"cells: {len(rows)} unique tags "
          f"(profiles: {sorted(set(r['profile'] for r in rows))}, "
          f"families: {sorted(set(r['family'] for r in rows))})")
    print()

    for prof in sorted(set(r['profile'] for r in rows)):
        sub = [r for r in rows if r['profile'] == prof]
        print(f"=== profile {prof}  ({len(sub)} cells)")
        # 1. is `gl_size < CEIL` a both-ways separator?
        tp = [r for r in sub if r['gl_size'] < CEIL and r['arm'] == 'inlined']
        fp = [r for r in sub if r['gl_size'] < CEIL and r['arm'] != 'inlined']
        tn = [r for r in sub if r['gl_size'] >= CEIL and r['arm'] != 'inlined']
        fn = [r for r in sub if r['gl_size'] >= CEIL and r['arm'] == 'inlined']
        print(f"  rule `.gl SIZE < {CEIL}` vs the verdict:")
        print(f"    SIZE <  {CEIL} and inlined : {len(tp):>3}")
        print(f"    SIZE <  {CEIL} and KEPT    : {len(fp):>3}   <- refutes 'SIZE<ceil => inlined'")
        print(f"    SIZE >= {CEIL} and kept    : {len(tn):>3}")
        print(f"    SIZE >= {CEIL} and INLINED : {len(fn):>3}   <- refutes 'SIZE>=ceil => kept'")
        for r in fp:
            print(f"       KEPT below the ceiling : {r['tag']:<16} SIZE {r['gl_size']:>3} "
                  f"attr {r['gl_attr']} callee_text {r['callee_text']}")
        for r in fn[:8]:
            print(f"       INLINED above          : {r['tag']:<16} SIZE {r['gl_size']:>3} "
                  f"attr {r['gl_attr']} callee_text {r['callee_text']}")
        if len(fn) > 8:
            print(f"       ... and {len(fn)-8} more")
        # 2. per-family boundary in gl_size
        print("  per-family boundary in .gl SIZE (last inlined, first kept]:")
        for fam in sorted(set(r['family'] for r in sub)):
            fr = sorted([r for r in sub if r['family'] == fam],
                        key=lambda r: r['gl_size'])
            ins = [r['gl_size'] for r in fr if r['arm'] == 'inlined']
            kep = [r['gl_size'] for r in fr if r['arm'] != 'inlined']
            print(f"    {fam:<8} n={len(fr):>3}  inlined SIZE max "
                  f"{max(ins) if ins else '-':>5}   kept SIZE min "
                  f"{min(kep) if kep else '-':>5}")
        print()
    return 0


if __name__ == '__main__':
    sys.exit(main())
