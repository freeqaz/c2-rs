# PREREG — lane `w-align16`, board #1120: ALIGN_16, or a registered decline

    Lane:    w-align16 (`wt-w-align16`), branched at master `bcd0b3be`
    Board:   #1147–#1156 reserved
    Rung:    docs/rungs/2026-08-08-w-align16.md
    Written: BEFORE the first `cl.exe` on any cell of this lane, before any
             `capture` / `diff` / `gap` / `census` run on this tree, and before
             one line of `crates/` was edited.

This file is committed **before** `work/w-align16/cells/SHA256SUMS`, and both are
committed before anything is compiled.

---

## §0 What I inherit, stated so a later reader can check I did not move it

`w-align` (merge `bcd0b3be`, rung `docs/rungs/2026-08-08-w-align.md`) established
and this lane does **not** re-litigate:

* **`TAG_WIDE` (`0x40`) marks only the mark byte's presence and is orthogonal to
  the width field, so alignment = `tag & !0x40`.** 21 of 21 named object records
  confirmed against **c2's own obj**, 0 contradicted, at `/GR /O1 /Oi /EHsc`.
* `82`→1, `84`→2, `86`→4, `88`→8; `8A`/`CA` read as **16** and are **REFUSED**.
* **`__declspec(align(N))` MOVES the tag** (`w-align` P5, registered at even
  odds, and it held). The width field tracks the type's *required* alignment,
  not its natural layout. A reader that inferred natural alignment would emit
  ALIGN_4 where c2 emits ALIGN_8 on `T16`, **and `T16` converts** — a live wrong
  emit, not a harmless refusal. **Every alignment this lane emits is read off
  the tag. None is inferred from the type.**
* The mark byte's **value** is required to be `0x81`; `0x84` exists in `.ex` and
  its meaning is unknown. That narrowing stays.

## §1 What `8A` / `CA` mean, registered before measuring them again

`8A` is the non-wide width-16 tag; `CA` is `8A` with the wide bit, i.e. with one
mark byte before the kind. `w-align` saw only `CA` (cells `T09`, `G04`) because
both of its 16-aligned cells were `__declspec(align(16))` — which is exactly the
case its §3 co-occurrence says goes wide. **`8A` bare has never been observed by
this project.** This lane's `A05` (a type that is 16-aligned because a *member*
is, with no attribute on the outer type) and `A15` (`__vector4`, if the compiler
has it) exist to find out whether the bare form occurs at all.

**If `8A` bare is never produced, the table entry for it is an extrapolation and
must be labelled as one**, not quietly shipped as "measured".

## §2 The three call sites: identical change, or different?

Read from the tree at `bcd0b3be` **before compiling anything**. The promotion
table is `crates/c2-core/src/coff/container.rs::placement_align`, and there are
**three functions in `crates/c2-core/src/coff/` that share it**:

| # | function | file | what 16 costs it |
|---:|---|---|---|
| 1 | `placement_align(n, natural)` | `container.rs:202` | one arm in the `match natural.max(implied)` guard: `16` joins `1\|2\|4\|8`. The `implied` clause is **untouched** — see P8 |
| 2 | `align_nibble(n, natural)` | `container.rs:172` | one arm: `16 => Some(5)` |
| 3 | `section_nibble(objs)` | `data.rs:155` | one arm: `16 => Some(5)`. **A separate body from `align_nibble` with the same log2+1 table** — this is the duplicate the brief's "three functions sharing one promotion table" is counting, and it is a real duplicate, not a re-export |

Two further consumers are **transitively** affected and take no textual change,
which is the part a diff review will miss:

* `data.rs::bump_layout` — `cursor.checked_next_multiple_of(align)` starts
  rounding to 16. **This is an extrapolation of Rule A3′ past every cell it was
  fitted on** (`OBJ_DATA_BSS_SHAPE.md` §5.7 measured 1/2/4/8 only). It is the
  direction I expect to lose on (P16).
* `data.rs:226` — `emit_data_obj`'s class check stops refusing 16-aligned
  objects, so TUs that are `codegen-gap` today become emit attempts.

And **one reader**, outside `c2-core`, in this lane's other seam:
`crates/c2-il/src/func/gl.rs::align_of_type_tag` gains `0x8A => Some(16)`.

