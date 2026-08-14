#!/usr/bin/env python3
"""labelil.py — is c2's control-flow LABEL SURCHARGE derivable from the IL?

Lane **w-backedge**, board **#3082**. Control: `work/w-backedge/PREREG.md`,
committed with both grids' sha256 before the first `cl.exe`.

# The question, and the channel

`LABEL_COUNTER.md` §4.1 closed two routes **by measurement**: the surcharge is
not a function of the emitted obj (closed at the BYTE level in §4.2.2 — three
source shapes with one 24-byte `.text` charging +1 / +3 / +1) and it is not in
the `.gl` label seed. §4.1's own remaining sentence is *"a per-function `.ex`
field is the only unexamined channel, and it is open"*.

The channel this script opens is **`.sy`**, not `.ex`.
`crates/c2-il/src/func/sy.rs` documents the per-function record run

    ( 03 <k != 01> <tok> <2 B | 00 <name> 00> <b> <b> )*   label declarations
    ( 1A <b> <tok> <type prefix> <type extent> )*          an undecoded decl
    03 01 <tok> 1F 00 01 01                                block open

so the front end **declares its label tokens per function** in the IL the port
already reads, and `IL_STMT_GRAMMAR.md` §12.5 records that `while` *allocates
three label tokens and uses two* — an allocation count that `.ex` definitions
cannot recover, on a form that charges +2.

# Two measurements per cell, from ONE source file

* **the charge**, seed-free, by `scripts/gt_label_stride.py`'s own construction
  (`a0 · P · a1 · a2`, three identical framed anchors, the base measured in-obj
  so no mode constant appears anywhere). Reused as a module rather than copied:
  a second copy of the stride walker is a second instrument to keep honest.
  §4's rule is `surcharge = stride − base − (minted − 5)`, which for both frame
  classes is `stride − minted`; both columns are printed so a re-derivation can
  see the `minted` it is most likely to drop.
* **the IL features**, from the `.sy` and `.ex` of the *same* `.cpp`.

# The control on the READER, on every row

The three anchors are identical functions. **Their `.sy` label-declaration
counts must be equal**, and the run refuses a row whose anchors disagree rather
than reporting its probe. That is what makes the decl count a reading and not a
guess: a block-boundary parser that has drifted will not produce three equal
anchor counts by accident. A row whose anchors disagree prints `ANCHOR-SPLIT`
and is excluded from every fit.

    work/w-backedge/labelil.py                     # grid1, /O1 (the workload's)
    work/w-backedge/labelil.py --grid 2            # THE HELD-OUT SET
    work/w-backedge/labelil.py --mode '/Ox /GS- /c'
    work/w-backedge/labelil.py --tsv out.tsv       # machine-readable rows

Exit status is non-zero only if a **control** failed (an anchor pair
disagreeing with the in-obj base, or an anchor split). Never because a
prediction did — the table is the result.
"""

import hashlib
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G  # noqa: E402

# The grids are frozen BY CONTENT HASH in PREREG.md §0. `w-keygen` (#2966)
# froze a hold-out by NAME and its population moved -10.8 % underneath it while
# the file stayed byte-identical. A hash that does not match is a hard stop.
FROZEN = {
    1: "3dd6e18f2b857875a9b11ee873137a6c1d0c5f9cd6a3cce1dfbf7e52120a62cd",
    2: "e1e2a5a2623479b472ba10a80eb8a6deb8deeb4daaae11e004de3059a96d1e54",
}


def load_grid(n):
    path = os.path.join(HERE, "grid%d.tsv" % n)
    raw = open(path, "rb").read()
    got = hashlib.sha256(raw).hexdigest()
    if got != FROZEN[n]:
        sys.stderr.write(
            "FATAL: grid%d.tsv moved.\n  frozen %s\n  now    %s\n"
            "The hold-out is the hash, not the filename (PREREG §0).\n"
            % (n, FROZEN[n], got))
        sys.exit(2)
    cells = []
    for line in raw.decode().splitlines():
        if not line or line.startswith("#"):
            continue
        name, cls, body = line.split("\t", 2)
        cells.append((name, cls, body))
    return cells


