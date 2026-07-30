// A member function on **source line 70** — the line whose marker is `4F 01 46`,
// whose payload byte is `0x46`, the same byte that marks the formals list.
//
// This was a live wrong-bytes emit, and it is the second one that byte has caused.
// `parse_formals` was fixed for it once: it anchors on the `46` whose region ends
// exactly on the `LO` marker, and `il_expr_deref.cpp` records that taking the first
// `0x46` gave one of sixteen otherwise-identical bodies an empty formals list. But
// `parse_this_token` was still locating the *same marker* with a plain first-`0x46`
// search, so on line 70 it looked for the `this` group ahead of the line marker,
// found none, and returned `None` — which the caller could not tell apart from a
// genuine non-member. `this` vanished, every explicit formal dropped one register,
// and `C::gp` emitted `lwz r3,0(r3)` where the reference has `lwz r3,0(r4)`.
// `Port=Mismatch @ offset 537`.
//
// Two things were wrong and both are fixed:
//
//   * the marker is now located in exactly one place
//     (`expr::formals_marker`), so nothing can disagree about where it is;
//   * `parse_this_token` returns a three-valued answer — bound, positively absent,
//     or undetermined — and the caller refuses on the third. "Absent" now requires
//     seeing the function-token push run straight into the marker, rather than
//     being inferred from a failed search.
//
// `gp` and `gp2` are identical but for their line, which is the whole point: under
// the old anchor `gp` mis-emitted and `gp2` two lines later was byte-exact, so
// nothing about the *source* distinguished the broken case. It was found by review
// reading the anchor, not by any fixture — the fixtures had no member function
// anywhere near line 70, and the three pinned `.ex` segments in `func/mod.rs` were
// truncated at the formals marker, so the pre-body region where `this` lives was in
// no test at all. They now carry their real `53 53 26 <fn>` prologue.
//
// Keeping this file working means keeping line 70 at line 70. If a line is added or
// removed above, the padding below must absorb it — `gp` must stay on 70, and the
// assertion that makes this fixture mean anything is that `c2rs diff` still says
// `Port=Match` for it.

struct C {
    int m;
    int gp(int* q) const;
    int gp2(int* q) const;
};

// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
// (padding to line 70 — see the note above)
int C::gp(int* q) const { return *q; }
int C::gp2(int* q) const { return *q; }
