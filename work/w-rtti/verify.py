#!/usr/bin/env python3
"""verify.py — check `docs/OBJ_RDATA_R_SHAPE.md` against FRESH captures.

    work/w-rtti/verify.py <obj-dir> [--detail]

Reads the COFF with `scripts/gt_dump.py`'s own `Obj` class and its relocation
table (which itself reads `crates/c2-obj/src/reloc.rs`), so nothing here is a
second copy of a table the project already owns.

Every claim is printed as `CLAIM <id> <held>/<checked> ...` — a **count**, never
a status, and a claim with `checked=0` prints `NO-CASE` rather than passing
(`docs/STATUS.md` traps 4 and 5).

Outside the std-only Rust workspace on purpose, same status as `gt_dump.py`.
"""

import os
import sys
import zlib

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "scripts"))
import gt_dump  # noqa: E402

CH_RDATA_R = 0x40301040  # CNT_INIT_DATA | ALIGN_4 | LNK_COMDAT | READ
CH_R0_DATA = 0xC0301040  # ...| WRITE

# The relocation type is checked by NAME through `gt_dump.reloc_name`, which
# reads `crates/c2-obj/src/reloc.rs`. Hardcoding a number here is how this file
# first "refuted" the spec 0/268: `IMAGE_REL_PPC_ADDR32` is **0x0002**, and
# 0x0006 (which it had) is i386's ADDR32 and PPC's REL24.


def be32(b, o):
    """The record fields are BIG-endian (spec §3). `gt_dump.u32` is the COFF
    container's little-endian reader and is the wrong one for record contents —
    reading `??_R3.numBaseClasses` with it gives 33554432 for 2."""
    return int.from_bytes(b[o:o + 4], "big")


def sel_of(obj, sec):
    """COMDAT selection + checksum from the section symbol's aux record.

    Found by NAME + section index, never by position (#644): the section symbol
    is the one whose storage class is STATIC(3) and whose `sec` is this section.
    """
    for s in obj.symbols:
        if s["sec"] == sec["idx"] and s["sc"] == 3 and s["naux"] >= 1:
            a = s["aux"][0]
            # Section aux: Length@0, NumberOfRelocations@4, NumberOfLinenumbers@6,
            # **CheckSum@8**, Number@12, Selection@14. Reading the checksum at +4
            # gives the relocation count, which is how this file first "refuted"
            # the CheckSum claim 0/174.
            return {"checksum": gt_dump.u32(a, 8), "selection": a[14], "sym": s}
    return None


def defining_sym(obj, sec):
    """The EXTERNAL symbol defined in this section (the COMDAT's name)."""
    for s in obj.symbols:
        if s["sec"] == sec["idx"] and s["sc"] == 2:
            return s
    return None


def coff_checksum(b):
    """The port's `coff/checksum.rs`: reflected CRC-32, poly 0xEDB88320,
    init 0, NO final inversion."""
    return (zlib.crc32(b) ^ 0xFFFFFFFF) ^ 0xFFFFFFFF if False else _crc0(b)


def _crc0(b):
    crc = 0
    for byte in b:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ (0xEDB88320 if crc & 1 else 0)
    return crc


def demangle_num(s, i):
    """MSVC number mangling: `A@`=0, `0`..`9`=1..10, `?`=negate,
    `A`..`P` nibbles terminated by `@`. Returns (value, next_index)."""
    neg = False
    if i < len(s) and s[i] == "?":
        neg = True
        i += 1
    if i < len(s) and s[i].isdigit():
        v = int(s[i]) + 1
        i += 1
    else:
        v = 0
        while i < len(s) and "A" <= s[i] <= "P":
            v = v * 16 + (ord(s[i]) - ord("A"))
            i += 1
        if i < len(s) and s[i] == "@":
            i += 1
        else:
            return None, i
    return (-v if neg else v), i


