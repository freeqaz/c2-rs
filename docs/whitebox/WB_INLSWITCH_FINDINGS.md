# `WB_INLSWITCH` — c2's own inliner decision surface, read

**Lane `w-inlswitch`, 2026-08-28.** Decision 22, `ADOPTION_BRIEF_2026-08-28.md`
§L4. Board **#3768**–**#3773**. Characterization lane, `docs/rungs/README.md`
kind 3.

Prereg: [`../../work/w-inlswitch/PREREG.md`](../../work/w-inlswitch/PREREG.md),
committed at `5eec1a7d5` **before the image was opened**.
**Predicted reach 0, delivered 0** — zero `crates/` bytes, no `DISCLOSURE.md`
row, no `scripts/gate.sh` row (`#3691`), no clause row added, removed,
renumbered or restated.

Evidence tiers, per `P_INLINE.md`'s convention: **`[R]`** read from
disassembly · **`[O]`** confirmed against a real obj or a real toolchain
invocation with the witness named · **`[I]`** interpretive.

Instrument: [`scripts/dump_inlswitch.py`](scripts/dump_inlswitch.py).
Output: [`../../work/w-inlswitch/inlswitch.out`](../../work/w-inlswitch/inlswitch.out).
Toolchain measurement: [`../../work/w-inlswitch/cl_argv_modes.out`](../../work/w-inlswitch/cl_argv_modes.out).

---

## 0. The headline, and the four things it refutes

> **The 24 `-inl*` switches are the command-line overrides for 24 named fields
> of the very parameter tables `P_INLINE` §5 declines to quote — and the
> tables' DEFAULTS are recoverable from the image after all, by exactly the
> method §5 itself prescribes for the descriptor table.** `[R]`
>
> **`DAT_10c3de20` is c2's EFFECTIVE POGO mode**, and the switch that sets it
> to `2` is **`-pgo#`** (alias `-po#`) or **`-pgu#`**, via `DAT_10c6f1c8`.
> It narrates nothing. `[R]`
>
> **`FUN_10b5da2f` is a budgeted statement-cost test** whose budget is
> `k · (n + 2 + …)`, and its "second read of `k`" is a **loop reload after a
> `neg`**, not a second parameter use. `[R]`
>
> **`k`'s run-time value is now settled at `3`** for every compilation this
> project runs — read (only the numeric-option setter writes it, only on a
> name match) plus measured (no mode passes `-vol`). `#3734`'s open question
> is closed. `[R]` + `[O]`

Refuted, in the order the numbers appear:

| what | said | is |
|---|---|---|
| `#3718` / brief §L4 | **21** `-inl*#` switches | **24** switches over 24 contiguous dwords (23 numeric + 1 boolean `-inlnlw`) |
| `w-lowerband` §7 / brief §L4 | `DAT_10c3de20` has **10 writers** | **19 write instructions in 13 owner functions**, agreed to the address by two independent instruments |
| `w-lowerband` §7 | naming the switch *"would make c2 narrate its own inline decisions"* | it would put c2 in **POGO optimize/update**, which **changes** the decisions and reports nothing |
| this lane's own **P1** | ≥ 12 of 24 carry an initializing store | **0 of 24 do.** The operative default is installed at the *destination field*, not at the switch word |
| this lane's own **P2** | ≤ 11 of 24 have a reader | **24 of 24 have one.** Not one is vestigial |
| this lane's own **P4** | `FUN_10b5da2f` is off the inline path | **it is on it** — sole caller `0x10b5eb27`, inside the inliner band |

Two of five prereg predictions held (**P3(b)**, **P5**); three were refuted by
this lane's own measurements (**P1**, **P2**, **P4**), and **P3(a)** was
refuted in the direction that produced the lane's best result — the switch is
nameable, and it is named.

---

## 1. The count, re-derived rather than relayed `[R]`

```
$ python3 work/w-inlfit/optmap.py > work/w-inlswitch/optmap_rerun.out
$ diff work/w-inlfit/optmap.out work/w-inlswitch/optmap_rerun.out && echo IDENTICAL
IDENTICAL
$ grep -c '  -inl' work/w-inlswitch/optmap_rerun.out
24
$ grep '  -inl' work/w-inlswitch/optmap_rerun.out | awk '{print $3}' | sort -u | wc -l
24
```

24 rows, **24 distinct value words**, spanning `0x10c45db4`–`0x10c45e10`.
`(0x10c45e10 − 0x10c45db4)/4 + 1 = 24`: the block is **contiguous with no
gap**, so there is no screen under which the answer is 21. 23 carry the
numeric suffix `#` (kind `0x2401`); **`-inlnlw` at `0x10c45db8` is a boolean**
(kind `0x0101`) and is the one the `*#` spelling in `#3718` and the brief
excludes without saying so.

`optmap.py` is `w-inlfit`'s instrument, re-run unmodified in this tree. The
recovery is unchanged: 484 stores over `0x10c3cc10`–`0x10c6770c`, record phase
anchored on the found `-EHs`/`-EHa` pair at `0x10c46ddc`.

