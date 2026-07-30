# W6 at `/O1` — the comparison-leaf byte table (CHARACTERIZATION)

The complete `/O1` reference for the comparison-leaf class `return a <rel> k`
(`rel` ∈ `< <= > >= == !=`, `a` ∈ {`int`, `unsigned`}, `k` over
`0 1 -1 5 -5 2 32767 -32768 65535` and their unsigned forms) — the same matrix
the `/Ox` spines in `docs/CODEGEN_W6_COMPARE.md` were built from, so the port
can be re-targeted from this table without re-capturing. This is what
`docs/OPT_MODE.md` §4.1 item 3 lists as the first open `/O1` gap (`w6_rel_k`,
14 of 19 leaves reallocated).

**Every byte below is transcribed from a live 16.00.11886.00 capture**
(2026-07-30, `c2rs compile --flags-file … --keep-obj …`, cl.exe under wibo).
Nothing is inferred or computed. Method and sanity gates:

* One 108-function TU (6 relations × 2 signednesses × 9 literals) compiled
  twice, differing only in the flag: `/O2 /GS- /c` vs `/O1 /GS- /c`. `/O2`
  stands in for `/Ox` because `/O1` and `/O2` both imply `/Gy` and `/Ox` does
  not — comparing `/Ox` directly would mix the COMDAT-vs-packed section layout
  into the byte compare (`OPT_MODE.md` §3.3).
* Function bytes extracted **via each symbol's section+value** from the COFF
  symbol table, never by concatenating `.text` sections.
* Sanity gate 1: the same extractor on `il_bool_materialization.cpp` at `/Ox`
  reproduces the §3 listing of `CODEGEN_W6_COMPARE.md` byte-for-byte.
* Sanity gate 2: `w6_rel_k.cpp` at `/O2` vs `/Ox` — all 19 functions
  byte-identical, so the `/O2`-for-`/Ox` substitution is sound on exactly this
  class. At `/O2` vs `/O1` the same fixture gives 5 identical / 14
  register-only, matching `OPT_MODE.md` §4.2.
* Determinism: the `/O1` TU captured twice; all 108 functions byte-identical.

## 1. The verdict: allocator tweak, not a separate lowering

Across all 108 cells: **34 byte-identical, 74 differ, and every one of the 74
differs only in register fields.** Same instruction count, same opcodes, same
operand *order*, same immediates, same schedule (including the interleaved
`li`/`srawi`/`rlwinm` placements inside the wide-literal spines). This was
checked mechanically: masking the register fields of every instruction makes
all 108 pairs equal. No cell differs in opcode, operand order, or instruction
count. The `/Ox` spines, folds, literal-materialization choices and emission
order all carry over to `/O1` untouched.

### 1.1 The rule, stated exactly

`OPT_MODE.md` §3.1's chain rule survives, with the comparison class pinning two
details the chains could not distinguish:

> **`/O1` is `/Ox` with exactly one change — the temp allocator:**
> a temp whose defining instruction makes the **last use of the value currently
> in r11** is written to r11 rather than to a fresh descending register.
> Fresh temps are numbered descending from r11 in the same (value-numbering)
> order as `/Ox`, but the descending counter **advances only on fresh
> allocations** — after a reuse it stays where it was, so `/O1`'s fresh
> registers sit higher than `/Ox`'s from that point on. The `subfe` don't-care
> source, which `/Ox` names with a fresh descending number, is named **r11**
> at `/O1` (r11's value is always dead there in this class).

A simulator implementing exactly this — and nothing else — reproduces the real
`/O1` bytes from the `/O2` bytes for **108/108** matrix cells and **19/19**
`w6_rel_k` functions.

Two consequences that a naive "rewrite temps to r11" transform gets wrong:

* **Reuse requires consumption, not just death.** `u_le_k`'s `li r10,-1` is
  numbered while the `subfic` result in r11 is already dead *as a register*
  (only its carry is pending), yet it does **not** take r11 — it reads nothing,
  so it cannot be the last use of r11's value. Two readings fit every capture:
  (a) reuse requires the def to *read* r11's dying value; (b) liveness counts
  a pending carry as part of the defining value (the `subfic` value is "live"
  until `subfze` consumes CA). No cell in this class distinguishes them; the
  discriminating probe (a constant materialized after r11's register *and*
  carry are both dead) needs a shape outside the leaf class. UNRESOLVED which
  is c2's mechanism; both give identical bytes here.
* **The counter does not advance on reuse.** `s_ne_lo`: `/Ox` numbers
  `li`→r11, `subf`→r10, `addic`→r9; `/O1` folds the `subf` onto r11, and the
  `addic` — a fresh temp (its `subf` operand is still live) — takes **r10**,
  not r9.

### 1.2 Which cells are mode-identical (34 of 108)

* Ten of the twelve `k == 0` folds: `s_lt_0`, `s_ge_0`, `s_eq_0`, `s_ne_0`,
  `u_lt_0`, `u_le_0`, `u_gt_0`, `u_ge_0`, `u_eq_0`, `u_ne_0`. The other two —
  `s_le_0` and `s_gt_0` — **differ**: their 3-instruction fold has a
  foldable-onto-r11 step (`orc`/`andc` consumes the dying `neg`), which the
  1–2-instruction folds do not.
* Signed and unsigned `!=` with an `addi`-reachable `k` (10 cells):
  `s_ne_{1,m1,5,m5,2,hi}`, `u_ne_{1,5,2,hi}` — the `addic` after the `addi`
  still sees r11 live (the `subfe` tail reads both), so both modes are forced
  into two registers.
* Unsigned `>=`/`<=` with a SIMM16-encodable `k` (14 cells):
  `u_ge_{1,max,5,m5,2,hi,lo}`, `u_le_{1,max,5,m5,2,hi,lo}` — the
  li/subfic/subfze (resp. li/subfc/subfze) triple is fully constrained in both
  modes. Note `/Ox` itself already reuses r11 for the `subfc` destination in
  `u_ge_k` (the `?uc_ge` quirk of `CODEGEN_W6_COMPARE.md` §6), which is why
  these are identical rather than merely reg-only.

Everything signed with `k ≠ 0` except the `!=` family differs; every unsigned
`<`/`>` with `k ≠ 0` differs; every wide-literal (`65535`) cell differs.

## 2. `/O1` spine templates, transcribed

Registers below are the captured `/O1` allocation for a first (and only)
parameter in r3. `k`-dependent immediates are marked. All tails end `blr`
(`4e800020`), omitted here.

**signed `<`/`>` (6 instr; `<` swaps the roles exactly as at `/Ox`):**
```
s_gt_k   li r11,k ; subfc r10,r3,r11 ; eqv r11,r3,r11  ; rlwinm r11,r11,1,31,31 ; addze r11,r11 ; rlwinm r3,r11,0,31,31
s_lt_k   li r11,k ; subfc r10,r11,r3 ; eqv r11,r11,r3  ; rlwinm r11,r11,1,31,31 ; addze r11,r11 ; rlwinm r3,r11,0,31,31
```
The dead `subfc` destination stays at fresh r10 in both modes (r11 holds the
literal, still live for the `eqv`); everything after the `eqv` collapses onto
r11. The `eqv` rS/rB order still records the source operand order.

**signed `>=`/`<=` (5 instr; the two shift terms still emit in source order):**
```
s_ge_k   li r11,k ; srawi r10,r3,31        ; rlwinm r9,r11,1,31,31 ; subfc r11,r11,r3 ; adde r3,r9,r10
s_le_k   li r11,k ; rlwinm r10,r3,1,31,31  ; srawi r9,r11,31       ; subfc r11,r3,r11 ; adde r3,r10,r9
```
Only the dead `subfc` destination moves (r8 → r11, its `li` operand dies
there); the three live terms keep the `/Ox` registers r10/r9.

**unsigned `>` (SIMM16 k: 3 instr) / `<` (4 instr):**
```
u_gt_k   subfic r11,r3,k ; subfe r11,r11,r11 ; rlwinm r3,r11,0,31,31
u_lt_k   li r11,k ; subfc r11,r11,r3 ; subfe r11,r11,r11 ; rlwinm r3,r11,0,31,31
```
The `/Ox` never-defined `subfe` source register (r10 resp. r9) does not exist
at `/O1`: the entire spine runs in r11, and the `subfe` source is a *defined*
(dead) value. `/O1` code of this class reads no undefined register.

**unsigned `>=`/`<=` (SIMM16 k — mode-identical):**
```
u_ge_k   li r11,k ; li r10,-1 ; subfc r11,r11,r3 ; subfze r3,r10
u_le_k   li r10,-1 ; subfic r11,r3,k ; subfze r3,r10
```

**`==`/`!=` (identical for int/unsigned at the same encodable k):**
```
eq_k  (addi path)   addi r11,r3,-k ; cntlzw r11,r11 ; rlwinm r3,r11,27,31,31
ne_k  (addi path)   addi r11,r3,-k ; addic r10,r11,-1 ; subfe r3,r10,r11     [mode-identical]
eq/ne (li path)     li r11,k ; subf r11,{a-k or k-a: see §3.3} ; then the tail with r10 for ne's addic
```

**wide literal (65535):** the `lis r11,0 ; ori r11,r11,65535` pair keeps its
`/Ox` schedule (the `srawi`/`rlwinm`/`li -1` of the surrounding spine still
interleaves between `lis` and `ori`); the `ori` destination folds onto r11
where `/Ox` used a fresh register (`u_ge_w`/`u_le_w`: `ori r9` → `ori r11`).

## 3. Asymmetries and traps (all mode-independent, all captured)

### 3.1 `/Ox` skips r11 entirely in `u_eq_w`/`u_ne_w`

`a == 65535u` at `/O2`/`/Ox` is `addis r10,r3,-1 ; addi r10,r10,1 ;
cntlzw r9,r10 ; rlwinm r3,r9,27,31,31` — the temp numbering **starts at r10**,
r11 is never touched. Unexplained. At `/O1` the same skeleton allocates from
r11 as normal (`addis r11 ; addi r11 ; cntlzw r11`). Any port logic that
assumes "/Ox first temp = r11" mis-emits these two cells on the `/Ox` side.

### 3.2 unsigned wide `==`/`!=` use a different materialization than signed

`s_eq_w` (`== 65535`) is `lis r11,0 ; ori r11,r11,65535 ; subf` (k−a), but
`u_eq_w` (`== 65535u`) is the two-instruction `addis a,-1 ; addi +1` trick
(computing a−65535 directly) — one word shorter. Same k, same relation,
signedness flips the *selection*. `CODEGEN_W6_COMPARE.md` §4.1's wide-literal
row (`lis;ori;subf`) is therefore signed-only for `==`/`!=`.

### 3.3 The `subf` direction flips between `s_*_lo` and every other li-path cell

`s_eq_lo`/`s_ne_lo` (`k = -32768`) compute `subf t,r11,r3` = **a − k**;
`u_eq_lo`, `u_eq_m5`, `u_eq_max`, `u_ne_*` (li path) and `s_eq_w`/`s_ne_w`
(lis/ori path) all compute `subf t,r3,r11` = **k − a**. Value-equivalent under
the ==0 tail, but `subf` is non-commutative in the bytes — a spine hardcoding
one direction mis-emits the other cells. Not explained by signedness alone
(s_eq_w is k−a); only the signed li-materialized `-32768` cells use a−k.

### 3.4 unsigned `eq/ne` addi-path eligibility is `k ∈ [0, 32767]`, not "−k fits SIMM16"

`u_eq_max` (`k = 0xFFFFFFFF`) takes the li+subf path even though `addi
r11,r3,1` would compute a−k exactly; likewise `u_eq_m5` (`0xFFFFFFFB`,
`addi +5` would work). Meanwhile `u_gt_m5` happily encodes the same literal as
`subfic …,-5`. So for `==`/`!=` the front-end literal's *unsigned value* gates
the addi path; for the carry spines the raw sign-extended SIMM16 encodability
gates it. A port reusing one predicate for both mis-emits.

### 3.5 No new folds at `/O1`

`u_ge_1` (`a >= 1u` ≡ `a != 0u`) still emits the general 4-instruction
`>=` spine in both modes, not the 2-instruction `!=0` fold — the `k == 0`
folds of `CODEGEN_W6_COMPARE.md` §4.6 are keyed on the literal being 0, not on
value equivalence, and `/O1` adds nothing.

## 4. The complete table

All 108 cells, `/O2` (≡ `/Ox` bytes for this class, gate 2) side by side with
`/O1`. `*` marks differing words. Every line is machine-extracted from the two
captured objs; the listing generator and the rule simulator live in the
session scratch (`/tmp/o1rel`, gitignored) and are reproducible from
`fixtures/cpp/`-style sources in minutes.

    ### s_lt_0  `int a; return a < 0;`  [IDENTICAL]
        54630ffe  rlwinm r3,r3,1,31,31       | 54630ffe  rlwinm r3,r3,1,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_lt_1  `int a; return a < 1;`  [reg-only]
        39600001  li r11,1                   | 39600001  li r11,1
        7d4b1810  subfc r10,r11,r3           | 7d4b1810  subfc r10,r11,r3
      * 7d691a38  eqv r9,r11,r3              | 7d6b1a38  eqv r11,r11,r3
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_lt_m1  `int a; return a < -1;`  [reg-only]
        3960ffff  li r11,-1                  | 3960ffff  li r11,-1
        7d4b1810  subfc r10,r11,r3           | 7d4b1810  subfc r10,r11,r3
      * 7d691a38  eqv r9,r11,r3              | 7d6b1a38  eqv r11,r11,r3
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_lt_5  `int a; return a < 5;`  [reg-only]
        39600005  li r11,5                   | 39600005  li r11,5
        7d4b1810  subfc r10,r11,r3           | 7d4b1810  subfc r10,r11,r3
      * 7d691a38  eqv r9,r11,r3              | 7d6b1a38  eqv r11,r11,r3
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_lt_m5  `int a; return a < -5;`  [reg-only]
        3960fffb  li r11,-5                  | 3960fffb  li r11,-5
        7d4b1810  subfc r10,r11,r3           | 7d4b1810  subfc r10,r11,r3
      * 7d691a38  eqv r9,r11,r3              | 7d6b1a38  eqv r11,r11,r3
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_lt_2  `int a; return a < 2;`  [reg-only]
        39600002  li r11,2                   | 39600002  li r11,2
        7d4b1810  subfc r10,r11,r3           | 7d4b1810  subfc r10,r11,r3
      * 7d691a38  eqv r9,r11,r3              | 7d6b1a38  eqv r11,r11,r3
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_lt_hi  `int a; return a < 32767;`  [reg-only]
        39607fff  li r11,32767               | 39607fff  li r11,32767
        7d4b1810  subfc r10,r11,r3           | 7d4b1810  subfc r10,r11,r3
      * 7d691a38  eqv r9,r11,r3              | 7d6b1a38  eqv r11,r11,r3
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_lt_lo  `int a; return a < -32768;`  [reg-only]
        39608000  li r11,-32768              | 39608000  li r11,-32768
        7d4b1810  subfc r10,r11,r3           | 7d4b1810  subfc r10,r11,r3
      * 7d691a38  eqv r9,r11,r3              | 7d6b1a38  eqv r11,r11,r3
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_lt_w  `int a; return a < 65535;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        616bffff  ori r11,r11,65535          | 616bffff  ori r11,r11,65535
        7d4b1810  subfc r10,r11,r3           | 7d4b1810  subfc r10,r11,r3
      * 7d691a38  eqv r9,r11,r3              | 7d6b1a38  eqv r11,r11,r3
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_lt_0  `unsigned a; return a < 0u;`  [IDENTICAL]
        38600000  li r3,0                    | 38600000  li r3,0
        4e800020  blr                        | 4e800020  blr
    
    ### u_lt_1  `unsigned a; return a < 1u;`  [reg-only]
        39600001  li r11,1                   | 39600001  li r11,1
      * 7d4b1810  subfc r10,r11,r3           | 7d6b1810  subfc r11,r11,r3
      * 7d094910  subfe r8,r9,r9             | 7d6b5910  subfe r11,r11,r11
      * 550307fe  rlwinm r3,r8,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_lt_max  `unsigned a; return a < 4294967295u (0xFFFFFFFF);`  [reg-only]
        3960ffff  li r11,-1                  | 3960ffff  li r11,-1
      * 7d4b1810  subfc r10,r11,r3           | 7d6b1810  subfc r11,r11,r3
      * 7d094910  subfe r8,r9,r9             | 7d6b5910  subfe r11,r11,r11
      * 550307fe  rlwinm r3,r8,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_lt_5  `unsigned a; return a < 5u;`  [reg-only]
        39600005  li r11,5                   | 39600005  li r11,5
      * 7d4b1810  subfc r10,r11,r3           | 7d6b1810  subfc r11,r11,r3
      * 7d094910  subfe r8,r9,r9             | 7d6b5910  subfe r11,r11,r11
      * 550307fe  rlwinm r3,r8,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_lt_m5  `unsigned a; return a < 4294967291u (0xFFFFFFFB);`  [reg-only]
        3960fffb  li r11,-5                  | 3960fffb  li r11,-5
      * 7d4b1810  subfc r10,r11,r3           | 7d6b1810  subfc r11,r11,r3
      * 7d094910  subfe r8,r9,r9             | 7d6b5910  subfe r11,r11,r11
      * 550307fe  rlwinm r3,r8,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_lt_2  `unsigned a; return a < 2u;`  [reg-only]
        39600002  li r11,2                   | 39600002  li r11,2
      * 7d4b1810  subfc r10,r11,r3           | 7d6b1810  subfc r11,r11,r3
      * 7d094910  subfe r8,r9,r9             | 7d6b5910  subfe r11,r11,r11
      * 550307fe  rlwinm r3,r8,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_lt_hi  `unsigned a; return a < 32767u;`  [reg-only]
        39607fff  li r11,32767               | 39607fff  li r11,32767
      * 7d4b1810  subfc r10,r11,r3           | 7d6b1810  subfc r11,r11,r3
      * 7d094910  subfe r8,r9,r9             | 7d6b5910  subfe r11,r11,r11
      * 550307fe  rlwinm r3,r8,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_lt_lo  `unsigned a; return a < 4294934528u (0xFFFF8000);`  [reg-only]
        39608000  li r11,-32768              | 39608000  li r11,-32768
      * 7d4b1810  subfc r10,r11,r3           | 7d6b1810  subfc r11,r11,r3
      * 7d094910  subfe r8,r9,r9             | 7d6b5910  subfe r11,r11,r11
      * 550307fe  rlwinm r3,r8,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_lt_w  `unsigned a; return a < 65535u;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        616bffff  ori r11,r11,65535          | 616bffff  ori r11,r11,65535
      * 7d4b1810  subfc r10,r11,r3           | 7d6b1810  subfc r11,r11,r3
      * 7d094910  subfe r8,r9,r9             | 7d6b5910  subfe r11,r11,r11
      * 550307fe  rlwinm r3,r8,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_le_0  `int a; return a <= 0;`  [reg-only]
        7d6300d0  neg r11,r3                 | 7d6300d0  neg r11,r3
      * 7c6a5b38  orc r10,r3,r11             | 7c6b5b38  orc r11,r3,r11
      * 55430ffe  rlwinm r3,r10,1,31,31      | 55630ffe  rlwinm r3,r11,1,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_le_1  `int a; return a <= 1;`  [reg-only]
        39600001  li r11,1                   | 39600001  li r11,1
        546a0ffe  rlwinm r10,r3,1,31,31      | 546a0ffe  rlwinm r10,r3,1,31,31
        7d69fe70  srawi r9,r11,31            | 7d69fe70  srawi r9,r11,31
      * 7d035810  subfc r8,r3,r11            | 7d635810  subfc r11,r3,r11
        7c6a4914  adde r3,r10,r9             | 7c6a4914  adde r3,r10,r9
        4e800020  blr                        | 4e800020  blr
    
    ### s_le_m1  `int a; return a <= -1;`  [reg-only]
        3960ffff  li r11,-1                  | 3960ffff  li r11,-1
        546a0ffe  rlwinm r10,r3,1,31,31      | 546a0ffe  rlwinm r10,r3,1,31,31
        7d69fe70  srawi r9,r11,31            | 7d69fe70  srawi r9,r11,31
      * 7d035810  subfc r8,r3,r11            | 7d635810  subfc r11,r3,r11
        7c6a4914  adde r3,r10,r9             | 7c6a4914  adde r3,r10,r9
        4e800020  blr                        | 4e800020  blr
    
    ### s_le_5  `int a; return a <= 5;`  [reg-only]
        39600005  li r11,5                   | 39600005  li r11,5
        546a0ffe  rlwinm r10,r3,1,31,31      | 546a0ffe  rlwinm r10,r3,1,31,31
        7d69fe70  srawi r9,r11,31            | 7d69fe70  srawi r9,r11,31
      * 7d035810  subfc r8,r3,r11            | 7d635810  subfc r11,r3,r11
        7c6a4914  adde r3,r10,r9             | 7c6a4914  adde r3,r10,r9
        4e800020  blr                        | 4e800020  blr
    
    ### s_le_m5  `int a; return a <= -5;`  [reg-only]
        3960fffb  li r11,-5                  | 3960fffb  li r11,-5
        546a0ffe  rlwinm r10,r3,1,31,31      | 546a0ffe  rlwinm r10,r3,1,31,31
        7d69fe70  srawi r9,r11,31            | 7d69fe70  srawi r9,r11,31
      * 7d035810  subfc r8,r3,r11            | 7d635810  subfc r11,r3,r11
        7c6a4914  adde r3,r10,r9             | 7c6a4914  adde r3,r10,r9
        4e800020  blr                        | 4e800020  blr
    
    ### s_le_2  `int a; return a <= 2;`  [reg-only]
        39600002  li r11,2                   | 39600002  li r11,2
        546a0ffe  rlwinm r10,r3,1,31,31      | 546a0ffe  rlwinm r10,r3,1,31,31
        7d69fe70  srawi r9,r11,31            | 7d69fe70  srawi r9,r11,31
      * 7d035810  subfc r8,r3,r11            | 7d635810  subfc r11,r3,r11
        7c6a4914  adde r3,r10,r9             | 7c6a4914  adde r3,r10,r9
        4e800020  blr                        | 4e800020  blr
    
    ### s_le_hi  `int a; return a <= 32767;`  [reg-only]
        39607fff  li r11,32767               | 39607fff  li r11,32767
        546a0ffe  rlwinm r10,r3,1,31,31      | 546a0ffe  rlwinm r10,r3,1,31,31
        7d69fe70  srawi r9,r11,31            | 7d69fe70  srawi r9,r11,31
      * 7d035810  subfc r8,r3,r11            | 7d635810  subfc r11,r3,r11
        7c6a4914  adde r3,r10,r9             | 7c6a4914  adde r3,r10,r9
        4e800020  blr                        | 4e800020  blr
    
    ### s_le_lo  `int a; return a <= -32768;`  [reg-only]
        39608000  li r11,-32768              | 39608000  li r11,-32768
        546a0ffe  rlwinm r10,r3,1,31,31      | 546a0ffe  rlwinm r10,r3,1,31,31
        7d69fe70  srawi r9,r11,31            | 7d69fe70  srawi r9,r11,31
      * 7d035810  subfc r8,r3,r11            | 7d635810  subfc r11,r3,r11
        7c6a4914  adde r3,r10,r9             | 7c6a4914  adde r3,r10,r9
        4e800020  blr                        | 4e800020  blr
    
    ### s_le_w  `int a; return a <= 65535;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        546a0ffe  rlwinm r10,r3,1,31,31      | 546a0ffe  rlwinm r10,r3,1,31,31
        616bffff  ori r11,r11,65535          | 616bffff  ori r11,r11,65535
        7d69fe70  srawi r9,r11,31            | 7d69fe70  srawi r9,r11,31
      * 7d035810  subfc r8,r3,r11            | 7d635810  subfc r11,r3,r11
        7c6a4914  adde r3,r10,r9             | 7c6a4914  adde r3,r10,r9
        4e800020  blr                        | 4e800020  blr
    
    ### u_le_0  `unsigned a; return a <= 0u;`  [IDENTICAL]
        7c6b0034  cntlzw r11,r3              | 7c6b0034  cntlzw r11,r3
        5563dffe  rlwinm r3,r11,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_le_1  `unsigned a; return a <= 1u;`  [IDENTICAL]
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        21630001  subfic r11,r3,1            | 21630001  subfic r11,r3,1
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_le_max  `unsigned a; return a <= 4294967295u (0xFFFFFFFF);`  [IDENTICAL]
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        2163ffff  subfic r11,r3,-1           | 2163ffff  subfic r11,r3,-1
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_le_5  `unsigned a; return a <= 5u;`  [IDENTICAL]
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        21630005  subfic r11,r3,5            | 21630005  subfic r11,r3,5
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_le_m5  `unsigned a; return a <= 4294967291u (0xFFFFFFFB);`  [IDENTICAL]
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        2163fffb  subfic r11,r3,-5           | 2163fffb  subfic r11,r3,-5
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_le_2  `unsigned a; return a <= 2u;`  [IDENTICAL]
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        21630002  subfic r11,r3,2            | 21630002  subfic r11,r3,2
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_le_hi  `unsigned a; return a <= 32767u;`  [IDENTICAL]
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        21637fff  subfic r11,r3,32767        | 21637fff  subfic r11,r3,32767
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_le_lo  `unsigned a; return a <= 4294934528u (0xFFFF8000);`  [IDENTICAL]
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        21638000  subfic r11,r3,-32768       | 21638000  subfic r11,r3,-32768
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_le_w  `unsigned a; return a <= 65535u;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
      * 6169ffff  ori r9,r11,65535           | 616bffff  ori r11,r11,65535
      * 7d634810  subfc r11,r3,r9            | 7d635810  subfc r11,r3,r11
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### s_gt_0  `int a; return a > 0;`  [reg-only]
        7d6300d0  neg r11,r3                 | 7d6300d0  neg r11,r3
      * 7d6a1878  andc r10,r11,r3            | 7d6b1878  andc r11,r11,r3
      * 55430ffe  rlwinm r3,r10,1,31,31      | 55630ffe  rlwinm r3,r11,1,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_gt_1  `int a; return a > 1;`  [reg-only]
        39600001  li r11,1                   | 39600001  li r11,1
        7d435810  subfc r10,r3,r11           | 7d435810  subfc r10,r3,r11
      * 7c695a38  eqv r9,r3,r11              | 7c6b5a38  eqv r11,r3,r11
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_gt_m1  `int a; return a > -1;`  [reg-only]
        3960ffff  li r11,-1                  | 3960ffff  li r11,-1
        7d435810  subfc r10,r3,r11           | 7d435810  subfc r10,r3,r11
      * 7c695a38  eqv r9,r3,r11              | 7c6b5a38  eqv r11,r3,r11
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_gt_5  `int a; return a > 5;`  [reg-only]
        39600005  li r11,5                   | 39600005  li r11,5
        7d435810  subfc r10,r3,r11           | 7d435810  subfc r10,r3,r11
      * 7c695a38  eqv r9,r3,r11              | 7c6b5a38  eqv r11,r3,r11
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_gt_m5  `int a; return a > -5;`  [reg-only]
        3960fffb  li r11,-5                  | 3960fffb  li r11,-5
        7d435810  subfc r10,r3,r11           | 7d435810  subfc r10,r3,r11
      * 7c695a38  eqv r9,r3,r11              | 7c6b5a38  eqv r11,r3,r11
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_gt_2  `int a; return a > 2;`  [reg-only]
        39600002  li r11,2                   | 39600002  li r11,2
        7d435810  subfc r10,r3,r11           | 7d435810  subfc r10,r3,r11
      * 7c695a38  eqv r9,r3,r11              | 7c6b5a38  eqv r11,r3,r11
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_gt_hi  `int a; return a > 32767;`  [reg-only]
        39607fff  li r11,32767               | 39607fff  li r11,32767
        7d435810  subfc r10,r3,r11           | 7d435810  subfc r10,r3,r11
      * 7c695a38  eqv r9,r3,r11              | 7c6b5a38  eqv r11,r3,r11
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_gt_lo  `int a; return a > -32768;`  [reg-only]
        39608000  li r11,-32768              | 39608000  li r11,-32768
        7d435810  subfc r10,r3,r11           | 7d435810  subfc r10,r3,r11
      * 7c695a38  eqv r9,r3,r11              | 7c6b5a38  eqv r11,r3,r11
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_gt_w  `int a; return a > 65535;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        616bffff  ori r11,r11,65535          | 616bffff  ori r11,r11,65535
        7d435810  subfc r10,r3,r11           | 7d435810  subfc r10,r3,r11
      * 7c695a38  eqv r9,r3,r11              | 7c6b5a38  eqv r11,r3,r11
      * 55280ffe  rlwinm r8,r9,1,31,31       | 556b0ffe  rlwinm r11,r11,1,31,31
      * 7ce80194  addze r7,r8                | 7d6b0194  addze r11,r11
      * 54e307fe  rlwinm r3,r7,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_gt_0  `unsigned a; return a > 0u;`  [IDENTICAL]
        3163ffff  addic r11,r3,-1            | 3163ffff  addic r11,r3,-1
        7c6b1910  subfe r3,r11,r3            | 7c6b1910  subfe r3,r11,r3
        4e800020  blr                        | 4e800020  blr
    
    ### u_gt_1  `unsigned a; return a > 1u;`  [reg-only]
        21630001  subfic r11,r3,1            | 21630001  subfic r11,r3,1
      * 7d2a5110  subfe r9,r10,r10           | 7d6b5910  subfe r11,r11,r11
      * 552307fe  rlwinm r3,r9,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_gt_max  `unsigned a; return a > 4294967295u (0xFFFFFFFF);`  [reg-only]
        2163ffff  subfic r11,r3,-1           | 2163ffff  subfic r11,r3,-1
      * 7d2a5110  subfe r9,r10,r10           | 7d6b5910  subfe r11,r11,r11
      * 552307fe  rlwinm r3,r9,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_gt_5  `unsigned a; return a > 5u;`  [reg-only]
        21630005  subfic r11,r3,5            | 21630005  subfic r11,r3,5
      * 7d2a5110  subfe r9,r10,r10           | 7d6b5910  subfe r11,r11,r11
      * 552307fe  rlwinm r3,r9,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_gt_m5  `unsigned a; return a > 4294967291u (0xFFFFFFFB);`  [reg-only]
        2163fffb  subfic r11,r3,-5           | 2163fffb  subfic r11,r3,-5
      * 7d2a5110  subfe r9,r10,r10           | 7d6b5910  subfe r11,r11,r11
      * 552307fe  rlwinm r3,r9,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_gt_2  `unsigned a; return a > 2u;`  [reg-only]
        21630002  subfic r11,r3,2            | 21630002  subfic r11,r3,2
      * 7d2a5110  subfe r9,r10,r10           | 7d6b5910  subfe r11,r11,r11
      * 552307fe  rlwinm r3,r9,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_gt_hi  `unsigned a; return a > 32767u;`  [reg-only]
        21637fff  subfic r11,r3,32767        | 21637fff  subfic r11,r3,32767
      * 7d2a5110  subfe r9,r10,r10           | 7d6b5910  subfe r11,r11,r11
      * 552307fe  rlwinm r3,r9,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_gt_lo  `unsigned a; return a > 4294934528u (0xFFFF8000);`  [reg-only]
        21638000  subfic r11,r3,-32768       | 21638000  subfic r11,r3,-32768
      * 7d2a5110  subfe r9,r10,r10           | 7d6b5910  subfe r11,r11,r11
      * 552307fe  rlwinm r3,r9,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_gt_w  `unsigned a; return a > 65535u;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        616bffff  ori r11,r11,65535          | 616bffff  ori r11,r11,65535
      * 7d435810  subfc r10,r3,r11           | 7d635810  subfc r11,r3,r11
      * 7d094910  subfe r8,r9,r9             | 7d6b5910  subfe r11,r11,r11
      * 550307fe  rlwinm r3,r8,0,31,31       | 556307fe  rlwinm r3,r11,0,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_ge_0  `int a; return a >= 0;`  [IDENTICAL]
        546b0ffe  rlwinm r11,r3,1,31,31      | 546b0ffe  rlwinm r11,r3,1,31,31
        69630001  xori r3,r11,1              | 69630001  xori r3,r11,1
        4e800020  blr                        | 4e800020  blr
    
    ### s_ge_1  `int a; return a >= 1;`  [reg-only]
        39600001  li r11,1                   | 39600001  li r11,1
        7c6afe70  srawi r10,r3,31            | 7c6afe70  srawi r10,r3,31
        55690ffe  rlwinm r9,r11,1,31,31      | 55690ffe  rlwinm r9,r11,1,31,31
      * 7d0b1810  subfc r8,r11,r3            | 7d6b1810  subfc r11,r11,r3
        7c695114  adde r3,r9,r10             | 7c695114  adde r3,r9,r10
        4e800020  blr                        | 4e800020  blr
    
    ### s_ge_m1  `int a; return a >= -1;`  [reg-only]
        3960ffff  li r11,-1                  | 3960ffff  li r11,-1
        7c6afe70  srawi r10,r3,31            | 7c6afe70  srawi r10,r3,31
        55690ffe  rlwinm r9,r11,1,31,31      | 55690ffe  rlwinm r9,r11,1,31,31
      * 7d0b1810  subfc r8,r11,r3            | 7d6b1810  subfc r11,r11,r3
        7c695114  adde r3,r9,r10             | 7c695114  adde r3,r9,r10
        4e800020  blr                        | 4e800020  blr
    
    ### s_ge_5  `int a; return a >= 5;`  [reg-only]
        39600005  li r11,5                   | 39600005  li r11,5
        7c6afe70  srawi r10,r3,31            | 7c6afe70  srawi r10,r3,31
        55690ffe  rlwinm r9,r11,1,31,31      | 55690ffe  rlwinm r9,r11,1,31,31
      * 7d0b1810  subfc r8,r11,r3            | 7d6b1810  subfc r11,r11,r3
        7c695114  adde r3,r9,r10             | 7c695114  adde r3,r9,r10
        4e800020  blr                        | 4e800020  blr
    
    ### s_ge_m5  `int a; return a >= -5;`  [reg-only]
        3960fffb  li r11,-5                  | 3960fffb  li r11,-5
        7c6afe70  srawi r10,r3,31            | 7c6afe70  srawi r10,r3,31
        55690ffe  rlwinm r9,r11,1,31,31      | 55690ffe  rlwinm r9,r11,1,31,31
      * 7d0b1810  subfc r8,r11,r3            | 7d6b1810  subfc r11,r11,r3
        7c695114  adde r3,r9,r10             | 7c695114  adde r3,r9,r10
        4e800020  blr                        | 4e800020  blr
    
    ### s_ge_2  `int a; return a >= 2;`  [reg-only]
        39600002  li r11,2                   | 39600002  li r11,2
        7c6afe70  srawi r10,r3,31            | 7c6afe70  srawi r10,r3,31
        55690ffe  rlwinm r9,r11,1,31,31      | 55690ffe  rlwinm r9,r11,1,31,31
      * 7d0b1810  subfc r8,r11,r3            | 7d6b1810  subfc r11,r11,r3
        7c695114  adde r3,r9,r10             | 7c695114  adde r3,r9,r10
        4e800020  blr                        | 4e800020  blr
    
    ### s_ge_hi  `int a; return a >= 32767;`  [reg-only]
        39607fff  li r11,32767               | 39607fff  li r11,32767
        7c6afe70  srawi r10,r3,31            | 7c6afe70  srawi r10,r3,31
        55690ffe  rlwinm r9,r11,1,31,31      | 55690ffe  rlwinm r9,r11,1,31,31
      * 7d0b1810  subfc r8,r11,r3            | 7d6b1810  subfc r11,r11,r3
        7c695114  adde r3,r9,r10             | 7c695114  adde r3,r9,r10
        4e800020  blr                        | 4e800020  blr
    
    ### s_ge_lo  `int a; return a >= -32768;`  [reg-only]
        39608000  li r11,-32768              | 39608000  li r11,-32768
        7c6afe70  srawi r10,r3,31            | 7c6afe70  srawi r10,r3,31
        55690ffe  rlwinm r9,r11,1,31,31      | 55690ffe  rlwinm r9,r11,1,31,31
      * 7d0b1810  subfc r8,r11,r3            | 7d6b1810  subfc r11,r11,r3
        7c695114  adde r3,r9,r10             | 7c695114  adde r3,r9,r10
        4e800020  blr                        | 4e800020  blr
    
    ### s_ge_w  `int a; return a >= 65535;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        7c6afe70  srawi r10,r3,31            | 7c6afe70  srawi r10,r3,31
        616bffff  ori r11,r11,65535          | 616bffff  ori r11,r11,65535
        55690ffe  rlwinm r9,r11,1,31,31      | 55690ffe  rlwinm r9,r11,1,31,31
      * 7d0b1810  subfc r8,r11,r3            | 7d6b1810  subfc r11,r11,r3
        7c695114  adde r3,r9,r10             | 7c695114  adde r3,r9,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_ge_0  `unsigned a; return a >= 0u;`  [IDENTICAL]
        38600001  li r3,1                    | 38600001  li r3,1
        4e800020  blr                        | 4e800020  blr
    
    ### u_ge_1  `unsigned a; return a >= 1u;`  [IDENTICAL]
        39600001  li r11,1                   | 39600001  li r11,1
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        7d6b1810  subfc r11,r11,r3           | 7d6b1810  subfc r11,r11,r3
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_ge_max  `unsigned a; return a >= 4294967295u (0xFFFFFFFF);`  [IDENTICAL]
        3960ffff  li r11,-1                  | 3960ffff  li r11,-1
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        7d6b1810  subfc r11,r11,r3           | 7d6b1810  subfc r11,r11,r3
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_ge_5  `unsigned a; return a >= 5u;`  [IDENTICAL]
        39600005  li r11,5                   | 39600005  li r11,5
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        7d6b1810  subfc r11,r11,r3           | 7d6b1810  subfc r11,r11,r3
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_ge_m5  `unsigned a; return a >= 4294967291u (0xFFFFFFFB);`  [IDENTICAL]
        3960fffb  li r11,-5                  | 3960fffb  li r11,-5
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        7d6b1810  subfc r11,r11,r3           | 7d6b1810  subfc r11,r11,r3
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_ge_2  `unsigned a; return a >= 2u;`  [IDENTICAL]
        39600002  li r11,2                   | 39600002  li r11,2
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        7d6b1810  subfc r11,r11,r3           | 7d6b1810  subfc r11,r11,r3
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_ge_hi  `unsigned a; return a >= 32767u;`  [IDENTICAL]
        39607fff  li r11,32767               | 39607fff  li r11,32767
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        7d6b1810  subfc r11,r11,r3           | 7d6b1810  subfc r11,r11,r3
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_ge_lo  `unsigned a; return a >= 4294934528u (0xFFFF8000);`  [IDENTICAL]
        39608000  li r11,-32768              | 39608000  li r11,-32768
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
        7d6b1810  subfc r11,r11,r3           | 7d6b1810  subfc r11,r11,r3
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### u_ge_w  `unsigned a; return a >= 65535u;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        3940ffff  li r10,-1                  | 3940ffff  li r10,-1
      * 6169ffff  ori r9,r11,65535           | 616bffff  ori r11,r11,65535
      * 7d691810  subfc r11,r9,r3            | 7d6b1810  subfc r11,r11,r3
        7c6a0190  subfze r3,r10              | 7c6a0190  subfze r3,r10
        4e800020  blr                        | 4e800020  blr
    
    ### s_eq_0  `int a; return a == 0;`  [IDENTICAL]
        7c6b0034  cntlzw r11,r3              | 7c6b0034  cntlzw r11,r3
        5563dffe  rlwinm r3,r11,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_eq_1  `int a; return a == 1;`  [reg-only]
        3963ffff  addi r11,r3,-1             | 3963ffff  addi r11,r3,-1
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_eq_m1  `int a; return a == -1;`  [reg-only]
        39630001  addi r11,r3,1              | 39630001  addi r11,r3,1
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_eq_5  `int a; return a == 5;`  [reg-only]
        3963fffb  addi r11,r3,-5             | 3963fffb  addi r11,r3,-5
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_eq_m5  `int a; return a == -5;`  [reg-only]
        39630005  addi r11,r3,5              | 39630005  addi r11,r3,5
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_eq_2  `int a; return a == 2;`  [reg-only]
        3963fffe  addi r11,r3,-2             | 3963fffe  addi r11,r3,-2
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_eq_hi  `int a; return a == 32767;`  [reg-only]
        39638001  addi r11,r3,-32767         | 39638001  addi r11,r3,-32767
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_eq_lo  `int a; return a == -32768;`  [reg-only]
        39608000  li r11,-32768              | 39608000  li r11,-32768
      * 7d4b1850  subf r10,r11,r3            | 7d6b1850  subf r11,r11,r3
      * 7d490034  cntlzw r9,r10              | 7d6b0034  cntlzw r11,r11
      * 5523dffe  rlwinm r3,r9,27,31,31      | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_eq_w  `int a; return a == 65535;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        616bffff  ori r11,r11,65535          | 616bffff  ori r11,r11,65535
      * 7d435850  subf r10,r3,r11            | 7d635850  subf r11,r3,r11
      * 7d490034  cntlzw r9,r10              | 7d6b0034  cntlzw r11,r11
      * 5523dffe  rlwinm r3,r9,27,31,31      | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_eq_0  `unsigned a; return a == 0u;`  [IDENTICAL]
        7c6b0034  cntlzw r11,r3              | 7c6b0034  cntlzw r11,r3
        5563dffe  rlwinm r3,r11,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_eq_1  `unsigned a; return a == 1u;`  [reg-only]
        3963ffff  addi r11,r3,-1             | 3963ffff  addi r11,r3,-1
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_eq_max  `unsigned a; return a == 4294967295u (0xFFFFFFFF);`  [reg-only]
        3960ffff  li r11,-1                  | 3960ffff  li r11,-1
      * 7d435850  subf r10,r3,r11            | 7d635850  subf r11,r3,r11
      * 7d490034  cntlzw r9,r10              | 7d6b0034  cntlzw r11,r11
      * 5523dffe  rlwinm r3,r9,27,31,31      | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_eq_5  `unsigned a; return a == 5u;`  [reg-only]
        3963fffb  addi r11,r3,-5             | 3963fffb  addi r11,r3,-5
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_eq_m5  `unsigned a; return a == 4294967291u (0xFFFFFFFB);`  [reg-only]
        3960fffb  li r11,-5                  | 3960fffb  li r11,-5
      * 7d435850  subf r10,r3,r11            | 7d635850  subf r11,r3,r11
      * 7d490034  cntlzw r9,r10              | 7d6b0034  cntlzw r11,r11
      * 5523dffe  rlwinm r3,r9,27,31,31      | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_eq_2  `unsigned a; return a == 2u;`  [reg-only]
        3963fffe  addi r11,r3,-2             | 3963fffe  addi r11,r3,-2
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_eq_hi  `unsigned a; return a == 32767u;`  [reg-only]
        39638001  addi r11,r3,-32767         | 39638001  addi r11,r3,-32767
      * 7d6a0034  cntlzw r10,r11             | 7d6b0034  cntlzw r11,r11
      * 5543dffe  rlwinm r3,r10,27,31,31     | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_eq_lo  `unsigned a; return a == 4294934528u (0xFFFF8000);`  [reg-only]
        39608000  li r11,-32768              | 39608000  li r11,-32768
      * 7d435850  subf r10,r3,r11            | 7d635850  subf r11,r3,r11
      * 7d490034  cntlzw r9,r10              | 7d6b0034  cntlzw r11,r11
      * 5523dffe  rlwinm r3,r9,27,31,31      | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### u_eq_w  `unsigned a; return a == 65535u;`  [reg-only]
      * 3d43ffff  addis r10,r3,-1            | 3d63ffff  addis r11,r3,-1
      * 394a0001  addi r10,r10,1             | 396b0001  addi r11,r11,1
      * 7d490034  cntlzw r9,r10              | 7d6b0034  cntlzw r11,r11
      * 5523dffe  rlwinm r3,r9,27,31,31      | 5563dffe  rlwinm r3,r11,27,31,31
        4e800020  blr                        | 4e800020  blr
    
    ### s_ne_0  `int a; return a != 0;`  [IDENTICAL]
        3163ffff  addic r11,r3,-1            | 3163ffff  addic r11,r3,-1
        7c6b1910  subfe r3,r11,r3            | 7c6b1910  subfe r3,r11,r3
        4e800020  blr                        | 4e800020  blr
    
    ### s_ne_1  `int a; return a != 1;`  [IDENTICAL]
        3963ffff  addi r11,r3,-1             | 3963ffff  addi r11,r3,-1
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### s_ne_m1  `int a; return a != -1;`  [IDENTICAL]
        39630001  addi r11,r3,1              | 39630001  addi r11,r3,1
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### s_ne_5  `int a; return a != 5;`  [IDENTICAL]
        3963fffb  addi r11,r3,-5             | 3963fffb  addi r11,r3,-5
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### s_ne_m5  `int a; return a != -5;`  [IDENTICAL]
        39630005  addi r11,r3,5              | 39630005  addi r11,r3,5
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### s_ne_2  `int a; return a != 2;`  [IDENTICAL]
        3963fffe  addi r11,r3,-2             | 3963fffe  addi r11,r3,-2
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### s_ne_hi  `int a; return a != 32767;`  [IDENTICAL]
        39638001  addi r11,r3,-32767         | 39638001  addi r11,r3,-32767
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### s_ne_lo  `int a; return a != -32768;`  [reg-only]
        39608000  li r11,-32768              | 39608000  li r11,-32768
      * 7d4b1850  subf r10,r11,r3            | 7d6b1850  subf r11,r11,r3
      * 312affff  addic r9,r10,-1            | 314bffff  addic r10,r11,-1
      * 7c695110  subfe r3,r9,r10            | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### s_ne_w  `int a; return a != 65535;`  [reg-only]
        3d600000  lis r11,0                  | 3d600000  lis r11,0
        616bffff  ori r11,r11,65535          | 616bffff  ori r11,r11,65535
      * 7d435850  subf r10,r3,r11            | 7d635850  subf r11,r3,r11
      * 312affff  addic r9,r10,-1            | 314bffff  addic r10,r11,-1
      * 7c695110  subfe r3,r9,r10            | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### u_ne_0  `unsigned a; return a != 0u;`  [IDENTICAL]
        3163ffff  addic r11,r3,-1            | 3163ffff  addic r11,r3,-1
        7c6b1910  subfe r3,r11,r3            | 7c6b1910  subfe r3,r11,r3
        4e800020  blr                        | 4e800020  blr
    
    ### u_ne_1  `unsigned a; return a != 1u;`  [IDENTICAL]
        3963ffff  addi r11,r3,-1             | 3963ffff  addi r11,r3,-1
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### u_ne_max  `unsigned a; return a != 4294967295u (0xFFFFFFFF);`  [reg-only]
        3960ffff  li r11,-1                  | 3960ffff  li r11,-1
      * 7d435850  subf r10,r3,r11            | 7d635850  subf r11,r3,r11
      * 312affff  addic r9,r10,-1            | 314bffff  addic r10,r11,-1
      * 7c695110  subfe r3,r9,r10            | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### u_ne_5  `unsigned a; return a != 5u;`  [IDENTICAL]
        3963fffb  addi r11,r3,-5             | 3963fffb  addi r11,r3,-5
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### u_ne_m5  `unsigned a; return a != 4294967291u (0xFFFFFFFB);`  [reg-only]
        3960fffb  li r11,-5                  | 3960fffb  li r11,-5
      * 7d435850  subf r10,r3,r11            | 7d635850  subf r11,r3,r11
      * 312affff  addic r9,r10,-1            | 314bffff  addic r10,r11,-1
      * 7c695110  subfe r3,r9,r10            | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### u_ne_2  `unsigned a; return a != 2u;`  [IDENTICAL]
        3963fffe  addi r11,r3,-2             | 3963fffe  addi r11,r3,-2
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### u_ne_hi  `unsigned a; return a != 32767u;`  [IDENTICAL]
        39638001  addi r11,r3,-32767         | 39638001  addi r11,r3,-32767
        314bffff  addic r10,r11,-1           | 314bffff  addic r10,r11,-1
        7c6a5910  subfe r3,r10,r11           | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### u_ne_lo  `unsigned a; return a != 4294934528u (0xFFFF8000);`  [reg-only]
        39608000  li r11,-32768              | 39608000  li r11,-32768
      * 7d435850  subf r10,r3,r11            | 7d635850  subf r11,r3,r11
      * 312affff  addic r9,r10,-1            | 314bffff  addic r10,r11,-1
      * 7c695110  subfe r3,r9,r10            | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
    ### u_ne_w  `unsigned a; return a != 65535u;`  [reg-only]
      * 3d43ffff  addis r10,r3,-1            | 3d63ffff  addis r11,r3,-1
      * 394a0001  addi r10,r10,1             | 396b0001  addi r11,r11,1
      * 312affff  addic r9,r10,-1            | 314bffff  addic r10,r11,-1
      * 7c695110  subfe r3,r9,r10            | 7c6a5910  subfe r3,r10,r11
        4e800020  blr                        | 4e800020  blr
    
