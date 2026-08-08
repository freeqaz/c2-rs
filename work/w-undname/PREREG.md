# w-undname — PRE-REGISTRATION

Lane `w-undname`, worktree branch `wt-w-undname` off master **`5dd89969`**.
Committed **before the first change to `crates/` and before the first new
`cl.exe` cell**. Everything in §1 is a SURVEY: it reads committed artifacts
(`work/w-extdata/ref/undname/dis.txt`, `work/w-extdata/grida/*.cpp`), one IL
capture of a workload TU, and `c2rs census`/`c2rs gap` at this base — no cell
this lane authored has been compiled, and `crates/` is untouched.

Baseline at `5dd89969`: **match 14** · mismatch 0 · codegen-gap 0 · vocab-gap
857 · capture-fail 7 · **FRONTIER 13** · tests 1,262 / 36 targets · gate 5,310
fixture-verdicts.

---

## 1. The survey, verified at THIS base

### 1.1 The three-TU scan reproduces

`c2rs gap --list work/w-extdata/three.txt` at this base:

```text
  [1/3] vocab-gap    src/xdk/LIBCMT/undname.cpp  (il function decode failed)
  [2/3] vocab-gap    src/xdk/LIBCMT/osfinfo.cpp  (il function decode failed)
  [3/3] match        src/xdk/LIBCMT/vswprnc.cpp
```

`c2rs census` gives the fall-through blocker for each, and **w-extdata §6.2's
prediction that its own `.gl` widening would move `osfinfo`'s first cause is
CONFIRMED**: `gl-stop-name-not-mangled` is gone from both. What is left:

| TU | census verdict | blocker | first blocked byte |
|---|---|---|---|
| `undname.cpp` | 0/1 in class | `expr-cmp-ne` | the `20` (`!=`) of `if (node != 0)` |
| `osfinfo.cpp` | 0/1 in class | `expr-cmp-ge` | the `23` (`>=`) of its first guard |

Both are `cflow-if-n`, `eh-none`, `labels 3`, and each TU has exactly **one**
emitted function — so each converts on its own class or on none.

### 1.2 `undname`'s five priced refusals, re-verified

w-extdata's §6.1 table, checked against this base rather than inherited:

| # | claim | verified how | verdict |
|---|---|---|---|
| 1 | the recognizer: 455 IL bytes, contains a `goto` | the `4C 4F 11` anchor is at `.ex+2721` of 3176 → **455 bytes**, one segment. The decoded stream (§1.4) has `29 11 0a · 3a 10 0a` — a synthesized label whose only statement is a jump to the error label, plus a second jump to it from the `node == 0` arm and a third from the `p == 0` arm | **HOLDS** |
| 2 | a 24-word emitter | `.text` is 140 B = 35 words; prologue 5 (`prolog_len` 0x14), epilogue 6, body **24** | **HOLDS** |
| 3 | `data_sym` single → list | `?gHeapManager@@3V_HeapManager@@A` and `?pairNode_vtable@@3PAXA` in one body | **HOLDS** |
| 4 | two `addis r11,0,0` in one body | `.text+0x24` and `.text+0x40`; `data_refs_of`'s uniqueness search refuses **by name** at this tip | **HOLDS** |
| 5 | a REFLO into the scratch itself | `.text+0x4c` is `addi 11,11,0`; the low-half search matches only `addi <ARG_REG>,r11,0` | **HOLDS** |

Two things w-extdata's table did **not** carry, both survey and both cheap:

* **The frame is free at `saved_gprs = 2`**, arithmetically: `out_slots` 3 →
  `param_area_end` 80 → `size() = ceil((80 + 8·3)/16)·16 = **112**`, and
  `gpr_slot` gives `-24`/`-16` for r30/r31. That is the reference's prologue and
  epilogue word for word, restore order included (`dis.txt` `.text+0x00`,
  `+0x74`). To be re-checked against emitted bytes, not left at arithmetic.
* **`data_refs_of`'s low-half search needs TWO relaxations, not one.** Row 5
  names the `addi r11,r11,0` form; the other is that the two pairs must be
  matched to each other — `addi 3,11,0` at `+0x2c` belongs to the `lis` at
  `+0x24` and `addi 11,11,0` at `+0x4c` to the one at `+0x40`, and a global
  "unique low half" search cannot say so. Pairing is by **position**: each
  `addis` opens a pair, and the first low half after it closes it.

### 1.3 GRID A's rule cannot be exercised by ANY shipped class — this is the sequencing fact

The four GRID A cells that carry a data symbol are **out of class at this base**,
measured (`c2rs census`, `/O1 /Oi /EHsc /GR`):

```text
  a1  0/1   callseq-multiarg-sym:eof
  a2  0/1   callseq-multiarg-sym:eof     <- the smallest refutation of the writer
  a3  0/1   callseq-multiarg-sym:eof     <- `undname.cpp`'s shape in four lines
  a4  0/1   callseq-multiarg-sym:eof
  a5  1/1   in class                     (the control: no data symbol)
```

