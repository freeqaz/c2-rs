struct BE { unsigned f0; unsigned f1; unsigned f2; unsigned f3; };
struct H {
    H* mLink; unsigned mA; BE mBlk; unsigned mB; unsigned mC; unsigned mD;
    H(H* w, unsigned q);
    BE* Take(H* n);
};

H::H(H* w, unsigned q) {
    BE& r = w->mBlk;
    mA = 0u;
    r.f0 = q;
    Take(w);
}
