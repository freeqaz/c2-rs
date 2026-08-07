#!/usr/bin/env python3
"""addrows.py — insert lane w-align16's board rows at their numeric positions.

`docs/BOARD.md` is hand-maintained (its own header says so) and a row minted
without one is a number `ROADMAP.md` references and nothing lists. This script
exists so the insertion is reviewable and re-runnable rather than a hand edit,
and it **fails hard** if any anchor it needs is missing — a skipped insert is the
failure mode the brief names.
"""
import sys

P = "docs/BOARD.md"
s = open(P, encoding="utf-8").read()

DONE_HDR = "## Done\n\n| # | item | number | where settled |\n|---|---|---|---|\n"
DECL_HDR = ("## Declined and refuted — the rows that saved work\n\n"
            "| # | item | verdict | number | where |\n|---|---|---|---|---|\n")
OPEN_HDR = ("## Open\n\n| # | item | worth (measured, not estimated) | defined | notes |\n"
            "|---|---|---|---|---|\n")
for h in (DONE_HDR, DECL_HDR, OPEN_HDR):
    if h not in s:
        sys.exit("FAIL: anchor missing — refusing to guess:\n%r" % h[:60])
for n in ("1147", "1148", "1149", "1150", "1151", "1152"):
    if "**%s**<sub>w-align16" % n in s:
        sys.exit("FAIL: row #%s already present" % n)

r1147 = (
 "| **1147**<sub>w-align16</sub> | **#1120 IS CLOSED — ALIGN_16 IS READ AND EMITTED, AND THE SIZE-IMPLIED CLAUSE IS PROVEN TO CAP AT 8** | "
 "**DONE, on a 20-cell grid frozen by `sha256` before the first `cl.exe`, varying STRUCTURE and not values.** "
 "Every cell's tag read off `.gl`, every cell's alignment read off **c2's own obj**: **24 of 24** section "
 "`Characteristics` nibbles and **25 of 25** symbol `Value`s agree, **0 contradicted**. "
 "`__declspec(align(16))` on a scalar (size **4**, align 16 — the object smaller than its own alignment), on a "
 "plain aggregate, on an empty class, on an array (size 64), on a polymorphic class, on internal linkage, on an "
 "initialized `.data`, on two objects sharing one `.bss`, **and on a type made 16-aligned by a MEMBER with no "
 "attribute on the outer type** — all spell a 16 tag and all get nibble **5**. "
 "Three functions in `crates/c2-core/src/coff/` share the promotion table and all three moved: `placement_align` "
 "(one guard arm), `align_nibble` (`16 => 5`) and **`data::section_nibble`, which #1120 did not name and which is a "
 "SECOND COPY of the `log2 + 1` table, not a call to the first**. Two more consumers changed with **no textual "
 "edit**: `bump_layout`'s cursor now rounds to 16 (extending Rule A3' past every cell that fitted it — prereg P14 "
 "at 0.55, the registered likelier loss, and it held: cell `A13` puts the 16-aligned object at offset **16** behind "
 "a one-byte `char`) and `emit_data_obj`'s class check. "
 "**The `implied` clause CAPS AT 8 and this is now measured, not assumed**: `char g[4096]` is tag `82` and c2 gives "
 "it nibble **4**. Everything above 8 arrives through the *tag*, never through the size — which is what makes "
 "`w-align`'s trap (`__declspec(align(N))` MOVES the tag) load-bearing at 16 too. "
 "Frozen grid at the workload's `/GR /O1 /Oi /EHsc`: **match 5 -> 14, mismatch 0**, and `mismatch 0` at all four "
 "profiles (`/Ox` 4 -> 11, `/O2` 5 -> 14, `/Od` 4 -> 11). Both consumers `w-align` §5 identified are graded: "
 "`data_tu` (7 cells) **and** `dyninit_tu` (2). `factor-c` **169 -> 169** from a scan; all **139** `gap-metric` "
 "lines byte-identical | "
 "rungs/2026-08-08-w-align16.md §2, §5; `work/w-align16/` (`PREREG.md`, `cells/SHA256SUMS`, `oracle.py`); "
 "`crates/c2-core/src/coff/container.rs`, `.../data.rs`; `crates/c2-il/src/func/gl.rs` `align_of_type_tag`; "
 "`fixtures/cpp/wa16_data_align16.cpp`, `wa16_data_scalar_align16.cpp`, `wa16_bss_two_align16.cpp`, "
 "`wa16_dyninit_plain_align16.cpp` |\n")

