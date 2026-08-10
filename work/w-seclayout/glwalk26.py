#!/usr/bin/env python3
"""w-seclayout — `gl_defined_names_framed` transcribed at THIS tree, and run as
a COUNTERFACTUAL with `GlBindStop::Name26Introduced` REMOVED and nothing else.

Why not `selective_bind`: its `records` comes from `gl_bound_names`, which is
`unwrap_or_default()` over a walk the 380 were selected for STOPPING, so it
reads 0 on 380 of 380 and answers nothing (PREREG §1.1).  This runs the walk
past that one clause, per record, and reports what `Bindings::selective` would
then be handed.

Transcribed from `crates/c2-il/src/func/gl.rs` at `5127a20e`:
  * framing   = `codec::gl_offset_framed_relaxed` (#2783 SHIPPED — the
                `gl[o-5]==0x10` window dropped, `GL_OFFSET_MAX` pinned)
  * name fit  = `NameFit::InlineOrStringTable` (the gate's binding policy,
                w-decouple), so the `INLINE_NAME_MAX` clause does NOT fire and
                the `VariadicRecord` clause does
  * stops     = NameTooFar, RunEndsAt26, DllexportLinkage, VariadicRecord
                — and Name26Introduced RECORDED but NOT taken

  glwalk26.py <bundle.gl> [<bundle.ex>] [--list] [--tsv PATH]
"""
import sys
from collections import Counter

INLINE_NAME_MAX = 8
MAX_NAME_TO_OFFSET = 32
GL_OFFSET_MAX = 0x0100_0000
SEP26 = 0x26
RETSIZE_ESCAPE = 0x80
FN_FLAG_VARARGS = 0x40
LINKAGE_EXPORT_BIT = 0x08


def symbol_runs(gl, sep26=True):
    """`gl::symbol_runs` — (start, end, name), `end` is the terminator index."""
    def is_sep(b):
        return b == 0 or (sep26 and b == SEP26)
    out, i, n = [], 0, len(gl)
    while i < n:
        if not is_sep(gl[i]):
            i += 1
            continue
        start = end = i + 1
        while end < n and not is_sep(gl[end]):
            end += 1
        if end >= n or end == start:
            i += 1
            continue
        b = gl[start:end]
        if all(0x21 <= c <= 0x7E for c in b) and (
                b[0:1] == b"?" or chr(b[0]).isalpha() or b[0:1] == b"_"):
            out.append((start, end, b.decode("ascii")))
        i = end
    return out


def framed_relaxed(gl, o):
    return (o >= 7 and o + 4 < len(gl)
            and gl[o] == 0x80 and gl[o - 7] == 0x80
            and gl[o - 4] == 0 and gl[o - 3] == 0
            and gl[o - 2] == 0 and gl[o - 1] == 0
            and int.from_bytes(gl[o + 1:o + 5], "little") < GL_OFFSET_MAX)


def ex_starts(ex):
    out, i = [], 0
    while i + 1 < len(ex):
        if ex[i] == 0x4F and ex[i + 1] == 0x1F:
            out.append(i)
            i += 2
        else:
            i += 1
    return out


