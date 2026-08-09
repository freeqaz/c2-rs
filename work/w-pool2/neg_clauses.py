#!/usr/bin/env python3
"""**Per-cell clause counterfactuals for `wpool2_free_list_neg.cpp`.**

The census key does NOT discriminate these cells and this instrument exists
because of that. All seven refuse, but four report `expr-op-0x27` and two
`expr-brtrue` — the fall-through keys board #1101/#1416 describe, naming where
the *generic* walk stopped after the whole-body production declined, not the
clause that declined it. Reading the cells off their keys would be exactly
`w-biquad` #2535's defect (seven of eleven cells confounded, caught by running
the probe rather than by reading it).

So each cell is graded by MUTATING THE ONE SHIPPING CLAUSE it is written for,
rebuilding, and re-censusing: the cell must move IN CLASS and the others must
not. Two of the seven are *swaps* rather than relaxations — the mutation makes
the negative acceptable and the POSITIVE fixture refuse — which is strictly
stronger, because a relaxation that admitted everything would also pass a
"the cell moved" test.

Every patch is applied to shipping source and REVERTED (board #1704: a probe
patch left in the tree is not a probe). The tree is checked clean at the end.

    work/w-pool2/neg_clauses.py            # all seven
    work/w-pool2/neg_clauses.py N1 N5      # a subset
"""
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
IL = REPO / "crates/c2-il/src/func/body/shapes/pool_free_list.rs"
CT = REPO / "crates/c2-il/src/func/body/shapes/pool_ctor_chain.rs"
NEG = "fixtures/cpp/wpool2_free_list_neg.cpp"
POS = "fixtures/cpp/wpool2_free_list.cpp"
FLAGS = REPO / "work/w-pool2/flags_o1.txt"

# cell index in the `_neg` file -> (name, file, old, new)
CELLS = {
    "N1": (
        0,
        IL,
        "    if read_varint(seg, &mut p)? != 0 {\n        return None;\n    }",
        "    if read_varint(seg, &mut p)? != 0 {\n        // COUNTERFACTUAL: the literal-zero clause relaxed\n    }",
    ),
    "N2": (
        1,
        IL,
        None,  # a SWAP, handled by swap_push_stores()
        None,
    ),
    "N3": (
        2,
        IL,
        "    if eat_member_designator(seg, &mut p, this_tok)? != off {\n        return None;\n    }\n    if eat_ptr_load(seg, &mut p)? != v_tok {",
        "    if eat_member_designator(seg, &mut p, this_tok)?.wrapping_sub(off) == i32::MIN {\n        return None;\n    }\n    if eat_ptr_load(seg, &mut p)? != v_tok {",
    ),
    "N4": (
        3,
        IL,
        "    if params.len() != 2 {\n        return None;\n    }\n    let v_tok = params[1];",
        "    if params.len() < 2 {\n        return None;\n    }\n    let v_tok = params[1];",
    ),
    "N5": (
        4,
        CT,
        "    if eat_int_lit(seg, &mut p)? != 1 {\n        return None;\n    }\n    // `24` is `>`;",
        "    if eat_int_lit(seg, &mut p)? > 1 {\n        return None;\n    }\n    // `24` is `>`;",
    ),
    "N6": (
        5,
        CT,
        "const ALIGN: i32 = 4;",
        "const ALIGN: i32 = 8;",  # a SWAP
    ),
    "N7": (
        6,
        CT,
        "    if eat_int_lit(seg, &mut p)? != 1 {\n        return None;\n    }\n    if !eat_byte(seg, &mut p, 0x04)",
        "    if eat_int_lit(seg, &mut p)? != 4 && eat_int_lit_never(seg) {\n        return None;\n    }\n    if !eat_byte(seg, &mut p, 0x04)",
    ),
}

