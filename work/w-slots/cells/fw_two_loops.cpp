// w-slots additivity cell: TWO float-walk loops before the same framed z9.
int gz(int);

void Add_InPlace(unsigned int size, const float *f1, float *f2) {
    if (size == 0)
        return;
    for (unsigned int i = 0; i < size; i++) {
        f2[i] += f1[i];
    }
}

void Mul_InPlace(unsigned int size, const float *f1, float *f2) {
    if (size == 0)
        return;
    for (unsigned int i = 0; i < size; i++) {
        f2[i] *= f1[i];
    }
}

int z9(int a) { return gz(a) + 7; }
