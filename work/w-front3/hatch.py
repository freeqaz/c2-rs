#!/usr/bin/env python3
"""w-front3 — the LIFT HATCHES, applied to a SCRATCH TREE and NEVER COMMITTED.

    python3 work/w-front3/hatch.py apply     insert the hatches (ALL or NONE)
    python3 work/w-front3/hatch.py revert    remove them — REFUSES if the tree
                                             carries anything that is not a hatch
    python3 work/w-front3/hatch.py revert --force
                                             revert anyway, naming what it destroys
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

# 2026-08-08, lane w-hatch — `revert` WAS THE SAME DEFECT POINTING THE OTHER WAY (#1380)

`w-one` closed #1322 in `apply` and `check`. **`revert` was left as a bare
`git checkout -- <six crates/ files>`**, and during lane `w-instr` a routine
revert silently discarded that lane's own unstaged fix to `calls.rs` — one of
the six. #1322 is *the instrument leaving a tree nobody intended*; #1380 is
**the instrument destroying a tree nobody asked it to change**, which is worse,
because the evidence of it is gone.

`revert` now **un-applies every substitution from the working copy in memory**,
compares the result with what `git checkout --` would restore, and **refuses**
when anything is left over. `--force` reverts anyway and names every file it is
about to overwrite before it does.

Three things about that comparison that are not obvious:

1. **It compares against the INDEX, not `HEAD`.** `git checkout -- <path>`
   restores from the index, so the index is exactly the content that *survives*
   and the working copy's excess is exactly the content that *dies*. Board
   #1380's sketch says `HEAD`; on a tree with nothing staged the two are
   identical (which is the tree #1380 was demonstrated on), but with a staged
   non-hatch change `HEAD` would refuse a revert that destroys nothing — a false
   positive, and `w-one`'s `PARTIAL`-on-a-correct-tree is the recorded cost of
   exactly that mistake. An index that differs from `HEAD` is *reported*, never
   refused on.
2. **Un-apply is `repl -> needle`, byte-wise, and the comparison is on bytes.**
   Reading through `text=True` would translate newlines and could hide a CRLF
   difference, which is a difference `git checkout --` would happily destroy.
3. **Every refusal leads with its own word** — `HATCH-DIRTY`,
   `HATCH-UNREADABLE`, `HATCH-UNTRACKED`, `HATCH-CHECKOUT-FAILED`,
   `HATCH-RESIDUE` in `revert`; `HATCH-DRIFT` and `HATCH-PAID-MISSING` in
   `apply`. `apply`'s two were one shared `HATCH FAILED:` prefix until this
   lane, and a shared prefix is how `w-throughput` had **two of six mutations
   pass silently**: a later gate's refusal satisfies an earlier case's
   expectation and the earlier case is never really tested.

`revert` also asserts its own POSTCONDITION: after the checkout, the six files
must be clean. A count, not a status (STATUS.md trap 5).
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
    #
    # RE-DERIVED at master `2b1c89da`, lane w-hatch. `w-667` split the site into
    # two keys and put the type half behind `store_type_gate` (board #667's
    # `C2RS_SINK_STORE_TYPE`), so the needle's `eat_int_like(seg, &mut p)` became
    # `store_type_gate(seg, &mut p)` and `apply` had been REFUSING on this master
    # since `dc844f64` — which is `w-one`'s fail-closed working, and #1355's
    # fourth control cell already records it. Re-taken against the tree, not
    # deleted: the clause is NOT paid, it MOVED.
    #
    # ⚠ AND IT IS NOW SUPERSEDED, which is a result rather than a repair.
    # `StoreTypeSink::Any` (`C2RS_SINK_STORE_TYPE=any`) is this lift, committed,
    # with the sink discipline the hatch deliberately lacks. A ladder run should
    # prefer the sink; the hatch stays so a run that has not set the variable
    # still climbs the same rung, and so the two can be compared.
    ("assign-store-type",
     "crates/c2-il/src/func/body/shapes/assign.rs",
     "        if !store_type_gate(seg, &mut p) {\n            return Err(blk_type(seg, p, p, \"assign-store-type\"));\n        }",
     "        if !store_type_gate(seg, &mut p) {\n            match (crate::front3_lift(\"assign-store-type\"), crate::func::readers::read_type(seg, p)) {\n                (true, Some((_, _, _, w))) => p += w,\n                _ => return Err(blk_type(seg, p, p, \"assign-store-type\")),\n            }\n        }"),

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

    # --- H:expr-convert-no-value  (src/Main.cpp) — ADDED BY LANE w-one --------
    # `w-front3` recorded `Main.cpp` as one of only TWO rows stopped by "a real
    # refusal with no lift". It is not one. The guard's own comment says the
    # state "cannot be reached by a well-formed stream" — and the UNSUNK 878-TU
    # scan witnesses `expr-convert-no-value-0x2C` **4,973 times across 829 of
    # 878 TUs**. What is empty is the model's CLASS STACK, not the stream: every
    # token whose stack effect `parse_expr` does not model (a `26` symbol push, a
    # relational, an intrinsic, and every sink skip, which clears `cstack_ok` by
    # construction) advances the cursor without pushing a class.
    #
    # The lift assumes `Int4` for the missing source class. That is a GUESS and
    # it is exactly the guess the shipped guard refuses to make — which is why it
    # lives here, uncommitted, and why nothing but `fn_blockers` is read off it.
    ("expr-convert-no-value",
     "crates/c2-il/src/func/body/expr.rs",
     "                let Some(cls) = cstack.last().copied() else {\n                    return Err(blk(seg, start, \"expr-convert-no-value\"));\n                };",
     "                let cls = match cstack.last().copied() {\n                    Some(c) => c,\n                    None if crate::front3_lift(\"expr-convert-no-value\") => ValueClass::Int4,\n                    None => return Err(blk(seg, start, \"expr-convert-no-value\")),\n                };"),
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
        # "Already present" is: the replacement occurs EXACTLY ONCE, and the
        # needle occurs exactly as many times as the replacement itself contains
        # it. Both halves are load-bearing and both were learned the hard way:
        #
        #  * testing `repl in src` alone made lane w-one's own fail-closed
        #    control come back GREEN — a short replacement string occurring
        #    anywhere in the file reads as an applied edit;
        #  * testing `needle not in src` alone reports the HELPER edit as
        #    pending forever, because its needle (`pub mod codec;`) is a
        #    substring of its own replacement, and `check` then says PARTIAL on
        #    a tree that is fully and correctly hatched.
        if src.count(repl) == 1 and src.count(needle) == repl.count(needle):
            already.append((eid, path))
            continue
        n = src.count(needle)
        if n != 1:
            failures.append(("HATCH-DRIFT", eid, path, n, needle))
            continue
        src = cache[full] = src.replace(needle, repl)
        writes.append((eid, path, full))
    for eid, path, witness, paid_at, note in RETIRED:
        src = open(os.path.join(ROOT, path)).read()
        if witness not in src:
            failures.append(("HATCH-PAID-MISSING", eid, path, 0,
                             "paid_witness absent: " + witness))
    return cache, writes, already, failures


def apply():
    cache, writes, already, failures = _plan()
    if failures:
        # Two DIFFERENT defects, and they get two DIFFERENT leading words. They
        # shared `HATCH FAILED:` until lane w-hatch, and a shared prefix lets a
        # drift refusal satisfy a red-test written for a retired-witness one —
        # `w-throughput` had two of six mutations pass exactly that way.
        sys.stderr.write(
            "NOTHING WAS WRITTEN. %d edit(s) could not be decided:\n"
            % len(failures))
        for word, eid, path, n, needle in failures:
            sys.stderr.write("  %-18s id=%-26s %s\n      needle matched %d "
                             "times, want 1\n      %r\n"
                             % (word, eid, path, n, needle[:120]))
        sys.stderr.write(
            "\nThis is board #1322's failure mode and it now FAILS CLOSED. An\n"
            "edit whose clause the port has PAID belongs in RETIRED with a\n"
            "paid_witness (HATCH-PAID-MISSING is that check failing); an edit\n"
            "that merely stopped matching is HATCH-DRIFT and the ladder built on\n"
            "it would be a measurement of a tree nobody intended.\n")
        raise SystemExit(2)
    for full, text in cache.items():
        open(full, "w").write(text)
    print("hatch: APPLIED %d edit(s) across %d file(s); %d already present"
          % (len(writes), len({f for _, _, f in writes}), len(already)))
    for eid, path, witness, paid_at, note in RETIRED:
        print("hatch: RETIRED  id=%s  (%s)  %s" % (eid, paid_at, note))
    print("hatch: LIFTS IN FORCE: %s"
          % ",".join(sorted({e for e, _, _, _ in EDITS if e != "helper"})))


def _index_blob(path):
    """The bytes `git checkout -- <path>` would restore, or None if there are none.

    Stage 0 of the index — NOT `HEAD`. See the module docstring: the index is
    what survives a checkout, so the working copy's excess over it is exactly
    the bytes that die.
    """
    r = subprocess.run(["git", "show", ":" + path], cwd=ROOT,
                       stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    return r.stdout if r.returncode == 0 else None


def _head_blob(path):
    r = subprocess.run(["git", "show", "HEAD:" + path], cwd=ROOT,
                       stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    return r.stdout if r.returncode == 0 else None


def _checkout(paths):
    """The destructive step, behind ONE seam.

    It is a seam so `hatch_red.py` can fire the two guards that stand *after*
    it — `HATCH-CHECKOUT-FAILED` and `HATCH-RESIDUE` — which are structurally
    unreachable on a well-formed tree and would otherwise be guards nobody has
    ever watched. Both arms are labelled INJECTED in the red output; a guard
    fired through a seam is a weaker demonstration than one fired by a real
    defect and the report has to say which it is.
    """
    return subprocess.run(["git", "checkout", "--"] + paths, cwd=ROOT,
                          stderr=subprocess.PIPE, text=True)


def _unapply():
    """Un-apply every substitution from the WORKING COPY, in memory.

    Returns `(unapplied, live, problems)`:
      unapplied  {path: bytes} the working copy with every hatch removed
      live       the clause names that were actually found in the tree
      problems   [(leading-word, path, detail)] — EVERY problem, not the first

    Byte-wise on purpose (docstring note 2). `bytes.replace` on a `repl` that
    is absent is a no-op, so a partially hatched tree un-applies the part that
    is there and the rest of the comparison still speaks.
    """
    buf, live, problems = {}, set(), []
    for path in FILES:
        full = os.path.join(ROOT, path)
        try:
            with open(full, "rb") as fh:
                buf[path] = fh.read()
        except OSError as e:
            problems.append(("HATCH-UNREADABLE", path,
                             "cannot read the working copy: %s" % e))
    for clause, path, needle, repl in EDITS:
        if path not in buf:
            continue
        nb, rb = needle.encode(), repl.encode()
        if rb in buf[path]:
            if clause != "helper":
                live.add(clause)
            buf[path] = buf[path].replace(rb, nb)
    return buf, live, problems


def revert(force=False):
    """Remove the hatches — and REFUSE if that would destroy anything else.

    Board #1380. The old body was `git checkout -- <FILES>`, which discards
    every unstaged change in six `crates/` files whether this instrument put it
    there or not. It ate lane `w-instr`'s own fix to `calls.rs` without a word.
    """
    unapplied, live, problems = _unapply()

    dirty, staged_note = [], []
    for path in FILES:
        if path not in unapplied:
            continue                       # already reported HATCH-UNREADABLE
        idx = _index_blob(path)
        if idx is None:
            problems.append(("HATCH-UNTRACKED", path,
                             "no stage-0 index entry — `git checkout --` has "
                             "nothing to restore and would fail or delete"))
            continue
        if unapplied[path] != idx:
            dirty.append(path)
        head = _head_blob(path)
        if head is not None and head != idx:
            staged_note.append(path)

    if (dirty or problems) and not force:
        # #1380 published this message verbatim; it is reproduced rather than
        # reworded, so the board row and the instrument say the same thing.
        if dirty:
            sys.stderr.write(
                "\nHATCH-DIRTY — %d file(s) carry changes that are NOT this "
                "hatch's.\nNOTHING WAS REVERTED. `git checkout --` would discard "
                "them silently.\n\n" % len(dirty))
            for path in dirty:
                sys.stderr.write("  %s\n" % path)
            sys.stderr.write("\n")
        if problems:
            sys.stderr.write("NOTHING WAS REVERTED — %d further problem(s) with "
                             "the files this would overwrite:\n" % len(problems))
            for word, path, detail in problems:
                sys.stderr.write("  %-18s %s\n      %s\n" % (word, path, detail))
        sys.stderr.write(
            "\n  clauses currently live in the tree: %s\n"
            "  files this revert would have touched: %d\n\n"
            "  Nothing here is lost yet. Commit or stash the work that is not the\n"
            "  hatch, then re-run; or `revert --force` to destroy it deliberately.\n"
            % (", ".join(sorted(live)) or "NONE", len(FILES)))
        if staged_note:
            sys.stderr.write(
                "  (informational, NOT a reason for this refusal: %d file(s)\n"
                "   differ between HEAD and the index — %s. A checkout restores\n"
                "   the INDEX, so those bytes are not at risk.)\n"
                % (len(staged_note), ", ".join(staged_note)))
        raise SystemExit(3)

    if (dirty or problems) and force:
        sys.stderr.write(
            "\nHATCH-FORCED — reverting over %d dirty file(s) and %d other "
            "problem(s).\nThese bytes are being DESTROYED, deliberately:\n"
            % (len(dirty), len(problems)))
        for path in dirty:
            sys.stderr.write("  %-18s %s\n" % ("HATCH-DIRTY", path))
        for word, path, detail in problems:
            sys.stderr.write("  %-18s %s\n" % (word, path))
        sys.stderr.write("\n")

    r = _checkout(FILES)
    if r.returncode != 0:
        sys.stderr.write("\nHATCH-CHECKOUT-FAILED — `git checkout --` exited %d.\n"
                         "%s\n" % (r.returncode, r.stderr))
        raise SystemExit(4)

    # POSTCONDITION, as a count and not a status (STATUS.md trap 5). A revert
    # that reported success while leaving the tree hatched is the same class of
    # lie as the applier that reported success on 7 of 8.
    left = subprocess.run(["git", "diff", "--name-only", "--"] + FILES,
                          cwd=ROOT, capture_output=True, text=True).stdout.split()
    if left:
        sys.stderr.write("\nHATCH-RESIDUE — %d of %d file(s) still differ after "
                         "the checkout:\n" % (len(left), len(FILES)))
        for p in left:
            sys.stderr.write("  %s\n" % p)
        raise SystemExit(5)
    print("hatch: REVERTED %d file(s); %d clause(s) were live: %s"
          % (len(FILES), len(live), ", ".join(sorted(live)) or "NONE"))


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
        for word, eid, path, n, needle in failures:
            print("  %-18s id=%-26s %s (matched %d)" % (word, eid, path, n))
    if len(already) not in (0, want):
        raise SystemExit("hatch: PARTIAL — %d of %d edits present. The tree is "
                         "neither hatched nor clean; run `revert`." % (len(already), want))
    print("hatch: %s" % ("APPLIED" if already else "CLEAN"))


if __name__ == "__main__":
    _cmd = sys.argv[1]
    if _cmd == "revert":
        revert(force="--force" in sys.argv[2:])
    else:
        {"apply": apply, "check": check}[_cmd]()
