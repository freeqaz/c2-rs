# WCO — the designator on a chain's result, and the ceiling that named a different construct

    Tag:       WCO
    Slug:      chain-tail-load
    Date:      2026-07-31
    Fixtures:  wco_chain_tail_load.cpp wco_chain_tail_load_neg.cpp wco_explicit_return.cpp
    Census:    685165 → 685165 (27.82 % → 27.82 %), +0
    Record:    this file

Two things landed. The one that was scheduled admits **0 functions on the
workload**, and the one that was not is a **live wrong-bytes emit on mainline**.

## 1. The scheduled rung: `return p->a()->b()->m;`

WCH shipped the chain that ends at its outermost call and WCL added the
arguments on its links. This is the one instruction that may follow it. Both
cells read off the reference obj (`work/WCO/probe/p1.cpp`, `/O1 /GS- /c`):

```text
  int  c_off  (O* p) { return  p->Next()->gf()->m; }   bl ; bl ; lwz  r3,4(r3)
  int  c_off0 (O* p) { return  p->Next()->gf()->a; }   bl ; bl ; lwz  r3,0(r3)
  int* c_addr (O* p) { return &p->Next()->gf()->m; }   bl ; bl ; addi r3,r3,4
  int* c_addr0(O* p) { return &p->Next()->gf()->a; }   bl ; bl ;   (nothing)
```

**The two middle rows are the whole content**, and they are four bytes of
`.text` apart. The address cell is `SeqTail::CallValue`, shipped since #35 rung
1, and folds `+0` away by itself — a recognizer and nothing else, exactly as
WCL predicted. The load cell is one new tail, `SeqTail::CallLoad`, and it does
**not** fold at 0: `*(r3 + 0)` is a memory read that has to happen. WCL's note
said "offset 0 emits nothing"; that is true of the address form only, and is
the first thing this rung refuted.

A bare `30` with no offset add — `*p->a()->b()` — is the same `lwz r3,0(r3)`,
so the recognizer is anchored on **either** byte. Anchoring on the add alone
would have been a private limit inside a recognizer that already handles the
general case, which is §6n item 1 for the seventh time.

### The locator, both directions

The offset run goes through **`eat_offset_adds`**, shared. Not a style
preference: the indirect-load *leaf* carried a private single-add copy until
W35, it refused **5,161** functions the address and store leaves beside it
accepted, and a private copy here would have reproduced that defect verbatim one
production over. `p->Next()->gf()->in.y` is `27 · 27` and folds to one
`lwz r3,20(r3)`; `&…->arr[3]` is `27 · 28` and folds to one `addi r3,r3,36`.

The other direction — the type tail **agrees rule for rule** with
`finish_indirect_load_of`, measured across the whole width table in one TU
(`work/WCO/probe/p6.cpp`), base already in r3:

```text
  int / int* / nested / subscripted   lwz  r3,k(r3)        — ADMITTED
  char, unsigned char                 lbz  r3,k(r3)
  short, unsigned short               lhz  r3,k(r3)
  long long                           ld   r3,k(r3)
  char widened to int                 lbz  r11,k(r3) ; extsb r3,r11
  float / double                      lfs / lfd  f1,k(r3)  — a different file
```

Only the first row is admitted; every other is refused **by name**
(`mcall-chain-tail-load-width`, `-load-class`, `-off-wide`, `-addr-class`,
`-load-convert`, `-load-result`). Widening to the rest means moving
`finish_indirect_load_of`'s width/sext dispatch into a locator both can call —
a rung, not a line — and the table above is its ceiling handed over intact.
`-load-convert` and `-load-result` have **no witness**: the width gate fires
first on every spelling a caller can write, which is exactly why they refuse
instead of being skipped.

**Every gate is raised after the whole-body parse succeeds**, and the ordering
is a contract. `Err(Some(b))` means "this IS the production and it parsed to the
end of the segment"; raising the width refusal at the `30` would claim bodies
carrying a further construct and replace their measured `-then-…-more` key with
an uninformative one. The wild capture `mcall::WILD_CHAIN_AS_RECV_LOAD` is the
witness that forced the ordering, and it is now asserted in `shapes::mcall_chain`
under its new name.

## 2. The alarm: `void f() { g(); return; }` was emitting a frame

**`mismatch` outranks widening work**, so this is the rung's real content.

`void h1() { g1(); return; }` is `b ?g1@@YAXXZ` — four bytes, byte-identical to
the same body without the `return;`. c2 records the fallthrough as a **second**
`3A <label>` to the label the return plumbing then uses and emits nothing for
it. The port emitted the 36-byte framed Class A body: the statement-call
production's tail-call probe runs the plumbing at `BODY_SCOPE_DEPTH`, which
cannot parse the double `3A`, so the body fell through to `parse_call_sequence`
— where a **`debug_assert` declared the state unreachable and was wrong**.

```text
  work/WCO/probe/ret.cpp   /Ox /GS- /c
    reference .text: 12 B for two functions — 48000000, 4BFFFFF8
    port:            Port=Mismatch @ offset 2      ← live on mainline
```

