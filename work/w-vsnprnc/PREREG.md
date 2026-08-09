# PREREG — lane `w-vsnprnc`

**Frozen before the first probe cell and before the first change to `crates/`.**
Branch `wt-w-vsnprnc` off master `42fe7cb1`.

Commission: convert `src/xdk/LIBCMT/vsnprnc.cpp` to byte-exact — match 19 → 20.

---

## 0. What was read before this file was written, and what was NOT

**Read (orientation and baseline, no cell authored):** `CLAUDE.md`,
`docs/STATUS.md`, board rows **#1417 #1418 #1466 #1467 #260 #1346**,
`docs/rungs/2026-08-08-w-clear.md` §§1–4, `2026-08-08-w-extdata.md` §1,
`2026-08-08-w-undname.md` §§1–2, the shipped
`crates/c2-il/src/func/body/shapes/guard_chain_shared_tail.rs` fence list and
`crates/c2-core/src/codegen/guard_chain_shared_tail.rs` word table, the two
sources, and — with the real `c2.dll` under wibo at the workload's own flags and
cwd — the **reference objs** of `vsnprnc.cpp` and `vswprnc.cpp`
(`work/w-vsnprnc/obj/`) plus a one-TU `gap` on `vsnprnc`.

**NOT done before this file:** no probe cell authored or compiled, no change to
`crates/`, no fixture written.

## 1. The baseline this lane starts from

