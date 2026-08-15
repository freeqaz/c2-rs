// w-slots: a CONSTANT-FREE float leaf + z9. If the lead reads +1, the +1 in the
// float-walk cells is the TU`s _fltused slot and not the loop`s.
int gz(int);

float ff(float a, float b) { return a + b; }

int z9(int a) { return gz(a) + 7; }
