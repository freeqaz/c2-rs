# PREREG — lane `w-ildecode`: the opaque middle's two interfaces

Registered **before** any confirming run against the live tap or a real obj.
Committed on branch `wt-w-ildecode`, base master `72207b86f`.
Authority: `docs/ARCH_REVIEW_2026-08-21.md` finding 3; brief
`work/coordinator/ildecode-brief.md`. Board **#3357**–**#3360**.
Findings land in [`WB_MIDDLE_INTERFACES.md`](WB_MIDDLE_INTERFACES.md); this file
is scored there and is never edited after the first confirming run.

**Lane kind: characterization. Predicted reach 0 TUs.** Required-zero on
match 26 / mismatch 0 / fnbyte-exact 35894.

## What was already read before this file was written

The reading is disassembly of `c2.dll`
`sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
plus one `c2rs stage snap` run on `fixtures/cpp/mvp_add3.cpp` and one
`scripts/gt_dump.py` of its obj. **Both of those ran before this file was
written**, so predictions P0.x and P2.2/P2.3 below are *not* blind — they are
registered as **post-dictions on one fixture, to be tested on fixtures this lane
has not snapped**, and that is stated here rather than dressed up. Everything
marked BLIND was written with no run of any kind behind it.

The worked fixture is `fixtures/cpp/mvp_add3.cpp`
(`int add3(int a,int b,int c){return a+b+c;}`), which the port reproduces
**byte-exact** (`c2rs diff` → `Port=Match`) at `/Ox /GS- /c`.

---

## Interface 0 — the opcode-number space

| # | prediction | blind? |
|---|---|---|
| **P0.1** | Every tuple whose flag byte `+0x9` has bit 0 set carries at `+0x4` an opcode `≤ 0x294`, i.e. an index into c2's own machine mnemonic table at `0x10b1b260` (stride 12, `[+0] char* name`), whose `_last` sentinel is index `0x295`. | no (1 fixture) |
| **P0.2** | Every tuple whose `+0x9` bit 0 is **clear** carries an opcode **above** that table (`> 0x297`) — a structural/pseudo opcode with no mnemonic. | no (1 fixture) |
| **P0.3** | P0.1 and P0.2 hold on **≥ 4 further fixtures not yet snapped**, spanning at least `lis`/`lwz`/`stw`/`bl`, with **zero** counterexamples. | **BLIND** |
| **P0.4** | For every real-instruction tuple, the mnemonic named by `0x10b1b260[opcode]` is the mnemonic of the `.text` word at the same position in the same function. | **BLIND** |

## Interface 1 — IL record → tuple

Field correspondence, on `add3`'s `.ex` body
`… 4C 4F 11 53 | B9 e3 09 86 41 74 | B9 e4 09 86 41 74 | 02 | B9 e5 09 86 41 74 | 02 | 41 86 41 74 | 3A e7 09 | 54 02 | 29 e7 09 | …`

| # | prediction | blind? |
|---|---|---|
| **P1.1** | The two payload-free `0x02` tokens (operand class `00`, `DAT_10b25e48`) become **exactly two** tuples with opcode `0x001` = `add`. Count equality 2 = 2. | no (1 fixture) |
| **P1.2** | The three `B9 <varU sym> <TYPE>` parameter loads become **zero** machine tuples. | no (1 fixture) |
| **P1.3** | The `41 <TYPE>` token becomes exactly one tuple, and that tuple's opcode at `sched1` is **not** a machine opcode (it is `> 0x297`), while at `sched0` — after the lowering band — the corresponding real-instruction tuple is `0x284` = `ret`. | no (1 fixture) |
| **P1.4** | **The tuple byte at `+0xa`, masked `& 0x1f`, is the operand SIZE IN BYTES, not a condition code.** It is derived from the IL TYPE word's size index `(v >> 9) & 7` (1→1, 2→2, 3→4, 4→8; `WB_READER_FINDINGS.md` §3.2 step 7). On `add3` (`int`, TYPE word `0x641`, size index 3) it is `4`. | no (1 fixture) |
| **P1.5** | P1.4's *discriminating* test: on a fixture whose arithmetic is `double`, at least one real-instruction tuple shows `+0xa & 0x1f == 8`; on a fixture whose arithmetic is `short`, at least one shows `2`. A condition-code reading predicts neither. **If both come back `4` or `0`, P1.4 is REFUTED** and the tap's own comment (`stagetap.c:299`) and `WB_DAGORDER_FINDINGS.md` §2 stand. | **BLIND** |
| **P1.6** | The number of IL binary-arithmetic tokens of class `00` in a body is **not** in general the number of machine tuples — i.e. P1.1 is a coincidence of the one-to-one shape and will fail on a fixture with a constant operand (where c2 emits `addi`, opcode `0x00b`, and the IL still shows one arithmetic token). Registered as a **pessimistic** prediction about my own subset. | **BLIND** |

## Interface 2 — final tuple order → COFF `.text`

| # | prediction | blind? |
|---|---|---|
| **P2.1** | The `sched0`-entry tuple list, restricted to `+0x9` bit 0 set, is in the **same order** as the emitted `.text` words of that function, and has the same length. (`sched0` is the last schedule and the tap observes its *input*; if `sched0` reorders, this fails and that failure is the finding `ARCH_REVIEW` §1 asked for.) | no (1 fixture) |
| **P2.2** | The emitted 32-bit word for a real-instruction tuple is `base_word[opcode] \| <operand fields per the encode form>`, where `base_word` is the stride-4 table at **`0x10c3a578`** and the form index is the stride-4 table at **`0x10c39b18`**, both read by the encoder `FUN_10bf9f15` @ `0x10bf9f15` (`code.c`). Concretely on `add3`: `word & 0xFC0007FF == 0x7C000214` for both `add`s. | no (1 fixture) |
| **P2.3** | `ret` (`0x284`) and `blr` (`0x285`) share base word `0x4C000020`, and the emitted word for the `ret` tuple is `0x4E800020` = base `\| (BO=20) << 21` — i.e. **the `BO` bits are contributed by the encode form, not by the base word**. | no (1 fixture) |
| **P2.4** | On **≥ 4 further fixtures not yet snapped**, for every real-instruction tuple in the `sched0` list the emitted word at the same index satisfies `word & mask(form) == base_word[opcode]`, where `mask(form)` blanks only the register/immediate fields that form defines. Zero counterexamples. | **BLIND** |
| **P2.5** | The register numbers are **not** in the tuple record: the tap's 128-byte window is byte-identical across COLOR (`P_DAG.md` §2's 2026-08-20 correction), so no extension of the *tuple* walk can supply the `RT`/`RA`/`RB` fields. They live one and two pointers away (`tuple+0x28`/`+0x2c` → operand, `operand+0x1c` → register descriptor, `descriptor+0x28` → the hardware number). **This lane therefore predicts that a tuple-only lowering check can reproduce the non-register bits of every word and NOT the register bits**, and that is the honest boundary of deliverable 2b. | no |

## Reach and outcome

| # | prediction | blind? |
|---|---|---|
| **P3.1** | 0 TUs converted, 0 fixtures claimed, 0 obj bytes moved. | **BLIND** |
| **P3.2** | `match` stays 26, `mismatch` stays 0, `fnbyte-exact` stays 35894. | **BLIND** |
| **P3.3** | At least one prediction above is REFUTED. (Registered because five of this project's whitebox lanes scored all-hits on the reading and missed on the *inference*; a sheet with no miss is a sheet that was not risky.) | **BLIND** |

## Decline clauses

* If the `sched0` list's real-instruction count does **not** equal the `.text`
  word count on the worked fixture, deliverable 2 ships as the decode half plus
  a written lowering-seam spec, and says **FAILED** for the byte check in those
  words.
* If any tap change moves a single obj byte
  (`stage neutrality` → `stage-tap-obj-differs > 0`), the tap change is
  reverted and the lane reports it.
