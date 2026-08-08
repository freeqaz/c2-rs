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

# 2026-08-08, lane w-one — THIS APPLIER FAILED OPEN AND NOW FAILS CLOSED (#1322)

`apply()` used to write each edit as it walked `EDITS` and raise on the first
needle that did not match. `store-run-bind-mixed-kind` was **PAID** between
`503f8937` and `04727f37`, so its needle stopped matching — and because that edit
is **last**, the other seven were already on disk. The tree was left *partially
hatched*, `git diff -- crates/` was non-empty, and a run that read one red line
and kept going measured a tree nobody intended.

Three changes, all of them about the same thing — an instrument that degrades
must not degrade silently:

1. **Two phases.** Every needle is checked against the on-disk text **before a
   byte is written**. If any check fails, nothing is written at all and the
   process exits non-zero naming the edit's `id` and its file.
2. **Every edit has an `id`**, so the failure message says *which clause* is
   unavailable rather than printing 120 characters of Rust.
3. **A PAID edit is RETIRED explicitly, with a positive check.** `RETIRED` names
   the clause, the commit that paid it, and a `paid_witness` string that must be
   **present** in the file. An edit that simply stopped matching is a DRIFT and
   is a hard error; an edit whose clause was paid is a retirement and is
   *printed on every apply*, so the hatch set in force is never implicit.
   Retiring by deletion would have made the drift and the payment look identical.

`check` is likewise no longer "print the diff". It reports the applied count
against the expected count, and an all-or-nothing verdict — `APPLIED`, `CLEAN`
or `PARTIAL`, the last being a hard error.
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

# (id, file, needle, replacement). Every needle is matched EXACTLY ONCE or the
# applier is a hard error — a silently unapplied hatch reads as "the clause was
# not the binding one", which is the failure this whole lane exists to stop.
# `id` exists so that hard error can NAME the clause (lane w-one, board #1322).
EDITS = [
    # --- the helper -------------------------------------------------------
    ("helper",
     "crates/c2-il/src/lib.rs",
     "pub mod codec;",
     HELPER + "\npub mod codec;"),

    # --- H:param-width  (src/Main.cpp's round-0 key) ----------------------
    # `Formals::Undetermined` means the `.sy` did not bind the parameter list.
    # The lift is the branch the tree ALREADY has under `#[cfg(test)]`:
    # `AllOneRegisterByConstruction` — assume every formal is one GPR.
    ("param-width",
     "crates/c2-il/src/func/sy.rs",
     "        if formals.len() <= 1 {\n            return Ok(());\n        }\n        let declared = match self.formals {",
     "        if formals.len() <= 1 {\n            return Ok(());\n        }\n        if crate::front3_lift(\"param-width\") {\n            return Ok(());\n        }\n        let declared = match self.formals {"),
    ("param-width",
     "crates/c2-il/src/func/sy.rs",
     "        if formals.is_empty() {\n            return Ok(Vec::new());\n        }\n        let declared = match self.formals {",
     "        if formals.is_empty() {\n            return Ok(Vec::new());\n        }\n        if crate::front3_lift(\"param-width\") {\n            return Ok(vec![ArgClass::Gpr; formals.len()]);\n        }\n        let declared = match self.formals {"),

    # --- H:assign-store-type  (negate_test, keygen_xbox) ------------------
    # The store's TYPE is not int-like. The lift eats whatever type IS there,
    # by the tree's own `read_type` width — never a guessed width.
    ("assign-store-type",
     "crates/c2-il/src/func/body/shapes/assign.rs",
     "        if !eat_int_like(seg, &mut p) {\n            return Err(blk_type(seg, p, p, \"assign-store-type\"));\n        }",
     "        if !eat_int_like(seg, &mut p) {\n            match (crate::front3_lift(\"assign-store-type\"), crate::func::readers::read_type(seg, p)) {\n                (true, Some((_, _, _, w))) => p += w,\n                _ => return Err(blk_type(seg, p, p, \"assign-store-type\")),\n            }\n        }"),

    # --- H:call-arg-lit-permuted  (vsnprnc) -------------------------------
    ("call-arg-lit-permuted",
     "crates/c2-il/src/func/body/shapes/calls.rs",
     "    if !in_place && !one_moved_at_two {\n        return Err(refuse(\"call-arg-lit-permuted\"));\n    }",
     "    if !in_place && !one_moved_at_two && !crate::front3_lift(\"call-arg-lit-permuted\") {\n        return Err(refuse(\"call-arg-lit-permuted\"));\n    }"),

    # --- H:call-arg-outer-formal  (keygen_xbox) ---------------------------
    ("call-arg-outer-formal",
     "crates/c2-il/src/func/body/shapes/calls.rs",
     "        if arg_sources.iter().any(|&ix| ix >= arg_sources.len()) {\n            return Err(refuse(\"call-arg-outer-formal\"));\n        }",
     "        if arg_sources.iter().any(|&ix| ix >= arg_sources.len())\n            && !crate::front3_lift(\"call-arg-outer-formal\")\n        {\n            return Err(refuse(\"call-arg-outer-formal\"));\n        }"),

    # --- H:expr-shr-mixed-sign  (osfinfo, jsonwriter) ---------------------
    # The guard refuses `>>` whose operand classes disagree, because `sraw` and
    # `srw` are different instructions. The lift picks `ShrS` ARBITRARILY: it
    # exists to move the cursor, and the verdict it produces is discarded.
    ("expr-shr-mixed-sign",
     "crates/c2-il/src/func/body/expr.rs",
     "                    (true, true) => {\n                        return Err(Block::refuse(seg, *p, \"expr-shr-mixed-sign\"))\n                    }",
     "                    (true, true) => {\n                        if crate::front3_lift(\"expr-shr-mixed-sign\") {\n                            IlOp::ShrS\n                        } else {\n                            return Err(Block::refuse(seg, *p, \"expr-shr-mixed-sign\"));\n                        }\n                    }"),

]

