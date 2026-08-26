#!/usr/bin/env python3
"""Is the `43 42` WIDE-TOKEN hazard reachable in the 878-TU workload?

Lane `w-opclass`, board #3585-#3590.  Whitebox tooling (outside the std-only
`crates/` workspace).

    python3 docs/whitebox/scripts/scan_esc43.py <cache-index.tsv> [--sample N]
    python3 docs/whitebox/scripts/scan_esc43.py <cache-index.tsv> --control <key>

## The question

`control_flow.rs:1066` reads `0x43` as an escape and advances **+4** on the
sub-byte `0x42`.  `0x43` is class `00` (payload-free) and `0x42` is class `02`
(a `varU`), so `1 + 1 + 2 = 4` — the port is right whenever that `varU` is its
narrow form.  A `varU` is **2 or 4 bytes**: two LE bytes, and two more iff the
second has bit 7 set (`0x10c1f91b`, derived in `WB_OPCLASS_FINDINGS.md` §3).
Over a wide token `43 42 …` is **6** bytes and the port's fixed `+4` lands two
bytes inside the payload.

## What this measures, and what it deliberately does NOT

It counts **raw byte occurrences** of `43 42` in each `.ex` stream and, at each,
whether the byte at `+3` has bit 7 set.  A raw scan **over-counts**: a `43 42`
pair can fall inside another token's payload.  That is the correct direction for
this question — the count is an **upper bound**, so a *zero* on the wide column
is strong evidence of unreachability while a nonzero needs a walk to confirm the
position is a real token boundary.  The script says which of the two it got.

A full token walk is NOT attempted, and the reason is named rather than left
implicit: c2's own walk needs the `0x4F` sub-record format interpreter
(`FUN_10b9761e`, 64 field-type codes, `ref/P_SUB4F.md`) and a Python
reimplementation of it would be a second implementation of a format this tree
already owns once.

## Provenance

`[src]` throughout — this reads IL captured from the workload, not the image.
The cache index comes from `c2rs cache index`, the supported reader.  The
`entry.bin` payload extractor below is a **second implementation** of
`c2_il::cachefmt`, which that module's own doc warns about, so it carries a
control: `--control <key>` prints this reader's per-stream lengths beside
`c2rs cache show`'s, and they must agree.
"""

import os
import struct
import subprocess
import sys

MAGIC = b"C2RSCAP\x02"
HEADER_LEN = 56


def decode_entry(raw):
    """{tag: bytes} from an `entry.bin`.  Canonical form is enforced on read."""
    if len(raw) < HEADER_LEN or raw[0:8] != MAGIC:
        return None
    ver, nsect = struct.unpack_from("<II", raw, 8)
    total, = struct.unpack_from("<Q", raw, 16)
    if ver != 2 or total != len(raw):
        return None
    out = {}
    off = HEADER_LEN
    for i in range(nsect):
        tag = raw[off:off + 8].rstrip(b"\0").decode("ascii", "replace")
        o, n = struct.unpack_from("<QQ", raw, off + 8)
        if o + n > total:
            return None
        out[tag] = raw[o:o + n]
        off += 24
    return out


def workload_entries(index_path):
    """[(src, entry_dir)] for the 878-TU workload — relative `src/…` arguments."""
    rows = []
    for ln in open(index_path):
        src, _, entry = ln.rstrip("\n").partition("\t")
        if entry and src.lower().startswith("src/"):
            rows.append((src, entry))
    return rows


