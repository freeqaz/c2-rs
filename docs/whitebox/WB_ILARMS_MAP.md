# WB_ILARMS — the IL-record dispatch arm → port decode-site MAP

**Lane `w-ilarms`, wave 11, 2026-08-25. Characterization lane, docs-only.**
Prereg: [`WB_ILARMS_PREREG.md`](WB_ILARMS_PREREG.md), committed first
(`7dac34e5e`). Grade and method: [`WB_ILARMS_FINDINGS.md`](WB_ILARMS_FINDINGS.md).
Board **#3567**–**#3572**. Funded by
[`../DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md) **decision 13**,
row 4a(i) / I1.

> **PROVENANCE — TWO HALVES, KEPT APART ON PURPOSE.**
>
> * **`[R]`** — read from `compilers/X360/16.00.11886.00/c2.dll`, sha256
>   `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
>   verified before the first address was quoted. `[R]` says *"the
>   instructions were read correctly"*; it does **not** say *"this is what c2
>   does"*. Whitebox analysis is authorized and encouraged (`CLAUDE.md`,
>   project owner, 2026-08-17).
> * **`[src]`** — read from **this repository**, at the tree this page was
>   written on. Not a read of c2 at all. Every port-site row is `[src]`.
>
> **This lane adopts NOTHING into `crates/` and owes ZERO `DISCLOSURE.md`
> rows.** §9 states what a future adopter would owe, with a number.

**Instruments, both re-runnable:**

```
python3 docs/whitebox/scripts/dump_ilarms.py <c2.dll> --verify   # the tables, from raw bytes
python3 docs/whitebox/scripts/dump_ilarms.py <c2.dll> --tsv > /tmp/ilarms.tsv
python3 docs/whitebox/scripts/scan_port_opcodes.py --coverage /tmp/ilarms.tsv --detail
```

Committed output: [`labels/ilarms_tables.txt`](labels/ilarms_tables.txt) and
[`labels/ilarms_portmap.txt`](labels/ilarms_portmap.txt).

---

## 0. THIS IS NOT A RANKING, AND THAT IS A REQUIREMENT

The table in §3 is ordered by **arm number**, which is the order of the DWORD
target table in the image. It is ordered by nothing else — not by opcode
frequency, not by residue mass, not by body size, not by "how close the port
is". The standing finding is that **a lane dispatched off a blocked-key size
ranking finds the ranking was an artifact, five times over** (`#3505`), and
this lane's prereg §2 registered "ordered by mass anywhere" as a self-grade
failure before a byte was read.

If you are reading this page for *what to work on first*, it does not answer
that and was not built to. It answers **what exists on each side**, so that a
later lane can price a slice it has already chosen.

**And one number here would be the most misreadable on the page if left
alone.** §3 records that the port has an ungated reader for **68 of the 95**
opcodes this dispatch handles. That is a statement about **width** — the
cursor advances by the right amount — and it is **not** a statement that 72 %
of I1 exists. **No port site anywhere mints an IR node**: `P_ILRECORD.md` §6
fixes the node space at `≥ 0x2af`, and a grep of all five crates for the
sixteen node opcodes that dispatch mints (`0x2af`, `0x2b4`, `0x2b5`, `0x2c5`,
`0x2d4`, `0x2dd`, `0x2f4`, `0x2f5`, `0x310`, `0x311`, …) returns ~~**zero
non-comment hits**~~ **exactly ONE — and §0's conclusion survives it.
CORRECTED 2026-08-26 (`w-price4a` found it, `w-opclass` confirmed it and owns
this page; §10).** The hit is
`crates/c2-harness/tests/pwords_bijection.rs:57`,
`const OP_PROLOGUE: [u32; 2] = [0x2f0, 0x2f4];` — **a `#[cfg(test)]` constant
in a test**, naming `0x2f4` as an *expander pseudo-op* out of
`ref/P_EXPAND.md` §4.1, not as an IL-record node. `0x2f4` is a **homograph**:
it is minted by this dispatch *and* is a prologue arm of a different table, and
the test means the second one. So no port site on any production path mints a
node in this space, which is what the paragraph is for — but the sentence as
written was false and is struck rather than reworded.** The port has no analogue
of the artifact these arms build. Width is the part that is cheap; `P_ILRECORD.md` §8.1 already said the
cost is in the **76 tree builders**, and nothing on this page moves that.

---

## 1. The premise, RE-DERIVED — 61 real arms, 95 opcodes, 94 refusals

`P_ILRECORD.md`'s ⛔ banner says the read plan's "189 arms" is an **opcode**
count. **The coordinator had not verified that**, and this lane's brief was
explicit that *"the instructions were read correctly"* is `[R]`, not *"this
is what c2 does"*. So it was re-derived rather than inherited.

**`dump_ilarms.py` deliberately shares no code with `dump_ilrecord.py`.**
That script hard-codes `BYTE_TABLE_VA`, `JUMP_TABLE_VA`, both table lengths
and both opcode bounds as module constants, so re-running it cannot test
whether those constants are right — it can only test that the bytes at those
addresses have not changed. `dump_ilarms.py` hard-codes **one** address, the
dispatch head `0x10bc2e08`, and derives every table VA, stride, extent and
bound from the operand bytes of the instructions it decodes there.

### 1.1 The head, decoded byte by byte `[R]`

```
10bc2e08  8b55cc          mov edx, [ebp-0x34]
10bc2e0b  8d42ff          lea eax, [edx-0x1]
10bc2e0e  3dbc000000      cmp eax, 0xbc
10bc2e13  0f872a130000    ja 0x10bc4143
10bc2e19  0fb6804a42bc10  movzx eax, byte ptr [eax + 0x10bc424a]
10bc2e20  ff24855241bc10  jmp dword ptr [eax*4 + 0x10bc4152]
```

Everything below falls out of those six operands and out of no other input:

