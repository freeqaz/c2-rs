struct S { char pad[65536]; int b; };
S s;
int* p = &s.b;
