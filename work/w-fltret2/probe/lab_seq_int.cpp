// w-fltret — the LABEL-LEAD counterfactual, in w-json's form: two TUs differing
// in exactly ONE function body, with the SAME framed `z9` second in every one.
// `z9`'s own `$M`/`$M`/`$T` triple is the readout.
//
// CELL lab_seq_int: the FREE integer value tail -- in class since #35 step 2, and the control the FP one is measured against
struct S {
    void a();
    int get();
    float f();
};
void gv();
int gz(int);
int gi();

int lab_first() {
    gv();
    return gi();
}

int z9(int a) {
    return gz(a) + 7;
}
