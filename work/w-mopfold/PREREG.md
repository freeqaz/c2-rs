# PREREG — lane `w-mopfold` (construct rung)

Frozen and committed **before the first edit to `crates/`**. Never edited after;
graded in the rung, and a miss is said in the word **MISS**.

Base: master `0dcfca959`, branch `wt-w-mopfold`.
Charter: `docs/DECISIONS_2026-08-22.md` § Decision 17, board `#3654`; the
finding being acted on is `#3637`, with `#3638` and `#3640` as sub-repairs.

## 0. What I had already read when this was written — declared, not hidden

Decision 17 says a prereg is *"committed before the first measurement"*; my
charter says *"before the first edit"*. They differ, and the honest thing is to
say which one this document satisfies and what I knew.

**Committed before the first edit. NOT before the first read.** Before freezing
this I had already read, on my own tree:

- `docs/BOARD.md` rows `#290`, `#3336`, `#3637`–`#3641`, `#3654`;
- `crates/c2-core/src/codegen/mop.rs` — the whole `mod op` list, the 85-row
  `OPCODES` table, `MachineOp`, `Field`/`FieldPlan`/`plan`, `base_word`;
- `crates/c2-core/src/codegen/calls.rs` lines 1–160 and `frame.rs` lines 1–100;
- `crates/c2-core/src/codegen/encode.rs` lines 1–60;
- one grep: `to_be_bytes` counted per file across `crates/c2-core/src` (93 raw
  hits, comments and `cfg(test)` included);
- one grep for the `#3638` pledge string, with comment markers stripped.

So **P2 and P6 below are informed predictions, not blind ones**, and a HIT on
either is weak evidence. P1, P3, P4, P5, P7 are not: nothing I have read
answers them. I have run no scanner, executed no code, and built nothing.

## 1. Predictions

**P1 — my own enumeration of live (non-`cfg(test)`, non-comment) instruction-word
production sites outside the `mop` seam, over all of `crates/c2-core/src`.**
`#3637` reports **eleven**. I predict my scan finds **more than eleven** —
point estimate **13**, range 11–20, **bias: high**. Reason for the bias: the
raw `to_be_bytes` grep is 93 hits across 13 files and `#3637`'s eleven come
from only two of them (`calls.rs`, `frame.rs`); `coff/ehscope.rs` alone has 10
raw hits and `#3637` lists it only as a *consumer*. A scan that admits
`guard_chain_shared_tail.rs`, `labels.rs`, `osf_handle_guard.rs`,
`counted_accum_loop.rs`, `close_call_chain.rs`, `alloc_init_or_fail.rs` will
find sites `#3637` did not name — some of which will be legitimate.
**HIT iff my count is ≥ 12 and I can name every site above eleven.**

**P2 — the number of DUPLICATE sites (informed).** The charter and Decision 17
both say **seven** and then both enumerate **eight** file:line pairs
(`calls.rs:36/98/102/106` + `frame.rs:54/57/63/74`). I predict the true count
of duplicates among `#3637`'s eleven is **eight**, and that the headline
"seven" is an arithmetic slip in the row, not a different set — the three
non-duplicates being `bl` (`calls.rs:142`), `mfspr` (`frame.rs:60`) and
`stwux` (`frame.rs:70`), because `OPCODES` carries no `bl`, no `mfspr` and no
`stwux` row. `#3637` also says *"the four that are NOT duplicates"* and names
three. **8 + 3 = 11; 7 + 4 = 11 as well, which is how a row can be internally
consistent in its arithmetic and wrong in both terms.**
**HIT iff duplicates = 8 and non-duplicates = 3 among `#3637`'s eleven.**

**P3 — how many of the duplicates fold byte-neutrally.** I predict **8 of 8**,
i.e. all of them, with the required-zero identity diff at 0 lines.
**Named risk, and the site I expect to fail if any does: `calls.rs:36`
`encode_tail_branch`.** It masks the displacement with `& 0x03FF_FFFC` over a
*byte* displacement, while `mop`'s form 6 reads `Slot::DispWord` — an
**arithmetic** `>> 2` then a mask then a `<< 2` — and `mop.rs`'s own `DispWord`
doc says in as many words that the two functions coincide only while the
displacement is a multiple of 4 and that this is *"a caller precondition rather
than a property of the field"*. If the port ever calls it at a non-multiple of
4, the two rules disagree — which would make `calls.rs:36` a **live** second
producer rather than a latent one, and that would be a finding above the fold.
Second-most-likely refusal: `frame.rs:63` `mtspr`, because form 62's SPR field
is **split** (`8 → 0x100`) and a baked full word does not show which half is
which. **HIT iff 8 fold with a 0-line identity diff.**

