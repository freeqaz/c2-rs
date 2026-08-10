# w-mmio3 — PREREG

**Frozen before the first `crates/` byte changed.** Merge-base `de16f6cc`
(`merge w-main2: TU MATCH 23 -> 24`). Base binary kept at
`work/w-mmio3/c2rs.base`, md5 `c33cb862e33b6a54bdf3380048f6c982`, built from the
merge-base tree with `c2rs_dirty = False` on its own provenance row.

Commission: convert `src/xdk/nuispeech/mmio.cpp`, TU match **24 → 25**.

---

## 0. The stamps every number below is at

| | |
|---|---|
| c2-rs merge-base | `de16f6cc978a15f550c30cb5e8c7c00e60823493`, clean |
| dc3 workload tree | **`bf3ba961e`** — it MOVED under this lane before the first measurement (`31c7bd4e5` at `w-main2`'s STATUS regen, `104e7df9c` five lanes before that). **Eighth move in ten lanes.** Every number in this PREREG is at `bf3ba961e` |
| workload list | `work/dc3-workload/files.txt` md5 `09189d4a41713c77e14dca9af5050b58`, **878 lines, committed, never regenerated** (#2700) |
| workload flags | `flags.txt` md5 `ef3b32e8ac8d3ab89a8be0a0a60e40c8` — `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc …` |
| base 878-TU scan | `work/w-mmio3/base.txt` / `.jsonl` |

**Base metrics, from that scan and nothing else:**

```
match 24 · mismatch 0 · codegen-gap 0 · vocab-gap 847 · port-error 0 · capture-fail 7
fnbyte-exact 35810 · fnbyte-exact-relocated 3827 · fnbyte-exact-bytes 36342
per-function census 714555/2463471 · census/gate disagreement 0
factor-a 28 · factor-b 338 · factor-c 169 · factor-d 23 · factor-e 3
a-and-b-and-c 27 · a-and-b-and-c-and-d 21 · a-and-b-and-c-and-d-or-e 24
frontier 3 · frontier-if-a 125
selbind-emit-subset-gate-tus 34 · selbind-one-to-one-tus 22 · selbind-selective-tus 12
```

**`mmio.cpp`'s own row at base** (`work/w-mmio3/base_one.jsonl`, re-read inside
the full scan):

```
class            vocab-gap
gate_cause       body-out-of-class
gate_causes      [body-out-of-class, unclaimed-gl-symbol]
fn_total 11 · fn_in_class 10 · fn_names 1
fn_blockers      {expr-cmp-eq: 1}
fn_dispatch      disp-expr-load|BLOCKED|expr-cmp-eq : 1
fn_cflow         cflow-if-n|BLOCKED : 1  (cflow-if-2 1, cflow-if-n 2, straight 8)
gl_body_starts   [11, 11]
selective_bind   [11, 11, 1, 2]
selbind-emitted 11 · selbind-emitted-named-gate 11
```

---

## 1. `docs/CEILING.md` §11.4, worked FIRST and off this lane's own capture

Capture: `c2rs capture src/xdk/nuispeech/mmio.cpp --flags-file <the committed
flags.txt> --cwd <dc3>` → `.ex 5153 · .gl 3621 · .sy 1114 · .in 439 · .db 1118`.
Reference obj: `work/w-ifn/ref/mmio.dump.txt` (18 sections, 61 symbols),
re-read here rather than quoted from the rung.

**1. Ask the BYTE judge, not the census.** `fn_in_class 10 of 11`; the byte
judge (`frontier-bytefrac`) says **256 of 380 accepted bytes**. The one
`fnbyte`-blocked function is `mmioClose`, 124 B. So the census and the byte
judge agree here, which they do not always.

**2. T1 does NOT fire.** Not every body is `fnbyte-exact`; one is unwritten. So
NC-1/NC-2's lists are not the first question — codegen is. (Item 9 is asked
anyway, below, because `w-decouple` #2756 says a fence behind an unwritten body
is still part of the price.)

