#!/usr/bin/env python3
"""sweep.py — which of the port's emission productions has the ORACLE never seen?

Lane w-frame, at the coordinator's request. The defect this answers is trap 5's
thirteenth instance, found in this lane: `bt` and `cmpwi` were WRITTEN, passing a
unit test that compared the port's table to ITSELF, and never once graded against
real `c2`. That failure mode emits nothing wrong -- the code simply never runs --
so no byte compare, no census disagreement and no `mismatch` can see it.

METHOD -- derived from the source via coverage, not from a hand-written list.
The list is the defect: a table test proves completeness only over the list it
was written from.

Two profiles, and the difference between them is the whole instrument:

  A = GRADED.  Coverage of `c2rs perf --fixtures <the 104 Port=Match fixtures>`.
      Every one of those builds produced a whole obj that was then byte-compared
      against real c2 and matched, so every region executed under A is graded by
      the oracle.

  B = REACHED. Coverage of `c2rs gap` over all 221 fixtures at all 12
      `scripts/lanes.txt` flag lanes. This includes the 117 NotImplemented
      fixtures, whose codegen paths run as far as the refusal and are NEVER
      byte-compared. B therefore OVER-credits, on purpose.

  never in B          -> never executed at all. Definitively ungraded.
  in B but not in A   -> executed only on a refusal path, or only at a flag lane
                         whose obj the port declined. THE SUSPECT BAND -- this is
                         where `branch_sense`'s five rows lived.
  in A                -> graded.

Both bounds are reported because neither alone is honest: A alone under-credits
(a region reachable only at /O1 looks dead), B alone over-credits (a refusal
path looks graded).
"""
import json, os, sys, collections

HERE = os.path.dirname(os.path.abspath(__file__))

# Which module the instrument reports on. The default is the codegen emitters,
# which is what this sweep was written for — but F-c is a rule about *any* code
# path a rung adds, and a rung whose new path lives elsewhere (`coff/order.rs`,
# lane w-order) could otherwise run this, see a clean report, and read the
# instrument's SCOPE as coverage. Overridable so a lane can point it at the
# module it touched instead of answering F-c by hand:
#
#     C2RS_SWEEP_KEEP=/crates/c2-core/src/coff/ work/w-frame/sweep.py
KEEP = os.environ.get("C2RS_SWEEP_KEEP", "/crates/c2-core/src/codegen/")


def load(path):
    """-> {file: {(l0,c0,l1,c1): count}} over the codegen dir, plus per-function."""
    d = json.load(open(path))
    regions = collections.defaultdict(dict)
    funcs = {}
    for exp in d["data"][0].get("functions", []):
        fnames = [f for f in exp.get("filenames", []) if KEEP in f]
        if not fnames:
            continue
        funcs[exp["name"]] = (exp["count"], fnames[0])
    for f in d["data"][0]["files"]:
        if KEEP not in f["filename"]:
            continue
        for r in f.get("segments", []):
            pass
    # region detail lives under functions/regions with a file index
    for exp in d["data"][0].get("functions", []):
        for r in exp.get("regions", []):
            # [line_start, col_start, line_end, col_end, count, file_id, ...]
            fid = r[5]
            fn = exp["filenames"][fid] if fid < len(exp["filenames"]) else None
            if not fn or KEEP not in fn:
                continue
            key = (r[0], r[1], r[2], r[3])
            cur = regions[fn].get(key, 0)
            regions[fn][key] = cur + r[4]
    return regions, funcs


A_reg, A_fn = load(os.path.join(HERE, "cov", "export.json"))
B_reg, B_fn = load(os.path.join(HERE, "cov2", "export.json"))

print("=" * 78)
print("PER-FUNCTION: never called anywhere (B), and called-but-never-graded (B\\A)")
print("=" * 78)
never, band = [], []
for name, (bcount, fname) in sorted(B_fn.items(), key=lambda kv: kv[1][1]):
    acount = A_fn.get(name, (0, fname))[0]
    short = fname.split(KEEP)[-1]
    if bcount == 0:
        never.append((short, name))
    elif acount == 0:
        band.append((short, name, bcount))

print("\n--- NEVER EXECUTED, in any lane, by any fixture (%d) ---" % len(never))
for short, name in never:
    print("  %-24s %s" % (short, name))
print("\n--- EXECUTED but NEVER under a graded (Port=Match) build (%d) ---" % len(band))
for short, name, c in band:
    print("  %-24s %-60s reached %dx" % (short, name[:60], c))