# ---------------------------------------------------------------------------
# `.sy` — the label-declaration count, per function block.
#
# Nothing here parses a block's BODY. The six widths `sy.rs` records getting
# wrong are all inside the body's `0D <depth> <record>*` groups, and this
# reader never enters one: it locates every block-open header by its fixed
# `1F 00 01 01` tail and reads only the declaration run that ends at it.
# ---------------------------------------------------------------------------
#
# Two corrections the first run forced, both measured off the bytes and both
# the kind of thing a grid that had only been run on `while` would never see:
#
# 1. **The block-open tail is `1F 00 <b> 01`, not `1F 00 01 01`.** `a-if`'s P
#    block is `03 01 ec 09 1f 00 02 01`; a fixed `01` in the third position
#    found 3 of its 4 blocks and the row was refused as `IL SHAPE`. It refused
#    LOUDLY, which is the only reason it was found in the first six cells.
# 2. **The declaration kind is not always `03`.** `a-dowhile` declares its
#    three labels as kind **`02`** (`03 02 f0 09 …`) where `a-while` and
#    `a-for` use kind `03`. A reader that admits only `03` reports `a-dowhile`
#    as having ZERO labels while `.ex` defines four — an absence that looks
#    exactly like a measurement. Every kind != 01 is now counted, and the
#    per-kind histogram is carried into the table, because the kind is itself a
#    candidate feature and merging it into one count would destroy it.
BLOCK_OPEN_TAIL = bytes([0x1F, 0x00])


def token_at(d, p):
    """`crates/c2-il/src/func/readers.rs::read_token_var`, transcribed."""
    if p + 1 >= len(d):
        return None
    b0, b1 = d[p], d[p + 1]
    if b1 & 0x80 == 0:
        return ((b0 << 8) | b1, 2)
    if p + 3 >= len(d):
        return None
    return ((b0 << 24) | (b1 << 16) | (d[p + 2] << 8) | d[p + 3], 4)


def block_opens(sy):
    """Every `03 01 <tok> 1F 00 <b> 01`, as (start, token), in file order."""
    out = []
    i = 0
    while True:
        j = sy.find(BLOCK_OPEN_TAIL, i)
        if j < 0:
            return out
        # walk back over the token to the `03 01`
        for w in (2, 4):
            s = j - w - 2
            if (s >= 0 and sy[s] == 0x03 and sy[s + 1] == 0x01
                    and j + 3 < len(sy) and sy[j + 3] == 0x01):
                t = token_at(sy, s + 2)
                if t and t[1] == w:
                    out.append((s, t[0]))
                    break
        i = j + 1


def decl_record_end(d, p):
    """End offset of one `03 <k!=01> <tok> <2 B | 00 <name> 00> <b> <b>`."""
    if p + 1 >= len(d) or d[p] != 0x03 or d[p + 1] == 0x01:
        return None
    t = token_at(d, p + 2)
    if t is None:
        return None
    q = p + 2 + t[1]
    if q < len(d) and d[q] == 0x00:
        # named form: `00 <name> 00` — a source-level `goto` label.
        e = d.find(b"\x00", q + 1)
        if e < 0:
            return None
        name = d[q + 1:e]
        if not name or not all(0x20 <= c < 0x7F for c in name):
            return None
        q = e + 1
    else:
        q += 2
    return q + 2


def decl_run(d, start, stop):
    """Parse a run of declaration records from `start`; must land on `stop`.

    Returns a list of (kind, token, tail-bytes) or None if the run does not end
    exactly at the block open. Landing exactly is the validation: a run begun
    at the wrong offset overwhelmingly does not.
    """
    p, recs = start, []
    while p < stop:
        e = decl_record_end(d, p)
        if e is None or e > stop:
            return None
        t = token_at(d, p + 2)
        recs.append((d[p + 1], t[0], bytes(d[p + 2 + t[1]:e])))
        p = e
    return recs if p == stop else None


def sy_label_decls(sy):
    """Per function block, the label declarations ahead of it.

    The run is found by trying every start offset in the gap left by the
    previous block and keeping the **longest** run that lands exactly on the
    block open. Reported per block in `.ex` segment order (`sy.rs`: one block
    per `.ex` function segment, in the same order).
    """
    opens = block_opens(sy)
    out = []
    for i, (start, tok) in enumerate(opens):
        lo = 0 if i == 0 else opens[i - 1][0] + 8
        best = None
        # **The run starts where the previous block CLOSED.** The first cut of
        # this reader took the longest run that landed on the block open from
        # anywhere in the gap, and on 8 of 28 cells that reached back into the
        # previous block's body and invented a declaration for an anchor — the
        # ANCHOR-SPLIT control caught every one. A declaration run begins
        # immediately after a `06` block close (or at the file start), so only
        # those offsets are candidates now, and the run must still land exactly
        # on the block open.
        for s in range(lo, start + 1):
            if not (s == 0 or sy[s - 1] == 0x06):
                continue
            r = decl_run(sy, s, start)
            if r is not None and (best is None or len(r) > len(best)):
                best = r
        out.append({"tok": tok, "recs": best})
    return out


# ---------------------------------------------------------------------------
# `.ex` — corroboration only. A raw scan of an operand stream can miscount, so
# these columns are never the primary evidence; they exist so that a `.sy`
# count with no `.ex` definitions behind it is visible as such.
# ---------------------------------------------------------------------------
BODY_MARK = bytes([0x4C, 0x4F, 0x11])


