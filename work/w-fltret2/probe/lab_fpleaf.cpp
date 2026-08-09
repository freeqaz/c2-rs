// w-fltret — the LABEL-LEAD counterfactual, in w-json's form: two TUs differing
// in exactly ONE function body, with the SAME framed `z9` second in every one.
// `z9`'s own `$M`/`$M`/`$T` triple is the readout.
//
// CELL lab_fpleaf: a KNOWN FP-touching leaf -- prices the TU's _fltused slot on its own
struct S {
    void a();
    int get();
    float f();
};
void gv();
int gz(int);

float lab_first(float x) {
    return x + 1.0f;
}

int z9(int a) {
    return gz(a) + 7;
}
