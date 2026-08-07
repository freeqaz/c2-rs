#!/usr/bin/env python3
"""w-front3 — the LIFT HATCHES, applied to a SCRATCH TREE and NEVER COMMITTED.

    python3 work/w-front3/hatch.py apply     insert the hatches
    python3 work/w-front3/hatch.py revert    `git checkout` every touched file
    python3 work/w-front3/hatch.py check     report applied / not applied

`w-mrslot/ladder.sh` is the prototype: it lifted ONE clause behind an
uncommitted env hatch, re-ran, and read what the TU reported next. That is the
only method on this project that produces a MEASURED rung rather than an
inferred one — a first-blocker key is a NAME, not a DISTANCE, so the only way to
learn a ladder's length is to climb it.

This generalises the hatch to every clause on the FRONTIER's seventeen that a
one-line lift can reach, plus the committed `C2RS_SINK_CHAIN` family for the
expression layer.

# The discipline, which is what makes this a measurement and not a widening

* **Nothing here is ever committed.** `work/w-front3/ladder.py` re-applies it.
  `git diff --stat -- crates/` must be EMPTY at the end of the lane, and the
  rung doc shows it measured.
* **A hatched run's DIFFERENTIAL VERDICT is never quoted, in either
  direction.** Unlike the committed `C2RS_SINK_*` sinks, these hatches have **no
  poison** — that is deliberate, because a poisoned lift cannot reach
  `select_function` and the CODEGEN column is exactly what a poisoned lift
  cannot see. The price is that a hatched tree CAN emit, so:
    - a `Port=Match` under a hatch is **not** evidence of anything;
    - a `Port=Mismatch` under a hatch is **not** an alarm;
    - only `fn_blockers`, `emit_blockers` and `fn_gate_refusals` are read.
* **`assign-rhs-call` is deliberately ABSENT and its absence is a result.** It
  is a `return Err` whose alternative branch does not exist — the production is
  unimplemented, not guarded — so there is nothing to lift and the row it blocks
  gets a BOUND, never a LIFTED. A hatch that "lifted" it by inventing a parse
  would manufacture a fictitious successor key, which is the one way this
  instrument could lie (the `chain_skip_form` header states the same rule for
  widths).
* **`call-arg-outer-formal` is expected to PANIC downstream** —
  `permute_args_text` indexed `permutation_cycles`' `seen` array out of bounds,
  which is why the guard exists. A panic is a loud, distinguishable outcome and
  is reported as one.
"""

import subprocess
import sys
import os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

HELPER = '''
// ---------------------------------------------------------------------------
// UNCOMMITTED — lane w-front3's MEASUREMENT HATCH (work/w-front3/hatch.py).
// Reads `W_FRONT3_LIFT`, a comma-separated list of clause names to disable so
// the ladder BELOW each one can be read. Never committed; `hatch.py revert`
// removes it. Absent variable => every clause behaves exactly as shipped.
// ---------------------------------------------------------------------------
pub(crate) fn front3_lift(name: &str) -> bool {
    static ON: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    ON.get_or_init(|| {
        std::env::var("W_FRONT3_LIFT")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
    .iter()
    .any(|s| s == name)
}
'''

