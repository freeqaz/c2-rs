# w-mcall — PREREG

**Frozen before the first change to `crates/`.** Lane `w-mcall`, worktree branch
`wt-w-mcall` off master **`c5c94058`** (the w-value merge). Board rows
**#1960**–**#1979**.

Everything below was written after §0's base derivation and before one line of
`crates/` moved. Nothing in it is quoted from the commission: every number has a
script and a command behind it in §0.

---

## §0 — the base, re-derived rather than inherited

Six inherited survey prices have been wrong this week, so nothing here comes
from a rung, a board row or the commission's prose.

`c2rs gap` over the 878-TU dc3 workload at its own flags
(`work/w-mcall/scan_base.out`, tree `c5c94058`):

| | base |
|---|---:|
| TU match | **18** |
| mismatch · codegen-gap · port-error | **0 · 0 · 0** |
| vocab-gap · capture-fail | **853 · 7** |
| FRONTIER | **9** |
| factor A / B / C | **28 / 338 / 169** |
| `b-and-c` · `a-and-b-and-c` | **151 · 27** |
| `gap-metric` keys | **251** |
| function census | **711,494 / 2,463,443** |
| emitted census | **39,193 / 178,977** |
| workspace tests | **1,327 passed, 0 failed, 36 targets** |
| fixtures at `/O1` (307) | **154 match · 9 codegen-gap · 144 vocab-gap** |
| fixtures at `/Ox` (307) | **139 match · 17 codegen-gap · 151 vocab-gap** |

`work/w-mcall/fam.py` reproduces w-value's tip exactly — `expr-call-in-expr`
**423,925 bodies / 35,583 emitted** — so this really is the tree that rung was
written against.

The three first-blocker keys this lane's class sits under, counted by
`work/w-mcall/keys.py` (exact keys, not prefixes):

| key | bodies | emitted |
|---|---:|---:|
| `expr-call-in-expr-recv-load-whole` | **18,310** | **1,505** |
| `expr-call-in-expr-recv-load-then-call-nested-call-whole` | 1 | 1 |
| `call-token-0xB9` | 6 | 6 |

**1,512 emitted is the arithmetic ceiling on what this lane can move**, and the
real class is a strict subset of it: `-recv-load-whole` also holds W41's Class-B
value-live residue and the framed post-op shapes.

### §0.1 — the cells that define the class, at base

`work/w-mcall/probe/p2.cpp`, `c2rs census` at `/O1 /Oi /EHsc /GR`:

| cell | base verdict |
|---|---|
| `void free2(){ g1(); g2(); }` | **ok `call-sequence`** |
| `void mem2(S* s){ s->a(); s->b(); }` | GAP `expr-call-in-expr-recv-load-whole` |
| `void mem3(S* s){ s->a(); s->b(); s->a(); }` | GAP `…-recv-load-whole` |
| `void mem2arg(S* s,int x){ s->set(x); s->set(x); }` | GAP `…-recv-load-whole` |
| `void mem2two(S* s,S* t){ s->a(); t->a(); }` | GAP `…-recv-load-whole` |
| `void memfree(S* s){ s->a(); g1(); }` | GAP `…-then-call-nested-call-whole` |
| `void freemem(S* s){ g1(); s->a(); }` | GAP `call-token-0xB9` |

And the **free-function analogues emit byte-exact today**
(`work/w-mcall/probe/p3.cpp`, `c2rs diff` → `Port=Match`): `g1(a); g2(a);`
(Class B, one saved GPR), `g1(a); g2(b);` (Class B, two), `g1(a); g0();`
(Class A). Every word of `work/w-mcall/probe/p2.obj`'s member-call bodies is the
same shape with the receiver in argument slot 0.

---

## §1 — THE CLASS, and the commission's own spelling DECLINED (D1)

**The seam this lane opens is the reader's, and the commission's premise that
the LOWERING seam does not exist is wrong on this class.**

