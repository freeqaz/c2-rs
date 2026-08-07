struct T { int w0; int w1; int w2; int w3; int w4; int w5; };
struct R { T q0; T q1; };
struct N {
    int v0; int v1; int v2; int v3; int v4; int v5;
    int v6; int v7; int v8; int v9; int va; int vb;
    int pad[8];
    R blk;
    R spare;
};
void s(N* y, N* z, int e) {
    T& a = y->blk.q0;
    T& c = a;
    y->v0 = 5;
    y->v1 = 5;
    y->v2 = 5;
    y->v3 = 5;
    y->v4 = 5;
    c.w0 = (int)&a;
    c.w1 = (int)&a;
    c.w2 = (int)&a;
}
