// C++ try/catch at /EHsc — the OTHER EH shape, to separate "EH" from "SEH".
int ctl_a(int x) { return x + 1; }

extern "C" int cxx_probe(int *p);

int cxx_a(int x)
{
    int r = 0;
    try {
        r = cxx_probe(&x);
        r += x * 3;
    } catch (int e) {
        r = e - 1;
    }
    return r;
}
