# w-align — PREREG

    Lane:    w-align (`wt-w-align`), branched at master `fe114e0e`
    Rung:    board #1110 — `align_of_type_tag(0xC6)`, the one item on
             `w-rdata3`'s nine-item `.rdata$r` checklist that is takeable ALONE
    Written: BEFORE any probe. At the time this file is committed no `c2rs
             capture`, `census`, `diff`, `gap`, `prefilter` or `cl.exe` has run
             on any of this lane's cells. The ONE toolchain invocation that has
             happened on this tree is `scripts/configure_existing_worktree.sh`'s
             own liveness assertion — `c2rs census fixtures/cpp/w5_chain.cpp` →
             `4/4 functions in class` — which is the anti-SKIP gate and touches
             no cell of mine. Declared here rather than omitted.
    Read first (source + docs only, no measurement):
             `docs/rungs/2026-08-08-w-rdata3.md` (whole), `docs/OBJ_RDATA_R_SHAPE.md`
             §8/§8.2, `docs/IL_TYPE_TAGS.md` §1, `docs/IL_TYPE_WIDE_TAG.md` (whole),
             `docs/BOARD.md` rows #1107–#1112, #931, #300–#302, #232, #918, #1045,
             `docs/STATUS.md` traps 0/1/2, and the source:
             `crates/c2-il/src/func/gl.rs` (`data_object_at`, `align_of_type_tag`,
             `gl_data_objects{,_ordered}`), `crates/c2-il/src/func/bundle.rs`
             (`data_tu`, `dyninit_tu`, `shell_only_tu`),
             `crates/c2-core/src/coff/container.rs` (`placement_align`,
             `align_nibble`), `crates/c2-core/src/coff/data.rs`,
             `crates/c2-core/src/coff/dyninit.rs`.
    Task:    read tag `0xC6`'s alignment HONESTLY, or refuse; report what
             `gl_data_objects_ordered` becomes on the probe cell and what
             `factor-c` does FROM A SCAN.

---

## §0 The pre-search the brief requires, and what it found

`grep -ril 'align_of_type_tag\|0xC6\|type tag' docs/` → 13 files. Board searched
separately by topic (`align`, `#1108`–`#1112`, `#931`). **Three artefacts exist
that this lane would otherwise have rebuilt, and all three are consumed, not
redone:**

1. **`crates/c2-il/tests/in_init_probe.rs`** — the standing `.in` instrument.
   Not needed here (this is a `.gl` rung) but it is the reason #1108's refutation
   is credible, and it is NOT re-written.
2. **`work/w-rdata3/p01/glhex.py`** — a read-only `.gl` hexdumper that already
   locates a named record and prints the bytes after its terminating NUL. **This
   lane's crate-free second instrument is built on it**, not beside it.
3. **`docs/IL_TYPE_WIDE_TAG.md`** — the wide-tag rule was *derived* on 2026-07-31
   for `.ex`/`.sy`, with `C6 81 06` / `C6 81 03` / `CA 81 0D` as witnesses. Its §8
   item 2 already registers the residual risk ("the mark byte's meaning is
   UNKNOWN, and so is its value SET"). **This lane does not re-derive the wide
   rule; it asks only what the WIDTH FIELD under it means in `.gl`.**

Nothing in `docs/` or the board records a lane that has already read `0xC6`'s
alignment. #1110 is the only row and it says *"not taken here"*.

---

## §1 The incumbent, and the decline floor registered against it

**The incumbent is `align_of_type_tag` returning `None` on `0xC6`.** It is not a
weak baseline. On the population it refuses it is **right 100 % of the time**,
because refusing produces no obj at all and the port reports
`NotImplemented` — and a wrong alignment nibble is a wrong `Characteristics`
word, i.e. a wrong obj that the differential grades `mismatch`. Board #232 is
the standing record of what the other kind of error costs: a live wrong emit
that sat on master for **255 commits** behind a green scan.

So the bar is not "mostly right". **A reader that is mostly right is strictly
worse than the incumbent.**

### The floor — the arm ships ONLY if all four hold

* **F1 — zero disagreements, not few.** Every frozen cell whose tag the new
  reading ACCEPTS must have its alignment confirmed by **c2's own obj** (the
  `Characteristics` alignment nibble of the section c2 puts that symbol in). One
  disagreement anywhere in the grid and nothing ships in `crates/`.
* **F2 — a graded consumer, byte-exact.** ≥ 1 real obj graded **byte-exact
  against `c2.dll` under wibo** through a path that actually CONSUMES
  `natural_align` (`dyninit_tu` → `coff::dyninit::align_nibble`, or `data_tu` →
  `coff::data::placement_align`). **If no cell can reach a graded consumer at
  all, the arm is unreachable and this lane DECLINES** and publishes the
  reachability price. An accepted record nothing can emit is #232's direction.
* **F3 — two instruments agree.** The production cursor (`data_object_at`, via a
  Rust probe) and a crate-free Python `.gl` parser must agree **cell by cell** on
  the tag byte, the size field and the accept/refuse verdict. Discrepancies get
  explained, not closed.
