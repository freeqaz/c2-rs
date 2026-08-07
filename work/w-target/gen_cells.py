#!/usr/bin/env python3
"""gen_cells.py — write GRID-W, the R-CLOSE boundary grid.

Lane w-target measurement tooling. **Read-only with respect to `crates/`.**

WHAT THE GRID ASKS
------------------
The 878-TU counterfactual says R-CLOSE* converts 158 of 861 and fires on **zero**
of the 3,803 relocating functions the judge credits today. That is a statement
about *this corpus*. GRID-W asks the boundary questions the corpus cannot:

    w01/w02/w03   does c2 close a chain of depth 1 / 2 / 3?
    w04*          **IS THERE A CHAIN c2 DOES NOT CLOSE?** — the cell that
                  decides the lane. If one exists and the port cannot read the
                  thing that stops c2, then `regress 0` on the workload is a
                  property of the workload and not of the rule.
    w05           a callee c2 did NOT inline — the ~6,176 control class in
                  miniature. The rule must leave it alone.
    w06           a cycle. Termination, not a verdict.
    w07           the chain's next link is EXTERNAL — the `seq|local->extern`
                  family's 16.
    w08           the next link has NO census row in this TU — w-splice's
                  `S6-chain-open`, which cost it 1 wrong relocation of 4 firings.

THE PER-CELL POSITIVE CONTROL
-----------------------------
Every cell carries `void anchor(){ ext_anchor(); }`, whose callee this TU does
NOT define, so R-CLOSE must NOT fire on it and its single REL24 must still name
`ext_anchor`. Without it, "the rule left this alone" cannot be told from "the
scan graded nothing" — `docs/STATUS.md` trap 5, and the failure `w-relo`'s C1/C2
pair exists to prevent: a rule that fires on everything passes the conversion
cells and fails this one; a rule that fires on nothing does the reverse.

Usage:  gen_cells.py <outdir>
"""

import hashlib
import os
import sys

ANCHOR = "void ext_anchor();\nvoid anchor() { ext_anchor(); }\n"

# (cell, source, what it grades)
CELLS = [
    (
        "w01_chain1",
        "void ext();\nvoid g() { ext(); }\nvoid f() { g(); }\n",
        "DEPTH 1 — c2 must name ext from ?f. Both bodies are the word "
        "48000000, so the RELOCATION is the entire verdict (#882)",
    ),
    (
        "w02_chain2",
        "void ext();\nvoid h() { ext(); }\nvoid g() { h(); }\nvoid f() { g(); }\n",
        "DEPTH 2 — the `chain2` family's 73. Does ?f name ext, or h?",
    ),
    (
        "w03_chain3",
        "void ext();\nvoid i() { ext(); }\nvoid h() { i(); }\n"
        "void g() { h(); }\nvoid f() { g(); }\n",
        "DEPTH 3 — does the closure keep going, or stop at 2? The workload "
        "has no depth-3 witness, so this is the only place it can be asked",
    ),
    (
        "w04a_noinline",
        "void ext();\n__declspec(noinline) void g() { ext(); }\nvoid f() { g(); }\n",
        "**THE CELL THAT DECIDES THE LANE** — c2 is told not to inline the "
        "intermediate. If c2 obeys, ?f names g and R-CLOSE names ext: a "
        "DEMONSTRATED WRONG EMIT unless the port can read the attribute",
    ),
    (
        "w04b_addr_taken",
        "void ext();\nvoid g() { ext(); }\nvoid (*gp)() = g;\nvoid f() { g(); }\n",
        "w04 variant — the intermediate's address is taken. c2 must still "
        "emit g standalone; does it still inline at the direct site?",
    ),
    (
        "w04c_virtual",
        "void ext();\nstruct S { virtual void g(); };\nvoid S::g() { ext(); }\n"
        "void f(S* s) { s->S::g(); }\n",
        "w04 variant — the intermediate is virtual, called non-virtually",
    ),
    (
        "w04d_optimize_off",
        "void ext();\n#pragma optimize(\"\", off)\nvoid g() { ext(); }\n"
        "#pragma optimize(\"\", on)\nvoid f() { g(); }\n",
        "w04 variant — the intermediate is compiled at a different optimize "
        "mode. `splice.rs` has S6-mode-mismatch for the body; this asks the "
        "same question of the TARGET",
    ),
    (
        "w05_control_not_inlined",
        "int gsink;\n"
        "int g(int a) { int t = 0; for (int i = 0; i < a; ++i) t += i * a; "
        "gsink = t; return t; }\n"
        "int f(int a) { return g(a); }\n",
        "THE CONTROL CLASS — a callee c2 does not inline and the port cannot "
        "lower. R-CLOSE must NOT fire: ?f keeps its REL24 against ?g",
    ),
    (
        "w06_cycle",
        "void f();\nvoid g() { f(); }\nvoid f() { g(); }\n",
        "TERMINATION — a two-cycle. The walk must refuse, not loop",
    ),
    (
        "w07_next_link_extern",
        "void ext();\nvoid g() { ext(); }\n"
        "int side;\nvoid f() { side = 1; g(); }\n",
        "the `seq|local->extern|chain1` family's 16 — a seq caller whose "
        "chain's next link leaves the TU",
    ),
    (
        "w08_chain_open",
        "void ext1();\nvoid ext2();\n"
        "void h() { ext1(); ext2(); }\nvoid g() { h(); }\nvoid f() { g(); }\n",
        "w-splice's S6-chain-open — the chain's end carries MORE THAN ONE "
        "call, so its target is not a single name. `close_target` must refuse "
        "with `callee-multi-call` rather than pick one",
    ),
    (
        "w09_leaf_no_call",
        "int g(int a) { return a + 1; }\nint f(int a) { return g(a); }\n",
        "CONTROL — the chain's end carries NO call at all. c2 inlines g into "
        "f and there is no relocation left to get wrong; `close_target` must "
        "refuse with `callee-no-call`",
    ),
]


def main():
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    stamp = hashlib.sha256()
    names = []
    for name, body, why in CELLS:
        text = "// GRID-W cell %s — %s\n%s\n%s" % (name, why, body, ANCHOR)
        p = os.path.join(out, name + ".cpp")
        with open(p, "w") as fh:
            fh.write(text)
        h = hashlib.sha256(text.encode()).hexdigest()
        stamp.update(name.encode())
        stamp.update(text.encode())
        names.append((p, h))
    print("cells: %d" % len(CELLS))
    print("GRID-W sha256: %s" % stamp.hexdigest())
    for p, h in names:
        print("  %s  %s" % (h, os.path.relpath(p)))


if __name__ == "__main__":
    main()
