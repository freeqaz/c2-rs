# WFL — the chain result's member is a `float`, and the ceiling held exactly

    Tag:       WFL
    Slug:      chain-tail-fp-load
    Date:      2026-07-31
    Fixtures:  wfl_chain_tail_fp_load.cpp wfl_chain_tail_fp_load_neg.cpp
    Census:    685165 → 685882 (27.82 % → 27.85 %), +717
    Record:    this file

WCO shipped the integer form of one designator step on a chained member call's
result — `lwz r3,k(r3)` — and refused the floating-point form by name. It then
did the thing that made this rung cheap: it **measured what that refusal cost by
counterfactual**, ran the 878-TU first-blocker histogram before and after its own
change, and handed over a *key* rather than a prose description. The number was
717.

**Realized: 717. Ratio 1.000×, and the post-change histogram differs from the
pre-change one in exactly one entry.**

## What it admits, and what it refuses

`float f(O* p) { return p->a()->b()->m; }` is one `lfs f1,off(r3)` after the last
`bl`, and a `double` member is `lfd`. Read off the reference obj
(`work/WFL/probe/p1.cpp`, `/O1 /GS- /c`), base already in r3:

```text
  float  member                  lfs f1,4(r3)     c0230004
  double member                  lfd f1,16(r3)    c8230010
  bare deref of a float*         lfs f1,0(r3)     — does NOT fold at 0
  float member, DOUBLE return    lfs f1,4(r3)     — BYTE-IDENTICAL to row 1
  double member, FLOAT return    lfd f0,k(r3) ; frsp f1,f0    — REFUSED
  float member, INT return       lfs f0 ; fctiwz ; stfd ; lwz — REFUSED
  &(float member)                addi r3,r3,4     — already `ok` (CallValue)
```

`SeqTail::CallLoadFp { off, double }` is a **sibling** of `CallLoad`, not a width
flag on it, for the reason `CallLoad` is a sibling of `CallValue`: it is a
different **register file**. Two consequences no integer tail has — the
destination is `f1`, and the obj acquires the undefined external `_fltused`.

**Three things this rung did not have to invent**, and checking first is why it
is one afternoon rather than three. `is_fp_type` (the two-channel, nibble-reading
FP type predicate), `encode_lfs` (which takes the width bit and emits `lfd` for
`double`), and the `_fltused` producer/consumer pair
(`IlFunction::touches_floating_point` → `coff::Function::is_float`) were all
already there, from W28/W31/W34. The whole port-side change is a `SeqTail`
variant, one four-line encoder call, one clause in the producer, and the class
tail in the parser.

### The width bit follows the MEMBER, and that is measured

`lfs` loads **and converts** — the FP register holds a double either way — so a
`float` member returned as a `double` is byte-identical to the unpromoted body
and the emitted opcode still follows the member. Reading the width off the
*result* type instead is four bytes wrong on every promotion, with no operator,
no shape and no census number changed.

This is the one place a blanket rule would have been a **discount applied to a
ceiling**, which is the failure mode the brief names: "conversions refuse" is
tidy, costs nothing to write, and would have thrown away part of the 717 for a
cell that was measured free before the estimate was written.

### The refusals, each with its measured cost

| key | what it is | reference emit | workload cost |
|---|---|---|---|
| `mcall-chain-tail-load-fp-narrow` | `double` member, `float` result | `lfd f0 ; frsp f1,f0` | **0** |
| `mcall-chain-tail-load-fp-convert` | FP member leaving the FP file | `fctiwz` + spill + reload | **0** |
| `mcall-chain-tail-load-fp-result` | `41` disagrees with the post-conversion width | — | **0**, no witness |
| `mcall-chain-tail-load-class` (residue) | `volatile float` member | `lfs f1,k(r3)` — the SAME one word | **0** |

