# w-xtea2 — PREREG, frozen before the first `crates/` change

Lane `w-xtea2`, 2026-08-09. Branch `w-xtea2`, base `af81b869` (master tip, the
`w-front5` merge). **Nothing under `crates/` is modified at the moment this file
is committed** — checked with `git status` and recorded in the commit that adds
it.

Commission: convert `src/system/utl/EncryptXTEA.cpp`, TU match **22 → 23**.

---

## 0. WORKLOAD STAMP (#2392 — dc3 is not pinned)

```text
c2-rs        af81b869  (worktree .claude/worktrees/agent-aec9c3cccb647adc3, branch w-xtea2)
base binary  work/w-xtea2/c2rs-base   md5 a9c8c28a72f129aa53831a350e7a8b10
             (scan provenance row: binary_sha 1a1f69c2a95f593a82b790cb05045f5e)
dc3-decomp   d7a3c1aa9d5d57a1176790c0e15a723edd2e03a0   2026-08-09T13:09:42Z
             workload_dirty false as the scan read it; 2 untracked paths in the tree
cl.exe       compilers/X360/16.00.11886.00/cl.exe
c2.dll       compilers/X360/16.00.11886.00/c2.dll
wibo         wibo 1.2.0-c2rs.1 (Linux x86_64), known-good 1.0.1-23
workload     878 TUs, flags /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc + 8 /I roots
```

Base scan, this lane's own run (`work/w-xtea2/base_metrics.txt`):

```text
match 22 · mismatch 0 · codegen-gap 0 · vocab-gap 849 · port-error 0
capture-fail 7 · frontier 5
fnbyte-exact 35798 · fnbyte-differs 1898 · fnbyte-refused 114634
```

Every digit agrees with `w-front5`'s tip table, so the base is the same tree it
measured and no re-derivation of *its* numbers is inherited unchecked.

---

## 1. DECLARED PRIORS — measured BEFORE this file froze, and therefore NOT scored

These are read-only measurements taken during orientation. They are declared
here so that nothing below can be scored as a prediction that was already known,
which is the defect `w-pool` #2571 recorded against its own C11.