> **Denominator note (`#3470`/`#1002`):** 24 of **the 46-field parameter
> record**, not 24 of some open set. The record's size is `0x2e` dwords, read
> off the `rep movsd` count at `0x10b5e516`, not assumed.

---

## 2. The shape nobody had: a SCATTER into the two POGO tables `[R]`

The 24 words are never read by the inliner. **`FUN_10b5b88f`** (`0x10b5b88f`,
335 B) is a straight-line scatter of **37** value words into `[ecx+0x00 …
0xb4]` — 37 of the record's 46 fields — from the **contiguous** source block
`0x10c45d80`–`0x10c45e10`:

```
10b5b88f:  a1 10 5e c4 10    mov  eax,ds:0x10c45e10      ; -inlS#
10b5b894:  89 01             mov  DWORD PTR [ecx],eax    ;   -> +0x00
10b5b896:  a1 0c 5e c4 10    mov  eax,ds:0x10c45e0c      ; -inlT#
10b5b89b:  89 41 04          mov  DWORD PTR [ecx+0x4],eax;   -> +0x04
…
10b5b9d2:  a1 80 5d c4 10    mov  eax,ds:0x10c45d80
10b5b9d7:  89 81 b4 00 00 00 mov  DWORD PTR [ecx+0xb4],eax
10b5b9dd:  c3                ret
```

It has **exactly two callers**, and each passes one of `P_INLINE` §5's two
"unquotable" tables:

| caller | `ecx` | what it is |
|---|---|---|
| `FUN_10b5ba71` `0x10b5ba78` | `0x10c45ed0` | §5's **second** POGO parameter table — call it **B** |
| `FUN_10b5bc6e` `0x10b5bc73` | `0x10c45e18` | §5's **first** POGO parameter table — call it **A** |

Each caller then runs a **zero-guarded default sweep** over its own table:

```
10b5bc7a:  39 05 18 5e c4 10       cmp  DWORD PTR ds:0x10c45e18,eax   ; eax == 0
10b5bc80:  75 0a                   jne  0x10b5bc8c
10b5bc82:  c7 05 18 5e c4 10 3c…   mov  DWORD PTR ds:0x10c45e18,0x3c  ; default 60
```

**33 such stores in each caller, all 33 zero-guarded in both** — measured, not
eyeballed (`inlswitch.out`). 46 − 33 = the 13 fields that get no default at
all, and they are exactly the 13 that no switch names and nothing reads (§4).

### 2.1 This amends `P_INLINE` §5 `[R]`

§5 says of `DAT_10c45e18`/`DAT_10c45ed0`: *"they are zero at load and written
at run time — none of their values is quotable from the image and this page
does not quote them."* The first clause is exactly right and the second is
**too strong**. The tables' contents are not in the image *as data*; the code
that installs them is, and §5's own page already sanctions recovering a BSS
table from the run of stores that builds it — that is what `optmap.py` does for
the descriptor table. **Both 33-value default sets are recoverable, and §3
prints them.** What remains unquotable is only a *run-time* table under a
command line that overrides fields.

---

## 3. The 24 switches, with defaults and readers `[R]`

`defA` = the value installed by `FUN_10b5bc6e` into table **A** when the switch
is absent; `defB` = the same for table **B** via `FUN_10b5ba71`. `live` =
`0x10c3f510 + off`, the record the readers actually address.

