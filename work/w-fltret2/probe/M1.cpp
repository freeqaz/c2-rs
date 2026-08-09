// w-fltret — the MUST-FAIL cell for the result-CONVERSION fence.
//
// One body, so the whole-TU verdict is this cell's. With the fence in place the
// port refuses (`Port=NotImplemented`). With the `2C` arm of
// `eat_member_value_call` widened to accept the conversion and emit
// `SeqTail::CallValueFp`, the port emits the unconverted value and the obj is
// four bytes short of c2's, which carries `fc200818` = `frsp 1,1`.
struct S {
    void a();
    double d();
};

float m1_narrow(S *s) {
    s->a();
    return (float)s->d();
}
