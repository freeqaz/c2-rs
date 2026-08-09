// **w-blockir — the LABEL-COUNTER cell.** The float array-walk loop beside a
// FRAMED function, in one TU, which must refuse **whole**.
//
// `IlFunction::label_slots` returns `None` for this class, so
// `IlBundle::functions`' gate — `label_slots(false)? != label_lead() + 1` —
// propagates the `None` and the TU is refused before an obj is written. That is
// `wbdnz_ctr_then_framed_neg.cpp`'s shape one class over, and it is here for the
// same reason: the gate is only ASKED when the TU holds a framed function, so
// without this file the `None` would be a claim no cell exercises.
//
// **The separating control is `wblockir_float_walk.cpp`** — the same loop class
// with no framed function beside it, which is a whole-TU `match` at `/O1`. The
// pair is what makes this a measurement of the gate rather than of the class.
//
// # Why the charge is `None` and not a number
//
// Measured, not inherited: `work/w-blockir/LABEL_LEAD.md` takes the lead in
// w-json's counterfactual form at `/O1` and at `/Ox`, and
// `docs/LABEL_COUNTER.md`'s published surcharges are **not** quoted — three
// separate lanes have now measured that table wrong and the charge is
// mode-dependent. `label_slots` has no mode parameter, which is `w-bdnz` board
// #1983's reason and is sufficient on its own: a value that is right at one
// optimization level and wrong at the other cannot be returned from a function
// that is not told which.
//
// And a correct number would not be enough either. #1983's second counterfactual
// is the one that prices the next rung: `Some(k)` for the *measured* k produces
// a **refusal** rather than a match, because the gate asks whether the charge
// agrees with what `coff::plan_labels` will advance — and `plan_labels` advances
// exactly 1 for a non-framed function. Two layers, and this lane moves neither.

// The framed function, and it is `wbdnz_ctr_then_framed_neg.cpp`'s own — a
// single framed non-leaf call, which is one of the three classes `CLAUDE.md`
// names as the port's byte-exact MVP and which the census **accepts**. That is
// what makes this cell unconfounded: the TU refuses because of the LABEL GATE
// and not because a function in it is out of class. A framed body the reader
// declined would have refused the TU for the wrong reason, and the first
// spelling of this file did exactly that (`call-ref-0xB9`, 1 of 2 in class).
int gz(int);

// **THE LOOP COMES FIRST, AND THE ORDER IS THE WHOLE CELL.** With the framed
// function first this file passes the must-fail mutation — `Some(label_lead() +
// 1)` turns it into a `match`, not a mismatch — because a wrong charge on the
// LAST function moves nothing after it. The first spelling of this file had that
// order and was a cell that could not fail. With the loop first, the same
// mutation is a live `Port=Mismatch`, which is what a `_neg` cell is for.
void Add_InPlace(unsigned int size, const float *f1, float *f2) {
    if (size == 0)
        return;
    for (unsigned int i = 0; i < size; i++) {
        f2[i] += f1[i];
    }
}

// …and the framed function after it: a single framed non-leaf call, one of the
// three classes `CLAUDE.md` names as the port's byte-exact MVP, and one the
// census **accepts** — which is what makes this cell unconfounded. A framed body
// the reader declined would have refused the TU for the wrong reason, and the
// first spelling of this file did exactly that (`call-ref-0xB9`, 1 of 2 in
// class).
int z9(int a) { return gz(a) + 7; }
