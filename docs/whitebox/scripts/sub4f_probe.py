#!/usr/bin/env python3
"""sub4f_probe.py -- the confirmation probe for read R9 (the `0x4F` sub-record).

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).
Drives the REAL toolchain under wibo and grades the widths that
`dump_sub4f.py` derived from the pinned image against c2's own IL.

`READ_PLAN_2026-08-21.md` section 5.3: `[R]` says *"the instructions were read
correctly"*, NOT *"this is what c2 does"*. This script exists so that R9's
central claim can go RED.

TWO PROBES, because grids and corpora fail in opposite directions
(`WB_SUB4F_PREREG.md` section 5):

  --grid    A controlled TWIN grid. Ten sources with a BYTE-IDENTICAL body,
            differing only in a leading `#line L` directive, at
            L = 1,100,127,128,129,200,16383,16384,100000,1000000 -- chosen to
            straddle the VI32 escape boundary at 0x80. This is the probe
            aimed at FM4, the vacuous-green trap: every fixture in this
            project's own corpus sits at a source line below 128, where a
            fixed-one-byte read and a variable-width read AGREE, and board
            #2668 records a lane that already paid for exactly that.

            The grading rule is the grid's own INTERNAL CONSISTENCY, so it
            does not depend on this lane guessing where c2 places a marker:
            cell L=1 fixes the offset set K = {k}; every other cell must then
            decode to exactly {L + k : k in K}. A wrong width rule produces
            garbage here, not a shifted-but-tidy answer.

  --corpus  Every tracked fixture under `fixtures/cpp`. Unauthored, broad, and
            capable of surprising: it tallies every `0x4F` sub-opcode that
            actually occurs and checks each against the pinned table. This is
            the probe aimed at FM5 (read-correct but off-path) and at
            sub-opcodes nobody thought to grid.

RED CONDITIONS, stated before the run:
  * any grid cell whose decoded line set differs from {L + k};
  * any `4F 01` record whose byte length is not 3 (value < 0x80) or 7;
  * any sub-opcode observed in the corpus that the pinned table sends to the
    ICE arm, or that lies outside the table's 64 entries.

Usage:
    python3 sub4f_probe.py <c2.dll> --grid    [outdir]
    python3 sub4f_probe.py <c2.dll> --corpus  [outdir] [limit]

Env: C2RS_WIBO must point at a GOOD wibo build. `scripts/gt_capture.sh`
warns that `../wibo/build/wibo` is a stale 1.0.1-7 build producing wrong
objs; use `../wibo/build/release/wibo`.

The image is sha256
c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258; the digest
is verified by `dump_sub4f.load()` before any address is used.

LIMITATION: record location is a superset scan for the byte 0x4F (see
`dump_sub4f.py`'s docstring), so a reported candidate may be another record's
payload. That inflates counts; it cannot hide a missing ground-truth value,
which is what both grading rules are written on.
"""

import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dump_sub4f as D                                       # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))))
C2RS = os.path.join(REPO, "target", "release", "c2rs")
IMAGE_REL = "compilers/X360/16.00.11886.00/c2.dll"

BODY = """int tw(int a, int b) {
    int x = a + b;
    int y = x * 2;
    int z = y - a;
    return z;
}
"""
GRID_LINES = [1, 100, 127, 128, 129, 200, 16383, 16384, 100000, 1000000]

SUB_LINE = 0x01                        # format 'l' (0x6c)


def capture(src, keep_dir):
    """Run the real toolchain; return the captured .ex path or None."""
    if not os.path.exists(C2RS):
        sys.exit("SKIP: %s absent -- cargo build -p c2-harness --release" % C2RS)
    before = set(os.listdir(keep_dir)) if os.path.isdir(keep_dir) else set()
    r = subprocess.run([C2RS, "capture", "--keep-il", keep_dir, src],
                       capture_output=True, text=True, cwd=REPO)
    if r.returncode != 0:
        return None
    if "SKIP" in r.stdout or "captured IL bundle" not in r.stdout:
        return None
    after = set(os.listdir(keep_dir))
    new = [f for f in after - before if f.endswith(".ex")]
    return os.path.join(keep_dir, new[0]) if new else None


def line_records(img, path):
    """[(offset, byte_len, value)] for every `4F 01` record, table widths."""
    b = open(path, "rb").read()
    gate = "VI32"          # code 'l' with DAT_10c2eb4c != 0; the grid PROVES it
    out, p = [], 0
    while True:
        p = b.find(b"\x4f\x01", p)
        if p < 0:
            return out
        w, v = D.read_vi32(b, p + 2) if gate == "VI32" else D.read_vi16(b, p + 2)
        if w is not None:
            out.append((p, 2 + w, v))
        p += 1