| derived | value | from |
|---|---|---|
| opcode domain | `0x01`…`0xbd`, **189** opcodes | `lea` displacement `−1` + `cmp` bound `0xbc`, unsigned `ja` |
| byte index table | `0x10bc424a`…`0x10bc4306`, stride 1, **189** entries | the `movzx` displacement |
| arm target table | `0x10bc4152`…`0x10bc4249`, stride 4, **62** entries | the `jmp` displacement; `62 = max(index)+1` |
| index value range | `0`…`61` | the table's own bytes |
| out-of-range target | `0x10bc4143` | the `ja` rel32 |

### 1.2 The five checks, and what each excludes `[R]`

| # | check | result | the alternative it kills |
|---|---|---|---|
| 1 | index values span `0..61` | **62 arms** | an arm table sized by guess |
| 2 | **are the 62 targets distinct?** | **62 of 62 distinct**, no duplicates | *this is the one `62 entries` alone does not exclude* — a 62-entry table can still name fewer than 62 arms, which is exactly R2's `111 → 79` at the encoder (`P_ENCODE.md` §4) |
| 3 | containment | **62 of 62** targets inside `[0x10bc2d7a, 0x10bc4152)` | targets escaping into a neighbouring function |
| 4 | the refusal | **arm 61** is the sole index whose target equals the `ja` destination; **94** in-range opcodes route there, **95** are handled, **61** arms are real | a refusal reachable only from out of range |
| 5 | **table extent, read a second way FROM RAW BYTES** | 189 entries end at `0x10bc4306`; `0x10bc4307` opens `55 8b ec` = `push ebp; mov ebp,esp` | a **longer** byte table whose tail the `cmp/ja` never reaches. `P_ILRECORD.md` took this boundary from `c2_strings.tsv:833`, an artifact; this is the prologue itself |

Check 5 was pushed one step further because "consistent with 189" is not
"189". The 16 bytes past the table are
`55 8b ec 83 ec 30 53 8b d9 8b 03 56 8b 75 08 81`, of which only **3 of 16**
are even legal arm indices — a longer table would need **all** of them to be.
The three tables are also **exactly packed**: `0x10bc2d7a + 5080 = 0x10bc4152`
(body ends where the arm table starts), the arm table ends at `0x10bc4249`
and the byte table starts at `0x10bc424a` — **a zero-byte gap**.

### 1.3 The independent second implementation agrees on all nine comparisons

`dump_ilarms.py --cross` re-runs `dump_ilrecord.py`'s reader and compares:
both table VAs, both lengths, both opcode bounds, the `ja` target, **the 189
index bytes element for element**, and **the 62 target words element for
element**. `ALL AGREE`.

### 1.4 The opcode→arm assignment agrees with `P_ILRECORD.md` §3 on all 62 rows

Prereg **T5** registered this at p = 0.65, because `#3547` had already found
one of that table's *prose* cells wrong in both clauses and nobody had ever
re-derived its *opcode* column. It is **right on all 62 rows**, including the
two that look like typos and are not: arm 1 serves ten opcodes because `0x07`
refuses, and arm 4 serves ten because `0x14` refuses.

> **So the correction stands, at the third independent decode.** **61 arms
> serve 95 opcodes, plus one refusal serving 94.** Every count on this page
> uses those denominators and no other.

---

## 2. How to read the map's two ends

### 2.1 The c2 end `[R]`

`arm VA` and `opcode(s)` and `class` are this lane's own raw reads
(`dump_ilarms.py`; the class byte comes from `0x10b25e48`, board **#1591**'s
table, indexed by opcode). **`role / verdict` is INHERITED from
`P_ILRECORD.md` §3 and was NOT re-derived here** — that column is one lane's
`[R]` carried forward, marked so a reader can tell. Arm 7's cell in that page
is amended by **#3547**; nothing in this lane touched it.

### 2.2 The port end `[src]`

Sites come from `scan_port_opcodes.py`, which walks the five crates for match
arms whose pattern is an 8-bit hex literal, drops `#[cfg(test)]`, and
attributes each to its enclosing `fn`. Its exclusions are **in the source
with their reasons**, not applied silently: six files that decode a
**different stream** (`.sy`, `.gl`, `.in`, EH scope records — the same
literal space, a different numbering), ten functions whose literals are TYPE
tags / PPC `rlwinm` masks / VI32 wide markers, and two **lines** where an
opcode value is matched in **sub**-opcode position.

Each site carries a **GATE** class, established by reading its callers:

| gate | meaning | established by |
|---|---|---|
| **U** | **ungated** — runs on every body | `control_flow::{step,operand}` is reached from `census.rs:448 scan_full`, which the census runs on every body in the workload; `codec::try_*_token` is the container codec, fenced by a byte-exact round-trip; `body/mod::{expr,cflow}_opcode_name` are census names |
| **G-adm** | **admission-gated** | `parse_expr_classed` has exactly one caller, `body/mod.rs:2832`, on the accepting path; `parse_segment_shape` **is** the admission gate — decision 13's *"decode and admission are fused"* |
| **G-env** | **environment-gated** | `chain_skip_form` is the chain sink's width table: *"poisoned, environment-gated, off on every gate lane and every default scan"*, its own module doc |
| **G-shp** | **shape-gated** | reached only after a named body shape has matched |

and a **DEPTH** class, which is orthogonal: `name` (a census key or enum
discriminant, **zero** operand bytes consumed), `width` (the cursor advances
correctly, the payload is discarded), `field` (at least one operand field is
retained for a downstream consumer).

### 2.3 The verdicts, defined in the prereg BEFORE anything was counted

* **ABSENT** — no site names the byte.
* **NARROW(gate)** — every site is behind a precondition the arm does not
  impose.
* **MATCHED\*** — at least one **ungated** site. The asterisk is load-bearing
  and is not decoration: it means **limb 1 of the prereg's NARROW test is
  clear and limb 2 — "consumes fewer operand fields than the arm's class
  implies" — is UNCHECKED for this row.** §6 is the sub-population where
  limb 2 is decidable, and it is decided there.
  **THE ASTERISK IS RETIRED as of 2026-08-26 — §10 decides limb 2 on ALL 68,
  and the 68 `MATCHED*` become 35 `MATCHED` · 30 `NARROW(fields)` ·
  2 `WIDE(fields)` · 1 `UNRESOLVED`. Every count in §3 and §4 below is a
  LIMB-1 count and stays exactly as measured; read them as such.**
