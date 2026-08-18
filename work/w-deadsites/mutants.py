#!/usr/bin/env python3
"""`w-deadsites` — the named mutants, applied and reverted by exact text.

Separate from `sites.py` (which owns the 34-row reachability probe) because
these are `w-mutcensus`-style COLOUR mutations: each one is registered in the
prereg, applied to a clean tree, graded by a full suite run, and reverted.

    mutants.py list
    mutants.py apply <ID>
    mutants.py revert
"""

import os
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
CENSUS = "crates/c2-il/src/func/census.rs"
LEAF = "crates/c2-il/src/func/body/shapes/leaf_store.rs"
MOD = "crates/c2-il/src/func/body/mod.rs"
CALLS = "crates/c2-il/src/func/body/shapes/calls.rs"

MUTANTS = {
    # --- DATA_SYM_STRLIT_FENCED, one mutant per RAISE SITE (w-calleeguard F3) --
    # `w-mutcensus`' frame froze before `w-fence163` landed this key, so neither
    # site has ever been mutated. k = 2, and the two are input-distinguishable
    # from outside: the first is the PRE-parse `sym_fail` probe, the second the
    # POST-parse `Some(f)` gate, and no input reaches both.
    "MS1": (CENSUS,
            '                                                Some(n) if n.starts_with("??_C@") => {\n'
            "                                                    DATA_SYM_STRLIT_FENCED\n",
            '                                                Some(n) if n.starts_with("??_C@") => {\n'
            "                                                    DATA_SYM_LINKAGE\n"),
    "MS2": (CENSUS,
            "FnVerdict::Blocked(Block::at_end(seg, DATA_SYM_STRLIT_FENCED))",
            "FnVerdict::Blocked(Block::at_end(seg, DATA_SYM_LINKAGE))"),

    # --- the standing fence-site census, proven GREEN -> RED BY CONSTRUCTION ---
    # MC1 MOVES a site between two keys: `multi-producer` 2 -> 1 and
    # `mixed-kind` 1 -> 2. The TOTAL is unchanged at 22, so a census kept as one
    # integer — which is the shape `w-mutcensus` F4 literally asked for — cannot
    # see it. The per-key table must.
    "MC1": (LEAF,
            "    if (!lits.is_empty() || !addrs.is_empty()) && 3 + params.len() > POOL_TOP {\n"
            "        return Err(STORE_RUN_BIND_MULTI_PRODUCER);\n",
            "    if (!lits.is_empty() || !addrs.is_empty()) && 3 + params.len() > POOL_TOP {\n"
            "        return Err(STORE_RUN_BIND_MIXED_KIND);\n"),
    # MC3 SWAPS ONE KEY AT ONE ARM — `w-mutcensus`' own `CS3`, a site that lane
    # measured GREEN, i.e. one the entire 1,666-test suite cannot fail on. It
    # moves `static-scan-loop-object-out-of-class` 1 -> 0 and
    # `store-run-call-no-emitter-carrier` 1 -> 2 while the TOTAL stays at 24, so
    # a census kept as one integer — the shape F4 literally asked for — is blind
    # to it, and the per-key table is the SOLE thing in the suite that can fail.
    "MC3": (CENSUS,
            '"static-scan-loop" => STATIC_SCAN_LOOP_OBJECT,',
            '"static-scan-loop" => STORE_RUN_CALL_NO_CARRIER,'),
    # MC2 renames the CONSTANT and every use of it, leaving the published key
    # string alone. Nothing observable moved, so the census must stay GREEN —
    # this is the half of `w-guards`' rule that a counting test can get wrong in
    # the other direction, by pinning a name instead of a key.
    "MC2": ("RENAME", "STORE_RUN_BIND_GROUP_SHAPE", "STORE_RUN_BIND_GROUP_SHAPE_RENAMED"),
    # MC3 moves the KEY STRING while leaving the constant alone. A scan's
    # published vocabulary changes; the census must go RED.
    "MC4": (MOD,
            'pub(crate) const STORE_RUN_BIND_GROUP_SHAPE: &str = "store-run-bind-group-shape";',
            'pub(crate) const STORE_RUN_BIND_GROUP_SHAPE: &str = "store-run-bind-group-shape-v2";'),
    # MC5 adds a `refuse("…")` literal-key site — E1's population, which the
    # per-key table is blind to by construction and the third test covers.
    "MC5": (CALLS,
            "    if has_repeated_leaf(&arg_ops) {\n"
            '        return Err(refuse("call-arg-repeated-leaf"));\n'
            "    }\n",
            "    if has_repeated_leaf(&arg_ops) {\n"
            '        return Err(refuse("call-arg-repeated-leaf"));\n'
            "    }\n"
            "    if arg_ops.is_empty() {\n"
            '        return Err(refuse("call-arg-empty-probe"));\n'
            "    }\n"),

    # --- the named control (docs/rungs/README.md probe rule 1) ---------------
    "C1": (CALLS,
           "if syms > 1 && !two_sym_thunk {",
           "if syms > 2 && !two_sym_thunk {"),
}

TOUCHED = [CENSUS, LEAF, MOD, CALLS]


def rename_all(old, new):
    """Rename an identifier across every `crates/c2-il/src` file."""
    n = 0
    for dirpath, _, files in os.walk(os.path.join(ROOT, "crates/c2-il/src")):
        for f in files:
            if not f.endswith(".rs"):
                continue
            p = os.path.join(dirpath, f)
            with open(p, encoding="utf-8") as fh:
                t = fh.read()
            if old not in t:
                continue
            n += t.count(old)
            with open(p, "w", encoding="utf-8") as fh:
                fh.write(t.replace(old, new))
    return n


def apply(mid):
    dirty = subprocess.run(["git", "-C", ROOT, "status", "--porcelain", "--", "crates"],
                           capture_output=True, text=True).stdout.strip()
    if dirty:
        sys.exit("REFUSING: crates/ is dirty:\n" + dirty)
    spec = MUTANTS[mid]
    if spec[0] == "RENAME":
        n = rename_all(spec[1], spec[2])
        print(f"{mid}: renamed {spec[1]} -> {spec[2]} at {n} occurrences")
        return
    path, old, new = spec
    p = os.path.join(ROOT, path)
    with open(p, encoding="utf-8") as fh:
        t = fh.read()
    if t.count(old) != 1:
        sys.exit(f"REFUSING: {mid} matched {t.count(old)} times in {path}")
    with open(p, "w", encoding="utf-8") as fh:
        fh.write(t.replace(old, new))
    print(f"{mid}: applied in {path}")


def revert():
    subprocess.run(["git", "-C", ROOT, "checkout", "--", "crates/c2-il"], check=True)
    left = subprocess.run(["git", "-C", ROOT, "status", "--porcelain", "--", "crates/c2-il"],
                          capture_output=True, text=True).stdout.strip()
    print("reverted; crates/c2-il status:", left if left else "CLEAN")


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "list"
    if cmd == "apply":
        apply(sys.argv[2])
    elif cmd == "revert":
        revert()
    else:
        for k in MUTANTS:
            print(k)
