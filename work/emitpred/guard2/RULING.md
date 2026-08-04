# guard2 — Phase B ruling on six axes2 cells

Agent `guard2`, lane `w-emitpred`. Phase-A derivation frozen and committed at
**`e8c6fb7`** *before* any truth artifact was read; this file was written after.

**Provenance.** The lead audited every file I read and every command I ran
before I derived anything: worktree `CLAUDE.md`, the six `cell.cpp` sources,
`docs/PHASE7_PLAN.md` §2, the prereg. My one directory walk used
`find -type f`, which lists names without opening them, and **none of the six
cell directories contains a `spec.json`** — each holds exactly one file,
`cell.cpp` — so that hazard never arose. I ran no `git log` and compiled
nothing in Phase A. Recorded here so a later reader does not have to re-open it.

**Normalization applied**, per the rule granted before scoring: `QAA`/`UAA` vs
`QAE` and `?A@` vs `?A0x…` are spelling differences and score as neither a hit
nor a violation in either direction. Two applications, both declared:

* My Phase-A guess `QAA`/`UAA` (PPC `__cdecl` members) was **correct** as
  spelled; nothing to normalize.
* My Phase-A guess `?A@` for the unnamed namespace was **wrong**; this
  toolchain spells it `?A0x########`. `a4_06`'s entity `S::m` is therefore
  scored as a match: my `?m@S@?A@@QAAHH@Z` ≡ truth's
  `?m@S@?A0xe6264bf0@@QAAHH@Z`.
* **Instrument note:** in `a4_06` the obj and the `/FAsc` listing carry
  *different* hashes for the same entity (`?A0xe6264bf0` vs `?A0x596d3a50`) —
  they are separate compiles and the hash is per-compilation. Any reader that
  cross-checks obj against `.cod` must normalize `?A0x########` to a wildcard or
  it will report a spurious disagreement. `readings_agree` was `true` for all
  six cells.

## 1. Per-cell table

Sets are entity sets after normalization. "mine" = the frozen `e8c6fb7`
derivation; "axes2" = `axes2/PREDICTIONS.md`; "truth" = `observed.json`
`code_leaders` (obj `.text` COMDAT leaders, the pre-registered ground truth).

| cell | my derivation | axes2's | truth | ruling |
|---|---|---|---|---|
| `a2_04` | **3** — `anchor`, `H<int>::a`, **`H<int>::b`** | **3** — identical, same clause | **2** — `anchor`, `H<int>::a`; **no `H<int>::b`** | **VIOLATION CONFIRMED** |
| `a3_01` | **7** — `anchor`, `??0D`, `??0A`, `??0B`, `A::f`, `D::g`, `B::g`; **no `??_G`**; +1 `[GAP]` thunk | **10** — same 7 **+ `??_GD`, `??_GA`, `??_GB`** | **7** — exactly my seven | **AMBIGUOUS** |
| `a3_02` | **7** — `anchor`, `??0D`, `??0A`, `??0B`, `A::f`, `D::h`, `B::g`; no `??_G` | **10** — same 7 + three `??_G` | **7** — exactly my seven | **AMBIGUOUS** |
| `a3_08` | **6** — `anchor`, `??0D`, `??0A`, `A::f`, `A::only_a`, `D::g`; no `??_G` | **8** — same 6 + `??_GD`, `??_GA` | **6** — exactly my six | **AMBIGUOUS** |
| `a4_05` | **1** — `anchor` only | **4** — `anchor`, `V::f`, `V::g`, `V::~V` | **1** — `anchor` only | **AMBIGUOUS** |
| `a4_06` | **2** — `anchor`, `S::m`; **not** `S::inl` | **3** — same 2 **+ `S::inl`** | **2** — `anchor`, `S::m` | **AMBIGUOUS** |

**Confirmed violations promoted from my six: 1** (`a2_04`, axis **A2**).

On the other five my derivation matched truth **exactly, entity for entity**,
while axes2's did not — which under the pre-registered rule is `AMBIGUOUS`, not
a violation, and I have not resolved any of them by pointing at truth. Note the
direction: every one of those five is axes2 **over-**predicting relative to both
me and truth, so promoting them would have inflated the lane's headline on a
reading truth rejects.

## 2. REACH vs GAP, per cell

| cell | failure class |
|---|---|
| `a2_04` | **REACH failure.** §2 produces a definite prediction (R1 root ⇒ emitted unreferenced) and it is wrong. Not a GAP: the symbol is ordinary and §2 speaks directly to it. |
| `a3_01`, `a3_02`, `a3_08` | **No failure of either class against my derivation.** The `??_G` divergence is a *statement* defect in the VT rider, not a REACH failure, because §2-as-written admits the conditional reading that truth confirms. |
| `a4_05`, `a4_06` | **No failure of either class against my derivation.** Statement defect in R1's anon-ns clause scope. |

