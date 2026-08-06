struct S { void (*f)(); int a; };
S s = { (void (*)())0x11223344, 5 };