* **F4 — alarms unchanged.** `mismatch` 0; `fnbyte-exact` does not shrink;
  `differs` does not grow; `match-tu-differs` / `match-tu-reloc-differs` 0;
  `IlBundle::functions()` not widened.

Failing any of F1–F4 the deliverable is the measurement and `crates/` stays
byte-identical to `fe114e0e`.

### The boundary — `DATA_ATTR = 0xA0` is NOT this lane's

Board #1109. `w-rdata3` established all three remaining `.gl` gates mean COMDAT
or read-only, which `emit_data_obj` refuses anyway. **`0xA0` and the `00 04`
read-only frame stay failing closed, and if my work makes `0xA0` look takeable I
stop and report rather than take it.** Registered here so it cannot be
rationalised later.

---

## §2 The hypothesis, stated so it can lose

**H1 (the mask reading).** In a `.gl` DATA record the TYPE tag's bit 6
(`TAG_WIDE`, `0x40`) marks only the presence of the extra mark byte and is
**orthogonal** to the width field `IL_TYPE_TAGS.md` §1 tabulates
(`0x80 + 2*(log2(size)+1)`). So the alignment is read off `tag & !0x40`:

    C2 -> 1    C4 -> 2    C6 -> 4    C8 -> 8    everything else -> None

and in particular `CA` (= wide, width 16) keeps refusing, because
`placement_align` models only 1/2/4/8 and no cell here proves 16.

### The adjacent meanings H1 must beat, and the cell that kills each

| adjacent meaning | cell that refutes it | what it would predict |
|---|---|---|
| the tag is the object's **SIZE** | `T03` (poly + `char`, size 8) and `T04` (poly + `char[64]`, size 68) | `C8` / `~C8E`; H1 predicts **`C6` for both** |
| **`0xC6` is an atomic "wide aggregate" tag**, always align 4 | `T02` (poly + `double`) and `T05` (poly + `long long`) | `C6`; H1 predicts **`C8`** |
| the tag is the **largest member's** width | `T04` (`char[64]` member) | a 64-width tag (`8E`/`CE`); H1 predicts **`C6`** |
| the tag is **natural** alignment, ignoring `__declspec(align)` | **`T08`** — poly + `int`, `__declspec(align(8))` | `C6` while the obj wants ALIGN_8. **This is the trap cell: under that meaning H1 reads 4 and the truth is 8, and H1 must then be narrowed to refuse the declspec case rather than shipped.** |
| the tag saturates at 8 | **`T09`** — poly + `int`, `__declspec(align(16))` | `C8` while the obj wants ALIGN_16, which `placement_align` cannot express. H1 must **refuse**, not return 8 |

**T08 and T09 are why this rung is cells-and-not-a-match-arm.** They are the two
places a plausible one-line widening emits a wrong `Characteristics` word.

---

## §3 The frozen grid — structural axes, not values

18 tag cells (`T*`) + 5 graded cells (`G*`), frozen by
`work/w-align/cells/SHA256SUMS` **committed before the first `cl.exe`**. Axes
varied — *structure*, per the standing rule that a grid varying values
exhaustively and structure not at all has misled this project three times:

* **what makes the type wide** — virtual function (`T01`), virtual destructor
  (`T15`), virtual base (`T14`), and the negative controls that should NOT be
  wide: plain aggregate (`T10`), derived-but-not-polymorphic (`T13`), nested
  aggregate (`T17`).
* **the aligned type's size, held against its alignment** — 4 (`T06`), 8 at
  align 4 (`T01`, `T03`), 16 at align 8 (`T02`, `T05`), 68 at align 4 (`T04`),
  crossing `placement_align`'s own 2/64 promotion thresholds.
* **member kinds** — `int`, `char`, `char[64]`, `double`, `long long`, a nested
  struct.
* **array vs scalar** — `T07` (poly `A[4]`) against `T01`; `T18` (plain `A[4]`)
  against `T10`.
* **explicit `__declspec(align)` vs natural** — `T08`/`T09` on a polymorphic
  type, `T16` on a plain one (which separates "the declspec is in the tag" from
  "the wide bit is polymorphism").
* **scalar controls the incumbent already reads** — `T11` (`double`, tag `88`),
  `T12` (`char`, tag `82`), so a zero anywhere is a reading and not a broken
  probe. This is `w-rdata3` §4's positive-control discipline, reused.

The `G*` cells are `fixtures/cpp/wr1c_dyninit_extern.cpp`'s shape with exactly
one axis changed — the class made polymorphic — so the difference between a
graded and an ungraded cell is one keyword:

| cell | source | size | expected true align | nibble |
|---|---|---:|---:|---:|
| `G01` | poly, 3 `int` members, extern | 16 | 4 | 3 |
| `G02` | as `G01`, `static` | 16 | 4 | 3 |
| `G03` | poly, one `double` member | 16 | **8** | **4** |
| `G04` | poly, `__declspec(align(16))` | ≥16 | 16 | — **must refuse** |
| `G05` | **NOT poly**, one `double` — the incumbent already accepts tag `88` | 8 | 8 | 4 |

