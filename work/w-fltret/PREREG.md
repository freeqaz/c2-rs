# w-fltret — PREREG

**Frozen before the first `crates/` change and before the first fixture line.**
Everything in §0 is re-derived in this worktree at master `05d743f7` by
`work/w-fltret/rows.py` over `work/w-fltret/scan_base.jsonl`; nothing in it is
inherited from a rung's prose. Everything in §2–§4 is a prediction and is scored
in the rung whatever it does.

Lane: `w-fltret`, worktree branch `worktree-agent-a7d759e2be9f5e7ef`.
Commission: w-callprice's **R2** — the float value tail of a statement-position
member-call sequence (`docs/rungs/2026-08-09-w-callprice.md` §7, ROADMAP
§10.26.7, board #2032).

---

## 0. THE BASE, RE-DERIVED IN-TREE (not inherited)

Master `05d743f7`; binary `6e034c8140d3`; workload `152e1e32`; 878 TUs, 871
captured, 6.4 s.

| quantity | value |
|---|---:|
| TU match / mismatch | **18 / 0** |
| per-function census | **711,514 / 2,463,443** |
| emitted census | **39,200 / 178,977** |
| family `expr-call-in-expr-*` | **423,905 bodies / 35,576 emitted** |
| distinct first-blocker keys | **635** bodies · **614** emitted |
| distinct `prod` tags | **914** |
| `gap-metric` lines | **251** |
| workspace `#[test]` count (`git grep -c`) | **1,355** |

**w-callprice §1 reproduces digit for digit** on the family (423,905 / 35,576),
which is the check that this lane is on the base it thinks it is.

### 0.1 R2's population, re-derived — and it is **545**, not 544

| key | emitted | bodies | TUs |
|---|--:|--:|--:|
| `expr-call-in-expr-recv-load-then-type-real-whole` | **439** | 933 | 714 |
| `expr-call-in-expr-chained-then-type-real-whole` | **105** | 1,472 | 735 |
| `expr-call-in-expr-recv-object-then-type-real-whole` | **1** | 1 | 1 |
| **total** | **545** | 2,406 | |

w-callprice §7 summed the first two and published **544 over 9 constructs**.
There is a **third** `-type-real-whole` key in the family at 1 emitted. The
correction is +1 and the point is that the population was re-walked rather than
copied.

### 0.2 …and it is **two different reader routes**, which §7 does not separate

On the `prod` axis, by bodies:

| key | `prod` tag | bodies |
|---|---|--:|
| `recv-load-then-type-real-whole` | `tail-void-body-does-not-end-at-the-call` | **714** |
| `recv-load-then-type-real-whole` | `tail-returned-body-does-not-end-at-the-call` | 3 |
| `chained-then-type-real-whole` | `chain-returned-body-does-not-end-at-the-call` | **1,472** (all of it) |

`chained-then-type-real-whole`'s 105 emitted are **100 % on `mcall_chain`'s
route**, not on `CallSeq`'s. The commission's *"`BodyShape::CallSeq` already
lowers the statement half — reuse, do not duplicate"* is a statement about the
439, not about the 105. **This lane takes the `CallSeq` route only** (D5).

### 0.3 The fences that exist at base, with their sizes

| census key | bodies | emitted | what it is |
|---|--:|--:|---|
| `result-type-0x41` | **810** | **1** | `eat_return_plumbing`'s `41 <TYPE>` gate — `eat_int_like_or_ptr4` refuses a real result. This is the FREE-function FP value tail's fence |
| `call-ret-fp` | **0** | **0** | `CallRet::discarded`. **The key does not appear anywhere in the 878-TU workload.** Stated as a zero so its absence cannot read as coverage |
| `result-type-0x33` | 1 | 1 | a different position; not this lane's |

### 0.4 The class, MEASURED off c2's own listing before a line of code

`work/w-fltret/probe/v3.cpp` through `c2rs listing` (`cl /FAsc`), and the
matching IL through `c2rs capture --keep-il`. **The IL draws the line the
emitter needs:**

| cell | source | after the last `bl` | IL after `4C` |
|---|---|---|---|
| `w_ff` | `float f(O*o){o->Poll(); return o->F();}` | **nothing** | `41 86 45 40` |
| `w_dd` | `double`←`double` | **nothing** | `41 88 85 41` |
| `w_df` | `float`←`double` | **`frsp fr1,fr1`** | **`2C 86 45 40 00`** `41 …` |
| `w_fd` | `double`←`float` | **nothing** | **`2C 88 85 41 00`** `41 …` |
| `w_post` | `return o->F()+1.0f;` | `lfs`+`fadds` (`.rdata` pool) | `33 86 4a 40 <8 B> 04 00 02 41 …` |
| `w_disc` | `void f(O*o){o->F(); o->Poll();}` | nothing; TU carries `_fltused` | `4C 4B` … `4C 4B` |

**So "the `41 <TYPE>` immediately follows the `4C`" is exactly the same-width
rule**, and it refuses `w_df` (which costs an instruction) and `w_fd` (which
does not) alike. This lane requires the immediate `41` and declines both
conversions by name (D6); `w_fd` is a **free** conversion this lane leaves on
the table, and it says so rather than smuggling it in.

`_fltused` is one `EXTRN` per TU in both listings and its placement is the
already-shipped rule (`c2_core::coff::writer`, after the first FP-touching
function's **complete** symbol group). **This lane predicts, and must show,
that no new insertion point is needed** (P7).

---

## 1. What this lane will ship, stated before it is written

A **reader admission only**, in `crates/c2-il`:

1. The **member** spelling of `parse_call_sequence_from`'s value-tail arm —
   `26 <method> B9 <recv> <T> 99 <T> 00 BD <ret T> 00 <id> <args> 4C 41 <T>`.
   Built from the *same* locators `eat_member_stmt_call` already uses
   (`eat_callee_push`, `mcall_tail::eat_receiver_this`, `eat_call_token`,
   `eat_call_args`), sharing one head reader rather than a second copy.
2. A **real** result type in the sequence's value tail — member and free
   spelling alike — with the FP-ness carried to
   `IlFunction::touches_floating_point` so the obj grows `_fltused`.
3. Whatever `crates/c2-core` needs to keep compiling (an exhaustive-match arm
   that emits **no instruction**), and nothing else.

`crates/c2-core`'s emitted bytes for this class are **the ones it already
emits**: c2's listing above shows the float and the int body are the same
instruction stream. If any *instruction* has to be added to `c2-core`, this
lane has mis-read the class and says so.

---

## 2. PREDICTIONS, in probability form

Scored `HIT / MISS / HALF`; each MISS is marked OPTIMISTIC or PESSIMISTIC.

**Registered direction: this lane registers OPTIMISTIC.** Board #770's streak is
optimistic predictions missing; w-callprice #2031 is its mirror (a
prior calibrated on seven artifact-rankings misfired on the eighth). This lane
has a `-whole` signal, a route split (§0.2) and c2's own listing (§0.4), which
is more evidence than either streak was calibrated on — so it registers up and
takes the hit if that is wrong.

| # | prediction | p |
|---|---|--:|
| **P1** | the admission converts **≥ 1** emitted function on the 878-TU workload (emitted census > 39,200) | 0.88 |
| **P2** | it converts **≥ 100** emitted | 0.62 |
| **P3** | it converts **≥ 400** emitted | 0.50 |
| **P4** | it converts **≥ 1,000** emitted (the int siblings carry it past R2's own row) | 0.22 |
| **P5** | `?SplitMs@Timer@@QAAMXZ` — the 434× function w-callprice hand-checked — is among the converted, checked BY NAME and not by a count | 0.70 |
| **P6** | **mismatch stays 0** at every level: 878 TUs, 312 fixtures at `/O1` and `/Ox`, all 18 gate lanes, the sweep and the cross | 0.93 |
| **P7** | `_fltused` for this class needs **no new insertion point** — the shipped "after the first FP-touching function's complete symbol group" rule places it byte-exact, verified against a reference obj in which the FIRST function is one of this class | 0.75 |
| **P8** | the **int** member value tail converts strictly **more** emitted than the float one | 0.55 |
| **P9** | TU match moves off **18** | 0.05 |
| **P10** | the free-function FP value tail (`result-type-0x41`, 810 bodies / **1** emitted) also converts, and moves the emitted census by **exactly 1** | 0.45 |
| **P11** | `#[test]` **DELTA** is **+9**, counted by name with `git grep -c '#[test]'` at both revs and never by subtracting totals | 0.30 |
| **P11b** | that DELTA is in **[+4, +16]** | 0.75 |
| **P12** | the `chained-then-type-real-whole` row (105 emitted) converts **0** — it is `mcall_chain`'s route and this lane does not touch it | 0.85 |
| **P13** | **≥ 1 unnamed refusal fires**, pre-armed at the two places named in §3 | 0.65 |
| **P14** | **≥ 1 of `wmcall_seq_neg.cpp`'s six cells is fully paid and must be RETIRED**, and the `_neg` fixture still grades `Port=NotImplemented` **before and after** — i.e. the fixture gate cannot see the change, which is w-bdnz's confounded-cell hazard in the form where it is *my* fixture that goes stale | 0.90 |
| **P15** | at least one of this lane's own `_neg` cells is **confounded** on first writing (fires on an earlier clause than the one it names), found by the committed-`Err` scratch probe and repaired | 0.55 |
| **P16** | every one of the 635 body-blocker keys and 614 emitted-blocker keys that this lane does **not** convert is byte-identical at tip — no re-key (w-mcall D7) | 0.80 |
| **P17** | the emitted conversion is **concentrated**: the single largest mangled name is **≥ 60 %** of the converted emitted column | 0.70 |

---

## 3. THE UNNAMED-REFUSAL BUDGET — **one**, pre-armed at two places

Both are where the last three lanes' unnamed refusals hid.

* **FENCE ORDER.** The member value-tail attempt has to sit *behind* the
  free-function `eat_call_head` probe, exactly as w-mcall's statement arm does,
  or a refusal that is a free-function refusal today gets re-keyed. Armed on:
  any key in the 635/614 maps changing that this lane did not convert (P16).
* **CLAUSE REACHABILITY.** The new real-result arm can be **unreachable**
  because `eat_return_plumbing`'s `result-type` gate fires first, or because the
  `-whole` census walk and the parser disagree about what finishes the body. A
  clause that never fires converts nothing and looks exactly like a clause that
  does — #2025's lesson at the third order. Armed on: a per-clause counter, run
  over the workload, that must be **non-zero** for every arm this lane adds.

A second, unbudgeted refusal is reported as a **MISS of the budget**, not
absorbed (w-park's streak).

---

## 4. DECLINE CLAUSES — each named AND sized

| # | declined | size, re-derived at base |
|---|---|---|
| **D1** | any `IlOp::Call` variant — a call as an operand. Inherited from w-mcall #1961 and w-callprice's own D1 | the two populations that want one (`-then-call-recv-load-and-deref-load-more` 2,183 emitted; `MessageTimer::~MessageTimer`'s 419) stay blocked |
| **D2** | any relaxation of `seq_call_arg_slots`' blanket `SlotArg::SymAddr` refusal. w-callprice #2026: an address-taken stack local wears the same `26 <sym>` a relocation does, and admitting it emits a relocation where c2 emits a frame offset — **wrong bytes, not a refusal** | `recv-object-*` = **10,144 emitted, 28.5 %** of the family. KEPT AS IS, not fenced, not touched |
| **D3** | a **chained** receiver anywhere in the sequence | `chained-*` 795 emitted in the family |
| **D4** | a **virtual** (`67`) call in any position | 296 emitted (w-callprice §5.2) |
| **D5** | the **chain route's** value tail — `chain-returned-body-does-not-end-at-the-call` | **1,472 bodies / 105 emitted**, all of `chained-then-type-real-whole` (§0.2) |
| **D6** | a `2C` **conversion** on the returned real value, in either direction | `w_df` costs `frsp` and `w_fd` costs **nothing** — the free one is declined too, because the fence that refuses the costly one cannot tell them apart without a width model this lane does not build |
| **D7** | an **FP post-op** (`return o->F() + 1.0f`) | `lfs` from the `.rdata` FP pool + `fadds`; `expr-call-in-expr-recv-load-then-type-real-lit-and-op-more` = 219 bodies |
| **D8** | a member call in a **guarded (W10)** or **early-return (W11)** sequence | w-mcall's exclusion, unchanged |
| **D9** | **re-keying any refusal this lane does not convert** | P16 is the measurement |
| **D10** | widening `IlBundle::functions()`, `PORT_CFG_CLASSES`, any whole-TU recognizer, or adopting any `DISCLOSURE.md` row | held |
| **D11** | quoting a **body** count as a population. Every number in the rung's price columns is an **emitted** count with its constructs beside it | w-callprice D2, inherited |

**Conditional, and registered as a prediction rather than a decline:** the
**discarded** FP result inside the sequence loop (`call-ret-fp`, **0 workload
bodies**, `w_disc` in §0.4). It is the literal `CallRet::discarded` obligation
the commission names. It ships **only if** it grades byte-exact on its own
fixture cell; if it does not, it becomes **D12** and is sized at **zero on this
workload**, which is a statement about the workload and not about the language.

---

## 5. The gate this lane will run before it reports

1. `cargo test --workspace --release` — pass count and `#[test]` DELTA by name.
2. `scripts/gate.sh --require-graded --jobs N` — 18/18 lanes, sweep row, cross
   row, `hatch-red`, `ladder-red`; **0 mismatch anywhere**.
3. `scripts/board_audit.sh` — 0 cited-but-rowless, 0 unresolved anchors, 0
   duplicates, 0 rows-behind-the-prose.
4. `cargo test -p c2-harness --release --test rung_registry` after
   `scripts/gen_rung_index.sh`.
5. Three-level neutrality, by NAME and by MAP, never by subtracting counts:
   878 TUs by name; all 251 `gap-metric` keys + both blocker maps; all 312
   fixtures at `/O1` **and** `/Ox`.
6. `work/w-fltret/probe/*.obj` and every `--jsonl` scan stay **uncommitted**;
   the committed `.txt` analyses are path-scrubbed and the scrubber **asserts**
   no `/home/` survives.