Three things kept it invisible, and each is a recorded shape:

* **an assertion is not a gate.** A `debug_assert` compiles out of the release
  scan, and the false-*green* direction is the hazard (`GAPS.md` §6, the same
  lesson as the never-rebuilt gate binary).
* **98.5 % of the workload is `vocab-gap`** and never byte-compared, so the
  functions that reach this state on `src/system/hamobj/DancerSequence.cpp`
  are counted, not compiled.
* **no fragment in `sweep.d` had ever written an explicit `return;`.** It
  changes no operator, no shape and no type — two bytes of IL and nothing else
  — which is precisely the class that has now found seven live mis-emits.

It surfaced by accident: a debug `c2rs census` on a real workload TU, run to
find a witness for something else, tripped the assertion. `scripts/sweep.d/
99-explicit-return.py` is the axis that would have found it, and
`fixtures/cpp/wco_explicit_return.cpp` is the regression.

The repair routes the body to the same `tail_call_shape` the caller would have
used. It is not a refusal: the body **is** the tail call.

## Estimate vs outcome

Written to `work/WCO/ESTIMATE.md` before any scan.

| | |
|---|---|
| handed ceiling | **14,244** (`-then-type-ptr-and-off-add-more` 10,568 + `-then-type-ptr-and-op-more` 3,676) |
| estimate | **4,000**, range 1,500–14,244, bias low |
| realized | **0** |

**The handed ceiling names a different construct, and probes said so before the
scan.** Measured by census on `work/WCO/probe/p1.cpp`–`p5.cpp`:

| body | census key | workload count |
|---|---|---|
| `return p->Next()->gf()->m;` | `chained-then-off-add-and-deref-load-whole2` | **0** |
| `return &p->Next()->gf()->m;` | `chained-then-off-add-whole` | **0** |
| `return p->Next()->gq(q)->m;` | `chained-then-type-ptr-and-off-add-whole3` | **0** |

Not "small" — **absent**. `chained-then-off-add-*` totals **1 function** across
878 TUs. The construct WCL read the obj for does not occur in dc3 at all.

The 10,568 it quoted is a different body. Decoded by hand from a wild segment
and confirmed byte-identical in 6 of 6 sampled TUs, it is

```text
  return p->M(q->fltA)->M(q->fltB);        …and then more
    26 <M> 26 <M> B9 <p> 99 BD <ptr> B9 <q> 33 <int> 00 27 <ptr>
    30 A6 45 F3 30   2C 86 45 40 00   55 86 45 40   4C   …
```

— a chain whose link arguments are **`float` members of another pointer**. The
`type-ptr` in the key is the *argument's base pointer*, the `off-add` is the
designator on **it**, and `-more` is everything after. Nothing in that row is a
designator on the chain's own result.

**The estimate rule did not fail; the input to it did.** "Take the ceiling and
count the refusals between it and the emitter" assumes the ceiling names the
production. WCR's rule — *a refusal key bounds only the feature it is named
after* — has a second half this rung supplies: **a key bounds only the feature
it is named after, and the previous rung's prose is not that name.** WCL read an
obj and did not census the body it compiled; the two-line check that separates
`-then-off-add-and-deref-load-whole2` from `-then-type-ptr-and-off-add-more`
would have cost one `c2rs census` invocation.

**The operand-type cross came out FREE here and it still did not help.** Probed
before the estimate: `p->Next()->gq2(q,3)->gi()` and `return p->Next()->gq(q);`
both census `ok` **today** — `link_arg_slots` has no type gate, so `type-ptr` in
a census key names the *diagnostic walk's* int-only vocabulary and not an
acceptance refusal. That is the opposite of what the same cross said in WCR, and
it is why the estimate was set at 4,000 rather than at 0: the cross was run,
answered "free", and was answering about the wrong row.

### Category

**§6n (5), mis-described** — and the brief assigned it to the sixth category
("declared, sized, split and named by the previous rung on its way past"). The
sixth category is real, but it requires the previous rung to have named the row
by its **key**, not by an obj it compiled from a hand-written probe. WCH earned
its successor a 1.0002× estimate by instrumenting its own boundary with a
counterfactual; WCL earned its successor a 0× by reading an obj and quoting a
neighbour's key.

## What the row actually was, measured

The counterfactual is exact — the baseline and post-change first-blocker
histograms differ in **four** entries and the total blocked delta is **0**:

```text
   -717  expr-call-in-expr-chained-then-deref-load-more
     -1  expr-call-in-expr-chained-then-convert
     +1  call-arg-nonformal:eof
   +717  mcall-chain-tail-load-class:eof
```

**717 functions are one FP-load lowering away.** They are the chain whose
result's member is a `float`: `lfs f1,k(r3)` plus the `_fltused` obligation, and
they now say so by name instead of hiding inside `-then-deref-load-more`. That
is this rung's hand-off, and unlike the one it received it is a *key*, measured
by counterfactual, not a prose description of a probe.

