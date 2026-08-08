#!/usr/bin/env python3
"""THE FLOOR UNDER THE FLOOR — is `0x4C`, the CALL-END, payload-free?

Pinning `0xBD` moved every one of the nine ladders that terminated on it onto
`expr-chain-noform-0x4C`. So `0x4C` is the binding floor now, and this asks the
same two questions of it that `bdwalk.py` asked of `0xBD`.

    cewalk.py <il-root>

## The claim under test

`4C` closes a call's argument region and carries NOTHING —
`control_flow.rs`'s `0x4C` arm is `s.p += 1`, `mcall::eat_call_args_region`'s is
`*p += 1; return true`, and `codec.rs`'s `ExToken` model gives `4C` width 1
(its `4C 4B` VoidCallEnd is `4C` + the separate statement-end `4B`). Call it
**P** (payload-free). The rivals:

    B1  4C <one raw byte>
    T   4C <TYPE>
    K   4C <token>

## The anchor, and why it is EXACT rather than heuristic

A raw `4C` byte scan hits data. This anchors instead on the ONE position where a
`4C` is certain: the byte immediately after a `BD` token read at the width
`bdwalk.py` just confirmed over 3.5 M sites. Those are zero-argument calls —
`26 <callee> BD <TYPE> <flags> <id> 4C` — and the `4C` is the call-end by
construction, not by guess.

## How a reading is judged

Same predicate as `bdwalk.py`: the landing byte must open an operand token
(`LEGAL_OPEN`, taken from the tree's own `control_flow.rs` vocabulary). Plus one
test the `0xBD` walk did not need and that is the decisive one for a
payload-free claim, the one `w-divsplit` used at all 4,674 division sites:
**is there anywhere for a payload to BE?** `4C <TYPE>` requires bit 7 of the
next byte to be set. If the next byte's bit 7 is clear at essentially every
site, there is no room for a TYPE, and the same argument bounds `4C <token>`.
"""

import collections
import glob
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bdwalk import (  # noqa: E402
    LEGAL_OPEN,
    read_token_var,
    read_type,
    read_varint,
)


def bd_end(b, p):
    """The confirmed `BD <TYPE> <1 byte> <varint>` width."""
    t = read_type(b, p + 1)
    if t is None:
        return None
    q = p + 1 + t[3] + 1
    v = read_varint(b, q)
    return None if v is None else v[1]


def main(root):
    tus = sites = 0
    nxt = collections.Counter()
    land = {"P": collections.Counter(), "B1": collections.Counter()}
    desync = collections.Counter()
    room_for_type = 0
    room_for_token = 0
    short = 0

    for d in sorted(glob.glob(os.path.join(root, "*"))):
        exs = glob.glob(os.path.join(d, "*.ex"))
        if not exs:
            continue
        tus += 1
        b = open(exs[0], "rb").read()
        n = len(b)
        i = 0
        while True:
            j = b.find(b"\xbd", i)
            if j < 0:
                break
            i = j + 1
            ok = False
            for tw in (2, 4):
                k = j - 1 - tw
                if k >= 0 and b[k] == 0x26:
                    r = read_token_var(b, k + 1)
                    if r is not None and r[1] == tw and k + 1 + tw == j:
                        ok = True
                        break
            if not ok:
                continue
            e = bd_end(b, j)
            # A ZERO-ARGUMENT call: the byte the CALL token ends on is the `4C`.
            if e is None or e >= n or b[e] != 0x4C:
                continue
            sites += 1
            if e + 1 >= n:
                short += 1
                continue
            c = b[e + 1]
            nxt[c] += 1
            # P — payload-free: the next byte must open a token.
            land["P"][c] += 1
            if c not in LEGAL_OPEN:
                desync["P"] += 1
            # B1 — one raw byte of payload: the byte AFTER that must open one.
            if e + 2 < n:
                land["B1"][b[e + 2]] += 1
                if b[e + 2] not in LEGAL_OPEN:
                    desync["B1"] += 1
            # T — is there ROOM for a TYPE? Only if the next byte decodes as one.
            if read_type(b, e + 1) is not None:
                room_for_type += 1
            # K — is there room for a TOKEN? A token is 2 or 4 bytes and has no
            # tag of its own, so "room" is asked the only way it can be: does a
            # token read here land on a byte that opens an operand token?
            r = read_token_var(b, e + 1)
            if r is not None and e + 1 + r[1] < n and b[e + 1 + r[1]] in LEGAL_OPEN:
                room_for_token += 1

    print("=" * 72)
    print("w-bd — is 0x4C (CALL-END) payload-free? The floor under the 0xBD floor")
    print("=" * 72)
    print(f"TUs                                     {tus}")
    print(f"EXACT `4C` sites (a zero-arg call's end) {sites}")
    print(f"  of which at the very end of the stream {short}")
    print()
    print(f"-- P (payload-free): desync {desync['P']} / {sites} --")
    print(f"   top landings: " + ", ".join(f"0x{v:02X}x{c}" for v, c in land['P'].most_common(8)))
    print()
    print(f"-- B1 (one raw byte): desync {desync['B1']} / {sites} --")
    print(f"   top landings: " + ", ".join(f"0x{v:02X}x{c}" for v, c in land['B1'].most_common(8)))
    print()
    print("-- IS THERE ANYWHERE FOR A PAYLOAD TO BE? (w-divsplit's own question) --")
    hi = sum(c for v, c in nxt.items() if v & 0x80)
    print(f"   next byte has bit 7 SET (could be a TYPE tag)   {hi} / {sites}")
    print(f"   next byte decodes as a whole TYPE               {room_for_type} / {sites}")
    print(f"   a TOKEN read here lands on a legal opcode       {room_for_token} / {sites}")
    print()
    print("-- the byte after `4C`, in full (this is the successor distribution) --")
    for v, c in nxt.most_common(24):
        mark = "" if v in LEGAL_OPEN else "   <- opens nothing"
        print(f"   0x{v:02X}  {c:9d}{mark}")
    print(f"   ... {len(nxt)} distinct bytes in all")


if __name__ == "__main__":
    main(sys.argv[1])
