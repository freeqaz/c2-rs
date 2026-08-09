// w-biquad — LABEL_COUNTER §7.6's in-the-middle stride, for the float-store
// diamond.
//
// Subject in the MIDDLE (§7.6 step 2): `a0 · P · a1 · a2`. `base` is measured in
// the SAME obj as `first(a2) - first(a1)` and must read **5** under `/Gy` or the
// row is void (step 3). Never the counterfactual form (step 1).
//
// The prediction, from §1.1's surcharge table and NOT from a fit:
//     P's stride = 1 (leaf base) + 1 (`_fltused`, first FP function)
//                + 2 + 2 (two newly pooled constants)     = 6
// so `first(a1) - first(a0)` should be `5 + 6 = 11`.
extern int ga(int);
int a0(int a) { return ga(a) + 1; }

namespace LS {
    class P {
    public:
        void SetCoefficients(float *);
        float coefs[7];
    };
    void P::SetCoefficients(float *flts) {
        if (flts == 0) {
            coefs[4] = 0.0f; coefs[3] = 0.0f; coefs[2] = 0.0f;
            coefs[1] = 0.0f; coefs[0] = 1.0f;
        } else {
            coefs[0] = flts[0] / flts[3];
            coefs[1] = flts[1] / flts[3];
            coefs[2] = flts[2] / flts[3];
            coefs[3] = flts[4] / flts[3];
            coefs[4] = flts[5] / flts[3];
        }
        coefs[6] = 0.0f;
        coefs[5] = 0.0f;
    }
}

int a1(int a) { return ga(a) + 2; }
int a2(int a) { return ga(a) + 3; }