The residue row is the interesting one and it is recorded rather than hidden:
**c2 emits the identical single `lfs` for a `volatile float` member** (measured,
`c_vol` in `work/WFL/probe/p4.cpp`), so this refusal costs coverage and not
correctness. It is kept because the predicate asked here is the **shared**
`is_fp_type`, whose volatile refusal is right at the position it was written for
— a `volatile float` *formal* is a spill, and `float f(float x, volatile float y)
{ return gf(y); }` is a 40-byte framed body where the FP tail call emits 8.
Splitting that locator by position is a rung in `readers.rs`, not a line here,
and on this workload it is worth exactly **0**.

### The locator check, in both directions

Six of the last nine rungs were "a private limit inside a recognizer that already
exists", and the mirror is a shared locator nobody asks. Both were run.

* **Nothing private was written.** The offset run is `eat_offset_adds` (shared,
  and the reason `p->Next()->gn()->m.g` folds to one `lfs f1,8(r3)`); the FP type
  test is `is_fp_type`; the width→opcode step is `encode_lfs`. The `-off-wide`
  bound, the class ordering and the `Err(Some(b))` contract are WCO's, untouched.
* **The sibling that disagreed, and it is not a defect.** There are two "is this
  an FP type, how wide" locators in the tree: `is_fp_type` (readers.rs) refuses a
  volatile tag, `store_fp_value_width` (designator.rs) does not. That is exactly
  the shape of the W35 defect, so it was measured rather than argued:
  `work/WFL/probe/p6.cpp` compiles the four volatile-FP copy leaves the store
  leaf accepts today and the port is **`Port=Match`, byte-exact**. The two
  locators legitimately differ because the two positions do — a volatile *store*
  through a pointer is already the memory access, and so is a volatile *load*.
  **Refuted, at the cost of one capture.**

### `lfd` needs no alignment gate, and the load leaf's does not carry over

`finish_indirect_load_of` guards its 8-byte integer load with `off % 4 != 0`
because `ld` is **DS-form**. `lfd` is primary 50 and **D-form**: the displacement
has no low-two-bits constraint, so an 8-byte FP load at an odd displacement
encodes fine. Copying the leaf's gate over would have been a refusal with no
reference behind it — the inverse of the mistake this family usually makes.

## Estimate vs outcome

Written to `work/WFL/ESTIMATE.md` before any scan.