**`G05` is the control that says the judge works at all.** If `G05` does not
grade byte-exact today, then the alignment nibble is not what the differential
is comparing and every verdict below is void.

**`G01` against `G03` is the discriminating pair**: identical size (16),
different natural alignment (4 vs 8), and `container.rs`'s own doc says so — *"a
`double` member gives ALIGN_8 at n = 8 where a `char[8]` gives ALIGN_4"*. Two
objs that differ in one nibble, judged by real c2.

---

## §4 Predictions, registered

| | prediction | conf |
|---|---|---:|
| **P1** | `T01` (`?g@@3UA@@A`) spells tag **`C6`**, and c2 gives the object **ALIGN_4** | 0.90 |
| **P2** | `T02`/`T05` (poly + `double` / `long long`) spell **`C8`**, not `C6` — H1's main falsifier | 0.80 |
| **P3** | `T03` (size 8, align 4) and `T04` (size 68, align 4) both spell **`C6`** — the tag is alignment, not size, for WIDE aggregates too | 0.85 |
| **P4** | `T07` (array of 4) spells **`C6`** with size **32** — the tag is the ELEMENT's alignment, the size field the whole array's | 0.70 |
| **P5** | `T08` (`__declspec(align(8))`, naturally 4) spells **`C8`**. **This is the one I most expect to lose**, and if it spells `C6` while c2 emits ALIGN_8 then H1 is REFUTED for declspec types and the shipped arm must additionally refuse them — or ship nothing | 0.50 |
| **P6** | `T09` (`__declspec(align(16))`) spells a tag H1 **refuses** (I predict `CA`) | 0.60 on the value, **1.00 on "must refuse"** |
| **P7** | the non-polymorphic controls `T10`/`T13`/`T16`/`T17`/`T18` are **NOT wide** (bit 6 clear) — i.e. the wide bit is polymorphism, not aggregate-ness | 0.75 |
| **P8** | `T14` (virtual BASE) and `T15` (virtual DESTRUCTOR) are wide | 0.85 |
| **P9** | `gl_data_objects_ordered` on the `?g@@3UA@@A` cell goes **1 of 12 → 2 of 12**, not 12 of 12: the other ten are the `??_R*`/vftable records that `0xA0` and the `00 04` frame still refuse, and this lane does not touch either | 0.75 |
| **P10** | **`factor-c` 169 → 169**, read from a scan on this branch, never asserted | 0.90 |
| **P11** | **878-TU match 10 → 10. THIS LANE CONVERTS ZERO WORKLOAD TUs**, and that is registered up front as the expected outcome, not discovered afterwards. The value of the rung is that it is on the critical path and cheap | 0.85 |
| **P12** | ≥ 1 of `G01`/`G02`/`G03` grades **byte-exact** at ≥ 1 mode lane — i.e. the arm has a real graded consumer. **If this loses, F2 fires and the lane DECLINES** | 0.55 |
| **P13** | `0xA0` and the `00 04` read-only frame are untouched and still refuse | 1.00 |
| **P14** | the four alarms stay 0 and `IlBundle::functions()` is not widened | 1.00 |
| **P15** | `work/w-splice/peerkeys.py` reports **0 vanished families** at both ends | 0.95 |

**The direction I expect to be wrong in.** Toward *"the tag carries less than I
think"* — specifically P5. A width field that tracks `__declspec(align)` is a
strictly stronger claim than one that tracks the natural layout, and the c2 front
end has every reason to spell only the latter. **The dangerous direction is the
other one** — scoring a tag as READ when it is merely PLAUSIBLE — so every
ACCEPT below requires a positive reading from c2's own obj, never the absence of
a disagreement.

**I expect at least one of P1–P8 to lose.** A grid that confirms every cell is
weak evidence the grid varied anything.

---

## §5 What this lane will NOT do

1. **Not touch `crates/c2-core/src/coff/`** or `crates/c2-core/src/codegen/`
   (concurrent lane `w-front2` owns the latter; item 3 of `w-rdata3`'s checklist
   is theirs).
2. **Not widen `DATA_ATTR`** (#1109) or the `00 04` read-only frame.
3. **Not add a name to `PORT_WRITER_SECTIONS`** — `.rdata$r` is not this rung.
4. **Not write a third `.gl` reader.** The production cursor is
   `data_object_at`; the second instrument is a crate-free Python parser derived
   from `work/w-rdata3/p01/glhex.py`, and it exists to DISAGREE with the cursor,
   not to replace it.
5. **Not key anything on `IlFunction::mangled_name`** (#918).
6. **Not price `.rdata$r`.** It has been declined three times; this rung takes
   one checklist item and reports the row.

## §6 The profile

Every capture and every grade at the **workload's own**
`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc` (board #1112 — `census` and
`diff` take no `--flags-file` and default to `/Ox /GS- /c`, at which a refusal
can read as paid that is genuinely unpaid). Where a default-profile number is
quoted it will say so. The mode lanes grade the fixtures at all 18 lanes
regardless, and that is the broader judge.
