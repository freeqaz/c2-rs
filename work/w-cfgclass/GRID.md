# w-cfgclass — the fixture grid, FROZEN before the first `cl.exe` on it

Committed **before** `fixtures/cpp/wcfg1_*.cpp` exist on disk and before any of
these cells has been compiled. Board **#198** and w-clear's confound (bitten
twice) are the reason: a grid whose predictions are written after the run cannot
separate rivals, and a family exhaustive on the axis it varies and blind on the
one it holds fixed reads as complete.

`src/system/negate_test.cpp` itself already grades `match` at the tip
(`a71c7599`); that is the *conversion*, and it is one cell. This grid is the
**fence**, and its job is to say what the class does at the edges the workload
does not contain.

## The two rivals

* **R-TIGHT** (what shipped): the class is the exact block plan, and every axis
  below that varies a *value* stays byte-exact while every axis that varies the
  *structure* refuses.
* **R-LOOSE** (the emitter a lane would write from the one workload instance): a
  general one-compare/two-guard/two-arm lowering that does not check the arms'
  argument lists, the dead arm's literal, or the formal kinds. R-LOOSE and
  R-TIGHT agree on every POSITIVE cell and disagree on every NEGATIVE one — so
  the grid separates them iff the negative cells come back refused rather than
  wrong.

## Positive cells — `fixtures/cpp/wcfg1_if_call_join.cpp`

Predicted `Port=Match` (byte-exact against real `c2.dll`) at every graded lane
whose mode word is `/O1`, and `NotImplemented` at `/Ox` and `/O2`.

| cell | what it varies | prediction |
|---|---|---|
| **p0** | the dc3 body verbatim, `==` spelling | Match |
| **p1** | the `!(x != k)` spelling of the middle test | Match, **and byte-identical to p0** |
| **p2** | both literals moved (`k1 = 3`, `k2 = 7`) | Match; exactly two words differ from p0 |
| **p3** | a negative `k1` (`-1`) | Match |
| **p4** | different callees and a different accumulator pointee | Match |

**The separating cell is p1.** Nothing else in the corpus grades two source
spellings that must emit one word, and it is the clause the reader's
`1F`/`20` alternation exists for.

## Negative cells — `fixtures/cpp/wcfg1_if_call_join_neg.cpp`

Predicted **`NotImplemented`** — never `Mismatch`. Each is one clause of the
fence, and the prediction is registered per cell rather than per file because
the file's verdict is the conjunction and would be satisfied by any one of them
firing.

| cell | what it breaks | the clause it must trip |
|---|---|---|
| **n0** | the two arms call with **different arguments** | the hoist is illegal: a setup goes back inside each arm |
| **n1** | the dead arm stores a **different literal** | the middle block is no longer empty |
| **n2** | **two formals**, no `float` | the park/hoist register assignment is arity-specific |
| **n3** | the accumulator is a **file-scope** pointer | `li r11,0` would drop a real memory store |
| **n4** | the middle test is `<` rather than `==`/`!=` | a different successor for the shared compare |

Per-cell verdicts are read with `c2rs census`, because a file-level
`NotImplemented` is satisfied by one cell refusing and says nothing about the
other four.

## Decline clause on the grid itself

If any negative cell comes back **`Match`**, the fence is wider than its
declaration and the class is reverted before the gate — a refusal becoming a
wrong emit is strictly worse than a gap (board #232, 241 commits). If any
positive cell comes back **`Mismatch`**, likewise.

If p1 is NOT byte-identical to p0, the reader's two-spelling alternation is
wrong and must be narrowed to whichever spelling the workload contains, with the
other refused.
