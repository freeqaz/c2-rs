// w-slots grid cell: ONE float array-walk loop + the SAME framed z9, so the
// framed function`s $M triple is the only thing that varies. z9 is
// wblockir_float_walk_then_framed_neg.cpp`s own framed function.
int gz(int);

void Mul(unsigned int size, const float *f1, const float *f2, float *f3) {
    if (size == 0)
        return;
    for (unsigned int i = 0; i < size; i++) {
        f3[i] = f1[i] * f2[i];
    }
}

int z9(int a) { return gz(a) + 7; }