| | |
|---|---|
| handed ceiling | **717** (`mcall-chain-tail-load-class:eof`, WCO's counterfactual) |
| estimate | **717**, range 650–717, bias **low** |
| realized | **717** |
| ratio | **1.000×** |

**Pre-filter named.** The 717 is a *first-blocker* count on the 878-TU dc3
workload at the workload's own flags: a function blocked earlier by another
feature is not in it and could not be gained here. The key carries `:eof`, i.e.
the body parsed to the end of the segment and the refusal is a codegen-class one
over a complete body — which is what makes it a ceiling rather than a bound, and
is the property WCL's handoff lacked.

**The bias was low and it did not materialize.** The estimate's range allowed
650–717 because three refusals could have subtracted from the key (a non-FP class
inside it, the narrowing, the conversion out of the FP file). All three measure
**0**: every one of the 717 is a plain `float` or `double` member, with no
conversion or with the free promotion. There is no residue at all —
`mcall-chain-tail-load-class` is now **0 on the workload**, and so is every other
`mcall-chain-tail-*` key WCO declared (`-load-width`, `-load-convert`,
`-off-wide`, `-addr-class`). **The chain-tail designator family is exhausted on
dc3.**

### The counterfactual, run both ways

Baseline and post-change scans, same corpus, same cache, both from this tree:

```text
  in-class   685,165 → 685,882          +717
  blocked  1,777,406 → 1,776,689        −717
  histogram entries that moved: ONE
     -717  mcall-chain-tail-load-class:eof
```

WCO's handoff predicted `+717 mcall-chain-tail-load-class:eof` and this rung
consumed exactly that entry and nothing else. **A ceiling measured by a
counterfactual *of the production being widened* is not an estimate with error
bars; it is the answer, and the only work left is counting the refusals between
the key and the emitter.** That rule has now landed at 1.0002×, 1.061× and
1.000×. Every miss on the board came from applying a discount to it or borrowing
a rate from another population.

## What this rung refuted

1. **`mcall-chain-tail-load-convert` has a witness.** WCO's header records it as
   witness-free — "the width gate fires first on every spelling a caller can
   write". It does not: an **integer** member returned as a `float`
   (`float f(O* p) { return p->Next()->gf()->a; }`) reaches the load with a
   width-4 integer, so `value_class` answers, the width gate never runs, and the
   `2C` to `float` is a cross-class conversion that lands on exactly that key.
   Measured, `work/WFL/probe/p2.cpp`: three functions, three
   `mcall-chain-tail-load-convert:eof`. Its reference emit is
   `lwa ; std ; lfd ; fcfid ; frsp` — five words. It is in the negative fixture
   and in the sweep, and it is worth **0** on the workload. *A gate believed to
   have no witness is a claim about the spellings someone thought of.*
2. **WCO's wild acceptance witness was proving less than it looked like.**
   `WILD_CHAIN_FLOAT_MEMBER` is a fragment transcribed from
   `src/system/hamobj/Ham.cpp` starting at the `4C`, with no `46 2D` formals
   marker — and `parse_params` runs *after* the tail designator. The old
   assertion passed because the tail gate short-circuited in front of it, so the
   fragment never demonstrated that the whole body parsed. It now asserts what it
   can: unmodified the refusal has moved past the tail entirely
   (`formals-marker`), and one byte changed — the `const` tag `A6` to the
   `volatile` `96` — puts it straight back on the tail's own key. **A refusal
   assertion over a truncated fragment is satisfied by any gate ordered before
   the truncation**, which is a general hazard for every pinned wild segment in
   this repo that is a fragment rather than a whole function.
3. **The `store_fp_value_width` / `is_fp_type` divergence is not a defect** —
   see above, refuted by capture.

## The sweep axis, and its separation counts

`scripts/sweep.d/99-chain-tail-fp-load.py`, **265 cases**. Graded by reverting
each rule individually and re-running the whole `expr_sweep` corpus (13,485
cases) against a separately-built binary per mutation.

| mutation | mismatches |
|---|---|
| fold `CallLoadFp` at displacement 0, as `CallValue` folds | **9** |
| take the width from the RESULT type, not the loaded member | **24** |
| ignore the width bit — always `lfs` | **45** |
| drop the `_fltused` producer clause | **153** |
| emit into `f0` (the FP pool's scratch) instead of `f1` | **154** |
| emit the integer sibling's `lwz r3,k(r3)` for the FP load | **154** |

Every rule separates, and the axis is graded by **six** distinct numbers rather
than by one green run. Three of them are worth reading:

* **24 against 45.** "Read the result type" and "always emit `lfs`" agree on
  every case where the member and the result have the same width; the 21 cases
  between them are exactly the promotions, which is the pair of rows section 2 of
  the fragment exists to produce. Neither rule is caught by any case that does
  not carry a conversion.
* **153 against 154.** Dropping the `_fltused` clause misses **one fewer** case
  than emitting the wrong instruction does — because in one mixed TU an FP leaf
  or FP store leaf ahead of the chain body already produces the symbol, so the
  obj is correct even with this rung's producer clause gone. That one case is
  the whole reason the mixed-ordering rows are in the fragment: a corpus of
  single-function TUs would have made the producer look load-bearing everywhere
  and said nothing about *which* function it hangs off.
* **9.** The offset-0 fold is the smallest and the easiest to write by accident,
  since the sibling variant four lines above it does exactly that.

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace` | **500 pass / 0 fail** (was 494 / 0) |
| `c2rs bench` | **189 pass / 0 fail / 0 error** (was 187 / 0 / 0) |
| `scripts/mode_lane.sh` `/Ox` / `/O1` / `/O2` / `/Ox /Gy` | **90 / 88 / 88 / 88**, 0 mismatch in all four (was 89/87/87/87) |
| `scripts/expr_sweep.sh` | **13,485 cases, 0 mismatches** (13,220 before) |
| `scripts/cross_sweep.sh` | **23,841 × 4 configurations, 0 mismatches** (20,194 × 4 before) — the count grew because `call-sequence-load-fp` is the 27th declared family and the lane crosses it against all 26 others |
| 878-TU workload scan | mismatch **0**, census **685,882 / 2,462,571 = 27.85 %**, disagreement **0** |
| 878-TU workload scan, **debug build** | **0 panics**, mismatch **0**, census **685,882**, disagreement **0** |
| fixtures, `c2rs census` | `wfl_chain_tail_fp_load.cpp` **17/17**, `wfl_chain_tail_fp_load_neg.cpp` **0/8** |

The debug lane is run because WCO made it standing: a `debug_assert` compiles out
of every lane that grades the port, so every assertion in the parser is an
unchecked claim until this lane runs. This rung adds no assertion, and the lane
is clean.

## Found and not taken

Ranked, **by census key**, with the instrument named for each.

1. **`expr-call-in-expr-recv-load-then-type-ptr-and-off-add-more` — 22,570**, the
   largest key WCO left and still the fourth-largest on the board. **Its name is
   not its content**: its chained twin (10,568) was decoded by hand and turned
   out to be a `float`-argument marshalling row with nothing designator-shaped in
   it. Census a witness body before quoting it — the probe costs one capture and
   has now caught four mis-described rows in two days.
2. **The `-then-type-*` instrument defect, still unrun and now cheap.** WCO's
   closing warning: `type-ptr` in a `-then-` key names the *diagnostic walk's*
   int-only vocabulary, not an acceptance refusal, and four of the twenty largest
   keys on the board are `-then-type-ptr-…`. **This rung supplies a second,
   sharper instance**: `mcall-chain-tail-load-class` was named for a *class* and
   its whole content was one register file; had it contained two constructs the
   name would have said nothing about the split. The check is one `c2rs census`
   per key against a hand-written witness body, and nobody has run it.
3. **`expr-call-in-expr-chained-then-type-int1-and-type-aggregate-whole2/3` —
   1,476 + 735 = 2,211**, and `-more` a further 736. `-whole` means the
   constructs must be admitted together, so its ceiling is not its size — but it
   *is* `-whole`, which the 22,570 is not.
4. **`float`/`double` in the OTHER shapes of this family, unsized.** This rung
   put an FP value in `f1` at a call sequence's tail. The neighbours that have
   never been asked: an FP member as a *link argument* (`p->a(q->fltA)->b()` —
   which is what the 10,568 row actually is), an FP-returning callee whose result
   is discarded (`calls.rs` already refuses this by name and says `_fltused`'s
   placement is why), and an FP comparison of two call results. None is sized;
   all three are in the same family and all three are `_fltused` producers, so
   the producer's shape enumeration is where a fourth miss would land.
5. **The `volatile` FP load, worth 0 today and a one-line rung when it is not.**
   Measured free (identical `lfs`), refused only because `is_fp_type` bundles the
   formal position's spill rule with the type test. The fix is to split that
   locator by position in `readers.rs` — one predicate per fact, both next to
   each other — and it should be done by whoever owns that file, with the count
   measured first.

## The riskiest thing left unmeasured

**`_fltused`'s placement is graded by exactly one shape of TU ordering, and this
rung is the first to make a *framed* function produce it.** The symbol goes after
the first FP-touching function's **complete** symbol group, and that group just
grew from three symbols (a leaf: `.text` aux, the function, one `$M`) to six (a
framed body: `.text` aux, the function, `$M`, `$M`, the `.pdata` aux, `$T`). It
is measured — `work/WFL/probe/p1.obj` symbol [21], and the sweep's fourteen mixed
TUs put the FP body in every position relative to an integer chain tail, an FP
leaf and an FP store leaf — but all of that is **one obj layout rule graded by
one producer**. What is *not* covered: a TU where the first FP-touching function
is framed and the second is a leaf that pools a constant, under `/Gy`, where the
`.rdata` COMDAT and the `_fltused` external compete for position in the same
symbol region. The two lanes that would show it (`/Ox /Gy` and the cross sweep's
`/Gy` configuration) both already refuse the pooled-constant-under-`/Gy` case for
an unrelated reason, so **the interaction is unreachable from any gate that runs
today** — it is not covered, and no green number here says otherwise.
