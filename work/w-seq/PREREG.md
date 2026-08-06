# w-seq — PREREG

    Lane:   w-seq
    Base:   master `039b718`
    Date:   2026-08-08
    Ships:  measurement only. **No `crates/c2-il/` change and no change to any
            emitted byte.** The one `crates/` change this lane permits itself is
            an *instrument* widening inside `crates/c2-harness/src/gap/fnbytes.rs`
            — new `fnbyte-` keys, no existing key's value altered.

Committed **before any probe exists**. Nothing below was measured first.

---

## 0. The question

`fnbyte-differs` is **3,195** (`tail` 1,531 · `seq` 1,541 · `framed` 123) after
mechanism E and its fixpoint. Known inside it: **370** `??$_Destroy_Range@…`
whose callee is parse-refused as `expr-intrinsic-memset` (`w-empty` §5), and the
**123** `framed` are one signature (`?back@?$vector@…`). The rest is unmapped.

The mission is to map all 3,195 by **mechanism**, and then to **price** the two
mechanisms it expects to find:

* **(a) mechanism I** — c2 expanded a same-TU callee into the caller, so the
  reference body *contains the callee's code* and the port emits a branch to it.
* **(b) E behind a parse refusal** — c2 applies E after its own dead-code
  elimination; the port cannot establish emptiness because the callee's IL body
  is refused by a named production.
* **(c) something else** — to be named, not to be a remainder.

## 0.1 The instrument

Per `(TU, symbol)` keyed on **`FnCensus::emit_name`** (**#918** — the positional
`IlFunction::mangled_name` disagrees on 74,955 workload rows and keying a
name-matched fact off it turned 14 byte-exact bodies wrong). For each differing
function the scan will additionally publish, from the port's own selection:

* the **callee set** of the port's body (`tail_call` / `framed_call.callee` /
  `call_seq.calls[].callee` / `cond_pair` arms), and
* each callee's **disposition** in this TU: `extern` (no census row binds it) ·
  `refused:<production>` · `empty` · `reduces` (in `TuEmptyCallees`) · `body`
  (parses, non-empty), and
* whether c2's own obj emitted a `.text` COMDAT for that callee.

## 0.2 SPLICE-P — the hypothesis family (a) is priced against

> **SPLICE-P.** For a caller `F` the port selects `Selected::Tail` with callee
> `G` defined in the same TU, the emission c2 actually produces is the port's own
> argument setup with the **branch word replaced by `G`'s complete emitted
> body**:
>
> ```
> splice(F)  =  port_body(F)[.. len-4]  ++  ref_body(G)
> ```
>
> where `ref_body(G)` is **real c2's own `.text` COMDAT for `G` in the same
> obj** — so the hypothesis is graded by the sole judge on the whole workload
> and not only on hand cells. When `F`'s port body is the single branch word,
> `splice(F) = ref_body(G)` exactly.

This is testable on every workload differ at once, because c2's obj carries both
COMDATs. The hand grid then asks the separate question SPLICE-P does not: **can
the port produce those bytes**, i.e. is `G` in the port's accepted class, and
does the splice perturb the register allocation, the schedule or the frame.

---

## 1. Registered claims

### P1 — **THE CLAIM I MOST EXPECT TO LOSE.** SPLICE-P holds on the workload's `tail` family

Among `shape=tail` differs whose port body's callee `G` is bound by a census row
of the same TU **and** has its own `.text` COMDAT in the reference obj:

* **registered: `splice-exact / splice-graded` ≥ 0.60**, and the sub-population
  with `port_words == 1` (a bare `b G`, no argument setup) is **≥ 0.75**.

**LOSS CONDITION**: either ratio comes in below its bound. That is the outcome
that kills the cheap path — it would mean inlining re-allocates registers or
re-lays the frame, and the caller's correct bytes are not any function of the
callee's own emitted bytes.

I expect this to lose on the `port_words > 1` half (an argument setup ahead of an
inlined body is exactly where a scheduler would interleave) and to hold on the
bare-branch half. Registering both separately is the point: **only one of them
can be the finding.**

### P2 — the splice does **not** extend to `seq`, and the reason is the frame

For `shape=seq` differs the port's body opens `7d8802a6` (`mflr r12`) and c2's
does not.

* **registered: ≥ 0.90 of `seq` differs have `ref_words < port_words`**, and
* **registered: `splice-exact` over `shape=seq` is 0** under the same
  concatenation rule (setup ++ callee bodies, in call order).

