// w-fltret — the LABEL-LEAD counterfactual, in w-json's form: two TUs differing
// in exactly ONE function body, with the SAME framed `z9` second in every one.
// `z9`'s own `$M`/`$M`/`$T` triple is the readout.
//
// CELL lab_mem_fp: THIS LANE: the MEMBER FP value tail -- Timer::SplitMs's own shape
struct S {
    void a();
    int get();
    float f();
};
void gv();
int gz(int);

float lab_first(S *s) {
    s->a();
    return s->f();
}

int z9(int a) {
    return gz(a) + 7;
}