def r1_fields(name):
    """`??_R1<mdisp><pdisp><vdisp><attrs><class>@8` — the four numbers the spec
    says are spelled in the symbol's own name."""
    if not name.startswith("??_R1"):
        return None
    i = 5
    out = []
    for _ in range(4):
        v, i = demangle_num(name, i)
        if v is None:
            return None
        out.append(v)
    return out


def dfs_order(obj, rdata_r, r0_data, vft):
    """§5: DFS pre-order over the relocation graph, rooted at the vftables in
    forward base order, children in ascending relocation offset."""
    by_sym = {}
    for sec in rdata_r + r0_data:
        d = defining_sym(obj, sec)
        if d:
            by_sym[d["name"]] = sec
    children = {}
    for sec in rdata_r + r0_data + vft:
        d = defining_sym(obj, sec)
        if not d:
            continue
        kids = []
        for off, symidx, ty in sorted(obj.relocs(sec)):
            t = obj.sym_by_index(symidx)
            if t:
                kids.append(t["name"])
        children[d["name"]] = kids
    # roots: the vftables, in REVERSE section order (§6 says the vftable block
    # is internally reversed relative to the DFS root order).
    roots = []
    for sec in reversed(vft):
        d = defining_sym(obj, sec)
        if d:
            roots.append(d["name"])
    seen, order = set(), []
    stack = list(reversed(roots))
    while stack:
        n = stack.pop()
        if n in seen:
            continue
        seen.add(n)
        if n in by_sym:
            order.append(n)
        for k in reversed(children.get(n, [])):
            if k not in seen:
                stack.append(k)
    return order



def is_vtbl(obj, s):
    d = defining_sym(obj, s)
    return (
        s["name"] == ".rdata"
        and d is not None
        and (d["name"].startswith("??_7") or d["name"].startswith("??_8"))
    )


def is_record(obj, s):
    d = defining_sym(obj, s)
    if d is None:
        return False
    return s["name"] == ".rdata$r" or (
        s["name"] == ".data" and d["name"].startswith("??_R0")
    )


def groups(obj):
    """Walk the section table once and cut a new group at every VFTABLE BLOCK.

    Returns [(vftable-sections, rtti-record-sections)] per block.

    **The cut is the vftable run, NOT the `.text` COMDAT.** Cutting on `.text`
    is what `OBJ_RDATA_R_SHAPE.md` §6 describes ("immediately after the emitting
    function's `.text`") and it is wrong at `/Od` and `/Ox`, where
    `f11_three_classes` has **one** `.text` COMDAT and **three** vftable+RTTI
    blocks. The order of the blocks still follows the order the constructors are
    DEFINED in, so the key is the function, not the section it did or did not
    get.
    """
    out, cur_v, cur_r = [], [], []
    prev_v = False
    for s in obj.sections:
        if is_vtbl(obj, s):
            if not prev_v and (cur_v or cur_r):
                out.append((cur_v, cur_r))
                cur_v, cur_r = [], []
            cur_v.append(s)
            prev_v = True
            continue
        prev_v = False
        if is_record(obj, s):
            cur_r.append(s)
    if cur_v or cur_r:
        out.append((cur_v, cur_r))
    return out


def groups_partition(obj, rdata_r, r0):
    """Every RTTI record belongs to exactly one group, and no group is empty of
    vftables while holding records."""
    seen = []
    for v, r in groups(obj):
        if r and not v:
            return False
        seen.extend(r)
    return len(seen) == len(rdata_r) + len(r0)


