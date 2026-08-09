// p3 — two leaves then a framed one: a SECOND label-charge cell at a different
// leaf count, so "0 per leaf" is separated from "0 for the first leaf only".
int gg(int);
int cal5(int, int, int, int, int);
int leafA(int a, int b, int c, int d, int e) { return cal5(a, b, c, d, e); }
int leafB(int a, int b, int c, int d, int e) { return cal5(e, b, c, d, a); }
int framed(int a) { return gg(a) + 1; }
