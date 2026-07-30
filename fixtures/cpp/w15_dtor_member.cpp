// **Positive** — the *second* compiler-generated **empty destructor**: the one
// that destroys a single **member** sub-object rather than a base.
//
// `docs/IL_CALL_IN_EXPR.md` §5 characterized the generated destructor as
// delegating to its base through the class-layout intrinsic **2113**, and §14.3
// measured that that is only half of the shape. A class with **no destructible
// base and exactly one destructible member** produces the same skeleton with a
// completely different receiver: `this` plus a literal byte offset through a plain
// `27` add, with **no intrinsic anywhere**. The `27` form was hiding inside the
// same `expr-call-in-expr` bucket as the base form and was 48 % / 17 %
// whole-body-complete — by far the most complete thing left in it — while every
// larger sub-bucket sat at 0.0 %.
//
// The two are one production differing in one literal, and that literal is the
// whole codegen difference. Captured (`h0`/`h4`, fixture profile):
//
//   ??1HasMem@@QAA@XZ:   4bfffff0  b ??1MemA@@QAA@XZ           member at 0
//
//   ??1HasMem4@@QAA@XZ:  38630004  addi r3,r3,4                member at 4
//                        4bffffe4  b ??1MemA@@QAA@XZ
//
// So at offset 0 the address arithmetic emits nothing and this is byte-identical
// to what `w14_dtor_delegate.cpp` already emits; at a nonzero offset it is one
// `addi r3,r3,k`, handed to codegen as the argument-setup operand stream
// `[Load(this), Lit(k), Add]` — which is `return g(a + k)`, an emitter four mode
// lanes have graded since the MVP rather than a new one.
//
// Captured body (`h4`, from the `LO` marker). Everything from the `2C` strip
// onward is `w14`'s skeleton byte for byte; only the receiver differs:
//
//   53                                SS
//   33 86 41 74 00                    LIT int 0        (role UNKNOWN, as in w14)
//   26 <??1MemA>                      the MEMBER's destructor, pushed first
//   b9 <this> a6 43 91 20             the object pointer -- no intrinsic frame
//   33 86 41 74 04                    LIT int 4        -- the member's OFFSET
//   27 a6 43 8a 20                    byte-offset add -> the member's address
//   2c a6 43 8b 20 00                 cv strip (pointer->pointer: no code)
//   99 86 43 8c 20 00                 member bind -- DIRECT dispatch
//   bd 82 07 03 00 80 0c 10 00 00     CALL void, cdecl, fn-type id
//   4c                                ZERO explicit arguments
//   5c 86 41 74 11                    opaque statement trailer
//   4b                                statement end
//   3a <lbl> 54 02 29 <lbl>           return plumbing
//   5e 01 31                          opaque SUB-OBJECT trailer -- ONE sub-object
//   4b  4f 12 47 54 01 54 00          function tail
//
// The `5E 01` count is the gate that matters here, and it is not the same gate it
// was for the base form. A class with **two** destructible members carries
// `5E 02` and two statements, and the reference does *not* emit two branches — it
// emits a frame, `or r31,r3,r3`, and two `bl`s in **reverse** declaration order,
// because `this` is live across the first call. That shape is refused twice over
// (the count, and reaching the segment end) and is swept as a neighbour in
// `scripts/expr_sweep.sh` rather than sitting here, where every case must match.
//
// The member's destructor is declared and not defined on purpose, for the same
// reason as `w14`'s bases: c2 may inline a callee it can also see.

struct MemA { ~MemA(); int a; };

// (a) the member is first in the layout: offset 0, no address arithmetic at all.
struct HasMem { ~HasMem(); MemA m; };
HasMem::~HasMem() {}

// (b) the same member after four bytes of padding: one `addi r3,r3,4`.
struct HasMem4 { ~HasMem4(); int pad; MemA m; };
HasMem4::~HasMem4() {}

// (c) offset 8, so the literal is not the only nonzero value ever tested.
struct HasMem8 { ~HasMem8(); double d; MemA m; };
HasMem8::~HasMem8() {}

// (d) a `const` member. The receiver's TYPE tag picks up the const bit (`A6`), and
// the destructor is still called on it, so both tag spellings must be admitted.
struct HasConst { ~HasConst(); int pad; const MemA m; };
HasConst::~HasConst() {}

// (e) a member whose own destructor is **virtual**. Destroying a member sub-object
// of known type is still a DIRECT call — the bind is `99`, not `67`/`9A` — so this
// emits a bare branch to `??1MemV@@UAA@XZ` and not a vtable dispatch. It is the
// witness that the licence to branch comes from the bind and not from the callee.
struct MemV { virtual ~MemV(); int a; };
struct HasVirt { ~HasVirt(); MemV m; };
HasVirt::~HasVirt() {}

// (f) a member sub-object that itself has a member sub-object: two generated
// destructors, each destroying one thing, at offsets 0 and 8.
struct Inner { ~Inner(); MemA m; };
struct Outer { ~Outer(); double d; Inner i; };
Outer::~Outer() {}

// (g) a large member. The pointer TYPEs are unchanged by the pointee's size — the
// gate is on the pointer's own width, never the pointee's — so this separates
// `is_ptr4_kind` from `is_ptr_to_4` inside this shape.
struct BigMem { ~BigMem(); double a, b, c; char pad[100]; };
struct HasBig { ~HasBig(); double d; BigMem m; };
HasBig::~HasBig() {}

// (h) a NON-destructible base plus a destructible member. Still one destroyed
// sub-object and one branch; the base contributes only to the member's offset.
struct NoD { int n; };
struct MixMem : NoD { ~MixMem(); MemA m; };
MixMem::~MixMem() {}

// (i) braces on their own lines, as in `w14`'s `d3`: the `}`'s source line lands
// as a `4F 01 <line>` marker inside the return plumbing, which a one-line
// definition never shows.
struct HasNl { ~HasNl(); int pad; MemA m; };
HasNl::~HasNl()
{
}

// (j) the offset at the top of the accepted range. MEASURED at the boundary:
// 32,764 is one `addi r3,r3,32764`, and 32,768 is **two** instructions
// (`addis r3,r3,1 ; addi r3,r3,-32768`) and is refused. The gate is at the signed
// 16-bit edge, not at a round number.
struct HasFar { ~HasFar(); char pad[32764]; MemA m; };
HasFar::~HasFar() {}
