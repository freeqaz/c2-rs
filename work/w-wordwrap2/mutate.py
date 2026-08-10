#!/usr/bin/env python3
"""w-wordwrap2 — apply ONE must-fail mutation, or revert it.

Each mutation DELETES one conjunction of the shipped class and nothing else. A
cell that still refuses after the deletion has graded nothing — the repair for
that is MERGING the clauses into one mutation, not counting it (#2665, #2698).

    mutate.py apply M1     mutate.py revert
"""
import subprocess
import sys

W = "crates/c2-core/src/coff/writer.rs"
B = "crates/c2-il/src/func/bundle.rs"

MUTATIONS = {
    # THE CONTROL, and it is RUN rather than asserted: restore `w-wordwrap`'s
    # `return None` on an uninitialized def and both accepted cells must fall
    # back to `codegen-gap`. If they did not, they would be accepted by some
    # OTHER path and this lane's production would be crediting itself with an
    # obj it does not emit.
    "M0": (
        W,
        """            if d.uninitialized {
                if d.lo_offs.is_empty() {
                    return None;
                }
                continue;
            }""",
        """            if d.uninitialized {
                return None;
            }""",
    ),
    # Rule S1' slot C, as ONE conjunction over two crates. The first run deleted
    # only the READER's linkage clause and came back GREEN, because the WRITER
    # re-asserts it — "neither crate assumes the other ran" is a real property of
    # the code and it is also an over-fence for a mutation aimed at one half.
    # #2665's shape: the two clauses are one conjunction over this cell, so the
    # deletion has to delete both. The failed first run is recorded beside this.
    "M1": [
        (
            B,
            "if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 || o.size == 0 || !o.external {",
            "if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 || o.size == 0 {",
        ),
        (
            W,
            "if o.bytes.is_some() || !o.external || o.size == 0 || !o.relocs.is_empty() {",
            "if o.bytes.is_some() || o.size == 0 || !o.relocs.is_empty() {",
        ),
    ],
    # Board #184's object-count bound.
    "M2": (
        W,
        "if bss.len() > super::data::MAX_OBJECTS_PER_SECTION {",
        "if bss.len() > 8 {",
    ),
    # Rule Y1's external clause — the SYMBOL group is the reverse of the walk.
    # Deleting the reversal makes the symbol order the storage order, which is
    # right at n = 1 and wrong at n = 2.
    "M3": (
        W,
        "let bss_symbol_order: Vec<usize> = (0..bss.len()).rev().collect();",
        "let bss_symbol_order: Vec<usize> = (0..bss.len()).collect();",
    ),
    # Rule B1 — the section nibble is the MAX over the objects.
    #
    # **The first aim was `&bss_refs[..1]` and it came back GREEN**, and the
    # reason is a property of the cell rather than of the clause: this TU's `.gl`
    # record order puts the 8-byte object FIRST, so "the first object's nibble"
    # and "the max" are the same number here. Re-aimed at the LAST object, where
    # they are 3 and 4. A mutation that coincides with the rule on the only cell
    # that can grade it has graded nothing, which is what the first run was.
    "M4": (
        W,
        "let nibble = super::data::section_nibble(&bss_refs)?;",
        "let nibble = super::data::section_nibble(&bss_refs[bss_refs.len() - 1..])?;",
    ),
    # Rule S1' slot B — BETWEEN the watermarks. Index 4 is after the second one,
    # which is where a `.data` goes (cell p6) and where a `.bss` never does.
    "M5": (
        W,
        """        sections.insert(
            3,""",
        """        sections.insert(
            4,""",
    ),
    # The panic this lane shipped for one commit, restored as ONE conjunction
    # over two crates — the second merged mutation, and for the same reason M1
    # is merged. Re-gating the writer's dangling-def test alone came back GREEN
    # because the READER's separated refusal now catches the cell first; the two
    # repairs are one conjunction over `wwbss_static_neg.cpp`.
    #
    # The must-fail signal here is a PANIC, not a `mismatch`, and that is the
    # honest grading: `every relocation target got a symbol` is what the defect
    # produced, and a crash is strictly louder than a wrong obj.
    "M6": [
        (
            W,
            """    for f in funcs {
        for d in &f.data_defs {
            if d.uninitialized && !bss.iter().any(|o| o.symbol == d.symbol) {
                return None;
            }
        }
    }""",
            """    for f in funcs {
        for d in &f.data_defs {
            if !bss.is_empty() && d.uninitialized && !bss.iter().any(|o| o.symbol == d.symbol) {
                return None;
            }
        }
    }""",
        ),
        (
            B,
            """        if bad_uninit || (other_section && !out.is_empty()) {
            return None;
        }
        Some(out)""",
            """        if out.is_empty() {
            return Some(Vec::new());
        }
        if bad_uninit || other_section {
            return None;
        }
        Some(out)""",
        ),
    ],
    # The thread-local clause.
    "M7": (
        B,
        "if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 || o.size == 0 || !o.external {",
        "if o.size == 0 || !o.external {",
    ),
}


def revert():
    subprocess.run(["git", "checkout", "--", W, B], check=True)


def apply(name):
    revert()
    edits = MUTATIONS[name]
    if isinstance(edits, tuple):
        edits = [edits]
    for path, old, new in edits:
        s = open(path).read()
        n = s.count(old)
        if n != 1:
            raise SystemExit(f"{name}: anchor found {n} times in {path} — not a unique deletion")
        open(path, "w").write(s.replace(old, new))
        print(f"{name}: applied to {path}")


if sys.argv[1] == "apply":
    apply(sys.argv[2])
else:
    revert()
    print("reverted")