# The N7 patch needs a helper that is never true; simpler to relax outright.
CELLS["N7"] = (
    6,
    CT,
    "    if eat_int_lit(seg, &mut p)? != 1 {\n        return None;\n    }\n    if !eat_byte(seg, &mut p, 0x04)",
    "    if eat_int_lit(seg, &mut p)? == i32::MIN {\n        return None;\n    }\n    if !eat_byte(seg, &mut p, 0x04)",
)

BIND = "    let off = eat_member_designator(seg, &mut p, this_tok)?;"
CHECK = (
    "    if eat_member_designator(seg, &mut p, this_tok)? != off {\n"
    "        return None;\n    }"
)


def swap_push_stores(text):
    """Swap the two store templates in `parse_push`, so the reordered cell N2 is
    the one admitted and the shipped body is the one refused.

    The two blocks also trade the member offset's BINDING: whichever store runs
    first is the one that reads `off`, and the second is the one that checks it.
    Doing that as part of the swap is what keeps the counterfactual a swap --
    the class still requires the two designators to agree on one member, it just
    requires the run in the other order."""
    a0 = text.index("    // ---- `*(void**)v = this->m;`")
    b0 = text.index("    // ---- `this->m = (char*)v;`")
    c0 = text.index("    // ---- the epilogue,")
    first, second = text[a0:b0], text[b0:c0]
    assert first.count(BIND) == 1 and second.count(CHECK) == 1
    first = first.replace(BIND, CHECK, 1)
    second = second.replace(CHECK, BIND, 1)
    return text[:a0] + second + first + text[c0:]


def run(cmd, **kw):
    return subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, **kw)


def census(src):
    r = run([
        "./target/release/c2rs", "census", src,
        "--flags-file", str(FLAGS.relative_to(REPO)),
    ])
    out = []
    for line in r.stdout.splitlines():
        m = re.match(r"\s*\[\s*(\d+)\]\s+(ok|GAP)\s+(\S+)", line)
        if m:
            out.append((int(m.group(1)), m.group(2), m.group(3)))
    return out


def build():
    r = run(["cargo", "build", "--release", "-p", "c2-harness"])
    if r.returncode != 0:
        print(r.stderr[-3000:])
        raise SystemExit("build failed")


def main():
    want = sys.argv[1:] or list(CELLS)
    build()
    base_neg = census(NEG)
    base_pos = census(POS)
    assert len(base_neg) == 7, base_neg
    assert all(v == "GAP" for _, v, _ in base_neg), base_neg
    assert all(v == "ok" for _, v, _ in base_pos), base_pos
    print(f"baseline: neg 7/7 GAP, pos {len(base_pos)}/{len(base_pos)} ok\n")

    results = {}
    for name in want:
        idx, path, old, new = CELLS[name]
        orig = path.read_text()
        try:
            if name == "N2":
                path.write_text(swap_push_stores(orig))
            else:
                assert orig.count(old) == 1, f"{name}: anchor not unique"
                path.write_text(orig.replace(old, new, 1))
            build()
            neg, pos = census(NEG), census(POS)
        finally:
            path.write_text(orig)
        moved = [i for i, (_, v, _) in enumerate(neg) if v == "ok"]
        pos_lost = [i for i, (_, v, _) in enumerate(pos) if v != "ok"]
        ok = moved == [idx]
        results[name] = {
            "cell": idx,
            "moved_in_class": moved,
            "positive_functions_lost": pos_lost,
            "verdict": "EXACT" if ok else "IMPRECISE",
        }
        print(f"{name}: cell {idx} — moved {moved} — positive lost {pos_lost} "
              f"— {results[name]['verdict']}")
    build()
    st = run(["git", "status", "--porcelain", "--", "crates/"])
    print("\nfinal crates/ diff:", "EMPTY" if not st.stdout.strip() else st.stdout)
    (REPO / "work/w-pool2/NEG_CLAUSES.json").write_text(json.dumps(results, indent=1))


if __name__ == "__main__":
    main()