def main(argv):
    objdir = argv[1]
    detail = "--detail" in argv
    names = sorted(f for f in os.listdir(objdir) if f.endswith(".obj"))

    tally = {}

    def claim(cid, ok, note=""):
        h, c = tally.setdefault(cid, [0, 0])
        tally[cid] = [h + (1 if ok else 0), c + 1]
        if not ok and note:
            print("  MISS %-22s %s" % (cid, note))

    per_obj = []
    for fn in names:
        with open(os.path.join(objdir, fn), "rb") as f:
            obj = gt_dump.Obj(f.read())
        rdata_r = [s for s in obj.sections if s["name"] == ".rdata$r"]
        r0 = []
        vft = []
        for s in obj.sections:
            d = defining_sym(obj, s)
            if not d:
                continue
            if s["name"] == ".data" and d["name"].startswith("??_R0"):
                r0.append(s)
            if s["name"] == ".rdata" and (
                d["name"].startswith("??_7") or d["name"].startswith("??_8")
            ):
                vft.append(s)
        per_obj.append((fn, len(obj.sections), len(rdata_r), len(r0), len(vft)))

        # ---- P5: characteristics + selection + relocation kind --------------
        for s in rdata_r:
            a = sel_of(obj, s)
            claim(
                "P5-rdata$r-chars",
                s["chars"] == CH_RDATA_R,
                "%s %s chars=0x%08x" % (fn, defining_sym(obj, s)["name"], s["chars"]),
            )
            claim(
                "P5-rdata$r-sel2",
                a is not None and a["selection"] == 2,
                "%s %s sel=%s" % (fn, defining_sym(obj, s)["name"], a and a["selection"]),
            )
            for off, si, ty in obj.relocs(s):
                claim("P5-reloc-ADDR32", gt_dump.reloc_name(ty) == "ADDR32",
                      "%s %s ty=0x%02x" % (fn, gt_dump.reloc_name(ty), ty))
        for s in r0:
            claim(
                "P8-R0-chars",
                s["chars"] == CH_R0_DATA,
                "%s %s chars=0x%08x" % (fn, defining_sym(obj, s)["name"], s["chars"]),
            )
            a = sel_of(obj, s)
            claim("P8-R0-sel2", a is not None and a["selection"] == 2,
                  "%s sel=%s" % (fn, a and a["selection"]))
            for off, si, ty in obj.relocs(s):
                claim("P5-reloc-ADDR32", gt_dump.reloc_name(ty) == "ADDR32",
                      "%s R0 %s ty=0x%02x" % (fn, gt_dump.reloc_name(ty), ty))

        # ---- P7: the aux CheckSum ------------------------------------------
        for s in rdata_r + r0:
            a = sel_of(obj, s)
            raw = obj.raw(s)
            claim(
                "P7-checksum",
                a is not None and a["checksum"] == _crc0(raw),
                "%s %s aux=0x%08x crc=0x%08x"
                % (fn, defining_sym(obj, s)["name"], a["checksum"], _crc0(raw)),
            )

        # ---- P8: ??_R0 is 8 + strlen(name) + 1, unpadded --------------------
        for s in r0:
            d = defining_sym(obj, s)
            mid = d["name"][len("??_R0"):]
            if mid.endswith("@8"):
                mid = mid[: -len("@8")]
            want = 8 + len("." + mid) + 1
            claim(
                "P8-R0-size",
                s["rawsize"] == want,
                "%s %s size=%d want=%d" % (fn, d["name"], s["rawsize"], want),
            )
            raw = obj.raw(s)
            claim(
                "P8-R0-name",
                raw[8:].rstrip(b"\0").decode("latin1") == "." + mid,
                "%s %s name=%r" % (fn, d["name"], raw[8:]),
            )
            claim("P8-R0-spare0", be32(raw, 4) == 0, "%s %s spare" % (fn, d["name"]))

        # ---- §3 record sizes and the two-readings-of-one-number check -------
        r3_counts, r2_sizes = {}, {}
        for s in rdata_r:
            d = defining_sym(obj, s)
            n = d["name"]
            raw = obj.raw(s)
            if n.startswith("??_R4"):
                claim("SZ-R4-20", s["rawsize"] == 20, "%s %s %d" % (fn, n, s["rawsize"]))
            elif n.startswith("??_R3"):
                claim("SZ-R3-16", s["rawsize"] == 16, "%s %s %d" % (fn, n, s["rawsize"]))
                r3_counts[n[len("??_R3"):]] = be32(raw, 8)
            elif n.startswith("??_R2"):
                r2_sizes[n[len("??_R2"):]] = s["rawsize"]
                claim(
                    "R2-null-terminated",
                    be32(raw, s["rawsize"] - 4) == 0,
                    "%s %s" % (fn, n),
                )
            elif n.startswith("??_R1"):
                claim("SZ-R1-28", s["rawsize"] == 28, "%s %s %d" % (fn, n, s["rawsize"]))
                f = r1_fields(n)
                ok = f is not None and (
                    f[0] == be32(raw, 8)
                    and (f[1] & 0xFFFFFFFF) == be32(raw, 12)
                    and f[2] == be32(raw, 16)
                    and f[3] == be32(raw, 20)
                )
                claim(
                    "R1-fields-in-name",
                    ok,
                    "%s %s name=%s bytes=%s"
                    % (
                        fn,
                        n,
                        f,
                        [be32(raw, o) for o in (8, 12, 16, 20)],
                    ),
                )
        for k, n in r3_counts.items():
            if k in r2_sizes:
                claim(
                    "R3count-eq-R2size",
                    r2_sizes[k] == 4 * (n + 1),
                    "%s %s n=%d r2=%d" % (fn, k, n, r2_sizes[k]),
                )

        # ---- P6: the DFS pre-order, PER EMITTING FUNCTION -------------------
        #
        # §6 says two independent classes in one TU do not interleave: each
        # function's `.text`, vftable block and complete RTTI graph precede the
        # next function's. So the DFS roots are the vftables of ONE group, not
        # all the vftables in the obj. A single global walk rooted at every
        # vftable in reverse section order is exact on every single-function
        # cell here and wrong on both multi-function ones.
        for g_vft, g_rec in groups(obj):
            if not g_rec:
                continue
            want = dfs_order(obj, [s for s in g_rec if s["name"] == ".rdata$r"],
                             [s for s in g_rec if s["name"] == ".data"], g_vft)
            got = [defining_sym(obj, s)["name"] for s in g_rec]
            claim("P6-DFS-order", want == got,
                  "%s\n     want %s\n     got  %s" % (fn, want, got))
        claim("P6-groups-dont-interleave", groups_partition(obj, rdata_r, r0),
              "%s" % fn)

        # ---- §6: the vftable COMDAT ----------------------------------------
        for s in vft:
            a = sel_of(obj, s)
            d = defining_sym(obj, s)
            # `??_7` (vftable) and `??_8` (vbtable) are NOT the same shape, and
            # OBJ_RDATA_R_SHAPE.md §3/§6 states one rule for both. Measured
            # separately here because the fresh grid separates them.
            kind = "VFT7" if d["name"].startswith("??_7") else "VBT8"
            claim(kind + "-sel6", a is not None and a["selection"] == 6,
                  "%s %s sel=%s" % (fn, d["name"], a and a["selection"]))
            claim(kind + "-value4", d["value"] == 4,
                  "%s %s value=%d" % (fn, d["name"], d["value"]))
            claim(kind + "-chars", s["chars"] == CH_RDATA_R,
                  "%s %s chars=0x%08x" % (fn, d["name"], s["chars"]))

    print()
    print("%-30s %6s %9s %6s %6s" % ("obj", "nsec", ".rdata$r", "??_R0", "vftbl"))
    for fn, nsec, nr, nr0, nv in per_obj:
        print("%-30s %6d %9d %6d %6d" % (fn, nsec, nr, nr0, nv))
    print()
    total_r = sum(p[2] for p in per_obj)
    total_r0 = sum(p[3] for p in per_obj)
    print("objs=%d  .rdata$r-sections=%d  ??_R0-data-comdats=%d  records=%d"
          % (len(per_obj), total_r, total_r0, total_r + total_r0))
    print()
    bad = 0
    for cid in sorted(tally):
        h, c = tally[cid]
        state = "HELD" if h == c else "REFUTED"
        if c == 0:
            state = "NO-CASE"
        if h != c:
            bad += 1
        print("CLAIM %-22s %-8s %d/%d" % (cid, state, h, c))
    print()
    print("claims=%d refuted=%d" % (len(tally), bad))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
