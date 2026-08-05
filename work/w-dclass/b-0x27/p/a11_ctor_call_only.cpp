struct S { unsigned a; S(unsigned x); void g(unsigned); };
S::S(unsigned x) { g(x); }
