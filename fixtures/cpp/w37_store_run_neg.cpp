// W37 **negative**: the boundary of the store run, one case per refusal row.
// Every body here must census OUT of class and `PortC2` must return
// `NotImplemented` for the TU. Each is a *captured* neighbour that emits
// something the run production does not, not a shape nobody thought about.
//
// The two gates this rung ADDS are the first two, and both are wrong-bytes
// emits if they are missing rather than gaps:
//
//   n_lit   { s->a = 1; s->b = 2; }        39600001 39400002 91630000 91430004
//                                          two `li`s HOISTED, r11 then r10 DESCENDING,
//                                          and at run 4+ interleaved with the stores;
//           { s->a = 1; s->b = u; }        39600001 90830004 91630000
//                                          the two STATEMENTS emitted in the OPPOSITE
//                                          order to the source — c2 schedules around the
//                                          scratch register, and nothing in the
//                                          single-store capture predicts it
//   n_dead  { s->a = u; s->a = w; }        90a30000                 ONE store: c2
//                                          eliminates the dead one, so emitting both
//                                          would be a byte too many at the same address
//
// and the rest are inherited or measured elsewhere:
//
//   n_ret   { …; return u; }               7c6b1b78 … 7c832378   `mr r11,r3` + `mr r3,r4`:
//                                          the result register displaces the base and the
//                                          later stores are RE-BASED onto r11
//   n_ret2  int k first, return s          7c832378              one `mr r3,r4` — free only
//                                          at formal position 0
//   n_load  { s->a = o->b; …}              lwz r11 ; stw r11     a scratch round trip
//   n_calc  { s->a = u+v; …}               add r11 ; stw r11     the same, computed
//   n_or    { s->a = u; s->b |= v; }       read-modify-write, a `0x19` compound assign
//   n_conv  { s->a = f; s->b = v; }        clrlwi r11,r4,24      a narrowing `2C` is a mask
//   n_late  base past the eighth formal    lwz 11,84(1)          frame-homed; needs a frame

struct O { int a; int b; int c; };
struct S {
    int a; int b; int c; int d;
};

// 1. A LITERAL value in a run of more than one — the `li` scheduling rule.
void n_lit(S* s)                            { s->a = 1; s->b = 2; }
// 2. Two statements writing the same bytes of the same base — dead-store
//    elimination collapses them to one instruction.
void n_dead(S* s, int u, int w)             { s->a = u; s->a = w; }
// 3. A returned value that is NOT the first formal.
int n_ret(S* s, int u, int v)               { s->a = u; s->b = v; return u; }
// 4. …and the same thing spelled as a pointer at formal position 1.
S* n_ret2(int k, S* s, int u)               { s->a = u; return s; }
// 5. The stored value is an indirect LOAD — a scratch-register round trip.
void n_load(S* s, O* o, int v)              { s->a = o->b; s->b = v; }
// 6. The stored value is COMPUTED.
void n_calc(S* s, int u, int v)             { s->a = u + v; s->b = v; }
// 7. A compound assignment: a read-modify-write, not a store.
void n_or(S* s, int u, int v)               { s->a = u; s->b |= v; }
// 8. A narrowing conversion on the value is a real `clrlwi` through r11.
void n_conv(S* s, bool f, int v)            { s->a = f; s->b = v; }
// 9. A base past the eighth argument register is stack-homed.
void n_late(int a,int b,int c,int d,int e,int f,int g,S* s,int u)
{ s->a = u; s->b = u; }

// 10. A `volatile` stored VALUE. **This was live on mainline** — the single
//     store leaf emitted a bare `stw r4,0(r3)` for it since W25, and c2 emits
//     `stw r4,28(r1) ; lwz r11,28(r1) ; stw r11,0(r3)`: a volatile parameter is
//     a memory object, c2 homes it in the frame and reloads it, and the body is
//     not a leaf at all. `readers::is_volatile_tag` at a THIRD position — the
//     gate was on the base LOAD (GAPS §6 #13) and the same bit at the `27`/`30`
//     designator positions is free (W35), so only the value position was left.
//     Found by the generated cv axis, not by any fixture.
void n_vol1(S* s, volatile int v)           { s->a = v; }
void n_vol2(S* s, int u, volatile int v)    { s->a = u; s->b = v; }
// 11. A run of THREE or more mixing the two register files. c2 SCHEDULES rather
//     than emitting source order — `{ i=u; j=v; x=w; }` comes back
//     `stfs ; stw ; stw`, the FP store hoisted past both — and `{ a=u; x=v;
//     y=w; }` comes back `stfs ; stw ; stfd`, so it is not "FP first" either.
//     A mixed run of exactly TWO is source order at all 42 ordered type pairs
//     and stays in class; the boundary is where the evidence stops.
struct FS { int i; int j; float x; double y; };
void n_mix3(FS* s, int u, int v, float w)   { s->i = u; s->j = v; s->x = w; }
void n_mix3b(FS* s, int u, float v, double w) { s->i = u; s->x = v; s->y = w; }