print()
print("=" * 78)
print("PER-REGION, per file: uncovered branch/arm counts")
print("=" * 78)
print("%-22s %7s %7s %7s   %s" % ("file", "regions", "0 in A", "0 in B", "meaning"))
tot = [0, 0, 0]
for fname in sorted(set(A_reg) | set(B_reg)):
    short = fname.split(KEEP)[-1]
    keys = set(A_reg.get(fname, {})) | set(B_reg.get(fname, {}))
    a0 = sum(1 for k in keys if A_reg.get(fname, {}).get(k, 0) == 0)
    b0 = sum(1 for k in keys if B_reg.get(fname, {}).get(k, 0) == 0)
    tot[0] += len(keys); tot[1] += a0; tot[2] += b0
    print("%-22s %7d %7d %7d   %d in the SUSPECT BAND" % (short, len(keys), a0, b0, a0 - b0))
print("%-22s %7d %7d %7d   %d in the SUSPECT BAND" % ("TOTAL", tot[0], tot[1], tot[2], tot[1] - tot[2]))

print()
print("=" * 78)
print("THE SUSPECT BAND, by source line — regions reached but never graded")
print("=" * 78)
src = {}
for fname in sorted(set(A_reg) | set(B_reg)):
    short = fname.split(KEEP)[-1]
    path = os.path.join(HERE, "..", "..", KEEP.strip("/"), short)
    try:
        src[short] = open(path).read().splitlines()
    except OSError:
        src[short] = []
    keys = set(A_reg.get(fname, {})) | set(B_reg.get(fname, {}))
    rows = sorted(k for k in keys
                  if A_reg.get(fname, {}).get(k, 0) == 0 and B_reg.get(fname, {}).get(k, 0) > 0)
    if not rows:
        continue
    print("\n-- %s (%d regions)" % (short, len(rows)))
    seen = set()
    for k in rows:
        ln = k[0]
        if ln in seen:
            continue
        seen.add(ln)
        text = src[short][ln - 1].strip() if ln - 1 < len(src[short]) else "?"
        print("   %5d  %s" % (ln, text[:96]))

print()
print("=" * 78)
print("NEVER EXECUTED AT ALL (not in B) — by source line, first line of each region")
print("=" * 78)
for fname in sorted(set(A_reg) | set(B_reg)):
    short = fname.split(KEEP)[-1]
    keys = set(A_reg.get(fname, {})) | set(B_reg.get(fname, {}))
    rows = sorted(k for k in keys if B_reg.get(fname, {}).get(k, 0) == 0)
    if not rows:
        continue
    lines = sorted({k[0] for k in rows})
    print("\n-- %s: %d regions on %d lines" % (short, len(rows), len(lines)))
    # collapse to contiguous runs so the shape is readable
    runs, cur = [], [lines[0], lines[0]]
    for ln in lines[1:]:
        if ln <= cur[1] + 2:
            cur[1] = ln
        else:
            runs.append(cur); cur = [ln, ln]
    runs.append(cur)
    for a, b in runs:
        text = src[short][a - 1].strip() if a - 1 < len(src[short]) else "?"
        print("   %5s  %s" % ("%d-%d" % (a, b) if b > a else str(a), text[:88]))

print()
print("=" * 78)
print("TRIAGE — a refusal emits NO BYTES, so it has nothing for the oracle to")
print("compare. Only an EMISSION region that never ran is the `branch_sense` shape.")
print("=" * 78)
REFUSAL = ("out_of_class", "return None", "unreachable!", "map_err", "ok_or_else",
           "Err(", "?;", ".ok_or", "=> None", "return Some(Err")
EMIT = ("extend_from_slice", "encode_", "push(", ".to_be_bytes", "text.extend")


def kind(text):
    t = text.strip()
    if any(s in t for s in EMIT) and not any(s in t for s in ("out_of_class", "unreachable!")):
        return "EMIT"
    if any(s in t for s in REFUSAL):
        return "refusal/guard"
    return "other"


print("%-22s %8s %8s %8s %8s" % ("file", "never", "EMIT", "refusal", "other"))
grand = collections.Counter()
emit_lines = []
for fname in sorted(set(A_reg) | set(B_reg)):
    short = fname.split(KEEP)[-1]
    keys = set(A_reg.get(fname, {})) | set(B_reg.get(fname, {}))
    lines = sorted({k[0] for k in keys if B_reg.get(fname, {}).get(k, 0) == 0})
    c = collections.Counter()
    for ln in lines:
        text = src[short][ln - 1] if ln - 1 < len(src[short]) else ""
        k = kind(text)
        c[k] += 1
        if k == "EMIT":
            emit_lines.append((short, ln, text.strip()))
    grand.update(c)
    if lines:
        print("%-22s %8d %8d %8d %8d" % (short, len(lines), c["EMIT"], c["refusal/guard"], c["other"]))
print("%-22s %8d %8d %8d %8d" % ("TOTAL", sum(grand.values()), grand["EMIT"],
                                 grand["refusal/guard"], grand["other"]))
print("\n--- the never-executed EMISSION lines, the only ones of `branch_sense`'s shape ---")
for short, ln, text in emit_lines:
    print("  %-18s %5d  %s" % (short, ln, text[:88]))
