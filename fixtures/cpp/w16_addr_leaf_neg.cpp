// **Negative** — everything one token away from the address leaf accepted in
// `w16_addr_leaf.cpp`. Every function here must keep refusing
// (`NotImplemented`), and the file must never produce a `mismatch`.
//
// The address leaf is admitted because `addi rD, rBase, K` is one instruction
// the port already emits and the member's own type never reaches it. Every gate
// that makes that true has a neighbour that looks the same in the IL to within
// one token and costs something else — those are below, one per gate.
//
//   n_edge_hi   &p->t at 32768   `addis r3,r3,1 ; addi r3,r3,-32768`. TWO
//                                instructions. 32764 is accepted in the positive
//                                file, so the pair straddles the signed 16-bit
//                                edge and a `wrapping` cast would show up here as
//                                `addi r3,r3,-32768` alone — wrong bytes, not a
//                                gap.
//   n_zero_r4   &s->a from r4    `mr r3,r4`. A zero offset emits nothing only
//                                because the address is already in the return
//                                register; from any other argument register c2
//                                pays a move. Same boundary
//                                `straight_line_is_out_of_class` draws for the
//                                bare-parameter identity, drawn again here rather
//                                than assumed — and the parser, not the emitter,
//                                is what draws it.
//   n_zero_i_r4 &p->a0 from r4   the same, through the intrinsic designator.
//   n_vbase     &p->v            a member of a **virtual** base is intrinsic
//                                **2118**, not 2117, and it is a vbtable
//                                indirection: `lwz r11,0(r3) ; lwz r11,4(r11) ;
//                                add r3,r3,r11`. The selector is required
//                                literally, and this keeps that discriminating.
//   n_store     b = v            the STORE through the same designator: `stw
//                                r4,12(r3)`. The 2117 production computes an
//                                address and this file is about what happens
//                                after it — a write is not a value.
//   n_narrow    return wc        a LOAD through the same designator at width 1:
//                                `lbz`. The address path deliberately ignores the
//                                member width; the load path must not, and the
//                                two share one designator decoder.
//   n_var_ix    &s->arr[i]       a variable index: `slwi r11,r4,2 ; add r3,r3,r11`
//                                — the offset is not a literal at all, so there is
//                                nothing to fold into a displacement.
//   n_glob      &g_s.b           the base is a **global**: `lis`/`addi` with two
//                                relocations, and the token is not a formal, so
//                                the parse refuses by absence from the list it
//                                built rather than by a failed search.
//   n_as_int    (int)&s->b       the address feeds an integer expression. It is
//                                the same `addi` today, but the conversion is a
//                                `2C` from pointer to int that no capture covers,
//                                and admitting a cross-class `2C` is how a
//                                reinterpret gets in.
//   n_addr_add  &s->b + 1        pointer arithmetic on the result, which c2
//                                **scales** by the element size — the same trap
//                                `w12_ptr_leaf_neg.cpp`'s `n_padd` pins from the
//                                load side.
//   n_two       two statements   a second statement after the address. The
//                                production must reach the return plumbing and
//                                the end of the segment, not merely parse a
//                                prefix.

struct S { int a; int b; int arr[4]; };
struct Edge { char pad[32768]; int t; };

struct A { int a0; int a1; };
struct B { int b0; int b1; };
struct D : A, B { int d; };

struct VA { int v; };
struct VD : virtual VA { int d2; int* pv(); };

struct W { char wc; int wi; };
struct DW : B, W { char gwc(); void swi(int v); };

S g_s;

int*  n_edge_hi(Edge* p)          { return &p->t; }
int*  n_zero_r4(int x, S* s)      { return &s->a; }
int*  n_zero_i_r4(int x, D* p)    { return &p->a0; }

int*  VD::pv()                    { return &v; }          // n_vbase
void  DW::swi(int v)              { wi = v; }             // n_store
char  DW::gwc()                   { return wc; }          // n_narrow

int*  n_var_ix(S* s, int i)       { return &s->arr[i]; }
int*  n_glob()                    { return &g_s.b; }
int   n_as_int(S* s)              { return (int)&s->b; }
int*  n_addr_add(S* s)            { return &s->b + 1; }
int*  n_two(S* s, int* q)         { *q = 1; return &s->b; }