* **UNRESOLVED** — reported and counted, never guessed.

The *primary port site* column names the **first ungated, cursor-moving**
site for the arm's first covered opcode. The `sites` column is the total
count over all the arm's opcodes, so a row with a modest primary and a large
count is one many readers touch.

---

## 3. THE MAP — 62 arms, ordered by arm number

| # | arm VA | opcode(s) | class | role / verdict — `[R]`, INHERITED | primary port site — `[src]`, under `crates/c2-il/src/` | verdict | sites |
|--:|---|---|---|---|---|---|--:|
| 0 | `0x10bc3ff6` | `01` `2a` `2b` `43` | `00/03/04` | STATE/SELECT | `func/body/shapes/control_flow.rs:1066` `operand` | 3×ABSENT + 1×MATCHED* | 4 |
| 1 | `0x10bc2ec1` | `02` `03` `04` `05` `06` `09` `0a` `0b` `0c` `0d` | `00` | ROUTE/DEFER | `codec.rs:1278` `try_ex_token` | MATCHED* | 57 |
| 2 | `0x10bc386f` | `08` | `00` | ROUTE/DECODE | **none** | ABSENT | 0 |
| 3 | `0x10bc3868` | `0e` | `00` | ROUTE/DECODE | `func/body/shapes/control_flow.rs:882` `operand` | MATCHED* | 2 |
| 4 | `0x10bc37c3` | `0f` `10` `11` `12` `13` `15` `16` `17` `18` `19` | `01` | ROUTE/DEFER | `func/body/shapes/control_flow.rs:897` `operand` | MATCHED* | 18 |
| 5 | `0x10bc31fb` | `1a` | `00` | STATE/SELECT | `func/body/shapes/control_flow.rs:882` `operand` | MATCHED* | 4 |
| 6 | `0x10bc3254` | `1b` `1c` | `00` | REWRITE/SELECT | `func/body/shapes/control_flow.rs:882` `operand` | MATCHED* | 6 |
| 7 | `0x10bc38a1` | `1f` `20` `21` `22` `23` `24` | `00` | ROUTE/DEFER | `func/body/shapes/control_flow.rs:882` `operand` | MATCHED* | 30 |
| 8 | `0x10bc2e27` | `26` | `02` | ROUTE/DEFER | `codec.rs:1134` `try_prefix_token` | MATCHED* | 12 |
| 9 | `0x10bc3891` | `27` | `01` | ROUTE/DEFER | `codec.rs:1297` `try_ex_token` | MATCHED* | 7 |
| 10 | `0x10bc3881` | `28` | `02` | ROUTE/DEFER | `func/body/shapes/control_flow.rs:1033` `operand` | MATCHED* | 5 |
| 11 | `0x10bc3117` | `29` | `02` | STATE/SELECT | `func/body/shapes/control_flow.rs:751` `step` | MATCHED* | 4 |
| 12 | `0x10bc2f75` | `2c` `34` | `05` | ROUTE/SELECT | `codec.rs:1315` `try_ex_token` | 1×ABSENT + 1×MATCHED* | 8 |
| 13 | `0x10bc2ff1` | `30` `5a` | `01` | ROUTE/SELECT | `codec.rs:1306` `try_ex_token` | 1×ABSENT + 1×MATCHED* | 6 |
| 14 | `0x10bc3784` | `32` | `01` | REWRITE/SELECT | `codec.rs:1323` `try_ex_token` | MATCHED* | 6 |
| 15 | `0x10bc2fc5` | `33` | `06` | BUILD/SELECT | `codec.rs:1256` `try_ex_token` | MATCHED* | 10 |
| 16 | `0x10bc37d3` | `35` `36` | `01` | BUILD/SELECT | `func/body/shapes/control_flow.rs:897` `operand` | MATCHED* | 3 |
| 17 | `0x10bc2f43` | `37` | `00` | STATE/DECODE | **none** | ABSENT | 0 |
| 18 | `0x10bc304d` | `38` `39` | `02` | BUILD/SELECT | `func/body/shapes/control_flow.rs:757` `step` | MATCHED* | 8 |
| 19 | `0x10bc30ec` | `3a` | `02` | BUILD/DECODE | `codec.rs:1331` `try_ex_token` | MATCHED* | 6 |
| 20 | `0x10bc32ec` | `3b` | `02` | REWRITE/SELECT | `func/body/shapes/control_flow.rs:769` `step` | MATCHED* | 2 |
| 21 | `0x10bc34e0` | `3c` | `07` | BUILD/DECODE | `func/body/shapes/control_flow.rs:774` `step` | MATCHED* | 2 |
| 22 | `0x10bc3510` | `3d` | `08` | BUILD/DECODE | `func/body/shapes/control_flow.rs:780` `step` | MATCHED* | 2 |
| 23 | `0x10bc371c` | `3e` `bd` | `09/19` | ROUTE/DEFER | `codec.rs:1233` `try_ex_token` | 1×ABSENT + 1×MATCHED* | 6 |
| 24 | `0x10bc3697` | `40` | `01` | STATE/SELECT | `func/body/shapes/control_flow.rs:1054` `operand` | MATCHED* | 3 |
| 25 | `0x10bc3771` | `41` | `01` | ROUTE/DEFER | `codec.rs:1288` `try_ex_token` | MATCHED* | 7 |
| 26 | `0x10bc3555` | `42` | `02` | ROUTE/DEFER | **none** | ABSENT | 0 |
| 27 | `0x10bc3166` | `44` | `00` | REWRITE/SELECT | `func/body/shapes/control_flow.rs:1025` `operand` | MATCHED* | 2 |
| 28 | `0x10bc3621` | `46` | `00` | ROUTE/DEFER | `codec.rs:1138` `try_prefix_token` | MATCHED* | 2 |
| 29 | `0x10bc3651` | `47` | `00` | STATE/SELECT | **none** | ABSENT | 0 |
| 30 | `0x10bc38ae` | `4b` | `00` | ROUTE/DEFER | `func/body/shapes/control_flow.rs:789` `step` | MATCHED* | 5 |
| 31 | `0x10bc3645` | `4c` | `00` | STATE/DECODE | `codec.rs:1192` `try_ex_token` | MATCHED* | 6 |
| 32 | `0x10bc38bb` | `4f` | `0c` | ROUTE/DEFER | `codec.rs:1120` `try_prefix_token` | MATCHED* | 4 |
| 33 | `0x10bc3580` | `53` | `00` | STATE/SELECT | `codec.rs:1133` `try_prefix_token` | MATCHED* | 5 |
| 34 | `0x10bc35b4` | `54` | `0d` | STATE/SELECT | `codec.rs:1335` `try_ex_token` | MATCHED* | 3 |
| 35 | `0x10bc3570` | `55` | `01` | ROUTE/DEFER | `codec.rs:1281` `try_ex_token` | MATCHED* | 6 |
| 36 | `0x10bc38d2` | `56` | `00` | STATE/DECODE | **none** | ABSENT | 0 |
| 37 | `0x10bc2ece` | `59` | `00` | BUILD/SELECT | **none** | ABSENT | 0 |
| 38 | `0x10bc3b00` | `5c` | `13` | ROUTE/DEFER | `func/body/shapes/control_flow.rs:969` `operand` | MATCHED* | 2 |
| 39 | `0x10bc3b10` | `5d` `5e` | `14` | BUILD/SELECT | `func/body/shapes/control_flow.rs:984` `operand` | MATCHED* | 6 |
| 40 | `0x10bc3b7d` | `60` `62` | `00` | BUILD/SELECT | **none** | ABSENT | 0 |
| 41 | `0x10bc3b4f` | `61` | `15` | BUILD/DECODE | **none** | ABSENT | 0 |
| 42 | `0x10bc3569` | `64` | `01` | STATE/DECODE | `func/body/shapes/control_flow.rs:1154` `operand` | MATCHED* | 1 |
| 43 | `0x10bc3fe0` | `66` | `1a` | BUILD/DECODE | `func/body/shapes/control_flow.rs:1088` `operand` | MATCHED* | 6 |
| 44 | `0x10bc39b6` | `67` | `1b` | BUILD/DECODE | `func/body/shapes/control_flow.rs:1110` `operand` | MATCHED* | 3 |
| 45 | `0x10bc3ba7` | `68` | `00` | BUILD/SELECT | **none** | ABSENT | 0 |
| 46 | `0x10bc3a11` | `77` | `01` | REWRITE/SELECT | **none** | ABSENT | 0 |
| 47 | `0x10bc3c1c` | `8b` | `00` | BUILD/SELECT | **none** | ABSENT | 0 |
| 48 | `0x10bc38f5` | `8d` | `01` | BUILD/DECODE | **none** | ABSENT | 0 |
| 49 | `0x10bc3967` | `8e` | `00` | BUILD/DECODE | **none** | ABSENT | 0 |
| 50 | `0x10bc3953` | `8f` | `00` | BUILD/DECODE | **none** | ABSENT | 0 |
| 51 | `0x10bc396e` | `90` | `00` | BUILD/DECODE | **none** | ABSENT | 0 |
| 52 | `0x10bc3987` | `99` | `1c` | STATE/SELECT | `func/body/shapes/control_flow.rs:1164` `operand` | MATCHED* | 6 |
| 53 | `0x10bc39aa` | `9a` | `01` | STATE/DECODE | `func/body/shapes/control_flow.rs:1129` `operand` | MATCHED* | 2 |
| 54 | `0x10bc39c1` | `9b` | `12` | BUILD/SELECT | `func/body/shapes/control_flow.rs:1170` `operand` | MATCHED* | 4 |
| 55 | `0x10bc3aef` | `9d` | `00` | ROUTE/DEFER | **none** | ABSENT | 0 |
| 56 | `0x10bc3c66` | `a0` | `01` | REWRITE/SELECT | **none** | ABSENT | 0 |
| 57 | `0x10bc3fd6` | `a2` | `17` | ROUTE/DEFER | **none** | ABSENT | 0 |
| 58 | `0x10bc2e38` | `b9` | `18` | BUILD/SELECT | `codec.rs:1242` `try_ex_token` | MATCHED* | 11 |
| 59 | `0x10bc3975` | `bb` | `07` | STATE/DECODE | **none** | ABSENT | 0 |
| 60 | `0x10bc3159` | `bc` | `00` | ROUTE/DEFER | **none** | ABSENT | 0 |
| **61** | `0x10bc4143` | **94 opcodes** — listed in §5 | many | **REFUSE** — C1001 `reader.c:3295` | **out of scope by construction** | — | — |

