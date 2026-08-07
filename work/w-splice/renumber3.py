#!/usr/bin/env python3
"""renumber3.py — the cross-reference sweep `renumber2.py`'s explicit list missed.

`renumber2.py` lists every site it edits by exact text, which is what keeps it
from corrupting another lane's citation — and the cost of that discipline is
that a site nobody enumerated is left behind. Four were: `#994` twice, `#991`
twice.

This is the safe half of the sweep: it runs **only over files whose every board
citation belongs to this lane** — the w-splice rung, `splice.rs`, `fnbytes.rs`'s
splice block, and this lane's own prereg. Shared pages (`STATUS.md`,
`INLINE_PREDICATE.md`, `FUNCTION_BYTE_MATCH.md`, `BOARD.md`) are NOT swept here,
because they carry other lanes' numbers in the same ranges — `INLINE_PREDICATE`
alone cites w-inl0's own `#990`-`#995` two paragraphs from this lane's.
Those stay on the explicit list.
"""

import re
import sys

MAP = {
    "990": "1021", "991": "1022", "992": "1023", "993": "1024",
    "994": "1025", "995": "1026",
    "1006": "1017", "1007": "1018", "1008": "1019", "1009": "1020",
}

# Every board citation in these files is this lane's own — checked by reading
# them, not assumed.
OWNED = [
    "docs/rungs/2026-08-08-w-splice.md",
    "crates/c2-core/src/splice.rs",
    "work/w-splice/PREREG.md",
]


def main():
    total = 0
    for p in OWNED:
        try:
            s = open(p).read()
        except FileNotFoundError:
            print("  (absent) %s" % p)
            continue
        n = 0

        def sub(m):
            nonlocal n
            k = m.group(2)
            if k in MAP:
                n += 1
                return m.group(1) + MAP[k] + m.group(3)
            return m.group(0)

        out = re.sub(r"(#|\*\*)(\d{3,4})(\*\*|\b)", sub, s)
        if n:
            open(p, "w").write(out)
        print("  %-44s %d reference(s)" % (p, n))
        total += n
    print("total: %d" % total)


if __name__ == "__main__":
    main()
