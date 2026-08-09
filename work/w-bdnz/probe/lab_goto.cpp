// The CALIBRATION row: `?HashString`'s own pointer-walk shape, which
// `docs/LABEL_COUNTER.md` §4.2.1 records at +3. If this instrument does not
// reproduce that number, the instrument is wrong and nothing else here is
// evidence.
int gz(int);
int lead(const char *str, int i) {
    int ret = 0;
    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {
        ret = (*u + ret * 0x7F) % i;
    }
    return ret;
}
int z9(int a) { return gz(a) + 7; }
