#!/usr/bin/env python3
"""CONFIRMATION 1 — read every `5C` out of a captured `.ex`, and VERIFY THE
STRUCTURE before any result is read off it.

    read_5c.py <file.ex> [...]

For each `LO`-anchored body it prints, per `5C` site, the bytes the CLAIM
(`5C <TYPE> <varint state>`) says the token is made of and the byte that follows
it — so the reader can check that the probe produced the structure the rung
claims rather than being told it did.

The walk here ASSUMES the claim, and that is fine on a probe of six functions
whose source is on the page: the non-circular measurement is `../scwalk.py`,
which never steps over a `5C`. This file is for reading a small capture out
loud.
"""

import glob
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "w-bd"))
from bdwalk import read_type, read_varint  # noqa: E402


def main():
    files = []
    for a in sys.argv[1:]:
        files.extend(sorted(glob.glob(a)))
    for f in files:
        b = open(f, "rb").read()
        n = len(b)
        print("== %s  (%d B) ==" % (f, n))
        # body starts, the tree's own `LO_MARKER` + `53`
        los, i = [], 0
        while True:
            j = b.find(b"\x4c\x4f\x11", i)
            if j < 0:
                break
            i = j + 3
            if j + 3 < n and b[j + 3] == 0x53:
                los.append(j + 3)
        los.append(n)
        for k in range(len(los) - 1):
            s, e = los[k], los[k + 1]
            sites = []
            p = s
            while p < e:
                if b[p] == 0x5C:
                    sites.append(p)
                p += 1
            print("  body %d  [%d..%d)  raw 5C bytes: %d" % (k, s, e, len(sites)))
            for p in sites:
                t = read_type(b, p + 1)
                if t is None:
                    print("     @%-6d 5c  <TYPE UNREADABLE>  %s" % (p, b[p : p + 8].hex(" ")))
                    continue
                q = p + 1 + t[3]
                v = read_varint(b, q)
                if v is None:
                    print("     @%-6d 5c %s  <STATE UNREADABLE>"
                          % (p, b[p + 1 : q].hex(" ")))
                    continue
                val, nxt = v
                esc = "ESCAPED" if b[q] == 0x80 else ""
                print(
                    "     @%-6d 5c  TYPE %-14s (%d B)  state %-6s = %-6d %-7s  -> next %02x %s"
                    % (
                        p,
                        b[p + 1 : q].hex(" "),
                        t[3],
                        b[q:nxt].hex(),
                        val,
                        esc,
                        b[nxt] if nxt < n else 0,
                        "(4B = STATEMENT END)" if nxt < n and b[nxt] == 0x4B else "",
                    )
                )


if __name__ == "__main__":
    main()
