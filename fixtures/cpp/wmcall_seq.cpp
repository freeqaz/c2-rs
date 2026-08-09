// w-mcall — the MEMBER CALL IN STATEMENT-SEQUENCE POSITION, positive cells.
//
// Every body here is a sequence of statement-position calls at least one of
// which is a **member** call, and every one of them is graded byte-exact against
// real `c2.dll` under wibo by the fixture gate at `/O1`, `/Ox`, `/O2`, `/Od` and
// both the `/EHsc` and `/Gy` crossings.
//
// **These are codegen cells, and NOT ONE BYTE OF `crates/c2-core` was written
// for them.** `p->m(a…);` is `m(p, a…)` on this ABI, so a statement-position
// member call is a statement-position call with one more argument slot; the
// receiver is appended to the argument list as slot 0 and the body is the
// `BodyShape::CallSeq` the port has emitted since #35 step 2. The whole rung is
// two reader routes — `mcall_tail::try_parse_member_tail_call` (this call is the
// FIRST statement) and `calls::parse_call_sequence_from` (it is a later one).
//
// What the cells separate, read off `work/w-mcall/probe/p2.obj` and
// `p4.obj` word by word before any of this was written:
//
//   P1  Class B, ONE saved GPR      mr r31,r3 · bl · mr r3,r31 · bl
//   P2  three statements            the same, with the receiver restored twice
//   P3  Class B, TWO saved GPRs     r31 = this, r30 = the argument; the second
//                                   call restores r4 THEN r3 — descending slot
//                                   order, which the free-function sequence
//                                   already pins
//   P4  two distinct receivers      the SECOND receiver is the one parked
//                                   (mr r31,r4), because the first call's is
//                                   already in r3
//   P5  member THEN free            Class A — `s` dies at its own call, so the
//                                   frame is the 3-word prologue and nothing is
//                                   saved
//   P6  free THEN member            the other reader route: this one enters
//                                   through `parse_call_shape` and reaches the
//                                   member call inside the sequence LOOP.
//                                   Its base census key was `call-token-0xB9`
//   P7  a literal tail behind it    `SeqTail::Lit` — `li r3,5` after the last bl
//   P8  the receiver is `this`      the workload's dominant spelling: a member
//                                   function calling its own methods, where the
//                                   parked formal is the implicit `this`
//
// Board rows #1960–#1963; `docs/rungs/2026-08-08-w-mcall.md`.

struct S {
    int m;
    void a();
    void b();
    void set(int);
    void both();
};

void wmcall_free();

// P1 — the minimal member-call sequence. The receiver is live across the first
// `bl`, so this class is Class B from its smallest member: there is no Class A
// two-member-call body, because a member call needs its receiver.
void wmcall_two(S *s) {
    s->a();
    s->b();
}

// P2 — three statements, so the restore is emitted twice.
void wmcall_three(S *s) {
    s->a();
    s->b();
    s->a();
}

// P3 — an explicit argument beside the receiver: two callee-saved GPRs.
void wmcall_two_args(S *s, int x) {
    s->set(x);
    s->set(x);
}

// P4 — two receivers. The first call's receiver is already in r3 and dies
// there; the second's is what gets parked.
void wmcall_two_recv(S *s, S *t) {
    s->a();
    t->a();
}

// P5 — a member call followed by a free one, and nothing lives across the
// first `bl`: Class A, the 96-byte frame with no `std`/`ld` pair.
void wmcall_then_free(S *s) {
    s->a();
    wmcall_free();
}

// P6 — the free call FIRST, so the member call is read by the sequence loop
// rather than by the member-tail production. Both routes must exist; a lane
// that shipped only the first would leave this cell blocked.
void wmcall_free_then(S *s) {
    wmcall_free();
    s->a();
}

// P7 — the literal tail. `SeqTail::Lit` is shared with the free-function
// sequence and needed no widening; the cell is here because a tail that reads
// nothing is the one place a receiver could have been dropped silently.
int wmcall_tail_lit(S *s) {
    s->a();
    return 5;
}

// P8 — the receiver is the implicit `this`. This is the spelling the 878-TU
// workload is made of, and it is a different formals list (one implicit
// parameter, no explicit ones), not a re-spelling of P1.
void S::both() {
    a();
    b();
}
