# w-sect pre-registration — the `.data`/`.bss` writer (board #174)

Committed **before** any line of the writer is written. Scored verbatim in the
rung doc. Nothing below is edited after the fact; corrections are appended with
a date.

Tip at registration: `caff20d` (master), branch `wt-w-sect`.
Baseline **re-measured on this tip**, not transcribed:

| instrument | baseline |
|---|---|
| `cargo test --workspace --release` | **724 passed, 0 FAILED, 25 targets** |
| `c2rs gap` TU match / mismatch | **8 / 0** |
| factor **C** | **114** of 871 |
| greedy ladder, next step | `+.data → C = 169 (+55)` |
| factor A / B / D / E | 28 / 338 / 8 / 2 |
| `A∧B∧C` / FRONTIER | 25 / 17 |
| wibo | `1.0.1-23-g4a9dd6f` |
| `../dc3-decomp` HEAD, before | `940d07dc` |

---

## 0. What this lane found before registering, and why it is in the prereg

Two measurements were taken **before** this document and they change what is
worth registering, so they are stated here rather than presented later as
results.

**F1 — a family of live wrong emits.** A TU with **no functions but with defined
namespace-scope data** is emitted as the bare four-section shell by
`PortC2::build`'s `emit_empty_obj` arm. Eight of eleven probe shapes read
`Port=Mismatch @ offset 2`. `is_empty_module` is a property of `.ex` alone and
is true however much data `.gl` declares.

**F2 — the workload has ZERO data-only TUs.** Measured on
`work/w-bss/census/sections.jsonl`: 8 of 871 objs carry no `.text`, and all
eight are either shell-only (6) or the two `??__E` license TUs. **No obj's
section set is a subset of `{shell, .data, .bss}` with a data section present.**

F2 is the reason the TU-match registration below is a point mass at zero and not
a guess. F1 is the reason this lane's first commit is a refusal.

---

## 1. The independent refusals between the ceiling and a conversion

The brief's rule: *when a row's blocker is a class whose emitter already exists,
the ceiling IS the estimate — count the independent refusals and apply no
discount; "independent" means **what varies between these refusals?**, and if the
answer is "nothing, it is the same variable read at different thresholds", it is
one refusal.*

Applied to **factor C's 169**, i.e. *"what stands between a TU being inside C and
that TU being byte-exact?"*:

| # | refusal | what varies between it and its neighbours? | independent? |
|---|---|---|---|
| R1 | the TU's function bodies are outside the port's codegen class | the **IL body shape** (`vocab-gap` is 863 of 878) | **yes** |
| R2 | factor **A** — `.ex` segments ≠ obj `.text` COMDATs | the **emit set**, a count equality over symbols, not over bodies | **yes** |
| R3 | factor **B** — some emitted symbol does not bind | the **`.gl` name binding**, which fails on records R1 and R2 both read fine | **yes** |

Three independent refusals, **none of which this lane touches.** Every one of
the 169 TUs inside C after this rung carries a `.text`; the writer this lane
builds emits no `.text` at all. So the conversion count contributed by +55 to C
is **exactly 0**, and that is board #213's structure — *"+82 TUs that become
REACHABLE BY CODEGEN, every one of them still gated on codegen that does not
exist"* — restated one rung down. This is **not a discount**; it is the count.

Applied to **the class this lane's writer actually emits** (a TU with no
functions and one or two ordinary namespace-scope objects per non-COMDAT
section), the refusals between it and a *workload* conversion are:

| # | refusal | what varies? | independent? |
|---|---|---|---|
| R4 | the TU must define no functions | presence of `.text` | **yes** |
| R5 | ≤ 2 objects per non-COMDAT section | the object **count** | **yes** |
| R6 | no COMDAT data object (`selectany`, `??_R0` RTTI) | the object's **COMDAT-ness** | **yes** |
| R7 | no `.rdata` (string literal, `const` pool) | the **section set** | **yes** |
| R8 | no `.tls$` | the **section kind** | **yes** |
| R9 | alignment ≤ 8 | the **alignment** | **yes** |

R5 and R9 look like "the same variable at two thresholds" and are not: R5 is a
count over objects, R9 is a per-object attribute, and a one-object section can
fail R9 while a three-object section passes it. R6 and R7 look like one and are
not: `__declspec(selectany) int sa = 3;` fails R6 with no `.rdata` anywhere,
and `extern const int ce = 9;` fails R7 with no COMDAT anywhere. Six
independent refusals — and **R4 alone is satisfied by 0 workload TUs** (F2), so
the workload conversion count is 0 before the other five are consulted.

---

## 2. Registered quantities — point and interval, stated separately

**Units are named on every row, because the brief records a lane that scored 3
of 4 and missed on a unit mismatch.** Factor C is measured in **TUs of
reachability**; TU match in **TUs of conversion**; the wrong-emit rows in
**probe TU shapes**. They are three different populations and are never summed.