Master `42fe7cb1`: TU **match 19**, mismatch **0**, codegen-gap **0**,
vocab-gap **852**, capture-fail **7**, FRONTIER **8**. `vsnprnc` reads
`vocab-gap`, `2 blocked | 2 emitted`, byte fraction **0/164 = 0.0 %**, `labels 3`,
and the codegen column reads `reader 2 / wrong 0 / cg-ref 0` — i.e. **both**
functions are behind the IL parser and the codegen price is not readable off the
instrument at all (#1474's own caveat).

`fnbyte-exact`, the workspace test count and the target count are being collected
at this same tree; every call below is a **DELTA**, so the freeze does not depend
on them.

## 2. What the two objs already say (measured, not predicted)

`_vsprintf_s_l` (152 B) is `_vswprintf_s_l` (156 B, **already converted** by
`w-extdata`) with **exactly three** differences:

1. **five formals, not six** — the rotate is 4 steps, not 5;
2. the interleaved `lis` sits after rotate step **1**, not step **2**;
3. `stb r11,0(r31)` where `vswprnc` has `sth` — a `char*` buffer, not `wchar_t*`.

Every other word, the block order, the two condition registers and the merged
tail are identical.

`vsprintf_s` (12 B) is `mr r7,r6 ; li r6,0 ; b _vsprintf_s_l` — an **unframed**
forwarding tail call in its own `.text` COMDAT with **no `.pdata` record** and
**no `$M` label**, in a TU whose other function is framed and does have both.

**This is the second witness the shipped fence asked for in writing.** Its own
words: *"with one witness there is no way to tell 'after the second' from 'three
before the last', so the arity is pinned"*. At 4 steps, "after the second"
predicts the `lis` after step 2 and the obj has it after step 1; **"three before
the last" is the surviving hypothesis and "after the second" is refuted.**

## 3. Registered calls, in probability form

| # | call | P |
|---|---|---:|
| **C1** | **TU match delta = +1** (`vsnprnc` converts) | **0.60** |
| C1a | match delta = 0 (declined, or fn1 only) | 0.40 |
| C1b | match delta < 0 anywhere | 0.00 — a lost TU is a defect, not an outcome |
| **C2** | **`fnbyte-exact` delta = +2** (both functions) | **0.55** |
| C2a | `fnbyte-exact` delta = +1 (fn1 only, TU still `vocab-gap`) | 0.20 |
| C2b | `fnbyte-exact` delta = 0 | 0.20 |
| C2c | `fnbyte-exact` delta ≥ +3 (the widened arity admits a body I have not seen) | 0.05 |
| **C3** | **workspace test-count delta = +14**, **target count 38 → 38** | 0.35 within ±3 |
| C4 | mismatch stays 0 everywhere, at every gate row | **0.99** |
| C5 | `fn_gate_refusals` is 0 keys over the whole 878-TU scan at the tip (#1467's over-claim does **not** ship) | 0.95 |
| C6 | the "three before the last" `lis` rule survives a probe grid at arities the workload does not supply | 0.75 |
| C7 | I need a change to `crates/c2-core/src/codegen/coff.rs` (w-ifn's single-occupancy seam) and must STOP | 0.30 |

**No discount factor is applied to C1/C2.** The blocker class for fn1 has a
shipped emitter, so per the standing rule the ceiling *is* the estimate.

## 4. My own re-derived price, refusal by refusal

Counting **independent** refusals — "what varies between these two refusals? if
nothing, it is one refusal":

**`_vsprintf_s_l`** — the shipped class refuses it for:

| # | refusal | independent of the others because |
|---|---|---|
| R1 | `FORMALS != 6` (reader) and `params.len() != 6` (emitter) | one fact, one refusal — the two sites cannot disagree by construction |
| R2 | the `lis`'s position inside the rotate | varies with arity but is **not implied by it**: this is exactly the pair the shipped fence says it could not separate |
| R3 | `gcst-store-is-a-word-not-a-halfword` — the store is a **byte** | varies independently of arity |

**R1–R3 = three refusals, and R2 is the only one that needs a measurement
rather than a widening.**

**`vsprintf_s`** — refused for:

| # | refusal | |
|---|---|---|
| R4 | `call-arg-lit-permuted` — a literal in a middle slot beside a formal that is not already home | the census key #1417 names; `hatch.py` has a lift for it |
| R5 | an **unframed** function sharing a TU with a framed one: no `.pdata` record and **no `$M` label** for the leaf | a writer/label-plan question, not a body question |

**Total: five independent refusals**, of which three are fence widenings on a
class that already emits thirty of the thirty-eight words, one is a measurement,
and one (R5) is the risk that owns C7.

**Against #1418's `≥ 7 measured + 1 inferred + 1 masked`.** That price was taken
**before** `vswprnc` converted; it counted the distance from
`w11_early_return.cpp::mm` to `vsnprnc`, and eight of those rungs have since been
paid by `guard_chain_shared_tail` landing. #1418's decline is honest and I am not
refuting it — I am re-deriving the price **from the class that exists today**,
which is five, not nine. If the measurement in §5 refutes that, this file says so
in advance.

## 5. Registered failure modes

* **F1 — fitting the `lis` rule at n = 2 and shipping it.** Board #260's exact
  warning: a plausible-looking wrong instruction. **Mitigation, registered
  before the probe:** grade probe cells at arities the workload does not supply
  (4 and 7 formals) against real `c2`. If I get n ≥ 3 the rule generalizes; **if
  I cannot, I ship an ENUMERATED arity set `{5, 6}` and refuse everything else**,
  which is narrower than a rule and cannot emit a wrong word.
* **F2 — the census over-claiming (#1467).** `fn_gate_refusals` must be 0 keys
  over the whole scan at the tip. Checked and published either way.
* **F3 — the cr0/cr6 discriminator (#260).** I measure it on this body and state
  it; I do **not** generalize it. The shipped emitter already pins `cmpwi cr0`
  for `r < 0` and `cmpwi cr6` for `r != S` and both are byte-confirmed on two
  objs now. Anything beyond "these two sites, this class" is out of scope and
  saying so here is what keeps a later reader from mistaking two cells for a rule.
* **F4 — a refusal becoming a wrong emit.** Every widening is fenced so a body
  the port cannot do byte-exactly is refused. Checked by `mismatch 0` at three
  levels and by the 878-TU per-TU **set** comparison by name, not by count.
* **F5 — a wrong label charge going inert.** `vsprintf_s` charges the compiler
  label counter **0** (the obj has 3 labels, the same as single-function
  `vswprnc`'s 3). A fixture that puts the unframed leaf **last** cannot detect a
  wrong charge. **At least one fixture puts the unframed leaf FIRST**, ahead of a
  framed function, so a wrong charge shifts that function's `$M` numbers and the
  cell is live. Measured at **/O1 and /Ox**; no label charge is taken from
  `docs/LABEL_COUNTER.md`.

## 6. Decline clauses — stated before the outcome is known

* **D-COFF.** If the conversion needs `crates/c2-core/src/codegen/coff.rs`
  changed, I **STOP and report**. It is w-ifn's single-occupancy seam and a
  second occupant is the merge failure git cannot see.
* **D-IFN.** If it needs the general `cflow-if-2` / `cflow-if-n` emitter class, I
  do **not** build a second one. `guard_chain_shared_tail` is a transcription
  and its branch layout is already shipped; extending *that* is not building an
  `if-n` emitter, and this file draws the line there in advance.
* **D-SHARED.** I widen `calls.rs`' `call-arg-lit-permuted` fence **only** if the
  widening is provably byte-neutral on the population it already admits. If it is
  not, R4 gets its own narrowly fenced module and the shared predicate is left
  alone. **Wideners only; never a narrowing, never a shadow.**
* **D-PRICE.** If the re-derived price exceeds **8** independent refusals, I
  decline and publish the table. **A priced decline is a good outcome.**

## 7. Seams

Mine: `guard_chain_shared_tail` (both halves — a **widening in place**, because
a second reader of one fact is worse than a wide fence), any new shape module I
author, my fixtures, my rung, board rows **#2350–#2369**.

Not mine, not touched: `codegen/coff.rs`, the `cflow-if-2`/`if-n` general
emitter, `src/xdk/nuispeech/mmio.cpp`, harness scan instruments, gap-metric
keys, `scripts/status.sh`.
