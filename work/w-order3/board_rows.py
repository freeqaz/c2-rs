#!/usr/bin/env python3
"""board_rows.py — insert lane w-order3's board rows and close #1152 in place.

**Fails hard rather than guessing.** Every anchor must be found exactly once,
every new number must be absent before the edit and present after, and the row
count must grow by exactly the number of rows inserted. A board editor that
silently no-ops is how a row goes missing.

Placement follows the board's actual convention, checked before writing rather
than assumed: the sections are NOT globally sorted, and the most recent lanes
(`w-align16`'s #1147/#1148/#1150, #1149/#1151, #1152) put their rows at the head
of the section, in ascending order among themselves. This does the same.
"""
import re
import sys

PATH = "docs/BOARD.md"
src = open(PATH).read()
orig = src

ROW_RE = r"^\|\s*\*{0,2}(\d+)\*{0,2}(?:<sub>[^<]*</sub>)?\s*\|"


def nums(text):
    return sorted({int(m) for m in re.findall(ROW_RE, text, re.M)})


before = nums(src)

DONE = {}
OPEN = {}

DONE[1177] = (
    "| **1177**<sub>w-order3</sub> "
    "| **#1152 IS CLOSED — the `.bss` slot is chosen by its FIRST CONTRIBUTOR, "
    "not by the section's kind, and #1148's two live-wrong-emit cells are "
    "byte-exact** "
    "| **DONE, on an 18-cell grid frozen by `sha256` and committed before the "
    "first `cl.exe` and before one line of `crates/`.** Frozen grid **match 3 → "
    "11** with **`mismatch` 0** at the workload's `/GR /O1 /Oi /EHsc` *and* at "
    "`/Ox`, `/O2`, `/Od` — eight graded runs, base and tip at four profiles. "
    "`D01` (align 4), `D02` (align 8) and `A11` (align 16) all convert "
    "`codegen-gap` → `match`. Workload movement: **0**, and `factor-c` "
    "**169 → 169** from a scan (#1181) "
    "| **Rule S1′.** `OBJ_DATA_BSS_SHAPE.md` §2.2's Rule S1 reads as though the "
    "*kind* of section picks its slot. It does not — the slot is picked by "
    "which contributor materialised the section first, and a `.bss` has three "
    "answers: **`A`** before `.XBLD$W(C2)` (a STATIC first reached from a "
    "`.data` initializer), **`B`** between the watermarks (an EAGER EXTERNAL — "
    "this is S1's middle clause), **`C`** after the code groups (a STATIC first "
    "reached from a FUNCTION body, and every DEFERRED object). **S1's middle "
    "clause is not refuted**: on 247 real non-COMDAT `.bss` sections all **138** "
    "in slot `B` contain an external and **0 of 25** purely-static sections are "
    "there. In slot `A` the symbol group is in **`.gl` record order** — "
    "**Rule Y3**, and Rule Y1's *declaration*-order static clause is refuted "
    "there (#1180) "
    "| **Eight rivals, each killed by its own cell and not by an argument** "
    "(board #259's method). `O02` (`A g; A* p = &g;` — extern, relocation "
    "present, address taken, and `.bss` stays in slot `B`) kills **the reloc "
    "rival** and **the address-taken rival**, which is exactly the separation "
    "#1152 said its three cells could not make. `O03`/`O04` kill "
    "**functionless** in both directions. `O08` kills **all-static**. "
    "`O03`/`O04` kill **`.data` present**. `O01`/`O12`/`O17` at align 4 kill "
    "**alignment**. `O01` kills **object count**. `O12` kills **aggregate "
    "type**. What this does NOT do: mixed linkage stays refused (#1178), "
    "`MAX_OBJECTS_PER_SECTION` stays 2 (#184), and slot `C` is outside "
    "`emit_data_obj` by construction (#1179) "
    "| rungs/2026-08-08-w-order3.md; `work/w-order3/PREREG.md` (frozen at "
    "`92f20b3d`), `work/w-order3/cells/SHA256SUMS`; "
    "`crates/c2-core/src/coff/data.rs`; `fixtures/cpp/wa16_bss_static_reloc.cpp`, "
    "`worder3_bss_slot_extern.cpp`, `worder3_bss_slot_y3.cpp`; "
    "`docs/OBJ_DATA_BSS_SHAPE.md` §2.2 |"
)

