# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- D5: DATA-SYMBOL ADDRESSES ---------------------------------------------------
    # `docs/IL_CALL_IN_EXPR.md` §17. The port emits NOTHING for this class — a data
    # symbol's address needs a REFHI/REFLO pair, and two of them in one call need a
    # `.rdata`-pool-relative selection that is not modeled — so every case below must be
    # `NotImplemented`. That is exactly what makes the sweep worth having here: a
    # relocation is the place where a wrong byte is invisible in a probe and fatal in a
    # real TU, and the failure mode this guards is the port emitting a 5-section obj for
    # a TU whose reference obj carries a `.rdata` string pool, a `.data`/`.bss` for a
    # defined global, or four extra relocation records.
    #
    # The cross product is over the axes the capture showed matter: how many symbols one
    # call materializes (1 / 2 / 3 — the workload is entirely 2), where they sit among
    # the other arguments, whether the symbol is a string literal (its own `$SG…`
    # `.rdata` entry) or a named object (an undefined external, no section at all), the
    # object's LINKAGE (extern / defined here / static / const), and whether the address
    # is at offset 0 or through a subscript.
    DECLS = (
        "struct T;\n"
        "extern void s1(const char*);\n"
        "extern void s2(const char*, const char*);\n"
        "extern void s3(const char*, const char*, const char*);\n"
        "extern void ps(T*, const char*);\n"
        "extern void sp(const char*, T*);\n"
        "extern void psls(T*, const char*, int, const char*);\n"
        "extern void isli(int, const char*, int);\n"
        "extern int  rs(const char*);\n"
        "extern void ui(int*);\n"
        "extern void uii(int*, int*);\n"
        "extern int  gi(int);\n"
    )
    for body in (
        # one symbol, at every position, string and named object, void and int result
        'void f(){ s1("a"); }',
        'void f(T* p){ ps(p, "a"); }',
        'void f(T* p){ sp("a", p); }',
        'int  f(){ return rs("a"); }',
        'void f(){ ui(gA); }',
        'void f(){ ui(&gA[2]); }',
        'void f(){ ui(&gS.b); }',
        # two symbols in one call — the shape the whole workload row is
        'void f(){ s2("a", "b"); }',
        'void f(T* p){ psls(p, "expr", 42, "file"); }',
        'void f(){ uii(gA, gB); }',
        'void f(){ uii(gA, &gA[4]); }',
        'void f(){ s2("a", gC); }',
        # three
        'void f(){ s3("a", "b", "c"); }',
        # the same symbol twice: one `.rdata` entry or two?
        'void f(){ s2("a", "a"); }',
        'void f(){ uii(gA, gA); }',
        # a symbol also read elsewhere in the same body / TU
        'void f(){ ui(gA); }\nint  h(){ return gA[0]; }',
        'int  f(){ return gi(gA[1]); }',
        # literals interleaved, the assert-macro spellings
        'int  f(int a){ return gi(a + gA[0]); }',
        'void f(){ isli(1, "a", 2); }',
        # a symbol whose address is stored rather than passed
        'extern const char* gp;\nvoid f(){ gp = "a"; }',
    ):
        for prefix in (
            "extern int gA[8];\nextern int gB[4];\nstruct S{int a;int b;};\nextern S gS;\n"
            "extern const char gC[4];\n",
            # …and the same bodies with the objects DEFINED here, which puts a `.data`
            # or `.bss` section in the middle of the reference obj's section table.
            "int gA[8];\nint gB[4];\nstruct S{int a;int b;};\nS gS;\nconst char gC[4]={'a','b','c',0};\n",
            # …and static, which mangles to an undecorated name and is `.bss` too.
            "static int gA[8];\nstatic int gB[4];\nstruct S{int a;int b;};\nstatic S gS;\n"
            "static const char gC[4]={'a','b','c',0};\n"
            "int keep(){ return gA[0]+gB[0]+gS.a+gC[0]; }\n",
        ):
            emit(DECLS + prefix + body + "\n")

    # The shapes one byte away that must keep refusing: the same calls with no symbol at
    # all (already in class as tail calls — these must stay `Match`, and a regression
    # here would show up as a Mismatch), and the neighbours that differ from a symbol
    # address by one construct.
    for src in (
        # in class today: no data symbol anywhere
        "extern void s1(const char*);\nvoid f(){ s1(0); }\n",
        "extern void v1(int);\nvoid f(int a){ v1(a); }\n",
        "extern int gi(int);\nint f(int a){ return gi(a); }\n",
        # a string literal used for its LENGTH, not its address
        "extern void v1(int);\nvoid f(){ v1(sizeof(\"abcd\")); }\n",
        # a function's address, not a data object's
        "extern void h();\nextern void fp(void(*)());\nvoid f(){ fp(h); }\n",
        # a local array's address: a frame, not a relocation
        "extern void ui(int*);\nvoid f(){ int a[4]; ui(a); }\n",
        # a wide string literal: a different `.rdata` entry width
        "extern void uw(const wchar_t*);\nvoid f(){ uw(L\"ab\"); }\n",
        # an empty string literal, and one exactly at the 4-byte pool alignment
        "extern void uc(const char*);\nvoid f(){ uc(\"\"); }\n",
        "extern void uc(const char*);\nvoid f(){ uc(\"abc\"); }\n",
        "extern void uc(const char*);\nvoid f(){ uc(\"abcd\"); }\n",
    ):
        emit(src)
