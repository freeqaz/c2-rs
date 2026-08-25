# w-keymap — PREREG

    Lane:      w-keymap (wave 10 PRIMARY, board #3509, decision 12 item 1)
    Kind:      characterization. Fixtures: none. Census: +0. Docs-only,
               ZERO `crates/` bytes. Reading `crates/` is unfenced; writing
               is forbidden and a needed write is a STOP-AND-REPORT.
    Date:      2026-08-25
    Worktree:  `.claude/worktrees/w-keymap`, branch `wt-w-keymap`,
               off master `a8593651b`.
    Board:     #3528–#3533 reserved. Minted in the commit that uses them.
    Rung file: `docs/rungs/2026-08-25-w-keymap.md`.

**Committed as this lane's FIRST commit, before the first measurement.**
Predictions below are never edited afterwards. Navigation pointers may be
repaired by amending beside, marked as such.

---

## 0. What this lane owes

`#3509`: **map Phase 1's ten constructs to census keys, and publish each
one's TU-denominated reach.** Phase 1 = `ROADMAP_SLICING_2026-08-21.md` §5:

> C1 off-add · C2 intrinsic · C3 bind · C4 load-type · C5 temp · C6 lit-type ·
> C7 compare · C8 bitwise · C9 materialize-64 · C10 virtual-slot

Three hard rules carried from the brief, each of which has cost a lane:

1. **No ranking.** `#3505`, bound five times. The output is a mapping with
   denominators, ordered **C1…C10** and, within a construct, by **key name**.
   `docs/data/census_key_populations.tsv` is name-sorted for this reason and
   that discipline is preserved.
2. **Name what cannot be keyed.** A read that keys 6 of 10 and declares the
   other 4 unkeyable is a SUCCESS; keying 10 of 10 by inference is a failure
   nobody catches for weeks (`w-joint3` declined exactly this).
3. **Every number carries its scope in the same breath**, in the form
   `slot/body denominator N · family denominator M · could touch X of Y = Z%`.
   A first-blocker count is **not a distance and not a reach** (`#3131`).

---

## 1. What was ALREADY READ before this prereg was written

Registered as **read, not predicted**, so that §5's scoring cannot claim
credit for them. Each is a citation into the tree at `a8593651b`.

**R-A. The ten names are `Scan::off_class` REASON strings, not census keys.**
`docs/rungs/2026-08-08-w-instr.md` §4.1 is the origin of the ten-way
decomposition `ROADMAP_SLICING` §3 quotes: `Scan::off_class(why)` takes a
`&'static str` reason at 21 sites in
`crates/c2-il/src/func/body/shapes/control_flow.rs`, and its published table
(`off-add` · `load-type` · `intrinsic` · `bind` · `lit-type` · `deref` ·
`compare` · `store-type` · `div-mod` · `temp` · `materialize-64` ·
`virtual-slot` · `rmw` · `convert-out-of-class` · `logical` · `bitwise` ·
`shift` · `ternary` · `subscript` · `eh-trailer` · `call-cc`) is where every
one of the ten names comes from. **21 reasons exist; Phase 1 took ten of
them.** The reasons live in `TuResult::fn_cflow_off`, a **different map**
from `TuResult::fn_blockers`, which is what the census keys live in. That is
the mechanical reason nothing in the tree converts bodies to TUs.

**R-B. The opcode for each of the ten, from the arm that raises its reason**
(`control_flow.rs`, `fn operand`):

| slice | reason string | opcode / predicate | site |
|---|---|---|---|
| C1 off-add | `off-add` | `0x27 <TYPE>` | `control_flow.rs:941` |
| C2 intrinsic | `intrinsic` | `0x40 <TYPE>` | `:1057` |
| C3 bind | `bind` | `0x99 <TYPE> <varint>` | `:1168` |
| C4 load-type | `load-type` | `0xB9 <tok> <TYPE>`, TYPE outside int4/ptr4 | `:823` |
| C5 temp | `temp` | `0x9B <TYPE> <tok>` | `:1174` |
| C6 lit-type | `lit-type` | `0x33 <TYPE> …`, TYPE outside int4/ptr4 | `:833` |
| C7 compare | `compare` | `0x1F 0x20 0x21 0x22 0x23 0x24` | `:885` |
| C8 bitwise | `bitwise` | `0x0B 0x0C 0x0D 0x0E` | `:885` |
| C9 materialize-64 | `materialize-64` | `0x64 <TYPE>` | `:1157` |
| C10 virtual-slot | `virtual-slot` | `0x67 <varint slot> <tok>` | `:1114` |

**R-C. `ROADMAP_SLICING` §3's C3 opcode set is WRONG against the instrument
that produced C3's own mass number.** §3 writes C3 as `bind (0x99/9A/9B)`.
In `control_flow.rs` the three opcodes raise **three different reasons**:
`0x99` → `bind`, `0x9A` → `vbind` (`:1132`), `0x9B` → `temp` (`:1174`). So
`0x9B` is **C5**, not part of C3, and `0x9A`/`vbind` is **not one of the ten
constructs at all**. Pooling them under C3 double-counts C5 and silently
imports an eleventh construct.

**R-D. `materialize-64` has a same-named sibling that is NOT in the ten.**
`0x44` raises `materialize-44` (`:1027`) and is separately documented as
meaning-unknown, width-known. C9 is `0x64` and only `0x64`.

**R-E. `expr-op-0x32` — `ChecksumData_xbox.cpp`'s single surviving key, the
one TU in the whole workload at a floor of 1 (`#3507`) — is OUTSIDE Phase 1
by construction.** `control_flow.rs:912-922` raises `0x32`/`0x41`/`0x55`
under reason **`store-type`**, and `store-type` is not one of the ten. The
one TU Phase 1 would most need is blocked by a construct Phase 1 does not
contain.

**R-F. The census key spelling rules** (`Block::feature`,
`crates/c2-il/src/func/body/mod.rs:1607`, and `Complete::name` at `:1540`):
`opt-mode-*` · `{ctx}-{intrinsic_name}` for `expr-intrinsic`/`call-intrinsic`
· `expr-op-0xNN-{tag}{kind}` for the typed div/mod refinement ·
`mcall::feature` for `call-in-expr` · `{ctx}-{tag}{kind}` when `aux != 0`
(the whole `*-type-*` family) · `{ctx}:eof` / `{ctx}:mid` for byte-less
refusals · `expr-{named}` or `expr-op-0xNN` in ctx `expr` ·
`{ctx}-cflow-{name}` for control-flow bytes · `{ctx}-0xNN` otherwise.
**So one construct's opcode appears under many `ctx` prefixes**, and the
mapping is a key SET per construct, never a single key.

**R-G. `expr_opcode_name` (`mod.rs:1816`) names C7 entirely and C8
partially.** `0x1F..0x24` → `cmp-eq/ne/le/lt/ge/gt`; `0x0B/0x0C/0x0D` →
`bit-and/bit-or/bit-xor`; **`0x0E` has no name**, so C8's fourth opcode
renders as a hex bucket while its other three render as named ones.

---

## 2. Denominators as I believe them NOW, before measuring

Every one is **re-measured on this tree** in §3 and the filed value is
recorded beside the fresh one. Filed figures rot (`../dc3-decomp` took four
distinct values in one day) and **the coordinator verified none of them**.

| # | denominator | filed value | filed source |
|---|---|--:|---|
| D-1 | TUs in the dc3 workload | **878** | `work/dc3-workload/files.txt` |
| D-2 | non-matching TUs | **845** | `#3508` |
| D-3 | `match` | **25** | `#3508` cross-tab |
| D-4 | blocked bodies, census route | **1,710,066** | `census_key_populations.tsv` totals |
| D-5 | census slot denominator | **2,417,794** | same |
| D-6 | distinct blocker keys | **785** | same |
| D-7 | per-TU construct floor: median | **186** | `#3508` |
| D-8 | floor > 50 | **821 of 845 (97.2 %)** | `#3508` |
| D-9 | floor == 1 | **1 TU** (`ChecksumData_xbox.cpp`) | `#3507` |
| D-10 | workload stamp | `15a64d92f197` clean | `census_key_populations.tsv` header |

Read at prereg time on this tree: `../dc3-decomp` HEAD **`15a64d92f`**,
`git status --porcelain` **0 lines**. `work/dc3-workload/files.txt` **878**
lines. Both re-read after the scan; a move VOIDS the run.

---

## 3. Method — a READ plus one scan, no probe

1. **The read** (done in §1, extended in the rung): for each of the ten,
   name the opcode/predicate from `control_flow.rs`, then find the key(s)
   the SAME opcode/predicate produces on the **census route**
   (`IlBundle::census_functions`, the `4C 4F 11` splitter) by tracing
   `Block::feature`'s renderers. Cite a file:line for every row.
2. **One scan**: `c2rs gap --list work/dc3-workload/files.txt
   --flags-file … --cwd ../dc3-decomp --jsonl work/w-keymap/scan.jsonl`,
   giving per-TU `fn_blockers` maps. This is the same seam `w-joint3`'s
   route 2 used. No probe grid, no fitted search, nothing compiled that the
   scan does not already compile.
3. **The join**: per TU, the head-key SET over its blocked bodies (=
   `#3508`'s floor). Report per construct: bodies it heads, TUs it appears
   in, and — the number `#3509` actually asks for — **TUs whose entire floor
   set is inside the construct**, and **inside the union of all ten**.
4. **Controls**, each printed and each able to be loud:
   - **K1** total blocked bodies re-derived from the scan must equal the sum
     over `fn_blockers`; a mismatch VOIDS.
   - **K2** the key vocabulary from the fresh scan is diffed against the 785
     of `census_key_populations.tsv`; any difference is printed, not folded.
   - **K3** workload stamp read before and after; a move VOIDS.
   - **K4** the "TUs entirely inside Phase 1" count must be computed the
     same way for a **positive control** — a key set trivially known to
     contain a TU's floor (the TU's own floor) must return that TU — so a
     reported 0 cannot be confused with an instrument that only prints 0.
     This is `w-joint3`'s K5 in this lane's units, and it is required.
   - **K5** every published count names its denominator in the same
     sentence.

---

## 4. PREDICTIONS — frozen, never edited

| id | prediction | p |
|---|---|--:|
| **K-1** | **Of the ten, the number I can key to ≥1 census key with a cited decode site is exactly 8.** | 0.45 |
| **K-1b** | …and it is **≥ 7**. | 0.80 |
| **K-1c** | …and it is **< 10**, i.e. at least one of the ten is declared UNKEYABLE. | 0.75 |
| **K-2** | **Of the 845 non-matching TUs, the number whose ENTIRE floor set lies inside the union of Phase 1's ten key sets is ZERO.** This is the lane's headline prediction and the one that decides Phase 1. | 0.90 |
| **K-3** | Nor does relaxing it help: **allowing every `complete-*` reading and every `:eof`/`:mid` byte-less key as free, the count is still 0.** | 0.85 |
| **K-4** | On the **body** denominator, the union of the ten heads **≥ 50 %** of the 1,710,066 blocked bodies. (Mass is not reach — registered precisely so the two can be seen to diverge.) | 0.60 |
| **K-5** | Even after deleting every Phase-1 key from every TU's floor, the **median residual floor over the 845 stays ≥ 100**. | 0.70 |
| **K-6** | **At least one of the ten heads ZERO census keys** — its opcode never appears as a first blocker on the census route at all. | 0.45 |
| **K-7** | The single largest TU-count key inside Phase 1's union appears in **> 800 of 878** TUs — i.e. Phase-1 keys are near-universal per-TU and still convert nothing, which is the shape that makes mass and reach come apart. | 0.65 |
| **K-8** | `ChecksumData_xbox.cpp` — the one TU at floor 1 — is **not** converted by Phase 1 (follows from R-E, but registered so it is scored rather than assumed). | 0.95 |

**Direction I expect to be wrong in, registered in advance:** `w-joint3`'s
D1/D4 both missed in the same direction (the distance instrument *overstates*
nearness). If I miss, I expect to miss by finding Phase 1 covers **less** than
K-4 says, because the ten reasons were counted on the **cf-scan** population
(bodies the control-flow walker decoded end to end) and the census route
refuses earlier and on different vocabulary.

---

## 5. DECLINE FLOOR — stated before the measurement

This lane reports **`FAILED`**, in that word, if any of:

- fewer than **5 of 10** constructs can be keyed with a cited decode site
  (a mapping that thin is not the artifact `#3509` funds, and publishing a
  guessed remainder is exactly the failure `w-joint3` refused);
- the scan does not complete on a nonzero denominator, or K1/K3 VOID;
- **K4's positive control does not fire** — a 0 from an instrument that
  cannot print anything else is not a measurement;
- the mapping cannot be expressed without inventing a key→slice edge that no
  file:line supports.

It reports **`instrument`** if the mapping and its TU-denominated reach are
published with controls green, whichever way the numbers fall. A **priced
decline of Phase 1** is an explicitly good outcome and is NOT a `FAILED`.

**Not in scope, named so absence is not read as coverage:** ruling on Phase 1
(the owner's, decision 12); dispatching any slice; `#3510`'s diagnosis;
regenerating `STATUS.md`; any `crates/` byte; any successor-key claim (what a
key moves to when closed — `#150`/`#3506` territory, and this lane varies
nothing).