DONE[1180] = (
    "| **1180**<sub>w-order3</sub> "
    "| **RULE Y1's STATIC CLAUSE IS REFUTED ON A FUNCTIONLESS TU — the order is "
    "`.gl`, not DECLARATION. A scope error, and NOT a live one** "
    "| **Not an alarm, and the distinction is the whole point.** Y1's static "
    "clause was already fenced by #1148's refusal, so `emit_data_obj` had not "
    "applied it since `8fa6b119` and no wrong bytes ever reached an obj through "
    "it. What this row adds is the measurement that widening it would have "
    "needed, and the replacement "
    "| `work/w-order3/cells/O09` is the first functionless TU with two statics "
    "in one `.bss`, and the two candidate orders **disagree** on it: `.gl` "
    "order is `h g`, declaration order is `g h`, and c2 answers `h g` with "
    "addresses `h`@0 `g`@4. **Rule Y3**: in slot `A` the group is in `.gl` "
    "record order — the same permutation as Rule A1's walk, so ascending "
    "address. Confirmed out of sample at n = 3 by `O16` (`.gl`, addresses and "
    "symbol table all `i h g`; declaration `g h i`). The same obj is its own "
    "control: its `.data` group **is** in declaration order, so one obj carries "
    "both permutations "
    "| **Y1 is not refuted where it was fitted.** Every static-bearing row "
    "behind it comes from a TU **with functions**, which is what keeps its "
    "statics alive; and Y1's **EXTERNAL** clause is in scope (§7.1's families B "
    "and C are functionless cells) and untouched, still carrying its 89 real "
    "sections. **STILL OPEN and untested by either lane: whether Y1's mixed row "
    "holds for a TU WITH functions**, which is where it was originally fitted. "
    "A writer for that shape must re-measure rather than inherit "
    "| rungs/2026-08-08-w-order3.md §5.1; `fixtures/cpp/worder3_bss_slot_y3.cpp`; "
    "`docs/OBJ_DATA_BSS_SHAPE.md` §6.2; `crates/c2-core/src/coff/data.rs` |"
)

DONE[1181] = (
    "| **1181**<sub>w-order3</sub> "
    "| **THE WORKLOAD CANNOT MOVE ON THIS RUNG — 0 of 871 objs put a `.bss` "
    "before `.XBLD$W:C2`, and the reason is structural** "
    "| **MEASURED from the section census, not inferred from a flat scan.** "
    "`work/w-order3/census_order.py` and `census_slot.py` read the `order` "
    "array and every `.bss` symbol's storage class out of "
    "`work/w-bss/census/sections.jsonl` (871 objs, 247 non-COMDAT `.bss`). Slot "
    "`A`: **0**. Slot `B`: 138, every one containing an external. Slot `C`: "
    "109. Purely-static sections in slot `B`: **0 of 25**. Functionless objs on "
    "the whole workload: **6**, none with a surviving static `.bss`. So #1177 "
    "converts fixtures and **cannot** convert one workload TU — *cannot*, by "
    "construction, not *did not* "
    "| The scan confirms it: `match 10 · mismatch 0 · codegen-gap 0 · vocab-gap "
    "861 · capture-fail 7`; factors **A 28 · B 338 · C 169 · D 10 · E 2** all "
    "unchanged; every FBM figure byte-identical (`fnbyte-exact` 36,209 · "
    "`reloc-differs` 861 · `differs` 2,111 · `refused` 130,579 · `unbound` "
    "9,217 · `match-tu-differs` 0). Registered as prereg P8 and P9 **before** "
    "the census was run "
    "| **The two out-of-sample predictions this census settles are worth more "
    "than the zero**: no purely-static `.bss` is ever in Rule S1's middle slot "
    "(0 of 25), and every section that IS in that slot contains an external "
    "(138 of 138). #1177's model was fitted on 18 probe cells and holds on 247 "
    "real sections it never saw. Same shape as #1150 — *cannot* is a far more "
    "useful answer than *did not* "
    "| rungs/2026-08-08-w-order3.md §6; `work/w-order3/scans/tip_metrics.txt`; "
    "`work/w-order3/census_slot.py` |"
)

