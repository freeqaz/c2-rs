// GRID R cell `lc_tt_fl` — w-mrslot, board #1212 (the mr-slot `u`).
// class=leaf-control syms=TT vals=FL tail=(none) kind=leaf
// nprod=1 count=1 nsym=1 can-separate=False
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { unsigned f0; unsigned f1; unsigned f2; unsigned f3; };
struct H {
    H* mLink;          // 0
    unsigned mA;       // 4
    BE mBlk;           // 8   f0@8 f1@12 f2@16 f3@20
    unsigned mB;       // 24
    unsigned mC;       // 28
    unsigned mD;       // 32
    H(unsigned p, unsigned q);
    H(H* w, unsigned q);
    void lf(unsigned p, unsigned q);
    BE* Grab(unsigned n);
    BE* Take(H* n);
    BE* Reset();
};

void H::lf(unsigned p, unsigned q) {
    mA = q;
    mB = 0u;

}