# --- RETIRED: clauses this hatch used to lift and that the PORT HAS PAID ------
#
# (id, file, paid_witness, paid_at, note). `paid_witness` must be PRESENT in the
# file. That is what separates a payment from a drift: a needle that stopped
# matching because someone reformatted the function is a DRIFT and must be a hard
# error, while a needle that stopped matching because the clause was rewritten
# when the rung was paid is a RETIREMENT. Both look identical to a `count() != 1`
# check, which is why board #1322 read as "one of six edits no longer applies"
# rather than as "one of six rungs is done".
RETIRED = [
    ("store-run-bind-mixed-kind",
     "crates/c2-il/src/func/body/shapes/leaf_store.rs",
     # The clause is no longer an unconditional `return Err(...)` on
     # `addr_producer`; `w-mrslot`/`w-midrun` replaced it with the `served`
     # predicate, which ADMITS the run whose stores all go through the bind that
     # names the address. There is nothing left to lift: the refusal that
     # remains is the complement, and lifting THAT is a widening, not a probe.
     "return Err(STORE_RUN_BIND_MIXED_KIND);",
     "503f8937..04727f37",
     "PAID by w-mrslot/w-midrun; xboxheap.cpp is a `match` on this master"),
]

FILES = sorted({f for _, f, _, _ in EDITS} | {f for _, f, _, _, _ in RETIRED})


def _plan():
    """Phase one. Decide every edit against the on-disk text, WRITE NOTHING.

    Returns (writes, already, failures). A non-empty `failures` is fatal in
    `apply` — the whole point of the split is that a tree is never left half
    hatched by an edit that could not be decided until the previous ones were
    already on disk (board #1322).
    """
    cache, writes, already, failures = {}, [], [], []
    for eid, path, needle, repl in EDITS:
        full = os.path.join(ROOT, path)
        src = cache.get(full)
        if src is None:
            src = cache[full] = open(full).read()
        # "Already present" is `repl` present AND `needle` gone. Testing only
        # the first is how lane w-one's own fail-closed control came back GREEN
        # on its first run: a short replacement string that happens to occur
        # somewhere else in the file reads as an applied edit. Every real edit
        # here rewrites its needle, so the conjunction is exact.
        if repl in src and needle not in src:
            already.append((eid, path))
            continue
        n = src.count(needle)
        if n != 1:
            failures.append((eid, path, n, needle))
            continue
        src = cache[full] = src.replace(needle, repl)
        writes.append((eid, path, full))
    for eid, path, witness, paid_at, note in RETIRED:
        src = open(os.path.join(ROOT, path)).read()
        if witness not in src:
            failures.append((eid + " [RETIRED]", path, 0,
                             "paid_witness absent: " + witness))
    return cache, writes, already, failures


def apply():
    cache, writes, already, failures = _plan()
    if failures:
        sys.stderr.write(
            "HATCH FAILED — NOTHING WAS WRITTEN. %d edit(s) could not be "
            "decided:\n" % len(failures))
        for eid, path, n, needle in failures:
            sys.stderr.write("  id=%-26s %s\n      needle matched %d times, "
                             "want 1\n      %r\n" % (eid, path, n, needle[:120]))
        sys.stderr.write(
            "\nThis is board #1322's failure mode and it now FAILS CLOSED. An\n"
            "edit whose clause the port has PAID belongs in RETIRED with a\n"
            "paid_witness; an edit that merely stopped matching is DRIFT and the\n"
            "ladder built on it would be a measurement of a tree nobody intended.\n")
        raise SystemExit(2)
    for full, text in cache.items():
        open(full, "w").write(text)
    print("hatch: APPLIED %d edit(s) across %d file(s); %d already present"
          % (len(writes), len({f for _, _, f in writes}), len(already)))
    for eid, path, witness, paid_at, note in RETIRED:
        print("hatch: RETIRED  id=%s  (%s)  %s" % (eid, paid_at, note))
    print("hatch: LIFTS IN FORCE: %s"
          % ",".join(sorted({e for e, _, _, _ in EDITS if e != "helper"})))


def revert():
    subprocess.run(["git", "checkout", "--"] + FILES, cwd=ROOT, check=True)
    print("hatch: REVERTED (%d files)" % len(FILES))


def check():
    """All-or-nothing, and a COUNT rather than a status (STATUS.md trap 5)."""
    _, writes, already, failures = _plan()
    want = len(EDITS)
    dirty = subprocess.run(["git", "diff", "--stat", "--", "crates/"],
                           cwd=ROOT, capture_output=True, text=True).stdout
    print("hatch: %d of %d edit(s) present, %d pending, %d undecidable"
          % (len(already), want, len(writes), len(failures)))
    print(dirty if dirty.strip() else "crates/ diff: EMPTY")
    if failures:
        for eid, path, n, needle in failures:
            print("  UNDECIDABLE id=%-26s %s (matched %d)" % (eid, path, n))
    if len(already) not in (0, want):
        raise SystemExit("hatch: PARTIAL — %d of %d edits present. The tree is "
                         "neither hatched nor clean; run `revert`." % (len(already), want))
    print("hatch: %s" % ("APPLIED" if already else "CLEAN"))


if __name__ == "__main__":
    {"apply": apply, "revert": revert, "check": check}[sys.argv[1]]()
