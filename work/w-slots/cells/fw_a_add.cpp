// w-slots grid cell: ONE float array-walk loop + the SAME framed z9, so the
// framed function`s $M triple is the only thing that varies. z9 is
// wblockir_float_walk_then_framed_neg.cpp`s own framed function.
int gz(int);

void Add_InPlace(unsigned int size, const float *f1, float *f2) {
    if (size == 0)
        return;
    for (unsigned int i = 0; i < size; i++) {
        f2[i] += f1[i];
    }
}

int z9(int a) { return gz(a) + 7; }
