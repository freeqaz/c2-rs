// **Board #1148 -> #174/#1152 — the live wrong emit that lane `w-align16`
// found, closed as a REFUSAL, and lane `w-order3` closed as a MATCH.**
//
// History, because both halves of it are worth keeping:
//
//   * This TU graded `mismatch` against real c2 on an **unmodified tree**, at
//     alignment 4. Not a gap, not a refusal — wrong bytes, board **#232**'s
//     shape, invisible to every scan because no fixture in the corpus could
//     generate it. `w-align16` found it with a grid built for something else.
//   * It was then fixed by REFUSING every `.bss` holding an internal-linkage
//     object, on purpose, because the right order was a three-cell observation
//     and not a rule.
//   * `w-order3` derived the rule, and this file is now **byte-exact**, at
//     `/GR /O1 /Oi /EHsc` and at `/Ox`, `/O2` and `/Od`.
//
// **The rule (S1′).** Rule S1 states three insertion points as if the *kind* of
// section chose one. It does not — the slot is chosen by which contributor
// materialised the section first, and a `.bss` has three answers:
//
//     A  a STATIC first reached from a `.data` initializer  <- this file
//            .drectve .debug$S .bss     .XBLD$W .XBLD$W .data
//     B  an EAGER EXTERNAL  (Rule S1's middle clause)
//            .drectve .debug$S .XBLD$W  .bss    .XBLD$W .data
//     C  a STATIC first reached from a FUNCTION body, and every DEFERRED
//        (dynamic-initializer) object, whatever its linkage
//            .drectve .debug$S .XBLD$W  .XBLD$W .text   .bss
//
// S1's middle clause is exactly `B` and is not refuted: across 247 real
// non-COMDAT `.bss` sections every one of the 138 in that slot contains an
// external, and **0 of 25** purely-static sections are there.
//
// **Why nobody had seen it.** `wsect_drop_static.cpp` records that an
// uninitialized *unreferenced* static is dropped by c2 entirely, and
// `wsect_data_linkage.cpp`'s header concluded from that: *"mixed linkage is
// unreachable in a `.bss` of a functionless TU"*. True of the cells that
// existed. The route around the drop is to **reference** the static — a `.data`
// initializer holding its address keeps it alive — and that is this file's third
// line. It is one line of C++ and it had never been written. The same one-line
// gap hid slot `C`: see `worder3_bss_slot_after_text.cpp`.
//
// **Rule Y1's STATIC clause is wrong here and is no longer applied.** Every cell
// behind it is a TU *with functions*, which is what keeps *their* statics alive;
// this writer only ever serves functionless TUs. `worder3_bss_slot_y3.cpp` is
// the witness for the replacement, Rule Y3. Y1's EXTERNAL clause is in scope and
// untouched.
//
// This cell is at alignment 4, so it grades slot `A` **without** depending on one
// byte of #1120. `work/w-align16/cells/A11_static_align16.cpp` is the same shape
// at 16 and is how it was found; `work/w-order3/cells/O10` and `O11` are 8 and
// 16 and both convert too.

struct A{int a;};
static A g;
A* p = &g;