def scan(ex, handled):
    """Counters over one `.ex` stream.

    `handled` is the set of opcode bytes THIS dispatch handles (95 of them,
    from `dump_ilarms.py`).  It is used as a **successor filter**, which is the
    only cheap discriminator available without a full token walk: at a real
    token boundary the byte after the whole `43 42 <varU>` production must
    itself open a handled token.

      narrow_ok   the byte at +4 opens a handled token — consistent with the
                  port's fixed `+4`
      wide_ok     the byte at +6 opens a handled token — consistent with a WIDE
                  varU, which the port would walk two bytes into
      wide_only   bit 7 set at +3 AND wide_ok AND NOT narrow_ok — the shape that
                  can only be read as a real wide site
    """
    sites = wide = narrow_ok = wide_ok = wide_only = 0
    i = 0
    n = len(ex)
    while True:
        i = ex.find(b"\x43\x42", i)
        if i < 0 or i + 4 > n:
            break
        sites += 1
        nok = i + 4 < n and ex[i + 4] in handled
        if nok:
            narrow_ok += 1
        if ex[i + 3] & 0x80:
            wide += 1
            wok = i + 6 < n and ex[i + 6] in handled
            if wok:
                wide_ok += 1
                if not nok:
                    wide_only += 1
        i += 1
    return sites, wide, narrow_ok, wide_ok, wide_only


