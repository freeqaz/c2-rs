# The early-return-seq decline probe — applied, run, reverted

Board **#1704**: `c2rs census` reports ONE key per body, the fall-through
blocker of whichever production came last. On an unpatched tree six of the seven
cells of `fixtures/cpp/wpark_lit_permuted_neg.cpp` read `expr-cmp-eq`, which is
`parse_expr`'s key and says nothing about why any of them declined.
w-cfgclass §6.2's method, paying an **eighth** time.

The production that declines them returns `Option`, not `Result`, so — unlike
w-json's, which only had to make a committal dispatch report its `Err` — the key
here is *thrown away at the call site*. The patch, applied to
`crates/c2-il/src/func/body/shapes/early_return.rs`, run twice, and reverted
before the fixtures were committed:

```diff
     let shape = match parse_call_sequence_from(seg, &mut p, lo, Vec::new(), None, early) {
         Ok(shape) => shape,
-        Err(_) => return None,
+        Err(e) => {
+            eprintln!("W-PARK-PROBE {}", e.feature());
+            return None;
+        }
     };
```

Output: `work/w-park/neg_clauses.txt`. **Seven cells, seven DISTINCT keys.**

| cell | what it changes | key reached |
|---|---|---|
| n1 | no guard at all | `callseq-multiarg-lit-unguarded` |
| n2 | the literal in slot 0 | `callseq-multiarg-lit-slot0` |
| n3 | two literals | `callseq-multiarg-lit-two` |
| n4 | a second call | `callseq-multiarg-lit-later-call` |
| n5 | a literal wider than `li`'s signed 16 bits | `call-arg-lit-wide` |
| n6 | nine argument slots | `call-args-overflow` |
| n7 | a permutation the park's unimodal clause refuses | `callseq-early-return-permuted-args` |

## The three the probe REWROTE, and the fact behind them

The first draft of n2, n3 and n4 all reported a key other than the one they were
written for, and **the reason is the same for the first two and is a fact about
this file's fence ORDER that no prior document states**:

> The `callseq-early-return-permuted-args` fence runs **before** the
> `callseq-multiarg-lit-*` fence. So every `callseq-multiarg-lit-*` clause is
> reachable **only when the non-literal slots are already in place** — with a
> formal moved, the permutation fence answers first.

n2 was written as GRID-P's `c_s0` control (`c2l(72, a0)`), which moves `a0` into
slot 1 and therefore reached `callseq-early-return-permuted-args`; the same for
n3. Both were rewritten to leave the formals in place. **This also means GRID-P's
`c_s0` and `c_two` controls are green for a different reason than the one they
were generated to test** — they refuse, which is the property the control
asserts, but the deciding clause is the permutation fence rather than the
literal fence. Recorded here rather than restated as a success.

n4's first draft made its second call read a formal, which puts that formal live
across the first call, which is Class B — so it stopped at
`callseq-saved-with-first-call-setup`. Its second call takes no argument now.

n6 stops at `call-args-overflow`, the **argument** bound, one word before the
**formal** bound (`callseq-over-eight-formals`) it was written for. Recorded and
not reworded (w-osfinfo §5): the formal bound has no cell in this pair and is
not credited with one.
