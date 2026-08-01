// W-RERANK — the CALL-ARGUMENT OPERAND VOCABULARY, in front of the differential.
//
// Board #139's repair is to the census's completeness *measure*, which is
// diagnostic: `mark_whole`'s `Err` stays an `Err`, so no byte the port emits can
// change and `census/gate disagreement` cannot see the repair at all. That is
// exactly the trap `docs/ROADMAP.md` §9.13's E4 records — a control that cannot
// see the failure mode is not a control.
//
// What the repair *asserts* is a claim about the shipping path, and the
// differential can grade that: **`eat_call_args` -> `parse_expr` admits, at a
// call-argument operand, a width-4 pointer (since W22), the one-byte-unsigned
// class, and a class-preserving `2C` conversion.** If any of that were false the
// measure would now be WIDER than its emitter, which is the direction that emits
// wrong bytes rather than refusing (§9.13 E4 again), and this TU would mismatch.
//
// So every function below must census in class and the whole TU must be
// `Port=Match`. The refusals that must KEEP refusing are in
// `wrr_arg_vocab_neg.cpp` — two files, because the port emits an obj only when
// every function in the TU is in class, so a positive sharing a file with a
// refused sibling grades nothing (`docs/GAPS.md` §6).

struct S {
    int take_p(int* p);
    int take_v(void* p);
    int take_i(int a);
    int take_two(int* p, int a);
};

int free_p(int* p);
int free_v(void* p);
int free_two(int* p, int a);

// --- a width-4 pointer as a whole call argument ----------------------------
// The class W22 admitted at this position and the completeness measure refused
// until #139, charging a `Blocker::Type(Ptr)` grant for the difference and
// printing `-then-type-ptr` for a construct that was never a blocker.
int mp(S* s, int* p) { return s->take_p(p); }
int fp(int* p) { return free_p(p); }

// --- the same, in a cv-qualified spelling ----------------------------------
int mpc(S* s, int* const p) { return s->take_p(p); }

// --- a class-preserving `2C` conversion on the argument ---------------------
// `int*` -> `void*` is a ptr4 -> ptr4 cv/identity conversion that c2 emits
// nothing for. The measure grew an arm for it only because repairing the
// operand TYPE let the walk reach past the pointer and stop here instead:
// `…-then-convert` went 829 -> 13,325 bodies in one scan.
int mv(S* s, int* p) { return s->take_v(p); }
int fv(int* p) { return free_v(p); }

// --- a pointer beside an int, so the operand run has more than one entry -----
int mtwo(S* s, int* p, int a) { return s->take_two(p, a); }
int ftwo(int* p, int a) { return free_two(p, a); }

// --- the control: the same shapes with no pointer in them -------------------
// If these were the only ones that matched, the pointer cases would be passing
// for a reason that has nothing to do with pointers.
int mi(S* s, int a) { return s->take_i(a); }
