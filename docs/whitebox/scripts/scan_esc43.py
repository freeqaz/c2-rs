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