### 2.1 Factor C — unit: TUs of 871, REACHABILITY not conversion

| | value |
|---|---|
| **point estimate** | **C = 169** (Δ **+55**) |
| **interval** | **[160, 172]** |
| **decline clause keys on** | the **point** |
| **bias direction, in writing** | **upward.** I expect to overstate, because `PORT_WRITER_SECTIONS` is a *vocabulary* and a writer that emits `.data` only for a functionless TU still puts the name in it. The +55 counts 169 TUs whose `.data` this writer would not survive contact with. I am registering it anyway because that is exactly how `.bss`, `.CRT$XCU` and `.text$yc` were counted when `emit_dyninit_obj` gained a caller (§10.21's C +30), and changing the accounting convention mid-ladder would make the ladder unreadable. **The overstatement is in the metric's definition, not in this measurement**, and the rung doc will say so in the same paragraph as the number. |

The interval is wider than the point's arithmetic warrants (the ladder is
recomputed by every `gap` run, so 169 should be exact) because peer lanes may
advance master's C between registration and merge.

**Decline clause.** If realized C < 160 I have not built what the ladder priced
and must say what the ladder was pricing instead.

### 2.2 TU match — unit: TUs of 878, CONVERSION

| | value |
|---|---|
| **point estimate** | **8** (Δ **0**) |
| **interval** | **[8, 8]** |
| **decline clause keys on** | the **point** |
| **bias direction, in writing** | **none available.** This is not an estimate. F2 measured the population directly: the workload contains **no** TU whose obj sections are the shell plus `.data`/`.bss`. A point mass at zero is the measurement, and any movement — up **or** down — is a finding I owe an explanation for. Movement **down** is a regression and fails the lane. |

Registered separately from C **because +55 to C is reachability and not
conversions**, which is the conflation board #213's title had to be corrected
for on 2026-08-04.

### 2.3 Wrong emits — unit: probe TU shapes, of the 11 in the grid

| | value |
|---|---|
| **point estimate** | **0 Mismatch**, **5 Match**, **6 NotImplemented** |
| **interval on Match** | **[3, 7]** |
| **decline clause keys on** | the **Mismatch count**, which must be **0**. Match and NotImplemented trade against each other freely; a wrong emit does not. |
| **bias direction, in writing** | **upward on Match.** I have read the spec and not yet the bytes, and every previous lane that predicted from `OBJ_DATA_BSS_SHAPE.md` found one clause the doc states correctly and one it states for a class it was not measured on. |

Named split of the 11:

| probe | registered verdict |
|---|---|
| `int g = 5;` | Match |
| `char b1;` | Match |
| `char b1; char b2;` | Match |
| `int d1=1; int d2=2;` | Match |
| `char b1; char d1=1;` | Match |
| `extern const int ce = 9;` | NotImplemented (R7 — `.rdata`) |
| `const char* s = "hi";` | NotImplemented (R7 + a relocation) |
| `__declspec(thread) int t1;` | NotImplemented (R8 — `.tls$`) |
| `__declspec(selectany) int sa = 3;` | NotImplemented (R6 — COMDAT) |
| `extern int e;` | Match (already; no change) |
| `typedef int T;` | Match (already; no change) |

### 2.4 Workspace tests — unit: test functions

| | value |
|---|---|
| **point estimate** | **+18** unit tests, 724 → **742** |
| **interval** | **[+8, +35]** |
| **decline clause keys on** | the **FAILED count**, which must be **0**. The passed count is reported for reconciliation only — a failing target aborts the run, so a *lower* passed count reads as green. |

---

## 3. Falsifiable mechanism predictions

Committed now, scored in the rung doc. These are the rows that earn their keep
if they are **wrong**.

