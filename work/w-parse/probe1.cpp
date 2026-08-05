struct E { unsigned e0,e1,e2,e3; };
struct M {
  unsigned m0,m1,m2,m3,m4,m5,m6,m7,m8,m9; E e; unsigned t0,t1;
  void h(unsigned);
  M* a1(unsigned f0, unsigned f1);
  M* a2(unsigned f0, unsigned f1);
  M* a3(unsigned f0, unsigned f1);
  M(unsigned f0, unsigned f1);
};
M* M::a1(unsigned f0, unsigned f1){ m0=f0; m1=(unsigned)this; m2=1; m3=(unsigned)this; E& l=e; l.e0=(unsigned)&l; l.e1=(unsigned)&l; h(f0); return this; }
M* M::a2(unsigned f0, unsigned f1){ m0=f0; m1=f1; h(f0); return this; }
M* M::a3(unsigned f0, unsigned f1){ m0=f0; m1=1; m2=1; h(f0); return this; }
M::M(unsigned f0, unsigned f1){ m0=f0; m1=(unsigned)this; m2=1; m3=(unsigned)this; E& l=e; l.e0=(unsigned)&l; l.e1=(unsigned)&l; h(f0); }
