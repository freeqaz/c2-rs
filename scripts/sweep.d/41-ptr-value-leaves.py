# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- pointer-VALUED leaves: the class T1 admitted, +88,116 functions -------------
    # The gate here is on the loaded value's OWN width, never the pointee's — loading a
    # `char*` member is `lwz` while loading THROUGH a `char*` is `lbz`, and both spell
    # `char` somewhere in the type. Two predicates, `is_ptr4_kind` and `is_ptr_to_4`,
    # for those two questions. Sweeping the cross is what separates them: the fixtures
    # pin roughly a dozen shapes, and this class is the largest single admission the
    # project has made.
    PSTRUCT = (
        "struct H {\n"
        "  int i; char c;\n"
        "  int* pi; const int* pci; int* const cpi; char* pc; void* pv;\n"
        "  int** ppi; void (*pf)(); int (*pfi)(int);\n"
        "};\n"
    )
    # Pointer-valued member getters: pointee type x cv-spelling x through `this` or not.
    # A member's own declared type is returned where C++ can spell it as a return type;
    # the two function-pointer members are reached through the cast forms below, which
    # is also the interesting case (a function pointer is kind class 4, not 3, and both
    # must lower to the same bare `lwz`).
    for mem, ret in (('pi', 'int*'), ('pci', 'const int*'), ('cpi', 'int*'),
                     ('pc', 'char*'), ('pv', 'void*'), ('ppi', 'int**')):
        emit(PSTRUCT + "%s f(H* h) { return h->%s; }\n" % (ret, mem))
    for mem in ('pi', 'pci', 'cpi', 'pc', 'pv', 'ppi', 'pf', 'pfi'):
        emit(PSTRUCT + "void* f(H* h) { return (void*)h->%s; }\n" % mem)
        emit(PSTRUCT + "void* f(const H* h) { return (void*)h->%s; }\n" % mem)
    # The same read through `this`, const and non-const — `this` is A6-tagged in BOTH,
    # so a gate written for one tag would refuse the commoner spelling.
    for mem in ('pi', 'pc', 'pv', 'ppi'):
        emit(PSTRUCT + "struct C : H { void* g(); };\n"
                 "void* C::g() { return (void*)%s; }\n" % mem)
        emit(PSTRUCT + "struct C : H { void* gc() const; };\n"
                 "void* C::gc() const { return (void*)%s; }\n" % mem)
    # Pointer IDENTITIES, which emit no instruction at all: the value is already in its
    # argument register. Swept across argument position, because "already in r3" is the
    # whole reason they are free and position is what breaks it.
    for ty in ('int', 'char', 'void', 'H'):
        star = '%s*' % ty
        emit(PSTRUCT + "%s f(%s p) { return p; }\n" % (star, star))
        emit(PSTRUCT + "void* f(%s p) { return p; }\n" % star)
        emit(PSTRUCT + "%s f(int a, %s p) { return p; }\n" % (star, star))
        emit(PSTRUCT + "%s f(int a, int b, %s p) { return p; }\n" % (star, star))
    emit(PSTRUCT + "struct C : H { C* self(); const C* cself() const; };\n"
             "C* C::self() { return this; }\n")
    emit(PSTRUCT + "struct C : H { const C* cself() const; };\n"
             "const C* C::cself() const { return this; }\n")
    # The neighbours that MUST refuse rather than emit. Each is one byte or one
    # production away from an admitted shape, and each costs a real instruction the
    # identity/getter lowerings do not emit: an `addi` for an address-of, element-size
    # scaling for pointer arithmetic, an extra load for a double deref, an `mr` when the
    # result is not already in r3. A mismatch here is the alarm; NotImplemented is right.
    for expr in ('&h->i', '&h->pi', 'h->pi + 1', 'h->pi - 1', '*h->ppi',
                 'h->ppi[1]', '(char*)h + 4'):
        emit(PSTRUCT + "void* f(H* h) { return (void*)(%s); }\n" % expr)
    emit(PSTRUCT + "H* f(int a, H* h) { return a ? h : h; }\n")
    emit(PSTRUCT + "int* f(H* h) { return 0; }\n")
