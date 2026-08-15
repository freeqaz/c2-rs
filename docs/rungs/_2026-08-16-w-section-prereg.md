# PREREG — `w-section`: what the `.rdata`/`.data` COMDAT SECTION EMITTER actually does

    Lane:      w-section (`wt-w-section`)
    Kind:      characterization — what does c2's `.rdata`/`.data` COMDAT section
               emitter do at the WORKLOAD's profile, and what does discharging
               `data-sym-not-extern` actually cost?
    Base:      master `202bfc3f`, worktree branch `wt-w-section`
    Frozen:    BEFORE the first `crates/` change of this lane. Everything in §2
               below was measured BEFORE this file was frozen and is labelled
               **MEASURED, not predicted** — it is reported as measurement and
               scores nothing.
    Predicts:  §4, probability form, denominator beside every numerator.
    Mutants:   §5, colours registered here BEFORE any run.

---

## 0. Why this lane exists

`w-bind16` (board **#3195**–**#3199**, `docs/rungs/2026-08-16-bind.md`, measured at
base `55933035`) laddered the reachable head and terminated on
**`data-sym-not-extern` — 1,458 of 3,062 (47.6 %)**, naming it *"§17.2 item 7's
`.rdata`/`.data` COMDAT **SECTION EMITTER**"*, and closed with *"the next lane on
this head should be a **section-emitter** lane, not a reader lane."* This is that
lane. `coff/` is single-occupancy and nobody is in it.

## 1. Populations, named every time (#3125)

| tag | population | denominator | base at `202bfc3f`, **measured in this worktree** |
|---|---|---|---|
| **P-W** | 878-TU **workload scan** | **878** TUs | `match` **25** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **845** · `capture-fail` **8** |
| **P-E** | **emitted functions** in P-W | **162,049** | `fnbyte-refused-parse` **113,612** · `fnbyte-exact` **35,734** |
| **P-M** | **modeled-reachable** subset of P-E (`emit-cflow-modeled-key\|*`) | **3,062** fns / **30** keys | head **801** / **529** / **495** / **464** |
| **P-F** | **fixture gate** — `gate.sh`, 381 fixtures × 18 mode lanes | **381**/lane | — |
| **P-C** | this lane's **capture cells** — probe `.cpp` compiled by real `c2.dll` | named per cell | §2 |

**`match` in this document always means P-W**, never the 381×18 fixture gate and
never `c2rs perf`'s `/Ox` gate. P-M reproduces `#3177`'s and `w-bind16`'s table
**exactly** at this lane's own base — 30 keys, 3,062 accounted, head
`801 / 529 / 495 / 464` — so the two bases agree and no figure is inherited.

The anchored `gap-metric` key count on this tree is **394** over 394 lines
(`grep -oE "^ +gap-metric [^ ]+" | sed 's/.*gap-metric //' | sort -u | wc -l`),
independently reproducing `w-bind16` §10.1 — **third** lane, and still not 370
or 372.

## 2. MEASURED BEFORE THE FREEZE — the section grid (P-C). Scores nothing.

Cells in `work/w-section/cells/`, compiled by real `c2.dll` under wibo at the
**workload's own profile** `/nologo /c /GR /O1 /Oi /EHsc`, dumped with
`scripts/gt_dump.py`. These are stated as **measurement**, not prediction,
because they were taken before this file was frozen. Registering them as
predictions would be scoring a coin after it landed.

**R-SEC — the `/O1` string-literal `.rdata` COMDAT rule, as a series:**

1. **One `.rdata` COMDAT per DISTINCT literal**, deduped by bytes over the whole
   TU (`s4`: `f1` and `f2` both pass `"aa"` → **one** section; `rev`: `"bb"` in
   `f1` and `f3` → **one**).
2. Placed **immediately after the `.text` of the FIRST function that references
   it** (`s3`, `s7`, `rev` — `.text .rdata .text .rdata .text`).
