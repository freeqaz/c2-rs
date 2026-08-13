static __declspec(noinline) int wfcnt_big(int a, int b) {
    return a + b;
}

int wfcnt_wrap(int a, int b) {
    return wfcnt_big(a, b);
}
