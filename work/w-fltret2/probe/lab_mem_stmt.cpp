// w-fltret — the LABEL-LEAD counterfactual, in w-json's form: two TUs differing
// in exactly ONE function body, with the SAME framed `z9` second in every one.
// `z9`'s own `$M`/`$M`/`$T` triple is the readout.
//
// CELL lab_mem_stmt: w-mcall's statement sequence -- the same frame class WITHOUT the value tail, so the value tail's own charge is isolated
struct S {
    void a();
    int get();
    float f();
};
void gv();
int gz(int);

void lab_first(S *s) {
    s->a();
    s->a();
}

int z9(int a) {
    return gz(a) + 7;
}