**A. `CEILING.md` §11.4 item 8 — the GATE binds, off this lane's own scan row.**
`EncryptXTEA.cpp`'s row reads `gate_cause "body-out-of-class"`, `gate_causes
["body-out-of-class"]` — **no `gl-stop-*` and no `bind-*` clause** — with
`emit-bound 5`, `emit-gate-segments 5`, `emit-record-offsets 5`, `emit-records
5`. `w-front5`'s 5:5 is confirmed by the field §11.4 item 8 names, not by
`fn_names` (which reads **4** on this TU against `fn_total 5` — the loose
census scan, and exactly the field #2621 warns is not the answer).

**B. The byte judge (item 1).** `fnbyte-denominator 5 · fnbyte-exact 1 ·
fnbyte-differs 0 · fnbyte-refused 4`, all four `fnbyte-decline|parse`.
`bytefrac 16 / 272`. The whole remaining distance is 4 bodies / **256 B**.

**C. The symbol table (item 3).** `work/w-xtea2/ref/xtea.dump`, captured at the
workload's own flags: 10 sections, 34 symbols, 1,942 B. NC-1 obligations
present: `memcpy` (undefined external, placed immediately after `?SetKey`'s own
function symbol), `__restgprlr_26` and `__savegprlr_26` (placed AFTER the
`.pdata` group and its `$T`, LIFO), and the label triple **`$M2757` (idx 27,
value 0x60) · `$M2756` (idx 28, value 0xc) · `$T2758` (idx 31, `.pdata`)** —
the `$M` pair itself LIFO. **No `_fltused`** (no float function) and **no
`__real@` pool**, so NC-1 items 1 and 2 are vacuous here and item 7's memcpy
external is live. NC-2: 5 distinct section names, `.pdata` present because one
function is framed; factor C already passes (the TU is inside `A∧B∧C`).

**D. The label channel, measured by `LABEL_COUNTER.md` §7.6's IN-THE-MIDDLE
form and never the counterfactual** (`work/w-xtea2/labgrid.py`,
`LABGRID.txt`). `.gl` counter = **2721**; `plan_labels` seeds
`2721 + 9 + 3·5 = 2745`; the obj's first label is **2756**, so **11 slots** are
consumed ahead of `?Encrypt@`'s `$M`. The grid decomposes them, at the
workload's own `/O1`, with `base = 5` holding on every row:

| probe | stride `/O1` | stride `/Ox` |
|---|---:|---:|
| `ctl-plain` (framed control) | 5 | 4 |
| `ctl-leaf` | 1 | 1 |
| `ctl-leaf-for` | 3 | 9 |
| `x-ctor` | **1** | 1 |
| `x-setkey` | **2** | 2 |
| `x-setnonce` | **1** | 1 |
| `x-encipher` | **3** | **13** |
| `x-encrypt` | 9, **extra 4**, minted 7 | 41, extra 37 |
| `x-encrypt-alone` | 9, extra 4 | 8, extra 4 |

`1 + 2 + 1 + 3 + 4 = 11`, which is the obj's own number **exactly**, with no
term fitted. Today's `plan_labels` charges `1 + 2 + 1 + 1 + 0 = 5`, so the
port is **6 slots low**. Every `/Ox` column is a different number.

**E. The board (item 7), grepped before sizing.** `#2344` (no 64-bit
rotate/mask encoder anywhere in `c2-core`), `#2340`/`#2341` (the label charge as
the binding constraint), `#2567` (`stdu`/`stdx` missing, `encode_addic`
present), `#1980`/`#1981` (the counted-loop class explicitly excludes a memory
reference and declines the update-form pass), `#746`/`#747` (`label_slots`'
`None` for every loop class and the fixture that grades it).

**F. Factor A (item 6).** Inside `A∧B∧C` — it is a frontier member, so codegen
alone can convert it.

---

## 2. PREDICTIONS — the scored list

Probabilities are the lane's own, frozen here. A census-only prediction is
unscored by standing instruction, so every row below is in bytes, verdicts or
symbols.

