// W8 NEGATIVE — the two `cflow-if-1` shapes next door that must stay REFUSED.
// Both census `cflow-if-1` exactly like w8_cond_tail.cpp does; neither is a
// conditional branch in the obj, and emitting one for either would be wrong
// bytes rather than a gap.
//
// `b2` is fold **band 2** (docs/CFG_SHAPE.md §3.5): one successor IS the
// function's epilogue, so c2 emits a conditional RETURN and no branch, no label
// and no displacement at all —
//
//     cmplwi cr6,r3,0 ; bnelr cr6 ; li r11,1 ; stw r11,0(r4) ; blr
//
// Band 2 is the majority band: six of seven `cflow-if-1` leaf probes fold, and
// **both** real `cflow-if-1` functions in the frontier TU `src/system/utl/Pool.cpp`
// are band-2 folds. An implementer who builds a branch lowering and grades it on
// `Pool.cpp` grades nothing. That is why the W8 class requires BOTH arms to end
// in a tail call: a body that does cannot be band 2 (neither successor is the
// epilogue) and cannot be band 1 (neither arm is a constant), so the class sits
// inside band 3 by construction rather than by reproducing §3.5's cost model,
// which that section declines outright.
//
// `merged` is board **#193**: both arms end in a call to the SAME callee. c2
// tail-merges the two `bl` sites, which empties the then-block, hoists its
// argument above the compare and **inverts the layout** — the one measured
// refutation of "block order is IL statement order" (§3.4.1). Block order is
// downstream of code motion, and this port models no code motion.
//
// If either of these ever censuses in class, the W8 gate has over-accepted.
void h3(void *, unsigned long, void *);
int gi(int);

void b2(void *p, int *q) {
    if (p == 0) {
        return;
    }
    *q = 1;
}

int merged(int a, int b) {
    if (a == 0) {
        return gi(1);
    }
    return gi(b);
}
