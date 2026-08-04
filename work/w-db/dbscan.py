#!/usr/bin/env python3
"""dbscan.py — READ THE `db` SUB-STREAM.  T1..T5 of the prereg.

`db` is sub-stream ordinal 4 (`push 0x4`, slot `[module+0x280]`), read at
`0x10be7f41` (`mov edx,0x10b1335c ; call 0x10b7e276`) inside the per-module loop
at `0x10be7ef5`, which is gated on `[module+0xcd8] & 0x2000` and feeds
`0x10be997b` / `0x10be9892`.  The container writer emits it only when
`ds:0x10c40ef8 & 0x2000` or `ds:0x10c40ecc != 0` (`0x10b73bb7`/`0x10b73bd3`).

This script does NOT assume what it holds.  It walks it as a CodeView
`<len:u16><leaf:u16>` record stream, publishes the leaf histogram by name, and
counts how many DEFINED symbol names (`D_all`) and emitted function names (`E`)
occur in it as NUL-terminated or length-prefixed strings.

Every failure prints a count.  stdlib only.  Reads no c2 output but the truth
it is being compared against, which is labelled as such.

    usage: dbscan.py <cacheidx.tsv> <dtruth-dir> [jobs]
"""
import json, os, sys, collections
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))

# CV type leaves (tpi), a subset named for the histogram; anything else is
# reported by its raw value so an unknown leaf can never read as absence.
LEAF = {0x1001:"LF_MODIFIER",0x1002:"LF_POINTER",0x1008:"LF_PROCEDURE",
        0x1009:"LF_MFUNCTION",0x1201:"LF_ARGLIST",0x1203:"LF_FIELDLIST",
        0x1400:"LF_BCLASS",0x1401:"LF_VBCLASS",0x1402:"LF_IVBCLASS",
        0x1409:"LF_VFUNCTAB",0x140c:"LF_ONEMETHOD",0x140d:"LF_NESTTYPE",
        0x1502:"LF_ENUM",0x1503:"LF_ARRAY",0x1504:"LF_CLASS",
        0x1505:"LF_STRUCTURE",0x1506:"LF_UNION",0x1507:"LF_ENUMERATE",
        0x150d:"LF_MEMBER",0x150e:"LF_STMEMBER",0x150f:"LF_METHOD",
        0x1510:"LF_NESTTYPE",0x1511:"LF_VFUNCTAB",0x1512:"LF_FRIENDFCN",
        0x1601:"LF_FUNC_ID",0x1602:"LF_MFUNC_ID",0x1603:"LF_BUILDINFO",
        0x1605:"LF_STRING_ID",0x1606:"LF_UDT_SRC_LINE"}


def slug(s): return s.replace("/", "__").replace("\\", "__")


def base_of(e):
    for n in os.listdir(e):
        if n.startswith("_CL_") and n.endswith("gl"):
            return os.path.join(e, n[:-2])
    return None


def strings_in(b, minlen=3):
    """every printable NUL-terminated run, the widest reading."""
    out, cur = set(), bytearray()
    for c in b:
        if 32 <= c < 127:
            cur.append(c)
        else:
            if len(cur) >= minlen:
                out.add(bytes(cur).decode("latin1"))
            cur = bytearray()
    if len(cur) >= minlen:
        out.add(bytes(cur).decode("latin1"))
    return out


def one(row, dtruth):
    src, entry = row[0], row[1]
    r = {"src": src, "ok": 0}
    base = base_of(entry)
    if base is None: return r
    p = base + "db"
    if not os.path.exists(p): return r
    b = open(p, "rb").read()
    r["ok"], r["size"] = 1, len(b)
    # walk the record stream as <len:u16><leaf:u16> from offset 2
    leaves = collections.Counter()
    off, walked, bad = 2, 0, 0
    while off + 4 <= len(b):
        ln = int.from_bytes(b[off:off+2], "little")
        if ln < 2 or off + 2 + ln > len(b): bad += 1; break
        leaf = int.from_bytes(b[off+2:off+4], "little")
        leaves[leaf] += 1
        off += 2 + ln; walked += 1
    r["walked"], r["walk_end"] = walked, off
    r["exact"] = 1 if off == len(b) else 0
    r["leaves"] = {hex(k): v for k, v in leaves.most_common(12)}
    r["n_known_leaf"] = sum(v for k, v in leaves.items() if k in LEAF)
    r["n_leaf"] = sum(leaves.values())
    T = json.load(open(os.path.join(dtruth, slug(src) + ".json")))
    S = strings_in(b)
    D, E = set(T["D_all"]), set(T["E"])
    Dd = set(T["D_data"])
    r["nD"], r["nE"], r["nDd"] = len(D), len(E), len(Dd)
    r["D_in_db"] = len(D & S); r["E_in_db"] = len(E & S); r["Dd_in_db"] = len(Dd & S)
    r["nstr"] = len(S)
    return r


def main():
    idxp, dtruth = sys.argv[1], sys.argv[2]
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 12
    rows = [l.rstrip("\n").split("\t") for l in open(idxp)]
    out = []
    with cf.ProcessPoolExecutor(max_workers=jobs) as ex:
        for r in ex.map(one, rows, [dtruth]*len(rows), chunksize=8):
            out.append(r)
    ok = [r for r in out if r["ok"]]
    print("T1  db present and non-empty: %d/%d  (empty: %d)"
          % (sum(1 for r in ok if r["size"]), len(rows),
             sum(1 for r in ok if not r["size"])))
    sizes = sorted(r["size"] for r in ok)
    print("T2  db bytes: median %d  min %d  max %d  total %d"
          % (sizes[len(sizes)//2], sizes[0], sizes[-1], sum(sizes)))
    nD = sum(r["nD"] for r in ok); nE = sum(r["nE"] for r in ok)
    nDd = sum(r["nDd"] for r in ok)
    print("T3  D_all names occurring as a string in db: %d/%d = %.5f"
          % (sum(r["D_in_db"] for r in ok), nD, sum(r["D_in_db"] for r in ok)/nD))
    print("    D_data names occurring as a string in db: %d/%d = %.5f"
          % (sum(r["Dd_in_db"] for r in ok), nDd, sum(r["Dd_in_db"] for r in ok)/nDd))
    print("T4  E names occurring as a string in db: %d/%d = %.5f"
          % (sum(r["E_in_db"] for r in ok), nE, sum(r["E_in_db"] for r in ok)/nE))
    print("T5  CV record walk: exact-consumption on %d/%d TUs ; records %d ; "
          "KNOWN leaves %d = %.5f"
          % (sum(r["exact"] for r in ok), len(ok), sum(r["n_leaf"] for r in ok),
             sum(r["n_known_leaf"] for r in ok),
             sum(r["n_known_leaf"] for r in ok)/max(1, sum(r["n_leaf"] for r in ok))))
    agg = collections.Counter()
    for r in ok:
        for k, v in r["leaves"].items(): agg[k] += v
    print("    leaf histogram (top 16):")
    for k, v in agg.most_common(16):
        print("      %-8s %-24s %d" % (k, LEAF.get(int(k, 16), "?"), v))
    print("    distinct strings in db, total %d" % sum(r["nstr"] for r in ok))
    with open(os.path.join(HERE, "dbscan.jsonl"), "w") as fh:
        for r in out: fh.write(json.dumps(r) + "\n")


if __name__ == "__main__":
    main()