| # | prediction | p |
|---|---|---:|
| **P1** | **`EncryptXTEA.cpp` CONVERTS — TU match 22 → 23** | **0.25** |
| P1a | if it does not convert, the decline is priced at **N ≤ 12** named mechanisms (against the standing `≥ 27`), because term 6 of that price — the label charge — is now a measured constant rather than an unpredictable one | 0.65 |
| **P2** | **`fnbyte-exact` delta = +4** (the four blocked bodies, and no other function moves) | 0.28 |
| P2a | `fnbyte-exact` delta ≥ +1 | 0.55 |
| P2b | `fnbyte-exact` delta ≥ 0 — **nothing regresses** | 0.95 |
| P3 | exactly **5** new encoders are needed: `rldicl` (covering `clrldi`), `rldimi`, `stdu`, `stdx`, and a record form of `addic` | 0.70 |
| P3a | ≥ 4 new encoders | 0.90 |
| P4 | the new classes must **refuse at `/Ox`** — a mode gate in the PARSER (#1638) — because every `/Ox` label stride in prior D differs from its `/O1` twin | 0.85 |
| P5 | **`mismatch` 0 on every gate row**, both modes, including `expr_sweep` and `mode_cross` | 0.90 |
| P6 | the 878-TU verdict set moves for **`src/system/utl/EncryptXTEA.cpp` and nothing else** (0 other TUs move in either direction) | 0.90 |
| P7 | the label plumbing needs **≥ 3** separate changes — `IlFunction::label_slots`, `coff::plan_labels`, and `IlBundle::functions`' `label_slots(false)? != label_lead() + 1` comparison | 0.80 |
| P8 | `?Encipher@`'s 29 words can be **transcribed** byte-exact with no scheduler pass, exactly as `w-blockir` transcribed its twenty | 0.85 |
| P9 | test-count DELTA: **targets 41 → 41 or 42** (at most one new integration-test FILE, which is a new TARGET), test count strictly up | 0.75 |
| P10 | `?SetKey@`'s `b memcpy` needs the `memcpy` symbol placed in the **callee region** (immediately after its own function symbol), NOT on `helper_externals` after a `$T` — the opposite of `w-ifn` #2354's placement, because `?SetKey@` is a LEAF with no `$T` | 0.70 |
| P11 | the two `rlwinm` fusions in `?Encipher@` (`8,11,2,28,29` and `7,11,23,28,29`) are covered by the existing `encode_rlwinm` with no new encoder | 0.90 |

## 2.1 The falsifiability note this PREREG owes

`w-front5` marked three of its own high-confidence rows as ones a zero-diff lane
*cannot lose*. The same audit applies here **before** the fact: **P5 and P2b are
unlosable if this lane ships nothing**, and P6 is nearly so. They are registered
because they are the gate's own obligations, and they are flagged here so that
hitting them is not counted as calibration. **The losable rows are P1, P1a, P2,
P2a, P3, P4, P7, P8, P10 and P11** — ten of fourteen, and every one of them can
go either way on a lane that ships code.

---

## 3. DECLINE CLAUSES, each with a size

| # | clause | size |
|---|---|---|
| **D1** | If `?Encipher@`'s 116 B / 29 words cannot be transcribed byte-exact, DECLINE the TU rather than approximate the schedule. | 116 B, 29 words |
| **D2** | If the label charge cannot be paid without giving `label_slots` a **mode parameter** or a **sub-shape parameter**, DECLINE. `LABEL_COUNTER.md` §7.6's box forbids both and `w-blockir`'s "sub-shape dependence" turned out to be the seed. A class may ship a measured constant obtained by steps 1–5 and nothing else. | 6 slots |
| **D3** | Do **not** widen `gl_defined_names` / `bind::defined_name_set`. `w-front5` #2622/#2623 measured that repair at **0 conversions and −1 `fnbyte-exact`**: the walk and the inline fence are one function, so widening one tightens the other. | −1 `fnbyte-exact` |
| **D4** | Do **not** relax `LabelMap`'s invariant 4 (the backward-branch refusal). Every shipped loop class computes its back edge through `encode_bc` directly and never routes through the map; a new one does the same. | 0 lines in `labels.rs` |
| **D5** | If a fixture cell cannot compile at `/Ox /GS- /c`, declare a `// c2rs-profile:` marker with a reason (#2330–#2335) rather than dropping the cell. | per cell |
| **D6** | Do **not** widen `counted_accum_loop` to admit a memory reference. #1981 defines that class to contain none and declines the update-form pass BY NAME; a new class is a new class. | 0 lines in `counted_accum_loop` |
| **D7** | One **unnamed refusal** is budgeted. If a second unnamed refusal appears, stop and price rather than pay it. | 1 |

---

## 4. BOARD CHECK, PRE-ARMED

Rows **#2660–#2689** are this lane's. Before sizing any rung, `grep BOARD.md`
for the key — five rows have re-entered a ranking after already measuring zero
([[check-the-board-before-dispatching]]). The keys already grepped are in prior
E. Any row this lane does not mint is declared UNMINTED in the rung, not left to
be inferred from a gap.

---

## 5. WHAT THIS LANE WILL NOT CLAIM

* It will not quote `w-xtea`'s `≥ 27` or `w-front5`'s inventory as a *result*.
  Both are re-derived here or explicitly left standing.
* It will not treat a census gain as a goal gain (§10.2 — `+444` emitted moved
  `fnbyte-exact` by zero).
* It will not treat a `gate-cause` SET as a price (#2560).
* `mismatch 0` is not evidence of correctness (trap 1).
