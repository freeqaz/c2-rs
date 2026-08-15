// w-fencea series cell: 3 loop functions of the ptr-walk-mod class,
// then ONE framed function whose $M/$M/$T triple is the readout.
// #3147: the parameter w-fenceb did not vary is n, and this varies it.
int gz(int);
int HashString0(const char *str, int i) {
    int ret = 0;
    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {
        ret = (*u + ret * 0x7F) % i;
    }
    return ret;
}
int HashString1(const char *str, int i) {
    int ret = 0;
    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {
        ret = (*u + ret * 0x7F) % i;
    }
    return ret;
}
int HashString2(const char *str, int i) {
    int ret = 0;
    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {
        ret = (*u + ret * 0x7F) % i;
    }
    return ret;
}
int z9(int a) { return gz(a) + 7; }
