// w-keygen — THE INTERIOR SPLICE SITE, and the axis that varies is its POSITION.
//
// # What this pins, and why it is not `winlfence_opaque_callee.cpp` again
//
// That fixture's N6 already pins the *shape* — "a `CallSeq` of several calls to
// loop-bodied functions defined in the same TU", `?supershuffle@@YAXPAD@Z`'s
// shape, the one function in the whole 878-TU workload the inline fence takes
// back. What no cell anywhere varies is **where in the sequence the inlined
// call sits**, and that is the axis this file sweeps.
//
// It matters because of a fact this lane measured on the real anchor and the
// published record has backwards. `docs/whitebox/WB_INLINE_FINDINGS.md` §6.3 and
// board **#1844** both say c2's inlined `?shuffle2` is *"14 frameless
// re-allocated words, not `?shuffle2`'s own 15-word COMDAT"*. It is not
// re-allocated. Compared word for word out of the reference obj, the 14 words at
// `?supershuffle+0x10` are **byte-identical, 0 of 14 differing**, to
// `?shuffle2`'s own COMDAT with its trailing `blr` dropped — which is exactly
// `SPLICE-0`, the transform `crates/c2-core/src/splice.rs` already ships.
//
// So the distance between what the port has and what the anchor needs is not a
// register allocator. It is that `splice.rs` requires the CALLER's whole emitted
// body to be one call, and here the call is one of several, with real words on
// both sides of it. A widening from "the whole body" to "a call in a sequence"
// has to place the spliced run at the right offset, and **position is the field
// it can get wrong**. Every cell below refuses today; the file exists so that a
// widening which places the run at the wrong index turns a gate row red in the
// same commit rather than three lanes later.
//
// # The cells
//
//   pos1..pos4  four callers, each a four-call sequence over the SAME callee
//               set, with the inlinable callee moved to position 1, 2, 3 and 4.
//               The real anchor is position 2 of 6 and every published word
//               about it is about that one index.
//   two_spliced a sequence in which the inlinable callee appears TWICE, at
//               non-adjacent positions — the COUNT axis, which is independent of
//               position and which `SPLICE-N` (`w-seq`, graded 0 of 548) is the
//               only prior instrument to have touched.
//   framed_tail LAST on purpose. It is the only framed non-leaf here, so it is
//               the only function `coff::plan_labels` mints `$M`/`$T` slots for,
//               while the eight loop bodies above it each charge c2's compiler-
//               label counter. Ordering it last makes a wrong label charge LIVE:
//               a mis-charge upstream lands on this function's slot numbers and
//               the byte compare sees it. (docs/LABEL_COUNTER.md; the counter is
//               shared with c1xx, which is what lane `wb-label` settled.)
//
// # STRUCTURAL BLIND SPOT — stated so absence does not read as coverage
//
//   * Every callee here takes ONE `char *` and returns `void`. Arity, return
//     type and pointee type are NOT varied, so nothing here would catch a
//     splice that is right about position and wrong about argument mapping.
//     The real anchor is also one-`char*`-one-`void`, so this file inherits the
//     anchor's blind spot rather than covering it.
//   * The inlinable callee's parameter register and the caller's happen to
//     COINCIDE (both `r3`) — which is precisely why the anchor's splice is
//     byte-identical. A cell where they do not coincide would separate
//     `SPLICE-0` from `SPLICE-P`, and there is no such cell here.
//   * There is no floating-point literal anywhere in this file, deliberately:
//     board **#2343** says an IL chain read past one is a corrupted stream and
//     any depth quoted off it is void.
//   * The sizes below are TUNED against real `c2` 16.00.11886.00 at the
//     workload's own flags. They are not portable to another optimisation mode:
//     `docs/whitebox/WB_INLINE_FINDINGS.md` measures the loop-class ceiling
//     moving with FAVOR-SPEED, so at `/Ox` a different set of callees inlines
//     and the position sweep is about different indices. The gate compiles this
//     at every mode, and at the modes where the split moves the cells still
//     refuse — refusal is mode-independent, the *reason* is not.

// The two leaves. Both are inlined into every callee below at /O1, which is what
// makes the callees' emitted sizes — not their source sizes — the axis.
static void kg_swap(char &a, char &b) {
    char t = a;
    a = b;
    b = t;
}

static int kg_roll(int i) {
    i += 19;
    i %= 32;
    return i;
}

// SMALL — modelled on `?shuffle2@@YAXPAD@Z`, the one callee of the real anchor
// c2 inlines. No `kg_roll`, so the loop body is two swaps and nothing else.
void kg_small(char *c) {
    for (int i = 0; i < 8; i++) {
        kg_swap(c[(7 - i) * 4 + 1], c[i * 4 + 2]);
        kg_swap(c[(7 - i) * 4], c[i * 4 + 3]);
    }
}

// BIG1..BIG3 — modelled on `?shuffle3`…`?shuffle6`, the five c2 declines. Each
// adds one `kg_roll` to the loop body, which is the whole difference between 60
// bytes and 84 on the real TU.
void kg_big1(char *c) {
    for (int i = 0; i < 8; i++) {
        kg_swap(c[kg_roll((7 - i) * 4 + 1)], c[i * 4 + 2]);
        kg_swap(c[(7 - i) * 4], c[i * 4 + 3]);
    }
}

void kg_big2(char *c) {
    for (int i = 0; i < 8; i++) {
        kg_swap(c[(7 - i) * 4 + 1], c[i * 4 + 2]);
        kg_swap(c[kg_roll((7 - i) * 4)], c[i * 4 + 3]);
    }
}

void kg_big3(char *c) {
    for (int i = 0; i < 8; i++) {
        kg_swap(c[(7 - i) * 4 + 1], c[kg_roll(i * 4 + 2)]);
        kg_swap(c[(7 - i) * 4], c[i * 4 + 3]);
    }
}

// ---- the POSITION sweep -----------------------------------------------------

void kg_pos1(char *c) {
    kg_small(c);
    kg_big1(c);
    kg_big2(c);
    kg_big3(c);
}

void kg_pos2(char *c) {
    kg_big1(c);
    kg_small(c);
    kg_big2(c);
    kg_big3(c);
}

void kg_pos3(char *c) {
    kg_big1(c);
    kg_big2(c);
    kg_small(c);
    kg_big3(c);
}

void kg_pos4(char *c) {
    kg_big1(c);
    kg_big2(c);
    kg_big3(c);
    kg_small(c);
}

// ---- the COUNT axis ---------------------------------------------------------

void kg_two_spliced(char *c) {
    kg_small(c);
    kg_big1(c);
    kg_small(c);
    kg_big2(c);
}

// ---- the label tripwire, LAST -----------------------------------------------

void kg_framed_tail(char *c) {
    kg_big1(c);
    kg_big2(c);
}
