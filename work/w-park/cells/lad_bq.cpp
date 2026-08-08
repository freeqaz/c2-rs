// CONSTRUCT LADDER — toward `src/system/synth_xbox/Biquad.cpp`'s two functions.
struct BQ {
    float coefs[7];
    BQ(float *flts);
    void SetCoefficients(float *flts);
    void one(float *flts);
    void run(float *flts);
    void arm(float *flts);
    float div1(float *flts);
};

// B0 — ONE pooled float constant stored through `this`. The smallest thing
// `?SetCoefficients` does.
void BQ::one(float *flts) { coefs[0] = 1.0f; }

// B1 — the else-arm's division, once: two loads, one `fdivs`, one store.
float BQ::div1(float *flts) { return flts[0] / flts[3]; }

// B2 — the whole else-arm: the five-division run whose reload order is
// B'-RULE.
void BQ::run(float *flts) {
    coefs[0] = flts[0] / flts[3];
    coefs[1] = flts[1] / flts[3];
    coefs[2] = flts[2] / flts[3];
    coefs[3] = flts[4] / flts[3];
    coefs[4] = flts[5] / flts[3];
}

// B3 — the whole then-arm: five constant stores from two pools.
void BQ::arm(float *flts) {
    coefs[4] = 0.0f;
    coefs[3] = 0.0f;
    coefs[2] = 0.0f;
    coefs[1] = 0.0f;
    coefs[0] = 1.0f;
}

// B4 — the target: both arms, the join, and the two trailing stores.
void BQ::SetCoefficients(float *flts) {
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
    coefs[6] = 0.0f; coefs[5] = 0.0f;
}

// B5 — the constructor: a framed same-TU call with NO argument setup and a
// park (`mr r10,3`) whose value is never read.
BQ::BQ(float *flts) { SetCoefficients(flts); }
