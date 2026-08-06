#!/usr/bin/env python3
"""resync.py — a VERDICT on w-emitp2 §4.1's uniform ±2.

w-emitp2 measured its crate-shape transcript short by **exactly 2 symbol
addresses on 806 of 850 TUs** against the shipping reader's own cursor
(`T = 1,427,984`, `B = 1,429,596`, `Σ|Δ| = 1,612 = 806 × 2`) and attributed it
to the crate's **resync**: after a refusal the anchor scan resumes and can
re-anchor inside the tail of the record it just refused, which a sequential
parser cannot do.  It counted the gap and did not decode it.

This decodes it.  It emulates `crates/c2-il`'s anchor scan — the `00 01`/`00 02`
anchors, the token-width check, the fail-closed arm, the `i += 2` resync after a
named refusal and the `i = p` advance after an accepted record — under the
**PRE-w-inread acceptance**, and diffs the records it frames against the
sequential parse's.  Every record the anchor scan frames and the sequential
parse does not is printed with its stream offset and its symbol addresses, so
the question *"are the extra two REAL records or phantoms?"* gets an answer and
not an adjective.

    usage: resync.py <cacheidx.tsv> [jobs] [limit] [--dump <src>]

stdlib only.  Reads no c2 output.  Imports the framing from
`work/w-emitp2/strictin.py` and nothing from `crates/`.
"""
import collections
import os
import struct
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT", os.path.abspath(os.path.join(HERE, "..", "..")))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(MAIN, "work", "w-emitp2"))
import localize as L   # noqa: E402
import strictin as si  # noqa: E402

SYM = 0x02
#: The acceptance as it stood at master `e928ee7`, BEFORE this lane.
OLD_TYPES = (0x01, 0x02)
OLD_WIDTHS = (1, 2, 4)


def read_token_var(b, p):
    if p + 1 >= len(b):
        return None
    if not (b[p + 1] & 0x80):
        return ((b[p] << 8) | b[p + 1], 2)
    if p + 3 >= len(b):
        return None
    return (((b[p] << 24) | (b[p + 1] << 16) | (b[p + 2] << 8) | b[p + 3]), 4)


def read_value(b, p, w):
    """`ininit.rs::read_value`."""
    if p >= len(b):
        return None
    b0 = b[p]
    if w == 1:
        return (1, p + 1)
    if b0 < 0x80:
        return (1, p + 1)
    if b0 != 0x80:
        return None
    if p + 1 + w > len(b):
        return None
    return (1, p + 1 + w)


def read_offset(b, p):
    if p >= len(b):
        return None
    b0 = b[p]
    if b0 < 0x80:
        return (b0, p + 1)
    if b0 != 0x80:
        return None
    if p + 5 > len(b):
        return None
    return (struct.unpack_from("<i", b, p + 1)[0], p + 5)


def read_elements_old(b, p):
    """`ininit.rs::read_elements` AT MASTER `e928ee7` — the narrow acceptance.

    -> (nbytes, refs, next_p) or ('ERR', reason, None).
    """
    out = 0
    refs = []
    while True:
        if p >= len(b):
            return ("ERR", "truncated", None)
        tag = b[p]
        if tag == 0x07:
            return (out, refs, p + 1)
        if tag == SYM:
            t = read_token_var(b, p + 1)
            if t is None:
                return ("ERR", "truncated", None)
            q = p + 1 + t[1]
            o = read_offset(b, q)
            if o is None:
                return ("ERR", "value-did-not-frame", None)
            _off, q = o
            if q >= len(b):
                return ("ERR", "truncated", None)
            if b[q] != 0x04:
                return ("ERR", "symbol-address", None)
            q += 1
            refs.append((t[0], out))
            out += 4
            p = q
            if out > (1 << 16):
                return ("ERR", "value-did-not-frame", None)
            continue
        if tag != 0x01:
            return ("ERR", "unknown-type", None)
        if p + 2 >= len(b):
            return ("ERR", "truncated", None)
        ty, w = b[p + 1], b[p + 2]
        if ty == 0x05:
            return ("ERR", "floating-point", None)
        if ty not in OLD_TYPES:
            return ("ERR", "unknown-type", None)
        if w not in OLD_WIDTHS:
            return ("ERR", "unknown-width", None)
        v = read_value(b, p + 3, w)
        if v is None:
            return ("ERR", "value-did-not-frame", None)
        out += w
        p = v[1]
        if out > (1 << 16):
            return ("ERR", "value-did-not-frame", None)


