// **Positive** — the callee spellings the `.gl` symbol index must keep binding
// after D14 rewrote how it locates a record.
//
// D14 changed `gl_symbol_index` from "the name is the run right after a NUL" to
// "the name is the rightmost run in a graphic run that a record SEPARATOR
// precedes, under a symbol record KIND, spelled in the symbol alphabet". That is
// a **binding** change — it decides which symbol a token names — and the oracle
// cannot grade a correspondence (`docs/GAPS.md` §6, the `.sy` bullet). What an obj
// compare *can* grade is the other half: that the names it still binds are the
// ones c2 relocates against, since a wrong name is a different string in the
// string table and a different `.text` byte count. This file is that half.
//
// Every function here is a tail call or a generated empty destructor, so each one
// emits a single `b <callee>` with one REL24 against the resolved symbol — the
// callee name is *the whole output*. The three at the top are the spellings an
// intermediate version of this change dropped:
//
//   * an `extern "C"` callee (`cbind`) has **no `@@` at all**, so gating the index
//     on `looks_mangled` — which is what the unclaimed-symbol accounting uses —
//     silently un-resolved it. That version lost five tail calls in
//     `src/system/jpeg/Jpeg.cpp` alone before the alphabet replaced it;
//   * a namespace-qualified callee;
//   * a class-template base destructor (`??1?$B23t@H@@QAA@XZ`), whose name carries
//     `$` — the character that forced the alphabet to be `[A-Za-z0-9_$?@]` rather
//     than "identifier chars".
//
// A *function*-template callee is deliberately absent, and the reason is a finding
// rather than an omission: `void tf23<int>()` makes c2 splice
// `/alternatename:??$tf23@H@@YAXXZ=…` into `.drectve`, so the directive count goes
// to `02` and `drectve_is_boilerplate` refuses the whole TU — correctly, since the
// port emits `.drectve` as a constant. The census still reads **1/1 in class**,
// because the per-function cross-check (roadmap #44) cannot see a TU-level gate.
//
// The destructors below are the same shape `w14_dtor_delegate.cpp` and
// `w15_dtor_member.cpp` grade, present here for a different reason: they are the
// only in-class shape whose callee comes from a *symbol push* rather than from the
// function's own record, so they are what a mis-located record shows up in.

extern "C" void cbind();
void t_extern_c() { cbind(); }

namespace NS23 { void nsf(); }
void t_namespace() { NS23::nsf(); }

// A base-sub-object destructor delegation: `b ??1B23@@QAA@XZ`, nothing else.
struct B23 { ~B23(); int x; };
struct D23 : B23 { ~D23(); int y; };
D23::~D23() {}

// The callee's own name carries `$` twice, through a class template.
template <class T> struct B23t { ~B23t(); T t; };
struct D23t : B23t<int> { ~D23t(); };
D23t::~D23t() {}

// The member form, at a nonzero offset: `addi r3,r3,4 ; b ??1M23@@QAA@XZ`.
struct M23 { ~M23(); int m; };
struct H23 { ~H23(); int pad; M23 m; };
H23::~H23() {}
