// w-blockir probe grid round 2 — the CLASS BOUNDARY. One cell per clause the
// reader will carry, so that every arm the emitter ships has a graded witness
// and every clause it refuses has a measured reason. Frozen after round 1.
namespace B {
    // Shape C op coverage --------------------------------------------------
    void d1(unsigned int n, const float *f1, const float *f2, float *f3) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f3[i] = f1[i] + f2[i];
    }
    void d2(unsigned int n, const float *f1, const float *f2, float *f3) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f3[i] = f1[i] - f2[i];
    }
    // Shape B op coverage --------------------------------------------------
    void d3(unsigned int n, float *f1, float f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f1[i] += f2;
    }
    void d4(unsigned int n, float *f1, float f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f1[i] -= f2;
    }
    // Shape A, the walker's third separating point: the LHS is the FIRST
    // declared and the RHS the LAST, with a fourth formal after both.
    void d5(unsigned int n, float *f1, const float *f2, unsigned int pad) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f1[i] *= f2[i];
    }
    // Refusal cells --------------------------------------------------------
    // e1: the induction variable is used for something other than subscripting.
    unsigned int e1(unsigned int n, const float *f1, float *f2) {
        if (n == 0) return 0;
        unsigned int s = 0;
        for (unsigned int i = 0; i < n; i++) { f2[i] += f1[i]; s += i; }
        return s;
    }
    // e2: the loop is not the function tail.
    void e2(unsigned int n, const float *f1, float *f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f2[i] += f1[i];
        f2[0] = 1.0f;
    }
    // e3: the step is 2, not 1.
    void e3(unsigned int n, const float *f1, float *f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i += 2) f2[i] += f1[i];
    }
    // e4: the bound is a different formal from the guard's subject.
    void e4(unsigned int n, unsigned int m, const float *f1, float *f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < m; i++) f2[i] += f1[i];
    }
    // e5: two statements in the loop body.
    void e5(unsigned int n, const float *f1, float *f2, float *f3) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) { f2[i] += f1[i]; f3[i] = f2[i]; }
    }
}
