# WB_GLOBARMS — gate A's twelve arms: what an obj decides, and the one function that decides the kind

Lane **`w-globarms`**, wave 19 L4 (`docs/ADOPTION_BRIEF_2026-08-29.md` §L4).
Prereg [`work/w-globarms/PREREG.md`](../../work/w-globarms/PREREG.md) (commit
`a0e5b58a3`, **before the image was opened**) and
[`PREREG_ADDENDUM.md`](../../work/w-globarms/PREREG_ADDENDUM.md). Classification
[`ARMS.tsv`](../../work/w-globarms/ARMS.tsv). Grids
[`grids/w-globarms/`](grids/w-globarms/). Instrument
[`scripts/grade_globarms.py`](scripts/grade_globarms.py). Transcripts
[`GRADE.txt`](../../work/w-globarms/GRADE.txt),
[`CONTROLS_RED.txt`](../../work/w-globarms/CONTROLS_RED.txt). Board
**#3808**–**#3813**.

**Reach 0, as predicted.** `git diff master..HEAD -- crates/` is empty at the
tip. No `scripts/gate.sh` row (`#3691`). **No `ported` numerator for globregs or
regalloc** (decision 21 §4, `#3505`) — §6 says what this lane found instead and
hands it to the owner.

---

## 0. THE HEADLINE — the kind byte has ONE writer, and it makes gate A a statement about COFF linkage

`w-globobj` reported the population and did not pursue it:

> a defensible site-level population does exist — **gate A is a 12-arm decision
> over the symbol `kind` field, each arm addressed**.

The arms are real, and **twelve is the right number** —
`grade_globarms.py` decodes them out of `c2.dll`'s own bytes at
`0x10b5511a`–`0x10b551c6` and asserts it. But the arms are unreadable until you
know what a `kind` value *is*, and `P_GLOBREGS` never says. It comes from one
function:

> ### `FUN_10bd2913` (`0x10bd2913`) is c2's front-end → back-end symbol map, and it is where every `kind` in gate A is decided. `[R]`
>
> Memoised on `gl+0x10` (early-out `0x10bd2917`, cache write `0x10bd299c`),
> writes the kind at **`0x10bd2a1d`**, sets `[sym+0x08] = sym` at
> `0x10bd2a20` — which is why gate A's **A3** leader test passes for
> everything it makes — and writes `[sym+0x00] = gl` at `0x10bd299f`, which is
> the pointer **A10** dereferences to reach `+0x37`. **32 distinct calling functions, 50 call sites** (`calls.tsv` / `xrefs.tsv`).
>
> The kind is a four-step `dec`-chain at `0x10bd2926` on the `.gl` record's own
> kind byte `[gl+0x30]` (`P_SYMBOL.md` §1: **1 data, 3 function, 4
> extern/alias**), and for a *data* record an **8-entry jump table at
> `0x10bd2a9f`** indexed by the **3-bit linkage field**
> `([gl+0x37] >> 0x15) & 7`.

```
  [gl+0x30] == 1 -> the linkage table      [gl+0x30] == 3 (a function) -> kind 0xb
  [gl+0x30] == 2 -> kind 4                 [gl+0x30] == 4 (extern/alias) -> kind 0xa
                                           anything else               -> kind 0xa

  linkage 0 -> a NULL table slot — unreachable by invariant
  linkage 1 -> kind 4          linkage 3 -> kind 5
  linkage 2 -> kind 8 iff ([gl+0x37] & 0x1e0) == 0x80, else kind 7    ( = linkage 6 )
  linkage 4 -> storage kind 1,2 -> 7;  4 -> 8;  else -> 9             ( = linkage 7 )
  linkage 5 -> ((gl+0x20) >> 4) & 2 | 5, i.e. kind 5 or kind 7
```

**That 3-bit field is not a new one.** `P_SYMBOL.md` §3 already reads it, at
`0x10b28bb4`/`0x10b28bbd`, and records that

> `(([sym+0x37] >> 0x15) & 7) ∈ {1, 3}` — a **linkage class that is suppressed
> outright**, producing **no COFF record at all**.

Put the two reads together:

> ## ⇒ **Gate A's A6 arm — kinds 4 and 5 — is exactly the set of symbols that get NO COFF RECORD.** Every symbol that *does* get one arrives at A8 or A9 as kind 7, 8 or 9. `[R]`