def ex_segments(ex):
    out, at = [], 0
    while True:
        j = ex.find(BODY_MARK, at)
        if j < 0:
            return out
        k = ex.find(BODY_MARK, j + 3)
        out.append(ex[j:k if k > 0 else len(ex)])
        at = j + 3


def ex_cflow(seg):
    """The control-flow label features of one `.ex` segment.

    A raw byte scan, and therefore corroboration rather than primary evidence —
    an operand byte can look like a `29`. What makes it usable is that every
    number here is cross-checked against the `.sy` declaration count, which is
    a positive per-function declaration list and not a scan.

    The **epilogue** label is excluded from every count: `IL_STMT_GRAMMAR.md`
    §9 says every function has exactly one, defined after the body scope
    closes, and every `return` is a `3A` to it. Leaving it in would make
    `return` statements look like control flow and put a +1 on every row.
    """
    # **Cut at the function tail.** `4F 12 47` ends the statement stream
    # (`IL_STMT_GRAMMAR.md` §9); after it come `4F 1F`/`4F 20`/`4F 33` marker
    # blobs whose bytes contain `38`/`3A` and a plausible token — `a-none`,
    # which has no control flow at all, read one forward reference before this
    # cut. Backward counts were never affected (a backward reference needs a
    # matching earlier definition), which is why the rule below rests on them.
    tail = seg.find(bytes([0x4F, 0x12, 0x47]))
    if tail > 0:
        seg = seg[:tail]
    order = []
    i = 0
    while i + 2 < len(seg):
        b = seg[i]
        if b in (0x29, 0x38, 0x39, 0x3A):
            t = token_at(seg, i + 1)
            if t:
                order.append(("d" if b == 0x29 else "r", t[0], b))
        i += 1
    defs = [t for k, t, _ in order if k == "d"]
    epi = defs[-1] if defs else None
    seen, f = set(), {"bwd_uncond": 0, "bwd_cond": 0,
                      "fwd_uncond": 0, "fwd_cond": 0}
    bwd_targets, fwd_targets = set(), set()
    for k, tok, b in order:
        if k == "d":
            seen.add(tok)
            continue
        if tok == epi:
            continue
        if tok in seen:
            f["bwd_uncond" if b == 0x3A else "bwd_cond"] += 1
            bwd_targets.add(tok)
        else:
            f["fwd_uncond" if b == 0x3A else "fwd_cond"] += 1
            fwd_targets.add(tok)
    f["defs"] = len(set(defs)) - (1 if epi is not None else 0)
    f["bwd_t"] = len(bwd_targets)
    f["fwd_t"] = len(fwd_targets - bwd_targets)
    return f


# ---------------------------------------------------------------------------
def flags_file(mode, wd):
    p = os.path.join(wd, "flags_%s.txt" % abs(hash(mode)))
    open(p, "w").write("/nologo " + mode + "\n")
    return p


def capture_il(cpp, mode, wd):
    out = os.path.join(wd, "il_" + os.path.basename(cpp)[:-4])
    os.makedirs(out, exist_ok=True)
    binp = os.path.join(REPO, "target", "release", "c2rs")
    r = subprocess.run([binp, "capture", cpp, "--keep-il", out,
                        "--flags-file", flags_file(mode, wd)],
                       capture_output=True, text=True)
    if r.returncode != 0:
        return None
    got = {}
    for f in os.listdir(out):
        got[f.rsplit(".", 1)[-1]] = os.path.join(out, f)
    if "sy" not in got or "ex" not in got:
        return None
    return got


