// W-PARK — the `_neg` half of `wpark_lit_permuted.cpp`. Board **#1920**.
//
// Every cell here is a shape ONE construct away from the positive file's class
// and out of it, and each reaches a DISTINCT clause key. The keys were read
// with a probe patch that was applied, run and reverted
// (`work/w-park/decline_probe.md`, output in `work/w-park/neg_clauses.txt`),
// because board **#1704** makes `c2rs census` report only the fall-through
// blocker — on an unpatched tree every cell below reads `expr-cmp-eq`, which
// says nothing about why any of them declined. w-cfgclass §6.2's method, paying
// an eighth time.
//
// **Three of these clauses were bought by a PRIOR lane with `Port=Mismatch`**
// (`w-memcpy`'s `callseq-multiarg-lit-*` fence: no literal in slot 0, at most
// one literal, a guarded early return). They are graded here rather than
// trusted, because this lane's widening is the first thing that could have
// reached past them — before it, `call-arg-lit-permuted` refused the whole
// family first and none of the three was ever the deciding clause on a
// permuted list.

void c2(void *, unsigned int);
void c2l(unsigned int, void *);
void c3(void *, void *, unsigned int);
void c3ll(void *, unsigned int, unsigned int);
void c9(void *, void *, void *, void *, void *, void *, void *, void *,
        unsigned int);
void c0();

// n1 — NO GUARD. The park exists only in front of a guarded early return, and
// `callseq-multiarg-lit`'s clause (b) is the fence. GRID-P control `c_ng0`.
unsigned long n1(void *a0, void *a1) {
    c2(a1, 72);
    return 0;
}

// n2 — the literal in SLOT 0, with the other slot IN PLACE. c2 hoists the
// constant into the entry block, INVERTS the branch and drops a word; clause
// (c') was bought with two `Port=Mismatch`.
//
// **The in-place part is load-bearing and was found by the probe, not by
// reading.** Written as `c2l(72, a0)` — the shape GRID-P's `c_s0` control uses
// — this cell reaches `callseq-early-return-permuted-args` instead, because
// the permutation fence runs BEFORE the literal fence and a literal in slot 0
// makes `slot_sources` non-injective. The `callseq-multiarg-lit-*` clauses are
// reachable only once the formals are in place, which is a fact about the
// fence ORDER that no prior file states.
unsigned long n2(void *a0, void *a1) {
    if (a0 == 0) return 5;
    c2l(72, a1);
    return 0;
}

// n3 — TWO literals, formals in place. Clause (c). Same fence-order caveat as
// n2: with the formal moved this cell reaches the permutation fence first.
unsigned long n3(void *a0, void *a1) {
    if (a0 == 0) return 5;
    c3ll(a0, 72, 5);
    return 0;
}

// n4 — a SECOND call. Clause (a): GRID-L put a literal in the first call of
// every cell it generated and none in a later one, so a later call's literal
// has no witness at all.
//
// The second call takes NO formal on purpose. Any second call that reads one
// makes that formal live across the first, which is Class B, and the body then
// stops at `callseq-saved-with-first-call-setup` — a different clause, and the
// one this cell reported before the probe was read.
unsigned long n4(void *a0, void *a1) {
    if (a0 == 0) return 5;
    c3(a0, a1, 72);
    c0();
    return 0;
}

// n5 — a literal too WIDE for `li`'s signed-16-bit immediate. c2 emits
// `lis`+`ori`, two words where this class emits one.
unsigned long n5(void *a0, void *a1) {
    if (a0 == 0) return 5;
    c2(a1, 0x30000);
    return 0;
}

// n6 — NINE argument slots. It stops at `call-args-overflow`, the ARGUMENT
// bound in `eat_call_args`, and NOT at `callseq-over-eight-formals`, the
// FORMAL bound this cell was written for: the argument list overflows one word
// before the formals list does, so the formal bound has no cell here and is
// stated as untested rather than credited. Recorded rather than reworded
// (w-osfinfo §5's method).
unsigned long n6(void *a0, void *a1, void *a2, void *a3, void *a4, void *a5,
                 void *a6, void *a7) {
    if (a0 == 0) return 5;
    c9(a1, a0, a2, a3, a4, a5, a6, a7, 72);
    return 0;
}

// n7 — the permutation the park's own unimodal clause REFUSES. GRID-P's
// `p021`/`p201` cells are 8 of the 14 refusals in that grid, and they refuse
// here for the same reason: when the first guard cannot anchor, c2 scans on to
// later guards and past that to the cycle minimum, and that clause was
// re-fitted by every population that measured it (w-mmio grids 1→2→3). Board
// #260's warning applies, so the population comes out as a gap.
unsigned long n7(void *a0, void *a1, void *a2, void *a3) {
    if (a0 == 0) return 5;
    c3(a2, a1, 72);
    return 0;
}