def anchor_scan_old(b):
    """`ininit.rs::in_scalar_initializers` AT MASTER `e928ee7`, keeping the
    stream OFFSET of every record it frames."""
    framed = []      # (offset, token, nsym)
    residue = 0
    i = 0
    n = len(b)
    while i + 1 < n:
        a1 = b[i] == 0x00 and b[i + 1] == 0x01
        a2 = b[i] == 0x00 and b[i + 1] == SYM
        if not a1 and not a2:
            i += 1
            continue
        matched = False
        for w in (4, 2):
            if i < w:
                continue
            t = read_token_var(b, i - w)
            if t is None or t[1] != w:
                continue
            r = read_elements_old(b, i + 1)
            if r[0] == "ERR":
                if a2:
                    break            # the fail-closed arm
                residue += 1
                framed.append((i, t[0], 0, "RESIDUE:" + r[1]))
                i += 2
                matched = True
                break
            nbytes, refs, p = r
            if nbytes == 0:
                if a2:
                    break
                residue += 1
                framed.append((i, t[0], 0, "RESIDUE:empty"))
                i = p
                matched = True
                break
            framed.append((i, t[0], len(refs), "OK"))
            i = p
            matched = True
            break
        if not matched:
            i += 1
    return framed, residue


def sequential_offsets(b):
    """The byte offset of every record's `00`/i32c field under the sequential
    parse, so the two walks can be compared position by position."""
    out = {}
    p = 0
    n = len(b)
    try:
        while p < n:
            if p == n - 1 and b[p] == 0x07:
                break
            if b[p] not in si.REC_TAGS:
                break
            q = p + 1
            if b[p] == 0x07:
                q += 1
            owner, q = si.instream.var_u_be(b, q)
            fld = q
            _, q = si.i32c(b, q)
            elems = []
            while q < n and b[q] not in si.REC_TAGS:
                q = si.node(b, q, elems, collections.defaultdict(int))
            out[fld] = (owner, elems)
            p = q
    except (si.Desync, IndexError, ValueError, struct.error):
        pass
    return out


def one(row):
    src, entry = row[0], row[1]
    p = None
    for nm in os.listdir(entry):
        if nm.startswith("_CL_") and nm.endswith("in"):
            p = os.path.join(entry, nm)
    if p is None:
        return None
    b = open(p, "rb").read()
    framed, _res = anchor_scan_old(b)
    seq = sequential_offsets(b)
    extra = [f for f in framed if f[0] not in seq and f[3] == "OK" and f[2] > 0]
    extra_sym = sum(f[2] for f in extra)
    real_sym = sum(f[2] for f in framed if f[0] in seq and f[3] == "OK")
    return {"src": src, "framed": len(framed), "extra": len(extra),
            "extra_sym": extra_sym, "real_sym": real_sym,
            "sym": sum(f[2] for f in framed if f[3] == "OK"),
            "ex": [(f[0], f[1], f[2]) for f in extra[:4]]}


def main():
    idxp = sys.argv[1]
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 8
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    rows = [l.rstrip("\n").split("\t") for l in open(idxp)]
    if limit:
        rows = rows[:limit]
    tot = collections.Counter()
    hist = collections.Counter()
    worst = []
    with cf.ProcessPoolExecutor(max_workers=jobs) as pool:
        for r in pool.map(one, rows, chunksize=8):
            if r is None:
                continue
            tot["tus"] += 1
            for k in ("framed", "extra", "extra_sym", "real_sym", "sym"):
                tot[k] += r[k]
            hist[r["extra_sym"]] += 1
            if r["extra_sym"]:
                worst.append((r["extra_sym"], r["src"], r["ex"]))

    print("== THE PRE-w-inread ANCHOR SCAN, EMULATED, OVER %d TUs ==" % tot["tus"])
    print("  records it framed                                : %d" % tot["framed"])
    print("  tag-02 symbol addresses it read                  : %d" % tot["sym"])
    print("  ... in records the SEQUENTIAL parse also frames   : %d" % tot["real_sym"])
    print("  ... in records the sequential parse does NOT      : %d  <== THE EXTRA"
          % tot["extra_sym"])
    print("  records it framed that the sequential parse does not: %d" % tot["extra"])
    print()
    print("== THE PER-TU DISTRIBUTION OF THE EXTRA ==")
    for k, v in sorted(hist.items()):
        print("    extra symbol addresses = %-4d : %4d TUs" % (k, v))
    print()
    worst.sort(reverse=True)
    print("== SAMPLE: the stream offsets the anchor scan re-anchored at ==")
    for n, src, ex in worst[:6]:
        print("  %-52s extra=%d" % (src[:52], n))
        for off, tok, nsym in ex:
            print("      offset %-8d token %04x  symbol addresses %d" % (off, tok, nsym))


if __name__ == "__main__":
    main()