**There are zero GAP failures in my six.** Every symbol in every one of the six
truth sets is derivable from §2's text; nothing appeared that §2 has no clause
capable of producing. The `MATCH on §2's domain + GAP` filing the lead asked me
to keep available therefore **does not apply to any of my cells**, and I am not
recording one. My single `[GAP]` prediction — the adjustor thunk in `a3_01` —
was **refuted**; see §4.

## 3. Do A3 and A4 break?

**On my cells, no — and A2 does, which is not the axis anyone was watching.**

* **A3 — does not break on `a3_01`, `a3_02`, `a3_08`.** Zero confirmed
  violations; three `AMBIGUOUS`. All three are the *same* statement defect.
* **A4 — does not break on `a4_05`, `a4_06`.** Zero confirmed violations; two
  `AMBIGUOUS`, again one shared statement defect.
* **A2 — breaks, on `a2_04`.** One confirmed violation, independently
  re-derived.

**Scope limit, stated plainly:** I ruled on six cells. A3 has eight and A4 has
nine; the remainder (`a3_03`–`a3_07`, `a4_01`–`a4_04`, `a4_07`–`a4_09`) are
guard 1's and I have not read its ruling. **"A3 and A4 do not break" is a claim
about my five A3/A4 cells only**, and the axis verdicts must be composed with
guard 1's before either axis is declared clean.

I decline to compute the lane headline. The leaked arithmetic that would have
let me (§6) proved false, so I report my contribution — **+1 confirmed
violation, on A2** — and leave the total to the lead.

## 4. The two statement defects, with the repair truth confirms

Both `AMBIGUOUS` clusters are one clause each, and in both the *content* of §2
is right while its *wording* is loose. That is a materially better outcome for
`#161` than a violation: it is a rewrite, not a refutation.

### 4a. The VT scalar-deleting-destructor rider (a3_01, a3_02, a3_08)

§2: "a kept constructor of C keeps C's vtable, whose slots force **every**
virtual of C plus the synthesized scalar-deleting destructor, called or not."

axes2 read the rider as unconditional (its `VT-dtor`, explicitly declared as a
stretch) and predicted `??_G` for classes with non-virtual destructors. I ruled
in Phase A (`R-a`) that the rider is **conditional on the class having a virtual
destructor**, on the grounds that "whose slots" names the slots as the forcing
mechanism and "**the** synthesized scalar-deleting destructor" is a definite
description that presupposes the entity exists.

**Truth confirms the conditional reading, and not only on my cells.** Across the
A2/A3 truth I inspected for attribution, `??_G` appears in exactly the classes
with a virtual destructor and nowhere else:

* `??_G` present, dtor virtual (`??1…@UAA@XZ` also emitted): `a2_02`, `a2_05`,
  `a2_08`.
* `??_G` absent, dtors non-virtual: `a3_01`, `a3_02`, `a3_08` — my three.

**Repair:** "…force every virtual of C, including the destructor when it is
virtual — for which the slot holds the synthesized scalar-deleting destructor
(`??_G`) rather than `??1` — called or not."

### 4b. The anon-namespace root clause's scope (a4_05, a4_06)

§2 R1: "…and anonymous-namespace functions not declared `static`".

axes2 read this wide (`A4-wide`: any function whose enclosing scope chain passes
through an unnamed namespace, including in-class members). I ruled in Phase A
(`R-b`) that it reaches namespace-scope and out-of-line definitions only, because
R1's head clause characterises roots as the strong-linkage-like class and the
member item is qualified "*any **out-of-line** definition*".

**Truth confirms the narrow reading, and the two cells separate the two
sub-clauses cleanly:**

* `a4_06`: the out-of-line `S::m` **is** emitted unreferenced (R1's out-of-line
  sub-clause holds for an internal-linkage member); the in-class `S::inl` is
  **not**. Exactly the split R-b predicts.
* `a4_05`: an unnamed-namespace polymorphic class with **no object constructed**
  emits **nothing** — not `V::f`, not `V::g`, not `V::~V`. VT idle, and the
  anon-ns clause does not reach in-class members.

axes2 pre-registered this outcome's grading itself: "If truth = 1, §2's anon-ns
root clause is over-broad as written and the defect is one of *statement*, which
… is graded AMBIGUOUS rather than VIOLATION unless the independent re-derivation
agrees with A4-wide." It does not. **AMBIGUOUS by axes2's own pre-registered
rule**, not by a judgement I made after seeing truth.

