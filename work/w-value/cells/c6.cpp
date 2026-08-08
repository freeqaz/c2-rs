struct L { L *Next(); int Val(); };
int n4(L *p, int a) { return a + p->Next()->Val(); }