---

## 4. The counts, every one with its denominator

**Arms** (denominator **61 real arms**, the 62nd being the refusal):

| | count | of |
|---|--:|--:|
| arms with ≥ 1 port site | **41** | 61 (67.2 %) |
| arms with **no** port site at all | **20** | 61 (32.8 %) |
| arms reachable from `control_flow.rs` | **40** | 61 |
| arms reachable from `expr.rs` | 33 | 61 |
| arms reachable from `mcall_tail.rs` | 18 | 61 |
| arms reachable from `codec.rs` | 17 | 61 |
| arms reachable from `mcall.rs` | 16 | 61 |
| arms reachable from `body/mod.rs` | 15 | 61 |
| arms reachable from each of the other **8** port files | 1–4 each | 61 |
| **arms for which the port mints the IR node the arm mints** | **0** | 61 |

(An arm may appear under several files; the column is *reach*, not a
partition, which is why it does not sum to 41.)

**Opcodes** (denominator **95 handled**; the other 94 are §5):

| | count | of |
|---|--:|--:|
| handled opcodes with ≥ 1 port site | **68** | 95 (71.6 %) |
| handled opcodes with **no** port site | **27** | 95 (28.4 %) |
| … of those 27, in an arm that is **wholly** uncovered | 21 | 27 |
| … of those 27, in an arm the port covers **partly** | 6 | 27 — `01` `2a` `2b` (arm 0), `34` (arm 12), `5a` (arm 13), `3e` (arm 23) |
| verdict **MATCHED\*** | **68** | 95 |
| verdict **NARROW(gate)** | **0** | 95 |
| verdict **ABSENT** | **27** | 95 |
| of the 68, those where **no operand field survives** (width- or name-only) | **24** | 95 |