**LOSS CONDITION**: `splice-exact` on `seq` is ≥ 1, or the shorter-reference
share is below 0.90. Either would say the cheap path reaches further than this
prereg thinks.

### P3 — the taxonomy's coverage

**registered: ≥ 0.85 of the 3,195 name at least one callee that is bound by a
census row of the same TU** (i.e. the divergence is *about* a same-TU callee, in
mechanism I's or mechanism E's sense).

**LOSS CONDITION**: < 0.85 — a large family diverges for a reason that is not a
same-TU callee at all, and the mission's (a)/(b) split is the wrong axis.

### P4 — family (b)'s productions

* **registered: `expr-intrinsic-memset` is the largest single refusal production
  behind a differ**, at **exactly 370** pairs whose reference body is the single
  word `4e800020`, reproducing `w-empty` §5's number from a different route.
* **registered: the whole `refused:*` disposition is < 0.25 of the 3,195** —
  family (b) is the smaller half.

**LOSS CONDITION**: the count is not 370, or another production is larger, or the
refused share is ≥ 0.25.

### P5 — CONTROL: nothing the port emits moves

* `git diff master..HEAD -- crates/c2-il/ crates/c2-core/` is **empty**.
* the FBM partition is **identical at both ends**: `fnbyte-exact` **35,982**,
  `fnbyte-differs` **3,195**, `-partial` 0, `-refused` 130,573, `-unbound`
  9,225, denominator **178,975**, `fnbyte-partition-broken` **0**.
* `fnbyte-elided` / `-elided-exact` **1,516 / 1,516**; `fnbyte-name-disagree`
  **74,955**.
* TU match **10**, mismatch **0**, codegen-gap **0**, vocab-gap **861**,
  capture-fail **7**, `census/gate disagreement` **0**.
* every `gap-metric` line that is not one of this lane's new keys is
  **byte-identical**, by `diff` of two sorted files and never by reading a
  summary.

### P6 — CONTROL: a known-answer test on the callee resolver

* **registered: 0** `shape=tail` differs have a callee whose disposition is
  `reduces` — such a function is elided by `drops_tail_call` and is
  `fnbyte-exact`, so a positive count means the resolver is looking at the wrong
  row. (**#918**'s failure mode, as a positive check with a printed count.)
* **registered: the resolver is run under BOTH name bindings and the
  disagreement is published.** Keyed on `mangled_name` instead of `emit_name`,
  the same-TU resolution rate is expected to be **strictly lower**. A tie would
  mean the control cannot see #918 on this population and must be said so.

### P7 — the hand grid is frozen before it is compiled

The splice grid's cell list is `sha256`-stamped and **committed before the first
`cl.exe`**. Every cell carries an **anchor** whose callee the TU does not define;
a cell whose anchor loses its relocation is refused, not scored. Graded from
**obj bytes** (#843), callee COMDATs resolved **by name through the symbol
table** (#644), and the caller's whole `.text` printed beside every verdict
(**#950** — the relocation observable cannot see a self-branch).

### P8 — hand verification against real c2

**≥ 2 cells per major family** printed word for word from an obj this lane
compiles itself, including frame-word counts (**w-seam #869**).

---

## 2. What this lane will NOT do

* **It ships no emitter change and no parser widening.** If SPLICE-P holds, the
  deliverable is a **spec** for the emitter change and a board row, not a patch —
  the mission says so and `w-fnbyte` §8.1's reason applies: narrowing or widening
  in the lane that measured is how a measurement becomes a wrong emit.
* It does not re-price mechanism I's `INLINE-P` decision rule. Whether c2 inlines
  is `docs/INLINE_PREDICATE.md`'s question and is **NOT MODELLED** at 2.84 %
  residual; this lane asks only *what the bytes are when it did*.

## 3. Decline clause

If the new instrument moves **any** existing `gap-metric` value, or
`fnbyte-partition-broken` goes non-zero, or the workload scan's `differs` count
changes, the instrument is reverted before anything else is reported. An
instrument that changes what it measures is not an instrument.

## 4. Traps this prereg is bound by

* **Trap 4/5** — totals are not controls; every claim above is a **count with a
  denominator**, checked positively; nothing is judged through `tail`/`head`.
* **The /QXSTALLS lesson** — any claim that discriminates one population from
  another is **stratified by body length** before it is believed.
* **#843** — `sub`/`subf` are not the same mnemonic; bytes, not mnemonics.
* **#644** — nothing is one contiguous field.
* **w-ilx** — one directory for byte-diffed captures: `work/w-seq/caps/`.