OPEN[1178] = (
    "| **1178**<sub>w-order3</sub> "
    "| **The MIXED-linkage functionless `.bss` is REFUSED — with the answer in "
    "hand, not for want of a cell** "
    "| **Worth: zero, measured.** `work/w-order3/census_order.py` over the "
    "871-obj census: **0 of 871** workload objs put a `.bss` before "
    "`.XBLD$W:C2` at all, so neither this row nor #1177 can move `factor-c`. "
    "Taking it buys the correctness statement, not a conversion "
    "| **What real c2 does is measured** (`work/w-order3/cells/O08`, "
    "`A g; static A h; A* p = &h;`): the section is in slot `A` because the "
    "STATIC created it; both objects are in `.gl` order at ascending addresses "
    "(`h`@0, `g`@4); and the EXTERNAL's symbol record is written **outside the "
    "group**, immediately after `__C2_11886` — the slot-`B` position the whole "
    "group would have taken had the external created the section. That reads "
    "exactly like #1177's first-contributor model one level down "
    "| **It is still ONE obj at n = 2.** `MAX_OBJECTS_PER_SECTION` is **2** "
    "(#184), so every cell available here has n = 2, and at n = 2 the reading "
    "above is indistinguishable from several orderings that would differ at "
    "n = 3. **Do not widen `emit_data_obj` to mixed linkage before #184** — "
    "this row is downstream of it. `fixtures/cpp/worder3_bss_slot_mixed.cpp` is "
    "the graded refusal: a lane that widens without teaching the writer the "
    "external's record position turns it from a refusal into a `mismatch` in "
    "every mode lane, which is the difference between this row and a comment "
    "nobody runs "
    "| rungs/2026-08-08-w-order3.md §5; `work/w-order3/cells/O08_mixed_reloc.cpp`; "
    "`docs/OBJ_DATA_BSS_SHAPE.md` §6.2 |"
)

OPEN[1179] = (
    "| **1179**<sub>w-order3</sub> "
    "| **SLOT `C` — an EAGER static `.bss` kept alive by a FUNCTION sits AFTER "
    "the code groups, and this document recorded that position as a DYNINIT "
    "property** "
    "| **109 of 871 workload objs put a non-COMDAT `.bss` there**, holding "
    "**33** of the corpus's 53 static-bearing `.bss` sections — against 0 in "
    "slot `A`. So slot `C` is the workload's common case for a static `.bss` "
    "and #1177's slot is the rare one "
    "| `OBJ_DATA_BSS_SHAPE.md` §2.1 rows 11–13 put a `.bss` after the code "
    "groups, and the document read that as a fact about **deferred** "
    "(dynamic-initializer) objects. It is not. "
    "`fixtures/cpp/worder3_bss_slot_after_text.cpp` — `static A g; void "
    "f(){g.a=1;}` — is an ordinary **eager** static with no dynamic "
    "initializer and lands in slot `C` too, because a static is materialised "
    "lazily at its FIRST reference and that reference is in the function body. "
    "Give the same object both referrers and slot `A` wins, the earlier of the "
    "two (`work/w-order3/cells/O06`) "
    "| **The one-line-of-C++ shape, twice.** #1148's slot-`A` cell was "
    "invisible because nobody had written `A* p = &g;`; slot `C`'s was "
    "invisible because nobody had written `void f(){ g.a = 1; }`. Neither is "
    "reachable from `coff::data::emit_data_obj`, which serves functionless TUs "
    "only — so **this is not a live wrong emit, it is a boundary with a cell on "
    "its far side**. It goes live the moment a writer emits a `.bss` for a TU "
    "that also has `.text`, which is #174's own named remainder. That writer "
    "must read Rule S1′ and not Rule S1 "
    "| rungs/2026-08-08-w-order3.md §4; `work/w-order3/census_slot.py`; "
    "`fixtures/cpp/worder3_bss_slot_after_text.cpp` |"
)