Every one of the 68 has an ungated site that **moves the cursor** — none is
covered by a census name table alone. Those ungated cursor-moving sites are
distributed `control_flow.rs` 80 · `codec.rs` 23 · `bundle.rs` 2 (site-hits,
not opcodes: one opcode may have several).

**`NARROW(gate)` is zero, and that is the map's most consequential single
result.** Wherever the port names one of these bytes at all, it names it in a
reader that is **not** behind the admission gate. Decision 13 funded
`w-unfuse` on the premise that *"decode and admission are fused"*; that is
true of `BodyShape`/`parse_expr_classed`, and it is **already false** of the
census walk `control_flow::{step,operand}`, which reaches **40 of 61** arms
with no gate at all. The unfusing `w-unfuse` is building has a working
precedent inside the same crate, and the map names it.

**Refusals** (denominator **94**):

| | count | of |
|---|--:|--:|
| refused opcodes with a port site | **1** | 94 — `0x2d` |
| port opcode literals outside the dispatch domain `0x01..0xbd` | **0** | — |

---

## 5. The 94 refusing opcodes — OUT OF SCOPE BY CONSTRUCTION

Stated with its count so a later reader does not price them. **94 of the 189
opcodes route to arm 61**, `0x10bc4143`, three instructions that raise C1001
through `reader.c` line 3295. Arm 61 is also the destination of the
out-of-range `ja`. Re-derived here from the index table; identical to
`P_ILRECORD.md` §1.3's set.

```
07 14 1d-1e 25 2d-2f 31 3f 45 48-4a 4d-4e 50-52 57-58 5b 5f 63 65
69-76 78-8a 8c 91-98 9c 9e-9f a1 a3-b8 ba
```

**These are legal `.ex` tokens** — the class table `0x10b25e48` assigns most
of them class `00` — and they are handled by **some other walk**, which is
unread (`P_ILRECORD.md` §8 item 4). They are out of scope for I1 because I1
is this dispatch, not because the bytes are impossible.

### 5.1 And the port reads exactly one of them — `0x2d`, in the PREFIX

Prereg **V4** registered at p = 0.35 that the port would carry a reader for a
byte this dispatch refuses, and said in advance that **either result is a
finding**. It hits, exactly once:

| opcode | port site `[src]` | what it reads |
|---|---|---|
| `0x2d` | `crates/c2-il/src/codec.rs:1139` `try_prefix_token`, and `:1344` `try_ex_token` | `ExToken::Formal(tok)` — `2D <2-byte token>` |

**This is not a contradiction; it is a confirmation, and a sharp one.**
`codec::try_prefix_token` reads the `.ex` per-function **metadata prefix**,
which is a different position in the stream from the body token loop
`FUN_10bc2d7a` walks. §1.3's claim — *"those opcodes are legal `.ex` tokens
… `0x10bc2d7a` is one consumer of the `.ex` stream, not the consumer"* — is
an unfalsifiable-sounding sentence until something exhibits a byte that is
read in one position and refused in the other. `0x2d` is that byte, and its
neighbour `0x46` (`ExToken::Formals`) is the control: `0x46` is read by the
**same two port functions** and is **handled** by this dispatch, at arm 28.

So the port's decode surface **spans at least two c2 consumers of `.ex`**,
and a slice that scoped itself to "the port's IL readers" would be scoping
across a boundary the binary draws. Arm 28's row and `0x2d`'s row look
identical from the port side and are on opposite sides of it.

---

## 6. Limb 2 — where "MATCHED\*" is decidable, and what it decides

`MATCHED*` means limb 1 (gate) is clear and limb 2 (fields) is unchecked.
Limb 2 needs c2's per-class operand grammar, which lives in the 29 class arms
at `0x10b3d954` and was **not** read by this lane. What this lane *can* do
without reading them is use the class byte — its own raw read — as an
**internal consistency check on the port**: opcodes sharing a class share a
payload grammar, so a class the port reads two different ways is a place the
port's widths were pinned per opcode from witnesses instead of from the
grammar.

```
    operand class (raw read of 0x10b25e48) vs port coverage, over the 95 HANDLED opcodes only:
      class 00  26/41   01- 02 03 04 05 06 08- 09 0a 0b 0c 0d 0e 1a 1b 1c 1f 20 21 22 23 24 37- 43 44 46 47- 4b 4c 53 56- 59- 60- 62- 68- 8b- 8e- 8f- 90- 9d- bc-   <-- PARTIAL: same payload grammar, read on some and not others
      class 01  20/24   0f 10 11 12 13 15 16 17 18 19 27 30 32 35 36 40 41 55 5a- 64 77- 8d- 9a a0-   <-- PARTIAL: same payload grammar, read on some and not others
      class 02  7/8   26 28 29 38 39 3a 3b 42-   <-- PARTIAL: same payload grammar, read on some and not others
      class 03  0/1   2a-
      class 04  0/1   2b-
      class 05  1/2   2c 34-   <-- PARTIAL: same payload grammar, read on some and not others
      class 06  1/1   33
      class 07  1/2   3c bb-   <-- PARTIAL: same payload grammar, read on some and not others
      class 08  1/1   3d
      class 09  0/1   3e-
      class 0c  1/1   4f
      class 0d  1/1   54
      class 12  1/1   9b
      class 13  1/1   5c
      class 14  2/2   5d 5e
      class 15  0/1   61-
      class 17  0/1   a2-
      class 18  1/1   b9
      class 19  1/1   bd
      class 1a  1/1   66
      class 1b  1/1   67
      class 1c  1/1   99
      5 of 22 classes are covered only partly
```

