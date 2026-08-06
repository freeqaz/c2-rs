#!/usr/bin/env python3
"""board_rows.py — insert lane w-inl0's board rows #990-#995 in numeric position.

The rows go between #989 (w-drop3's last) and #996 (w-inread's first), which is
the range the coordinator reserved. Written as a script rather than by hand so
the insertion point is the row number and not a line count.
"""
ROWS = [
    ("990", "w-inl0",
     "**THE `expr-intrinsic-memset` PRODUCTION BEHIND #980 IS NOT A `memset` — it is an EMPTY TAG TEMPORARY**",
     "**READ, from a capture, and pinned by a frozen cell.** The body is one discarded call whose argument list carries "
     "`33 86 41 74 80 AD 00 00 00` (selector 173) · `40 86 41 74` (**`int` result**) · three int literals (align `1`, count `1`, fill `0`) · "
     "`9B <TYPE> <tok> 2C <TYPE> <v> 55 <TYPE>` (the destination — a **temp bind**) · `4C` · **the same token again** `44 55 <TYPE>`. "
     "The `40`'s result type is a **measured** discriminator: on `BustAMovePanel.cpp` the selector run occurs **455** times and splits "
     "**446** `40 86 41 74` (this shape) / **7** `40 86 43 83 08` (a real `memset(p,0,n)`, which c2 lowers to `b <memset>` with a REL24) / "
     "**2** not followed by a `40` at all",
     "**12,690** refused workload rows carry the shape",
     "`crates/c2-il/src/func/body/shapes/no_effect.rs`; rungs/2026-08-08-w-inl0.md §2; `work/w-inl0/cells/m01.cpp`",
     "**The name was the misdirection.** `w-empty` §11.2 and `w-seq` §5 both call this production a *memset*, and a lane that widened "
     "*memset* would have built a lowering for `b <memset>`. What is actually there is `true_tag()` — a one-byte value-initialization "
     "into a stack temporary that is passed by reference and never read"),

    ("991", "w-inl0",
     "**138 OF #980's 370 ARE CLOSED, BY A DECODE-ONLY READER — `IlBundle::functions()` IS UNTOUCHED**",
     "**SHIPPED. `fnbyte-differs` 3,195 → 3,057, `fnbyte-exact` 35,982 → 36,120, `fnbyte-elided` 1,516 → 1,654, and a per-symbol diff of "
     "the two `--fnbyte-diff-jsonl` files reads 138 CLOSED and 0 OPENED** — every one `??$_Destroy_Range@…`. `parse_segment` is byte-for-byte "
     "unchanged, the row stays `FnVerdict::Blocked` with key `expr-intrinsic-memset`, `fnbyte-refused` is **130,573 at both ends**, "
     "`vocab-gap` **861 → 861**, `mismatch` **0**, TU match **10 → 10**. `c2_core::elide::Reduction::NoEffectCall` contributes a **link and "
     "never a seed**, so the cycle refusal and the round ceiling are unchanged and still tested",
     "**138** of 370",
     "rungs/2026-08-08-w-inl0.md §1, §3; `crates/c2-core/src/elide.rs`; `crates/c2-harness/tests/dead_temp_elision.rs`",
     "**#971's four conditions, satisfied by construction rather than by care.** (1) not graded on the net — every mover is named; "
     "(2) the emptiness question is asked of the *widened body*; (3) it enters the **existing** fixpoint (m05 is the cell that fails if it "
     "were one step); (4) the gate does not widen, so **#878**'s loaded gun is not touched"),

    ("992", "w-inl0",
     "**THE RESIDUE IS 232, IT SPLITS BY ELEMENT TYPE, AND IT IS STILL MECHANISM E — NOT I**",
     "**PRICED to one production, printed on every scan.** `fnbyte-blr-stop|expr-intrinsic-memset` **231** (+1 `callee-unbound`), and one "
     "level deeper `fnbyte-blr-stop2|return-scope-close-cflow-label` **228** (+2 `module-end-0x4D`, +1 `callee-unbound`). The 138 that closed "
     "are `_Destroy_Range` over **scalars and pointers**; the 232 that did not are over **class types**, whose "
     "`__type_traits<T>::has_trivial_destructor` is `__false_type`, so they take STLport's **loop** overload. GRID-M's **m06** compiles that "
     "shape: c2 emits one `4e800020` for the whole chain **at `/Ob0` as well**",
     "**228** behind one production",
     "rungs/2026-08-08-w-inl0.md §5; `work/w-inl0/cells/m06.cpp`; board **#922**",
     "**The `/Ob0` row is the finding and it was a registered prediction LOST.** The ADDENDUM predicted c2 would keep a call at `/Ob0` — that "
     "the loop vanished by *inlining*. It does not, so the loop is erased by c2's own dead-code elimination and the follow-on rung is a "
     "**parser** rung on `return-scope-close-cflow-label`, not an inlining rung. That production is also **76** differs on its own account "
     "(`w-seq` §5): two rungs, one dependency, and neither may quote the other's worth as its own"),

    ("993", "w-inl0",
     "**AN UNSOUND VERSION OF THIS RULE MAKES EVERY WORKLOAD NUMBER BETTER AND BREAKS NO WORKLOAD CONTROL — only the CELLS see it**",
     "**MEASURED, as registered mutation M2.** Making `Reduction::NoEffectCall` a **seed** instead of a link — i.e. eliding a call without "
     "asking anything about the callee — moves `fnbyte-differs` **3,057 → 2,878**, `fnbyte-exact` to **36,299**, moves **0** functions the "
     "wrong way, and leaves `fnbyte-noeffect-ref-other` at **0**. It turns **4 of the 8 GRID-M cells RED**, including m07, whose callee the TU "
     "does not define and for which c2 keeps a REL24",
     "**+179** of false credit",
     "rungs/2026-08-08-w-inl0.md §7; `work/w-inl0/mutate.sh`",
     "**This is #971 condition 1 with a number under it, and STATUS trap 0 in its most literal form.** `-ref-other` is green over the "
     "population it can reach, and that population is the *admitted row's own* bytes — never its caller's. A lane that had graded this "
     "widening on `fnbyte-differs` falling would have shipped the unsound rule and every standing instrument would have agreed with it"),

    ("994", "w-inl0",
     "**THE DEAD-TEMPORARY IDIOM IS GENERAL, NOT AN STLport ACCIDENT — 12,690 rows, 1,409 admitted, the rest priced by production**",
     "**MEASURED and printed on every scan.** `fnbyte-noeffect-rows` **12,690** · `-admitted` **1,409** · of the admitted, `-ref-blr` **363**, "
     "`-ref-other` **0**, `-ref-absent` **1,046** (c2 emitted no COMDAT at all). The 11,281 not admitted: `-callee-refused` **11,248**, "
     "`-callee-unbound` 33, `-callee-parsed-live` **0**. Where they stop: `return-scope-close-cflow-label` **4,197** · `body-0x67` **2,948** · "
     "`expr-lit-type-8207` **2,572** · `expr-call-in-expr-recv-load-then-type-void-and-op-more` **1,455** · `param-width-undetermined:mid` 73 · "
     "`module-end-0x4D` 3",
     "**11,248** blocked one link down",
     "rungs/2026-08-08-w-inl0.md §9.2; `crates/c2-harness/src/gap/fnbytes.rs`",
     "**Not one of those productions converts a function by itself** — each only lets a chain close, which is `w-seq` §5.1's caution "
     "repeating. It is also why a name-based special case was both forbidden and far narrower than the truth: the shape is tag dispatch, "
     "and STLport is merely where this workload keeps most of it"),

    ("995", "w-inl0",
     "**THE LAST `.ex` SEGMENT OF EVERY BUNDLE REFUSES AS `module-end-0x4D`, AND IT COSTS A REAL FUNCTION PER TU**",
     "**FOUND while building GRID-M, and it cost a five-level scaffold to work around.** `eat_fn_tail` accepts either the segment end or the "
     "full module trailer `4F 02 20 00 · 4F 01 <line> · 4D`; the **last** segment carries only the `4D`, so it always refuses. In a workload TU "
     "that is some anonymous instantiation — **3** rows of this lane's stop histogram — but in a cell it was `??$aux@…`, **the empty leaf the "
     "whole chain has to be seeded from**, and every verdict below it was worthless until a five-level template pad pushed something else "
     "into last place (one level was not enough: instantiations are **not** emitted in source order)",
     "**3** workload rows, and 1 function per bundle",
     "rungs/2026-08-08-w-inl0.md §4, §9.4; `crates/c2-il/src/func/body/expr.rs` (`eat_fn_tail`)",
     "**A one-byte reader repair with a known answer**, in the same family as §9.20's. Also the reason `w-empty`'s ANCHOR had to be "
     "**prepended** here rather than appended: a template instantiation's segment is emitted after every source-order function, so the last "
     "*source-order* function carries the module trailer without its `4D` and the ANCHOR control could not fire in four of eight cells"),
]

p = 'docs/BOARD.md'
lines = open(p).read().split('\n')
row_990 = lines.index([l for l in lines if l.startswith('| **996**<sub>w-inread</sub>')][0])
new = []
for n, lane, item, verdict, number, where, note in ROWS:
    new.append(f"| **{n}**<sub>{lane}</sub> | {item} | {verdict} | {number} | {where} | {note} |")
lines[row_990:row_990] = new
open(p, 'w').write('\n'.join(lines))
print(f"inserted {len(new)} rows before line {row_990 + 1}")