## The sweep axes, and their separation counts

Two fragments, graded by reverting each rule individually over the whole
`expr_sweep` corpus.

`scripts/sweep.d/99-chain-tail-load.py`, **165 cases**:

| mutation | mismatches |
|---|---|
| fold `CallLoad` at offset 0, as `CallValue` folds | **18** |
| take only the FIRST offset add (the pre-W35 private copy) | **19** |
| emit the load at displacement 0 always (ignore the sum) | **60** |
| emit the address form's `addi` for the load form | **78** |

`scripts/sweep.d/99-explicit-return.py`, **81 cases**:

| mutation | mismatches |
|---|---|
| revert the alarm fix — one call with a void tail stays a framed `CallSeq` | **13** |

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace` | **490 pass / 0 fail** (was 484 / 0) |
| `c2rs bench` | **186 pass / 0 fail / 0 error** (was 182 / 0 / 0) |
| `scripts/mode_lane.sh` `/Ox` / `/O1` / `/O2` / `/Ox /Gy` | **88 / 86 / 86 / 86**, 0 mismatch in all four (was 86/84/84/84) |
| `scripts/expr_sweep.sh` | **13,220 cases, 0 mismatches** (12,791 before) |
| `scripts/cross_sweep.sh` | **20,194 × 4 configurations, 0 mismatches** (16,985 × 4 before) — the count grew because `call-sequence-load` is a new declared family and the lane crosses it against all 25 others |
| 878-TU workload scan | mismatch **0**, census **685,165 / 2,462,571 = 27.82 %**, disagreement **0** |
| 878-TU workload scan, **debug build** (new lane) | **0 panics**, mismatch **0**, census **685,165**, disagreement **0** |
| fixtures, `c2rs census` | `wco_chain_tail_load.cpp` **19/19**, `wco_explicit_return.cpp` **13/13**, `wco_chain_tail_load_neg.cpp` **0/11** |

## Found and not taken

Ranked, **by census key with the counterfactual run**, not by prose.

1. **The `float` member on a chain result — 717, exact.**
   `mcall-chain-tail-load-class` is this rung's own refusal key and its whole
   content is `lfs f1,k(r3)` (or `lfd`) plus `_fltused`. The recognizer is
   already written and the offset already folded; what is missing is an FP tail
   variant and the TU-level `_fltused` model the port has refused since W36.
   It is the cheapest large thing left in this family **and it is bounded by a
   counterfactual, not by a name**.
2. **`expr-call-in-expr-recv-load-then-type-ptr-and-off-add-more` — 22,570**,
   the fourth-largest key on the board, and the ONE-LINK sibling of the 10,568
   this rung refuted. **Do not price it from its name.** Its chained twin is a
   `float`-argument marshalling row; census a witness body before quoting it.
3. **`expr-call-in-expr-chained-then-type-int1-and-type-aggregate-whole2/3` —
   1,476 + 735 = 2,211**, and `-more` a further 736. `-whole2`/`-whole3` means
   two or three constructs must be admitted together, so its ceiling is not its
   size — but it is `-whole`, which the 10,568 is not.
4. **The narrow widths on this tail — measured but unsized.** `lbz`/`lhz`/`ld`
   and the `extsb` widening are `mcall-chain-tail-load-width`, worth **0** on
   this workload. The table is in `chain_result_designator`'s header; the rung
   that wants them should share `finish_indirect_load_of`'s dispatch rather than
   copy it, and should expect no census for the trouble.
5. **The debug-build lane — RUN, and it is clean.** The `debug_assert` that
   found the alarm compiles out of every lane that grades the port: the scan is
   `--release`, the fixture suite is `--release`, and both sweeps pin a release
   binary. So every other assertion in the parser was an unchecked claim. The
   whole 878-TU workload was therefore re-run in a **debug** build
   (`cargo run -p c2-harness --bin c2rs -- gap …`, no `--release`): **0 panics**,
   and mismatch / census / disagreement identical to the release run
   (0 / 685,165 / 0). No other assertion in the parser fires on this corpus.
   That is one command and it should be a standing lane, because it is the only
   one that can turn an assertion into a mismatch report.

6. **The riskiest thing left unmeasured, and it is about the instrument.** *The
   census key family and the acceptance production disagree about what a type
   is, and nothing checks it.* `type-ptr` in a `-then-` key means the diagnostic
   walk's int-only operand vocabulary could not spell an operand; it says
   **nothing** about whether the acceptance side would take it — measured here,
   `p->Next()->gq2(q,3)->gi()` carries a `type-ptr` key and is `ok` today. Every
   ranking taken off a `-then-type-*` key is therefore reading a property of the
   *instrument* as a property of the *port*, and four of the twenty largest keys
   on the board are `-then-type-ptr-…`. The check is cheap and nobody has run
   it: for each `-then-type-*` key, census one witness body and record whether
   the named type is an acceptance refusal at all. Until that is done, the
   ranking those keys imply is not known to mean anything.
