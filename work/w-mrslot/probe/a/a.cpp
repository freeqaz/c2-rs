struct BE { unsigned f0; unsigned f1; unsigned f2; unsigned f3; };
struct H {
    H* mLink; unsigned mA; BE mBlk; unsigned mB; unsigned mC; unsigned mD;
    H(unsigned p, unsigned q);
    BE* Grab(unsigned n);
};

H::H(unsigned p, unsigned q) {
    BE& r = mBlk;
    mA = p;
    r.f0 = q;
    Grab(p);
}
