struct S { void (*f)(); int a; };
S s = { (void (*)())4, 5 };