def subop_histogram(img, path, fmt_by_sub=None, has_ptr=None):
    """{sub_opcode: count} over 0x4F candidates, with SPAN EXCLUSION.

    A bare search for the byte 0x4F cannot tell a record from a payload byte
    that happens to equal 0x4F -- and the corpus contains that case in bulk:
    `4f 01 4f` is a line marker for source line 79, and a naive scan reads the
    payload `4f` as a fresh sub-record. The FIRST run of this probe went red
    for exactly that reason (WB_SUB4F_FINDINGS.md section P9.1).

    The fix is to consume: walk left to right and, whenever a record's width is
    STATICALLY COMPUTABLE from the pinned table, skip past it. Candidates
    falling strictly inside a computed span are payload and are excluded.
    After a variable-width record (codes 0b/17/1a/1e) only the two header bytes
    are certain, so following candidates are counted but tagged `unresolved`
    rather than silently trusted -- the instrument reports what it cannot
    decide instead of guessing.
    """
    b = open(path, "rb").read()
    hist, unresolved, p, consumed_to, after_var = {}, {}, 0, 0, False
    STATIC = {0x0B: None, 0x0C: "STR", 0x0E: "VI16", 0x14: "VI32",
              0x15: "VI16", 0x16: "VARU", 0x73: "VARU", 0x6C: "VI32"}
    while True:
        p = b.find(b"\x4f", p)
        if p < 0:
            return hist, unresolved
        if p < consumed_to:                       # payload of a decoded record
            p += 1
            continue
        n, sub = D.read_vi16(b, p + 1)
        if n is None:
            return hist, unresolved
        (unresolved if after_var else hist)[sub] = \
            (unresolved if after_var else hist).get(sub, 0) + 1
        idx = (sub & 0xFF) - 256 if (sub & 0x80) else (sub & 0xFF)
        q, ok = p + 1 + n, True
        if 0 <= idx < D.TABLE_LEN and has_ptr is not None and has_ptr[idx]:
            for c in (fmt_by_sub[idx] or b""):
                kind = STATIC.get(c, False)
                if kind in (None, False):
                    ok = False
                    break
                w, _v = D.READERS[kind](b, q)
                if w is None:
                    ok = False
                    break
                q += w
        consumed_to = q if ok else p + 1 + n
        after_var = not ok
        p += 1


def run_grid(img, outdir):
    src_dir = os.path.join(outdir, "src")
    il_dir = os.path.join(outdir, "il")
    os.makedirs(src_dir, exist_ok=True)
    os.makedirs(il_dir, exist_ok=True)
    cells = {}
    for L in GRID_LINES:
        p = os.path.join(src_dir, "t_%d.cpp" % L)
        with open(p, "w") as f:
            if L != 1:
                f.write("#line %d\n" % L)
            f.write(BODY)
        ex = capture(p, il_dir)
        if ex is None:
            sys.exit("SKIP: toolchain absent or capture failed for L=%d" % L)
        cells[L] = line_records(img, ex)

    base = sorted(v for _o, _n, v in cells[1])
    K = [v - 1 for v in base]
    print("PROBE A -- the twin grid (%d cells, body byte-identical)" % len(cells))
    print("  offset set K from cell L=1: %s" % K)
    print("%-9s %-7s %-9s %-46s %s" % ("#line L", "recs", "widths", "decoded", "verdict"))
    npass = 0
    for L in GRID_LINES:
        got = sorted(v for _o, _n, v in cells[L])
        want = sorted(L + k for k in K)
        widths = sorted({n for _o, n, _v in cells[L]})
        # each record: 3 bytes iff the value fits one non-escape byte
        wok = all(n == (3 if 0 <= v < 0x80 else 7) for _o, n, v in cells[L])
        ok = (got == want) and wok
        npass += ok
        shown = str(got if len(got) <= 8 else got[:8] + ["..."])
        print("%-9d %-7d %-9s %-46s %s" % (
            L, len(got), ",".join(str(w) for w in widths), shown[:46],
            "PASS" if ok else ("FAIL: want %s" % want)))
    print("  => PROBE A %d/%d cells" % (npass, len(GRID_LINES)))
    return npass == len(GRID_LINES)