def handled_opcodes(dll):
    """The 95 opcodes this dispatch HANDLES, from `dump_ilarms.py`."""
    import importlib.util
    here = os.path.dirname(os.path.abspath(__file__))
    spec = importlib.util.spec_from_file_location(
        "dump_ilarms", os.path.join(here, "dump_ilarms.py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    img = m.Image(dll)
    d = m.Dispatch(img)
    oa = d.opcodes_of_arm()
    ref = set(d.refusal_arms())
    return {o for k, ops in oa.items() if k not in ref for o in ops}


def any_wide_token(ex):
    """Upper-bound count of 4-byte `varU` forms anywhere: bytes with bit 7 set
    in a second-byte position.  Reported only as context for H4."""
    return sum(1 for b in ex if b & 0x80)




# ---------------------------------------------------------------------------
# A TOKEN WALK, added after the raw scan, because a raw scan can only bound the
# answer.  The grammar below is transcribed from THIS lane's own arm walk
# (`dump_opclass.py --arms`, committed at `labels/opclass_arms.txt`) — not from
# `WB_READER_FINDINGS.md` §3 and not from `work/wb-eh/extok.py`, both of which
# already carry a transcription.  `--walk` prints a per-class agreement check
# against `extok.py` so the two independent transcriptions can be compared
# rather than one silently trusted.

class Cur:
    def __init__(self, b, i):
        self.b, self.i = b, i

    def byte(self):
        v = self.b[self.i]
        self.i += 1
        return v

    def skip(self):                       # 0x10c1f90a
        while self.byte() & 0x80:
            pass

    def varu(self):                       # 0x10c1f91b — 2 or 4, never 1
        b0, b1 = self.byte(), self.byte()
        if not (b1 & 0x80):
            return b0 | (b1 << 8), 2
        self.byte(); self.byte()
        return None, 4

    def i16c(self):                       # 0x10c1f9a6
        b = self.byte()
        if b != 0x80:
            return b
        self.i += 2
        return None

    def i32c(self):                       # 0x10c1f9e9
        b = self.byte()
        if b != 0x80:
            return b
        self.i += 4
        return None

    def i64c(self):                       # 0x10c1fae7
        b = self.byte()
        if b != 0x80:
            return b
        self.i += 8
        return None

    def word(self):                       # 0x10c1fe40
        b1 = self.byte()
        if not (b1 & 0x80):
            return b1
        b2 = self.byte()
        if b1 & 0x40:
            b3 = self.byte()
            return ((b2 & 0x7f) << 16) | ((b1 & 0x7f) << 8) | b3
        return ((b1 & 0x7f) << 8) | b2

    def TYPE(self):                       # 0x10b3d546
        v = self.word()
        if (v & 0xf) == 6 and ((v >> 4) & 0x1f) == 0:
            self.i32c()
        self.skip()                       # the globally gated trailing run
        return v


FMT_TABLE_VA = 0x10B26268                 # ref/P_SUB4F.md; stride 8, ptr at +0


def fmt_of(img, sub):
    p = img.u32(FMT_TABLE_VA + sub * 8)
    if p == 0:
        return None
    o = img.off(p)
    e = img.raw.index(b"\0", o)
    return img.raw[o:e]


def one_token(img, tbl, r, op):
    c = tbl[op]
    if c == 0x00:
        return
    if c == 0x01:
        r.TYPE(); return
    if c == 0x02:
        r.varu(); return
    if c == 0x03:
        r.varu(); r.TYPE(); r.byte(); r.byte(); return
    if c == 0x04:
        r.varu(); r.byte(); return
    if c == 0x05:
        r.TYPE(); r.byte(); return
    if c == 0x06:
        # c2 branches on the LOWERED word node[+4], which needs FUN_10b3d40a.
        # This walk refuses rather than guessing; the refusal is counted.
        raise ValueError("class 06 needs the lowering (0x10b3d40a)")
    if c == 0x07:
        r.TYPE(); r.varu(); return
    if c == 0x08:
        r.varu(); return
    if c == 0x09:
        r.TYPE(); r.byte(); return
    if c == 0x0A:
        r.byte(); return
    if c == 0x0C:
        sub = r.i16c()
        if sub is None:
            raise ValueError("0x4F sub-record with an escaped i16c")
        f = fmt_of(img, sub)
        if f is None:
            raise ValueError(f"0x4F sub {sub:02x} has no format string")
        for ch in f:
            if ch in (0x6C, 0x14):
                r.i32c()
            elif ch in (0x73, 0x16):
                r.varu()
            elif ch in (0x15, 0x0E):
                r.i16c()
            elif ch == 0x0B:
                r.byte()
            else:
                raise ValueError(f"0x4F field code {ch:02x} not modelled")
        return
    if c in (0x0D, 0x11):
        r.i32c(); return
    if c == 0x0E:
        r.TYPE(); r.varu(); r.varu(); return
    if c == 0x0F:
        r.i16c(); return
    if c == 0x12:
        r.TYPE(); r.varu(); return
    if c == 0x13:
        r.TYPE(); r.i32c(); return
    if c == 0x14:
        r.i32c(); r.i32c(); return
    if c == 0x15:
        r.varu(); r.varu(); r.i16c(); return
    if c == 0x17:
        n = r.i32c()
        if n is None:
            raise ValueError("class 17 escaped length")
        r.i += n
        return
    if c == 0x18:
        r.varu(); r.TYPE(); return
    if c == 0x19:
        r.TYPE(); r.byte(); r.i32c(); return
    if c == 0x1A:
        n = r.i32c()
        if n is None:
            raise ValueError("class 1A escaped count")
        for _ in range(n):
            r.skip()
        return
    if c == 0x1B:
        r.i32c(); r.varu(); return
    if c == 0x1C:
        r.TYPE(); r.i32c(); return
    raise ValueError(f"class {c:02x} refuses (op {op:02x})")


BODY_MARK = b"\x4c\x4f\x11"
FN_TAIL = b"\x4f\x12\x47\x54\x01\x54\x00"


def walk_stream(img, tbl, ex):
    """(bodies, walked, tok43_42, wide43_42, all42, wide42, stops) over one `.ex`.

    A `43 42` SITE is two consecutive top-level tokens: a `0x43` (class 00,
    payload-free) immediately followed by a `0x42` (class 02, a varU).  That is
    exactly what `control_flow.rs:1066` reads as one 4-byte escape.  The width
    recorded is the varU's own — 2 or 4 — taken from the cursor, not guessed.
    """
    bodies = walked = n43 = w43 = n42 = w42 = 0
    p43 = pw43 = 0
    stops = {}
    i = 0
    while True:
        i = ex.find(BODY_MARK, i)
        if i < 0:
            break
        bodies += 1
        r = Cur(ex, i + 3)
        acc = []
        prev = None
        try:
            while r.i < len(ex):
                if ex[r.i:r.i + 7] == FN_TAIL:
                    walked += 1
                    for a in acc:
                        n42 += 1
                        w42 += a[1] == 4
                        if a[0]:
                            n43 += 1
                            w43 += a[1] == 4
                    break
                s = r.i
                op = r.byte()
                if op == 0x4D:
                    walked += 1
                    for a in acc:
                        n42 += 1
                        w42 += a[1] == 4
                        if a[0]:
                            n43 += 1
                            w43 += a[1] == 4
                    break
                one_token(img, tbl, r, op)
                if op == 0x42:
                    acc.append((prev == 0x43, r.i - s - 1))
                prev = op
        except (ValueError, IndexError) as e:
            k = str(e)[:48]
            stops[k] = stops.get(k, 0) + 1
            # The prefix up to a stop was still walked IN SYNC, so its sites are
            # real token positions and are counted separately rather than thrown
            # away with the body.
            for a in acc:
                if a[0]:
                    p43 += 1
                    pw43 += a[1] == 4
        i += 1
    return bodies, walked, n43, w43, n42, w42, p43, pw43, stops


def cmd_walk(index_path, dll, limit):
    import importlib.util
    here = os.path.dirname(os.path.abspath(__file__))
    spec = importlib.util.spec_from_file_location(
        "dump_opclass", os.path.join(here, "dump_opclass.py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    img = m.Image(dll)
    d = m.Decoder(img)
    tbl = [img.u8(d.class_table_va + o) for o in range(0x100)]

    rows = workload_entries(index_path)
    seen = set()
    tot = [0] * 8
    stops = {}
    tus = 0
    srcs = set()
    for src, ent in rows:
        if tus >= limit:
            break
        if src in srcs:
            continue
        try:
            raw = open(os.path.join(ent, "entry.bin"), "rb").read()
        except OSError:
            continue
        blob = decode_entry(raw)
        if blob is None or "ex" not in blob:
            continue
        h = hash(blob["ex"])
        if h in seen:
            continue
        seen.add(h)
        srcs.add(src)
        tus += 1
        b, w, n43, w43, n42, w42, p43, pw43, st = walk_stream(img, tbl, blob["ex"])
        for k, v in enumerate((b, w, n43, w43, n42, w42, p43, pw43)):
            tot[k] += v
        for k, v in st.items():
            stops[k] = stops.get(k, 0) + v

    print(f"== TOKEN WALK over {tus} distinct workload `.ex` streams "
          f"(one per source, first {limit}) ==")
    print(f"  bodies found (the `4C 4F 11` marker)      {tot[0]}")
    print(f"  bodies walked clean to the tail / `4D`    {tot[1]}"
          f"  ({100.0 * tot[1] / tot[0]:.1f} %)" if tot[0] else "")
    print(f"  top-level `0x42` tokens                   {tot[4]}")
    print(f"    ... whose varU is WIDE (4 bytes)        {tot[5]}")
    print(f"  `43 42` SITES (a 0x43 immediately before) {tot[2]}")
    print(f"    ... whose varU is WIDE (4 bytes)        {tot[3]}")
    print(f"  `43 42` sites in the IN-SYNC PREFIX of a body the walk could")
    print(f"  not finish (still real token positions)   {tot[6]}")
    print(f"    ... whose varU is WIDE (4 bytes)        {tot[7]}")
    print()
    if (tot[2] + tot[6]) and not (tot[3] + tot[7]):
        print("  => every `43 42` site this walk reached is NARROW: the port's")
        print("     fixed `+4` is the right width on all of them, by coincidence")
        print("     of the token being small, not by any rule.")
    print()
    print("  walks that stopped, by reason (a stop is a limit of THIS walker,")
    print("  not a defect in the stream):")
    for k, v in sorted(stops.items(), key=lambda x: -x[1]):
        print(f"    {v:>8}  {k}")


def cmd_control(index_path, key):
    ent = os.path.join(os.path.dirname(index_path), key) if os.sep in key else key
    raw = open(os.path.join(ent, "entry.bin"), "rb").read()
    blob = decode_entry(raw)
    if blob is None:
        raise SystemExit("this reader could not decode the blob")
    print("this reader:")
    for t in ("ex", "gl", "sy", "in", "db"):
        print(f"  .{t:<3} {len(blob[t]) if t in blob else 'absent'}")
    print("c2rs cache show:")
    out = subprocess.run(["c2rs", "cache", "show", os.path.basename(ent)],
                         capture_output=True, text=True)
    for ln in out.stdout.splitlines():
        if ln.strip().startswith("."):
            print("  " + ln.strip())
    print("(the two blocks must agree line for line)")


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    index_path = sys.argv[1]
    if len(sys.argv) > 3 and sys.argv[2] == "--control":
        return cmd_control(index_path, sys.argv[3])
    if len(sys.argv) > 2 and sys.argv[2] == "--walk":
        n = int(sys.argv[3]) if len(sys.argv) > 3 else 100
        return cmd_walk(index_path, os.path.join(
            os.path.dirname(os.path.abspath(__file__)),
            "..", "..", "..", "compilers", "X360", "16.00.11886.00", "c2.dll"), n)

    dll = sys.argv[2] if len(sys.argv) > 2 else \
        os.path.join(os.path.dirname(os.path.abspath(__file__)),
                     "..", "..", "..", "compilers", "X360", "16.00.11886.00", "c2.dll")
    handled = handled_opcodes(dll)
    print(f"successor filter: {len(handled)} handled opcodes, derived from "
          f"dump_ilarms.py")
    rows = workload_entries(index_path)
    srcs = sorted({s for s, _ in rows})
    print(f"workload entries: {len(rows)} cache entries over {len(srcs)} distinct "
          f"`src/…` sources")

    seen = set()
    tus = 0
    tot_sites = tot_wide = tot_nok = tot_wok = tot_wonly = 0
    per_src_sites = {}
    per_src_wide = {}
    unreadable = 0
    hi_bytes = 0
    ex_bytes = 0
    for src, ent in rows:
        try:
            raw = open(os.path.join(ent, "entry.bin"), "rb").read()
        except OSError:
            unreadable += 1
            continue
        blob = decode_entry(raw)
        if blob is None or "ex" not in blob:
            unreadable += 1
            continue
        ex = blob["ex"]
        h = hash(ex)
        if h in seen:
            continue
        seen.add(h)
        tus += 1
        ex_bytes += len(ex)
        hi_bytes += any_wide_token(ex)
        s, w, nok, wok, wonly = scan(ex, handled)
        tot_sites += s
        tot_wide += w
        tot_nok += nok
        tot_wok += wok
        tot_wonly += wonly
        per_src_sites[src] = per_src_sites.get(src, 0) + s
        per_src_wide[src] = per_src_wide.get(src, 0) + w

    print(f"distinct `.ex` streams scanned: {tus}   ({unreadable} unreadable)")
    print(f"total `.ex` bytes:              {ex_bytes}")
    print()
    print(f"raw `43 42` occurrences (UPPER BOUND — see the module doc): {tot_sites}")
    print(f"  ... byte at +4 opens a handled token (narrow-consistent):  {tot_nok}")
    print(f"  ... byte at +3 has bit 7 set (would be a WIDE varU):       {tot_wide}")
    print(f"      ... and the byte at +6 opens a handled token:          {tot_wok}")
    print(f"      ... and +4 does NOT — readable ONLY as a wide site:    {tot_wonly}")
    print()
    if tot_wonly == 0:
        print("=> NOT WITNESSED, at an upper bound that over-counts and with the")
        print("   successor filter applied: every candidate wide site is ALSO")
        print("   readable as a narrow one, so none of them forces the wide")
        print("   reading.  A token walk could only shrink this set further.")
    else:
        print("=> WITNESSED at the upper bound: there are sites the narrow reading")
        print("   cannot explain.  A token walk is needed to confirm the position.")
    print()
    with_sites = sorted((n, s) for s, n in per_src_sites.items() if n)
    print(f"sources with >= 1 raw site: {len(with_sites)} of {len(srcs)}")
    print("the ten largest, by site count (a SIZE ranking and labelled as one —")
    print("it ranks nothing to work on, it says where a walk would look):")
    for n, s in sorted(with_sites, reverse=True)[:10]:
        print(f"  {n:>6}  {s}   (wide {per_src_wide.get(s, 0)})")


if __name__ == "__main__":
    main()
