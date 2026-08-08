int n1(int i) {
    static int a[8];
    for (int j = 0; a[j] != 0; j++) {
        if (a[j] >= i)
            return a[j];
    }
    return i;
}