r1148 = (
 "| **1148**<sub>w-align16</sub> | **A LIVE WRONG EMIT THAT WAS ALREADY ON MASTER, found by a grid built for something else: an INTERNAL-LINKAGE `.bss` MOVES THE WHOLE SECTION, and Rule S1 does not model it** | "
 "**FOUND and CLOSED FAIL-CLOSED — and it is NOT this lane's regression.** Frozen cell `A11` "
 "(`static A g; A* p = &g;` at align 16) graded **`mismatch`**. It is not an alignment fault: real c2 puts `.bss` "
 "**before both `.XBLD$W` watermarks** when the section holds an internal-linkage object, where Rule S1 puts it "
 "*between* them. The post-hoc diagnostic cells settle the provenance: **`D01` (align 4) and `D02` (align 8) are "
 "inside the INCUMBENT's own modelled range and BOTH grade `mismatch` against real c2 on an UNMODIFIED tree** — "
 "board **#232**'s shape, wrong bytes sitting live under a scan reading `mismatch 0`. "
 "**Why nobody had seen it:** `wsect_drop_static.cpp` records that an uninitialized *unreferenced* static is "
 "dropped entirely, and `wsect_data_linkage.cpp` concluded from that that *\"mixed linkage is unreachable in a "
 "`.bss` of a functionless TU\"*. True of the cells that existed. **The route around the drop is to REFERENCE the "
 "static** — one line of C++, `A* p = &g;` — and nobody had written it. "
 "**The same scope error hits Rule Y1**: every static-`.bss` cell in `OBJ_DATA_BSS_SHAPE.md` is a TU **with "
 "functions** (which is what keeps *its* statics alive), while `emit_data_obj` serves only **functionless** TUs. "
 "On a real functionless mixed-linkage `.bss` (`D07`) c2 emits the EXTERNAL `.bss` symbol **after the following "
 "section's group** with the static at offset 0 — neither Y1's order nor its walk. **Neither rule is refuted; both "
 "were being read past their cells.** "
 "`emit_data_obj` now refuses any `.bss` holding an internal-linkage object, which turns **two pre-existing live "
 "mismatches into honest refusals** and costs zero matches. Fixed by REFUSAL and not by reorder on purpose: the "
 "correct order is a three-cell observation and Rule S1 belongs to #174 with its own grid (#1152) | "
 "rungs/2026-08-08-w-align16.md §4; `work/w-align16/diag/` (`D01`, `D02`, `D07`, own `SHA256SUMS`); "
 "`fixtures/cpp/wa16_bss_static_reloc.cpp` (graded at align **4**, so it does not depend on one byte of #1147); "
 "`docs/OBJ_DATA_BSS_SHAPE.md` §2.2 and §6.2 scope corrections; `crates/c2-core/src/coff/data.rs` |\n")

r1150 = (
 "| **1150**<sub>w-align16</sub> | **THE WORKLOAD CANNOT MOVE ON THIS ITEM — 0 of 85,895 `.gl` DATA records carry an alignment tag at or above 16, and `C8` has zero witnesses either** | "
 "**MEASURED, not inferred from a flat scan.** A census of the `.gl` alignment tag of every ORDINARY-DATA-framed "
 "record across all **878** workload TUs at the workload's own flags "
 "(`work/w-align16/tagcensus.py` -> `tagcensus.txt`, one directory per TU, each removed after it is read): "
 "**85,895 records**, and the whole vocabulary is `82` (216 records / 102 TUs), `84` (877 / 871), "
 "`86` (84,334 / 836), `88` (33 / 16) and `C6` (435 / 153). **`8A`, `CA`, `8C`, `CC`, `8E`, `CE`: zero.** "
 "So #1147 converts the fixture corpus and **cannot** convert one workload TU — *cannot*, by construction, not "
 "*did not*. `match 10 -> 10` was the registered outcome (prereg P12, 0.85) and all 139 `gap-metric` lines are "
 "byte-identical. **Two corollaries worth more than the headline**: (a) even `C8` — `w-align`'s own `T08`/`T16` — "
 "has **zero** workload witnesses, so the wide form appears in this corpus at exactly ONE width, `C6`; (b) the "
 "port's bare-`8A` arm has 0 witnesses **and 0 counter-witnesses**, which is why it ships labelled as an "
 "extrapolation rather than counted among the 24 confirmations (#1151). "
 "**This is one item of nine on `w-rdata3`'s factor-C checklist and C is necessary, not sufficient** — no movement "
 "is the expected result, and it was registered before the first capture | "
 "rungs/2026-08-08-w-align16.md §6; `work/w-align16/tagcensus.txt`; `work/w-align16/PREREG.md` P12/P15 |\n")