**Repair:** "…and anonymous-namespace functions not declared `static`, where
this reaches namespace-scope definitions and out-of-line member definitions but
not in-class (implicitly inline) member definitions."

## 5. The confirmed violation, characterized

`a2_04` — `template <> int H<int>::b(int x) { return x-5; }`, never referenced,
is **not** emitted. Both derivations independently made it a root by R1's "*any
out-of-line definition* (member, static member, virtual)", both citing that
clause by name and predicting the same three-name set. **REACH failure of R1.**

I checked the neighbouring A2 truth to attribute the break rather than leave it
as "R1 is broken", and it is sharply localized — R1's out-of-line clause is
**correct everywhere else it was tested**:

| cell | out-of-line construct | emitted unreferenced? |
|---|---|---|
| `a2_03` | explicit specialization of a **function template**, namespace scope | **yes** — `??$cand@H@@YAHH@Z` |
| `a3_07` | out-of-line **virtual** member definitions, no object constructed | **yes** — `A::f`, `D::g` |
| `a2_09` | member-level explicit **instantiation** definition | **yes** — `?b@?$H@H@@…` (but that is R2, a different clause) |
| **`a2_04`** | explicit **specialization of a member of a class template** | **no** |

So the failing case is precisely **an explicit specialization of a member
function of a class template**, which this toolchain treats as reference-gated
(COMDAT-like) rather than as a strong out-of-line definition — while a
namespace-scope explicit function-template specialization (`a2_03`) is *not*
reference-gated. R1's parenthetical "(member, …)" is what over-reaches, and it
over-reaches only into that one construct.

