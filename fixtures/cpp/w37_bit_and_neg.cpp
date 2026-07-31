// **Negative, and the whole of W37.** W37 shipped no widening: the `&` operator
// was MEASURED on the 878-TU dc3 workload and is worth **0 functions**, so every
// case here must census OUT of class and the whole TU must read
// `Port=NotImplemented`.
//
// This file exists for the reason `GAPS.md` §6 gives for negative fixtures in
// general — "a negative fixture can silently stop being negative when a later
// rung widens the gate" — and for one more that is specific to a *declined* row.
// The decline rests on a measurement (`docs/rungs/2026-07-31-bit-and-declined.md`)
// whose whole content is *which shapes the real workload does and does not
// contain*. If a later rung admits `0B` because these bodies look easy, the
// measurement that says they are worthless has to fail loudly rather than be
// re-derived from memory.
//
// Each case carries its measured cost on the workload. `0` means the row exists
// and holds no function this port could reach; a number is the census row.

struct S {
    int flags;
    int  Flags() const;
    int  Mask(int) const;
};
int gk(int);

// ---------------------------------------------------------------------------
// 1. THE ROW. `expr-call-in-expr-recv-load-then-bit-and`, 102,382 functions,
//    5.5 % of everything blocked and #4 on the board when W37 opened it.
//    102,374 of them are exactly this: a member call on a loaded receiver, its
//    result masked, and the mask feeding a CONDITIONAL BRANCH. 102,379 of the
//    row is `calls-2plus` and 102,370 is `cflow-if-1`, so it needs a frame and
//    basic blocks before it needs an `and` — and control flow was declined the
//    same day at 718 realized against a 48,102 row.
//    Cost of refusing: **0** — measured, not argued (see 3 below).
int n_if_call_mask(const S *p) {
    if (p->Flags() & 4) {
        return gk(1);
    }
    return 0;
}

// 2. The same shape with the mask on the call's ARGUMENT rather than its result,
//    so that the `0B` sits inside the argument region instead of after the `4C`.
//    Same row, same verdict; here to keep the refusal from being read as
//    positional.
int n_if_call_arg_mask(const S *p, int k) {
    if (p->Mask(k & 3)) {
        return gk(2);
    }
    return 0;
}

// 3. THE SIBLING ROW. `expr-bit-and`, 32,381 functions — the free-standing mask
//    that never reaches `mcall` at all. 32,368 of it (99.96 %) redistributes to
//    `expr-brtrue` the moment the token is admitted, and the census gain over
//    2,462,571 bodies is **exactly 0**. So the two rows together — 134,763
//    functions, 7.2 % of everything blocked — release onto a branch and nothing
//    else. That is the measurement this fixture protects.
int n_if_mask(unsigned x) {
    if (x & 1) {
        return gk(3);
    }
    return 0;
}

// 4. The shape W37's estimate predicted would carry the row, and which the
//    workload does not contain: a mask as a **value**. Not one function in
//    102,382 completes on `bit-and` alone — the `-whole` count is 0.
int n_ret_mask(unsigned x) {
    return x & 7;
}
int n_ret_call_mask(const S *p) {
    return p->Flags() & 7;
}

// 5. The mask compared to zero, `(x & k) != 0` — the other value-position
//    spelling the estimate named. Blocks one token further on, at the compare.
int n_cmp_mask(unsigned x) {
    return (x & 8) != 0;
}

// 6. The operator's siblings, EXCLUDED FROM `BARE_BINARY_OPS` on purpose.
//    `09`/`0A`/`0C`/`0D` are named by `expr_opcode_name` and are not in the
//    grantable set, because neither of the two pieces of evidence `0B` has (a
//    capture witness that the token is bare, and a 1:1 redistribution over the
//    workload) has been taken for them. Their combined cost is **4,247
//    functions** — 3,686 `expr-shr`, 557 `…-then-shr`, 1 `expr-shl`, 1
//    `…-then-bit-or`, 1 `…-then-bit-xor`, 1 `…-then-or-or`, 3 `…-then-and-and`
//    — so leaving them UNMEASURED is a number rather than an argument.
int n_shr(unsigned x) {
    return int(x >> 3);
}
int n_shl(unsigned x) {
    return int(x << 3);
}
int n_or(unsigned x) {
    return int(x | 1);
}
int n_xor(unsigned x) {
    return int(x ^ 1);
}