3. Within one function, **source-argument / first-reference order** — `n = 1…4`
   cells `s1`, `s2`, `n3`, `n4` give `aa`, `aa bb`, `aa bb cc`, `aa bb cc dd`,
   which is the **reverse** of the `lis` emission order `w-bind16` §5.1
   measured.
4. Raw size = literal bytes **+ NUL, with no padding** (`len1`…`len65`:
   raw = n+1 exactly).
5. `Characteristics` = `0x40001040 | (nibble << 20)` — MEM_READ |
   CNT_INITIALIZED_DATA | LNK_COMDAT — nibble **3** (align 4) below raw 64 and
   **4** (align 8) at and above it (`len63` raw 64 → nibble 4; `len33` raw 34 →
   nibble 3).
6. `Selection` **2** (SELECT_ANY), aux `CheckSum` = the **real** CRC, `Number` 0.
7. Symbols: the `.rdata` section symbol (STATIC, 1 aux) then the `??_C@_0…`
   symbol, **EXTERNAL**, `Type` 0, `Value` 0.
8. Relocations are **REFHI + PAIR / REFLO + PAIR against the `??_C@` symbol
   itself**, addend 0 — **no pool anchor, no offset difference** (`s2`'s four
   relocation records name both literals).

**And the reason that matters:** rule 2 is the interleave
`crates/c2-core/src/coff/writer.rs` **already implements** for FP-constant
`.rdata` pools (*"the `.rdata` pools this function introduces … immediately
after its `.text`"*). At the workload's profile the string COMDAT is **not** *"a
section in the middle of the section table"*.

**Item 7's own population, separately.** A **defined** global (`s5`, `int g=5;`)
and a **static** one (`s6`, `static int sg=5;`) BOTH put a non-COMDAT `.data`
**before** `.text`, immediately after the second `.XBLD$W`. `IL_CALL_IN_EXPR.md`
§17.2 item 7's parenthetical says *"before the second `.XBLD$W` for a defined
one, **after `.text` for a static one**"* — the static half does not reproduce at
`/O1`.

## 3. The reading rule, applied to §17.2 item 7 (four for four: #3114, #3119, #3151, #3165)

* **TITLE:** *"A defined or static global is out of class and must stay out."*
* **ENFORCING LINE:** *"Today `gl_defined_names`'s unclaimed-name rule already
  refuses every such TU, and any rung that makes a data reference 'accounted
  for' must re-impose this or it will emit a 5-section obj against a 6-section
  reference."*

The title is about **defined/static globals**. The enforcing line names a
**whole-TU accounting** rule in `gl_defined_names`, not a section emitter. The
census key that cites it — `body/mod.rs:1464` `DATA_SYM_LINKAGE`, doc comment
*"puts a `.data`/`.bss` section into the middle of the section table"* — is
raised for **any** name that is not an undefined external, and after `w-bind16`'s
L3 its population is **79.9 % string literals**, which §2 shows are placed
**after** `.text` and are **COMDAT**. **Title, enforcing line and citing key have
three different populations.**

## 4. Predictions — registered here, scored in the rung doc

Every rung below is `crates/c2-il` only, built `--release`, scanned over the full
878-TU workload, then **reverted**; plus an identity control.

| id | prediction | denominator | P |
|---|---|---|---:|
| **P1** | **L1** (`gl.rs:1085` `NAME_SEPARATORS += 0x25`, and nothing else) moves **> 60 %** of `data-sym-unresolved:eof` into `data-sym-not-extern:eof` | 529 (P-M) | 0.75 |
| **P2** | **L2** (L1 + `calls.rs:431` + `calls.rs:437`) reproduces `w-bind16`'s **`data-sym-not-extern:eof` = 1,458** to within ±5 | 3,062 (P-M) | 0.70 |
| **P3** | **L3**'s prefix split puts **≥ 90 %** of `data-sym-not-extern` under `??_C@` (the **string** emitter) and **< 10 %** under the `.data`-global emitter | the L2 count | 0.80 |
| **P4** | `match` (P-W) is **25**, Δ0, at every rung | 878 | 0.95 |
| **P5** | `mismatch` is **0** at every rung | 878 | 0.93 |
| **P6** | `fnbyte-exact` is **35,734**, Δ0, at every rung | 162,049 | 0.90 |
| **P7** | `fnbyte-refused-parse` is **113,612**, Δ0, at every rung | 162,049 | 0.85 |
| **P8** | `codegen-gap` is **0** at every rung — nothing newly reaches the port | 878 | 0.85 |
| **P9** | the identity control (revert) reproduces base at **0 deltas over all 30 keys** of P-M | 30 keys | 0.95 |
| **P10** | at **`/Ox /GS- /c`** the same `s2` source puts its string pool **BEFORE** `.text`, in **one** section, and **does not dedup** `s4`'s repeated literal — i.e. §17.2 item 2 reproduces and §2's rules 1, 2 and 4 are `/O1`-only | 2 cells | 0.85 |
| **P11** | at **`/Ox /GS- /c`** a **static** defined global's `.data` is placed **after** `.text` — i.e. §17.2 item 7's parenthetical is right at its own profile and the §2 disagreement is a profile split, not an error | 1 cell | 0.70 |
| **P12** | **the lane DECLINES the build.** Discharging one `data-sym-not-extern` function needs **four** subsystems — the `.gl` name (separator), the `.in` literal bytes, `codegen`'s `lis`/`addi` pair with REFHI/REFLO, and the `.rdata` COMDAT section — of which **only the last is in `coff/`** | 4 subsystems | 0.60 |

**Registered bias.** I expect to **over-credit the section emitter's readiness**:
§2 rule 2 says the placement machinery already exists, and the temptation is to
read "the section emitter is nearly done" as "the row is nearly convertible". The
honest counterweight is registered in advance: **`match` will not move**
(#3182 — 845 must convert and the median unconverted TU needs six of six
mechanisms; #3190 — a *perfect* reader converts **two** TUs), so any positive
number this lane produces is in **P-E** or **P-M**, never in P-W.

## 5. Mutants — colours registered BEFORE any run

Probe: `cargo test --workspace --release --no-fail-fast`, base to be recorded.
One site per mutation; the patcher aborts unless the site count is exactly 1 and
prints it, so a vacuous patch fails loudly. Each is applied, built, tested,
reverted.

The mutants probe **the existing section emitter's rules**, because this lane's
central claim is *"the `.rdata` COMDAT placement machinery is already built and
graded"*. If those rules can be broken with nothing failing, the claim is worth
less and the lane must say so.

| id | site | mutation | **registered** |
|---|---|---|---|
| **M1** | `c2-core/src/coff/dyninit.rs` string `.rdata` | `checksum: coff_checksum(l.bytes)` → `checksum: 0` | **RED** |
| **M2** | `c2-core/src/coff/mangle.rs` | `LITERAL_TEXT_BYTE_LIMIT: usize = 32` → `31` | **RED** |
| **M3** | `c2-core/src/coff/writer.rs` per-function `.rdata` pool | `selection: 2` → `selection: 0` | **RED** |
| **M4** | `c2-core/src/coff/writer.rs` per-function `.rdata` pool | `if double { CH_RDATA_F64 } else { CH_RDATA_F32 }` → `CH_RDATA_F32` | **RED** |

## 6. Standing rules this lane holds itself to

* **`mismatch` must be 0.** It outranks every deliverable. A wrong emit is
  strictly worse than a gap; if one appears, stop, report it as an alarm and
  revert.
* **`codegen::labels` remains the single reader of a pending intra-section branch
  site.** No second fixup list.
* **Absence is not success.** The decline probe must be shown to FIRE — the
  count of discriminating cells is printed, and a rung that moves no key at all
  is not evidence of anything.
* **No build without a closed recognizer AND a series** (#3147 — `w-slots` read
  `3` off the objs and the series was `2n+1`).
* Every figure carries its **population** and the **commit it was measured at**.
