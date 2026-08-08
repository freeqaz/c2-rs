// E6 -- the third-witness cell for the 5C kind reading (WB_EH_FINDINGS §5.5):
// two destructible locals, a NESTED try, and a catch(...) alongside catch(int).
struct S { S(); ~S(); int m; };
struct T { T(); ~T(); int n; };
int g(int);
int f(int a){
    S s;
    try {
        T t;
        try { return g(a) + s.m + t.n; }
        catch (int e) { return e + 1; }
    }
    catch (...) { return -1; }
}