def walk(gl):
    """Returns (rows, stop) where a row is
    (pos, body_start, name, verdict, linkage, flags, introduced_by_26)."""
    runs = symbol_runs(gl, True)
    ends = [e for _, e, _ in runs]
    rows, p, stop = [], 0, None
    import bisect
    while p + 5 <= len(gl):
        if not framed_relaxed(gl, p):
            p += 1
            continue
        off = int.from_bytes(gl[p + 1:p + 5], "little")
        k = bisect.bisect_right(ends, p) - 1
        if k < 0 or p - runs[k][1] > MAX_NAME_TO_OFFSET:
            stop = stop or ("name-too-far", p)
            rows.append((p, off, None, "name-too-far", None, None, None))
            p += 5
            continue
        start, nul, name = runs[k]
        linkage = gl[nul + 3] if nul + 3 < len(gl) else None
        retsz = gl[nul + 4] if nul + 4 < len(gl) else None
        flags = gl[nul + 5] if nul + 5 < len(gl) else None
        intro26 = start > 0 and gl[start - 1] == SEP26
        v = "bound"
        if gl[nul] != 0:
            v = "run-ends-26"
        elif linkage is not None and linkage & LINKAGE_EXPORT_BIT:
            v = "dllexport"
        elif (len(name) <= INLINE_NAME_MAX or not ("?" in name and "@@" in name)) \
                and not (retsz is not None and retsz < RETSIZE_ESCAPE
                         and flags is not None and not flags & FN_FLAG_VARARGS):
            # NameFit::InlineOrStringTable pays for its widening at
            # VariadicRecord: guarded on !looks_mangled, fail-CLOSED.
            v = "varargs-record"
        elif intro26:
            v = "26-introduced"           # RECORDED, NOT TAKEN
        rows.append((p, off, name, v, linkage, flags, intro26))
        if v not in ("bound", "26-introduced") and stop is None:
            stop = (v, p)
        p += 5
    return rows, stop


def main():
    gl = open(sys.argv[1], "rb").read()
    ex = None
    for a in sys.argv[2:]:
        if not a.startswith("--") and a.endswith(".ex"):
            ex = open(a, "rb").read()
    rows, stop = walk(gl)
    c = Counter(r[3] for r in rows)
    print(f"{sys.argv[1]}: {len(gl)} B .gl, {len(rows)} framed defined records")
    for v, n in c.most_common():
        print(f"   {v:<16} {n}")
    print(f"   counterfactual walk (26-stop removed) STOPS AT: "
          f"{stop[0] + ' @' + str(stop[1]) if stop else 'nothing — it completes'}")
    kept = [r for r in rows if r[3] in ("bound", "26-introduced")]
    print(f"   records the counterfactual would hand Bindings::selective: {len(kept)}")
    if ex is not None:
        st = ex_starts(ex)
        offs = [r[1] for r in kept]
        onsplit = sum(1 for o in offs if o in set(st))
        asc = all(offs[i] < offs[i + 1] for i in range(len(offs) - 1))
        print(f"   .ex 4F 1F segments: {len(st)}")
        print(f"   of the {len(kept)} kept records, {onsplit} land on a split point"
              f"  (clause 1: {'PASS' if onsplit == len(kept) else 'FAIL'})")
        print(f"   strictly ascending in segment index (clause 2): "
              f"{'PASS' if asc else 'FAIL'}")
        print(f"   clause 3/4: records {len(kept)} vs segments {len(st)} -> "
              f"{'1:1 (per_record)' if len(kept) == len(st) else 'SELECTIVE — clause 4 EmitSetUnknown unless an unclaimed run fires clause 3 first'}")
    if "--list" in sys.argv:
        for p, off, name, v, lk, fl, i26 in rows:
            lk = "--" if lk is None else f"{lk:02x}"
            fl = "--" if fl is None else f"{fl:02x}"
            print(f"      @{p:<7} start {off:<8} {v:<14} lk={lk} fl={fl} "
                  f"{'26' if i26 else '00'} {name}")
    for a in sys.argv[2:]:
        if a == "--tsv":
            pass
    if "--tsv" in sys.argv:
        dest = sys.argv[sys.argv.index("--tsv") + 1]
        with open(dest, "w") as f:
            f.write("pos\tbody_start\tverdict\tlinkage\tflags\tintro26\tname\n")
            for p, off, name, v, lk, fl, i26 in rows:
                f.write(f"{p}\t{off}\t{v}\t{'' if lk is None else lk}\t"
                        f"{'' if fl is None else fl}\t{int(bool(i26))}\t{name}\n")
        print(f"   -> {dest}")


main()
