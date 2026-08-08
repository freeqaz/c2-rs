int p0(int i) {
    static int a[8] = { 3, 5, 7, 11, 13, 17, 19, 0 };
    for (int j = 0; a[j] != 0; j++) {
        if (a[j] >= i)
            return a[j];
    }
    return i;
}