So there is **no shipped class that can emit an obj whose data reference follows
a call**, and therefore no way to give A1's new arm a graded cell except by
converting a body that has one. That is not an argument for shipping A1 alone
later; it is the reason it cannot be. **The sequencing plan is in §3.**

### 1.4 The IL body, decoded before the rung

`work/w-undname/UNDNAME_BODY.md`, committed with this file. 455 bytes, one
linear token stream, no back-reference and no value merge at a join.

### 1.5 The 24 words, read off `work/w-extdata/ref/undname/dis.txt`

```text
   w00  mr    r31,r3          this
   w01  mr    r30,r4          node
   w02  cmplwi cr6,r4,0
   w03  bt    26,->Lerr       (node == 0)
   w04  lis   r11,0           REFHI  ?gHeapManager        <- data ref 0
   w05  li    r5,K_FLAG
   w06  addi  r3,r11,0        REFLO  ?gHeapManager        (into an ARG_REG)
   w07  li    r4,K_SIZE
   w08  bl    <?getMemory>    REL24                       <- the call
   w09  cmplwi cr0,r3,0
   w10  bt    2,->Llink       (p == 0)
   w11  lis   r11,0           REFHI  ?pairNode_vtable     <- data ref 1
   w12  stw   r30,OFF_A(r3)
   w13  li    r10,K_NEG
   w14  addi  r11,r11,0       REFLO  ?pairNode_vtable     (into the SCRATCH)
   w15  stw   r10,OFF_B(r3)
   w16  stw   r11,OFF_C(r3)
   w17  lwz   r11,OFF_D(r31)
   w18  stw   r11,OFF_E(r3)
  Llink:
   w19  stw   r3,OFF_D(r31)
   w20  cmplwi cr6,r3,0
   w21  bf    26,->epilogue   (p != 0)
  Lerr:
   w22  li    r11,K_STATUS
   w23  stb   r11,OFF_F(r31)
```

