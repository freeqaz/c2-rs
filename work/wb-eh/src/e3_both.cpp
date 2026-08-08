// E3 — a catch AND a destructible local: two funclets in one function.
// E4 rides along: `leaf` is a frameless non-EH function in the same TU.
struct S { S(); ~S(); int m; };
int g(int);
int f(int a){
    S s;
    try { return g(a) + s.m; }
    catch (int e) { return e + 1; }
}
int leaf(int a){ return a + 1; }