w-value's §4.2 states that the only thing that moves the 33,277 is *"a
member-call **lowering** — a call in an expression, which the emitter has no
representation for at all"*. That is true of a call that is an **operand** of
an enclosing expression — w-value's own 1,168 (3.2 %). It is **not** true of the
90.5 % with *nothing else in the expression*: those are calls in **statement**
position, and `BodyShape::CallSeq` / `crate::func::SeqCall` /
`c2_core::codegen::calls` already lower a sequence of statement-position calls
byte-exactly (§0.1's `p3.cpp` is the proof, at both Class A and Class B). What
the port cannot do is **read** a member call in a statement-sequence position —
which `mcall_tail.rs`'s own module doc already names: *"a body that does not end
at this call: a second statement after it is the Class A statement-call sequence
with a member call in it, which is a further rung"*.

> ### **D1 — NO `IlOp` CALL VARIANT.**
>
> Adding one would be a **second representation of a call** beside `SeqCall`,
> which carries a callee token, an argument slot list and a chain-link flag —
> everything an emitter needs and everything an `IlOp` variant would have to
> re-carry. `docs/GAPS.md` §6 instance #9 is one rule with two implementations
> and the corpus only ever exercising one; this project has paid for that four
> times. **Sized:** the population an `IlOp` call variant would serve is
> w-value's **1,168 emitted (3.2 %)** with a genuine expression construct behind
> the call — not the 33,277 — and it needs an operand-position lowering
> (a call result live in `r3` inside an arithmetic chain) that has **no capture
> in this repo**. Declined, with the number.
>
> If §4's measurement shows the shapes-layer route cannot reach the class at
> all, this clause is re-opened and the lane declines the rung instead of
> shipping an unfenced second call representation.

**THE CLASS, in port terms.** One production, one new acceptance:

```text
  seq := stmt_call+ tail                        (unchanged — BodyShape::CallSeq)
  stmt_call := ( 26 <callee> BD … )   4C 4B     free function   — ships today
             | ( 26 <method> B9 <recv> <ptr4> [2C <ptr4> 00] 99 <ptr4> 00 BD … )
                                      4C 4B     MEMBER CALL     — THIS LANE
```

The member call's receiver is appended to the argument list as **slot 0**, which
is exactly what `mcall_tail::try_parse_member_tail_call` already does for the
tail form and what `member_tail_call_puts_this_in_slot_zero` already pins. From
there the body is a `CallSeq` and **no byte of `crates/c2-core` changes**.

### The declines, each with its base size

| # | declined | why | size at base |
|---|---|---|---|
| **D2** | any receiver that is not a plain `B9 <tok> <ptr4>` load (optionally one class-preserving `2C`) — named object, chain, field, deref, `intrinsic 2113` this-adjust | each is a different receiver *production* with its own lowering, and `mcall_tail` already refuses each by name at the tail form | the `-recv-object-*` rows alone are **5,608 + 1,463 + 757 + 457 + 431 = 8,716 emitted**; `-recv-intrinsic-this-adjust-*` **828 + 803**; `-recv-field-*` **811 + 419 + 350** |
| **D3** | a member call in the sequence's **value tail** (`int f(S*s){ s->a(); return s->b(); }` → `SeqTail::CallValue`) | the tail's post-op region and the receiver's slot-0 marshalling have never been graded together | unsized at base — no key separates it; sized in §4 if the instrument is cheap, otherwise declared unsized |
| **D4** | a member call inside a **guarded** (W10) or **early-return** (W11) sequence | those two classes are Class A only and hoist the entry block; no cell crosses them with a receiver | `callseq-guard-*` / `callseq-early-*` keys, quoted in §4 |
| **D5** | a **virtual** call (`67`) as a sequence statement | w-value §4.3 counts it | **18 emitted** |
| **D6** | any change to `crates/c2-core` that emits a byte | the class is *defined* as "what `codegen::calls` already emits". If a byte has to be written, the class was mis-drawn — STOP and re-scope rather than widen | n/a |
| **D7** | re-keying a refusal | if the sequence parse fails after the member-call head is admitted, the body keeps **today's** census key byte for byte (the attempt runs on a scratch cursor and the production tag is re-armed). w-value #1942's finding — a re-key that makes the histogram less informative is not a measurement | n/a |
| **D8** | widening `IlBundle::functions()`, `PORT_CFG_CLASSES`, any whole-TU recognizer, or any `DISCLOSURE.md` adoption | out of scope | n/a |

---

## §2 — FENCE ORDER and clause reachability, pre-armed

w-park's finding (streak **7/10** after w-value #1945) is that the last unnamed
refusal hides in fence order. Two orderings are frozen here, before the code:

1. **The member-call arm is tried LAST in `parse_call_sequence_from`'s statement
   arm**, after the free-function `eat_call_head`. A free call and a member call
   share the `26 <tok>` head, so an arm that guessed "member" first would have to
   un-read it; trying the existing reader first leaves every free-function
   refusal key byte-identical by construction.
2. **On any failure of the member arm the cursor is restored AND the production
   tag is re-armed.** `prod_tag` is last-write-wins (`mod.rs`'s
   `prod_tag_is_the_seam_the_member_call_productions_write_against`), so a failed
   attempt would otherwise overwrite the member-call production's own tag and
   silently move the `prod` axis for bodies whose verdict did not change. **This
   is the pre-armed candidate for the budgeted unnamed refusal.**

**Budget: ONE unnamed refusal.** If a second turns up it is reported as a miss,
not absorbed.

**Clause reachability**: every `_neg` cell must be **probe-verified per cell** to
land on a *distinct* clause key. A `_neg` file whose six cells all land on one
key proves one clause and looks like six.

---

## §3 — Predictions, in probability form

| # | prediction | p |
|---|---|---:|
| **P1** | `mismatch` is **0** everywhere: 878 TUs, 307+N fixtures at `/O1` and `/Ox`, and every `gate.sh` row including the sweep and the cross | 0.90 |
| **P2** | TU match **18 → 18** — zero conversions on the workload | 0.90 |
| **P3** | FRONTIER stays **9**, same members by name | 0.85 |
| **P4** | emitted census rises by **≥ 200** (39,193 → ≥ 39,393) | 0.55 |
| P4b | …by **≥ 800** | 0.30 |
| P4c | …by **≥ 1** | 0.88 |
| **P5** | function census rises by **≥ 2,000** | 0.60 |
| **P6** | `expr-call-in-expr-recv-load-whole` falls by **≥ 100 emitted** | 0.70 |
| **P7** | `call-token-0xB9` goes **6 → 0 emitted** | 0.55 |
| **P8** | `codegen-gap` on the 878 stays **0** — the parser and the gate agree, #139/#1638 | 0.80 |
| **P9** | the positive fixture converts at **both** `/O1` and `/Ox`, so fixture match rises by ≥ 1 on each axis and no pre-existing fixture moves | 0.85 |
| **P10** | workspace test **DELTA +8** (#1749: a delta, never a total) | 0.40 |
| P10b | the delta is in **[+5, +12]** | 0.75 |
| **P11** | ≥ 1 refusal not named in this file turns up; **pre-armed on the §2.2 tag re-arm** | 0.75 |
| **P12** | D1 holds — no `IlOp` call variant ships, and `crates/c2-core` is untouched | 0.80 |
| **P13** | the `gap-metric` key count stays **251** | 0.70 |
| **P14** | `fnbyte-differs` does not rise (the class's bodies are `CallSeq`, which the instrument already grades) | 0.60 |

**Registered expectation on conversions, honestly:** the frontier has only two
members ever in this family (w-value §2: *"25 blocked emitted bodies over the 9
frontier TUs, of which two are in the `26` family"*), so **P2 says zero and the
lane's success is the seam, not a TU**.

---

## §4 — What will be measured, and with what

* 878-TU scan at base and tip, `--jsonl` both sides; `keydiff.py` (a key→value
  **map**, never a `diff`), `verdicts.py` (per-TU set **by name**),
  `metricdiff.py` (every `gap-metric` key accounted, including the unchanged).
* All 307+N fixtures at `/O1` **and** `/Ox`, compared **by name**.
* Both censuses, from the scan's own totals.
* The class's own cells: `fixtures/cpp/wmcall_seq.cpp` (positive, byte-exact
  against real `c2.dll` under wibo) and `fixtures/cpp/wmcall_seq_neg.cpp`
  (`_neg`, one distinct clause key per cell, probe-verified).
* Gate: `scripts/gate.sh --require-graded`, `cargo test --workspace --release`,
  `scripts/board_audit.sh`, `cargo test -p c2-harness --release --test
  rung_registry`. Hatch needles re-taken if partially paid, retired only if
  fully paid (w-park's precedent).

---

## §5 — What this lane will NOT do

No `IlOp` call variant (D1). No `crates/c2-core` byte (D6). No
`IlBundle::functions()` / `PORT_CFG_CLASSES` widening (D8). No `DISCLOSURE.md`
row. No reordering of `ops.is_empty()` relative to the sink poisons (#1538, and
w-value went in front of it rather than reorder it). No touching
`param-width-undetermined`, which is still `Main.cpp`'s head.