**They are not identical changes**: 1 is a guard, 2 and 3 are two copies of one
log2 table, and the two transitive sites change behaviour with no textual edit.
Registered here so that "it was one arm" cannot be claimed afterwards.

## §3 Is 16 the ceiling? Registered as a question, not an assumption

`w-align` measured `CA` and stopped. **This lane does not assume 16 is the top.**
`A09`/`A10`/`A18` are `__declspec(align(32))` and `align(64)` cells. Under the
`0x80 + 2*(log2(size)+1)` encoding of `IL_TYPE_TAGS.md` §1 the tags would be
`8C`/`CC` and `8E`/`CE`, and the `Characteristics` nibbles would be 6 and 7.

**Whatever they read as, they are REFUSED by this lane.** Extending the table by
log2 to a value no cell confirms is precisely the "mostly right" failure the
decline floor forbids. The report says what they measured as and that they stay
shut.

## §4 The decline floor, registered AGAINST THE INCUMBENT

The incumbent is a refusal, and **a refusal is right 100 % of the time on what
it refuses**. So a promotion table that is *mostly* right is strictly worse than
today's tree. The bar is not "an improvement", it is:

* **F1 — zero disagreements against c2's obj.** Every cell's tag read off `.gl`
  and every cell's alignment read off **c2's own obj** `Characteristics` nibble.
  **One contradiction anywhere and the lane DECLINES**, including on the control
  cells that are not about 16.
* **F2 — ≥ 1 obj byte-exact through `data_tu` AND ≥ 1 through `dyninit_tu`,
  at the workload's `/GR /O1 /Oi /EHsc` and at ≥ 1 other profile.** These are
  the two consumers `w-align` found (its §5 correction 2: pricing through
  `emit_data_obj` alone misses `dyninit_tu`). If only one moves, the *other*
  path's arm ships unproven — **DECLINE that arm and say which**.
* **F3 — if the size-implied clause turns out to reach 16 at some `n`** (P8
  lost), the `implied` table is wrong above its fitted range and this lane
  **widens only the `natural` axis and declines the size axis**, publishing the
  threshold it found.
* **F4 — nothing above 16 is accepted**, whatever it measures as (§3).
* **F5 — `mismatch` 0 everywhere, `fnbyte-exact` not smaller, `differs` not
  larger, `reloc-differs` 861 not larger, `match-tu-differs` /
  `match-tu-reloc-differs` 0, `IlBundle::functions()` not widened.**
