// w-fltret — the LIVE WRONG EMIT, if the rest of the TU can be got into class.
//
// M2 showed the port's `SplitMs` is 13 words with two `bl`s where c2's is 11
// with none. There the whole TU was `NotImplemented` because `Split` and `Ms`
// are out of class, so nothing reached an obj. This file tries to put the whole
// TU in class so the differential grades it.
struct T {
    int a;

    void s() {}
    int m() { return 7; }
    int both() {
        s();
        return m();
    }
};

int m3_call(T *t) {
    return t->both();
}