**Honest mitigation, recorded rather than used to demote:** both derivations
independently flagged the same rival reading in advance (mine as R-c's rejected
alternative, axes2's as its `a2_04` STRETCH, both predicting 2), and truth
matches that rival. The pre-registered rule keys on the *primary* derivations,
which agreed, so the ruling is **VIOLATION CONFIRMED** and I am not demoting it
— demotion here would be exactly the goalpost-move in the deflationary
direction. But a reader should know the defect is a narrow, pre-identified
over-reach with a stated repair, not a surprise.

**Detectability, for the R3 `Unknown ⇒ refuse` question:** the breaking
condition is syntactically obvious from the c1xx side — a `template <>` prefix
on an out-of-line member definition of a class template. It needs no obj-side
observable. I did not build a detector, so per the prereg this is an
*assertion*, not the demonstrated detectability that would unblock
SHIP-CANDIDATE.

## 6. My advance calls, scored

Seven were registered in the frozen derivation. **Five held, two failed**, and I
am reporting the failures first.

| # | call | outcome |
|---|---|---|
| 6 | `a3_01` contains an adjustor thunk, tagged `[GAP]` | **REFUTED** — see §7 |
| 7 | unnamed namespace spells as `?A@` | **REFUTED** — it is `?A0x########`; normalization, not a violation, per the rule granted in advance |
| 1 | `a2_04`'s `?b@…` **is** emitted | **FAILED AS A PREDICTION ABOUT c2, AND THAT IS THE POINT** — this is the confirmed violation; the call was §2's, made faithfully, and its failure is the result |
| 2 | `a4_06`: `?m@…` present, `?inl@…` absent | **HELD**, both halves |
| 3 | `a4_05` contains exactly one code symbol | **HELD** |
| 4 | no `??_G` in `a3_01`/`a3_02`/`a3_08` | **HELD**, and corroborated by the converse on `a2_02`/`a2_05`/`a2_08` |
| 5 | `a3_01`: `?g@B@@UAAHH@Z` present despite being overridden | **HELD** |

**R-d, the call I named in advance as my highest-risk, held.** I committed in
Phase A to propagation reaching implicitly-defined non-trivial special members,
so that a kept `D::D()` keeps `A::A()` and `B::B()`, which then feed the vtable
rule. Truth: `??0A@@QAA@XZ` and `??0B@@QAA@XZ` are emitted in all three A3
cells, and in `a3_01` `?g@B@@UAAHH@Z` **is** emitted although `D` overrides it
and it occupies no slot of either of `D`'s vtables. The mechanism I gave is
directly visible in the obj: `??_7B@@6B@` is emitted as an `.rdata` COMDAT and
its single slot relocates to `?g@B@@UAAHH@Z`. axes2 made the same call
independently (its `VT-recursion`), so this clause is now confirmed twice over
from separate derivations.

## 7. Post-hoc probes (my own design, outside the graded set)

`work/emitpred/guard2/probes/` — `thunk_where.py`, and
`probes/src/thunk_needed.cpp`. Labelled post-hoc; run only in Phase B.

**Question.** `a3_01` emits `??_7D@@6BB@@@` (D's vftable for the B subobject)
but no adjustor-thunk symbol appears among the code leaders. Three candidate
answers with different consequences: (a) no thunk exists; (b) a thunk exists as
a **non-leader** symbol, in which case the leader-based grading *hides*
synthesized emissions and the `#152`/`#161` attribution is unsafe; (c) a thunk
in a section the reader dropped.

**Answer: (a), decisively.** Dumping all 35 symbols and every relocation:

* No `W`-form, `??_9`, `??_D` or any other thunk symbol exists anywhere in the
  object — not as a leader, not as a non-leader, not in any section.
* `??_7D@@6BB@@@`'s single slot relocates **directly** to `?g@D@@UAAHH@Z`.
* (b) and (c) are excluded: every code section's full symbol list is
  `[leader@0]`, except `anchor`'s, which carries two `$M####` labels.

**The obvious objection, and I closed it.** In `a3_01` `D::g` is `return x-5`,
which ignores `this` — so the cell alone cannot distinguish "this target emits
no adjustor thunks" from "the thunk was elided because the target ignores
`this`". I compiled a variant of my own design in which every override genuinely
uses `this` (`probes/src/thunk_needed.cpp`: each class gains a data member and
`D::g` returns `x - dx`). **Still no thunk.** Disassembling the 12-byte body
shows why:

```
?g@B@@UAAHH@Z   lwz r11, 4(r3)    ; bx, at B+4
?g@D@@UAAHH@Z   lwz r11, 8(r3)    ; dx
```

`D` is laid out `{A: vfptr@0, ax@4}`, `{B: vfptr@8, bx@12}`, `dx@16`. Loading
`dx` at `8(r3)` is only consistent with **r3 being the B-subobject pointer**
(`D+8`, so `D+8+8 = D+16 = dx`). c2 compiles the override's out-of-line body
against the *secondary base's* `this` and points the secondary vftable slot
straight at it — which is exactly why no adjustor thunk is needed or emitted.

**Consequence for the lane.** §2's silence about adjustor thunks is **not a gap
in practice on this target**, at least for non-virtual MI: there are no `W`-form
thunks to fail to predict. The plan's standing caveat naming "`??_9` adjustor
thunks" as an out-of-grid unknown (§2, line 149) is, on this evidence, less
threatening than it reads — for these shapes. I make no claim about vtordisp
(`a3_05`) or the vbase cases, which I did not probe and which are guard 1's
cells.

## 8. The tasking leak — follow-up, because its content was false

I disclosed in Phase A that "decides whether the headline is 5 or 7" plus
"whether A3 and A4 break" jointly entail that exactly two of my six were
candidates, that axes2 matched truth on the other four, and that `a2_04` was
excluded. The lead has accepted this and recorded it as `b04d3aa`.

**For the record: that entailment is false, and in the most useful direction.**

* All **six** of my cells had axes2's prediction ≠ truth, not two.
* The one cell that promotes to a confirmed violation is **`a2_04`** — the very
  cell the leak excluded.
* The axes the leak put at stake, A3 and A4, are the two that **do not** break
  on my cells.

So the leak was not merely an information defect; it pointed the wrong way on
every specific it implied. Had I used it, it would have degraded my derivation
rather than rubber-stamping axes2's. I did not use it, and the fact that my
result contradicts it on all three points is independent evidence of that,
stronger than my assurance in §1 of the frozen file.

The lead's generalized rule — *any statement about how many cells could change,
which axes are at stake, or what a score would become is a truth leak, because
arithmetic about a score inverts to a constraint on truth* — is the right
lesson, and this outcome adds a corollary: **a leak computed from a partial or
provisional scoreboard can also be simply wrong**, so a downstream agent that
does use one is not merely compromised but potentially misled.

## 9. Bottom line

* **1 of 6** promotes to a confirmed violation: `a2_04`. **A2 breaks.**
* **A3 does not break on my three cells; A4 does not break on my two.** Both
  axis verdicts must still be composed with guard 1's remaining cells.
* The five `AMBIGUOUS` rulings are **two** statement defects, each with a
  determinate repair that truth confirms (§4). §2's *content* survives all five;
  its *wording* does not.
* **Zero GAP failures** in my six. §2's text could produce every symbol c2
  actually emitted in these cells.
* The single REACH failure is narrow and pre-identified: R1's "any out-of-line
  definition (member…)" over-reaches into **explicit specializations of members
  of class templates**, and nowhere else that A2/A3 tested.