| off | switch | value word | defA | defB | live | readers |
|---|---|---|---:|---:|---|---|
| `+0x00` | `-inlS#` | `0x10c45e10` | **60** | **2** | `0x10c3f510` | `0x10b5fd29` `0x10b5fe6b` `0x10b600d6` |
| `+0x04` | `-inlT#` | `0x10c45e0c` | **104** | **20** | `0x10c3f514` | `0x10b5ff90` |
| `+0x08` | `-inlfcsw#` | `0x10c45e08` | **32** | **5** | `0x10c3f518` | `0x10b5fde8` `0x10bb7cdb` |
| `+0x0c` | `-inlflcsw#` | `0x10c45e04` | 0 | 0 | `0x10c3f51c` | `0x10b5fddc` `0x10ba274b` `0x10bb7ccf` |
| `+0x10` | `-inlld#` | `0x10c45dd0` | 3 | 3 | `0x10c3f520` | `0x10b5ff5f` |
| `+0x14` | `-inld#` | `0x10c45dcc` | 7 | 7 | `0x10c3f524` | `0x10b5ff7e` |
| `+0x18` | `-inlniln#` | `0x10c45dc8` | 1 | 1 | `0x10c3f528` | `0x10b5ff10` |
| `+0x1c` | `-inlnild#` | `0x10c45dc4` | 2 | 2 | `0x10c3f52c` | `0x10b5ff19` |
| `+0x20` | `-inlnoln#` | `0x10c45dc0` | 2 | 2 | `0x10c3f530` | `0x10b5ff2e` |
| `+0x24` | `-inlnold#` | `0x10c45dbc` | 3 | 3 | `0x10c3f534` | `0x10b5ff37` |
| `+0x28` | `-inlnlw` **(bool)** | `0x10c45db8` | 0 | 0 | `0x10c3f538` | `0x10b5fee1` |
| `+0x2c` | `-inlocsa1#` | `0x10c45e00` | **96** | **15** | `0x10c3f53c` | `0x10b6007a` |
| `+0x30` | `-inlocsa2#` | `0x10c45dfc` | 0 | 0 | `0x10c3f540` | `0x10b6008f` |
| `+0x34` | `-inlocsa3#` | `0x10c45df8` | 0 | 0 | `0x10c3f544` | `0x10b600a1` |
| `+0x38` | `-inlocsa4#` | `0x10c45df4` | **96** | **15** | `0x10c3f548` | `0x10b600b3` |
| `+0x3c` | `-inlcrmax#` | `0x10c45db4` | 10 | 10 | `0x10c3f54c` | `0x10b600bb` |
| `+0x40` | `-inlfcsa#` | `0x10c45df0` | 5 | 5 | `0x10c3f550` | `0x10b600c8` |
| `+0x48` | `-inluserinl#` | `0x10c45dec` | **8** | **2** | `0x10c3f558` | `0x10b6003d` |
| `+0x4c` | `-inlnobr#` | `0x10c45dd4` | **48** | **3** | `0x10c3f55c` | `0x10b5fd89` |
| `+0x5c` | `-inlmlsa#` | `0x10c45de8` | **32** | **15** | `0x10c3f56c` | `0x10b5dca9` `0x10b5dcd6` |
| `+0x60` | `-inlcsw#` | `0x10c45de4` | 0 | 0 | `0x10c3f570` | `0x10b5fdba` `0x10ba2772` `0x10ba2bfe` `0x10bb7cee` |
| `+0x64` | `-inldasw#` | `0x10c45ddc` | 0 | 0 | `0x10c3f574` | `0x10b5fdc4` `0x10ba2765` `0x10ba2bef` `0x10bb7ce2` |
| `+0x68` | `-inlcasw#` | `0x10c45de0` | 0 | 0 | `0x10c3f578` | `0x10b5fdd0` `0x10ba2758` `0x10ba2be1` `0x10bb7cf9` |
| `+0x6c` | `-inlipfw#` | `0x10c45dd8` | 0 | 0 | `0x10c3f57c` | `0x10b5fe15` |

**24 named, 24 with at least one live reader — 24 of 24, denominator 24.**
**P2 is refuted twice over**: not only is it not "at most 11", it is *all of
them*, and the sub-prediction ("at most 6 tied to a named decision") is
refuted too — **§3.1 names the decision for all 24**, at the level of the
arithmetic each value enters. For five of them (`-inlfcsw#`, `-inlflcsw#`,
`-inlcsw#`, `-inldasw#`, `-inlcasw#`) there are *further* readers outside the
inliner band that were not opened (§9 item 3); the in-band decision is named
for every one.

**Table A vs table B are not two tunings of one model; nine fields differ and
every difference is in the same direction** — B's threshold and bonus values
are 3×–13× smaller. `[I]` The natural gloss is that B is the profile-guided
table, where a measured call count carries the weight the static heuristics
carry in A; this page does not claim it.

### 3.1 What each reader decides `[R]`

29 of the 32 distinct reader instructions live in **`FUN_10b5fcd8`**, which is
`P_INLINE` §5's POGO cost model. It accumulates a score in `esi` and returns
`esi < -inlS#`. Reading the arithmetic, not naming it:

* **`-inlS#` is the ACCEPT THRESHOLD.** `0x10b600d6`: `cmp esi,ds:0x10c3f510` /
  `setl al` — the model returns 1 (inline) exactly when the accumulated score
  is **below** it. Also an early bail at `0x10b5fd29` (`cmp eax,S; jle`) and a
  mid-model `S + 5` test at `0x10b5fe6b`.
* **`-inlcsw#`/`-inldasw#`/`-inlcasw#`/`-inlflcsw#`/`-inlfcsw#` are five LINEAR
  WEIGHTS** over five fields of one record, summed and subtracted from the
  score in one straight run at `0x10b5fdba`–`0x10b5fdf1`:
  `esi -= [x+0x1c]·csw + [x+0x18]·dasw + [x+0x14]·casw + [x+0x10]·flcsw + [x+0x04]·fcsw`.
  All five default to **0 in both tables**, so the whole term vanishes unless a
  switch turns it on — a dormant cost model inside the cost model.
* **`-inlipfw#`** (`0x10b5fe15`) is a sixth weight in the same shape, paired
  with the unnamed field `+0x7c`.
* **`-inlniln#`/`-inlnild#`** (`0x10b5ff10`/`0x10b5ff19`) are a **rational
  scale `n/d`** applied to the score — `eax = niln·esi; eax /= nild` — selected
  by a magnitude band on a 64-bit count (`0x2a05f200` = 705,032,704).
  **`-inlnoln#`/`-inlnold#`** are the same pair for the next band
  (`0x1dcd6500` = 500,000,000). Defaults `1/2` and `2/3` in both tables.