(`-` marks an opcode with no port site. **22 distinct classes** over the 95
handled opcodes; **5** are covered only partly.)

### 6.1 `0x28` — NARROW(fields), provable inside this lane

`0x28` is class **`02`**. Its class-`02` siblings in the handled set are
`26` `29` `38` `39` `3a` `3b`, and the port reads **every one of them** as
`<op>` followed by `Scan::tok()` → `read_token_var`, a **variable**-width
token. It reads `0x28` as a hard-coded literal `28 00 00`
(`control_flow.rs:1033`) and **refuses anything else**. Same c2 class, two
port widths, and the narrow one is a fixed literal. **Verdict `NARROW(fields)`,
and it is a refusal rather than a desync** — the port fails closed, which is
the correct direction.

The identical argument runs on `0x43` and lands the other way; see §7.

### 6.2 The rows where limb 2 is UNRESOLVED, named rather than assumed

| opcode | class | why limb 2 cannot be decided here | the lead (**prior art, NOT adopted as a premise**) |
|---|---|---|---|
| `0x2c` | `05` | its only class sibling `0x34` is ABSENT, so there is nothing to cross-check against | `WB_READER_FINDINGS.md` §5 reads class `05` as `TYPE` + one **raw byte**, against the port's varint — a *latent desync at any payload ≥ `0x80`*. A lead for a later lane, not a finding of this one |
| `0x54` | `0d` | it is the **only** class-`0d` opcode this dispatch handles — every other class-`0d` opcode (`50` `51` `52` `5b` `9e` `9f`) is in the 94 | same page reads class `0d` as `i32c`; `control_flow.rs:730` reads `+2` fixed and says byte-vs-varint is UNKNOWN in its own comment |
| the other 65 | various | ~~the class arms were not read~~ **CLOSED — §10** | — |

~~**65 of the 68 `MATCHED*` rows have limb 2 genuinely unchecked**, and that is
published rather than rounded away. Reading `0x10b3d954`'s 29 class arms
would close all of them at once and is the single cheapest follow-up this map
exposes.~~ **CLOSED 2026-08-26 by lane `w-opclass` — §10 below. 30 of the 65
change verdict. And the read this sentence prices as a follow-up had already
been taken, four times, one of them by the page this section cites twice
(§10.1).**

---

## 7. What I could NOT map, and one thing I mapped WRONG at first

### 7.1 The 20 arms with no port site

Ordered by arm number, per §0.

| arm | opcode | class | role/verdict `[R]` inherited |
|--:|---|---|---|
| 2 | `08` | `00` | ROUTE/DECODE → `0x10bc0f77(edx=0x2b5)` |
| 17 | `37` | `00` | STATE/DECODE — pops two, links, `flags \|= 4` |
| 26 | `42` | `02` | ROUTE/DEFER → `0x10bc00a1` (2,282 B) |
| 29 | `47` | `00` | STATE/SELECT — branches on the global `0x10c3cf96` |
| 36 | `56` | `00` | STATE/DECODE — resets three globals, a phase boundary |
| 37 | `59` | `00` | BUILD/SELECT — mints `0x2b0` and/or `0x2b6` |
| 40 | `60` `62` | `00` | BUILD/SELECT — `0x2fe + 2·(op != 0x60)` |
| 41 | `61` | `15` | BUILD/DECODE — mints `0x2ff` |
| 45 | `68` | `00` | BUILD/SELECT — mints `0x2c5` |
| 46 | `77` | `01` | REWRITE/SELECT — a multi-node sequence |
| 47 | `8b` | `00` | BUILD/SELECT — mints `0x2c6` |
| 48 | `8d` | `01` | BUILD/DECODE — mints `0x2f4` **and** `0x2f5`, read R6's prologue pair |
| 49 | `8e` | `00` | BUILD/DECODE — `0x2f0` |
| 50 | `8f` | `00` | BUILD/DECODE — `0x2ee` |
| 51 | `90` | `00` | BUILD/DECODE — `0x2ef` |
| 55 | `9d` | `00` | ROUTE/DEFER → `0x10bc4307`, the function right after the tables |
| 56 | `a0` | `01` | REWRITE/SELECT — the 880-byte peephole |
| 57 | `a2` | `17` | ROUTE/DEFER → `0x10bd3aa8` |
| 59 | `bb` | `07` | STATE/DECODE |
| 60 | `bc` | `00` | ROUTE/DEFER → `0x10bc1e79` |

**A row I could not resolve is a row, not an omission** — `w-keymap` declared
30.89 % of its population unattributable and that was the deliverable
working. Here it is 32.8 % of arms and 28.4 % of handled opcodes.

### 7.2 Arms 17 and 26 are NOT really absent — the port reads their bytes as sub-opcodes of a `0x43` escape that does not exist

`control_flow.rs:1066` reads `0x43` as an **escape**: `43 42` consumes 4
bytes total, `43 37` consumes 2, every other sub-byte refuses. But `0x43` is
class **`00`** in the table this lane read itself, and class `00`'s port
siblings are all read as **one payload-free byte**. Two readings of the same
class, so one of them is wrong — and the arithmetic says which:

| the port's "escape" | c2's two tokens | width |
|---|---|---|
| `43 42 XX XX` → `+4` | `43` (class `00`, payload-free) **then** `42` (class `02`, a `varU` token) | `1 + (1 + 2) = 4` ✔ |
| `43 37` → `+2` | `43` (class `00`) **then** `37` (class `00`, payload-free) | `1 + 1 = 2` ✔ |

Both widths reproduce **exactly**, from three class bytes this lane read out
of the image and the port's own 2-byte narrow token form. So the port is
consuming arm 17's and arm 26's bytes — as a private sub-opcode of a
fictitious escape rather than as top-level opcodes.

