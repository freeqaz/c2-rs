# WLA — `g3(a, b, 7)`: the literal call argument, which is one `li` and no move

    Tag:       WLA
    Slug:      lit-call-arg
    Date:      2026-07-31
    Fixtures:  wla_lit_call_arg.cpp wla_lit_call_arg_neg.cpp
    Census:    691,744 → 696,551 (28.09 % → 28.29 %), +4,807
    Record:    this file, and `docs/IL_CALL_IN_EXPR.md` §26

The multi-argument tail call modeled a **pure register permutation**, so every
argument had to be a bare formal LOAD. A literal fell out under
`call-arg-computed` — 5,537 functions, the largest `:eof` row on the board after
the `:eof` renderer was repaired, and `calls-1` on 5,537 of 5,537.

The name was wrong about its own content, and one counterfactual said so: **the
row contains no computed argument at all.** Zero operand streams, zero
non-formals, zero literals too wide for `li`. It is 5,537 literals, and 4,792 of
them are one shape.

## What it admits, and what it refuses

Admits a literal argument in a multi-argument tail call **beside formals that are
already in the argument register they are being passed in**. The lowering is one
`li r<3+i>,k` per literal slot, highest destination first, and no move at all.
Read off the reference obj (`work/WLA/probe/p1.cpp`, `/O1 /GS- /c`):

```text
  void f(int a)       { g2(a, 5); }           38800005  li 4,5   · b ?g2
  void f(int a,int b) { g3(a, b, 7); }        38a00007  li 5,7   · b ?g3
  int  f(O* p,int j)  { return p->gk(j, 7); } 38a00007  li 5,7   · b ?gk
  void f(int a,int b,int c){ g4(a,b,c,9); }   38c00009  li 6,9   · b ?g4
  void f(int a)       { g3(a, 5, 6); }        li 5,6 · li 4,5    · b ?g3
```

The member form **is** the free form — `this` is the formal in slot 0 — which is
why the row is large: W36's member call turns every one-argument member call into
a two-argument list, so `o->v1(7)` lands here too.

Refuses, each by name, each with the capture that would settle it:

* **`call-arg-lit-permuted` — 733 functions.** A literal beside a formal that has
  to *move*. Two of the three sub-shapes are captured and would come out of a
  descending walk correctly (`g3(a,7,b)` is `mr r5,r4 ; li r4,7`, `g3(7,a,b)` is
  `mr r5,r4 ; mr r4,r3 ; li r3,7`); the third is not — the same list over a real
  permutation **cycle** (`g3(b,a,7)`), where the r11 break temp wants a slot in
  the order too. The three share one gate because "the formals are in place" is
  the property the emitted bytes depend on.
* **`call-arg-lit-wide` — 0 on the workload.** Past `li`'s signed 16-bit field a
  literal is `lis`+`ori`: `g3(a,b,32767)` is one instruction and
  `g3(a,b,70000)` is `lis 5,1 ; ori 5,5,4464`, measured one line apart.
* **`callseq-multiarg-lit` — 0 on the workload.** A **framed** call's literal:
  the statement-call sequence and a chain's innermost call. Their marshalling
  interleaves with the callee-saved copies and with the previous `bl`'s result
  save, and every witness of that interleaving is a `mr`. Refused through the one
  shared locator (`seq_call_arg_sources`) so both callers refuse it the same way
  and under the same key — the fixture proves the key is reachable, since a
  refusal nothing can reach is invisible to every gate.
* `call-arg-computed` keeps its name and now means what it says: an argument that
  is an operand stream. **0 functions** on this workload.

## Estimate vs outcome

Stated before the build, from a counterfactual that re-keyed the refusal without
moving one function into class (in-class stayed at 691,744 under it, which is the
check that nothing was claimed):

| functions | `:eof` bucket | what it is |
|---:|---|---|
| 4,399 | `id-n3-lit-last` | 3 slots, 2 formals in place, the literal last |
| 393 | `id-n2-lit-last` | 2 slots, same |
| 12 | `id-multilit` | two or more literals |
| 733 | `lit-perm` | a formal out of its slot |
| **0** | `stream` · `nonformal` · `lit-wide` · lit **not** last | measured, not assumed |

> **Estimate: +4,804, biased LOW.** The counterfactual measured this population
> and the implementation gates on exactly it, so the point estimate is the
> measurement. Biased low by two named sources: the counterfactual counted only
> the `:eof` half (7 functions carry these shapes at `:mid`, in a value position
> with plumbing still ahead), and a second gate behind this one — callee `.gl`
> resolution — could refuse an unmeasured part of the population.

**Outcome +4,807.** The low bias was the first source and its population was
**3**: three `:mid` bodies completed too. The second source's population was
**0** — no body in this row lost its callee. Bucket arithmetic, with nothing
absorbed: 5,544 released, 735 refused again under `call-arg-lit-permuted`, 4,807
in class, and the residue of **2** is `call-multiarg-postop-0x33` +1 and
`result-type-0x41` +1 — both named, neither rounded away.

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace` | **521 pass, 0 fail** (unchanged) |
| `c2rs bench` | **199 pass, 0 fail, 0 error** |
| `scripts/gate.sh --jobs 4` | **12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, 2,388 fixture-verdicts (was 2,364), 0 mismatch in every lane |
| `scripts/expr_sweep.sh` | **13,808 cases, 0 mismatches** (was 13,707); the new fragment is 101 cases, **72 of them grade `Match`** |
| 878-TU workload scan | 878 rows graded, `fn_total` 2,462,571, match 6 / capture-fail 7, **mismatch 0**, census **691,744 → 696,551**, **census/gate disagreement 0** |
| fixtures, `c2rs census` | `wla_lit_call_arg.cpp` **34/34**, whole obj byte-exact; `wla_lit_call_arg_neg.cpp` **0/20**, `Port=NotImplemented` |

0 TUs changed class, **0 TUs lost a function**, 563 TUs gained and the largest
single gain is 32 — a property of the corpus, not of one file.

## Found and not taken

1. **`call-arg-lit-permuted`, 733 functions**, all `calls-1`, all whole-body
   complete. The literal beside a formal that moves. Two of its three sub-shapes
   are already captured; the missing one is a literal beside a real permutation
   cycle, and it is **one probe TU**, not a rung's worth of work. Whoever takes
   it must grade the cycle cell specifically — the shift cells agree with a plain
   descending walk and the cycle cell is where a fitted rule would break.
2. **`callseq-multiarg-lit`, 0 on this workload** — the framed call's literal.
   Worth recording as a number rather than a suspicion: the framed side of this
   shape simply is not in this corpus, so the interleaving question is not
   blocking anything measurable here.
3. This rung leaves `call-arg-computed` at **0 functions**. The key that ranked
   third on the handoff list in `ROADMAP.md` §6l — "mixing a formal with a literal
   in a multi-argument call, still uncaptured" — is closed, and the general
   "computed argument into an argument register" rung it was standing in for has
   **no population on this workload at all**. That is the more useful result: it
   was never the same rung.
