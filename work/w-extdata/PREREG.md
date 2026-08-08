# w-extdata — PRE-REGISTRATION

Lane `w-extdata`, worktree branch `wt-w-extdata` off master **`3168b4e9`**.
Written and committed **before the first probe compiled for this lane's rungs**.
The survey in §1 is the *verification of the handover*, taken at this base and
recorded here; §2 onward is registered ahead of any change to `crates/`.

Baseline read off master `3168b4e9` (`docs/STATUS.md` generated block):
**match 13** · mismatch 0 · codegen-gap 0 · vocab-gap 858 · capture-fail 7 ·
**FRONTIER 14** · tests **1,255 passed / 0 failed / 36 targets** · gate 5,274
fixture-verdicts.

---

## 1. THE SURVEY — the handover's diagnosis, re-verified at MY base

Board **#1705** (w-data §6) overturned w-cfg2 §4's claim that `undname`,
`osfinfo` and `vswprnc` sit on the data-**object** seam. The brief says to
verify at my base rather than take a several-hour-old survey. Done, three ways.

### 1.1 The objs — reproduced, not inherited

`work/w-frame/refobj.sh` at the workload's own `flags.txt`, then
`scripts/gt_dump.py`. Committed as `work/w-extdata/ref/<tu>/dis.txt`.

**Every `.text` byte, every relocation and every symbol record is identical to
w-cfg2's committed dump.** The only difference in the three files is
`.debug$S` (152 → 172 bytes) and the section pointers it shifts — an artifact of
this worktree's own output path length, not of the compilation. So:

```text
  undname.cpp   6 sections: .drectve .debug$S .XBLD$W .XBLD$W .text(140) .pdata(8)
  osfinfo.cpp   6 sections: .drectve .debug$S .XBLD$W .XBLD$W .text(152) .pdata(8)
  vswprnc.cpp   6 sections: .drectve .debug$S .XBLD$W .XBLD$W .text(156) .pdata(8)
```

**#1705 HOLDS at `3168b4e9`. No `.data`, no `.bss`, no defined object.** Every
data symbol is an undefined external in section 0, linkage `02` — `data_sym`/WR1's
population, not `resolve_data_def`'s.

### 1.2 The scan — where the port stops today

`c2rs gap --list work/w-extdata/three.txt` at this base, workload flags:

| TU | verdict | A | B | C | frontier | `.text` byte fraction | CFG class |
|---|---|---|---|---|---|---|---|
| `undname.cpp` | vocab-gap (il function decode failed) | ✔ | ✔ | ✔ | yes | 0/140 = **0.0 %** | `cflow-if-n`, labels 3 |
| `osfinfo.cpp` | vocab-gap (il function decode failed) | ✔ | ✔ | ✔ | yes | 0/152 = **0.0 %** | `cflow-if-n`, labels 3 |
| `vswprnc.cpp` | vocab-gap (il function decode failed) | ✔ | ✔ | ✔ | yes | 0/156 = **0.0 %** | `cflow-if-n`, labels 3 |

`c2rs census` at the workload flags, one function each, **0/1 in class**, and the
blocking byte is a *relational opcode the general expression parser has no
production for*:

| TU | blocker key | blocking byte |
|---|---|---|
| `undname.cpp` | `expr-cmp-ne` | `0x20` |
| `osfinfo.cpp` | `expr-cmp-ge` | `0x23` |
| `vswprnc.cpp` | `expr-cmp-eq` | `0x1f` |

All three decode end-to-end for the *control-flow* class (`1/1 bodies decoded`),
`eh-none`, `maxState 0` — **no EH record**, which is what separates them from
`Main.cpp`.

### 1.3 The frames — three of `frame.rs`'s markers are ALREADY BUILT

w-cfg2 §2 claimed this and the brief says to verify. Re-derived from
`FrameLayout`'s own code at this base (`crates/c2-core/src/codegen/frame.rs`),
`locals = 0`, `out_slots = 0`, `saved_fprs = 0`:

| TU | `saved_gprs` | `size()` | prologue emitted by `FrameLayout::prologue` | reference `.text` |
|---|---:|---:|---|---|
| `osfinfo` | 0 | 96 | `mflr r12 · stw r12,-8(r1) · stwu r1,-96(r1)` | **identical**, 0x00–0x08 |
| `vswprnc` | 1 | 96 | `… · std r31,-16(r1) · stwu r1,-96(r1)` | **identical**, 0x00–0x0c |
| `undname` | 2 | 112 | `… · std r30,-24(r1) · std r31,-16(r1) · stwu r1,-112(r1)` | **identical**, 0x00–0x10 |

Epilogues likewise (`addi r1,r1,F · lwz r12,-8(r1) · mtlr r12 · ld r30/r31 · blr`),
word for word including the restore order. `needs_gpr_helper()` is `false` at 2
and only fires at 3, so `xlrcimpl`'s `__savegprlr_26` refusal does **not** reach
any of these three. **VERIFIED: the frame costs zero on all three.**

### 1.4 The symbol table — one rule, and it is NOT the one the writer ships

Read off all three reference symbol tables. The undefined externals are emitted
in **reverse order of first reference in `.text`**, as ONE list, without regard
to whether the symbol is a callee or a data name:

| TU | first-reference order (`.text` offset) | symbol table, indices 15… |
|---|---|---|
| `undname` | `gHeapManager`(0x24) `getMemory`(0x34) `pairNode_vtable`(0x40) | `pairNode_vtable` `getMemory` `gHeapManager` |
| `osfinfo` | `_nhandle`(0x14) `__pioinfo`(0x28) `_errno`(0x68) `__doserrno`(0x74) | `__doserrno` `_errno` `__pioinfo` `_nhandle` |
| `vswprnc` | `_woutput_s_l`(0x38) `_vswprintf_helper`(0x4c) `_errno`(0x68) `_invalid_parameter_noinfo`(0x80) | `_invalid_parameter_noinfo` `_errno` `_vswprintf_helper` `_woutput_s_l` |

`coff::writer` emits **callees (reverse first-reference) and THEN data symbols**,
in two separate loops with two separate index lists (`introduced` /
`introduced_data`, `writer.rs:462`–`468`). On the population shipped today the
two rules **coincide**, and they coincide for a reason worth writing down:

> **WR1's *"the `lis` is the body's FIRST word"* rule and the writer's
> *"callees then data symbols"* symbol order are the SAME FACT.** If the data
> reference is always the body's first word then the data symbol is always the
> **first**-referenced name, so reverse-first-reference always puts it **last** —
> which is exactly where the second loop puts it. Relaxing either without the
> other is a wrong symbol table.

`undname` is the witness that separates them: `data · callee · data` interleaved,
which no ordering of two separate loops can produce. `osfinfo` separates a weaker
version (its two data names need **reverse** order among themselves; the writer
pushes them in `data_refs` order). `vswprnc` does **not** separate them — its one
address-taken name is first-referenced and therefore last either way.

### 1.5 The three, priced by what the port has to GROW

Independent refusals, counted at this base, stopping at nothing (unlike #269,
which stopped once its clause fired):