**P4 — the control.** I predict the control ships as **two halves**, and that
**both** can be watched failing:
  (a) *completeness*, syntactic — every live word-emission site in
      `crates/c2-core/src` is either inside the `mop` seam or named in a
      committed inventory; a new ad-hoc site is red because it is in neither;
  (b) *discrimination*, semantic and **executed** — each inventory row carries
      a witness word produced by calling the site, and the row is red if that
      word is **coverable** by `mop`.
I predict (b) is where the design risk is and that a naive
primary-opcode-only discriminator **would be wrong**, giving a false positive
on `bl` (primary 18, same as `b`). **HIT iff both halves ship and both are
observed red on a planted defect.**

**P5 — the discriminator, stated in advance.** This is the sentence the test is
built on, and it is registered here so it cannot be back-fitted:

> A live word production outside the `mop` seam is a **DUPLICATE** iff the word
> it emits lies in the **image of `mop::encode_op` over some `OPCODES` row at
> the default `EncodeParams::C2`** — i.e. iff there is a row and an operand
> assignment that composes the identical 32 bits. It is **LEGITIMATE** iff no
> row can compose it, which means the port emits an instruction c2's
> transcribed subset does not carry and there is no second rule to disagree
> with.
>
> **Decided mechanically, over-approximating toward RED.** For row `r` with
> form `f`, let `mask(f)` be the union of the field masks in `plan(f)` (each
> `((1<<width)-1) << shift`). Word `W` is coverable by `r` iff
> `W & !mask(f) == (r.base | plan(f).fixed) & !mask(f)`. This is a superset of
> the true image (a field may not reach every value in its mask), so the test
> can only over-report duplicates, never under-report them. A row claiming
> LEGITIMATE therefore has to survive the **generous** test.

I predict this discriminator classifies `#3637`'s eleven as 8 duplicate / 3
legitimate with **zero** hand-written exceptions. **HIT iff no site needs an
exception clause.**

**P6 — the false pledges (informed).** `#3638` says the claim is written
**three** times at `mop.rs:38-39`, `mop.rs:111`, `encode.rs:36`. I predict:
(i) all three line numbers are **stale** — the real sites are `mop.rs:53-54`
and `mop.rs:127-128`, moved by `w-disclose`'s `#3643` insertion; and (ii) there
is a **fourth** site `#3638` missed, `mop.rs:680`, the doc comment on
`base_word` **itself**, which is the most load-bearing of the four.
I predict that after the fold the claim is **still FALSE**, because `bl`,
`mfspr` and `stwux` remain and each sources a primary opcode from a literal.
So the repair is a **correction to the true statement**, not a promotion to
true. **HIT iff four sites and still-false.**

**P7 — the axis on which this rung can fail with every byte identical**
(`rungs/README.md` cost clause, board `#3336`). Named before starting:
**the duplicate count itself is the axis, and it is a real one because it is
measured by an instrument that is watched failing.** The rung fails if, with a
0-byte delta, (a) the duplicate count does not reach **0**, or (b) the control
cannot be made red on a planted eighth producer, or (c) the control's site
count is **0 or unchanged**, which would mean the scanner sees nothing (the
`--check`-that-cannot-fail failure, board `#3336`, and the vacuity floor
`cli_flags.rs`'s `locate_is_reachable_only_through_the_arg_seam` already
pins). I will report the site count before and after, and it must **drop by
exactly the number folded**. I predict cost is **not** measurably moved and I
do **not** name it as the axis: the folded sites are prologue/epilogue and
call-sequence emissions, a handful of words per function, against `mop`'s
already-O(1) `row()`.

## 2. Decline floor

- If a duplicate cannot be folded **byte-neutrally**, I **STOP on that site**.
  I do not adjust the emitted word to match `mop`, and I do not adjust `mop` to
  match the site. It stays as it is, keeps its inventory row, and the row is
  marked as a **known-duplicate exception with a two-sided price** — what
  folding it would cost, and what leaving it costs — in the code, not only in
  the rung.
- A partial fold with an honest reason beats a complete one that moved a byte.
- If the identity diff is not 0 lines I revert the fold entirely and the rung
  outcome is `FAILED` on deliverable 2, whatever deliverable 1 did.
- If `#3637`'s eleven turn out to be a different set than I can reproduce, that
  is the lane's finding and it is reported as a **correction to `#3637`**,
  including if it makes the fold smaller.
- I add **no metric key**, no provenance reader, no DISCLOSURE row, and I write
  nothing outside `crates/c2-core/src/codegen/**`, `work/w-mopfold/**`,
  `docs/rungs/2026-08-26-w-mopfold.md` and board rows `#3655`–`#3660`.

## 3. What I will report either way

Outcome word; tip sha; the `GATE:` verdict line quoted; the 21-row identity
diff; base/tip `cargo test --workspace` test **and** target counts; my own
enumeration's count against `#3637`'s eleven; folded vs refused with each
refusal's two-sided price; the control and **the transcript of it going red**;
the pledge repairs; follow-ups deliberately not taken.
