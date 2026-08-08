# WB-G `wb-chooser` — the one-witness-per-side blockers

> **PROVENANCE — MIXED, and the split matters here.** The *mechanism* readings
> in §5 are **DISASSEMBLY-DERIVED** from Microsoft's `c2.dll` (image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0; `sha256sum ~/ghidra-projects/bin/c2dll`
> = `c80981…6258`, verified before the first VA was quoted) and are
> **navigation** until they earn a [`DISCLOSURE.md`](DISCLOSURE.md) row. The
> *rules* in §2–§4 are **BLACK-BOX**, derived from 27 manufactured cells graded
> by the real `c2.dll` under wibo, and need **no** disclosure row — which is the
> single most useful thing in this document for the follow-on code lane.
>
> Predictions frozen in [`WB_CHOOSER_PREREG.md`](WB_CHOOSER_PREREG.md) in two
> commits (`07f0e9ca` before the base re-derivation, `4091d837` before the first
> export grep and the first grid `cl.exe`). Scored in §7.

Base: master `9ed20248`. Lane branch `worktree-agent-a20932071db1f3a88`.
Grid sources: `docs/whitebox/grids/wb-chooser/`. Objs: `work/wb-chooser/`,
uncommitted.

---

## 1. The inherited description was a mis-copy, and the price it implied was zero

**Board #1770 and #1792 both say, in one clause:** *"`mmio`'s three clauses and
Biquad's FP two-plan both need a chooser with one witness of each side —
exactly what #1767's rule refuses."* That clause is the whole decline record.
It is the reason two frontier TUs have been unpriceable since 2026-08-08.

**It traces to `rungs/2026-08-08-w-cfg2.md` §2, and it does not say that.** That
table ranks the frontier by *what a lowering must build*, and its last two
relevant rows read:

| TU | fns blocked | what a lowering must BUILD |
|---|---|---|
| `Biquad.cpp` | **2** | **two plans**; FP indirect load/store …, a pooled-constant `lis` mid-body |
| `… mmio keygen_xbox` | 3/18 | **several plans each** — outside the brief's "ONE block plan" scope |

"**two plans**" is `Biquad.cpp`'s **blocked-function count** — it has two blocked
functions, so a lowering must build two block plans. "3" is `mmio.cpp`'s. The
row that names `mmio` says "several plans **each**", in the same sentence that
defines a plan as a *function*. **Neither phrase ever named a chooser, and
neither TU was ever measured for one.** w-osfinfo §10, where the clause was
minted, closes with the sentence *"no row here was compiled or disassembled by
this lane"* — it was a survey of published prices, and it says so.

So the honest statement of the inherited position is: **"`Biquad` has two
blocked functions and `mmio` has three"** — a fact, carrying no evidence
problem at all. The "one witness of each side" gloss was inferred from the word
*two*, and #1767's rule was then applied to a claim #1767 had never seen.

**This is deliverable 1's warning firing for the third time this week** (#1760:
a survey price wrong in one row each way; #1782: "one mechanism" was thirteen).
The failure mode is identical each time — a compressed phrase in a survey
paragraph is quoted forward as a measurement. §8 proposes the one-line
mitigation.

**What that does NOT mean.** It would be cheap and wrong to stop here. The
substance behind the worry — *does the port have to CHOOSE between two lowerings
in these TUs, on evidence too thin to fit?* — is a real question, and the two
reference objs answer it: **yes, three times over**, at choice points nobody had
named. §2–§4 are those three, each re-derived at base and each taken to ≥3
witnesses per side.

---

## 2. Choice point **M** — the park register (`mmio.cpp`)

### 2.1 What the base obj actually shows

Four values are parked out of an argument register at function entry.
c2 picks a **volatile** register for two and a **callee-saved** one for two,
and the callee-saved pick costs a `std r31,-16(1)` / `ld r31,-16(1)` pair:

| site | park | reg | live across |
|---|---|---|---|
| `mmioGetInfo` +0x0c | `mr 11,3` | **r11** VOL | nothing |
| `mmioSetInfo` +0x10 | `mr 31,3` | **r31** SAV | `bl memcpy` (external) |
| `mmioClose` +0x10 | `mr 31,3` | **r31** SAV | `bl mmioFlush`, then `bctrl` |
| `mmioClose` +0x14 | `mr 5,4` | **r5** VOL | `bl mmioFlush` **only** |