# (file, needle, replacement). Every needle is matched EXACTLY ONCE or the
# applier is a hard error — a silently unapplied hatch reads as "the clause was
# not the binding one", which is the failure this whole lane exists to stop.
EDITS = [
    # --- the helper -------------------------------------------------------
    ("crates/c2-il/src/lib.rs",
     "pub mod codec;",
     HELPER + "\npub mod codec;"),

    # --- H:param-width  (src/Main.cpp's round-0 key) ----------------------
    # `Formals::Undetermined` means the `.sy` did not bind the parameter list.
    # The lift is the branch the tree ALREADY has under `#[cfg(test)]`:
    # `AllOneRegisterByConstruction` — assume every formal is one GPR.
    ("crates/c2-il/src/func/sy.rs",
     "        if formals.len() <= 1 {\n            return Ok(());\n        }\n        let declared = match self.formals {",
     "        if formals.len() <= 1 {\n            return Ok(());\n        }\n        if crate::front3_lift(\"param-width\") {\n            return Ok(());\n        }\n        let declared = match self.formals {"),
    ("crates/c2-il/src/func/sy.rs",
     "        if formals.is_empty() {\n            return Ok(Vec::new());\n        }\n        let declared = match self.formals {",
     "        if formals.is_empty() {\n            return Ok(Vec::new());\n        }\n        if crate::front3_lift(\"param-width\") {\n            return Ok(vec![ArgClass::Gpr; formals.len()]);\n        }\n        let declared = match self.formals {"),

    # --- H:assign-store-type  (negate_test, keygen_xbox) ------------------
    # The store's TYPE is not int-like. The lift eats whatever type IS there,
    # by the tree's own `read_type` width — never a guessed width.
    ("crates/c2-il/src/func/body/shapes/assign.rs",
     "        if !eat_int_like(seg, &mut p) {\n            return Err(blk_type(seg, p, p, \"assign-store-type\"));\n        }",
     "        if !eat_int_like(seg, &mut p) {\n            match (crate::front3_lift(\"assign-store-type\"), crate::func::readers::read_type(seg, p)) {\n                (true, Some((_, _, _, w))) => p += w,\n                _ => return Err(blk_type(seg, p, p, \"assign-store-type\")),\n            }\n        }"),

    # --- H:call-arg-lit-permuted  (vsnprnc) -------------------------------
    ("crates/c2-il/src/func/body/shapes/calls.rs",
     "    if !in_place && !one_moved_at_two {\n        return Err(refuse(\"call-arg-lit-permuted\"));\n    }",
     "    if !in_place && !one_moved_at_two && !crate::front3_lift(\"call-arg-lit-permuted\") {\n        return Err(refuse(\"call-arg-lit-permuted\"));\n    }"),

    # --- H:call-arg-outer-formal  (keygen_xbox) ---------------------------
    ("crates/c2-il/src/func/body/shapes/calls.rs",
     "        if arg_sources.iter().any(|&ix| ix >= arg_sources.len()) {\n            return Err(refuse(\"call-arg-outer-formal\"));\n        }",
     "        if arg_sources.iter().any(|&ix| ix >= arg_sources.len())\n            && !crate::front3_lift(\"call-arg-outer-formal\")\n        {\n            return Err(refuse(\"call-arg-outer-formal\"));\n        }"),

    # --- H:expr-shr-mixed-sign  (osfinfo, jsonwriter) ---------------------
    # The guard refuses `>>` whose operand classes disagree, because `sraw` and
    # `srw` are different instructions. The lift picks `ShrS` ARBITRARILY: it
    # exists to move the cursor, and the verdict it produces is discarded.
    ("crates/c2-il/src/func/body/expr.rs",
     "                    (true, true) => {\n                        return Err(Block::refuse(seg, *p, \"expr-shr-mixed-sign\"))\n                    }",
     "                    (true, true) => {\n                        if crate::front3_lift(\"expr-shr-mixed-sign\") {\n                            IlOp::ShrS\n                        } else {\n                            return Err(Block::refuse(seg, *p, \"expr-shr-mixed-sign\"));\n                        }\n                    }"),

    # --- H:store-run-bind-mixed-kind  (xboxheap) --------------------------
    # `w-mrslot`'s own hatch, re-derived here rather than re-read, so its §5 is
    # reproduced by this lane's instrument and not carried.
    ("crates/c2-il/src/func/body/shapes/leaf_store.rs",
     "    if addr_producer {\n        return Err(if lits.is_empty() {",
     "    if addr_producer && !crate::front3_lift(\"store-run-bind-mixed-kind\") {\n        return Err(if lits.is_empty() {"),
]

FILES = sorted({f for f, _, _ in EDITS})


def apply():
    for path, needle, repl in EDITS:
        full = os.path.join(ROOT, path)
        src = open(full).read()
        if repl in src:
            continue
        n = src.count(needle)
        if n != 1:
            raise SystemExit("HATCH FAILED: %s: needle matched %d times, want 1\n%r"
                             % (path, n, needle[:120]))
        open(full, "w").write(src.replace(needle, repl))
    print("hatch: APPLIED to %d files" % len(FILES))


def revert():
    subprocess.run(["git", "checkout", "--"] + FILES, cwd=ROOT, check=True)
    print("hatch: REVERTED (%d files)" % len(FILES))


def check():
    dirty = subprocess.run(["git", "diff", "--stat", "--", "crates/"],
                           cwd=ROOT, capture_output=True, text=True).stdout
    print(dirty if dirty.strip() else "crates/ diff: EMPTY")


if __name__ == "__main__":
    {"apply": apply, "revert": revert, "check": check}[sys.argv[1]]()