**I am keeping the ABSENT count at 20, and reporting this beside it rather
than folding it in.** "FOLDED" is a fifth state and the prereg fixed four; a
category invented after the numbers are in is exactly how a count stops being
gradeable. What §7.1 claims is *"no site names this byte as a top-level
opcode"*, which is true of `37` and `42`, and this subsection says what is
true instead.

**And it names a live hazard.** A `varU` token with bit 15 set is **4** bytes,
not 2, so `43 42` over a wide token is **6** bytes and the port's fixed `+4`
walks two bytes into the payload. That is the same shape as `0x28`'s fixed
`00 00` (§6.1) except that `0x28` refuses and this one **advances**. ~~Not
graded here — no cell was compiled — and reported as a hazard, not a defect.~~
**MEASURED 2026-08-26 by `w-opclass` — §10.4: the hazard is REAL in the language
and NOT WITNESSED in the workload. A token walk of 867 sources found 2,404 real
`43 42` sites and ZERO wide tokens, and every top-level `0x42` in the workload
is preceded by a `0x43`. It stays a hazard, and the reading above stays right.**

### 7.3 What this map does not give I1

1. **The tree builders**, unchanged from `P_ILRECORD.md` §8.1: 61 arms route
   into **76 distinct direct callees over 174 call sites**, read to depth 1
   only, **0 of 76** bodies read. Nothing on this page reduces that.
2. **Any claim that a port site is CORRECT.** Every verdict is about
   existence, gating and width. `control_flow::operand` advancing the right
   number of bytes says nothing about whether the port would build what c2
   builds — and §0 records that it builds nothing at all.
3. **The `≥ 0x295` node space.** Zero port bytes model it.
4. **A price.** `#1767`'s rule and `#3421`'s refusal govern: the arm count is
   ~3× smaller than the planning documents state and the callee surface is
   unbounded by any read so far, and this lane declines to combine them.

---

## 8. What this map DOES give, stated once

Four things a later I1 slice can size from, and nothing beyond them:

1. **The denominators are fixed and re-derivable**: 61 real arms, 95 handled
   opcodes, 94 refusals, 22 operand classes, and the exact opcode set of each.
2. **The port's existing surface is located, both ends cited**: 41 arms have
   a site, 20 do not, and the ungated general reader is
   `control_flow.rs::{step,operand}` reached from `census.rs:448` — not the
   admission path.
3. **The scope boundary is drawn**: 94 opcodes are out of scope by
   construction, and `0x2d` shows the port already reads across the boundary
   into a second, unread c2 consumer.
4. **Two width hazards are named with their evidence** (§6.1, §7.2), both
   found by using the operand class as a cross-check on the port rather than
   as a fact about c2.

---

## 9. Disclosure

**This lane adopts nothing into `crates/` and owes ZERO `DISCLOSURE.md`
rows.** `crates/` is byte-identical across the whole lane; the fence is
checked, not asserted (`WB_ILARMS_FINDINGS.md` §7).

**What a future adopter would owe**, stated as a number rather than gestured
at. `DISCLOSURE.md`'s rule is a row naming the address in the same commit
that adopts a disassembly-derived constant. A slice that implemented the
DECODE arms of this dispatch would adopt, at minimum:

* **2** table addresses (`0x10bc424a`, `0x10bc4152`) plus the dispatch head
  `0x10bc2e08` — **3**;
* **1** per arm implemented — the arm VA is the provenance for its opcode set
  and behaviour; **61** if all real arms are taken, **17** for the DECODE
  subset `P_ILRECORD.md` §3.1 counts;
* **1** for the refusal `0x10bc4143` and its `reader.c:3295` line number;
* **2** for the joined tables `0x10b25e48` (operand class) and `0x10b25f10`
  (attributes), if either enters a predicate;
* **0** for any node opcode, until something mints one.

So **≥ 23 rows for the DECODE subset and ≥ 67 for the whole dispatch**, and
that is a floor: every callee body a slice reads adds its own.

---

# 10. SECOND ROUND — limb 2, closed

