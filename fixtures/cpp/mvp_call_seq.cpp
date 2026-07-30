// **Ported (#35 step 2, rung 1 — Class A many-calls).** The framed
// statement-call sequence, one function per fact the class turns on.
//
// Every body here is `prologue · (setup · bl)+ · tail · epilogue` with the
// shipped 96-byte Class A frame and **nothing callee-saved**, because no value
// is read after the first call. Byte evidence per row, `/O1 /GS- /c` (and the
// `.text` is identical at `/Ox` and `/O2`):
//
//   two          36 B  bl ?v0 ; bl ?v1
//   four         44 B  four `bl`s, two symbols — the repeats reuse the first index
//   arg_dies     36 B  `a` is already in r3 and dies at the first call: no setup
//   lits         44 B  li r3,1 ; bl ; li r3,2 ; bl
//   perm         48 B  mr r11,r4 ; mr r4,r3 ; mr r3,r11 ; bl ; bl
//   computed     40 B  addi r3,r3,1 ; bl ; bl
//   then_lit     36 B  bl ; li r3,5      — FRAMED on one call
//   value        36 B  bl ; bl           — the last call's result IS the return
//   value_plus   40 B  bl ; bl ; addi r3,r3,1
//   ret_explicit 36 B  identical to `two` — the trailing `return;` emits nothing
//
// Four of those are the whole point:
//
//   * `then_lit` is framed with a **single** call, so the class boundary is "is
//     there anything after the call", not "are there two calls". A lone statement
//     call with nothing after it is tail-called instead (`mvp_argtail.cpp`).
//   * `value` ends `bl ?i0 ; addi r1,r1,96 ; … ; blr` — the tail-call transform
//     is **off** once the function is framed. Emitting `b ?i0` there would be a
//     wrong-bytes obj that still links.
//   * `ret_explicit` is `two` with an explicit `return;`. c2 records the
//     fallthrough as a SECOND `3A <label>` branch to the same label the return
//     plumbing then uses and emits nothing for it — the two objs are byte-
//     identical (1090 B each, compared whole with the source path held fixed).
//     The label compare is the gate: a real early return branches elsewhere.
//   * `four` and the TU as a whole pin the symbol order: a function's new callees
//     go out in **reverse first-reference** order and a repeat introduces nothing
//     (docs/OBJ_GY_SHAPES.md §3.3 as extended, docs/CODEGEN_FRAMED_CALLS.md §4.1).
//
// Ten framed functions in one TU also exercise the label counter hard: under
// `/Gy` that is a flat `3 × 10` surcharge and then `+5` each, so a stride error
// anywhere shows up as six wrong bytes per label.

extern void v0();
extern void v1();
extern void a1(int);
extern void a2(int, int);
extern int i0();

void two() { v0(); v1(); }
void four() { v0(); v1(); v0(); v1(); }
void arg_dies(int a) { a1(a); v0(); }
void lits() { a1(1); a1(2); }
void perm(int a, int b) { a2(b, a); v0(); }
void computed(int a) { a1(a + 1); v0(); }
int then_lit(int a) { a1(a); return 5; }
int value() { v0(); return i0(); }
int value_plus() { v0(); return i0() + 1; }
void ret_explicit() { v0(); v1(); return; }