| # | prediction | named alternative |
|---|---|---|
| **P1** | `emit_empty_obj`'s arm is the **whole** of F1 — no other path in `PortC2::build` emits a data-bearing TU wrongly | the framed/`.text` paths also drop a `.bss` when the TU has both code and data |
| **P2** | the existing `data_object_at` reader **already fails closed** on `__declspec(selectany)`, because the attribute byte is `0xE0` where a plain definition is `0x80`, and the reader admits only `0x00`/`0x80`. So board #172's *"COMDAT-ness is not in tag 9"* is about the **section** record and does not apply to the **data** record | the reader admits `selectany` and I must add the gate |
| **P3** | the existing reader **does not** fail closed on `__declspec(thread)`: the record is byte-identical to an ordinary uninitialized object in every field the reader reads, and the discriminator is the byte **after** the attribute (`0x10` vs `0x00`), which it never reaches. Without a new gate `x_tls` is a wrong emit | the thread-local differs in a field already read (type tag or linkage) |
| **P4** | a `.data` object's raw bytes are written **big-endian** — `int i1 = 0x11223344;` gives `11 22 33 44` — while the `.in` initializer varint spells the same value little-endian (`80 44 33 22 11`) | both are the same endianness and no swap is needed |
| **P5** | section order for a TU with both kinds is `.drectve .debug$S .XBLD$W(C2) **.bss** .XBLD$W(C1) **.data**` — Rule S1, uninitialized **between** the watermarks and initialized **after** the second | `.data` before `.bss`, the ordinary link-order intuition, which is prereg P3's refuted clause |
| **P6** | eager-`.bss` symbol-table order is Rule Y1 — every EXTERNAL first in **reverse `.gl`** order, then every STATIC in **declaration** order — and it is *not* ascending address for the static case | symbol order is ascending address throughout |
| **P7** | the `.data` aux `CheckSum` is a **real CRC** even when the section is **not** a COMDAT (Rule D1), refuting `OBJ_DYNINIT_SHAPE.md` §2.3's *"0 for every non-COMDAT section"* | it is 0, as §2.3 says |
| **P8** | at **two** objects the walk order does not matter for `.data` (declaration order = `.gl` order restricted, or the two coincide often enough) but **does** for `.bss`, where `.gl` file order is the reverse of declaration order for `char b1; char b2;` | two-object sections coincide in both sections and the axis is invisible below three objects |

**Registered bias on the whole grid.** My hazard is the mirror of w-bss's: that
lane expected `.data`/`.bss` to be *boringly regular* and under-varied. I expect
them to be **irregular**, which makes me likely to **over-refuse** — to gate on
axes that do not move the bytes and ship a class narrower than the measurement
supports, then report the narrowness as rigour. The mitigation is that P8 and the
`≤ 2 objects` bound are stated as *thresholds to test*, not as gates to assume:
if a three-object section is exact on the grid, the rung says so and the number
moves.

**And the structural-axis rule from the brief is registered as a constraint on
the fixtures, not a hope**: *values are the axis that feels thorough and
discriminates least*. The fixture grid varies **object count per section**
(1/2/3), **alignment padding present vs absent**, **`.data` vs `.bss` vs both**,
**declaration order vs `.gl` order**, and **COMDAT vs not** — and holds the
initializer *values* nearly constant on purpose.

---

## 4. What is declined in advance, with the reason

| declined | reason |
|---|---|
| **`.tls$`** | Rule T1 (§5.8) is fitted on ten probe cells and has **never been seen on a real TU**; §8.9 records multiplicity and COMDAT behaviour as unmeasured, and the within-block sort key is **not separated** between size and alignment by any of the ten cells. `.tls$` is also **not one of the workload's 13 section names**, so it is worth **+0 to factor C**. Emitting it would be an unmeasured code path with no payoff. |
| **COMDAT `.data`/`.bss`** (`selectany`, `??_R0` RTTI) | 8,382 of 9,139 workload `.data` are COMDAT, so this is the *common* case and not an edge — but every one of them is in a TU that also has `.text`, and the RTTI half is board #160's rung three (`.rdata$r`, +421), not this row. Emitting a COMDAT `.data` here would put the `??_R0` payload and the `??_7type_info@@6B@` relocation in front of the differential with nothing asking for it. |
| **> 2 objects per non-COMDAT section** | 38 of 62 (§5.7). The residual is walk order (**#184**) and the brief is explicit that it is not a size problem. Refuse, do not guess. |
| **floating-point initializers** | §4.2.1's CRC exclusion is settled *as a specification* and the granularity finding is labelled *not pre-registered*. A `double` in a `.data` needs the FP byte-range omission and I would be encoding a rule from three exploratory cells. |
| **`.data` relocations** | §8.6: only pointer-valued initializers were exercised; member-pointer, vftable and cross-section initializers were not. A pointer initializer is refused. |

---

## 5. How this will be graded

* The real `c2.dll` under wibo, byte-exact, `TimeDateStamp` zeroed. No expected
  obj is constructed anywhere in this lane.
* Fixtures wired into a lane that **actually consumes them** — verified by
  reading the consumer, not assumed. `differential.rs` names a **fixed list of
  three fixtures** and will not see a new one; `mode_lane.sh` globs the
  directory and `census_gate.rs` reads it.
* `sweep.py` with **`C2RS_SWEEP_KEEP` set to this lane's seam**, honouring F-c:
  a rung that adds a code path with no coverage under the GRADED profile is
  adding a first witness and must say so.
* Graded at the **workload's own flags** (`/O1 /Oi /EHsc /GR …`) and not only at
  `/Ox`, because `expr_sweep.sh` runs only `/Ox` and w-order found a live wrong
  emit at the workload's profile that the sweep structurally cannot see.
