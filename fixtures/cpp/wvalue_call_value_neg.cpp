// w-value — the member-call VALUE model, NEGATIVE cells: the fences.
//
// Every cell here is a member call in expression position whose head **does not
// move**, and each one is held by a DIFFERENT clause. Verified per cell against
// the parent commit's binary: all six report the identical key before and after
// the value model, and the six keys are distinct — a `_neg` file whose cells
// share a clause tests one thing six times.
//
//   N1  `expr-call-in-expr-recv-load-whole`              the statement-layer
//                                                        fence, at the `41`
//                                                        result annotation
//   N2  `expr-call-in-expr-recv-load-then-branch-brfalse` the CONTROL-FLOW half
//                                                        of the fence, at `38`
//                                                        — and the key already
//                                                        names the branch, which
//                                                        is why yielding to the
//                                                        call keeps two facts
//                                                        instead of one
//   N3  `expr-call-in-expr-chained-whole`                the CHAIN: the walk must
//                                                        NOT stop at the inner
//                                                        `4C`, and must still be
//                                                        fenced at the `41`
//   N4  `expr-call-in-expr-op-0x9B`                      the walker's own PRICE —
//                                                        a `9B` temporary
//                                                        receiver it cannot
//                                                        tokenize. 50,023 bodies
//                                                        / 1,590 emitted on the
//                                                        878-TU workload, 69 % of
//                                                        the whole residue
//   N5  `expr-call-in-expr-other`                        a data-symbol address
//                                                        push as an argument —
//                                                        NOT a call, deliberately
//                                                        not handled (~18 % of the
//                                                        bucket, §2)
//   N6  `expr-call-in-expr-recv-object-whole`            a NAMED-OBJECT receiver,
//                                                        fenced at the `41`
//
// The first three are the fence; the last three are the model declining to
// tokenize. Both directions matter: a fence that fired on everything and a
// walker that consumed everything would each look like this file passing.

struct Obj {
    int Get();
    void Set(int);
};
struct V {
    int x;
    int G();
};
struct L {
    L *Next();
    int Val();
};
V mk();
extern Obj gO;
extern int gA[4];
int u2(int *);

// N1 — nothing behind the call but the statement's own `41`.
int wvalue_neg_stmt_end(Obj *p, int a) { return a + p->Get(); }

// N2 — a conditional branch behind the call.
int wvalue_neg_branch(Obj *p, int a, int b) {
    int t = a;
    if (b + p->Get()) {
        t = b;
    }
    return t;
}

// N3 — a member-call CHAIN. Two `BD` regions, and the inner `4C` is followed by
// a `99` that binds its result as the next receiver.
int wvalue_neg_chain(L *p, int a) { return a + p->Next()->Val(); }

// N4 — a by-value temporary receiver, spelled `9B`. The walker has no width for
// it and returns `None`, so the refusal is byte-for-byte the one this arm has
// always raised.
int wvalue_neg_temp_receiver(int a) { return a + mk().G(); }

// N5 — an array decaying to a pointer as a call argument. There is no `BD` at
// the head of this `26`'s production at all.
int wvalue_neg_data_addr(int a) { return a + u2(gA); }

// N6 — a named-object receiver, whose address decay is the `2C` that
// `IL_CALL_IN_EXPR.md` §3.1 records at 17.9 % of the bucket.
int wvalue_neg_named_object(int a) { return a + gO.Get(); }