* **`-inlnlw` gates that whole banded block.** `0x10b5fee1`:
  `cmp ds:0x10c3f538,ecx` (`ecx = 0`) / `je` past it. **Default 0 in both
  tables, so the four `niln/nild/noln/nold` knobs are dead unless `-inlnlw` is
  given** — four numeric switches behind one boolean.
* **`-inlld#`/`-inld#`** (`0x10b5ff5f`/`0x10b5ff7e`) are the denominator of a
  **blend**: `c = ld or d; result = (raw + (c−1)·esi)/c`. Defaults 3 and 7.
* **`-inlT#`** (`0x10b5ff90`) is the threshold the blended ratio is compared
  against: `cmp eax,esi; jl` — below it, the model takes the reject path.
* **`-inlocsa1#`…`-inlocsa4#`** (`0x10b6007a`, `0x10b6008f`, `0x10b600a1`,
  `0x10b600b3`) are a **cascade of flat score credits by call-count band** —
  `esi -= ocsaN` at `0x17d7840` (25 M), `0x2faf080` (50 M), `0x5f5e100`
  (100 M). Note **`ocsa2` and `ocsa3` default to 0 while `ocsa1` and `ocsa4`
  do not** (96/96 in A, 15/15 in B): the cascade's middle two bands are
  no-ops by default.
* **`-inlcrmax#`** (`0x10b600bb`) **caps a repeat count**: `cmp ecx,crmax; jge`
  skips the next credit entirely, and `cmp ecx,1; jle` skips it from below.
  Default 10 in both tables.
* **`-inlfcsa#`** (`0x10b600c8`) is the credit `crmax` gates:
  `esi -= (fcsa + esi)/ecx`.
* **`-inluserinl#`** (`0x10b6003d`) is a **flat credit when bit 7 of the
  caller-supplied byte `[ebp+8]` is set** — the `inline`-keyword arm. A **8**,
  B **2**.
* **`-inlnobr#`** (`0x10b5fd89`) is a flat credit when **bit 7 of
  `[sym+0xb1]`** is set. A **48**, B **3** — the largest single credit in
  table A.
* **`-inlmlsa#`** is the only one read **outside** the cost model:
  `FUN_10b5dc6c` at `0x10b5dca9` compares a byte counter against it and bails
  when it exceeds — a **depth/multiplicity cap**, A **32**, B **15**.

Four switches (`-inlfcsw#`, `-inlflcsw#`, `-inlcsw#`, `-inldasw#`,
`-inlcasw#` — five, in fact) additionally have readers in `FUN_10bb7aa3`,
`FUN_10ba24c4` and `FUN_10ba2948`, which are outside the inliner band and were
not read. **Named as not reached.**

### 3.2 …and every one of them is DEAD on this workload `[O]`

The parameter fill is gated twice, and the second gate is the whole story:

```
10b5e4f7:  83 3d c4 62 c4 10 00   cmp  DWORD PTR ds:0x10c462c4,0x0
10b5e4fe:  74 2e                  je   0x10b5e52e            ; skip the fill entirely
…
10b5e50f:  83 3d c8 f1 c6 10 00   cmp  DWORD PTR ds:0x10c6f1c8,0x0
10b5e518:  bf 10 f5 c3 10         mov  edi,0x10c3f510
10b5e51e:  be d0 5e c4 10         mov  esi,0x10c45ed0        ; table B
10b5e523:  75 05                  jne  0x10b5e52a
10b5e525:  be 18 5e c4 10         mov  esi,0x10c45e18        ; table A
10b5e52a:  f3 a5                  rep movs DWORD PTR es:[edi],DWORD PTR ds:[esi]
```

`DAT_10c6f1c8` is the **requested POGO mode** (§5 of this page). It is `0` on
every compilation this project runs (§5.2), so **table A is the live one** and
its defaults are the operative ones. But `FUN_10b5fcd8`, which holds 29 of the
32 reads, is entered only from `0x10b60a50` under a local gate on
`[ebp-0x4] != 0` — `P_INLINE` §5's *"reaches the model only when the callee
has a profile record"* — and `FUN_10b5dc6c`'s caller `FUN_10b60727` opens with
`cmp ds:0x10c3de20,0x1` at `0x10b60730` and `cmp ds:0x10c6f1c8,0x1` at
`0x10b60767`.

> **So all 24 switches are READ AND DEAD here**, in `C2_MAP_METHOD.md` §7 case
> 1's exact sense. They are the compiler's own named decision surface, they are
> now enumerated with their defaults, and **not one of them can move a byte of
> this project's obj.** That is the finding, and it is also the fence: nothing
> here licenses an emit, and nothing here is a candidate for adoption.

---

## 4. The 22 fields that are NOT `-inl*`, and the 13 that are dead `[R]`

| group | count | what |
|---|---:|---|
| fed by a switch | **24** | §3 |
| fed from a value word with **no descriptor name** | **13** | `+0x84`…`+0xb4` from `0x10c45d80`–`0x10c45db0`. **Zero readers each** at `0x10c3f594`–`0x10c3f5c4`, and **no default store in either sweep**. Copied and never used |
| **not fed at all**, but defaulted and read | **9** | `+0x44` `+0x50` `+0x54` `+0x58` `+0x70` `+0x74` `+0x78` `+0x7c` `+0x80` |

