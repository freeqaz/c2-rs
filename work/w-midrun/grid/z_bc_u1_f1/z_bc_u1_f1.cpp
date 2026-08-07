// GRID M cell `z_bc_u1_f1` — w-midrun, the emitter rung under xboxheap.cpp.
// class: zero
// OUT OF DOMAIN — offset 0: c2 emits no addi, the value IS the base register.  Graded to put the answer on record.
// Compiled at the WORKLOAD's own /GR /O1 /Oi /EHsc (#1112).
struct BE { BE* n0; BE* n1; BE* n2; BE* n3; };
struct H {
    BE mZero;          // 0    n0@0  n1@4  n2@8  n3@12
    unsigned mA;       // 16
    BE mBlk;           // 20   n0@20 n1@24 n2@28 n3@32
    unsigned mB;       // 36
    unsigned mC;       // 40
    BE mAlt;           // 44   n0@44 n1@48 n2@52 n3@56
    BE* mP0;           // 60
    BE* mP1;           // 64
    BE* mP2;           // 68
    H(unsigned p, unsigned q);
    void lf(unsigned p, unsigned q);
    BE* Grab(unsigned n);
};

H::H(unsigned p, unsigned q) {
    BE& z = mZero;
    z.n0 = &z;
    mA = q;

    Grab(p);
}