**Zero words are chosen by a scheduler or a register allocator; ten immediate
fields** (`K_FLAG` `K_SIZE` `K_NEG` `K_STATUS` and the six offsets). That is
PREREG **D1** below, and it is why the fence lives in the READER (board #1706).

Three facts a general lowering gets wrong, each pinned by a `#[test]`:

1. **`.text` order is `data · callee · data`** — the externals interleave, which
   is the whole of GRID A's rule and no ordering of two loops produces it.
   Reference symbol table from index 15: `?pairNode_vtable` (referenced at
   `+0x40`), `?getMemory` (`+0x34`), `?gHeapManager` (`+0x24`) — strictly
   descending index against ascending first-reference offset, kind ignored.
2. **Three tests, two condition registers, in the order cr6 · cr0 · cr6.**
   Nothing in the source distinguishes them.
3. **The two `lis`es are hoisted by different distances** — `w04→w06` is 2 and
   `w11→w14` is 3 — so no "the high half is N words above its low half" rule
   holds even inside one body, and the pairing must be positional.

---

## 2. What is being built

| piece | where | size |
|---|---|---|
| A1: one merged undefined-external list per function, reverse first-reference order over callees ∪ data refs, kind ignored | `coff::writer` (both paths), `coff::Function::introduced_externals` | 2 loops → 1, in 2 writers |
| `check_external_order` **deleted** — the merged emission IS the rule | `c2-core/src/lib.rs`, `comdat.rs` | −1 fence, −2 call sites |
| `data_sym: Option<String>` → `data_syms: Vec<String>` | `c2-il`, and its readers | ~9 sites (unverified count, w-extdata's D7) |
| positional pairing of N `addis`/`addi` quads; a REFLO into the scratch | `data_refs_of` | 1 rewrite |
| the recognizer | `c2-il/.../shapes/alloc_init_or_fail.rs` | ~500 lines |
| the emitter | `c2-core/src/codegen/alloc_init_or_fail.rs` | 24 words |
| fixtures: the class and its `_neg` | `fixtures/cpp/wundname_*.cpp` | 2 |

`osfinfo` is a stretch and is **not** planned in detail here; §4's R3 clause
gates it.

---

## 3. SEQUENCING — how trap 0 is avoided

w-extdata declined to ship A1 because it would change the symbol table on every
obj the port emits while the gate exercised the new arm over **zero** cells.
§1.3 shows the population of cells is still zero for every shipped class, so the
plan is not "ship it and add cells later":

* **A1 ships in the same commit as the conversion that gives it a cell, or it
  does not ship.** The cell is `undname`'s own class — the only shape in reach
  whose externals interleave — plus the fixture that reproduces it.
* **If R1 (the `undname` class) declines, A1 does NOT ship** and
  `check_external_order` stays exactly as it is. Recorded here so that a lane
  reading the tip cannot mistake a surviving fence for an oversight.
* **A1 is byte-neutral on the existing population BY CONSTRUCTION**, and this is
  the argument, not a hope: `check_external_order` refuses every body in which a
  data reference follows a call, so on every obj the port has ever emitted,
  every data reference precedes every call; reverse first-reference order over
  the union then places all callees before all data names — which is where the
  two loops put them. The two rules **provably coincide on exactly the
  population the fence admits**.
* **The proof that nothing regressed is measured anyway**, three ways: the
  878-TU scan at base and at tip compared as a key→value map; `scripts/gate.sh
  --require-graded`'s per-lane `match` counts; and the fixture-verdict count,
  which must grow by exactly `18 × (new fixtures)` and no less.

---

## 4. The conversion call, as a probability

| outcome | P |
|---|---:|
| **R1 converts** — `undname.cpp` byte-exact, match 14 → **15** | **0.55** |
| R1 and R2 (`osfinfo` too), match **16**, FRONTIER 11 | 0.10 |
| nothing converts; priced decline naming which of the five stopped it | 0.45 |

Registered reasons for holding this **at** 0.55 rather than above it, given that
§1.5 is a transcription with zero scheduled words:

* Every conversion lane on this board has found at least one refusal its survey
  did not name — w-heap two, w-data an eighth, w-cfg2/w-extdata a sixth each.
  Four for four. The survey above names **seven** things (five refusals + A1 +
  the positional pairing); the base rate says to expect an eighth.
* **This lane must ship a writer change that touches every obj**, which no
  recent conversion lane did. A1 is argued byte-neutral in §3 and the argument
  is a proof about a *refusal*, not about c2 — if `check_external_order` is not
  as total as it reads, the blast radius is every matching TU.
* The `.text` byte fraction is **0.0 %** (`gap` prints `0/140`), so there is no
  partial credit to build on — the same term w-extdata registered, which cost it
  nothing.

Reasons for not holding it lower: the frame is arithmetic (§1.2), the encoders
are all present (w-extdata §2 measured `stw stb lwz li mr addi addis cmplwi bc
bl` present, and this body needs nothing else), and the body has no scheduler
freedom to get wrong.

R2 is held low because `osfinfo` needs *two* new encoders and a REFLO in a `lwz`
displacement on top of everything R1 needs, and R1's budget is the lane's.

## 4.1 Predicted metrics, if R1 converts

| metric | predicted |
|---|---|
| TU match | **15** |
| mismatch | 0 |
| codegen-gap | 0 |
| vocab-gap | 856 |
| capture-fail | 7 |
| FRONTIER | **12** |
| frontier-if-A | 134 |
| factor A / B / C | 28 / 338 / 169 (unchanged) |
| `A∧B∧C` | 27 (unchanged) |
| factor D | 15 |
| `A∧B∧C∧D` | 13 |
| function census | 711,491 (+1) |
| emitted census | 39,190 (+1) |
| `fnbyte-exact` | 36,218 |
| `fnbyte-tus-full` | 11 |
| **peer keys** | **0 vanished, 1 appeared** (`fnbyte-shape-<tag>-exact`) — w-extdata's registered miss, corrected here by reading the *appeared*-key precedent rather than the changed-key argument |

**Predicted label lead: +1**, by analogy with the two other `cflow` classes —
registered as the *prediction*, to be settled against the oracle's own
`$M2591`/`$M2592`/`$T2593`, never by analogy.

## 4.2 Predicted final test count

Registered in `work/w-undname/TEST_COUNT_PREDICTION.txt`, committed **before**
the run that checks it (board #1710a). 1,262 at the base.

---

## 5. Decline clauses, with thresholds AND sizes

| clause | fires when | size at which it fires |
|---|---|---|
| **D1 — the block plan** | any of the 24 words needs a chooser (a scheduler decision, a register allocation, a spill). Expectation: **0 words chosen, 10 immediate fields** | 1 word |
| **D2 — the reader** | the recognizer needs a block IR, a value merge at a join, or a back-reference | 1 back-reference |
| **D3 — the SHIPPING of A1** (w-extdata §7.4's correction: the clause must name what the grid decides) | A1 is **not** shipped unless it lands with a graded cell in the same commit. Not "unless GRID A is confirmed" — it is, 5/5 | 0 cells |
| **D4 — previously-emitted objs** | any obj the port emitted at the base differs at the tip. Expectation **0**, and the instruments are the 878-TU scan, the per-lane `match` counts and the fixture gate | 1 obj |
| **D5 — the `data_syms` widening** | it cannot be done without a semantic choice about which name pairs with which relocation site. Expectation: the pairing is positional and the count must match, or refuse | 1 ambiguous pair |
| **D6 — `ptr_walk_loop`'s unpaid #1638** | registered **NOT TAKEN**. Still open, still behind a matched TU, still needs its guarding fixture same-commit | — |
| **D7 — R2 (`osfinfo`)** | declined unless R1 has landed, the gate is green, and its two missing encoders (`cmplw` register form, `rlwinm` record form) plus the `lwz`-displacement REFLO are each ≤ 1 clause | any of the three > 1 clause |
| **D8 — a refusal becoming a wrong emit** | any `mismatch` anywhere. Board #232's direction. Not a threshold — the lane reverts | 1 |

---

## 6. Board rows

Range **#1740–#1759**. Unused numbers will be recorded as explicitly unminted.