# ------------------------------------------------------------ #1152 closure ----
OLD_HEAD = ("| **1152**<sub>w-align16</sub> | **THE SECTION AND SYMBOL ORDER FOR AN "
            "INTERNAL-LINKAGE `.bss` IN A FUNCTIONLESS TU IS UNKNOWN — three cells, "
            "no rule** |")
NEW_HEAD = ("| **1152**<sub>w-align16</sub> | **CLOSED 2026-08-08 by lane `w-order3` — "
            "see #1177 (the rule), #1178 (what stayed refused), #1179 (a THIRD slot "
            "nobody had a cell for) and #1181 (the workload cannot move).** "
            "~~THE SECTION AND SYMBOL ORDER FOR AN INTERNAL-LINKAGE `.bss` IN A "
            "FUNCTIONLESS TU IS UNKNOWN — three cells, no rule~~ |")
OLD_TAIL = ("Belongs with board **#174**, whose grid this is | "
            "rungs/2026-08-08-w-align16.md §4.2 |")
NEW_TAIL = ("Belongs with board **#174**, whose grid this is. **CLOSED — and this row's "
            "central warning was RIGHT on both counts.** The three cells could not "
            "separate \"internal linkage moves it\" from \"a `.data` relocation into "
            "`.bss` moves it\"; `w-order3`'s `O02` (`A g; A* p = &g;` — the relocation "
            "*without* the linkage) makes exactly that separation, and the reloc rival "
            "dies. The block this row said not to encode was also **incomplete in a way "
            "three cells could not have shown**: there is a THIRD slot, after the code "
            "groups, and 109 of 871 workload objs use it (#1179). Declining to encode "
            "it was the right call | rungs/2026-08-08-w-align16.md §4.2; closed by "
            "rungs/2026-08-08-w-order3.md |")
for old, new in ((OLD_HEAD, NEW_HEAD), (OLD_TAIL, NEW_TAIL)):
    if src.count(old) != 1:
        sys.exit(f"FAIL: anchor found {src.count(old)} times, want 1: {old[:60]}…")
    src = src.replace(old, new)

# ---------------------------------------------------------------- insertion ----
lines = src.split("\n")


def head_of(section):
    """Index of the first row line in `section` (just past its `|---|` rule)."""
    start = lines.index(f"## {section}")
    for i in range(start + 1, len(lines)):
        if re.match(r"^\|\s*-{2,}", lines[i]):
            return i + 1
    sys.exit(f"FAIL: no table rule under '## {section}'")


for section, rows in (("Done", DONE), ("Open", OPEN)):
    at = head_of(section)
    lines[at:at] = [rows[n] for n in sorted(rows)]

src = "\n".join(lines)

after = nums(src)
new = set(DONE) | set(OPEN)
for n in new:
    if n in before:
        sys.exit(f"FAIL: #{n} was already on the board")
    if n not in after:
        sys.exit(f"FAIL: #{n} is not on the board after the edit")
if len(after) != len(before) + len(new):
    sys.exit(f"FAIL: board went {len(before)} -> {len(after)} rows, want +{len(new)}")
if src == orig:
    sys.exit("FAIL: no change")
if "#1152 IS CLOSED" not in src:
    sys.exit("FAIL: the #1177 row did not land")

open(PATH, "w").write(src)
print(f"OK: rows {sorted(new)} inserted, #1152 closed; board {len(before)} -> {len(after)}")
