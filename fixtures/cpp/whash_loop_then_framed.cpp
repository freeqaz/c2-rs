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
// `+1..+4` — never 1 — and that `?HashString`'s own pointer-walk shape charges
// **+3**. So `?z9`'s `$M`/`$M`/`$T` triple would come out **three low**: six
// wrong bytes in an obj that still links, which is board #263's exact shape.
//
// And the charge cannot be recovered from the emitted bytes. `do/while`,
// `for(;;)`+`break` and a backward `goto` emit the **identical 24 bytes** and
// charge **+1, +3, +1**, so any rule fitted to the body would be fitted to a
// body that does not distinguish them.
//
// # What this fixture asserts
//
// `IlFunction::label_slots` returns `None` for the loop shape, so the
// three-valued gate in `IlBundle::functions` **refuses this whole TU** —
// `Port=NotImplemented`, at every lane. The assertion is `mismatch 0`: the port
// declines rather than emitting a `$M` it cannot justify.
//
// Its separating control is `whash_ptr_walk_loop.cpp`, the identical loop with
// **no** framed function beside it, which is byte-exact at `/O1`. Together they
// say the refusal is conditional on the company the shape keeps and not on the
// shape — which is exactly what `w-loop` measured (34 of 34 leaf-only TUs mint
// zero labels, 28 of them carrying a backward branch; control 17 of 17).
int gz(int);
int HashString(const char *str, int i) {
    int ret = 0;
    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {
        ret = (*u + ret * 0x7F) % i;
    }
    return ret;
}
int z9(int a) { return gz(a) + 7; }