| # | `vswprnc` (156 B, 39 words) | `undname` (140 B, 35 words) | `osfinfo` (152 B, 38 words) |
|---|---|---|---|
| 1 | a `cflow-if-n` recognizer for this body (`53`-opening, relational) | same | same |
| 2 | a 30-word emitter | a 24-word emitter | a 30-word emitter |
| 3 | `data_refs_of` requires the `lis` be the body's **first** word; here it is word **14** | …word **9**, and there are **two** | …word **5**, and there are **two** |
| 4 | the REFHI/REFLO target is a **FUNCTION** (`Type 0x0020`); `emit_external_symbol(…, 0x0000)` is hardcoded | — | — |
| 5 | the relocation-target lookup picks `callee_syms` for REL24 and `data_syms` otherwise (`writer.rs:419`) — a REFHI against a callee-list name is a `panic!` | — | — |
| 6 | — | `data_sym: Option<String>` → a **list** | same |
| 7 | — | the symbol table's **union merge** (§1.4) | data symbols in **reverse** order among themselves |
| 8 | — | one REFLO is `addi r11,r11,0` — into the scratch itself, **not** an `ARG_REG`; `data_refs_of`'s search only matches `addi <ARG_REG>,r11,0` | one REFLO is the **displacement of a `lwz`** (`lwz r11,0(r11)`), a site form nothing models |
| 9 | — | **two** `addis r11,0,0` in one body, so "the unique `addis`" cannot identify either | — |
| 10 | — | — | `encode_cmplw` (register-form logical compare) does not exist |
| 11 | — | — | `rlwinm.` (record form — `clrlwi. r10,r10,31`) does not exist |
| **total** | **5** | **7** | **7** |

Encoders **already present** and therefore *not* counted: `sth` `stb` `stw` `lwz`
`lbz` `srawi` `mulli` `lwzx` `add` `rlwinm` (covers `slwi`/`clrlwi`) `cmpwi`
`cmplwi` `bc` `b_intra` `mr` `addi` `addis` `std` `ld` `stwu`. **The encoder gap
the brief flagged as unpriced is 2 encoders, both in `osfinfo`, and 0 in the
other two.**

`.pdata` + the `$M`/`$M`/`$T` triple is already built (`coff::plan_labels` mints
them for a framed function; `negate_test.cpp` matches with them), so it is not
counted either.

**Ladder, cheapest first: `vswprnc` (5) → `undname` (7) → `osfinfo` (7).** This
agrees with the brief's guess. The split between `undname` and `osfinfo` at 7
is a tie I do not claim to break; `undname` goes second because its source is
**fully self-contained C++ with no `#include` at all**, so its fixture is a
verbatim copy, and because it is the only witness for §1.4's union merge.

### 1.6 `vswprnc`'s IL body — DECODED, and it is a linear token stream

429 bytes from the `4C 4F 11` anchor to the `4D`, ~40 statements, **no nesting a
pattern matcher cannot walk and no value merge at a join**. Decode committed as
`work/w-extdata/VSWPRNC_BODY.md`. This is registered as a *measurement made
before the rung*, exactly as w-cfg2 §5 registered `Primes`'.

---

## 2. THE LADDER

### R1 — `vswprnc.cpp` → match

* **R1.a (reader)** `c2_il::…::shapes::vswprintf_guard_tail` — a whole-body
  recognizer hooked into the `0x53` arm of the statement dispatch (beside
  `try_parse_guarded_seq` / `try_parse_early_return_seq`), on the same
  non-committal terms every shape recognizer uses: its own cursor, `Err` on the
  first byte outside its grammar, no census key moved by a decline.
* **R1.b (mode gate)** the `/O1`-only clause goes in the **PARSER**, asked before
  any body byte is read. Board **#1638** has fired twice; `census_gate.rs` is the
  cross-check and it must be green **in the same commit**.
* **R1.c (writer)** an address-taken FUNCTION name: `Type 0x0020`, ordered with
  the callees, and the relocation-target lookup searching both tables.
