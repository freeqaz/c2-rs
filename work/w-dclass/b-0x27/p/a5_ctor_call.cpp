struct S { unsigned a; S(unsigned x); void g(unsigned); };
S::S(unsigned x) { a = x; g(x); }