**3. The reference obj's SYMBOL TABLE, read as a FORECAST.** 61 symbols. Every
non-`.text`/`.pdata`/label symbol: `@comp.id`, `__C2_11886`, `__C1_11886`
(the shell), `memcpy` (minted, already shipped by `w-ifn`), and
**`?FreeHandle@@YAXPAX@Z`** — one undefined external, an ordinary callee.
**No `_fltused`, no `__real@`, no `__savegprlr_N`, no `$M`/`$T` this class does
not already mint, no `.rdata`, no `.bss`, no `.data`.** The three `.pdata`
records and the six `$M`/`$T` labels are the three framed functions'. So the
writer owes nothing this TU that the emitter does not already produce for
`mmioGetInfo`/`mmioSetInfo` — which is the forecast `w-wordwrap` (#2727) says
to take, and here it comes back **empty**.

**4. Not a list-membership refusal.** `expr-cmp-eq` is a construct key, not a
hex type tag.

**5. The reported key's LAYER is not to be trusted, and here it is misleading
in the documented way.** `mmioClose` reports `expr-cmp-eq` under
`disp-expr-load`. Its body is `cflow-if-n` with **three** compares (one `1F`,
two `20`), a direct call, an **indirect** call and a void call. `expr-cmp-eq`
names the first byte the ladder tripped on and none of the mechanism.
**And the workload grep for the class was RUN before pricing**
(`work/w-mmio3/popn.sh`, `popn.txt`, `popn_fpnames.txt`), which is the half of
item 5 `w-wordwrap` (#2721) says has never been run. The construct grepped for
is the only one of the six that could plausibly have a population beyond this
TU: an **indirect call through a loaded member function pointer**. 126
`(*name)(` function-pointer members are declared in the workload's headers;
**0** workload `.cpp` spells `->name(` for any of them.

**That zero is not a bound, and the script prints its own control to say so.**
`mmio.cpp` — the known positive — is **missed by that instrument**, because
`pIOProc` is declared `LPMMIOPROC pIOProc;`, a typedef with no `(*` at the
member, so the name never enters the first grep at all. A population instrument
that cannot see its own known positive reports absence as evidence of absence.
The control grep (`->p<Upper>(`) returns exactly `src/xdk/nuispeech/mmio.cpp`
and nothing else.

**So the honest statement is: the construct's population is UNMEASURED above 1,
and this rung is priced at `+1 fnbyte-exact`** (P8/P12 below carry that as a
falsifiable claim rather than as a premise). This is `w-wordwrap`'s check run in
the direction it can also answer negatively — and it did answer negatively
about *itself*.

**6. Factor A.** `mmio.cpp` passes A: `selbind-emitted 11`, `gl_body_starts
[11,11]`, `selective_bind [11,11,…]`, and its `A∧B∧C` membership is why it is
on the frontier at all. No reader/emitter work here is spent on a TU that fails A.

**7. The board.** `grep BOARD.md` for `mmio`, `mmioClose`, `bctrl`,
`indirect-call`, `elide`, `volatile-park` — rows #2350–#2362 (`w-ifn`),
#2400–#2414 (`w-mmioclose`), #2750–#2762 (`w-decouple`). **No row records this
key already measuring zero.** The five rows `CEILING.md` item 7 warns about are
ranking rows; this is a frontier row with a named byte deficit.

**8. `gate_cause`, and NOTHING else.** `gate_cause = body-out-of-class`,
`gate_causes = [body-out-of-class, unclaimed-gl-symbol]`. The **binding is
PAID** (`w-decouple` #2750/#2751) and is **not re-paid here** — verified on this
lane's own row, not inherited: `selbind-emitted-named-gate 11 of 11`,
`selbind-emit-subset-gate-tus` counts this TU. `fn_names` reads **1** and is
the wrong field, exactly as item 8 says. `emit-bound`/`emit-gate-segments` read
11 == 11 and are also the wrong field. `gl_body_starts` reads `11 of 11` and
`selective_bind` reads `11, 11` — both consistent, neither the answer.
The `unclaimed-gl-symbol` run is `?FreeHandle@@YAXPAX@Z`, an undefined external
`mmioClose` calls; it **discharges with the body** (#2760) and is not a mechanism.

**8b. An instrument's population is bounded by its reader.** GRID-W's
`0–63 B: kept 0, inlined 5,881` band does **not** cover `mmioClose → mmioFlush`,
because an IL call edge needs the caller's body to parse and `mmioClose` is
`Decline::Parse`. Not re-run, not restated; noted so the 8-byte kept edge is
not scored against that band.

**9 (NC-5) — THE VERDICT: IT FIRES, ON ONE OF ITS TWO HALVES, AND THE OTHER
HALF IS ALREADY PAID.** Both fences named by item 9 were read in `crates/`, not
inferred:

* **`comdat::fenced_inlined_callee` → `callee_is_one_c2_expands`: PAID.**
  `w-mmioclose` shipped `if g.inlinable == Some(false) { return false; }` ahead
  of the size test, off `FN_FLAG_INLINABLE = 0x40` in the `.gl` record's ATTR
  byte. `mmioFlush` is `__declspec(noinline)` in the dc3 source and 11/11 of
  this TU's records decode. So the 8-byte callee is **not** fenced at the emit
  seam and `INLINE_DECLINE_BYTES = 128` never gets asked. **The NC-5 licence
  `w-decouple` #2761 priced is already in the tree.**
* **`IlBundle::functions`'s W-FENCE2 exemption: NOT PAID, and it is the one
  live term.** `exempt = gl::plain_external_defined_names(gl)`, which runs
  `gl_defined_names` — the **narrow** `NameFit::StringTableOnly` walk. That
  walk refuses `mmio.cpp` whole-TU (`mmioSeek`, 8 bytes, no `@@`), so
  `gl_defined_names` yields the empty pair and **`exempt = ∅`**. Then
  `callee_defined_here_unmodelled` finds `mmioFlush` and `functions()` returns
  `None`. This is not a guess: `gl::NameFit`'s own doc block states it as the
  named residue — *"`src/xdk/nuispeech/mmio.cpp` is such a TU … `work/w-decouple/`
  sizes the other half as a build that was measured and not shipped."*
* `elide`'s mechanism E and `splice`'s S7 do **not** fire: no body of this TU
  reaches either seam today, and the class this lane builds elides in the
  READER (§3) rather than at `elide`'s emit-time seam.

---

## 2. The `≥ 8` RE-DERIVED — 8 terms, 2 already paid, 6 to build

`w-decouple` #2761 priced *"`w-ifn`'s six + the fence exemption + an NC-5
licence"*. Re-derived at **this** tip against `crates/` and the obj:

| # | mechanism | verdict at this tip |
|---:|---|---|
| 1 | the **`bctrl` encoder** | **UNPAID.** `encode.rs` has `encode_mtctr` (line 209) and no `bctrl` anywhere in `crates/`. Script-count reconfirmed: it is the body's only missing word |
| 2 | an **indirect call as a `Selected` shape** | **UNPAID.** Every call shape carries a callee NAME; here the callee is an expression (`B9 <info> · 33 <int> 8 · 27 · 30`) |
| 3 | a **bound call statement** `26 <dst> · 26 <callee> BD … 4C · 2C <T> 0 · 32 <T> · 4B` | **UNPAID** |
| 4 | a **braceless early return on a call result, on cr0** | **UNPAID.** One `53`, one `54 04` — one scope shallower than `guard_ret_chain`'s arm, so a second grammar and not a parameter of the first |
| 5 | the **elision** and the **volatile park** — one analysis, two facts | **UNPAID** |
| 6 | **the acceptance seam for 5** | **PAID, by `w-mmioclose`.** `w-ifn`'s C6 said board #139 leaves nowhere to ask a sibling question. `IlBundle::functions`'s own module comment now refutes that in the tree: #139 puts acceptance in the PARSER and *this function is the parser's bundle level*, which already reasons across siblings four ways (`drectve_is_boilerplate`, the label-counter gate, the unclaimed-`.gl` accounting, `callee_defined_here`) and carries `gl_function_attrs` per name. What cannot see a sibling is `parse_segment`, and that is a different statement |
| 7 | **the W-FENCE2 exemption reaching this TU** | **UNPAID** — §1 item 9 |
| 8 | **an NC-5 licence for an 8-byte kept callee** | **PAID, by `w-mmioclose`** — §1 item 9 |

**So: 8 terms, 6 to build.** Two of `w-decouple`'s eight were paid between its
lane and this one, by the lane that ran *after* it in the same week, and neither
was paid as *"mmio's price"*.

### 2.1 The whole-TU route (factor E) — CHECKED, and it applies to NONE of the six

`CEILING.md` §16 / `w-main2` #2971: `src/Main.cpp` converted through a whole-TU
emitter with two reader clauses unpaid, because the obj was **two code regions
in one `.text` COMDAT** with **ten** labels where `plan_labels` mints three —
facts with no per-function representation at all.

**`mmio.cpp` is the opposite case and the check is what says so:**

* The port already emits **10 of this TU's 11 bodies** through
  `functions()` → `select_function` → `emit_comdat_obj`, byte-exact, at
  **256 of 380 bytes**. A whole-TU emitter would have to re-produce an
  **18-section** obj — 11 `.text` COMDATs, 3 `.pdata` — that the per-function
  path already produces correctly for 10 of them.
* Not one of `w-main2`'s three triggers is present: **one** code region per
  function, `Value = 0` on every function symbol (`[45] mmioClose sec=14
  val=0x0`), and the label demand is the ordinary framed **triple**
  (`$M3396`/`$M3397`/`$T3398`), which `plan_labels` already mints.
* The two mechanisms whose *only* published home was "nowhere in the port"
  (5 and 6) resolve at the **bundle level of the parser**, which is where
  `functions()` already lives — not at a new whole-TU emitter.

**So the whole-TU route makes NONE of the six unnecessary here, and this is
recorded as a checked negative rather than an omission.** `w-main2`'s operative
correction — *"when a TU's price is dominated by whole-obj obligations rather
than by body shape, ask whether the reader clauses are on the path at all"* —
is asked and answered: this TU's price is dominated by **body shape**
(124 of 380 bytes in one function) and its whole-obj obligations are **zero**
(§1 item 3).

---

## 3. What this lane will build

A **transcription** class, on `ARCHITECTURE_SEAMS.md` §7 / `guard_ret_chain`'s
precedent: thirty-one words of one named function class, `/O1` only,
`NotImplemented` outside, `PORT_CFG_CLASSES` **not** widened.

```c
  R f(P p, U u) {
      if (p == 0) return K;              // pointer guard, cr6, arm in source order
      V r1 = g(p, L1);                   // bound call stmt, same-TU callee
      if (r1 != 0) return r1;            // braceless early return, cr0, INVERTED
      T *t = (T *)p;                     // reinterpreting assignment, no code
      V r2 = t->fp(t, A1, u, A3);        // INDIRECT call through a loaded member
      if (r2 != 0) return r2;            // braceless early return, cr0, INVERTED
      h(p, 0, 0, 0);                     // ELIDED — same-TU, pure, result unused
      k(p);                              // void call to an EXTERNAL
      return 0;
  }
```

The thirty-one words, transcribed from `.text #14` and not composed:

```text
 0x00 mflr 12 / stw 12,-8(1) / std 31,-16(1) / stwu 1,-96(1)   FrameLayout{0,4,1,0}
 0x10 mr 31,3          THE r31 PARK — hmmio is live across bctrl and across an
                       external bl, so no volatile qualifies (M-RULE)
 0x14 mr 5,4           THE r5 PARK — fuClose crosses ONE call whose callee this
                       TU defines and whose footprint is {r3}, and r5 is the
                       register its next consumer wants (M-RULE, coalescing)
 0x18 cmplwi 6,3,0 / bf 26,+12 / li 3,K / b -> epi          the guard, cr6
 0x28 li 4,L1 / mr 3,31 / bl <g>                            REL24 #1
 0x34 cmplwi 3,0 / bf 2,-> epi                              cr0, INVERTED sense
 0x3c lwz 11,OFF(31) / li 6,A3 / li 4,A1 / mr 3,31 / mtctr 11 / bctrl
 0x54 cmplwi 3,0 / bf 2,-> epi                              cr0, INVERTED sense
 0x5c mr 3,31 / bl <k>                                      REL24 #2
 0x64 li 3,0
 0x68 addi 1,1,96 / lwz 12,-8(1) / mtlr 12 / ld 31,-16(1) / blr   common epilogue
```

**The elision and the park as they will SHIP, with their sources:**

* **The elision** ships as a clause of this class and **not** as a widening of
  `elide.rs`, which is `w-ifn`'s **D2 firing as registered**. The rule is
  `w-ifn` #2351, obj-derived, **10 cells at `/O1` and again at `/Ob0`**:
  *a call whose RESULT IS UNUSED and whose callee is defined in this TU with a
  body that has NO SIDE EFFECT is deleted* — explicitly **not** `noinline`
  (`e5` elides without it), **not** empty-body (`e6`'s callee emits
  `addi r3,r3,1 ; blr`), **not** tail position (`e9`), and identical at `/Ob0`,
  which is what separates it from the inliner.
* **The park** is `WB_CHOOSER_FINDINGS` §2.3's **M-RULE** (#1881, obj-derived
  from 16 manufactured cells, no DISCLOSURE row needed) plus its first
  sub-rule, *"coalescing beats allocation"*, whose **3 witnesses** are base
  `mmioClose`'s r5, M9-b and M14-b. **Sub-rule #1882 (`r11` when the value does
  not cross a call, `r10` when it does) does NOT apply to this body** — it is
  about which volatile a *move* picks, and both of this body's parks are
  decided one rule earlier (r31 by "no volatile qualifies", r5 by coalescing).
  Stated because the brief named #1882 and the body does not use it.
  `w-ifn`'s `probe/park.cpp` (5 cells, workload flags) is the second
  instrument: `p1` reproduces the 124 bytes exactly, `p2`/`p4` move the park to
  r30 and grow the frame 96 → 112 when the callee is external.

**Where each fact is asked** (six mechanisms, three seams):

| mechanism | seam |
|---|---|
| 3, 4, and the SHAPE of 2 and 5 | the body parser — a new production at the END of the `0xB9` ladder, non-committal, own cursor |
| the interprocedural half of 5 (purity of the elided callee, footprint of the parked-across callee) | `IlBundle::functions`, TU level, on `bind.is_varargs`'s precedent |
| 7 | `IlBundle::functions`, one exemption |
| 1 and the emitter half of 2 | `codegen::encode` + a new `codegen` module + one `Selected` arm |

**Mechanism 7's repair, and it is a DECOUPLING and not a widening — the same
shape `w-decouple` used for the binding.** The exemption stops being a function
of a WALK and becomes a function of the **binding's own names**: for each name
`Bindings` bound, the per-name three-byte test `record_is_plain_external`
(`linkage 0x05`, size `< 0x80`, flags **exactly** `0x00`) at that name's unique
symbol run. On every TU where the narrow walk succeeds this is the **same
set**, because `plain_external_defined_names` is exactly `bound ∩ that test` and
the two walks bind the same names wherever the narrow one does not stop. It can
differ **only** where the narrow walk returned the empty pair — which is the
same monotonicity `NameFit`'s doc block already proves — and there it turns a
wholesale `locally-defined-callee` refusal into the size/`noinline` question at
`comdat::fenced_inlined_callee`, which is the seam `w-inlfence2` built for it.

---

## 4. Predictions — probabilities, with the antecedent each claim ACTUALLY needs

Scoring: Brier, and a prediction whose antecedent did not hold is **VOID**, not
a hit. `w-main2`'s two freshest lessons are applied: **no row below is counted
as supporting evidence for another row whose falsifier is its negation** (P2/P3
and P8/P9 are marked as one item each for that reason), and **no structure is
read off one obj where a second instrument has already separated the readings**
(P5 and P6 name their second instrument).

| # | p | claim | falsifier |
|---:|---:|---|---|
| **P1** | 0.93 | `mmio.cpp` **CONVERTS**: `match 24 → 25`, `mismatch 0`, `frontier 3 → 2`. *Antecedent: P2 ∧ P4 ∧ P5 ∧ P6 ∧ P7 ∧ P10 all hold.* Stated with its antecedent because every one of them is a separate falsifiable claim and the conjunction is what the conversion needs | the scan reads `match 24` at the shipping tree |
| **P2** | 0.90 | The **six** of §2 are the whole remaining price: nothing else refuses this TU once they are built. Includes `unclaimed-gl-symbol` discharging with the body | the shipping tree's `mmio` row carries a `gate_cause` that is neither `body-out-of-class` nor absent |
| **P3** | 0.80 | **The `.gl` walk is the fence's ONLY remaining blocker at the parser** — i.e. after the body parses, `callee_defined_here_unmodelled` is the single clause between `functions()` and `Some(funcs)` | a second parser-level clause refuses (label counter, varargs, `drectve`, unclaimed) |
| **P4** | 0.85 | The **body is 124 bytes / 31 words on the first assembled build**, `fnbyte-exact` for `mmioClose`, both REL24 sites at `+0x30` and `+0x60` with targets `mmioFlush` and `?FreeHandle@@YAXPAX@Z` | any byte or either relocation differs |
| **P5** | 0.88 | **The label plan needs NO new rule**: `plan_labels` unmodified, `label_lead 0`, framed stride 5, gives `mmioClose` `$M3396`/`$M3397`/`$T3398`. *Second instrument (not the obj): the in-TU stride re-derives it with no free parameter — `mmioGetInfo` 3381 + 5 = 3386 = `mmioSetInfo`, + 5 + five leaves × 1 = 3396. The `memcpy` slot cancels out of both differences (`w-ifn` #2358).* | the emitted `$M` numbers differ from 3396/3397/3398 |
| **P6** | 0.88 | **The `.pdata` word is `40 00 1f 04`, computed and not stored**, from `Frame{prolog_len:16, func_len:124}`. *Second instrument: the same formula reproduces this TU's other two words `40001503` and `40001b04` at 84/12 and 108/16 (`w-ifn` mech. 4), so the reading is fitted on three points in one obj, not one.* | the emitted `.pdata` word differs |
| **P7** | 0.75 | **The exemption decoupling is NEUTRAL on the other 877**: `fnbyte-exact 35810 → 35810`, class verdicts moved **0** apart from `mmio.cpp`, and no `gap-metric` key changes value except the ones this conversion moves | any other TU's class verdict moves, or `fnbyte-exact` moves by anything other than +1 |
| **P8** | 0.70 | **`fnbyte-exact` 35810 → 35811 exactly** — one new byte-exact function, the census `714555 → 714555 + 1`. *This is the POSITIVE delta the calibration asks for; `w-main2`'s conversion moved it by zero because its route was whole-TU, and this one is per-function, which is the whole content of §2.1* | the delta is not exactly +1 |
| **P9** | 0.55 | **`factor-e` stays 3 and `factor-d` goes 23 → 24** — the conversion arrives through the ordinary A∧B∧C∧D path, not through a new whole-TU recognizer | `factor-e` moves, or `factor-d` does not |
| **P10** | 0.80 | **The `bctrl` encoder is the body's only missing WORD** — every other word already has an encoder in `codegen::encode` | a second word needs a new encoder |
| **P11** | 0.35 | The **elision needs the sibling's parsed body**, i.e. a purely local rule (result-unused ∧ callee-in-`defined`) would be unsound and the lane will find a cell that shows it | the shipped clause is local and the `_neg` grid cannot separate it |
| **P12** | 0.15 | The class has a **population above 1** in the 878 (some second TU gains a byte-exact body) | `fnbyte-exact` delta > +1 |
| **P13** | 0.65 | The `_neg` fixture will need at least one cell **REPLACED** because its first form grades nothing (a `mismatch`-impossible or over-fenced cell) — the `w-main2` / `w-decouple` §8.2 outcome | every `_neg` cell grades a distinct named clause on first writing |
| **P14** | 0.30 | `hatch-red` will show an arm count other than the pre-existing `R2 R6 A2 F1 C1` at the merge-base, i.e. this lane will have to reproduce the base before attributing | the base and the tip arm sets are equal |

**Rows I decline to count as independent evidence, and why.** P2's falsifier is
*"some other clause refuses"* and P3's is *"a second parser clause refuses"* —
P3 is a strict sub-claim of P2, so a P2 hit is **not** a second confirmation of
P3 and the two are scored as one item if both fire the same way for the same
reason. Likewise P1's falsifier is the disjunction of P2/P4/P5/P6/P7/P10's, so
**P1 is not evidence for any of them**; it is scored, and its antecedent is
written out, precisely so that a P1 hit cannot be read as six hits.

**The conversion in probability form, with the antecedent it actually needs:**

> P(`match 24 → 25` **given** that the six mechanisms of §2 are the complete
> remaining price and the exemption decoupling is neutral) = **0.93**.
> P(`match 24 → 25`) unconditionally = **0.93 × 0.90 ≈ 0.84**.

**The `fnbyte-exact` delta, predicted as a signed number:** **+1**
(35,810 → 35,811), p = 0.70; **≥ +1** p = 0.85; **0 or negative** p = 0.15.

---

## 5. Neutrality, the corpora, and the discipline this lane owes

* **Key 878-TU neutrality on the FULL PATH** (#2667), both binaries, same
  committed list and flags, same dc3 tree. Report **per-TU byte triples** for
  every TU whose verdict or byte count moves, and **both blocker histograms**.
* **A clean fixture scan is NOT sufficient** (35 wrong objs once read 0
  mismatch at both `/O1` and `/Ox`). The gate run must show `expr_sweep` and
  `mode_cross` **UNSAMPLED**.
* **Four-level verdict neutrality with directions**: TU class, `fnbyte`,
  per-function census, `gap-metric` keys.
* **Mode gates in the PARSER** (#1638) — the `/O1` clause is asked before any
  body byte; **acceptance in the parser** (#139) — the interprocedural clauses
  go at `functions()`, which is the parser's bundle level, and the emitter
  keeps its own copy of the mode gate so the two cannot silently disagree.
* **`_neg` cells**: a multi-cell file can never go `mismatch`; an over-fenced
  cell grades none of its clauses and the repair is **merging** (#2665/#2698);
  a merged clause's must-fail mutation must delete the **whole** conjunction
  (#2699); cells that grade nothing are **NAMED, not counted**.
* **The label channel** is reported in `LABEL_COUNTER.md` §7.6/§7.6a's
  **in-the-middle** form — every number an offset from the cell's own `.gl`
  seed — never the counterfactual form.
* **Gate discipline**: commit before running (`--require-graded` refuses a
  dirty `crates/` with exit 4 and prints `graded tree: <hash>` at both ends);
  expect the sweep row's own `.pyc` to dirty the tree on the first run (#2979)
  and clean it rather than fight it.
* **Repo**: base binary KEPT; `git checkout <rev> -- path` **stages** (#2512);
  never `git add -f` a directory; no large scan dumps; absolute paths scrubbed
  before committing; `docs/rungs/INDEX.md` is GENERATED.

---

## 6. What this lane will NOT do

* It will **not** widen `PORT_CFG_CLASSES`. Accepting one 31-word body is not a
  claim about `cflow-if-n`.
* It will **not** widen `elide.rs` mechanism E. `w-ifn` D2 registered the
  elision as a clause of its own class and it ships that way.
* It will **not** implement M-RULE as a register allocator. The park is
  transcribed with the shape fenced; M-RULE is the *explanation* and the
  *fence's* justification, not the code.
* It will **not** re-pay the binding (`w-decouple` #2750/#2751) or the
  `noinline` NC-5 licence (`w-mmioclose` #2400–#2414). Both were verified in
  the tree, not assumed.
