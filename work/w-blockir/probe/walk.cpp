// w-blockir probe grid — WALKER SELECTION and PARK POSITION.
// Frozen after PREREG §5. Each function is one cell; the cell name says what it
// varies. Compiled at the workload's own flags by probe/cc.sh.
namespace W {
    // W1's decisive cells --------------------------------------------------
    // c1: RHS order swapped against `Mul`. W1 predicts the walker is f1.
    void c1(unsigned int n, const float *f1, const float *f2, float *f3) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f3[i] = f2[i] * f1[i];
    }
    // c2: compound assign with the LHS as the FIRST-declared pointer.
    // W1 predicts the walker is f1 (the LHS load is last); W2 predicts f1 too.
    void c2(unsigned int n, float *f1, const float *f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f1[i] += f2[i];
    }
    // c3: plain assign into the FIRST-declared pointer, one RHS array.
    // The store destination is never loaded, so W1 predicts the walker is f2.
    void c3(unsigned int n, float *f1, const float *f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f1[i] = f2[i];
    }
    // c4: three RHS arrays, one store. W1 predicts the walker is f3.
    void c4(unsigned int n, const float *f1, const float *f2, const float *f3, float *f4) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f4[i] = f1[i] + f2[i] + f3[i];
    }
    // P1's decisive cells --------------------------------------------------
    // c5: shape A with a trailing UNUSED formal, so the walker is no longer in
    // the last GPR formal register. P1 predicts the park moves BELOW the guard.
    void c5(unsigned int n, const float *f1, float *f2, unsigned int unused) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f2[i] += f1[i];
    }
    // c6: shape C with the walker moved into the LAST GPR formal by reordering.
    // P1 predicts the park floats ABOVE the guard.
    void c6(unsigned int n, const float *f1, float *f3, const float *f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f3[i] = f1[i] * f2[i];
    }
    // Op coverage ----------------------------------------------------------
    void c7(unsigned int n, const float *f1, float *f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f2[i] -= f1[i];
    }
    void c8(unsigned int n, const float *f1, float *f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f2[i] /= f1[i];
    }
    // Guard / counter variations -------------------------------------------
    // c9: SIGNED counter and bound.
    void c9(int n, const float *f1, float *f2) {
        if (n == 0) return;
        for (int i = 0; i < n; i++) f2[i] += f1[i];
    }
    // c10: NO guard — the `if (n == 0) return;` removed.
    void c10(unsigned int n, const float *f1, float *f2) {
        for (unsigned int i = 0; i < n; i++) f2[i] += f1[i];
    }
    // c11: double instead of float.
    void c11(unsigned int n, const double *d1, double *d2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) d2[i] += d1[i];
    }
    // c12: shape B with the scalar on the LEFT of the multiply.
    void c12(unsigned int n, float *f1, float f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f1[i] = f2 * f1[i];
    }
    // c13: shape B, plain assign rather than compound.
    void c13(unsigned int n, float *f1, float f2) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) f1[i] = f2;
    }
    // c14: an INT array, same skeleton — is the class float-specific?
    void c14(unsigned int n, const int *a, int *b) {
        if (n == 0) return;
        for (unsigned int i = 0; i < n; i++) b[i] += a[i];
    }
}