* **F6 — `DATA_ATTR = 0xA0` and the `00 04` read-only frame stay failing
  closed.** If either starts to look takeable, **stop and report**, do not take
  it (#1109, #232, `w-rdata3`).
* **F7 — two instruments agree cell by cell** before the crate ships a reading.
  `crates/c2-il/tests/in_init_probe.rs` is extended (it carries `w-align`'s
  standing reading); the second is crate-free Python. No third probe.

### §4.1 The direction I expect to lose on

**The allocator, not the reader.** The reading (`tag & !0x40` → 16) is a
one-value extension of a table confirmed 21/21; I put that at 0.90. What has
*never* been measured is what happens **downstream** of a 16 — `bump_layout`
rounding a `.bss` cursor to 16, and `section_nibble` taking a max over a mixed
section. Rule A3′ was fitted on 1/2/4/8 and every real workload section it scored
on had `align ≤ 8`. If this lane fails, I expect it to fail on `A13` (two objects,
one 16-aligned) or on the **size-implied ceiling** (`A07`, `char[4096]`), and
**not** on `A02`.

Secondary: I expect `8A` **bare** may not exist at all, in which case that arm is
an extrapolation and is labelled as one (§1).

## §5 Predictions

| # | prediction | conf |
|---:|---|---:|
| **P1** | `A02` `__declspec(align(16)) struct A{int a;}; A g;` spells a 16 tag (`8A` or `CA`) and c2's obj gives nibble **5** | 0.90 |
| **P2** | it goes **wide** (`CA`), following `w-align` §3's alignment co-occurrence | 0.70 |
| **P3** | `A01` scalar `__declspec(align(16)) int g;` — **size 4, align 16** — spells a 16 tag and c2 gives nibble 5. Kills "the tag is the size" at 16 | 0.85 |
| **P4** | `A14` polymorphic align(16) spells `CA` (reproduces `w-align` `T09`) | 0.90 |
| **P5** | `A03` empty class align(16): `sizeof` **16**, 16 tag | 0.80 |
| **P6** | `A04` array `A[4]` of a 16-aligned type: **size 64**, still a 16 tag — alignment, not size | 0.85 |
| **P7** | `A05` natural-16 (an `align(16)` *member*, no attribute on the outer) is **also** 16 in the tag — the rule generalizes past the attribute | 0.75 |
| **P8** | the size-implied clause **caps at 8**: `A07` `char[4096]` and `A08` `char[256]` read tag `82` and c2 gives nibble **4**, not 5. *The one I most expect to lose* | 0.75 |
| **P9** | `A09` align(32) spells `8C`/`CC` and c2 gives nibble 6; **refused either way** | 0.60 / **1.00** |
| **P10** | `A06` control — size 16, natural align 4 — spells `86` and c2 gives nibble **3**. The discriminating pair with `A02`: same size, different alignment | 0.90 |
| **P11** | **`factor-c` 169 → 169**, from a scan | 0.90 |
| **P12** | **zero workload TUs convert**, scan `match 10 → 10`, all 139 `gap-metric` lines byte-identical. *This is one item of nine on `w-rdata3`'s factor-C checklist and C is necessary, not sufficient — no movement is the expected outcome, not a miss* | 0.85 |
| **P13** | ≥ 1 cell byte-exact through `data_tu` **and** ≥ 1 through `dyninit_tu` (F2), else DECLINE the unproven arm | 0.60 |
| **P14** | `A13` (two `.bss` objects, one 16-aligned) converts byte-exact — the allocator extrapolation. *The second one I expect to lose* | 0.55 |
| **P15** | **no tag above 16 occurs anywhere in the 878-TU workload's `.gl`**, and `8A`/`CA` itself occurs in **0** of them | 0.70 |
| **P16** | `fixtures/cpp/walign_dyninit_align16.cpp` — `w-align`'s **graded refusal** — turns into a **byte-exact match**, not a mismatch | 0.65 |
| **P17** | `0xA0` and the `00 04` frame untouched and still failing closed | invariant |
| **P18** | four alarms 0; `IlBundle::functions()` not widened; `crates/c2-core/src/codegen/` untouched | invariant |
| **P19** | `work/w-splice/peerkeys.py` — 0 vanished families at both ends | 0.95 |
| **aggregate** | **I expect at least one of P1–P10 to lose.** If every one hits, the grid did not vary structure enough and I say so | — |

## §6 Method, registered

* **Profile: the workload's own `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`**
  (board #1112 — at `/Ox` a refusal on this checklist reads as *paid* that is
  genuinely unpaid at the workload's flags). `c2rs census` and `c2rs diff` take
  **no** `--flags-file`; `c2rs gap` and `c2rs prefilter` do. **Every grade in
  this lane goes through `c2rs gap --flags-file`**, and the profile is printed
  with every number. Graded at four profiles: the workload's, `/Ox /GS- /c`,
  `/O2 /GS- /c`, `/Od /GS- /c`.
* **One directory per cell** (#1045).
* **Real `c2.dll` under wibo + byte-exact obj compare** (TimeDateStamp 4..8
  zeroed) is the sole judge. Outside what these cells prove, **refuse**, and
  publish the refusal with a count.
* Cells frozen by `sha256sum` in `work/w-align16/cells/SHA256SUMS`, committed
  before the first `cl.exe`, and re-verified at the tip.
* **Grep for an existing reader before adding one** — done for `placement_align`
  / `align_nibble` / `ALIGN_16` / `align(16)` across `docs/`, `crates/`,
  `scripts/`, and `docs/BOARD.md` rows searched separately by topic. The hits are
  the five files §2 names plus `docs/ABI_EDGES.md`,
  `docs/OBJ_DATA_BSS_SHAPE.md`, `docs/rungs/2026-08-04-w-small.md` and
  `scripts/sweep.d/64-data-only-tu.py`, all read.

## §7 Ownership

This lane owns `crates/c2-core/src/coff/`'s alignment promotion and the `.gl`
alignment reader in `crates/c2-il`. It does **not** touch
`crates/c2-core/src/codegen/` (peer lane), `scripts/sweep.d/` or
`scripts/expr_sweep.sh` (peer lane `w-gen`).
