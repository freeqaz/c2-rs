// WLR **negative**: the boundary of the one-value literal store run. Every body
// here must census OUT of class and `PortC2` must return `NotImplemented` for
// the TU. Each is a *captured* neighbour that emits something this production
// does not — not a shape nobody thought about.
//
// The whole file is one argument: **the moment a run carries two distinct
// literal values, c2's register allocation and its store order both stop being
// derivable from anything here.** Four rules were fitted to the grid below and
// each is refuted by another member of it, so the class is drawn at one value.
//
//   n2_two   { a=1; b=2; }              39600001 39400002 91630000 91430004
//                                       two `li`s, r11 then r10, first-use order
//   n2_perm  { a=1; b=2; c=1; d=2; }    39400001 39600002 …
//                                       the SAME two values, 1 -> r10 and 2 -> r11:
//                                       PERMUTED against n2_two, and both runs have
//                                       the same use counts, the same first-use
//                                       order and the same live-range lengths
//   n5_perm  { a=1; b=2; c=3; d=2; e=1;} 1 -> r10, 2 -> r11, 3 -> r9 — permuted again,
//                                       and against n4_ok below, which is the same
//                                       three values one statement shorter and comes
//                                       back 1 -> r11, 2 -> r10, 3 -> r9
//   n4_ok    { a=1; b=2; c=3; d=1; }    first-use order, r11/r10/r9 — the case that
//                                       refutes "use count" and "live-range length"
//   n5_sched { a=1; b=1; c=2; d=2; e=2;} 2 -> r11, 1 -> r10, and the STORES come back
//                                       2,0,1,3,4 — not source order at all
//   n6_recyc { a=1; b=2; c=3; d=4; }    FOUR distinct values into three registers:
//                                       r11 is recycled mid-body and a store is
//                                       hoisted between the `li`s to free it
//   n_mix    { a=1; b=u; }              39600001 90830004 91630000 — the two
//                                       STATEMENTS emitted in the OPPOSITE order to
//                                       the source; a literal mixed with a formal is
//                                       scheduled, and `MA`/`N1`/`N3` disagree about
//                                       where the literal store lands
//   n_dead   { a=1; a=1; }              ONE store: c2 eliminates the dead one
//                                       (inherited from W38's overlap gate)
//   n_load   { a=0; b=s->c; }           a literal mixed with an indirect load
//                                       (inherited from WSL's kind gate)
//
// `n4_ok` is deliberately in this file even though its own bytes are the
// first-use rule: it is the witness that makes the rule unfittable, and a
// negative fixture whose refusals are all "obviously hard" proves nothing.

struct S { int a; int b; int c; int d; int e; int f; };

// Two distinct values, at the lengths where the allocation permutes.
void n2_two (S* s)         { s->a = 1; s->b = 2; }
void n2_perm(S* s)         { s->a = 1; s->b = 2; s->c = 1; s->d = 2; }
void n4_ok  (S* s)         { s->a = 1; s->b = 2; s->c = 3; s->d = 1; }
void n5_perm(S* s)         { s->a = 1; s->b = 2; s->c = 3; s->d = 2; s->e = 1; }
// …and the one where the STORES are reordered as well as the registers.
void n5_sched(S* s)        { s->a = 1; s->b = 1; s->c = 2; s->d = 2; s->e = 2; }
// Four distinct values: r11 is recycled, so the run is not even a fixed
// register assignment.
void n6_recyc(S* s)        { s->a = 1; s->b = 2; s->c = 3; s->d = 4; }

// A literal mixed with a formal — scheduled, and in the opposite order to the
// source at length 2.
void n_mix  (S* s, int u)  { s->a = 1; s->b = u; }
void n_mix2 (S* s, int u)  { s->a = u; s->b = 1; }
void n_mix3 (S* s, int u, int v, int w)
                           { s->a = 1; s->b = u; s->c = v; s->d = w; }

// The same literal twice to the same member: c2 eliminates the dead store.
void n_dead (S* s)         { s->a = 1; s->a = 1; }

// A literal mixed with an indirect load.
void n_load (S* s, S* t)   { s->a = 0; s->b = t->c; }
