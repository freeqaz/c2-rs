// w-fltret — the LABEL-LEAD counterfactual, in w-json's form: two TUs differing
// in exactly ONE function body, with the SAME framed `z9` second in every one.
// `z9`'s own `$M`/`$M`/`$T` triple is the readout.
//
// CELL lab_mem_int: THIS LANE: the MEMBER integer value tail
struct S {
    void a();
    int get();
    float f();
};
void gv();
int gz(int);

int lab_first(S *s) {
    s->a();
    return s->get();
}

int z9(int a) {
    return gz(a) + 7;
}
