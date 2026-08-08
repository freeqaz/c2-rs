struct O { int Get(); void Set(int); };
int t1(O *p) { return p->Get(); }
int t2(O *p) { return p->Get() + 1; }
int t3(O *p) { return p->Get() == 0; }
void t4(O *p) { p->Set(5); }
int t5(O *p, int *q) { return p->Get() + *q; }
