// **MUST REFUSE — lane `w-pool` (#2563).** `wpool_store_leaf_member_value.cpp`
// with **one thing added**: a second store. Same struct, same two operands, same
// cast, no control flow anywhere in the body.
//
// This is `?Free@Pool@@QAAXPAX@Z` with its guard removed, and it is the **head
// of `src/system/utl/Pool.cpp`'s whole chain** — the census reports
// `expr-op-0x27` on 2 of that TU's 3 functions and on 2 of
// `EncryptXTEA.cpp`'s 5, and this two-statement body is the smallest thing in
// the workload's neighbourhood that produces it.
//
// ## What the matched pairs exclude, each measured on this tree
//
// Every one of these is a `c2rs census` + `c2rs diff` on a cell that differs
// from another by one axis (`work/w-pool/probe/`, lane `w-pool`'s base binary):
//
//     p4   p->mFree = (char*)v;                      store-leaf  Match  EXACT
//     p9   *(void**)v = p->mFree;                    store-leaf  Match  EXACT
//     p5   *v = p->mFree;              (no cast)     store-leaf  Match  EXACT
//     p6   q->a = (char*)v; q->b = (char*)v;         store-run   Match  EXACT
//     p10  *a = 0; *b = 0;             (two bases)   store-run   Match  EXACT
//     -----------------------------------------------------------------------
//     p1   THIS BODY                                 GAP  expr-op-0x27
//     p7   THIS BODY, statements reversed            GAP  expr-op-0x27
//     p8   THIS BODY, typed, no cast                 GAP  expr-op-0x27
//     p11  *v = p->mFree; *v = 0;                    GAP  expr-op-0x27
//
// So it is **not** the cast (p9 has it and matches, p8 lacks it and refuses),
// **not** the statement order (p7), **not** the number of base pointers (p10
// stores through two and matches), **not** the run length (p6 is two stores and
// matches) and **not** control flow (every cell above is `cflow-straight`).
//
// ## The fence, named
//
// It is `leaf_store::parse_store_stmt`'s **value** position. A `store-run`
// admits stores whose values are formals or one repeated literal — the
// hoist/allocate/reorder questions the doc comment there enumerates all vanish
// in that class — and it refuses the moment a stored value is a member **load**.
// The refusal surfaces as `expr-op-0x27`, the byte-offset add computing the
// loaded member's address, because that is the token the walk stops on.
//
// `store-leaf` admits exactly that value, which is why the positive control
// beside this file matches. **The two productions disagree about one operand
// position**, and that disagreement is `Pool.cpp`'s first rung.
//
// ## Why this is a refusal and not a widening
//
// The port emits nothing for this body today, and that is the correct outcome
// until the run's schedule is measured rather than assumed: with two or more
// distinct non-literal values c2 chooses which of r11/r10/r9 carries each and in
// what order the stores land, and `leaf_store`'s own doc records that boundary
// being fixed by a *reorder* neighbour rather than by argument. One witness is
// not a rule (#2306).

struct WPoolP {
    char *mFree;
};

void wpool_store_run(WPoolP *p, void *v) {
    *(void **)v = p->mFree;
    p->mFree = (char *)v;
}
