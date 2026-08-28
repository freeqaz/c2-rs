// __try / __except — the construct predicted to set sym+0x20 bit 12.
// ctl_a is repeated verbatim so a per-function split is visible: if the bit is
// per-function, ctl_a still reaches sched0 while seh_a does not.
int ctl_a(int x) { return x + 1; }

extern "C" int seh_probe(int *p);

int seh_a(int x)
{
    int r = 0;
    __try {
        r = seh_probe(&x);
        r += x * 3;
    }
    __except (1 /* EXCEPTION_EXECUTE_HANDLER */) {
        r = -1;
    }
    return r;
}

int seh_b(int x)
{
    int r = 0;
    __try {
        r = seh_probe(&x);
    }
    __finally {
        r += 7;
    }
    return r;
}
