#!/usr/bin/env python3
"""Read a COFF obj's CodeView streams and print each function's TYPE INDEX.

The cross-check for AB-g: `.gl`'s pinned `80 <LE32>` field is claimed to be the
signature's CodeView type index. `.debug$S`'s `S_GPROC32` / `S_LPROC32` record
carries that index for the same function by name, and `.debug$T` says what the
index denotes. If the two agree, the claim is a measurement; if they do not,
the field is a c1xx-internal number that merely behaves like one.

Usage: cvtypes.py <obj>            # procedures, by name, with their type index
       cvtypes.py <obj> --types    # also dump the .debug$T record index -> kind
"""
import struct, sys

S_GPROC32, S_LPROC32 = 0x1110, 0x110F
LEAF = {
    0x1201: "LF_ARGLIST",
    0x1008: "LF_PROCEDURE",
    0x1009: "LF_MFUNCTION",
    0x1203: "LF_FIELDLIST",
    0x1505: "LF_STRUCTURE",
    0x1506: "LF_UNION",
    0x1507: "LF_ENUM",
    0x1002: "LF_POINTER",
    0x1503: "LF_ARRAY",
    0x1601: "LF_FUNC_ID",
    0x1602: "LF_MFUNC_ID",
    0x1603: "LF_BUILDINFO",
    0x1605: "LF_STRING_ID",
    0x1606: "LF_UDT_SRC_LINE",
    0x150d: "LF_MEMBER",
    0x1511: "LF_ONEMETHOD",
}


def sections(b):
    if b[:2] == b"MZ":
        raise SystemExit("not an obj")
    nsec, symp, nsym = struct.unpack_from("<H", b, 2)[0], *struct.unpack_from("<II", b, 8)
    strtab_off = symp + 18 * nsym
    out = []
    for i in range(nsec):
        o = 20 + 40 * i
        raw = b[o : o + 8]
        if raw[:1] == b"/":
            idx = int(raw[1:].split(b"\0")[0].decode())
            e = b.index(b"\0", strtab_off + idx)
            name = b[strtab_off + idx : e].decode()
        else:
            name = raw.split(b"\0")[0].decode()
        size, ptr = struct.unpack_from("<II", b, o + 16)
        out.append((name, b[ptr : ptr + size] if ptr else b""))
    return out


def subsections(data):
    """`.debug$S`: 4-byte signature then (kind,len,payload) subsections."""
    if len(data) < 4:
        return
    p = 4
    while p + 8 <= len(data):
        kind, ln = struct.unpack_from("<II", data, p)
        p += 8
        yield kind, data[p : p + ln]
        p += ln
        p = (p + 3) & ~3


def procs(obj):
    out = []
    for name, data in sections(obj):
        if not name.startswith(".debug$S"):
            continue
        for kind, payload in subsections(data):
            if kind != 0xF1:  # DEBUG_S_SYMBOLS
                continue
            p = 0
            while p + 4 <= len(payload):
                ln, rec = struct.unpack_from("<HH", payload, p)
                body = payload[p + 4 : p + 2 + ln]
                if rec in (S_GPROC32, S_LPROC32) and len(body) >= 35:
                    typind = struct.unpack_from("<I", body, 24)[0]
                    nm = body[35:].split(b"\0")[0].decode(errors="replace")
                    out.append((nm, typind, "S_GPROC32" if rec == S_GPROC32 else "S_LPROC32"))
                p += 2 + ln
    return out


def types(obj):
    out = []
    for name, data in sections(obj):
        if not name.startswith(".debug$T"):
            continue
        p, idx = 4, 0x1000
        while p + 4 <= len(data):
            ln, leaf = struct.unpack_from("<HH", data, p)
            out.append((idx, leaf, LEAF.get(leaf, hex(leaf)), data[p + 4 : p + 2 + ln]))
            idx += 1
            p += 2 + ln
    return out


if __name__ == "__main__":
    obj = open(sys.argv[1], "rb").read()
    print("SECTIONS:", ", ".join(n for n, _ in sections(obj)))
    for nm, ti, kind in procs(obj):
        print(f"  {kind}  typind={ti:#06x}  {nm}")
    if "--types" in sys.argv:
        for idx, leaf, nm, body in types(obj):
            extra = ""
            if leaf == 0x1008 and len(body) >= 12:  # LF_PROCEDURE
                rv, cc, nparm, alist = struct.unpack_from("<IBxHI", body, 0)
                extra = f" rvtype={rv:#06x} nparm={nparm} arglist={alist:#06x}"
            print(f"  {idx:#06x}  {nm}{extra}")