def main(argv):
    mode = "/O1 /GS- /c"
    gridn = 1
    tsv = None
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    if "--grid" in argv:
        i = argv.index("--grid"); gridn = int(argv[i + 1]); del argv[i:i + 2]
    if "--tsv" in argv:
        i = argv.index("--tsv"); tsv = argv[i + 1]; del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]

    cells = load_grid(gridn)
    if want:
        cells = [c for c in cells if c[0] in want]
    wd = tempfile.mkdtemp(prefix="wbe")
    print("grid %d (sha256 verified)   mode: %s   %d cells" % (gridn, mode, len(cells)))
    print("  charge   = stride - minted            (LABEL_COUNTER.md §4's `minted` correction)")
    print("  sy_decls = `03 <k!=01>` label declarations ahead of P's `.sy` block")
    print("  CONTROL  : the three identical anchors' sy_decls must be EQUAL")
    print()
    print("%-18s %7s %7s %7s %6s %-11s %6s %7s %6s %5s %5s %5s %5s  %s"
          % ("cell", "stride", "minted", "CHARGE", "decls", "kinds",
             "owned", "anchors", "exdefs", "bwdT", "bwdU", "bwdC", "fwdT",
             "note"))
    bad = 0
    rows = []
    for name, cls, body in cells:
        tag = name.replace("-", "_")
        probe = "int P(int a){ %s }" % body
        src = G.build_src("int ga(int);", [], probe)
        o = G.capture(src, mode, wd, tag)
        if o is None:
            print("%-18s  CAPTURE FAILED" % name); bad += 1; continue
        gs = G.groups(o)
        def find(s):
            for g in gs:
                if g["name"].startswith("?" + s + "@@"):
                    return g
            return None
        a0, a1, a2, P = find("a0"), find("a1"), find("a2"), find("P")
        if not (a0 and a1 and a2 and P):
            print("%-18s  missing group" % name); bad += 1; continue
        f = lambda g: min(g["labels"]) if g["labels"] else None
        base = f(a2) - f(a1)
        if base not in (4, 5):
            print("%-18s  CONTROL FAILED: base %s" % (name, base)); bad += 1; continue
        stride = f(a1) - f(a0) - base
        mint = G.minted(P)
        charge = stride - mint
        framed = f(P) is not None
        if framed != (cls == "framed"):
            note = "CLASS-MISPREDICT(obj says %s)" % ("framed" if framed else "leaf")
        else:
            note = ""

        il = capture_il(os.path.join(wd, tag + ".cpp"), mode, wd)
        if il is None:
            print("%-18s  IL CAPTURE FAILED" % name); bad += 1; continue
        sy = open(il["sy"], "rb").read()
        ex = open(il["ex"], "rb").read()
        blocks = sy_label_decls(sy)
        segs = ex_segments(ex)
        # `a0 · P · a1 · a2` — P is function index 1 in source order, and the
        # `.sy` blocks are in `.ex` segment order (`sy.rs`).
        if len(blocks) != 4 or len(segs) != 4:
            print("%-18s  IL SHAPE: %d sy blocks, %d ex segments (want 4/4)"
                  % (name, len(blocks), len(segs))); bad += 1; continue
        nrec = lambda i: (None if blocks[i]["recs"] is None
                          else len(blocks[i]["recs"]))
        anc = [nrec(i) for i in (0, 2, 3)]
        anchors_ok = len(set(anc)) == 1 and anc[0] is not None
        precs = blocks[1]["recs"]
        pk = nrec(1)
        kinds = "-" if not precs else ",".join(
            "%d:%d" % (k, sum(1 for x in precs if x[0] == k))
            for k in sorted({x[0] for x in precs}))
        # `owned` = tokens the front end allocated between P's exit label and
        # the NEXT function's exit label. Every function pays 3 of it for the
        # next function's formal + own symbol + exit label (the three anchors
        # read exactly 3), so `owned - 3` is P's own locals + ALL its label
        # tokens, used and unused. §12.5's "allocates three and uses two" is
        # only visible here — an unused label is a token gap and nothing else.
        owned = (blocks[2]["tok"] - blocks[1]["tok"]) // 0x100
        if not anchors_ok:
            note = ("ANCHOR-SPLIT %s" % anc) + (" " + note if note else "")
            bad += 1
        x = ex_cflow(segs[1])
        print("%-18s %7d %7d %7d %6s %-11s %6d %7s %6d %5d %5d %5d %5d  %s"
              % (name, stride, mint, charge,
                 "?" if pk is None else pk, kinds, owned,
                 ",".join("?" if x2 is None else str(x2) for x2 in anc),
                 x["defs"], x["bwd_t"], x["bwd_uncond"], x["bwd_cond"],
                 x["fwd_t"], note))
        row = {"cell": name, "class": cls, "framed": framed,
               "stride": stride, "minted": mint, "charge": charge,
               "sy_decls": pk, "sy_kinds": kinds, "owned": owned,
               "anchors_ok": anchors_ok}
        row.update({"ex_" + k: v for k, v in x.items()})
        rows.append(row)
    if tsv:
        with open(tsv, "w") as fh:
            keys = ["cell", "class", "framed", "stride", "minted", "charge",
                    "sy_decls", "sy_kinds", "owned", "anchors_ok", "ex_defs",
                    "ex_bwd_t", "ex_bwd_uncond", "ex_bwd_cond", "ex_fwd_t",
                    "ex_fwd_uncond", "ex_fwd_cond"]
            fh.write("\t".join(keys) + "\n")
            for row in rows:
                fh.write("\t".join(str(row[k]) for k in keys) + "\n")
        print("\nwrote %s (%d rows)" % (tsv, len(rows)))
    print("\ncontrols failed: %d" % bad)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
