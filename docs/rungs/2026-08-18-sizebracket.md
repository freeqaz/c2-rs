# SIZEBRACKET — c2's inline size axis, LOCATED in the IL and then REFUTED as a rule: `w-dataseam`'s IL-byte cut is 39.6% wrong about real c2 at a `fnbyte-exact` cost of exactly ZERO, and the best axis this lane found is 0-for-330 on the workload and killed by a three-line `.cpp`

    Tag:       SIZEBRACKET
    Slug:      sizebracket
    Date:      2026-08-18
    Kind:      characterization — reads real `c2.dll`'s inline size decision
               (whitebox + obj grids) and lands address-cited findings under a
               prereg frozen before the first probe
    Outcome:   declined
    Fixtures:  none — characterization: what UNIT and what VALUE does c2's inline
               size test decide on, and is `w-dataseam` §6.1's `[180, 231]` IL
               segment-byte bracket a rule or a corpus fit?
    Census:    +0 — TU `match` 26 → 26. `crates/` lands **byte-identical to
               `1744ced1`** (`git diff --stat 1744ced1 -- crates/ fixtures/ scripts/` is empty); the
               measurement scaffold was reverted before the gate ran
    Record:    this file; prereg `docs/rungs/_2026-08-18-w-sizebracket-prereg.md`
               (frozen at `6ecf7b98`, the branch's first commit, before any probe)

Commits: `6ecf7b98` prereg · `dd127956` the probe instruments and the whitebox
location · this file. **The scaffold is not in the shipped tree**
(`work/w-sizebracket/scratch_grids.patch`, 274 lines over
`crates/c2-harness/src/gap/{fnbytes,scan}.rs`, reverted): the deliverable is the
grid it measured, not the code that measured it.

---

## 0. The binary question, answered in one sentence

**c2's inline size test reads `[sym+0x50]`, which arrives verbatim from the
`.gl` function record's `SIZE` field — the field the port already walks past to
reach the attribute byte — and that field is NOT the value the decision
compares, because whatever runs before the inliner REDUCES it; so the sound
implication is `SIZE < T ⇒ c2 inlined it`, which points in exactly the opposite
direction from the exemption `w-dataseam` needs, and its converse is refuted by
a three-line `.cpp`.**

The consequence for the dispatched question is measured rather than argued.
`w-dataseam`'s `[180, 231]` bracket, scored against **real `c2.dll`** on **7,667
oracle-graded workload call edges** instead of against the grader, is **wrong on
21.0 % – 39.6 % of them, and 99.9 % of the errors are in the unsound
direction** — while costing exactly **0** `fnbyte-exact`.

> **That pairing is the lane's headline and it generalizes past this fence: a
> predicate can be 39.6 % wrong about c2 and free in the metric that was used to
> choose it.** `fnbyte-exact Δ = 0` is not evidence a predicate is right. It is
> evidence the population where it is wrong is refused for some other reason
> today — which is the `w-fnbyte` #876–#879 latent hazard, not its absence.

`Outcome: declined`. The characterization landed in full; nothing entered
`crates/`.

---

## 1. What the dispatch asked, and what it got

| dispatched deliverable | outcome |
|---|---|
| 1. obj/listing-confirmed characterization across `[176, 232]`, workload profile | **delivered, and the range was the wrong one** — the flip is at emitted `(108,116]` / `.gl SIZE (97,103]` at `/O1`, well below 176. §3 |
| 2. resolve the unit mismatch | **resolved, and BOTH candidates are refuted.** The unit is a third thing, located in the IL. §2, §4 |
| 3. publish as a SERIES | **168 distinct cells, 10 series** (5 families × 2 profiles), plus a 7,667-edge workload grid. §3, §5 |
| 4. profile scope | **`/Ox` does not move the boundary — it changes WHICH UNIT SEPARATES.** §3.2 |
| 5. extend `docs/whitebox/ref/` | **done, including a correction: `P_INLINE.md` §2.1's four addresses are in the wrong function.** §2 |
| 6. ship if derived, else decline | **DECLINED.** §6, priced two-sided in §7 |

---

## 2. The whitebox half — `[sym+0x50]` LOCATED, and `P_INLINE.md` §2.1 corrected

Both amendments are landed in [`docs/whitebox/ref/P_INLINE.md`](../whitebox/ref/P_INLINE.md)
§2.1's ⛔ box and new §2.1a/§2.1b/§2.1c, beside the original text rather than over
it (`ref/README.md` §2.1). Summarised here; the page is authoritative.

### 2.1 `P_INLINE.md` §2.1's four addresses are in a different function

`FUN_10b5fb5f` is 377 bytes, spanning `0x10b5fb5f`–`0x10b5fcd7`. **All four
addresses §2.1 quotes are past its end** and land in `FUN_10b5fcd8`, the page's
own *"POGO-only profitability model"*. The bytes there are a different
computation (`movzx ecx,cl`, `imul edx,[0x10c3f58c]`).

The real test, inside the candidacy function:

```
10b5fc7e:  39 1d 10 e3 c2 10     cmp    DWORD PTR ds:0x10c2e310,ebx   <- FAVOUR-SPEED (ebx = 0)
10b5fc84:  75 33                 jne    0x10b5fcb9                    <- set => the size test is SKIPPED
10b5fc86:  0f b7 46 50           movzx  eax,WORD PTR [esi+0x50]       <- the callee's count
10b5fc8a:  3b 05 18 63 c4 10     cmp    eax,DWORD PTR ds:0x10c46318   <- the ceiling
10b5fc90:  7c 27                 jl     0x10b5fcb9                    <- below it => candidate
```

**The reading carries; the locations do not**, and one detail does not carry
either: §2.1's `__forceinline` mask reads `0x2000`, and the mask at
`0x10b5fcc1` is `0x2080`. This is `ref/README.md` §6.2's lesson in a second
place — *"address A is inside function F"* is a claim to check against F's entry
**and size**.

### 2.2 The field, and it is one the port already reads past

**There is exactly ONE 16-bit store to `[reg+0x50]` in the entire image:**

```
10b9bf57:  call 0x10c1f9e9   (il-read-varint32)  -> [esi+0x54]
10b9bf5f:  call 0x10c1f9e9   (il-read-varint32)  -> [esi+0x58]
10b9bf67:  call 0x10c1f9a6   (il-read-varint16)
10b9bf6c:  66 89 46 50   mov WORD PTR [esi+0x50],ax    <-- THE ONLY ONE
10b9bf70:  call 0x10c1f91b
10b9bf78:  89 46 4c      mov DWORD PTR [esi+0x4c],eax
```

It is inside `FUN_10b9b8e9`, which [`ADDR.tsv`](../whitebox/ref/ADDR.tsv)
**already** labels *"p2symtab `.gl` record reader; reads the emit flag word
`+0x4c` at `0x10b9bf70`"* — the same three instructions, written down from the
other side of the seam before anyone was looking for this one.

Lined up against `c2_il::func::gl::gl_function_attrs`' own record comment
(`00 <name> 00 <TYPE> 80 01 10 00 00 00 00 80 <LE32 offset> <SRCPOS> <SIZE> <ATTR>`):

| `.gl` field | reader | destination |
|---|---|---|
| `80 <LE32 offset>` | `i32c` | `[sym+0x54]` |
| `SRCPOS` | `i32c` | `[sym+0x58]` |
| **`SIZE`** | **`i16c`** | **`WORD [sym+0x50]`** |
| `ATTR` | `0x10c1f91b` | `[sym+0x4c]` — the port's `FN_FLAG_INLINABLE` byte |

> **`[sym+0x50]` is the `.gl` `SIZE` field. The port has been stepping over the
> input to c2's inline size test in order to reach the byte one field later.**

`[O]`, not `[R]`: decoded out of real `.gl` bytes by
`work/w-sizebracket/glsize.py` and confirmed **linear in source content** —
empty `int f(int)` is **19**, and each statement adds a fixed increment
(`s ^= a;` +4 · `s = -s;` +5 · `s = s<<3;` +6 · `s = s*3+1;` +8 ·
`s = (s*K+C)^(s>>j);` +12 · `if (s>3) s=1;` +13). Two framings of the record
(the incumbent `gl_offset_framed` and #2783's relaxed one) agree on **8 of 8**
control cells.

### 2.3 A live limit of the port's attribute map, found on the way past

`gl_function_attrs` **refuses the whole file** when the `SIZE` byte is `>= 0x80`
— it assumes SIZE is always one byte, and the `i16c` reader's `0x80` escape then
puts `ATTR` two bytes further along. The escape is not exotic: `SIZE` crosses
128 at about fourteen statements (cell `arith_016` reads **147**), and on the
878-TU workload the escape fires on **309 of 7,667** graded call edges — **4.0 %**.
The refusal is correct and fail-closed; what is new is that it is **measured
live** rather than hypothetical. Board **#3274**.

---

## 3. The series — 168 cells, 10 series, both profiles

Every cell: generate the `.cpp`, `c2rs capture --keep-il` for the `.gl`,
`c2rs compile --keep-obj` for real c2's own obj under wibo, then read the
callee's `SIZE` out of `.gl` and ask the obj **whether the CALLER's `.text`
COMDAT carries a `REL24` naming the callee** — `w-fence2` GRID-W's observable,
per cell. Driver `work/w-sizebracket/series.py`; log
`work/w-sizebracket/series.jsonl`; **every table below is re-derived from that
log, never accumulated** (`docs/rungs/README.md` probe rule 2).

| family | callee body | cells `/O1` | cells `/Ox` |
|---|---|---:|---:|
| `arith` | `n` × `s = s*K+C;` — composes to ONE affine function | 16 | 16 |
| `mix` | `n` × `s = (s*K+C)^(s>>j);` — does not compose | 21 | 19 |
| `fine` | `mix` at 6, then `n` × `s ^= (a>>j);` — a 6-unit `SIZE` step | 13 | 19 |
| `static` | `arith` with internal linkage | 16 | 16 |
| `loop` | a `for`-bodied callee (`WB_INLINE_FINDINGS` F9's class) | 16 | 16 |

**168 distinct cells, 0 errored, 120 `inlined` / 48 `kept`.**

### 3.1 `/O1` — the workload's own profile

`mix`, stepping `SIZE` by 12:

| `n` | `.gl SIZE` | emitted `.text` | real c2 |
|---:|---:|---:|---|
| 5 | 79 | 84 | inlined |
| 6 | 91 | 100 | inlined |
| **7** | **103** | **116** | **kept** |
| 8 | 115 | 132 | kept |

`fine`, stepping `SIZE` by 6 from the same prefix:

| `n` | `.gl SIZE` | emitted `.text` | real c2 |
|---:|---:|---:|---|
| 0 | 91 | 100 | inlined |
| 1 | 97 | 108 | inlined |
| **2** | **103** | **116** | **kept** |
| 3 | 109 | 124 | kept |

> **The `/O1` bracket is `(108, 116]` in emitted `.text`.** That sits inside
> `WB_INLINE_FINDINGS` F2's EXTERNAL `/O1` bracket `(100, 116]` and **shares its
> top** — an independent reproduction of that row, on new cells and a new
> generator, by a lane that was not looking for it.

**And the dispatched range `[176, 232]` contains no boundary at all.** It was
inherited from `w-dataseam`'s IL-byte sweep, which §4 shows is a different unit.

### 3.2 `/Ox` — and the disagreement is not a moved boundary, it is a CHANGE OF UNIT

`w-section` found `/Ox` disagreeing with `/O1` on 7 of 8 fields; `w-dagorder`
found it inverting the allocator order. Here it does something the repo has not
recorded before.

| cell | `.gl SIZE` | emitted `.text` | real c2 |
|---|---:|---:|---|
| `fine_004_Ox` | 115 | **304** | inlined |
| `fine_005_Ox` | 121 | **320** | **inlined** |
| `fine_006_Ox` | 127 | **196** | **kept** |
| `mix_008_Ox` | 115 | **320** | **inlined** |
| `mix_009_Ox` | 127 | **212** | **kept** |

> **At `/Ox` a 320-byte callee is INLINED beside a 196-byte one that is KEPT.**
> The emitted size is not merely a worse predictor at `/Ox` — it is
> **anti-correlated at the crossing**.

| profile | emitted `.text` separates? | `.gl SIZE` separates (non-folding families)? |
|---|---|---|
| `/O1` | **yes**, `(108, 116]` | no — `arith` is inlined to `SIZE = 211` |
| `/Ox` | **no**, inverted | `(121, 127]`, and `arith` still breaks it |

Consistent with §2.1's favour-speed bit turning this very test off, and with
`/Ox`'s growth transforms running *after* the decision so the emitted body stops
being a witness to what the inliner saw. **No single-profile size claim on this
axis may be quoted at the other profile.** Board **#3273**.

### 3.3 The two families that never flip, at either profile

`static` (16+16 cells) and `loop` (16+16) are **inlined in all 64 cells**, to
`SIZE = 211` / emitted 92 and to `SIZE = 96` / emitted 104 respectively. They are
reported because a family that never crosses is a bound, not a null: F1's STATIC
ceiling `(300,308]` is above everything `static` reaches here, so these cells
**cannot** discriminate and are not counted as agreement.

---

## 4. The unit, resolved — and it is a THIRD thing, with the discriminating pair to prove it

### 4.1 The `.gl SIZE` field is not the value the decision compares

The matched pair, at the workload profile:

| cell | `.gl SIZE` | `.ex` bytes | emitted `.text` | real c2 |
|---|---:|---:|---:|---|
| `arith_012_O1` — 12 × `s = s*K+C;` | **115** | **3,233** | 28 | **inlined** |
| `mix_008_O1` — 8 × `s = (s*K+C)^(s>>j);` | **115** | **3,221** | 132 | **kept** |

**Identical `SIZE`, `.ex` totals within 12 bytes, and OPPOSITE verdicts — with
the LARGER IL body the one c2 inlined.** The `arith` chain composes to a single
affine function so c2 folds it before the inliner looks; `mix` does not. `arith`
is inlined at **every** rung to `SIZE = 211` at both profiles.

**And it is not an inference from a relocation count — c2 narrates both, in its
own `/FAsc` listing, from the same source line.** Both callers are the identical
string `int caller(int a) { return callee(a) + 7; }`:

```
;; arith_012_O1.cod   -- callee's .gl SIZE = 115
?caller@@YAHH@Z PROC NEAR                      ; caller, COMDAT
; 17   : int caller(int a) { return callee(a) + 7; }
  00000  3d604667   lis    r11,18023
  00004  3d405ccc   lis    r10,23756
  00008  616bdaaf   ori    r11,r11,55983
  0000c  614a12af   ori    r10,r10,4783
  00010  7d6359d6   mullw  r11,r3,r11
  00014  7c6b5050   subf   r3,r11,r10
  00018  4e800020   blr
?caller@@YAHH@Z ENDP                           ; no `bl`, no relocation, no .pdata

;; mix_008_O1.cod     -- callee's .gl SIZE = 115
;       COMDAT .pdata
$T2556  DD  ?caller@@YAHH@Z
        DD  040000903H
?caller@@YAHH@Z PROC NEAR                      ; caller, COMDAT
; 13   : int caller(int a) { return callee(a) + 7; }
  00000  7d8802a6   mflr   r12
  00004  9181fff8   stw    r12,-8(r1)
  00008  9421ffa0   stwu   r1,-60h(r1)
.endprolog
  0000c  48000001   bl     ?callee@@YAHH@Z     <-- KEPT
  00010  38630007   addi   r3,r3,7
  ...
```

Twelve statements collapsed into **two constants and a `mullw`** on one side; a
framed call with a `REL24`, a `.pdata` record and two `$M` labels on the other.
Confirmed independently in the obj (`gt_dump.py`: `nrel=0` vs
`nrel=1  REL24 -> ?callee@@YAHH@Z`). Listings in `work/w-sizebracket/lst/`.

So `[sym+0x50]` is **initialized** from `SIZE` and then **reduced** by whatever
runs before the inliner. `SIZE` is an *upper bound* on the tested quantity.

### 4.2 The implication that survives — and it points the wrong way

Folding only reduces, so:

> **`.gl SIZE < T` ⇒ the tested count `< T` ⇒ c2 INLINED it.**

Checked over every cell: **`/O1`, T = 98 — 45 cells fire, 0 counterexamples.
`/Ox`, T = 122 — 55 cells fire, 0 counterexamples.**

The **converse** — *"`SIZE` large ⇒ c2 kept it"* — is what the fence needs, and
it is false: **14 counterexamples at `/O1` and 20 at `/Ox`** among the cells,
`arith_020_O1` at `SIZE = 179` the sharpest.

> **The one direction this axis supports soundly is the one the exemption cannot
> use, and the direction the exemption needs is refuted by a three-line `.cpp`.**
> Board **#3272**.

### 4.3 Answering the dispatch's unit question directly

| candidate | verdict |
|---|---|
| **emitted `.text` bytes** (`INLINE_DECLINE_BYTES` 128, `INLINE_DECLINE_LOOP_BYTES` 80) | a **downstream** proxy: faithful at `/O1` (§3.1 reproduces F2), **inverted at `/Ox`** (§3.2). Sound where it is used, because the parser gates the mode |
| **IL segment bytes** (`w-dataseam`'s `[180, 231]`) | an **upstream** proxy for the field's *initial* value, and §5 prices it against the oracle at 21–40 % wrong |
| **the `.gl` `SIZE` field** | the initial value **exactly**, and still not the tested one (§4.1) |
| **what c2 tests** | the post-fold count in `[sym+0x50]`. **Not readable from the container**, because the reduction is c2's own work |

---

## 5. GRID-S — `w-dataseam`'s cut scored against real c2 on 7,667 workload call edges

`work/w-sizebracket/scratch_grids.patch` extends GRID-W's per-edge question with
the two units under test. Base `1744ced1`, 878 TUs, `--jobs 16`, cache **870 hit
/ 8 miss / 0 uncacheable**, workload `ccd4c80362f1` (clean), **0 `unknown`
arms of 7,667**. Tables in `work/w-sizebracket/GRID-S.txt`, aggregation
`work/w-sizebracket/xs.py`, derived from `scan_xs3.jsonl`.

### 5.1 The two units, banded

**Emitted `.text` (GRID-W's unit), 16 B bands** — mixed only in 64–95, and
**ZERO `inlined` edges above 95 B** against 1,268 `kept`:

| band | kept | inlined |
|---:|---:|---:|
| 0–63 | 1 | 5,829 |
| 64–79 | 9 | 503 |
| **80–95** | **137** | **67** |
| **96+** | **1,121** | **0** |
| **TOTAL** | **1,268** | **6,399** |

**IL segment length (`w-dataseam`'s unit), 16 B bands** — mixed from 224 all the
way to 576, with `inlined` edges at every band up to 576:

| band | kept | inlined | | band | kept | inlined |
|---:|---:|---:|---|---:|---:|---:|
| 176 | 0 | 396 | | 288 | 1 | 6 |
| 192 | 0 | 278 | | 320 | 15 | 2 |
| 208 | 0 | 523 | | 464 | 7 | 3 |
| 224 | 4 | 588 | | 496 | 2 | 10 |
| 240 | 25 | 301 | | 512 | 31 | 25 |
| 256 | 29 | 434 | | 544 | 17 | 15 |
| 272 | 66 | 676 | | 576 | 5 | 1 |

### 5.2 The cut, scored — this is the number the lane was sent for

A size exemption says *"IL segment > N ⇒ assume c2 KEPT the call"*. Scored
against c2's own verdict:

| cut `N` | asserts KEPT, c2 INLINED | asserts INLINED, c2 KEPT | **WRONG** | of | **err** |
|---:|---:|---:|---:|---:|---:|
| 128 | 4,986 | 1 | **4,987** | 7,667 | **65.0 %** |
| **180** — `w-dataseam`'s lower end | **3,036** | 1 | **3,037** | 7,667 | **39.6 %** |
| 192 | 2,891 | 1 | 2,892 | 7,667 | 37.7 % |
| 224 | 2,090 | 1 | 2,091 | 7,667 | 27.3 % |
| **231** — `w-dataseam`'s upper end | **1,607** | 3 | **1,610** | 7,667 | **21.0 %** |
| 256 | 1,203 | 30 | 1,233 | 7,667 | 16.1 % |

**Across the whole of `[180, 231]` the cut is wrong on between one fifth and two
fifths of the call edges c2 graded, and 99.9 % of the errors are in the unsound
direction** — asserting that c2 kept a call it in fact inlined. Board **#3271**.

> ### ⚠ The POPULATION is a SUPERSET of the rule's own reach, and the rate is stated against it rather than smuggled
>
> GRID-S grades **every IL call edge to a callee this TU defines** — GRID-W's
> population, `TuContext::mentions`. `w-dataseam`'s rule fires on a **subset**:
> it also requires `∉ tu_modelled_callees` and `cflow_key ≠ eh-state1`, and its
> realized reach was **2,875 bodies**. So **39.6 % is the error rate of the cut
> as a size axis, not necessarily its error rate inside the fence's own
> conjunction.** Measuring the latter needs the conjunction re-expressed
> per-edge, which this lane did not build.
>
> **Three things are unaffected by the scoping and they are the load-bearing
> ones.** (a) The *mechanism* — IL that folds away — is unconditional, and §4.1's
> matched pair and §4.2's 34 counterexamples are constructed source files that no
> population argument can reach. (b) The *direction*: 3,036 of 3,037 errors at
> cut 180 are the unsound one, and a conjunction can only remove edges, never
> flip a verdict. (c) The comparison between the two units is on **identical**
> edges, so *"the emitted axis has 0 `inlined` above 95 B and the IL axis has
> 1,607 errors at its best cut"* is a like-for-like statement whatever the
> denominator.
>
> Registered bias #3 (*"reading a null as a negative result"*) pointed at the
> other direction; this is the same discipline applied to a positive one.

### 5.3 Why the IL unit fails, in one cell of the joint table

The joint `(emitted band, IL band)` cell says it without interpretation:

| emitted `.text` | IL segment | kept | inlined |
|---|---|---:|---:|
| **0–31 B** | 96–127 | 0 | 1,337 |
| **0–31 B** | 128–159 | 0 | 1,173 |
| **0–31 B** | 192–223 | 0 | 652 |
| **0–31 B** | **224–255** | 0 | **750** |
| **0–31 B** | 256–319 | 0 | 419 |

**A callee that emits under 32 bytes of `.text` routinely carries 224–319 bytes
of IL.** That is `arith`'s mechanism at workload scale, and it is named: 3,036
`XS-INLINED-BIG-IL` witnesses are printed by the scan, e.g.

```
src/lazer/meta_ham/AccomplishmentConditional.cpp
  ??1?$list@UAccomplishmentCondition@@… -> ??1?$_List_base@UAccomplishmentCondition@@…
  il=223  ref=4
src/lazer/game/HamUser.cpp
  ?SplitMs@Timer@@QAAMXZ -> ?Split@Timer@@QAAXXZ            il=287  ref=52
```

**223 bytes of IL that emit four bytes of code.** No cut on IL length can see
that, and `[180, 231]` sits directly on top of the population.

### 5.4 The `.gl SIZE` axis at workload scale — 0-for-330, and still not shippable

The same 7,667 edges, banded by `.gl SIZE` (SIZE resolved for **7,667 of
7,667**; the first run resolved only 74 because it used the incumbent framing,
which board **#2783** measured as truncating on large `.gl` — corrected to the
relaxed framing and the control in §8.2 pins that the two agree where both fire):

| band | kept | inlined | | band | kept | inlined |
|---:|---:|---:|---|---:|---:|---:|
| 8 | 0 | 1,367 | | 72 | 12 | 2 |
| 16 | 1 | 1,389 | | 80 | 6 | 7 |
| 32 | 52 | 2,003 | | 88 | 66 | 66 |
| 48 | 232 | 490 | | **96+** | **330** | **0** |
| 64 | 539 | 10 | | | | |

> **Zero `inlined` edges at `.gl SIZE ≥ 96`, over 330 edges and 7,667 total.**
> Against the IL-segment axis's 1,607 errors at its best cut, that is three
> orders of magnitude better **on the same edges**.

**And it is still refused**, because §4.2 exhibits **34 counterexamples to
exactly that implication** across two profiles, in three-line source files. The
workload's 0-for-330 is a statement about the workload.

> This is board **#1148**'s lesson pointing at the best result the lane produced:
> *"a recorded unreachability is a statement about the cells someone thought
> of… the route around it is one line of C++ nobody had written."* Here the
> route around it **was** written, in the same lane, before the rule was
> proposed. Board **#3275**.

---

## 6. The decision, against the frozen rule

Prereg §2's decision rule, applied clause by clause:

| clause | verdict |
|---|---|
| **D1** — the cut is selected from an ORACLE cross-tab, never from `fnbyte-exact` | **honoured, and it flipped the answer.** §5.2 is the oracle's own score and it disqualifies the cut. Had the selection statistic been `fnbyte-exact`, `[180,231]` reads free |
| **D2** — stated in the unit c2 decides on | **cannot be satisfied.** §4 shows the unit is a post-fold count that is not a container fact |
| **D3** — a SERIES, not a cell | honoured: 168 cells, 10 series, plus 7,667 workload edges |
| **D4** — both profiles, disagreement reported | honoured: §3.2, and the disagreement is a change of unit |
| **D5** — ship only if D1 and D2 both hold | **D2 fails. DECLINE.** |

---

## 7. The two-sided price of THIS decline

Per `CLAUDE.md` #1042 / NC-5. Counted, because a decline that does not price
itself is FAILED.

| | cost | in the goal's units |
|---|---|---|
| **shipping `lift, sz∈[180,231]`** | a predicate **21.0–39.6 % wrong about real c2** on its own workload, 3,036 errors in the unsound direction at the lower end, **invisible to `fnbyte-exact` by construction** | 0 measured, and the measurement cannot see the defect |
| **shipping `lift, .gl SIZE ≥ 96`** | 0-for-330 on the workload; **34 counterexamples already written down**, so it ships a rule this lane can already break | 0 measured, 34 known-wrong shapes |
| **declining (what this rung does)** | `w-dataseam`'s 1,807 latent-wrong bodies stay in the census's accepted set (#876–#879); the 1,189-body removal is foregone; clause (c2) stays a named exemption | **0 today, 1,807 latent** |

**The decline's own cost is real and is 1,807 bodies of latent hazard** — the
same number `w-dataseam` paid, paid a second time. What makes it the right call
is that this lane did not merely fail to derive the constant: **it measured that
the constant's unit cannot carry the rule**, and it hands the follow-on the axis
that *can* (§9).

Two-sided the other way, and stated because it is the argument *for* shipping:
1,189 wrong bodies is 47.6 % of every wrong body the instrument grades, and the
refusal is 64.6 % enriched. **That is a large prize and it is still not worth a
predicate whose error is 39.6 % and structurally invisible to the acceptance
test.** A wrong emit scores strictly below the refusal it replaced
(`PROGRESS_METRIC.md`), and the 3,036 misjudged edges are precisely the
population the next `functions()` widening would ship.

---

## 8. Instrument discipline

### 8.1 The `fnbyte-*` drift bracket (#3249) — closed at ZERO across three scans

#3249 requires the base re-read immediately before the tip, back to back, with
the cache state stated. This lane ran the **whole** 878-TU workload three times
on three different binaries inside one session:

| reading | `fnbyte-exact` | `differs` | `reloc-differs` | `refused-parse` | `match` | `mismatch` | keys | cache |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `scan_xs` (GRID-S v1) | **35,899** | 1,958 | 531 | 113,447 | 26 | 0 | **394** | 870 hit / 8 miss |
| `scan_xs2` (+ `.gl SIZE`, strict framing) | **35,899** | 1,958 | 531 | 113,447 | 26 | 0 | **394** | 870 hit / 8 miss |
| `scan_xs3` (+ `.gl SIZE`, relaxed framing) | **35,899** | 1,958 | 531 | 113,447 | 26 | 0 | **394** | 870 hit / 8 miss |

**Identical in every column.** The dispatch's `fnbyte-exact` **35,899** is
reproduced exactly — **P1 HIT**, and the ±2 floor did not fire at all. `match`
26 / `mismatch` 0 / `codegen-gap` 0 / `vocab-gap` 844 / `capture-fail` 8 on all
three — **P2 HIT**.

**Key count `394`, with `grep -cE '^ *gap-metric \S+ \S+$'`** — the anchored
pattern, on all three logs. #3269's artifact was not reached for. **P3 HIT.**

**Grids frozen by content hash, and the corpus did NOT move again:**

| input | sha256 | vs `w-dataseam` |
|---|---|---|
| `work/dc3-workload/files.txt` (878 TUs) | `4996839bf89780a2dea9ed005450d8953961355a9eb2292cc1bc22572a6853b6` | **identical** |
| `work/dc3-workload/flags.txt` | `fa8ba48aa21229773116bf0decff3b7e9e5e7f7ee356c3e347c506038ffbcb48` | **identical** |
| `dc3-decomp` head | `ccd4c80362f1` (clean) | **identical** |

So every `w-dataseam` figure this rung quotes is quoted against the same corpus
it was measured on. §12.4 of that rung — *"workload-dated, re-read rather than
re-quote"* — is discharged rather than inherited.

### 8.2 The instrument that was wrong, caught by its own denominator

The first `.gl SIZE` scan (`scan_xs2`) resolved a SIZE for **74 of 7,667** call
edges and reported `absent` for 7,593 — and the 74 it did resolve said **69
`kept` under `SIZE < 96`**, which would have refuted §5.4's headline outright.

The cause was the record framing: `codec::gl_offset_framed` pins `gl[o-5] ==
0x10`, which board **#2783** measured as a **value window on a rising counter**
— *"5 of 5 on a 2.5 KB `.gl` and 36 of 811 on a 65 KB one"*. Re-run with
`gl_offset_framed_relaxed`, resolution went **74 → 7,667, `absent` 0**.

**It was caught by the denominator being printed, not by the answer looking
wrong** — `absent: 7593` beside `no: 72` is not a result anyone can read as
success. STATUS trap 0's *"a green control is a statement about the population it
ran over"*, in the smallest possible instance.

**The control, pinned by NAME, not by count**: the two framings are run over
eight of this lane's own probe cells, whose `SIZE` is independently known from
§2.2's linearity. `arith_012` 115/115 · `mix_008` 115/115 · `mix_007` 103/103 ·
`fine_002` 103/103 · `fine_005_Ox` 121/121 · `fine_006_Ox` 127/127 ·
`arith_024` 211/211 · `loop_016` 80/80. **0 disagreements.**

### 8.3 Probe soundness (#3219 / #3231)

`C2RS_REQUIRE_TOOLCHAIN=1` on the suite and on **every** one of the 176 probe
invocations (`series.py` sets it in the child environment). The suite's
`SKIP: toolchain absent` count is **0** over 45 targets — the assertion is on
the *executed* count and the log's SKIP count, not on the exit code.

The probe cells are their own strongest environment control: an unprovisioned
worktree cannot produce a `.gl` with `SIZE = 115` in it, and 168 of 168 cells
returned both an IL bundle and an obj with a decodable symbol table.

### 8.4 Comparators NOT used, and why

* **The per-symbol `fnbyte-differs` set compare is VOID here** (#3237,
  `w-dataseam` §9) — this lane's scaffold changes no admitted population, and
  it is not used at all; the identity check is the three-scan column table above.
* **`fnbyte-reloc-differs` as a monotone control** — `w-dataseam` §12.2 measured
  it moving **up** by 67 under a change that only refuses. Not used.
* **Release-binary sha256 across worktrees** — voided by #3224. Not quoted.

---

## 9. Prereg scoring

| id | P | outcome |
|---|---:|---|
| **P1** | 0.70 | **HIT** — `fnbyte-exact` **35,899** exactly, three times |
| **P2** | 0.95 | **HIT** — 26 / 0 / 0 / 844 / 8 in every build |
| **P3** | 0.90 | **HIT** — **394**, anchored pattern, all three logs |
| **P4** | 0.65 | **MISS** — the IL-segment cross-tab is **not** monotone: mixed from band 224 to band 576, `inlined` edges at every band in between (§5.1) |
| **P5** | 0.70 | **HIT, and by more than predicted** — the emitted-byte table has **0** `inlined` above 95 B; the IL table's mixed region is 350 bands wide |
| **P6** | 0.50 | **MISS, and the frame was wrong** — no oracle-derived IL cut exists to land inside `[180,231]`, because no IL cut separates. The question the prediction asked has no answer |
| **P7** | 0.55 | **MISS** — the ratio is not concentrated in `[1.4, 2.4]`; §5.3 has 750 edges at IL 224–255 emitting under 32 B, a ratio above 7 |
| **P8** | 0.35 | **HIT** — the IL **does** carry the count, it is the `.gl` `SIZE` field, and `[sym+0x50]` is loaded from it verbatim (§2.2) |
| **P9** | 0.60 | **MISS** — the flip is at emitted `(108,116]` / `SIZE (97,103]` at `/O1`, **below** the dispatched `[176,232]` window entirely (§3.1) |
| **P10** | 0.80 | **HIT, and understated** — `/Ox` does not move the boundary, it changes which unit separates (§3.2) |
| **P11** | 0.45 | **MISS as written** — the lane's registered most-likely outcome was `built`; it is `declined`, which prereg §2 D5 required once D2 failed |
| **P12** | 0.30 | **MISS** — nothing shipped |
| **P13** | 0.85 | see §10 |
| **P14** | 0.55 | **not decidable as written** — `?MakeString@@YAPBDPBD@Z` is reached through `ContentMgr_Xbox.cpp`, whose `data-sym-strlit-fenced` population `w-dataseam` §3 measured at **1** on this corpus. Reported as **NOT MEASURED** rather than scored either way |
| **P15** | 0.55 | **HIT** — the tested unit is neither candidate. It is a pre-codegen count that both byte units proxy, and §4.1's matched pair is the demonstration |

**5 HIT, 6 MISS, 1 not decidable, 1 in §10** — and the two most consequential
misses (P4, P6) are misses because **the axis the prediction presupposed does
not exist**, which is the finding rather than a failure of estimation.

### 9.1 Mutant colours

**MS1–MS6: NOT RUN, by the pre-registered condition.** Prereg §6 froze them as
applying to a *shipped* `crates/` predicate and required that a lane which ships
nothing report them as not run in those words. The Outcome is `declined` and
`crates/` is byte-identical to `1744ced1`, so there is no predicate to mutate.
**Registered colours observed: none. Registered colours contradicted: none.**

MS6 — registered as a *predicted GREEN* — is worth a sentence anyway: it asked
whether a derived cut would be distinguishable from `w-dataseam`'s fitted 231 on
this workload. §5.2 answers a sharper version: **231 is distinguishable from
correct by 1,610 edges**, which is the question MS6 was a proxy for.

---

## 10. Gate evidence

`crates/` is **byte-identical to `1744ced1`** — `git diff --stat 1744ced1 --
crates/` is empty — so this lane's required-zero byte delta is satisfied by
construction and the identity control proves the scaffold left nothing behind.

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **1,666 passed / 0 failed / 45 targets**, 1 ignored, **`SKIP: toolchain absent` count 0** (`work/w-sizebracket/tests.log`) — identical to the dispatch's registered baseline `1,666 / 0 / 45` |
| `scripts/gate.sh --jobs 16 --require-graded` | **`GATE: PASS (HATCH-RED REFUSED)` — 18/18 lanes ran and every one graded a corpus** (`work/w-sizebracket/gate.log`) |
| 878-TU workload scan | `match` **26** / `mismatch` **0** / `codegen-gap` **0** on **all three** scans (§8.1) |
| identity control | **394** anchored keys, all three scans, every value identical |
| environment control | §8.3 — `C2RS_REQUIRE_TOOLCHAIN=1` on the suite **and** on all 176 probe invocations; 0 SKIP lines over 45 targets |
| required-zero byte delta | `git diff --stat 1744ced1 -- crates/ fixtures/ scripts/` is **empty** |
| release-binary sha256 | **not quoted** — board **#3224** voids it across worktrees |

### 10.1 The gate, quoted as it printed

| row | verdict | graded/total | match | mismatch |
|---|---|---|---:|---:|
| the 18 mode lanes | **18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT** | 386/386 each, **6,948** fixture-verdicts | — | **0 on all 18** |
| `expr-sweep` | **PASS** | 19,556 / 19,556 | 19,460 graded | **0** |
| `mode-cross` | **PASS** | 90,812 / 90,812 | 90,424 graded | **0** |
| **`debug-lane`** (DEBUG profile) | **PASS** | 18 / 18 lanes, 6,948 verdicts | 2,423 | **0 mismatch, 0 PANIC** |
| `ladder-red` | PASS | 5/5 arms (2 green controls) | 3 | n/a |
| `hatch-red` | **REFUSED — `HATCH-STALE`** | 0/14 arms | 0 | n/a |

Graded tree **`e8a9edfa0947`**, **740** files under `crates fixtures scripts`,
0 gitignored byproducts unhashed.

**`hatch-red` is reported as the gate reported it, not rounded up to PASS.** It
is `HATCH-STALE` (board **#1389**) — a **pre-existing** state of that arm which
no change of this lane's could have moved, because `crates/`, `fixtures/` and
`scripts/` are byte-identical to `1744ced1`. `w-dataseam` recorded the same row
in the same state at the previous master.

**The debug-profile row is the one that could have indicted a change here**, and
it is a pass on the merits: it is the only row where a false `debug_assert!` or a
wrapping overflow can execute, and it reports **0 panics on 18 of 18 lanes**.

### 10.2 The per-lane gate-count identity diff, with the range length asserted

`crates/`, `fixtures/` and `scripts/` are byte-identical to `1744ced1`, so the
identity is against **master itself** and every count above is master's own.
**Range length asserted**: `git rev-list --count 1744ced1..HEAD` = **8**
(prereg · instruments · whitebox · board · rung · 3 rung corrections), and
`git diff --name-only 1744ced1..HEAD | grep -vE '^(docs/|work/)'` is **empty** —
every file this lane touched is under `docs/` or `work/`.

| figure | `w-dataseam` tip (at `44794fa4`) | this lane (at `1744ced1`) | reading |
|---|---|---|---|
| lanes | 18/18 PASS | **18/18 PASS** | identical |
| fixture-verdicts | 6,948 | **6,948** | identical |
| sweep | `checked=19556 mismatches=0 graded=19460 ungraded=96` | **identical** | identical |
| cross | `checked=90812 mismatches=0 graded=90424 ungraded=388` | **identical** | identical |
| `debug-lane` | `PASS 18/18 2423 0` | **identical** | identical |
| graded tree hash | `5b550a38d90b` (738 files) | `e8a9edfa0947` (**740** files) | **DIFFERENT, and expected** |

**The tree hash differs and it is not a defect** — the two lanes sit on different
masters. `w-gateperf` merged between them and changed `crates/` and `scripts/`
(+2 files). The discriminating check here is that **every graded count is
identical across a master change that moved the hash**, plus the empty
`git diff` above. Quoting the hash as if it were comparable across bases is the
`#3224` error one level up, and it is named rather than committed.

---

## 11. Found and not taken

### 11.1 The one thing to do next, and it is NOT another size lane

`w-dataseam` §12.1 handed this lane a *"52-wide bracket with both ends
measured"*. **That handover is withdrawn**: the bracket's unit does not carry the
rule, and a lane sent to narrow `[180, 231]` further would be narrowing a
quantity that is 21–40 % wrong at every value in it.

What is worth doing instead, in order:

1. **Nothing on this axis, until someone can read the post-fold count.** §4 says
   what the tested quantity is; the port does not have it and cannot compute it
   without doing c2's own folding.
2. **`.gl SIZE ≥ 96` as a REFUSAL, not an exemption.** The direction §4.2 proves
   sound — `SIZE < T ⇒ inlined` — is a licence to *expand*, which is
   `splice::INLINE_UNBOUNDED_BYTES`' business, not this fence's. Whether it
   widens the splice is a separate, separately-priced rung.
3. **Fix `gl_function_attrs`' `0x80` escape** (§2.3). It refuses the whole file
   on 4.0 % of graded call edges' callees, it is a two-line decode, and it is
   the only item here with no measurement problem in front of it.

### 11.2 `INLINE_DECLINE_BYTES` is confirmed, sideways

§3.1 reproduces `WB_INLINE_FINDINGS` F2's `(100,116]` on new cells, and §5.1
reproduces GRID-W's *"zero `inlined` above 95 B"* on a corpus 61 commits later
(1,268 kept / 6,399 inlined here vs 1,101 / 6,451 there — the populations moved,
the boundary did not). **128 stands.** §3.2 also supplies the reason its
docstring's `/O1`-only scope is load-bearing in a way the docstring understates:
at `/Ox` the constant is not merely *"wrong"*, its unit is inverted.

### 11.3 The dispatch's own framing was wrong in one place, and it is recorded

The brief said *"Both ends are already pinned — lower **180** by a witness…
No size-constant lane in this repo has ever started with both ends pinned."*
Both ends were pinned **in a unit that does not decide**, which is a stronger
form of `w-dataseam` §12.6's finding: **a dispatched instruction is a prediction
and should be scored like one.** The brief's other five instructions each earned
their keep — measure against real c2, publish a series, measure both profiles,
extend `ref/` rather than forking it, and decline if the constant is not derived
— and the last one is the reason this rung exists in the form it does.

### 11.4 `fnbyte-exact Δ = 0` needs a warning label, and this is the second lane to say so

`w-dataseam` §6.1 reported the `[180,231]` bracket as *"1,189 wrong bodies for
nothing"*, replicated on two disjoint halves with effect sizes agreeing to one
body. **Every one of those statements is true**, and §5.2 shows the predicate
behind them is 39.6 % wrong about c2. Split-half replication does not detect
this, because both halves have the same blind spot: the population where the
predicate errs is refused for other reasons in both.

> **The transferable rule: a zero-cost result in `fnbyte-exact` is evidence about
> the predicate's *reach*, never about its *correctness*.** The only instrument
> that can grade correctness is the oracle's own per-edge verdict, and it is
> cheap — GRID-W built it in 68 lines, this lane re-used it in 274.
