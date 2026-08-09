# w-vec — PREREGISTRATION

**FROZEN BEFORE the first `crates/` change, the first probe cell and the first
fixture line.** Committed as its own commit; every number below is a claim made
in advance and scored in the rung's §ESTIMATE vs OUTCOME.

    Lane:      w-vec
    Branch:    worktree-agent-a19c39e5bd6377202, off master `111b6357`
    Board:     #2500–#2529
    Commission: convert `src/system/math/vec.cpp`, TU match 20 → 21. It is the
               last TU on the board whose emitted functions are already
               byte-exact (T1 = 1 at `w-fence2`'s tip).

---

## 0. WORKLOAD STAMP — the base, re-derived by this lane's own scan

Nothing below is quoted from another lane's rung. `work/w-vec/base.out` /
`base.jsonl` / `base.tsv`, this lane's own `gap` run:

| | |
|---|---|
| c2-rs | **`111b63576fb20e9f06dedc2e75922231e72d7d4d`** (clean) |
| binary | **`fac478feafeab975df24817e7af05b5c`** — the merge-base binary, copied to `work/w-vec/c2rs-base` **before** any edit (board **#2409**: `git checkout master -- crates/` is not a counterfactual) |
| workload | dc3-decomp **`d7a3c1aa9d5d57a1176790c0e15a723edd2e03a0`**, source clean (two untracked non-source paths, `-.cache` and `work/`) |
| wibo | `wibo 1.2.0-c2rs.1 (Linux x86_64)` |
| toolchain | `compilers/X360/16.00.11886.00/{cl.exe,c2.dll,c1xx.dll}` |
| command | `c2rs gap --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16 --jsonl … --factors-tsv …` |

**Base figures, all from that one scan (257 `gap-metric` keys):**

```text
tu-total 878   graded 871   match 20   mismatch 0   codegen-gap 0
vocab-gap 851  port-error 0  capture-fail 7
factor-a 28  factor-a-lo 27  factor-b 338  factor-c 169  factor-d 20  factor-e 2
b-and-c 151  a-and-b-and-c 27  a-and-b-and-c-and-d 18  a-and-b-and-c-and-d-or-e 20
frontier 7   frontier-if-a 129   progress-mass 0.21406
fnbyte-exact 35793   fnbyte-differs 1898   fnbyte-refused 114649
fnbyte-denominator 162092   fnbyte-decline-inlined-callee 1003
bytefrac-control-full 13
```

**Two published base figures re-checked, both hold at this tree:** `w-fence2`'s
`match 20`, `fnbyte-exact 35,793`, `fnbyte-differs 1,898`,
`fnbyte-refused 114,649`, `frontier 7`. `factor-d` reads **20** here, where
`CEILING.md` §1.2's table still says 11 and its own banner says to quote a scan.

---

## 1. THE BINDING PREDICATE — checked FIRST, before anything is priced

`CEILING.md` §11.4 item 8, `w-mmioclose` #2406. The TU's own `gap --jsonl` row,
one line, read before any mechanism was sized:

```text
"src": "src/system/math/vec.cpp",
"class": "vocab-gap",
"detail": ".ex 170578 B, 150 .gl names — c2_il::functions() and dyninit_tu() both None",
"fn_names": 150,   "fn_total": 811,   "fn_in_class": 237,
"emit": { "afail-row-emitted": 2, "afail-row-not-emitted": 358, "afail-row-unnamed": 451 }
```

> **`IlBundle::functions()` IS `None` ON `vec.cpp`, AND SO IS `dyninit_tu()`.
> THE GATE BINDS NOTHING.** `Bindings::per_record` requires
> `gl_defined_names(gl).len() == segs.len()` with every record's framed
> body-start offset equal to its `.ex` split point, in order and 1:1. `vec.cpp`
> has **811 `.ex` bodies**. It is not the `looks_mangled` shape `mmio.cpp` had
> — `vec.cpp`'s two subjects *are* C++-mangled (`??0Vector3@@QAA@MMM@Z`) — but
> the outcome is the same one: **no obj is produced at all**, so every
> per-function byte figure on this TU is keyed on `FnCensus::emit_name` and
> says nothing about the gate.

**This is registered as a finding of the PREREG, not a prediction**, because it
was read from the base scan before the PREREG was written. What follows is
predicted from it.

## 2. THE REFERENCE OBJ — captured by this lane, `scripts/gt_dump.py`

`work/w-vec/ref/vec.obj`, workload flags. **9 sections, 34 symbols, 1,819
bytes, ZERO relocations anywhere.**

```text
 1 .drectve  132  chars 0x00100a00
 2 .debug$S  176  chars 0x42100040
 3 .XBLD$W    16  chars 0xc0401040  sel=2   __C2_11886
 4 .XBLD$W    16  chars 0xc2301040  sel=2   __C1_11886
 5 .rdata      4  chars 0x40301040  sel=2   COMDAT  ?npos@?$basic_string@… = ff ff ff ff
 6 .text      16  chars 0x60401020  sel=2   COMDAT  ??0Vector3@@QAA@MMM@Z
 7 .text      20  chars 0x60401020  sel=2   COMDAT  ??0Vector4@@QAA@MMMM@Z
 8 .data     112  chars 0xc0300040  sel=0   NOT a COMDAT — 7 defined externals
 9 .bss       32  chars 0xc0300080  sel=0   NOT a COMDAT — 2 defined externals
```

`_fltused` is symbol **[17]**, immediately after `??0Vector3`'s own [16] and
before `??0Vector4`'s section symbol [18] — `w-blockir`'s NC-1 item 1 shape
exactly.

Every one of the seven distinct names is inside `PORT_WRITER_SECTIONS`
(`.drectve .debug$S .XBLD$W .text .pdata .rdata .text$yc .bss .CRT$XCU .data`),
which is why the TU is already inside factor **C** and fails only **A**.
**Factor C is NOT the blocker; the writer's section VOCABULARY is complete for
this obj and its COMPOSITION is not.**

---

## 3. THE CONVERSION CALL, IN PROBABILITY FORM

| # | claim | p |
|---|---|--:|
| **C1** | `vec.cpp` converts; TU match **20 → 21** | **0.08** |
| **C1a** | the **decline** branch: the lane ships a priced decline with every mechanism named and sized | **0.92** |
| **C2** | `fnbyte-exact` delta is **exactly 0** | **0.90** |
| **C2b** | `fnbyte-exact` delta in `[−2, +2]` | 0.96 |
| **C2c** | `fnbyte-exact` does not FALL | 0.95 |
| **C3** | the gate's first blocker on `vec.cpp` is `IlBundle::functions() == None`, i.e. `gl_defined_names` does not return 811 names at the 811 split points — **NOT** `_fltused` and **NOT** a section name | **0.93** |
| **C4** | `_fltused` is ALREADY emitted by the port for a float function in a `/Gy` COMDAT obj (`w-blockir` #2301's arm is live), so the `_fltused` half of the published price is **already paid** | **0.75** |
| **C5** | no production emitter composes `.text` COMDATs with a **non-COMDAT** `.data` **or** a `.bss`; `emit_comdat_obj`'s `.data` is per-function and COMDAT, `emit_data_obj`'s class is "defines no functions" | **0.90** |
| **C6** | `emit_comdat_obj` refuses a defined data object on a **float** function by an explicit clause, so even the COMDAT-`.data` path is closed on this TU | **0.80** |
| **C7** | mismatch stays **0** at all three levels (878 TUs, 331+ fixtures × `/O1` and `/Ox`, 18 gate lanes, the expr sweep, the mode cross) | **0.97** |
| **C8** | the T1 ALL-EXACT-NO-MATCH population at my tip is **1** (unchanged — it goes to 0 only if C1 fires) | **0.90** |
| **C9** | `#[test]` DELTA **+4**, `±3` is the whole claim; targets 38 → 38 | **0.60** |
| **C10** | ≥ 1 unnamed refusal fires at a **pre-armed** place (§5) | 0.55 |
| **C11** | no `gap-metric` key vanishes; the key count stays **257** | 0.85 |
| **C12** | `fn_gate_refusals` is `{}` on `vec.cpp` at both ends | 0.85 |
| **C13** | the number of independent unbuilt mechanisms between the port and `vec.obj` is **≥ 3** | **0.85** |
| **C14** | `hatch-red` REFUSES on the pre-existing `HATCH-DRIFT` in `body/shapes/calls.rs`, reproduced at master with `crates/` reverted (`w-fence2` §7.2, board #1406) | 0.85 |

**Registered direction: PESSIMISTIC on C1.** Board #770's tally says this
board's lanes err optimistic; this one is registered the other way and that is
declared here rather than discovered in the scoring.

**C9 is registered at +4 because the last five lanes over-estimated the test
delta in the same direction and `w-fence2` §9 says so explicitly** ("the next
lane should register +4 and treat ±3 as the whole claim"). Registering +4.

---

## 4. DECLINE CLAUSES, EACH WITH A SIZE — registered in advance

If C1a fires, these are the mechanisms it will have to name. Sizes are
registered now so the rung cannot fit them to what was found.

| # | mechanism | registered size |
|---|---|---|
| **D1** | **the gate binding** — `Bindings::per_record` at 811 segments | ≥ 1 lane. Registered: `gl_defined_names` returns **< 811** framed defined records on this `.gl` |
| **D2** | **the EMIT SET (factor A)** — selecting 2 of 811 bodies | the shipped root rule is a *model*, validated out-of-sample at TU reach 31/31 (`w-root`), and lives in **no `crates/` emitter**. Registered: `factor-a` on `vec.cpp` is false in the **surplus** direction |
| **D3** | **the writer composition** — `.text` COMDATs + non-COMDAT `.data` + `.bss` (+ a plain-data `.rdata` COMDAT) | `w-nc`'s price, re-derived here. Registered: 0 of the 5 production emitters (`emit_comdat_obj`, `emit_obj`, `emit_empty_obj`, `emit_data_obj`, `emit_dyninit_obj`) emits that composition |
| **D4** | `_fltused` | registered **already paid** (C4). If C4 misses, this is one line, `w-blockir`'s |
| **D5** | the label counter over the composed obj | 0 `$M`/`$T` in the reference obj (no framed function), so registered at **0** for this TU |
| **D6** | the `.rdata` COMDAT for `?npos@…` — a **plain 4-byte data** COMDAT, **not** `.rdata$r` RTTI (the `w-eh5` retraction's distinction) and **not** an FP constant pool | registered as its own mechanism, folded into D3 |
| **D7** | repairing the pre-existing `HATCH-DRIFT` in `calls.rs` | declined in advance — #1322 makes the disposition a judgement about the lane that moved the needle |

---

## 5. PRE-ARMED PLACES FOR THE UNNAMED REFUSAL — budget ONE

1. **FENCE ORDER / CLAUSE REACHABILITY.** Any fixture cell added must be
   reachable *past* `IlBundle::functions()`; a cell that dies in the parser
   grades the parser and not the writer. Armed: every new fixture is checked
   for its refusal KEY, not just its class.
2. **A `_neg` cell that shares a key with its positive.** `_neg` cells must have
   **distinct probe-verified keys** or they are one cell twice.
3. **The base counterfactual.** `work/w-vec/c2rs-base` is the merge-base binary,
   copied before any edit. #2409: a `git checkout master -- crates/` round trip
   is NOT a counterfactual.
4. **The fixture list regenerated AFTER the last fixture** and `wc -l`-checked
   against `ls fixtures/cpp/*.cpp | wc -l` — `w-fltret` §9.2's third unnamed
   refusal was a list that omitted its own `_neg` file.
5. **A fixture that cannot compile at `/Ox /GS- /c`** needs a `// c2rs-profile:`
   marker with a reason (#2330–#2335).
6. **Board #1380** — commit before any revert; every scratch patch is committed
   as a `.patch` and never as a `crates/` change.

---

## 6. WHAT THIS LANE WILL NOT DO

* It will **not** widen `Bindings::per_record` to bind fewer names than segments.
  That is the one change that would let a wrong obj out of the gate on 851 TUs.
* It will **not** add a section name to `PORT_WRITER_SECTIONS`. Factor C is
  already true on this TU; adding a name would inflate C and convert nothing
  (board #278, #301).
* It will **not** quote a per-function byte figure as a distance to a TU match.

---

## 7. THE LABEL CHANNEL

`LABEL_COUNTER.md` **§7.6**'s six-step procedure if a label question arises:
subject in the middle of the function list, base in the same obj, subtract
`minted`, predict at `/O1` then confirm with one compile. The **counterfactual
form is NOT used** — `wb-label` (#2430–#2440) established c1xx and c2 share one
symbol-id space, so that form measures Δseed + Δcharge.

**Registered: this TU asks no label question.** Its reference obj contains no
`$M`/`$T` symbol at all (34 symbols, enumerated in §2), because neither function
is framed. D5 is registered at 0.
