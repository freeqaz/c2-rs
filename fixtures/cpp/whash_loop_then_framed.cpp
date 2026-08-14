// **w-hash / board #747** — a LOOP LEAF followed by a FRAMED function: the
// shape a backward-branch label charge breaks on, and the shape **neither
// standing instrument can produce**.
//
// `w-loop` §6 registered this before anything shipped, and it is the fourth
// instance of "the corpus cannot express the failure" — the first one found
// *ahead* of the defect rather than after it. `scripts/expr_sweep.sh` generates
// **single-function** TUs at `/Ox`; `scripts/mode_cross.sh` crosses that same
// corpus with the lane registry. So neither can emit a two-function TU of mixed
// frame class, and **both would grade a wrong label charge green**.
//
// # What breaks without the gate
//
// `coff::plan_labels` advances the compiler-label counter once per function in
// `.text` order and charges **1** for a leaf. `w-loop` measured, seed-free over
// 17 probes with a 5/5 anchor control on every row, that a **loop** leaf charges
// `+1..+4` — never 1 — so `?z9`'s `$M`/`$M`/`$T` triple would come out low: six
// wrong bytes in an obj that still links, which is board #263's exact shape.
//
// **THIS HEADER SAID `?HashString`'s own shape charges `+3` AND THE TRIPLE WOULD
// COME OUT `three low`. Both were ONE HIGH — the lead is `+2`** (board **#3091**,
// lane `w-backedge`, corrected here by lane `w-fenceb`). The `+3` came from
// `LABEL_COUNTER.md` §4.2.1's `leaf-ptrwalk` row, which is `Sort.cpp`'s pointer
// walk and not this one. **This obj is the judge**: its `.gl` label counter is
// 2546, so a charge-0 `plan_labels` seeds `?z9` at 2562, and the reference obj
// mints `$M2564`/`$M2565`/`$T2566`. The error ran in the direction that made the
// fence look dearer to lift than it was, which is why it stood.
//
// # What this fixture asserts — CHANGED BY LANE `w-fenceb`, and it now CONVERTS
//
// It used to assert a whole-TU refusal: `IlFunction::label_slots` returned
// `None` for the loop shape and the three-valued gate in `IlBundle::functions`
// rejected the TU at every lane. **That fence (board #746's fence B) is lifted.**
// `IlFunction::label_lead` charges this shape **2**, `label_slots` falls through
// to `label_lead() + 1 = 3`, and `coff::plan_labels` advances exactly that for a
// non-framed function — so this TU is now **`match` at `/O1`** and the assertion
// is a byte-exact obj, still with `mismatch 0`.
//
// **Why a charge is honest here when no general loop charge is.** The old
// refusal's ground was that the charge cannot be recovered from the emitted
// bytes — `do/while`, `for(;;)`+`break` and a backward `goto` emit the
// **identical 24 bytes** and charge +1, +3, +1 — and that ground still stands
// for loops in general. `w-fenceb`'s grid3 hold-out made it worse, not better:
// five pairs of cells with identical backward-branch feature vectors charge
// differently, every one a `while` against a `for`. But this class is not a
// general loop. `IlFunction::ptr_walk_loop` is a closed recognizer over one
// `for` shape with no `break`, no `continue` and no `goto`, so every residual
// that hold-out found is excluded from it by construction — and its charge is
// **read out of this obj**, not fitted to a rule.
//
// The residual risk is the MODE: the same source's `?z9` is `$M2564` at `/O1`,
// `$M2559` at `/Ox`, `$M2565` at `/O2` and `$M2554` at `/Od`, and
// `label_slots` has no mode parameter. What holds it is
// `codegen::ptr_walk_loop` refusing every mode but `/O1`, which is a fact in
// another crate and is therefore PINNED BY A TEST —
// `differential_whash_loop_then_framed_refuses_outside_its_mode`.
//
// Its separating control is `whash_ptr_walk_loop.cpp`, the identical loop with
// **no** framed function beside it, which is byte-exact at `/O1`. Together they
// say the charge is observable only in the company the shape keeps — which is
// exactly what `w-loop` measured (34 of 34 leaf-only TUs mint zero labels, 28 of
// them carrying a backward branch; control 17 of 17). That control is what makes
// the three must-fail mutations readable: leads of 0, 1 and 3 each turn THIS
// file into a live `mismatch` while the control stays `match` under all three
// (`work/w-fenceb/mutants_o1.txt`).
int gz(int);
int HashString(const char *str, int i) {
    int ret = 0;
    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {
        ret = (*u + ret * 0x7F) % i;
    }
    return ret;
}
int z9(int a) { return gz(a) + 7; }
