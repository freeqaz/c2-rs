// w-slots: a float leaf AND one float-walk loop. Predicts +3 if the TU slot is
// charged once and the loop charges 2.
int gz(int);

float ff(float a, float b) { return a + b; }

void Add_InPlace(unsigned int size, const float *f1, float *f2) {
    if (size == 0)
        return;
    for (unsigned int i = 0; i < size; i++) {
        f2[i] += f1[i];
    }
}

int z9(int a) { return gz(a) + 7; }