The nine unfed-but-live fields are the model's non-settable constants — e.g.
`+0x44` (A **−4**, B **−1**) is *added* to the score at `0x10b60049` when
`[ebx+0x1c] & 2`, the only negative default in either table, i.e. the model's
only **penalty**. `+0x70`/`+0x74` are read only by `FUN_10b5f3f4`.

**37 + 9 = 46.** The record is fully accounted for.

---

## 5. `DAT_10c3de20` — the writer walk, and the claim it kills `[R]`

`w-lowerband` §7 filed it as *"389 refs, 10 writers, three values"* and
proposed that *"naming the switch that sets it to `2` would make c2 narrate
its own inline decisions."*

### 5.1 The writer set, by two instruments

```
$ python3 docs/whitebox/scripts/dump_inlswitch.py --modes
DAT_10c3de20 (effective mode): 389 references, 19 WRITE instructions in 13 distinct owner functions
$ awk -F'\t' 'tolower($2)=="10c3de20"{print $3}' xrefs.tsv | sort | uniq -c
    371 READ      6 READ_WRITE     13 WRITE
```

**19, not 10** — and the two instruments agree to the address: Ghidra's
`WRITE` (13) plus `READ_WRITE` (6, the `and ds:…,0x0` clears) is the same
19-instruction, 13-function set the listing yields. Ghidra's bare `WRITE`
count is 13; **neither instrument produces 10 under any screen.**

> **The two instruments disagree at exactly one address out of 390, and it is
> reported rather than reconciled.** Ghidra has a READ at `0x10bd5d2f` that the
> objdump listing does not: objdump decodes `ff 83 3d 20 de c3` at
> `0x10bd5d2e` as `inc DWORD PTR [ebx-0x3c21dfc3]`, swallowing a
> `83 3d 20 de c3 10 00` (`cmp ds:0x10c3de20,0x0`) that would begin one byte
> later. Both decoders are lost here — objdump emits `das` / `repz (bad)`
> immediately before, and Ghidra assigns the address no owning function. **This
> is exactly the desynchronisation hazard `w-lowerband`'s `bytescan.py` exists
> for**, it is 1 in 390, it is a READ, and it touches none of this section's
> conclusions, all of which are about writers.

### 5.2 The chain, complete

The only literal `2` is **`0x10b9e2bb`**, inside `FUN_10b9e1d2`, and that
function returns immediately unless `DAT_10c6f1c8 == 2`:

```
10b9e229:  83 3d c8 f1 c6 10 02   cmp  DWORD PTR ds:0x10c6f1c8,0x2
10b9e236:  0f 85 35 01 00 00      jne  0x10b9e371        ; -> ret
…
10b9e2bb:  c7 05 20 de c3 10 02   mov  DWORD PTR ds:0x10c3de20,0x2
```

`DAT_10c6f1c8` has **5 write instructions in 2 functions**, and only three of
them are enabling — all in `FUN_10b848dc`, immediately after the option-table
walk, with `ebx = 2` and `edi = 1`:

```
10b84b3e:  a1 cc 6b c4 10   mov  eax,ds:0x10c46bcc   ; -pgo# / -po#
10b84b45:  74 08            je   0x10b84b4f
10b84b47:  89 1d c8 f1 c6 10  mov DWORD PTR ds:0x10c6f1c8,ebx   ; = 2
10b84b4f:  a1 c4 6b c4 10   mov  eax,ds:0x10c46bc4   ; -pgu#
10b84b56:  74 0e            je   0x10b84b66
10b84b58:  89 1d c8 f1 c6 10  mov DWORD PTR ds:0x10c6f1c8,ebx   ; = 2
…
10b84b77:  a1 d0 6b c4 10   mov  eax,ds:0x10c46bd0   ; -pgi# / -pi#
10b84b7e:  74 0b            je   0x10b84b8b
10b84b80:  89 3d c8 f1 c6 10  mov DWORD PTR ds:0x10c6f1c8,edi   ; = 1
```

The two value words `0x10c46bcc` and `0x10c46bd0` are the ones `optmap.py`
prints as `(reg)` because their descriptor plants them from a register.
Resolved here by tracking the register:

```
10c29c23:  ba d0 6b c4 10   mov  edx,0x10c46bd0
10c29c28:  b8 cc 6b c4 10   mov  eax,0x10c46bcc
10c29d62:  89 15 14 70 c4 10  mov DWORD PTR ds:0x10c47014,edx  ; -pgi# value ptr
10c29d7b:  a3 20 70 c4 10     mov ds:0x10c47020,eax            ; -pgo# value ptr
10c29db3:  89 15 38 70 c4 10  mov DWORD PTR ds:0x10c47038,edx  ; -pi#  value ptr
10c29dc9:  a3 44 70 c4 10     mov ds:0x10c47044,eax            ; -po#  value ptr
```

