#!/usr/bin/env python3
"""**Must-fail mutations on the SHIPPING emitters**, graded by real `c2.dll`.

A guard nobody has seen fail is not known to work. Each mutation below changes
one emitted word — never a recognizer clause — and the positive fixture must go
from `match` to a live **`mismatch`**, not to a refusal: a mutation producing
`NotImplemented` would prove only that the class stopped accepting the body.

The four chosen are the four claims this lane could most plausibly have got
wrong, and two of them are about the SCHEDULE, which is the part of the
constructor that is transcribed rather than derived:

    M1  POP parks `this` in r10 instead of r11        (the scratch order)
    M2  the ctor's `rotlwi` is NOT hoisted above the
        member-init store                            (the schedule)
    M3  the ctor's `twi 6` moves one slot earlier,
        ahead of the `andc`                          (div_mod_leaf's own
                                                      MUTATION ANCHOR shape)
    M4  PUSH loads the member AFTER the guard's
        `bclr` is emitted -- i.e. the two stores of
        the run swap their emitted order             (the store order)

Every patch is applied to shipping source and REVERTED (board #1704). The tree
is checked clean at the end.
"""
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FL = REPO / "crates/c2-core/src/codegen/pool_free_list.rs"
CT = REPO / "crates/c2-core/src/codegen/pool_ctor_chain.rs"

MUT = {
    "M1": (
        FL,
        "            t.extend_from_slice(&encode_mr(R_S1, R_THIS));",
        "            t.extend_from_slice(&encode_mr(R_S2, R_THIS));",
    ),
    "M2": (
        CT,
        "    t.extend_from_slice(&encode_rlwinm(R_COUNT, R_TOTAL, 1, 0, 31));\n"
        "    t.extend_from_slice(&encode_stw(R_PTR, R_THIS, off));",
        "    t.extend_from_slice(&encode_stw(R_PTR, R_THIS, off));\n"
        "    t.extend_from_slice(&encode_rlwinm(R_COUNT, R_TOTAL, 1, 0, 31));",
    ),
    "M3": (
        CT,
        "    t.extend_from_slice(&encode_andc(R_OVF, R_STRIDE, R_OVF));\n"
        "    t.extend_from_slice(&encode_twi(TO_DIV_BY_ZERO, R_STRIDE, 0));",
        "    t.extend_from_slice(&encode_twi(TO_DIV_BY_ZERO, R_STRIDE, 0));\n"
        "    t.extend_from_slice(&encode_andc(R_OVF, R_STRIDE, R_OVF));",
    ),
    "M4": (
        FL,
        "            t.extend_from_slice(&encode_stw(R_S1, r_v, 0));\n"
        "            t.extend_from_slice(&encode_stw(r_v, R_THIS, off));",
        "            t.extend_from_slice(&encode_stw(r_v, R_THIS, off));\n"
        "            t.extend_from_slice(&encode_stw(R_S1, r_v, 0));",
    ),
}


def run(cmd):
    return subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)


def lane():
    """One `/O1` mode lane over every fixture, graded by real `c2.dll`."""
    r = run(["sh", "scripts/mode_lane.sh", "/O1"])
    for line in r.stdout.splitlines():
        if line.startswith("LANE-RESULT"):
            return line
    return "NO LANE-RESULT\n" + r.stdout[-2000:] + r.stderr[-2000:]


def main():
    want = sys.argv[1:] or list(MUT)
    print("baseline:", lane())
    for name in want:
        path, old, new = MUT[name]
        orig = path.read_text()
        try:
            assert orig.count(old) == 1, f"{name}: anchor not unique"
            path.write_text(orig.replace(old, new, 1))
            got = lane()
        finally:
            path.write_text(orig)
        m = re.search(r"mismatch=(\d+)", got)
        n = int(m.group(1)) if m else -1
        print(f"{name}: {'FAIL (as required)' if n > 0 else 'DID NOT FAIL'}  {got}")
    print("restoring…", lane())
    st = run(["git", "status", "--porcelain", "--", "crates/c2-core"])
    print("final c2-core diff:", "EMPTY" if not st.stdout.strip() else st.stdout)


if __name__ == "__main__":
    main()