* **R1.d (WR1)** `data_refs_of`'s "first word" clause relaxed to "the unique
  `addis rT,0,0`", **derived from the bytes** and refusing on ambiguity — never
  declared by the class (that is `data_refs_of`'s own stated discipline).
* **R1.e (emitter)** `c2_core::codegen::vswprintf_guard_tail` — 30 words.
* **R1.f (fixture)** `fixtures/cpp/wextdata_vswprintf_guard_tail.cpp` plus a
  negative-cell file, graded by real `c2.dll`.

### R2 — `undname.cpp` → match (stretch)

Adds §1.5 rows 6–9: the `data_sym` list, the symbol-table union merge, the
non-`ARG_REG` REFLO, and pairing two `addis` by position.

### R3 — `osfinfo.cpp` (registered, NOT expected to be reached)

Adds the `lwz`-displacement REFLO site form and two encoders.

---

## 3. PREDICTIONS, frozen

### 3.1 The conversion call, as a probability (board #770 calibration)

Board **#770** measures forward estimates on this project as **~12-to-1
optimistic overall**, and the brief records that the last three conversion calls
went *pessimistic-wrong, pessimistic-right-at-0.6, optimistic-wrong* — so a
point estimate is not the honest form. Registered as probabilities, before the
first line of `crates/` changes:

| outcome | P |
|---|---:|
| **R1 converts** (`vswprnc`, match 13 → 14) | **0.55** |
| R1 **and** R2 convert (match 15) | **0.20** |
| all three (match 16, FRONTIER 11) | **0.04** |
| **nothing converts, priced decline delivered** | **0.45** |

The 0.55 is deliberately *below* the "this is only a transcription" feeling. The
two reasons, both registered rather than reconstructed later: (a) §1.5's price of
5 is a count of things I can *name*, and every conversion lane on this board has
found at least one refusal its survey did not name (w-heap found two, w-data
found an eighth); (b) the `.text` byte fraction is **0.0 %** — nothing about
this body has ever been emitted, so there is no partial credit to build on.

### 3.2 Metric predictions

| metric | base `3168b4e9` | predicted at tip if R1 only | predicted if R1+R2 |
|---|---:|---:|---:|
| TU match | 13 | **14** | 15 |
| mismatch | 0 | **0** | 0 |
| codegen-gap | 0 | **0** | 0 |
| vocab-gap | 858 | **857** | 856 |
| capture-fail | 7 | **7** | 7 |
| FRONTIER | 14 | **13** | 12 |
| frontier-if-A | 136 | **135** | 134 |
| factor A / B / C | 28 / 338 / 169 | **unchanged** | unchanged |
| `A∧B∧C` | 27 | **27** | 27 |
| factor D | 13 | **14** | 15 |
| function census | 711,489 | **711,490** | 711,491 |
| emitted census | 39,188 | **39,189** | 39,190 |
| `fnbyte-exact` | 36,216 | **36,217** | 36,218 |
| workspace tests | 1,255 | **≥ 1,255** — the exact predicted number is written into §9 *before* the final run, and compared to it (board **#1710a**: a test vanished in a `git checkout` round trip with nothing going red) | |
| gate fixture-verdicts | 5,274 | **> 5,274** (two new fixtures) | |

Anything else moving is a **finding to report, not a rounding error** —
specifically `factor-c`, `b-and-c`, `writer-sections` (10), `fnbyte-differs`
(2,111), `fnbyte-reloc-differs` (861), `progress-mass` (0.20830) and every `in-*`
/ `gl-*` reader invariant must be **byte-identical** across this lane. The
comparison is a **key→value map**, not `diff` (w-data §1: a line diff over-reads
because the `cflow-offclass` block is emitted in count order).

### 3.3 The peer-key scan

246 `gap-metric` keys at each end. Predicted: **0 vanished, 0 appeared**, and the
only keys that change are arithmetic on the converted function(s).

---

## 4. DECLINE CLAUSES — thresholds AND sizes, frozen

w-cfg2's **D1** lesson: a decline clause must name the **SIZE** of what it
declines, or it cannot be told apart from "the reader has no production at all".
Every clause below carries one.

* **D1 — the block plan.** If `vswprnc`'s emitted `.text` cannot be produced by a
  **fixed word list with ≤ 6 immediate fields** — i.e. if any *word* of the 30
  has to be chosen by a register allocator or a scheduler — **decline R1**.
  **Size:** the number of words that need a chooser, and which. Registered
  expectation: **0 words and 4 immediate fields** (`0x16`, `0x22`, `-2`, `-1`).
* **D2 — the reader.** If the recognizer for R1 needs a general basic-block IR or
  a value merge at a join, **decline**. **Size:** §1.6's decode says the stream is
  linear with 5 labels and 6 transfers; if the built recognizer needs more than a
  single forward cursor over those, report the count of back-references it needed.
* **D3 — the union merge (R2).** If §1.4's reverse-first-reference union rule is
  **not** confirmed by GRID A's separating cells, **decline R2 and ship the
  measurement**. **Size:** the number of GRID A cells whose symbol table the rule
  mispredicts, out of the cells compiled.
* **D4 — WR1's position rule.** If relaxing "the `lis` is the first word" cannot
  be made to *derive* the site from the bytes without ambiguity on any shipped
  fixture, **decline the relaxation and therefore the rung**. **Size:** the number
  of currently-passing fixtures whose derived sites move. Registered expectation:
  **0**.
* **D5 — previously-emitted objs.** If **any** obj the port emitted at the base
  differs at the tip, the change is wrong and is reverted, not fenced.
  **Size:** the count of differing objs, by name. Registered expectation: **0**.
* **D6 — `ptr_walk_loop`'s unpaid #1638 defect.** Registered as **NOT TAKEN**.
  It sits behind a MATCHED TU (`Sort.cpp`); the bar is a guarding fixture in the
  same commit and this lane does not have the budget for it beside two
  conversions. **Size of what is declined: one mode clause in one emitter.** If
  R1 and R2 both land early, it is the first thing to reconsider — and only with
  the fixture.
* **D7 — the second data reference of `undname` and `osfinfo`.** If `data_sym`
  cannot be widened to a list without changing behaviour on the ≤ 1 population,
  decline R2/R3. **Size:** the number of `data_sym` readers that assume
  at-most-one. Registered expectation from the grep: **9 sites** across
  `lib.rs`, `splice.rs`, `elide.rs` and the leaf emitters.

---

## 5. GRIDS

Every rival's per-cell prediction is frozen **before the first `cl.exe` on that
grid's cells**, in `work/w-extdata/GRID_*.md`, and separation is asserted.
Cells are read **two ways** where a second reading exists (w-data's GRID C caught
a writer interleaving defect only because the cell was read by ORDER as well as
by content).

* **GRID A — the symbol-table order rule.** Rivals: (A1) one union list, reverse
  first reference; (A2) callees then data, each reverse; (A3) declaration order;
  (A4) `.gl` order. Read **twice**: by symbol INDEX and by the relocation each
  index is the target of.
* **GRID B — the R1 fence.** Positive cells that must all emit the same `.text`,
  and negative cells that must each decline on **its own distinct clause name**
  (w-cfgclass's discipline; w-data's #1704 is what happens when two negatives
  share a clause).

---

## 6. STANDING RULES ACKNOWLEDGED

* Real `c2.dll` under wibo + byte-exact obj compare (TimeDateStamp 4..8 zeroed) is
  the sole judge. A refusal becoming a wrong emit is strictly worse than a gap
  (#232).
* Label stride is **6**, not 5 (w-cfgclass §5.1).
* `_fltused` emits from a float FORMAL with no instruction (w-cfgclass) — none of
  these three has a float formal; if one appears, this is the trap.
* The census files object-refusals under `callee-unresolved-tail-call` (#1704) —
  that key's name is not to be trusted on this survey. §1.2 reports the *blocking
  byte*, which is.
* std only, zero external crates. No `Co-Authored-By` or agent trailer.
* Board range **#1720–#1739**; unused numbers left explicitly unminted.
* `scripts/gate.sh --require-graded` must PASS and
  `cargo test --workspace --release` must be 0 failed before reporting.

---

## 7. SCORING

§9 of the rung doc scores every clause in §3 and §4, misses in the registered
direction, and states which registrations were **mis-drawn** rather than merely
wrong (w-cfg2's D1 is the model: it fired as written *and* could not distinguish
the two readings it needed to).
