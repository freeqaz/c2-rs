// **w-bdnz — a COUNTED LOOP followed by a FRAMED function**: the TU shape a
// wrong label charge breaks on, and the shape **neither standing instrument can
// produce**.
//
// This is `whash_loop_then_framed.cpp`'s form, one class over, and it is here
// for the reason that one is: `scripts/expr_sweep.sh` generates single-function
// TUs and `scripts/mode_cross.sh` crosses that same corpus with the lane
// registry, so **neither can emit a two-function TU of mixed frame class** and
// both would grade a wrong label charge green.
//
// # What breaks without the gate — and this lane's reason is NOT the inherited
// one
//
// `coff::plan_labels` advances the compiler-label counter once per function in
// `.text` order and charges **1** for a leaf. `IlFunction::label_slots` returns
// `None` for this shape instead, so `IlBundle::functions` refuses this whole TU.
//
// The three existing loop classes return `None` because `w-loop` measured that
// *which* of a loop's `+1..+4` charges applies cannot be read off the emitted
// bytes — `do/while`, `for(;;)`+`break` and a backward `goto` emit identical
// words and charge +1, +3, +1. **That argument does not apply to this class**,
// and lane `w-bdnz` measured why rather than inheriting it
// (`work/w-bdnz/LABEL_LEAD.md`, w-json's counterfactual form over real
// `c2.dll`):
//
//     the first function's body          /O1 $M   lead     /Ox $M   lead
//     leaf-none, 0 locals                  2556     --       2550     --
//     straight line, 2 locals              2558     +2       2552     +2
//     THIS CLASS (`for`, s -= k)           2563     +7       2558     +8
//     the `while` spelling                 2563     +7       2558     +8
//     the `do/while` spelling              2562     +6       2556     +6
//     HashString's pointer walk            2564     +8       2559     +9
//     this class with `*=`                 2563     +7       2558     +8
//     this class with `unsigned`           2563     +7       2558     +8
//
// The `for` and `while` spellings emit **byte-identical text and charge
// identically**, and `do/while` — which charges differently — is not in the
// class at all, because c2 does not convert it. So the confound the other three
// classes cite is absent here. What replaces it is sharper:
//
// **THE CHARGE IS MODE-DEPENDENT — and `label_slots` has no mode parameter.**
// This class accepts BOTH modes, so any `Some(k)` would be right at one and put
// `?z9`'s `$M`/`$M`/`$T` triple off at the other: six wrong bytes in an obj that
// still links, board #263's shape.
//
// # w-counted, 2026-08-15 — THE CONCLUSION SURVIVES AND BOTH NUMBERS DID NOT
//
// The table above is `w-json`'s counterfactual form: the cell TU's framed `$M`
// minus a *separate* `leaf-none` TU's. **A TU's `.gl` label counter depends on
// its own source text**, so that difference is `Δcharge + Δseed` and not a
// charge — board **#3148**, which refuted the identical instrument on
// `float_walk_loop` a day earlier. Re-measured seed-cancelled (each TU's own
// counter subtracted inside it) over a one/two/three-loop series, with two
// zero-controls reading exactly 0 and residual 0 on every row:
//
//     mode    1 loop   2 loops   3 loops     charge
//     /O1       +2       +4        +6           2
//     /O2       +3       +6        +9           3
//     /Ox       +3        —         —           3    (absolute form void:
//     /Od        0        —         —           0     packed layout, so the
//                                                     charge is read at
//                                                     constant segment count)
//
// So the published `+7`/`+8` is `charge + 5`, and `LABEL_COUNTER.md` §4.2.1's
// `for` row — read literally, a lead of `+1` — is **one** low at `/O1` and two
// at `/Ox`, not six.
//
// **And the two-pole probe is what makes the `None` a demonstration.** With each
// candidate installed on `label_lead` (the route `float_walk_loop` took, so
// nothing under `coff/` moves) and graded against real `c2.dll`:
//
//     K=0  MISMATCH /O1   MISMATCH /Ox   MISMATCH /O2    <- Some(1), the claim
//     K=1  MISMATCH       MISMATCH       MISMATCH            below
//     K=2  match          MISMATCH       MISMATCH        <- right at /O1 ONLY
//     K=3  MISMATCH       match          match           <- right at /Ox ONLY
//     K=4  MISMATCH       MISMATCH       MISMATCH
//
// **There is no constant.** `/O2` shares `/Ox`'s optimization word byte for
// byte, so it moves with it and the arm covers eight of the 18 gate lanes.
//
// # What this fixture asserts
//
// `Port=NotImplemented` over the whole TU, at every lane, with `mismatch 0` —
// the port declines rather than emitting a `$M` it cannot justify. Its
// separating control is `wbdnz_ctr.cpp`, twenty of these loops with **no** framed
// function beside them, byte-exact at `/O1`, `/Ox` and `/O2`.
//
// **MUST-FAIL MUTATION, verified**: replacing that `None` with `Some(1)` turns
// this TU from `NotImplemented` into a live `mismatch` against real `c2.dll`
// while the control stays `match`. **Re-run 2026-08-15 by `w-counted` — the
// first thing to check it since it shipped — and it reproduces at `/O1`, `/Ox`
// and `/O2`, with the control `match` under that mutant and four others.**
int gz(int);
int p_sub(int n, int k) { int s = 0; for (int i = 0; i < n; ++i) s -= k; return s; }
int z9(int a) { return gz(a) + 7; }
