#!/usr/bin/env python3
"""rivals.py — the ATTR byte under RIVAL escape widths, over the same 99
escaped-SIZE records.

The shipped decode says the `0x80` escape is 3 bytes (`80 <LE16>`), so ATTR sits
at `q+3`.  The rivals are the widths a reader could plausibly have guessed:

    w=1   `0x80` is a plain byte (no escape at all)      -> ATTR at q+1
    w=2   `80 <one payload byte>`                        -> ATTR at q+2
    w=3   `80 <LE16>`   THE SHIPPED DECODE               -> ATTR at q+3
    w=5   `80 <LE32>` — SRCPOS's escape, the wrong reader-> ATTR at q+5

Scored against the ATTR vocabulary the 28,739 DIRECT records establish
independently: a width that lands outside it is reading an unrelated byte.

Also reports the background rate — what fraction of ALL bytes in these files
happen to be in that vocabulary — so "in the vocabulary" is priced rather than
assumed impressive.
"""

import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from glrec import framed_incumbent, symbol_runs, MAX_NAME_TO_OFFSET  # noqa: E402


def records_with_rivals(gl):
    runs = symbol_runs(gl)
    n = len(gl)
    p = 0
    while p + 5 <= n:
        if not framed_incumbent(gl, p):
            p += 1
            continue
        k = None
        for idx in range(len(runs) - 1, -1, -1):
            if runs[idx][1] <= p:
                k = idx
                break
        if k is None or p - runs[k][1] > MAX_NAME_TO_OFFSET:
            return
        q = p + 5
        if q >= n:
            return
        b = gl[q]
        if b < 0x80:
            q += 1
        elif b == 0x80:
            if q + 5 > n:
                return
            q += 5
        else:
            return
        if q >= n:
            return
        sb = gl[q]
        if sb == 0x80:
            yield ("escape", {w: (gl[q + w] if q + w < n else None) for w in (1, 2, 3, 5)})
            q += 3
        elif sb < 0x80:
            yield ("direct", {1: gl[q + 1] if q + 1 < n else None})
            q += 1
        else:
            return
        p += 5


def main(argv):
    d = argv[1]
    direct_vocab = collections.Counter()
    esc = []
    allbytes = collections.Counter()
    for fn in sorted(os.listdir(d)):
        if not fn.endswith(".gl"):
            continue
        gl = open(os.path.join(d, fn), "rb").read()
        allbytes.update(gl)
        for kind, m in records_with_rivals(gl):
            if kind == "direct":
                direct_vocab[m[1]] += 1
            else:
                esc.append(m)
    vocab = set(direct_vocab)
    tot = sum(allbytes.values())
    bg = sum(v for k, v in allbytes.items() if k in vocab) / tot
    print(f"DIRECT records: {sum(direct_vocab.values())}, ATTR vocabulary size {len(vocab)}")
    print("  " + " ".join(f"{k:02x}" for k in sorted(vocab)))
    print(f"BACKGROUND: {bg:.3%} of all .gl bytes are in that vocabulary")
    print(f"\nESCAPED records: {len(esc)}")
    print(f"{'width':>6} {'in-vocab':>9} {'of':>5}  {'p(all by chance)':>18}  distinct values")
    for w in (1, 2, 3, 5):
        vals = [m[w] for m in esc]
        good = sum(1 for v in vals if v in vocab)
        dist = collections.Counter(vals)
        p = bg ** len(vals)
        print(
            f"{w:>6} {good:>9} {len(vals):>5}  {p:>18.3e}  "
            + " ".join(f"{('%02x' % k) if k is not None else 'None'}:{v}" for k, v in dist.most_common(8))
        )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
