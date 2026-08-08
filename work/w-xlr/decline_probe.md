# w-xlr — the `_neg` decline clauses, read PER CELL

`c2rs census` reports the **fall-through** blocker, not the production's own —
board **#1704**. On an unpatched tree every one of `wxlr_create_guard_neg.cpp`'s
ten cells reads `assign-rhs-call-0x26`, which is the key the *next* production
raises after `try_parse_xlrc_create_guard` declines and the dispatcher moves on.
So a `_neg` fixture that is only **counted** establishes that ten functions
declined and **nothing** about whether they declined for ten different reasons.

This is the sixth lane to pay w-cfgclass §6.2's method.

## The probe

Applied, run, and **reverted before the next commit** — the tree that ships is
the one above and below this file.

```diff
--- a/crates/c2-il/src/func/body/mod.rs
+++ b/crates/c2-il/src/func/body/mod.rs
-                if let Ok(shape) = try_parse_xlrc_create_guard(seg, p, lo, sy.addr_locals) {
-                    disp("disp-xlrc-create-guard");
-                    return Ok(shape);
-                }
+                match try_parse_xlrc_create_guard(seg, p, lo, sy.addr_locals) {
+                    Ok(shape) => { disp("disp-xlrc-create-guard"); return Ok(shape); }
+                    Err(b) => { disp("disp-xlrc-create-guard"); return Err(b); }
+                }
```

```sh
printf -- '/nologo /c /O1 /Oi /EHsc /GR\n' > work/w-xlr/o1.txt
./target/release/c2rs census fixtures/cpp/wxlr_create_guard_neg.cpp \
    --flags-file work/w-xlr/o1.txt
```

## The result — ten cells, ten clauses, no collisions

`work/w-xlr/neg_clauses.txt` is the run.

| cell | what it changes | clause reached |
|---|---|---|
| n1 | `int size` instead of `unsigned` | `xlrc-stack-object-is-not-an-unsigned-word` |
| n2 | a **second** address-taken local | `xlrc-not-exactly-one-address-taken-local` |
| n3 | arm constants with different high halves | `xlrc-arm-constants-do-not-share-a-lis` |
| n4 | a status constant whose low half is 0 | `xlrc-status-constant-is-not-lis-plus-ori` |
| n5 | `result` initialized to 1 | `xlrc-result-not-initialized-to-zero` |
| n6 | three formals | `xlrc-not-four-formals-free-fn` |
| n7 | the outer test inverted (`!= 0`) | `xlrc-outer-test-relation` |
| n8 | a fourth statement in the success arm | `xlrc-ok-close-8` |
| n9 | the middle guard is `<=` | `xlrc-inner-test-relation` |
| n10 | a two-argument attach call | `xlrc-attach-arg2-is-not-the-first-formal` |

## Two things only running it per cell could show

### 1. The first draft collapsed FOUR cells into one, and counting would have hidden it

The cells were first written with the inner `if` **unbraced**
(`if (size < 4) result = X; else result = Y;`). Braces are not cosmetic in this
stream: an unbraced arm emits no `53 53` scope pair, so **n3, n4, n8 and n10 all
stopped at `xlrc-inner-then-scopes`** — one shared clause, three of the four
never reaching the fact they were written to test. The file still read
`0/10 functions in class` and the *count* was identical.

Every cell now differs from `wxlr_create_guard.cpp` in exactly one way, braces
included, and the four moved to their own clauses.

### 2. `n1` is the only cell guarding a LIVE WRONG EMIT

The other nine name shapes the emitter has no words for, so accepting one would
produce a short or mis-branched body. `n1` is different: `int size` and
`unsigned size` emit the **identical `22` relational byte** — the opcodes are
sign-agnostic (`docs/CODEGEN_W6_COMPARE.md` §1.1) — and differ only in the
operand TYPE. c2 emits `cmpwi cr6,r11,4` for the signed one where this class's
emitter has an unconditional `cmplwi`. Without the clause the port would accept
n1 and emit **one wrong word**, in an obj that links.

The clause was **not** in the first version of the recognizer. It was added
because this fixture was written, which is the argument for writing `_neg`
fixtures before the gate rather than after it.
