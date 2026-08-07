#!/usr/bin/env python3
"""insert_tests.py — splice this lane's tests into `leaf_store.rs`'s test module.

The two pinned `const`s are filled from the CAPTURED `.ex` segments by
`segconst.py`, so the bytes in the test are `c1xx`'s and not this lane's reading
of the grammar. Run once; the result is what is committed.
"""
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
TARGET = os.path.join(
    ROOT, "crates", "c2-il", "src", "func", "body", "shapes", "leaf_store.rs"
)


def seg(cell, name):
    out = subprocess.check_output(
        [sys.executable, os.path.join(HERE, "segconst.py"), cell, name],
        text=True,
    )
    # Drop the wrapper lines segconst prints; keep only the byte rows.
    return "\n".join(l for l in out.splitlines() if l.strip().startswith("0x"))


def main():
    frag = open(os.path.join(HERE, "tests.rs.frag")).read()
    frag = frag.replace(
        "    const BIND_XBOXHEAP_SHIPPED: &[u8] = &[\n    ];",
        "    const BIND_XBOXHEAP_SHIPPED: &[u8] = &[\n%s\n    ];"
        % seg("b_target_bind", "X"),
    )
    frag = frag.replace(
        "    const BIND_LEAF: &[u8] = &[\n    ];",
        "    const BIND_LEAF: &[u8] = &[\n%s\n    ];" % seg("b_leaf_bind", "X"),
    )
    src = open(TARGET).read()
    if "BIND_XBOXHEAP_SHIPPED" in src:
        raise SystemExit("already spliced")
    # The file's LAST `}` closes `mod tests`.
    i = src.rstrip().rfind("}")
    out = src[:i] + frag + "\n" + src[i:]
    open(TARGET, "w").write(out)
    print("spliced %d lines" % frag.count("\n"))


if __name__ == "__main__":
    main()