A promotion policy that has no size threshold, no use-count threshold and no
mode flag (`P_GLOBREGS` §3's headline negative, which survives) turns out to be
keyed on **the front end's linkage classification** — the same field that
decides whether the symbol reaches the object file at all. That is a
one-sentence model of gate A, and it is read, not fitted.

### And the obj half: 38 graded, 0 `U`, 0 misses, both profiles

Two grids, 19 cells, `/O1` and `/Ox`. **The deciding cell is
`gb_pair_yescape` against `gb_pair_xescape`** — two `int` locals of the same
type in the same body, one of which has its address escape:

```
gb_pair_yescape   x -> r31          y -> stw 10, 80(1)
gb_pair_xescape   x -> stw 10, 80(1)   y -> r31
gb_pair_none      x -> r31          y -> r30
```

Same kind, same arm, same TU, same profile. **Only the escape moves, and the
map moves with it.** That is A6's *internal* branch —
`sym+0x05 & 2` ⇒ join the `DAT_10c2e3e8` aliasing set — decided at the obj.

---

## 1. THE CONVERSIONS — 4 of 12 arms, each with its witness named

**Denominator: 12 arms. Converted `[R]` → `[O]`: 4 (A4, A6, A8-confounded,
A11-partial). The prereg registered a ceiling of "at most 5 convert, at least 6
are `CONSTR`". The outcome is 4 converted and 6 `CONSTR`. The ceiling held.**

### 1.1 `[O]` — **A6** (`0x10b5513e`, kinds 4 and 5) and its internal aliasing test

**Witness: `grids/w-globarms/arm2_grid.cpp`, `gb_pair_yescape` /
`gb_pair_xescape` / `gb_pair_none` / `gb_addr_local` / `gb_addr_escape`, and
`arm_grid.cpp`'s `ga_int` / `ga_escape` / `ga_param` / `ga_ref`. 18 verdicts
over two profiles, 0 `U`.** Graded by `grade_globarms.py --arms`, transcript
`work/w-globarms/GRADE.txt`.

An auto is a kind-4 or kind-5 symbol; it is eligible; and it joins the
aliasing set **only when `sym+0x05 & 2` is set**, which the pair cells show is
a **per-symbol** property and not a per-function one.

**And a refinement of that bit, from `gb_addr_local`:** `int x = p[0]; int *q =
&x; … return *q;` is **PROMOTED**. Taking a local's address is *not* what sets
the bit — the address **escaping to an opaque callee** is. `[O]` on the
partition; `[I]` on calling the bit "escape".

### 1.2 `[O]` — **A4** (`0x10b55134`) and **A11**'s accept side: kind 3 is the compiler-generated TEMPORARY, and it IS a candidate

**Witness: `arm_grid.cpp`, `ga_temp` and `ga_temp3`, both profiles.**

`return f1(x) + g1(y);` has an unnamed value that must survive a call:

```
0018  bl f1
001c  mr 30, 3        <- f1's result, into a CALLEE-SAVED register
0024  bl g1
0028  add 3, 30, 3    <- and still there
```

No frame traffic. `ga_temp3` does it twice (`r29`, `r31`). **Kind 3 is minted
by `FUN_10bd28a2` (`0x10bd28bf`) out of a bare type word**, allocated from
`FUN_10bd2492`'s kind-`0xf` pool, with `[sym+0x08] = sym` at `0x10bd28c3`. So a
temporary reaches A4, is dispatched to A11/A12, and is admitted. **The
accept side of A11 is `[O]`; its reject side is not** (§3).

### 1.3 `[O]`, but **CONFOUNDED and deliberately not banked** — **A8** (`0x10b5514a`, kinds 7 and 8)

**Witness: `ga_extern`, `ga_fstatic`, `ga_lstatic`, all MEMORY at both
profiles**, each through the relocated-static arm of the readout
(`stw 11, 0(31)` with a `REFLO` to the symbol).

**This lane refuses to score those three as evidence about A8.** A symbol with
a COFF record must be observable to another translation unit across an opaque
call **for language reasons**, so its MEMORY verdict is over-determined:
gate A and the C++ object model predict the same thing and the obj cannot
separate them. `w-globobj` §2.1 made exactly this mistake's mirror image —
banking `pc_struct2` as a gate-B confirmation when the mechanism was a
front-end artifact — and the lesson is the same one. What is **not**
confounded is the read: linkage 2, 4, 6, 7 are the classes that get a COFF
record and they are the only ones that reach A8/A9.

### 1.4 `[O]` — **A3**'s consequence: a member-wise aggregate promotes member by member, at any member width

**Witness: `ga_structmix` and `ga_struct4`, both profiles.**

```
ga_structmix   lwa 30, 0(11)   lbz 29, 4(11)   lha 28, 6(11)   ld 31, 8(11)
```

An `int`, a `char`, a `short` **and a `long long`** member, all four in
callee-saved registers across a call, **no frame traffic at all**. This is the
prediction that separates two readings of A10's `t+0x20 == 4` sub-symbol width
test, and the answer is unambiguous: **the width test belongs to the kind-10
arm and does not apply to a local aggregate**, because a local aggregate is a
kind-4/5 symbol taking the general path at `0x10b551ca`, where each sub-symbol
is gate-B'd individually and no width is tested. `[O]` on the verdicts, `[I]`
on the attribution.

---

## 2. REFUTATIONS — of `P_GLOBREGS`, of `w-globobj`, of the brief's framing, and of this lane's own instrument

**Six.**

### 2.1 ⛔ REFUTED — `P_GLOBREGS` §2's row for `0x10bd3225`

> *"`0x10bd3225` — one symbol allocation: bump `chunk+0x04` by `0x60`, or take
> the free list at `symtab+0x30` …"*

`FUN_10bd3225` is **the symbol-table constructor**. It allocates the `0x4c`-byte
symtab (`0x10bd322c`), a `0xfa0` array, **four** initial chunks into
`symtab+0x18/+0x1c/+0x24/+0x28`, three bitsets, and only then — as its last act
— mints **one** record and stamps it **kind `0x10`** at `0x10bd339c`, parking it
at `symtab+0x3c`. The mechanism §2 describes is real; it is the *inlined tail*
of a constructor that runs **once per compilation**, not the general allocator.

**The general allocator is `FUN_10bd2492`, and it is not one pool.** It
segregates the arena into **five sub-pools by kind**, each with its own free
list and its own current chunk, all drawing 32-slot `0x60`-stride chunks from
the single appended chain `FUN_10bd2343` maintains:

| kinds | free list | current chunk |
|---|---|---|
| 3, 6 | `symtab+0x2c` | `symtab+0x24` |
| 0, 1, 2, 4, 5, 0xd | `symtab+0x30` | `symtab+0x18` |
| 7, 8 | `symtab+0x34` | `symtab+0x1c` |
| 9, 0xa, 0xb | `symtab+0x38` | `symtab+0x28` |
| 0xc, 0xe, 0xf, ≥0x10 | *(none)* | `symtab+0x20` |

**Consequence for §6.4's recycling wrinkle, which gets sharper rather than
weaker:** a recycled slot comes off the free list **of its own kind class**, so
a slot that once held a kind-7 symbol can only ever be reused for kinds 7 and 8.
`[R]`

### 2.2 ⛔ REFUTED — §3's "gate A, then gate B" sequencing. Kind 10 never reaches gate B

The A10 path (`0x10b55171`–`0x10b551a3`) allocates its own aux records and jumps
straight to `0x10b55295`. It never touches `0x10b551ca`/`0x10b551d4`, which is
the `FUN_10bd7d24` type gate. **Gate B applies to kinds 3, 4, 5, 7 and 8 and to
nothing else**, and `t+0x20 == 4` is kind 10's *substitute* for it, not an
addition. §3 presents the two gates as sequential for every symbol; for kind 10
they are alternatives. `[R]`

### 2.3 ⛔ REFUTED — §3's reading of the `DAT_10c2e2cf` side set

§3 says `DAT_10c2e2cf` *"is consulted at `0x10b551dd` but only to add the index
to a side bitset"*. True, and incomplete: `0x10b551e6`–`0x10b5520b` admits a
symbol to `DAT_10c2e3ec` **only when the type word's top nibble is 5** on the
leader or on the sub-symbol (`and cx,0xf000` / `cmp cx,0x5000`) — which §9
already calls the FPR nibble. **The side set is the floating-point one.** `[R]`

### 2.4 ⛔ REFUTED — the prereg's own reason for filing A1 `CONSTR`, though not its verdict

`PREREG.md` §4 filed A1 `CONSTR` because *"kind `0x10` is not source-selectable"*
— an argument from ignorance, which is precisely the shape `#3505` is five for
five against. The read replaces it with a fact: **kind `0x10` is the symbol
table's sentinel record**, minted once at `0x10bd339c` and parked at
`symtab+0x3c`. It exists exactly once per compilation and appears in no tuple.
The verdict stands; the reason it stands on is now a read rather than a guess,
and that difference is the whole content of the `CONSTR` rule.

### 2.5 ⛔ REFUTED — a notation collision in this repo's own ledger

`DISCLOSURE W-STAGETAP-6` records *"`func+0x00` → the `.gl` symbol record,
**`sym+0x04` → a NUL-terminated `char *`**"*, while `W-STAGETAP-4` on the row
above records *"**`sym+0x4` kind**"*. Both are correct and they are **two
different record types**: `+0x04` of the *`.gl`* record is the name pointer
(`0x10b9acd0`), and `+0x04` of the *globregs* record is the kind byte
(`0x10bd2a1d`). `FUN_10bd2913` is the bridge — `[gl+0x10] ↔ backend`,
`[backend+0x00] = gl` — and it is what makes the two coexist. Prereg **R-D**
predicted this at p = 0.85 and it is a **HIT**; it is filed as a refutation
because a bare `sym+0x04` in this repo is ambiguous and nothing said so.

### 2.6 ⛔ REFUTED — this lane's own grader, by its own control

Planted control defect 5 removed the relocated-static arm of the frame-traffic
readout. The three A8 cells flipped to PROMOTED and the run printed

    GRADE: PASS  (3 prediction misses — a RESULT, not a failure)

— **a dead readout publishing a table of misses that reads like a finding.**
The `--selftest` caught it; the `--arms` path did not. The fix (commit
`eaeebd42a`) makes the four synthetic readout assertions a **precondition** of
the cell half, so no verdict is printed at all when the readout is dead.
`work/w-globarms/CONTROLS_RED.txt` carries the before and the after.

**This is `w-globobj` §2.6's third defect in a different costume**: a control
that reports on one path while the publishing path stays green.

---

## 3. THE CLASSIFICATION — unobservable by construction vs merely uncompiled

The full assignment is [`ARMS.tsv`](../../work/w-globarms/ARMS.tsv). Its rule,
inherited verbatim:

> **`CONSTR` is a claim about the corpus, not about my index.** Before an arm is
> filed unobservable the **two obj bodies that would have to differ** must be
> stated, and why they cannot exist. If they cannot be stated it is `UNCOMP`,
> however unlikely a cell looks. `#3505` is five for five, and `w-regcells`
> found 213 cells for a claim that said none existed.

**Outcome: 4 `OBS`, 6 `CONSTR`, 2 `UNCOMP`.**

### 3.1 UNOBSERVABLE BY CONSTRUCTION — 6 of 12, and one reason covers four of them

> **Every rejecting arm branches to the same address.** A5, A7, A9, A11's
> reject side and A12's reject side all jump to `0x10b552b8`, which increments
> `DAT_10c2e454` and clears `+0x34`/`+0x38` on every sub-symbol. **An obj can
> say that a symbol was rejected; it can never say which arm rejected it.**

| arm | the two bodies that cannot differ |
|---|---|
| **A1** | one in which the symtab **sentinel** (kind `0x10`, minted once at `0x10bd339c`) is promoted, and one in which it is skipped. It has no source form, exists once per compilation, and is in no tuple |
| **A2** | one in which `sym+0x40 &= ~1` does not happen. The write is unconditional and on the straight-line path; nothing chooses |
| **A3** | a non-leader processed directly vs processed through its leader's `+0x0c` chain. It **is** processed either way — §1.4 is the obj proof — so the two emit identically |
| **A5** | kind 0/1/2 rejected vs any other kind rejected. Kind 1 is a physical register and kind 2 is the **candidate record itself** (`0x10b54d6c`); neither is in the arena `FUN_10b550e5` walks, and neither has a source form |
| **A7** | kind 6 rejected vs any other kind rejected. Kind 6 is written only by `FUN_10c05f44` (×5) and `FUN_10c0c251`, the by-name symbol region `WB_S7_FINDINGS`:450 shows minting `__C_specific_handler` |
| **A9** | a kind-`0xb` **function** symbol coloured vs not. `gb_fnaddr2` uses a function's address for two calls and c2 emits two direct `bl`; c2 never materialises the symbol as a value |

### 3.2 MERELY UNCOMPILED — 2 of 12, each with the cell that would decide it

| arm | the cell |
|---|---|
| **A10** — kind 10 needs `(*(sym))+0x37 & 0x400` set and `& 0x200000` clear, then indexes only sub-symbols with `t+0x20 == 4` | an aggregate that is itself an **undefined external** and is assigned member-wise across a call, so a kind-`0xa` symbol reaches A10 **with sub-symbols**. This lane could not build one; the shape is nameable and so it is `UNCOMP`, not `CONSTR` |
| **A12** — kind 3 needs `sym+0x07 & 0x40` clear | a temporary with `sym+0x07` bit 6 set. **What sets that bit was not read**, so the cell cannot be written yet — the same state `w-globobj` filed `aux+0x18` in, and for the same reason |

Plus **A11's reject side**, which is `OBS` on its accept half and unknown on
its reject half: no source construct is known that makes `sym+0x14` nonzero on a
temporary.

### 3.3 The one route that would have broken the `CONSTR` wall, and it did not

`PREREG.md` §4.1 registered **G1** at p = 0.20: *"`DAT_10c2e454` has no reader
that reaches an emitted artifact."* If the reject-tail counter were printed or
consulted, A1/A3's silent skip and A5/A7/A9's charged reject would separate in
an obj and half of §3.1 would be wrong. **`DAT_10c2e454` has exactly two
references in the image** — the zeroing at `0x10b550e7` and the increment at
`0x10b552b8`. **It is written and never read.** G1 **HIT**, and §3.1 stands on a
measured absence of readers rather than on an assumption.

---

## 4. THE PREREG SCORE, by tier, never pooled

| tier | commit | predictions | hits | misses | ungraded |
|---|---|---:|---:|---:|---:|
| **PREREG** — before the image was opened, before any cell | `a0e5b58a3` | 34 | **31** | **1** | 2 |
| **ADDENDUM 1** — after the read, before the compile, **committed after** | `e3835448f` (grid header) | 6 | **6** | 0 | 0 |

**PREREG detail.** §3's twelve obj predictions: **12 hits, 0 misses**. §2's six
kind predictions: K1 HIT (kind 1 physical register, corroborated), K2 HIT
(kinds 4/5 are the autos), K3 **MISS** — kind 10 is *extern/alias*, not
"an aggregate with a member list"; the aggregate story is A3's, not A10's —
K4 HIT (kind `0x10` is a placeholder; it is the *sentinel*), K5 HIT in
substance (kind 3 is not the extern class but the temporary; scored **ungraded**
because the prediction named the wrong entity for the right arm), K6 HIT (**15**
of the 17 values `0…0x10` have an attributed writer — every one except kind 0 and kind `0x0f` — against a threshold of 8).
§6: R-A, R-B, R-D HIT; **R-C ungraded** — this lane did not enumerate every
switch on `sym+0x04` image-wide. §4's twelve classifications: 10 as registered,
**A10 moved `OBS` → `UNCOMP`** and **A11 moved `OBS` → partial**. G1 HIT.

**The ceiling held.** §4 registered *"at most 5 arms convert, at least 6 are
`CONSTR`"*. Outcome: **4 converted, 6 `CONSTR`, 2 `UNCOMP`.**

**The one MISS is the useful one.** K3 predicted kind 10 was the aggregate arm,
and `w-globobj`'s member-wise finding made that look obvious. It is wrong: a
local aggregate is a kind-4/5 symbol on the *general* path, and A10's width test
never sees it. `ga_structmix` is the cell that says so — a `long long` member
promoted alongside a `char` one, which `t+0x20 == 4` forbids.

---

## 5. CONTROLS — five planted defects, all watched RED, plus the cross-grader

`#3336`. `work/w-globarms/CONTROLS_RED.txt`.

| # | planted defect | how it went red |
|---|---|---|
| 1 | the kind byte read at `sym+0x08` | the arm decode **aborts**; no answer key, nothing published |
| 2 | frame-traffic scan starts at instruction 0 | two `--selftest` assertions fail **and** the readout precondition refuses the cell half |
| 3 | a no-frame body scored PROMOTED instead of `U` | *REJECT a body with no frame* fails; precondition refuses |
| 4 | the linkage jump table read at stride 3 | the kind-map decode **aborts** |
| 5 | the relocated-static arm removed | *a store to a RELOCATED static is MEMORY* fails; **and this is the one that found a real hole** (§2.6) |

`--selftest` carries **16 assertions, 3 of them the grader having to REJECT** a
mutated image, plus one that proves the kind→arm map is the *image's*: patching
`cmp al,5` to `cmp al,7` at `0x10b5513f` moves kinds 6 and 7 into A6, and the
assertion checks that it does. **A constant that lives in this file could not do
that.**

**C1** fired both ways (`ga_int` PROMOTED, `ga_vol` MEMORY, both profiles).
**C4**, the cross-grader control: `grade_globobj.py --promote`'s independent
readout **agrees on all 38 cell/profile verdicts**. **Premise test: 0 of 38
cells scored `U`.** No count here rests on an absence.

---

## 6. THE POPULATION, AND WHY THIS LANE DEFINES NOTHING ON IT

The brief asks for the arms and forbids a numerator, and the read makes the
temptation concrete, so it is named rather than left implicit:

> **There is now a defensible site-level population for globregs: 12 gate-A
> arms, each with an address, a set of kind values reaching it, and a
> classification. 4 of 12 have an obj witness.**

**This lane defines no metric on it** — decision 21 §4, `#3505` five for five —
and hands the owner two facts that would have to be settled first:

1. **6 of the 12 are `CONSTR`, and they are `CONSTR` for one shared reason**
   (every rejecting arm branches to `0x10b552b8`). A `ported`-style ratio over
   this population would have a ceiling of **6/12** built into its denominator
   on day one, which is `#3776`'s trap in a new place.
2. **The arms are not equally weighted.** A6 and A8 between them cover every
   symbol a C++ compiland actually declares; A1 covers **one record per
   compilation**. A count that treats them as twelve equal sites measures the
   binary's branch structure, not the compiler's behaviour.

---

## 7. HANDOFFS

* **A lane wanting A10**: build an aggregate that is an **undefined external**
  and assign it member-wise across a call, so a kind-`0xa` symbol reaches A10
  *with sub-symbols*. Then `t+0x20 == 4` bites and the width test is decidable.
  `ARMS.tsv` names it; do not re-file it `CONSTR` without the two bodies.
* **A lane wanting A12 or A11's reject side**: read what sets `sym+0x07 & 0x40`
  and `sym+0x14` on a kind-3 temporary. Both are `UNCOMP` for the same reason —
  the *cause* is unread, so the cell cannot be written.
* **`P_SYMBOL.md`'s owner** (not this lane's seam, not edited): §3's linkage
  field `([sym+0x37]>>0x15)&7` has a **second consumer**, `FUN_10bd2913`'s jump
  table at `0x10bd2a9f`, and that consumer reads **all eight** values where §3
  reads only the `{1,3}` suppression. Entry 0 of the table is a **null slot** —
  linkage 0 is unreachable by invariant and c2 would jump to address 0 if it
  ever arose.
* **`DISCLOSURE.md`'s reader**: `sym+0x04` means two different things in
  `W-STAGETAP-4` and `W-STAGETAP-6`. §2.5. Nothing was adopted here, so no row
  is owed; the ambiguity is a reading hazard, not a provenance defect.
* **Anyone porting the candidate set** (not this wave — decision 20 §2): the
  parameter to expose is **linkage class**, not variable kind. Kinds 4 and 5 are
  linkage 1 and 3, the no-COFF-record classes; everything with a COFF record is
  7/8/9. The escape flag `sym+0x05 & 2` is the second parameter and it is
  per-symbol, not per-function.