> **Added 2026-08-26 by lane `w-opclass` (wave 12, board #3585–#3590), as a
> clearly marked second-round section.** Nothing above is rewritten or deleted:
> the strikes are inline at the affected lines and each names this section, per
> [`../DOC_CONVENTIONS.md`](../DOC_CONVENTIONS.md) §2 mitigation 1. Full
> write-up and the prereg grade:
> [`WB_OPCLASS_FINDINGS.md`](WB_OPCLASS_FINDINGS.md). Instruments:
> [`scripts/dump_opclass.py`](scripts/dump_opclass.py),
> [`scripts/cross_opclass_port.py`](scripts/cross_opclass_port.py),
> [`scripts/scan_esc43.py`](scripts/scan_esc43.py).

## 10.1 First, the thing this map got wrong about itself

§6 prices reading the 29 class arms as *"the single cheapest follow-up this map
exposes"*. **The read had already been taken, four times**, and one of the four
is a page **this map cites twice in §6.2**:

* [`WB_READER_FINDINGS.md`](WB_READER_FINDINGS.md) §3 — all 29 arms, with VAs
  and a one-line grammar each (`wb-reader`, 2026-08-08, board **#1591**);
* `BOARD.md` **#1592** — nine port/c2 width disagreements, **including `0x28`
  and *"`0x43` is not an escape"***, i.e. both hazards §6.1 and §7.2 name;
* [`READ_PLAN_2026-08-21.md:73`](READ_PLAN_2026-08-21.md) — *"all 29 arms
  read"*, in the **already-read** section of the tree's own read index. This
  lane's consumer sweep amended **`:99`** and **`:174`** of that file;
* `work/wb-eh/extok.py`, committed, and its output at
  [`WB_EH_FINDINGS.md`](WB_EH_FINDINGS.md) §4.2 — a working tokenizer applying
  all 29 arms to a real body.

**What was genuinely unbuilt is the CROSS, not the read.** `WB_READER_FINDINGS.md`
§3.4 crossed **nine positions**; nobody had crossed the class grammar against
the port's readers over the handled set. That cross is §10.2, and it is what
closes limb 2.

## 10.2 The counts, with §4's own denominators

Verdict vocabulary fixed **before** measurement
(`../rungs/_2026-08-26-w-opclass-prereg.md` §1), four values, no fifth:
`MATCHED` · `NARROW(fields)` · `WIDE(fields)` · `UNRESOLVED`.

| | of **68** `MATCHED*` | of the **65** §6.2 called unchecked |
|---|--:|--:|
| **MATCHED** | 35 | 35 |
| **NARROW(fields)** | 30 | 28 |
| **WIDE(fields)** | **2** | 1 |
| **UNRESOLVED** | 1 | 1 |
| **change verdict** | **33** | **30** |

Strict field-**count** reading, as a second denominator: the port's field count
equals the class's on **57 of 68**. The width-function reading is primary, and
that is §6.1's own precedent — `0x28`'s counts are equal and its verdict is
`NARROW(fields)` on the width function alone.

**33 rows, 8 root causes, and 26 of the 33 are ONE function.** The 26 are
`readers::read_type` against c2's TYPE word, four separate narrownesses in one
place (the one-byte short form; the three-byte form's unmasked middle byte; the
aggregate escape below 32; the LEB id capped at 5 bytes). All four fail
**closed**. The other seven causes are one row each: `0x2c`, `0x43`, `0x28`,
`0x54`, `0x66`, `0x4f`, `0x33`.

## 10.3 The rows that change verdict, and why — every one named

| row | §3 said | now | why |
|---|---|---|---|
| `0f 10 11 12 13 15 16 17 18 19 35 36` (arm 4, 16) | `MATCHED*` | `NARROW(fields)` | `Scan::ty` vs the TYPE word — the shared primitive |
| `27 30 32 40 41 55 64 9a` (arms 9, 13, 14, 24, 25, 35, 42, 53) | `MATCHED*` | `NARROW(fields)` | same |
| `3c 5c 99 9b b9 bd` (arms 21, 38, 52, 54, 58, 23) | `MATCHED*` | `NARROW(fields)` | same, in a multi-field production |
| **`0x2c`** (arm 12) | `UNRESOLVED` (§6.2) | **`WIDE(fields)`** | class `05` reads **one raw `GetByte`**; `Scan::vint` takes **5** bytes at the payload byte `0x80`. `#1592`'s latent desync, re-derived and sharpened: the trigger is **exactly `0x80`**, not `≥ 0x80` |
| **`0x54`** (arm 34) | `UNRESOLVED` (§6.2) | `NARROW(fields)` | class `0D` is an **`i32c`**; the port reads a fixed byte. `IL_STMT_GRAMMAR.md` §12.1's *"byte-vs-varint UNKNOWN"* is resolved |
| **`0x43`** (arm 0) | `MATCHED*` | **`WIDE(fields)`** | class `00` is payload-free — the port advances 4 or 2 where c2 advances 1. §10.4 |
| **`0x28`** (arm 10) | `NARROW(fields)` (§6.1) | **`NARROW(fields)` — CONFIRMED** | class `02` calls `varU`; the port's `28 00 00` accepts exactly one of its values, and **refuses**, which is the right direction |
| `0x66` (arm 43) | `MATCHED*` | `NARROW(fields)` | class `1A` reads the arity as an **`i32c`**, whose short form is **signed**; `eat_class_descriptor` reads one **unsigned** byte |
| `0x4f` (arm 32) | `MATCHED*` | `NARROW(fields)` | class `0C` is `i16c` + a format-string interpreter over 64 field codes (`ref/P_SUB4F.md`); the port recognises four fixed `4F` shapes and ends the walk. Fail-closed |
| **`0x33`** (arm 15) | `MATCHED*` | **`UNRESOLVED`** | widths agree on every branch; the **discriminators are different fields** — the port tests the raw `tag`/`kind`, c2 tests the **lowered** word `node[+4]`. Needs `FUN_10b3d40a` (`0x10b3d40a`) |

The 35 that stay `MATCHED` are the payload-free class-`00` set, the `varU`
tokens (`26 29 38 39 3a 3b 3d`, where `readers::read_token_var` is c2's `varU`
bit for bit), the `i32c` pairs (`5d 5e 67`, where `readers::read_varint` is
c2's `i32c` bit for bit), and `44 46 4b 4c 53`.

## 10.4 §7.2's hazard, MEASURED

§7.2 is **right** — `0x43` is class `00`, there is no escape, and the port's
two witnessed widths fall out by coincidence. It names a hazard and says it was
not graded. It is graded now:

| | count |
|---|--:|
| workload `.ex` streams token-walked (one per source) | 867 |
| bodies walked clean to a tail | 567,367 |
| **`43 42` sites in real token position** | **2,404** |
| … whose `varU` is **wide** (4 bytes) | **0** |
| top-level `0x42` tokens **not** preceded by `0x43` | **0** of 2,404 |

**Not witnessed.** The hazard is real in the language and constructible under
the container model (`varU`'s wide form is `0x10c1f91b`'s own second branch and
`readers::read_token_var` decodes both widths), but no workload site takes it.
And §7.2's *pairing* claim is empirically exact: every `0x42` in the workload
is a `43 42`.

**A second way the fixed `+4` is wrong, which §7.2 does not have.** Class `02`'s
arm tests the global `DAT_10c67fc0` at `0x10b3d64d`; when it is **zero**, opcode
`0x42` alone takes **no operand**, so `43 42` is **two** bytes. The same
constant over-reads by 2 in that environment and by 2 over a wide token — at
opposite ends.

## 10.5 What did NOT change

* **Every limb-1 count in §3 and §4 stands** — `MATCHED*` 68, `NARROW(gate)`
  **0**, `ABSENT` 27, and the 20 arms with no port site. This section decides a
  different limb and re-derives none of them.
* **§0's warning stands and is now sharper.** 68 of 95 was *"a statement about
  width"*; 35 of 68 are width-**exact** and 33 are not, and **no** port site
  still mints an IR node in the `≥ 0x2af` space.
* **§5.1's `0x2d` finding stands**, untouched.
