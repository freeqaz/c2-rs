// no memory reference at all — the update form cannot apply
int c0(int n) {
    int s = 0;
    for (int i = 0; i < n; ++i) s += 3;
    return s;
}
int c1(int n, int k) {
    int s = 0;
    for (int i = 0; i < n; ++i) s += k;
    return s;
}
int c2f(int n, int k) {
    int s = 1;
    for (int i = 0; i < n; ++i) s *= k;
    return s;
}
int c3(int n, int k) {
    int s = 0;
    for (int i = 0; i < n; ++i) s -= k;
    return s;
}
// counter used in the body — R6 must refuse
int c4(int n) {
    int s = 0;
    for (int i = 0; i < n; ++i) s += i;
    return s;
}