r1149 = (
 "| **1149**<sub>w-align16</sub> | **16 IS NOT THE CEILING — `align(32)` and `align(64)` are REAL, and they are refused for the GRID, not for the arithmetic** | "
 "**MEASURED AND DELIBERATELY DECLINED.** `__declspec(align(32))` spells the `.gl` tag **`CC`** (wide, width field "
 "`8C`) and real c2 gives the object `Characteristics` nibble **6**; `align(64)` is **`CE`** and nibble **7**. So "
 "`IL_TYPE_TAGS.md` §1's `0x80 + 2*(log2(size)+1)` encoding and the `log2 + 1` nibble law are both confirmed all "
 "the way to **64** in `.gl`, and this refusal is *not* \"we do not know what c2 does\". It is refused because the "
 "grid stops: at 16 this lane varied structure **nine ways** (scalar, plain aggregate, empty class, array, "
 "polymorphic class, 16-aligned-via-member, internal linkage, initialized `.data`, two objects in one `.bss`); at "
 "32 and 64 it varied **one shape each**, and the untested consumers are the ones that bite — `bump_layout`'s "
 "cursor and `section_nibble`'s max. Extending a table by `log2` to a value three cells confirm and no cell "
 "*constrains* is precisely the \"mostly right\" table the decline floor was written against: **the incumbent "
 "refusal is right 100 % of the time on what it refuses** | "
 "**Price: one more grid of exactly this shape at 32/64, and the same three functions.** Nothing else — the "
 "reader, the writer and the fixture harness all already handle the value; only the evidence is missing. "
 "Instance count on the 878-TU workload: **ZERO**, measured (#1150), so the payoff is fixture-corpus only | "
 "rungs/2026-08-08-w-align16.md §3; `fixtures/cpp/wa16_data_align32.cpp` (the graded refusal, which INHERITED the "
 "guard role from `walign_dyninit_align16.cpp` when #1147 took the value that fixture was guarding); "
 "`work/w-align16/cells/A09_align32.cpp`, `A10_align64.cpp`, `A18_dyninit_align32.cpp` |\n")

r1151 = (
 "| **1151**<sub>w-align16</sub> | **THE PORT SHIPS ONE ALIGNMENT ARM WITH ZERO WITNESSES — bare `8A` is accepted by the ORTHOGONALITY RULE, not by a cell** | "
 "**CARRIED DELIBERATELY, and labelled in the code rather than counted as confirmed.** Every 16-, 32- and "
 "64-aligned cell in the grid spells the **wide** form (`CA`/`CC`/`CE`) — including a bare scalar "
 "(`__declspec(align(16)) int g;`) and a type made 16-aligned through a member rather than the attribute, which "
 "were the two shapes most likely to produce the narrow form. **Bare `8A` has never been produced by this project, "
 "at any of four profiles, and occurs 0 times in 85,895 workload records** (#1150). "
 "`align_of_type_tag` accepts it anyway, because it masks the wide bit off and reads the field underneath — "
 "`w-align`'s orthogonality rule, confirmed 21 of 21. **The alternative is worse**: a split table (both forms at "
 "1/2/4/8, only the wide form at 16) is an inconsistency a later reader would have to rediscover, and a shape "
 "nothing emits cannot mismatch | "
 "**No price — this is a labelling decision, not a gap.** What would settle it is the same probe "
 "`IL_TYPE_WIDE_TAG.md` §8 item 2 asks for: one that moves a type across the wide boundary while holding alignment "
 "fixed. Nothing needs it today | "
 "rungs/2026-08-08-w-align16.md §2.1; `crates/c2-il/src/func/gl.rs` `align_of_type_tag` doc comment; "
 "`docs/IL_TYPE_WIDE_TAG.md` §7.2 |\n")

r1152 = (
 "| **1152**<sub>w-align16</sub> | **THE SECTION AND SYMBOL ORDER FOR AN INTERNAL-LINKAGE `.bss` IN A FUNCTIONLESS TU IS UNKNOWN — three cells, no rule** | "
 "**Worth: unmeasured, and probably small.** The shape needs a `static` uninitialized object kept alive by a "
 "`.data` initializer holding its address, in a TU with no functions. It has **zero** instances on the 878-TU "
 "workload as far as this lane looked, and #1148 closed it as a refusal rather than leaving wrong bytes. The value "
 "is not conversion, it is that **Rule S1 and Rule Y1 currently have a hole in them that is documented as a hole** "
 "| **What is known, from three cells** (`work/w-align16/diag/` `D01`, `D02`, `D07`): `.bss` moves to **before "
 "both `.XBLD$W` watermarks**; with mixed linkage the STATIC takes offset 0 and the EXTERNAL takes 4; and the "
 "EXTERNAL's symbol record is emitted **after the following section's group**, not inside the `.bss` group. "
 "**That is a description of three cells and NOT a rule** — nothing here separates \"internal linkage moves it\" "
 "from \"a `.data` relocation into `.bss` moves it\", because every surviving cell has both | "
 "**Do NOT encode the block above from the doc.** `OBJ_DATA_BSS_SHAPE.md` §2.2 and §6.2 carry it as a SCOPE "
 "CORRECTION with that warning attached. Also open and untested by this lane: whether Y1's mixed row still holds "
 "for a TU **with** functions, which is where it was originally fitted — a writer for that shape must re-measure "
 "rather than inherit. Belongs with board **#174**, whose grid this is | rungs/2026-08-08-w-align16.md §4.2 |\n")

s = s.replace(DONE_HDR, DONE_HDR + r1147 + r1148 + r1150, 1)
s = s.replace(DECL_HDR, DECL_HDR + r1149 + r1151, 1)
s = s.replace(OPEN_HDR, OPEN_HDR + r1152, 1)

for n in ("1147", "1148", "1149", "1150", "1151", "1152"):
    if "**%s**<sub>w-align16" % n not in s:
        sys.exit("FAIL: row #%s did not land" % n)
open(P, "w", encoding="utf-8").write(s)
print("rows added: 1147 1148 1150 (Done) · 1149 1151 (Declined) · 1152 (Open)")