**So `-pgo#` and `-po#` are aliases for one word, and `-pgi#` and `-pi#` for
another** — a fact `optmap.py` could not see and this lane closes.

`FUN_10bae79c`'s two stores (`0x10bae7ce`, `0x10bae8db`) both write `ebx = 0`;
they only ever *disable*. **The enabling set is closed at three instructions.**

**And `DAT_10c3de20` is the EFFECTIVE mode where `DAT_10c6f1c8` is the
REQUESTED one** — `0x10b9e07d` is a bare mirror,
`mov eax,ds:0x10c6f1c8` / `mov ds:0x10c3de20,eax`, and the two neighbouring
stores in the same function reset it to `0` immediately after a diagnostic:

```
10b9e042:  push 0x10b16788   -> "ERR:\t%s was not profiled; Pogo disabled\n"
10b9e069:  mov  DWORD PTR ds:0x10c3de20,esi          ; esi == 0
10b9e1b1:  push 0x10b16724   -> "WRN:\t%s was not probed; Pogo disabled\n"
10b9e1c0:  mov  DWORD PTR ds:0x10c3de20,esi
```

(Strings read from the image at file-resolved VAs; they are ASCII, not the
UTF-16 the option names use.)

### 5.3 The verdict on the claim

**`DAT_10c3de20 ∈ {0, 1, 2}` is `{no POGO, instrument, optimize/update}`.**
`[R]`, corroborated by two diagnostics that say "Pogo disabled" on the paths
that zero it.

> **"Naming the switch would make c2 narrate its own inline decisions" is
> FALSE.** The switch is `-pgo#` / `-po#` / `-pgu#`. Setting it does not make
> c2 report anything — it puts c2 in profile-guided optimization, which
> **swaps the live inline parameter record from table A to table B**
> (`0x10b5e50f`) and turns on a cost model whose thresholds are 3×–13× tighter.
> It is a **mode selector that changes the answer**, and using it to observe
> the answer would be measuring a different compiler.
>
> This lane's **P3(b) held exactly as registered** ("a compilation-MODE
> selector, not a diagnostic; setting it to 2 would change c2's inline
> decisions, not report them"), including the registered guess at the mode axis
> — *whole-program operation*. **P3(a) is refuted**: a writer *does* trace to
> an option handler with a recoverable name, and the recovery is the deliverable.

> **The narration seam the follow-up was actually reaching for already exists
> and is already recorded** — `cl /FAsc`, `c2rs`'s listing seam. This page adds
> nothing to it and does not propose a probe.

---

## 6. `FUN_10b5da2f` — read end to end `[R]`

573 B, `0x10b5da2f`–`0x10b5dc6b`, **one caller: `0x10b5eb27`, inside
`FUN_10b5e9a5`** — which is in the inliner band (`0x10b5b86d`–`0x10b62b00`).
**P4 is refuted**: it is on the inline path, not off it.

**It is a budgeted statement-cost test. It returns 1 when the cost EXCEEDS the
budget, 0 otherwise.**

**The budget**, computed once at entry:

```
10b5da47..10b5da62   n = count of nodes in the list at [arg2+0x28] whose
                         kind byte gives (1 << [node+8]) & 6, and whose
                         [[node+0x18]+0x14] is non-null
10b5da64:  mov  ecx,DWORD PTR ds:0x10c2ea98     ; k
10b5da6a:  add  esi,0x2                          ; n + 2
10b5da6d:  imul esi,ecx                          ; budget = (n+2) * k
10b5da75:  cmp  DWORD PTR ds:0x10c2e310,edi      ; favour-speed (image value 1)
10b5da7d:  add  esi,ecx                          ;   += k when set
10b5da8d:  test eax,0x500000                     ; [DAT_10c2e2f4 + 0x94]
10b5da94:  test al,0x8
10b5da98:  lea  esi,[esi+ecx*2]                  ;   += 2k when both set
```

so **`budget = k · (n + 2 + [favour-speed] + 2·[attr])`**, `k = 3`, and with
§6.7.3's measured favour-speed image value of `1` that is `3n + 9` at minimum.

**The cost**, accumulated in `[ebp-4]` over a walk of the statement list at
`[[[arg1+8]]+0x1c]`, by node kind byte `[node+8]`:

| kind | charge | extra |
|---:|---:|---|
| `0x0d` | **+1** | plus a long structural match (`[node+4] == 0x2d4`, operand kinds 2/5/7, `[next+4] == 0x2dd`, …) ending in `call 0x10b4cc87`; on success the walk **jumps to the matched node** rather than the next |
| `0x0f` | **+2** | and, if the `0x500000|8` attribute flag was set at entry, **refunds `2k` from the budget** and clears the flag |
| `0x13` | **+2** | |
| other | 0 | |

The loop tail is `cmp [ebp-4],esi; jg` → return 1. So a *smaller* `k` makes the
test fire sooner: **`k` is a global scale on how much statement structure this
test tolerates**, and `-vol#` scales it and C8's `16 << k` ceiling together.

