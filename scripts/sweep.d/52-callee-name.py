# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- D14: the CALLEE NAME axis ---------------------------------------------------
    # Every shape above sweeps what the port *computes*; this block sweeps what it
    # *names*. A tail call and a generated destructor both emit one `b <callee>` with
    # one REL24, so the callee's name is essentially the whole obj — it is the string in
    # the string table, it sets the symbol count, and it is the only thing a wrong `.gl`
    # binding can change. D14 rewrote how `gl_symbol_index` locates a record (rightmost
    # separator-preceded run, symbol record kind, symbol alphabet, ambiguity dropped),
    # and no fixture had ever varied the SPELLING of a callee name.
    #
    # The axes are the ones the new rules turn on: whether the name contains `@@` at all
    # (an `extern "C"` callee does not, and gating the index on that silently
    # un-resolved five tail calls in one real TU), whether it carries `$` (template
    # instantiations), how long it is, and how many distinct callees compete for tokens
    # in one TU — token values are assigned in declaration order, so a TU with many
    # externals is the only way to reach a token whose bytes are printable, which is the
    # case the rightmost rule exists for.
    for nsyms in (1, 2, 8, 40):
        decls = ''.join('void s%d();\n' % k for k in range(nsyms))
        emit(decls + "void f() { s%d(); }\n" % (nsyms - 1))
        emit(decls + 'extern "C" void cs();\nvoid f() { cs(); }\n')
    for callee in ('void c1();', 'extern "C" void c1();',
                   'namespace N { void c1(); }\nusing N::c1;',
                   'extern "C" void c1(void);'):
        emit(callee + "\nvoid f() { c1(); }\n")
    # Long and punctuation-heavy mangled names, which are what the alphabet test admits
    # and the path/type-name records do not.
    emit("namespace A { namespace B { namespace C { void deep(); } } }\n"
             "void f() { A::B::C::deep(); }\n")
    emit("struct Outer { struct Inner { void m(); }; };\n"
             "void f(Outer::Inner* p) { p->m(); }\n")
    # The destructor delegation with the callee's name varied the same way: a namespace
    # base, a nested base, a class-template base (whose name carries `$` twice), and a
    # base whose name is long enough to sit in the COFF string table rather than inline.
    for decl, base in (
        ("namespace N14 { struct B { ~B(); int x; }; }", "N14::B"),
        ("struct O14 { struct B { ~B(); int x; }; };", "O14::B"),
        ("template <class T> struct B14 { ~B14(); T t; };", "B14<int>"),
        ("template <class T, class U> struct B15 { ~B15(); T t; U u; };",
         "B15<int, char>"),
        ("struct AVeryLongBaseClassNameThatOverflowsTheEightByteInlineSymbolField "
         "{ ~AVeryLongBaseClassNameThatOverflowsTheEightByteInlineSymbolField(); "
         "int x; };",
         "AVeryLongBaseClassNameThatOverflowsTheEightByteInlineSymbolField"),
    ):
        emit("%s\nstruct D : %s { ~D(); int y; };\nD::~D() {}\n" % (decl, base))
        emit("%s\nstruct D { ~D(); int q; %s m; };\nD::~D() {}\n" % (decl, base))
