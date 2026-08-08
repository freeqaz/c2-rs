static int n0a[8] = { 3, 5, 7, 11, 13, 17, 19, 0 };
int n0(int i) {
    for (int j = 0; n0a[j] != 0; j++) {
        if (n0a[j] >= i)
            return n0a[j];
    }
    return i;
}
