// **W-ALIGN's negative cell, CONVERTED by w-align16 (board #1120). Read the
// correction before the history.**
//
// `__declspec(align(16))` spells the `.gl` tag **`CA`** (wide, width field
// `8A` = 16) and c2 gives the object ALIGN_16 — `Characteristics` nibble **5**.
// That measurement is `w-align`'s and it stands to the digit. What has changed
// is the conclusion drawn from it.
//
// **This file is now a MATCH, byte-exact, at the workload's own
// `/GR /O1 /Oi /EHsc` and at `/O2`** — through `dyninit_tu` and
// `coff::dyninit::align_nibble(16, 16) = 5`. At `/Ox` and `/Od` it is still
// `codegen-gap`, for a reason that has nothing to do with alignment: neither
// profile implies `/GF`, so the literal is a non-COMDAT `$SG<n>` `.rdata` placed
// before `.text` and `emit_dyninit_obj` declines it, exactly as
// `wr1c_dyninit_extern.cpp` does there.
//
// ---- what this file used to say, and what became of it ----
//
// It was landed by `w-align` as a GRADED REFUSAL, and its header asked for one
// thing:
//
//   > "a later widening that adds `8A` without teaching `placement_align` about
//     16 turns this fixture from a refusal into a mismatch."
//
// **That instruction was followed and it worked.** Lane `w-align16` taught the
// promotion table 16 in all three functions that share it —
// `container::placement_align`, `container::align_nibble` and
// `data::section_nibble` — before touching `align_of_type_tag`, so the fixture
// went refusal → match and never once through mismatch.
//
// The three gates before the alignment one all passed then and pass now: mark
// `81`, frame `00 02`, linkage `01`, size varint `10` = 16, attr `00`.
//
// **The guard has moved up one power of two.** A boundary needs a cell on its
// far side, and 16 is no longer the far side: `wa16_data_align32.cpp` is
// `__declspec(align(32))`, which c2 gives nibble **6**, and it is the graded
// refusal now. See its header for why 32 is refused when 16 is not — the answer
// is the grid, not the arithmetic.

__declspec(align(16)) struct L { virtual void f(); L(const char* s, int r); int a; };
L gL("abc", 0);