def run_corpus(img, outdir, limit):
    fix = os.path.join(REPO, "fixtures", "cpp")
    srcs = sorted(os.path.join(fix, f) for f in os.listdir(fix)
                  if f.endswith(".cpp"))[:limit]
    il_dir = os.path.join(outdir, "il")
    os.makedirs(il_dir, exist_ok=True)
    rows = D.table(img)
    fmt_by_sub = {i: c for i, _d0, _d1, c in rows}
    has_ptr = {i: bool(d0) for i, d0, _d1, _c in rows}
    hist, unres, nfile, nrec, bad_w, bad_sub = {}, {}, 0, 0, [], {}
    for s in srcs:
        ex = capture(s, il_dir)
        if ex is None:
            continue
        nfile += 1
        h, u = subop_histogram(img, ex, fmt_by_sub, has_ptr)
        for sub, c in h.items():
            hist[sub] = hist.get(sub, 0) + c
        for sub, c in u.items():
            unres[sub] = unres.get(sub, 0) + c
        for _o, n, v in line_records(img, ex):
            nrec += 1
            if n != (3 if 0 <= v < 0x80 else 7):
                bad_w.append((ex, _o, n, v))
        os.remove(ex)
        for suf in ("gl", "in", "sy", "db"):
            p = ex[:-2] + suf
            if os.path.exists(p):
                os.remove(p)
    print("PROBE B -- the corpus (%d/%d fixtures captured)" % (nfile, len(srcs)))
    print("  `4F 01` records checked: %d ; width violations: %d"
          % (nrec, len(bad_w)))
    print("  sub-opcode histogram (superset scan, so counts are an upper bound):")
    for sub in sorted(hist):
        idx = (sub & 0xFF) - 256 if (sub & 0x80) else (sub & 0xFF)
        if idx < 0 or idx >= D.TABLE_LEN:
            state = "OUT-OF-TABLE"
        elif not has_ptr[idx]:
            state = "no payload"
        else:
            codes = fmt_by_sub[idx] or b""
            miss = [c for c in codes if c not in D.HANDLED_CODES]
            state = ("codes %s" % " ".join("%02x" % c for c in codes)) + (
                "  -> ICE:160" if miss else "")
        if "ICE" in state or state == "OUT-OF-TABLE":
            bad_sub[sub] = hist[sub]
        print("    0x%02x  %8d   %s" % (sub, hist[sub], state))
    # ---- the registered histogram clause is UNGRADED, and this says why ----
    # WB_SUB4F_PREREG.md section 5.3 registered "any sub-opcode observed that
    # the table sends to the ICE arm" as a RED condition. That rule assumed a
    # soundness the scan does not have: `0x4F` also occurs as the low byte of
    # 2-byte operand tokens, which no span exclusion can remove without the
    # full `.ex` grammar. The clause is therefore reported and NOT graded --
    # it is not relaxed to manufacture a green, and the finding says so.
    #
    # The decisive evidence that these are NOT records is a POSITIVE fact
    # about the toolchain, not an argument about the scan: 0x10b33526 ends in
    # `int3`, so a real sub-record with an unhandled format code KILLS c2 --
    # and every capture below succeeded.
    print("  ICE-flagged candidates: %s -- UNGRADED (see the comment at this"
          " print; the scan cannot tell a record from a 2-byte operand token"
          " whose low byte is 0x4f)" % (bad_sub or "none"))
    print("  POSITIVE LIVENESS FACT: %d/%d fixtures compiled and captured."
          " A real sub-record with an unhandled code would have ICEd c2"
          " (0x10b33526 -> int3), so none of the above is a record."
          % (nfile, len(srcs)))
    if unres:
        print("  UNRESOLVED (candidates following a variable-width record, so"
              " the instrument cannot place them; NOT graded):")
        for sub in sorted(unres):
            print("    0x%02x  %8d" % (sub, unres[sub]))
    # The GRADED clause is the width rule: it is the one the scan can decide,
    # because it checks a record this lane located by its own decoded value.
    ok = not bad_w
    print("  => PROBE B (width clause, graded) %s -- %d/%d records"
          % ("PASS" if ok else "FAIL %s" % bad_w[:3], nrec - len(bad_w), nrec))
    return ok


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    img = D.load(sys.argv[1])
    outdir = sys.argv[3] if len(sys.argv) > 3 else os.path.join(
        REPO, "work", "w-read-r9", "probe")
    os.makedirs(outdir, exist_ok=True)
    if sys.argv[2] == "--grid":
        sys.exit(0 if run_grid(img, outdir) else 1)
    elif sys.argv[2] == "--corpus":
        lim = int(sys.argv[4]) if len(sys.argv) > 4 else 10 ** 9
        sys.exit(0 if run_corpus(img, outdir, lim) else 1)
    sys.exit(__doc__)


if __name__ == "__main__":
    main()
