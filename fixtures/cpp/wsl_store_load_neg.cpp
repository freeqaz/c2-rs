// WSL **negative**: the boundary of the load-valued store, one case per refusal.
// Every body here must census OUT of class and `PortC2` must return
// `NotImplemented` for the TU. Each is a *captured* neighbour that emits
// something this production does not — not a shape nobody thought about.
//
// The refusals this rung ADDS are the first four, and each of them is a
// wrong-bytes emit if it is missing rather than a gap. Every word below was read
// off a reference obj (`work/wsl/probe/p1.cpp`, `p4.cpp`, `p7.cpp`):
//
//   n_mixf  { d->a=s->a; d->b=u; }      lwz r11,0(r4) ; stw r5,4(r3) ; stw r11,0(r3)
//                                       the load is HOISTED and its store SINKS past
//                                       the next statement — the two statements come
//                                       back in the opposite order to the source
//   n_mixl  { d->a=s->a; d->b=2; }      lwz r11 ; li r10,2 ; stw r10,4(r3) ; stw r11,0(r3)
//                                       and the literal gets a SECOND scratch register,
//                                       where a pure run uses only r11
//   n_self  { d->a=d->b; d->b=d->a; }   lwz r11,4(r3) ; stw r11,0(r3)   — ONE pair.
//                                       c2 forwards through the pair and the second
//                                       statement is gone entirely
//   n_long8 eight statements, two params  r11 … r5, **r4** — the descent reaches the
//                                       source base's own register on its last use;
//                                       one more statement and it WRAPS to r11 instead
//                                       (`L9` in `p7.cpp`), which needs a liveness
//                                       model this port does not have
//
// The rest are inherited boundaries restated at the new position: a conversion
// on the loaded value, a `volatile` pointer FORMAL, an out-of-range argument
// position, and the aggregate-pointer `27` whose width nibble is the pointer's
// alignment rather than the pointee's size.

struct S { int a, b, c, d, e, f, g, h, i, j; };
struct W { char c; short h; int i; long long q; float f; double g; };
struct F { float fa[8]; double da[8]; };

// 1. a run MIXING a loaded value with a formal one — c2 schedules.
void n_mixf(S* d, S* s, int u) { d->a = s->a; d->b = u; }
// 2. …and with a literal one, which additionally takes a second scratch.
void n_mixl(S* d, S* s)        { d->a = s->a; d->b = 2; }
// 3. one object both LOADED FROM and STORED TO in the same run — c2 forwards
//    through the pair and eliminates the dead half.
void n_self(S* d)              { d->a = d->b; d->b = d->a; }
// 4. a run past the plain scratch descent: eight statements with two parameters
//    is the first length where the descent reaches a parameter's register.
void n_long8(S* d, S* s) { d->a=s->a; d->b=s->b; d->c=s->c; d->d=s->d;
                           d->e=s->e; d->f=s->f; d->g=s->g; d->h=s->h; }
// 5. …and the bound moves with the parameter count: six statements with four
//    parameters is already past it (`P8` in `p7.cpp` skips r6/r5, uses the two
//    dead `int` registers, then wraps).
void n_long4(int k0, int k1, S* d, S* s) { d->a=s->a; d->b=s->b; d->c=s->c;
                                           d->d=s->d; d->e=s->e; d->f=s->f; }
// 6. two statements writing OVERLAPPING bytes of one destination. c2 keeps BOTH
//    here — the source may alias the destination, so the first store is
//    observable — where a run of formal values has the second one eliminated.
//    Two opposite behaviours behind one gate, so the gate refuses both.
void n_over(S* d, S* s)        { d->a = s->a; d->a = s->b; }
// 7. a WIDENING conversion on the loaded value: `lbz r11 ; extsb r11,r11 ; stw`
//    — a real instruction between the two.
void n_wide(W* d, W* s)        { d->i = s->c; }
// 8. a NARROWING one. Free in the reference (`lwz r11 ; stb r11`), and refused
//    anyway: admitting the free direction means deciding it from two type
//    triples, and only the widening has been captured at more than one width.
void n_narrow(W* d, W* s)      { d->c = (char)s->i; }
// 9. a `volatile` pointer FORMAL is a memory object — c2 homes it in the frame
//    and reloads it, so the body is not a leaf at all. (A pointer *to* volatile
//    is a different bit position and is free; it is in the positive file.)
void n_volp(S* d, S* volatile s) { d->a = s->a; }
// 10. the source base past the eighth argument, where it is stack-homed.
void n_arg9(int a0,int a1,int a2,int a3,int a4,int a5,int a6,S* d,S* s) { d->a = s->a; }
// 11. an 8-byte element reached through a subscript. The `27` re-types the
//     address to a pointer-to-ARRAY, whose tag width nibble is the POINTER's
//     alignment (4) and not the element's size (8), so the designator's
//     announced width contradicts the `30`. The same limit the indirect-load
//     leaf draws, at the same position, for the same reason.
void n_arrq(F* d, F* s)        { d->da[0] = s->da[0]; }
