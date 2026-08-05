struct S { S *p; unsigned a; S(unsigned x); };
S::S(unsigned x) { a = x; p = this; }