`mmioClose` is the separating witness and it is *already in the corpus*: two
values, same function, same call — one goes volatile and one goes callee-saved.
`mmioFlush` is a same-TU `__declspec(noinline)` leaf that compiles to
`li 3,0 ; blr`, and it is emitted **earlier in the same obj** (section 10 vs
`mmioClose`'s 14).

### 2.2 The grid — 10 registered cells, 7 unregistered

`docs/whitebox/grids/wb-chooser/m*.cpp`. Every cell is the same skeleton —
`int f(int a, …) { int r = <call>; return a + r; }` — so that `a` arrives in r3,
is live across the call, and the call itself wants r3. Which register `a` moves
to is the only free variable.

| cell | the call `a` is live across | c2 emitted | class | registered | verdict |
|---|---|---|---|---|---|
| **M1** | none | `a` never moves, stays r3 | VOL | VOL | **HIT** |
| **M2** | extern `ext` | `mr 31,3` + `std 31` | SAV | SAV | **HIT** |
| **M3** | same-TU clean leaf, defined **earlier** | `mr 10,3`, **no `std`** | VOL | VOL | **HIT** — R-M-A refuted |
| **M4** | the same leaf, defined **later** | `mr 10,3`, **no `std`** | VOL | **SAV** | **MISS** — see §2.4 |
| **M5** | same-TU leaf (earlier) that tail-calls an extern | `mr 31,3` + `std 31` | SAV | SAV | **HIT** |
| **M6** | indirect `bctrl` | `mr 31,3` + `std 31` | SAV | SAV | **HIT** |
| **M7** | extern, **two** live values | `mr 30,3` ; `mr 31,4`, `std 30` + `std 31`, frame 96→112 | SAV×2 | SAV×2, "r31 then r30" | **HIT** on class, **MISS** on order |
| **M8** | **two** calls to the clean earlier leaf | `mr 10,3` | VOL | VOL | **HIT** — R-M-A refuted again |
| **M9** | `mmioClose` reduced: `a` across leaf **and** `bctrl`; `b` across leaf only | `a`: `mr 31,3`+`std`; `b`: **stays in r4** | SAV + VOL | SAV + VOL | **HIT** |
| **M10** | same-TU callee (earlier) that is noinline and calls indirectly | `mr 31,3` + `std 31` | SAV | SAV | **HIT** |

Unregistered exploratory cells, scored separately and never as confirmations:

| cell | question | c2 emitted |
|---|---|---|
| **M11** | does the clean leaf's **linkage** matter? (`static`) | `mr 10,3` VOL — no |
| **M12** | leaf returns its argument rather than a constant | `mr 10,3` VOL — no |
| **M13** | leaf writes r3, r8, r9, r10, r11 and calls nothing | **`mr 7,3`** — see §2.3 |
| **M14** | two live values across the clean leaf | `mr 10,3`; the second **stays in r4** |
| **M15** | the caller also needs an addressing scratch | park r10, `lis 9` for the global, `mr 11,3` for a short temp |
| **M16** | leaf defined later, with a third function in between | `mr 10,3` VOL — order-independence again |

**M13 is the cell that turns a two-class rule into a mechanism.** Its callee
writes exactly r3, r8, r9, r10, r11. The caller passes four arguments (r3–r6)
and parks `a` in **r7** — the highest register that is neither an argument
register at that call **nor written by the callee**. c2 is not asking *"is this
callee clean?"*; it is reading the callee's **exact register footprint** and
allocating around it.

### 2.3 The rule, in port terms

> **M-RULE.** For each call in a body, c2 computes the callee's *register
> footprint*. For a direct call to a function defined **anywhere in the same
> translation unit** that makes no call of unknown footprint, the footprint is
> the exact set of registers that callee writes. For every other call —
> external, indirect (`bctrl`), or same-TU-but-itself-calling-unknown — the
> footprint is the whole volatile set.
>
> A value live across a call is placed in the lowest-cost register that is
> neither needed for argument passing at that call nor in the union of the
> footprints of the calls it is live across. If a volatile qualifies it is used
> and **no callee-saved register is touched, so no `std`/`ld` pair and no
> enlarged frame appear**. If none qualifies, the value goes to a callee-saved
> register.

Two sub-rules, each with its own witnesses, that the port needs to be
byte-exact rather than merely correct:

* **Coalescing beats allocation.** `mmioClose` parks `fuClose` in **r5** — not
  in the "next free volatile" — because r5 is the argument register its next
  consumer (the `bctrl` to `pIOProc(info, 4, fuClose, 0)`) wants. `M9`'s `b` and
  `M14`'s second value go one step further and are **never moved at all**: they
  already sit in the register the later call wants, and c2 knows the intervening
  callee does not write it. *Witnesses: 3 (base `mmioClose` r5, M9-b, M14-b).*
* **Which volatile, when a move is needed: r11 if the value does not cross a
  call, r10 if it does.** `mmioGetInfo` parks in r11 and crosses nothing;
  M3/M4/M8/M11/M12/M14/M15/M16 all cross a call and all pick r10, with r11 still
  free (M14 then *uses* r11 for a post-call temp, and M15 uses it for a
  short-lived one). Pooled-address bases in Grid B take r11 first, then r10
  (§3.2). **This is board #1762's open r11-vs-r10 question** — "the scratch
  register is the key; r11 vs r10 broke a walk keyed on r11 alone" — and the
  separating variable is *crosses a call*. *Witnesses: 6 for r11-no-cross
  (`mmioGetInfo`, B1, B2, B3, B4, B6), 8 for r10-crossing.* Marked `medium`:
  the grid establishes the correlation on 14 cells but this lane did not find
  the reservation in the disassembly (§5.4).

### 2.4 P1.3 is RETRACTED — the clobber knowledge is whole-TU, not emission-ordered

I registered, as the discriminating cell, that the same clean leaf **defined
after** the caller would force the callee-saved pick, because c2 would not yet
have emitted it. **M4 says `mr 10,3`: it does not.** M16 repeats it with a third
function interposed. Rival **R-M-C** (whole-TU knowledge, order-independent)
wins that cell, and per PREREG decline clause 3 the prediction is retracted
rather than hedged.

The objs also show *why*: in M4 the source order is `leaf` declared, `f`
defined, `leaf` defined — and the obj emits **`leaf` as section 5 and `f` as
section 6**. c2 does not emit in definition order, so "already emitted" was
never the right frame. The port must treat the callee footprint as a **whole-TU
property**, which is *easier* to implement than what I predicted.

### 2.5 P1.5 is RETRACTED — the callee-saved set is top-down, the assignment is not

I registered "r31 downward". **M7 emits `mr 30,3` before `mr 31,4`**: the *set*
is the top N of r14…r31 (so a two-value function uses r30 and r31, matching
`undname`'s `std r30/r31`), but the *assignment within the set is ascending by
first park*. The prologue then saves in ascending register order
(`std 30,-24(1)` before `std 31,-16(1)`), and the frame grew 96 → 112.
An emitter that gets this backwards produces the right instructions in the wrong
registers and every subsequent word is wrong.

### 2.6 Witness count — #1767's bar

| side | base | registered grid | total |
|---|---|---|---|
| **volatile** | 2 (`mmioGetInfo` r11, `mmioClose` r5) | M1, M3, M4, M8, M9-b | **7** |
| **callee-saved** | 2 (`mmioSetInfo`, `mmioClose` r31) | M2, M5, M6, M7 (×2 sites), M9-a, M10 | **9** |

**≥3 per side, with every grid outcome predicted before its cell was compiled,
and a mechanism reading consistent with all sixteen.** #1767's bar is met for
choice point M. It was met by *manufacturing* the witnesses, which is exactly
the remedy the rule anticipates: #1767 refuses a 2-point fit, not a 16-point one.

Discriminating cells actually delivered: **3**, the asserted minimum. M3
separated M-HYP from R-M-A (M-HYP won); M4 separated M-HYP from R-M-C (**R-M-C**
won); M1-vs-M2 separated every liveness hypothesis from R-M-D (R-M-D refuted —
the same formal in the same slot takes r3 in M1 and r31 in M2).

---

## 3. Choice point **B** — the pooled-constant `lis` placement (`Biquad.cpp`)

### 3.1 What the base obj shows

`?SetCoefficients` uses two `.rdata` float pools in their own COMDATs, so
#1786's shared-high-half hoist cannot apply — the halves are relocations
(`REFHI`/`REFLO`), not constants. The two `lis` land in different places:

| pool | `lis` | `lfs` | uses |
|---|---|---|---|
| `__real@00000000` | **+0x00**, above the `cmplwi` at +0x04 | +0x08 | 4 in the then-arm, 2 after the join |
| `__real@3f800000` | **+0x10**, first word of the then-block | +0x24, five words later | 1 |

### 3.2 The grid — 7 cells

| cell | shape | c2 emitted | verdict |
|---|---|---|---|
| **B1** | used only in the then-arm | `cmpwi`, `bclr`, then **`lis` = first word of the then-block**, `lfs` five words later at the use | **HIT** — R-B-A and R-B-B both refuted in one cell |
| **B2** | used in **both** arms | one `lis` hoisted to the **entry block**, but placed *between* the compare and the branch: `cmpwi`(0), `lis`(4), `bt`(8) | **HIT** on the block, **MISS** on the word |
| **B3** | then-arm **and** after the join | `lis`(0), `cmpwi`(4), `lfs`(8), `bt`(0xc) — **Biquad's exact shape, out of sample** | **HIT** |
| **B4** | only after the join | `lis` at **0x18, the first word of the join block**, below the branch | **HIT** — R-B-B refuted |
| **B5** | two constants, both then-arm only | two `lis` at the top of the then-block in **first-use order**, in **r11 then r10**, interleaved with the arm's other words rather than adjacent | **HIT** on order, partial on adjacency |
| **B6** | one constant used **twice** | **one `lis`, ONE `lfs`**, two `stfs` reusing `f0` | **MISS** — I registered two `lfs` |
| **B7** | used only inside a loop | **no pool at all** — `lis 10,0x3fc0` materialised in a GPR and stored with `stwu` | **VACUOUS** for this grid; see §4 |

### 3.3 The rule, in port terms

> **B-RULE.** One `lis` per pool symbol per function, emitted at the **top of
> the earliest basic block that dominates every use of that symbol**. The `lfs`
> is emitted **at the use**, not with the `lis`, and one `lfs` serves every use
> the FPR stays live across (B6). When two pools share a dominating block their
> `lis` are emitted in **first-use order**, taking the addressing scratch r11
> then r10.

And the correction B2 forced, which the base obj alone could not have given:

> **B-RULE-2 (compare/branch separation).** Within the entry block, **exactly
> one instruction sits between a compare and the branch that reads its CR
> field**, if one is available to fill the slot; the rest of the hoisted words
> go above the compare. Witnesses: `Biquad` (`cmplwi`, `lfs`, `bf`), B2
> (`cmpwi`, `lis`, `bt`), B3 (`cmpwi`, `lfs`, `bt`) — one filler each; B1, B4,
> B5, `mmioGetInfo`, M9 — nothing hoisted into the entry block, nothing to fill
> with, compare and branch adjacent. **Six witnesses filled, five empty.**

This is why the base obj's `lis`-at-word-0 looked like "hoist above the
compare": in `Biquad` *two* words were hoisted (the `lis` and the `lfs`), one
took the separation slot and the other was pushed above the compare. In B2 only
the `lis` was hoisted and it took the slot. **A port that transcribed
"pooled `lis` is the first word of the function" from `Biquad` alone would be
wrong on B2, and this is precisely the generalisation error the whole lane
exists to catch.**

### 3.4 Witness count — #1767's bar

| side | base | grid | total |
|---|---|---|---|
| **entry-block dominator** (hoisted past a branch) | 1 (`__real@00000000`) | B2, B3 | **3** |
| **block-local dominator** (arm or join) | 1 (`__real@3f800000`) | B1, B4, B5 (×2), B6 | **6** |

**≥3 per side. #1767's bar is met for choice point B.**
Discriminating cells delivered: **2**, the asserted minimum — B1 (kills R-B-B)
and B4 (kills R-B-B a second time in the opposite direction, and kills R-B-A,
whose `lis`-adjacent-to-`lfs` prediction fails in B1 by five words).

---

## 4. Choice point **B′** — the divisor load-order flip, and a fourth chooser found by accident

### 4.1 B′ — and my registered pessimism was wrong

`Biquad`'s else-arm issues five `fdivs` by the same divisor `flts[3]`, reloading
it every time (there is **no CSE of the reload**). Four load **divisor then
dividend**; the fifth loads **dividend then divisor**. Registered **P2.5**: the
flip is on the last division of the run. Registered **P2.6**, explicitly as the
pessimistic call: P2.5 would MISS and B′ would be the "not mechanism-driven"
finding the success floor allows.

**P2.5 is a HIT, 4 for 4, out of sample.** Runs of 2, 3, 4 and 6 divisions each
produce exactly one flip and it is always the last:

| cell | run | flips at |
|---|---|---|
| B′1 | 2 | division **2** |
| B′2 | 3 | division **3** |
| B′3 | 4 | division **4** |
| B′4 | 6 | division **6** |
| base `Biquad` | 5 | division **5** |

> **B′-RULE.** When a value is reloaded as a common subexpression across a run
> of statements, its load is emitted **first** within each statement of the run
> **except the last**; in the last statement — the reload's final use — the
> operands are loaded in **source order** (numerator, then denominator).

Witnesses: **5** of the flip side, **15** of the non-flip side. #1767's bar is
met a third time. **P2.6 is scored a MISS in the pessimistic direction**, which
board #770's streak makes the rarer and more useful kind.

### 4.2 The float-constant materialisation chooser (unregistered, found by B7)

B7 asked where the `lis` goes when the constant is used inside a loop. c2's
answer was that **there is no `lis`, no pool, no `.rdata` COMDAT, no relocation
and no FPR at all**: it emitted `lis 10,0x3fc0` — the bit pattern of `1.5f` —
and stored it with an integer `stwu 10,4(11)` in a `bdnz` loop.

That is a **fourth chooser**, on the same axis as #1786's: a float constant
whose low 16 bits are zero can be built in a GPR with one `lis`, and when the
only consumer is a store c2 does exactly that. The same `1.5f` in B1–B6 is
pooled, so the predicate is not "low half is zero" alone. This lane did **not**
grid it — it is named, priced as unmeasured, and left as the obvious next cell
set (vary: low half zero vs not; consumer is a store vs an arithmetic operand;
inside a loop vs not; one use vs many). **No claim is made about it beyond the
one obj.**

---

## 5. The mechanism in the binary (VAs) — what it does and does not settle

Image verified `c80981…6258` before any address below was quoted. Labels filed
in `docs/whitebox/labels/W-COLOR.tsv`; `docs/whitebox/c2_functions.tsv`
regenerated per method doc §5.

### 5.1 The assigned-register field is operand `+0x1c`, and register numbers are `r+1`

Three independent passes read the same field. `FUN_10bfebf7` reads
`*(u32*)(op[7] + 0x1c)` and bounds it to `0x0f ≤ n < 0x21`; `FUN_10b2ceb7`
(color.c) reads `*(u32*)(op[7] + 0x1c)` when deciding coalescing; `FUN_10b55eae`
(globregs.c) reads `*(u32*)(reg + 0x1c)` as the bitset element. The bound
`0x0f…0x20` is **r14…r31**, so the encoding is `n = r + 1`. Confidence **high**.

### 5.2 `10bfebf7` is a SCAN, not a decision — the prologue is a consequence

This corrects the existing `W-FRAME` label in one word. `FUN_10bfebf7` walks the
block chain (terminating on tags `0x2f1`/`0x2f6`), and for every operand of kind
1 whose class nibble `((u16 at op+10) >> 12)` is in `{1,2,3,4}` it ORs in
`1 << (n-1)` when `n` is in the callee-saved range. **The prologue saves whatever
the allocator already assigned.** The volatile-vs-callee-saved *choice* — §2.3's
M-RULE — is made upstream and is not readable at this address. Everything the
`wb-frame` lane's `prolog-flags` reading said about *when* a frame opens is
downstream of it too. Confidence **high**.

### 5.3 The callee-saved range constant, and its one runtime narrowing

`0x0f…0x20` (r14…r31), narrowed to `0x12…0x20` (**r17…r31**) whenever
`DAT_10c2e980 != 0` — the same datum, the same arithmetic
`(-(u32)(DAT_10c2e980 != 0) & 3) + 0xf`, at **eight** sites: `10bb4e9e`,
`10bffdd4`, `10bffe59`, `10c015a4`, `10c07c6c`, `10c0b8ca`, `10c113f3`, and
inside `10b2ceb7` where it also **refuses to coalesce into register numbers
0x0f…0x11** (r14–r16). Every dc3 obj this project grades was compiled with the
datum clear (r31 and r30 are in use in `mmioSetInfo` and M7), so the narrowed
range is a mode the corpus does not exercise. Confidence **high** on the
instructions, **unknown** on what sets the datum.

### 5.4 What this lane did NOT find, stated so it is not re-searched

* **The interprocedural clobber consult is UNLOCATED.** §2's M-RULE is
  established on 16 objs, but the code that records a callee's register
  footprint and unions it at a call site was not found in the time this lane
  had. Searched and eliminated: the prologue chain (`10bfebf7`/`10bfec72`/
  `10bff507` — all consumers), `globregs.c`'s live-set builder (`10b55eae` —
  builds per-block sets, no callee field), and `color.c`'s coalescer
  (`10b2ceb7` — reads register numbers, not symbols). `FUN_10b26eda` has **206
  call sites** and was not enumerable at this budget; that enumeration, filtered
  to the `regasg.c` range around `FUN_10bc58d5`, is the next probe. **`unknown`,
  per method doc §5, is the value this row keeps.**
* **The r11/r10 reservation is UNLOCATED** in the disassembly. §2.3's second
  sub-rule stands on 14 objs and is marked `medium` for that reason.
* **No mechanism reading in this section was used to derive any rule in §2–§4.**
  The rules were frozen and graded first; §5 is checked *against* them, and
  where it is silent the rules stand on the objs alone. That ordering is the
  whole point of the campaign's method, and it is why the reading being
  incomplete does not weaken the deliverable.

---

## 6. Pre-drafted DISCLOSURE rows

**The most important line in this section: §2–§4's four rules need NO row.**
They were derived from manufactured cells graded by the real compiler, not from
the disassembly, so a code lane can ship M-RULE, B-RULE, B-RULE-2 and B′-RULE
with no weakening of the clean-room claim. Only the §5 constants are
disassembly-derived, and only if a code lane copies them rather than deriving
them from the public PowerPC EABI.

| draft | address | what would be adopted | needed when |
|---|---|---|---|
| **D-CH-1** | `10bfebf7` | the callee-saved register range `0x0f…0x20` in the `r+1` encoding = r14…r31, and its narrowing to `0x12…0x20` under `DAT_10c2e980` | **only if** the port hardcodes the narrowed r17…r31 mode. Plain r14…r31 is public PowerPC EABI and needs no row — say which source was used |
| **D-CH-2** | `10b2ceb7` | the copy/`mr` opcode set `{0x270, 0x272, 0x293, 0x7b}` and the assigned-register field at operand `+0x1c` | only if the port's IL-side coalescer keys on those opcode numbers |
| **D-CH-3** | `10b26ecd` `10b26eb2` `10b26eda` `10b26efb` `10b26f37` `10b27290` | the bitset primitive map | navigation only; no adoption expected |

Per campaign rule 3 these are **drafts**. If the follow-on code lane does not
carry one, no row is added to `DISCLOSURE.md`.

---

## 7. PREREG score

| # | registered | outcome |
|---|---|---|
| **P0.1** | the `mmio` choice point is `memcpy` inline-vs-call | **MISS.** Both `memcpy(…,72)` sites do emit `bl memcpy`, but that is not a *choice point for this TU* — there is no inline witness anywhere in it. The real chooser is the park register |
| **P0.2** | "three clauses" is a blocked-function count, not three choosers | **HIT**, exactly |
| **P0.3** | the `Biquad` choice point is the common-divisor division plan; c2 emits five `fdivs` | **HIT on the emit** (five `fdivs`, no reciprocal), **MISS on the identification** — "two plans" was the blocked-function count, and the division run turned out to hide a *different*, real chooser (§4.1) |
| **P0.4** | the two pools are `0.0f` and `1.0f` and are the only float constants | **HIT**, exactly (`__real@00000000`, `__real@3f800000`) |
| **P0.5** | **pessimistic headline**: ≥1 inherited description is materially wrong | **HIT.** Both are, and from the same mis-copy |
| **P0.6** | the "one witness per side" count is itself wrong; `mmio` supplies 2 on the call side | **HIT on the substance, by a different route** — `mmio` supplies 2+2 on the park chooser, not on a memcpy chooser |
| **P0.7** | #1767's own chooser (`slwi`/`mulli`) is neither of these; there are three, not two | **HIT** |
| **P1.1** | M-HYP survives its grid | **HIT**, 16/16 objs, with P1.3 amended out |
| **P1.2** | the interprocedural clause is real | **HIT** — M3, M8, M9-b, M11–M16 |
| **P1.3** | the clobber tracking is emission-order-sensitive | **MISS — RETRACTED** (§2.4). R-M-C wins: whole-TU, order-independent |
| **P1.4** | an indirect call always forces callee-saved | **HIT** — M6, M9-a, M10, base `mmioClose` |
| **P1.5** | callee-saved allocated r31 downward | **MISS — RETRACTED** (§2.5). The *set* is top-down; the *assignment* is ascending by first park |
| **P2.1** | B-HYP survives: single-arm ⇒ arm top; both arms ⇒ entry, above the compare | **HIT on the block, MISS on the word.** B2 puts the `lis` between compare and branch; B-RULE-2 is the correction |
| **P2.2** | the `lfs` stays at the use even when the `lis` is far away | **HIT** — B1, five words |
| **P2.3** | two pools ⇒ two `lis` in first-use order | **HIT** — B5, and it also gave the r11-then-r10 scratch order |
| **P2.4** | *optimistic*: a loop-only pool gets its `lis` in the pre-header | **VACUOUS** — c2 emitted no pool at all (§4.2). Placement was in the pre-header, but of a materialised constant, so the cell does not score |
| **P2.5** | the divisor flip is on the last division, at runs 2/3/4/6 | **HIT**, 4/4 out of sample |
| **P2.6** | *pessimistic*: P2.5 will MISS and B′ is not mechanism-driven | **MISS**, in the pessimistic direction |

**18 scored: 12 HIT, 4 MISS (2 of them retractions), 1 partial, 1 vacuous.**
Board #770's streak gains **one pessimistic miss (P2.6)** and **one pessimistic
hit (P0.5)**. Both of this lane's outright retractions (P1.3, P1.5) are on the
*details* of a rule whose *class* survived — which is the failure mode a grid is
supposed to produce, rather than the one where a headline collapses.

---

## 8. What #1770 and #1792 should now say, and the one mitigation worth taking

`mmio.cpp` and `Biquad.cpp` are **not** blocked by an evidence shortage. They are
blocked by **three lowering rules the port does not implement**, each of which
now has ≥3 witnesses per side and a stated rule: the park register (M-RULE), the
pooled-`lis` placement (B-RULE + B-RULE-2), and the CSE reload order (B′-RULE).
Their prices are engineering prices, not evidence prices, and #1767 does not
refuse any of them.

The mitigation, one line, aimed at the failure that has now fired three times
(#1760, #1782, this lane): **a survey paragraph that re-states another rung's
price must quote the rung's own words, not paraphrase them.** All three errors
were paraphrases — "one mechanism" for a thirteen-item list, "two plans" for a
blocked-function count. The rungs themselves were accurate every time.

---

## 9. What this lane did NOT do, so it is not re-discovered

* **It shipped nothing into `crates/`.** Explicitly out of scope; the follow-on
  code lane carries the DISCLOSURE drafts in the same commit if it uses §5.
* **It did not grid the float-materialisation chooser** found by B7 (§4.2). One
  obj, named, not priced.
* **It did not locate the interprocedural clobber consult** (§5.4), and says so
  rather than guessing from `FUN_10b26eda`'s 206 call sites.
* **It did not re-open `memcpy`.** P0.1's guess was wrong and `mmio`'s two
  `memcpy` sites are ordinary `bl`s at size 72 — consistent with the `WB-C`
  lane's measured boundaries, and its question, not this one's.
* **It did not touch #1767 or #1786.** Both rules are correct and both were
  correctly applied to what their own lanes measured. What is corrected here is
  the *forwarding* of them onto two TUs neither lane had looked at.