### 6.1 `#3734`'s "three readers" is right about instructions and wrong about uses `[R]`

The second read, `0x10b5dacb`, is the **loop-carried reload**:

```
10b5dae8:  cmp  DWORD PTR [ebp-0xc],edi
10b5daed:  f7 d9              neg  ecx          ; ecx = -k  -- CLOBBERS k
10b5daef:  8d 04 4e           lea  eax,[esi+ecx*2]
…
10b5dc59:  0f 85 69 fe ff ff  jne  0x10b5dac8
10b5dac8:  8b 5d e8           mov  ebx,DWORD PTR [ebp-0x18]
10b5dacb:  8b 0d 98 ea c2 10  mov  ecx,DWORD PTR ds:0x10c2ea98   ; reload k
```

The `0x0f` arm negates `ecx` to compute the refund, so the loop head must
restore `k` on every iteration. **`k` has three read INSTRUCTIONS and two
semantic USES**: `FUN_10b5da2f`'s budget multiplier and `FUN_10b5e4cc`'s shift.
`#3734` is correct as filed and its implication — "a general inliner scaling
knob at two independent places" — is one place, read twice.

---

## 7. `k`'s run-time value is SETTLED, and the answer is 3

### 7.1 The read: there is no initialization sweep `[R]`

`#3734` left this open because `0x10c29800` plants `k`'s *address* in the
`-vol#` descriptor, *"precisely a handle for a generic numeric-option setter to
store through."* The setter is now read, and it stores **only on a name match**:

* **`FUN_10c1f746`** walks the descriptor table from `0x10c46bd8`
  (`mov ebx,0x10c46bd8` at `0x10b84a31`), **stride 12** (`add esi,0xc` at
  `0x10c1f7a4` — the stride confirmed independently of `optmap.py`), terminating
  on `BYTE [esi+9] == 0`. It compares the record's UTF-16 name against the
  token, honouring `-`, `#` and `*` as name metacharacters. **On a match, and
  only then, it calls `FUN_10c1f572`.**
* **`FUN_10c1f572`** dispatches on `BYTE [esi+9]`: `0x01` → store 1, `0x05` →
  store 0, `0x22` → string, `0x24` → **numeric**:

```
10c1f734:  8b cb             mov  ecx,ebx
10c1f736:  e8 11 fc ff ff    call 0x10c1f34c        ; parse the digits
10c1f73b:  8b 4f 04          mov  ecx,DWORD PTR [edi+0x4]   ; the value_ptr
10c1f73e:  89 01             mov  DWORD PTR [ecx],eax
```

**That is the only store through a descriptor's value pointer for kind
`0x2401`, and it is inside the match arm.** No loop initialises every
descriptor. So `DAT_10c2ea98` retains its `.data` value `3` (file offset
`0x12dc98`) unless `-vol<n>` is on c2's argument vector.

### 7.2 The measurement: no mode passes it `[O]`

`cl /Bd` makes cl.exe print each pass's own command line. Witness:
`work/w-inlswitch/cl_argv_modes.out`, produced in this tree against
`compilers/X360/16.00.11886.00` under wibo, over **every row of
`scripts/lanes.txt`** plus `/Os` `/Ot` `/Ox /Ob0` `/Ox /Ob1` as controls.

```
cl /O1 /Oi /EHsc   c2.dll -il <IL> -typedil -Fot.obj -W 1 -Gs4096 -G604
                   -QVMX128 -QDD2 -MT -Fdvc100.pdb -f t.cpp -Bd -Og -Ob2 -Gy -EHs
cl /Ox /Gy /EHsc   … -Bd -Og -Ob2 -Gy -EHs
cl /Od             … -Bd -Ob0
```

**Zero occurrences of `-vol`, of any `-inl*`, and of `-pgi`/`-pgo`/`-pgu`/
`-pi`/`-po`/`-pv` across every mode row.** The only inline-related switch cl
ever passes is **`-Ob0`/`-Ob1`/`-Ob2`** (value word `0x10c46bc0`), which is not
in the 24 and is not in the parameter record.

The reference seam's own standalone template
(`crates/c2-reference/src/lib.rs:1709`) is the same list and likewise carries
none of them.

