// **w-varloop / board #747** — TWO LOOPS OF DIFFERENT LENGTHS IN ONE TU, and
// the port-side discharge w-sched2 left to whichever lane built the lowering.
//
// w-rotate §7.1 named this shape in advance, w-sched2 §6 built it as a grid and
// filed **#792** because *"the port emits no variable-length body, so there is
// no `crates/` mutation for this fixture to bite"*. There is one now, and this
// fixture is the thing it bites.
//
// # Why neither standing instrument can produce it
//
// `scripts/expr_sweep.sh` generates **single-function** TUs from an enumerated
// axis set. `scripts/mode_cross.sh` crosses that same corpus with the lane
// registry. So neither can emit a TU containing two loops of different lengths,
// and **both would grade a one-length schedule GREEN**. That is the fourth
// recorded instance of "the corpus cannot express the failure", and the reason
// a fixture is the only thing that grades this.
//
// # What the two functions do NOT share
//
// `P`'s chain is two steps and `Q`'s is six, and the difference is not a count
// of words. They take **different register regimes and different entry forms**:
//
//     P  M = 2, pv = 0 -> TWO   `lbzu` writes r9, entry `extsb.` + `bclr`
//     Q  M = 6, pv = 0 -> SAME  `lbzu` writes r11 (the character's own
//                               register), entry is an unconditional `b` INTO
//                               the record form, and the function is one word
//                               shorter than the TWO form of the same length
//
// So a model that hard-codes one interleave is wrong about at least one of
// these two, in an obj that still links.
//
// # The must-fail mutation, RUN and not merely described
//
// `work/w-varloop/mutate.py`'s **M6** emits every chain loop in a TU with the
// **first** one's operation list — w-sched2's *"hard-codes one interleave"*,
// implemented literally where the TU's functions are laid out. Measured:
// it turns this fixture's grid cells (`f-1-3`, `f-2-6`, `f-3-8`, `f-4-1`) red
// and leaves `f-same` — two loops of the SAME length — and all ten single-loop
// length cells **green**. That isolation is the claim: the mutation bites
// exactly the shape #747 is about and nothing else.
//
// `/O1` only, like every fixture in this family.
int P(const char* s) {
    int r = 0;
    while (*s) {
        int c = *s;
        r = r + c;
        r = r ^ 3;
        s++;
    }
    return r;
}
int Q(const char* s) {
    int r = 0;
    while (*s) {
        int c = *s;
        r = r + c;
        r = r ^ 3;
        r = r + 5;
        r = r | 9;
        r = r + 11;
        r = r ^ 13;
        s++;
    }
    return r;
}
