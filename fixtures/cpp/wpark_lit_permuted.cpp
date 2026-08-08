// W-PARK — a LITERAL argument beside a PERMUTED formal, behind a guarded early
// return, legalised by the entry-block park. Board **#1920**.
//
// ## This file is a `match` at BOTH `/O1` and `/Ox`, and that was MEASURED
// ## rather than assumed
//
// The sentence that stood here on the first draft — *"at `/Ox` this file is
// deliberately 0/N in class"* — was written from `shapes::early_return`'s
// module doc and is **false**. It is **5/5 in class and a byte-exact `match`
// at `/Ox` too**, and the two objs are not the same obj:
//
// ```text
//   /O1   five COMDAT .text sections, 56 / 84 / 68 / 84 / 56 B, each arm's
//         `b` sharing ONE epilogue
//   /Ox   ONE packed 444-byte .text, every arm carrying its OWN
//         `addi/lwz/mtlr/blr` — the epilogue duplication that module doc
//         describes — with 4 bytes of padding between bodies
// ```
//
// So the mode split the doc records is real and the port already models both
// sides of it; what the doc does not license is the inference *"a class that
// splits by mode must refuse one of them"*. Measured with
// `c2rs gap --flags-file` at each profile, per file, and the claim is written
// from that output. The mode gate still lives in the PARSER (board #1638);
// this class simply is not one of the shapes it gates.
//
// ## The composition, and why it had no witness before this file
//
// Two prior grids each held one half of this cell fixed:
//
//  * `w-mmio`'s **886** cells gridded the entry-block park over permutations
//    with **no literal slot** — that is `codegen::calls::seq_entry_park` and
//    `park_in_class`, 496 of 496 on the sub-class where the first guard can
//    anchor;
//  * `w-memcpy`'s **GRID-L**, 747 cells, gridded the literal slot over
//    argument lists that were **already in place** — that is the
//    `callseq-multiarg-lit-*` fence, 416 of 416.
//
// Neither compiled a cell with both, and `lit_arg_tail_call` refused every one
// of them under `call-arg-lit-permuted`. That refusal is a statement about the
// **tail-call** site, where there is no park and no prologue and the argument
// setup *is* the body; at the SEQUENCE site the permutation is decided
// downstream by `park_in_class`, on the same [`slot_sources`] view in which a
// literal slot is a **fixed point** — a value about to be materialised there by
// a `li`, participating in no cycle. `slot_sources`' own doc says so, and says
// it is *"shared by the parser's fence and the emitter's backstop so the two
// cannot disagree about which slots move"*. The two could not disagree; they
// were never asked.
//
// GRID-P (`work/w-park/gridg.py`, frozen at `81ee6705` before its first
// `cl.exe`) is that cell: **45 cells, 45 graded, 31 `match`, 0 `mismatch`**,
// and against the same grid at base **16 moved `vocab-gap` → `match` and 0
// moved the other way**, read as a SET by name.
//
// ## What each function here is for
//
// `p2` is the two-formal head of the class; `p3lo`/`p3hi` are the two
// three-formal permutations; `p4` is the four-formal case with the deepest
// descent the park's unimodal clause admits. `id3` is the CONTROL: the identity
// permutation, which was in class before this lane and must stay `match` — it
// is the cell-level neutrality witness, and a widening that moved it would be
// changing a class rather than reaching a new one.
//
// **The literal is never in slot 0** and there is never more than one: both are
// clauses of the `callseq-multiarg-lit-*` fence, both were bought by a prior
// lane with `Port=Mismatch`, and both are graded on the `_neg` side of this
// pair rather than assumed here.
//
// ## The workload instance, and the reason this is a fixture and not a
// ## conversion
//
// `src/xdk/nuispeech/mmio.cpp`'s `mmioGetInfo` is exactly `p3lo` with the
// callee named `memcpy`: `memcpy(pmmioinfo, hmmio, 0x48)` reads
// `[Formal(1), Formal(0), Lit(72)]`. Its 84 bytes are reproduced word for word
// by `work/w-park/cells/l3.cpp`, which is a `match`. It is **still not
// emitted**, because `memcpy` arrives as `expr-intrinsic-memcpy` and lane
// `w-memcpy` measured that clause's rule not to exist: `M-ALWAYSCALL` scores
// **114 of 232**, four separately frozen thresholds all miss, and GRID-M's one
// unanimous sub-class was refuted by GRID-M2 at 114 of 176. This file closes
// the **third** item on the list board #1444 published as two; the second stays
// declined by measurement.

void c2(void *, unsigned int);
void c3(void *, void *, unsigned int);
void c4(void *, void *, void *, unsigned int);

// The two-formal head: one guard, the surviving formal moved into slot 0.
unsigned long p2(void *a0, void *a1) {
    if (a0 == 0) return 5;
    c2(a0, 72);
    return 0;
}

// `mmioGetInfo`'s own shape: two guards, the two pointer slots swapped, the
// literal last.
unsigned long p3lo(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    c3(a1, a0, 72);
    return 0;
}

// The same three-formal list with one guard rather than two.
unsigned long p3hi(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    c3(a1, a0, 5);
    return 0;
}

// Four formals, the deepest rotate the park's unimodal clause admits.
unsigned long p4(void *a0, void *a1, void *a2, void *a3) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    c4(a2, a1, a0, 72);
    return 0;
}

// CONTROL — the identity permutation. In class BEFORE this lane; a widening
// that moves it is changing an existing class rather than reaching a new one.
unsigned long id3(void *a0, void *a1, void *a2) {
    if (a0 == 0) return 5;
    c3(a0, a1, 72);
    return 0;
}