> **Therefore `k = 3` at run time, on every compilation this project runs, and
> `DAT_10c46318 = 0x10 << 3 = 128` follows.** `#3734`'s open question is
> **closed**, and P5 held exactly as registered.
>
> **This does NOT make 128 adoptable and this lane does not adopt it**
> (Decision 22 §3, `#3732`: eight counterexamples in each direction, and
> `w-lowerband` §6.7.1's `/O1` table). Settling `k` closes a *provenance*
> question. C8's remaining defect is a **unit** — `P_INLINE` §6.6.1's second
> missing link — and this lane does not touch it.

Same argument, same evidence: **`DAT_10c6f1c8 = 0`** at run time here, so
**table A is live**, and **`DAT_10c3de20 = 0`**, so every `== 1` and `== 2`
branch in the image — including §6.7.2's S2 gate and §6.7.3's `DAT_10c3dddc`
arm — is **dead on this workload**. `[O]`

---

## 8. Controls `[R]`

Three instruments over three populations, per the prereg §2, with the two that
can miss watched missing before any verdict here was written.

| control | population | required | observed |
|---|---|---|---|
| **C1 GREEN** — the reference enumerator finds `DAT_10c46318`'s known set | 424,232 decoded instruction starts | writers `0x10b5e4d7`/`0x10b5e4e8`, reader `0x10b5fc8a` (`P_INLINE` §6.6.1, established independently) | exactly that — **GREEN** |
| **C2 RED** — planted address `0xdeadbe00` | same | **0** hits | 0 — **no false positives** |
| **C3 RED** — the descriptor harvest must collapse when pointed elsewhere | 484 immediate-stores in `0x10c29000`–`0x10c2a800` vs a same-sized band at `0x10b5b000` | collapse | **484 → 16** — the window discriminates |
| **cross-population** — Ghidra `xrefs.tsv`, control-flow-driven | whole export | writer sets agree | 19/13 both ways; one READ disagreement at `0x10bd5d2f`, §5.1 |

### 8.1 A control this lane failed, and caught `[O]`

An earlier probe of `/Gy` propagation reported that `cl /Ox /Gy` does **not**
pass `-Gy` to c2 — which would have made two rows of `scripts/lanes.txt`
byte-identical duplicates and was on its way to being a board row.

**It was wrong, and the instrument was the bug.** The loop wrote the mode as an
unquoted `$m`, and **zsh does not word-split unquoted parameter expansions**:
`cl.exe` received `/Ox /Gy` as a *single* argument and parsed only `/Ox`. The
same defect silently dropped `/EHsc` from every multi-flag row. Re-derived by
hand with the flags as separate argv entries:

```
/Bd /c /Ox /Gy /EHsc        -> -Bd -Og -Ob2 -Gy -EHs
/Bd /Ox /Gy /EHsc /GS- /c   -> -Bd -Og -Ob2 -Gy -EHs
```

`-Gy` **is** passed. No lane is a duplicate, no board row is owed, and
`cl_argv_modes.out` is regenerated with zsh's explicit `${=m}` split and
carries the defect in its own header so the next reader cannot repeat it.
Recorded because the false reading was one command away from being published.

---

## 9. Found and not taken

Ranked, sized, with what stopped each.

1. **`DAT_10c462c4` gates the entire parameter fill** (`0x10b5e4f7`) and is
   compared against zero in ~110 places image-wide. Two writers:
   `0x10bec3e4` stores `1` unconditionally in what looks like driver startup,
   and `0x10b84bba` stores `1` when `ds:0x10c45fa0` is non-zero. Not read.
   **If it is ever 0, the 46-dword live record stays zero and §3's whole table
   is inert** — worth one hour and it bounds every statement in §3.
2. **`FUN_10b5b9de`** (`0x10b5b9de`, 147 B, one caller `0x10b5e50a`) adjusts
   **table A's `+0x04` (`-inlT#`) and `+0x08` (`-inlfcsw#`) by module size**,
   in six bands at 50 k / 100 k / 500 k / 1.5 M / 2.5 M: `+32/+24`, `+24/+16`,
   `+16`, `−8`, `−16`, `−24`. So `-inlT#`'s effective default is **80–136**,
   not 104, and `-inlfcsw#`'s is **32–56**. Read but not graded against an obj,
   because the fields are POGO-dead here.
3. **The three non-band reader functions** — `FUN_10bb7aa3`, `FUN_10ba24c4`,
   `FUN_10ba2948` — hold 17 of the 32 reads and were not opened. They are
   outside `0x10b5b86d`–`0x10b62b00`.
4. **`FUN_10b5e9a5`** (`0x10b5e9a5`, 607 B), `FUN_10b5da2f`'s sole caller: what
   it does with the returned 0/1 is unread, so §6's test is characterized
   without its consumer.
5. **The 13 dead fields** `+0x84`…`+0xb4` from `0x10c45d80`–`0x10c45db0`: no
   name, no default, no reader, but scattered every time. Most likely a
   removed feature's parameters `[I]`; a name for them would probably come from
   a later toolchain's `c2.dll`, not this one.
6. **`FUN_10c1f572`'s kinds `0x08`, `0x23`, `0x26`, `0x27`** — sub-tables and
   char-flag lists — were not read, so `optmap.py`'s `(reg)` rows other than
   the four POGO ones are still unresolved.

---

## 10. What this lane did not reach

* **Nothing was graded against an obj**, because everything it read is
  POGO-gated and this project compiles no POGO. §7.2's `[O]` rows are
  toolchain-invocation measurements, not obj comparisons.
* **`-inl*` semantics are given as arithmetic, not as names.** `csw`, `dasw`,
  `casw`, `ocsa`, `mlsa`, `crmax`, `fcsa`, `ipfw`, `nlw` are not expanded here;
  the abbreviations are suggestive and expanding them would be `[I]` dressed as
  `[R]`.
* **`DAT_10c462c4`** — item 1 above — bounds §3 and was not read.
