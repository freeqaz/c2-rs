# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- WLR: the ONE-VALUE literal store run ----------------------------------
#
# `82-store-run.py` sweeps runs whose values are formals; this fragment sweeps
# the axis that fragment could not reach, because W38 refused every literal in a
# run of more than one. WLR admits exactly the runs whose literals are all the
# SAME value — one materialization, one register, one live range — and refuses
# the moment a second distinct value appears.
#
# The class boundary here is unusually easy to cross by accident, and every axis
# below is one of the ways:
#
#   * the VALUE crossed with the run length. `emit_load_imm` has two forms
#     (`li` and `lis`+`ori`) and the hoist has to move the WHOLE pair; a
#     fragment that only sweeps small positives never emits an `ori`;
#   * two distinct values that are EQUAL AT ONE WIDTH — `0` stored to a `char`
#     and to a `long long` is one value, `-1` and `255` are not — which is the
#     `docs/GAPS.md` §6 shape where a comparison is made on the wrong
#     representation;
#   * the WIDTH of each statement crossed with its neighbour's, with one value:
#     `stb`/`sth`/`stw`/`std` all out of the same register, which is the case
#     the single-store captures cannot reach;
#   * the BASE slot and the number of distinct bases, because with one value
#     there is nothing to schedule around and c2 keeps source order — a
#     property that must not be inherited by the multi-value case;
#   * the TAIL crossed with the run length, since the hoisted `li` now sits
#     ahead of a prologue-free body whose epilogue is a bare `blr`;
#   * a literal run beside a FORMAL one in the same body, which is the mixed
#     case and must refuse — and beside a formal run in the same TU, which
#     must not.


def cases(emit):
    S = ('struct S { int a; int b; int c; int d; int e; int f; int g; '
         'char h; short i; long long j; int* k; float x; double y; };')
    MEM = ['a', 'b', 'c', 'd', 'e', 'f', 'g']
    VALS = ['0', '1', '-1', '7', '32767', '-32768', '100000', '65535', '65536',
            '2147483647']

    # 1. One value x run length 1..7 x the value's encoding form.
    for v in VALS:
        for n in range(1, 8):
            body = ' '.join('s->%s = %s;' % (MEM[j], v) for j in range(n))
            emit('%s\nvoid f(S* s) { %s }\n' % (S, body))

    # 2. The same value through every base slot 0..8: past the eighth formal the
    #    base is stack-homed and the whole body must refuse, hoist or no hoist.
    for slot in range(0, 9):
        pre = ''.join('int p%d, ' % j for j in range(slot))
        body = ' '.join('s->%s = 9;' % MEM[j] for j in range(3))
        emit('%s\nvoid f(%sS* s) { %s }\n' % (S, pre, body))

    # 3. Mixed WIDTHS out of one register — every ordered pair and a few
    #    triples. The value is materialized once and each statement picks its
    #    own store opcode; a `float`/`double` member is NOT this class at all
    #    (an FP literal pools a constant) and must refuse.
    W = ['a', 'h', 'i', 'j', 'k', 'x', 'y']
    for p in W:
        for q in W:
            if p == q:
                continue
            emit('%s\nvoid f(S* s) { s->%s = 0; s->%s = 0; }\n' % (S, p, q))
    for t in [('h', 'i', 'a'), ('a', 'h', 'j'), ('j', 'a', 'h'), ('h', 'j', 'i')]:
        emit('%s\nvoid f(S* s) { s->%s = 0; s->%s = 0; s->%s = 0; }\n'
             % (S, t[0], t[1], t[2]))

    # 4. TWO distinct values, every ordered pair from a small set, at lengths
    #    2..4 — the refusal side. c2 permutes r11/r10/r9 by a rule these
    #    captures do not determine and reorders the stores, so every one of
    #    these must census OUT of class.
    for u in ['0', '1', '-1', '100000']:
        for v in ['0', '1', '-1', '100000']:
            if u == v:
                continue
            emit('%s\nvoid f(S* s) { s->a = %s; s->b = %s; }\n' % (S, u, v))
            emit('%s\nvoid f(S* s) { s->a = %s; s->b = %s; s->c = %s; }\n'
                 % (S, u, v, u))
            emit('%s\nvoid f(S* s) { s->a = %s; s->b = %s; s->c = %s; s->d = %s; }\n'
                 % (S, u, v, u, v))

    # 5. Values that are equal at one width and not at another — the
    #    representation trap. `-1` and `255` into a `char` are the same byte and
    #    two different IL literals; the class is keyed on the LITERAL, so both
    #    orders must behave identically.
    for u, v in [('-1', '255'), ('0', '256'), ('1', '65537'), ('-1', '4294967295')]:
        emit('%s\nvoid f(S* s) { s->h = %s; s->h2 = %s; }\n'
             % ('struct S { char h; char h2; int a; };', u, v))
        emit('%s\nvoid f(S* s) { s->h = %s; s->h2 = %s; }\n'
             % ('struct S { char h; char h2; int a; };', v, u))

    # 6. MIXED with a formal, both orders, at lengths 2..4 — scheduled by c2 and
    #    refused here, and the neighbour most likely to be admitted by accident
    #    because it shares every other property with case 1.
    for n in range(2, 5):
        for litpos in range(n):
            args = ', '.join('int v%d' % j for j in range(n - 1))
            fi = 0
            parts = []
            for j in range(n):
                if j == litpos:
                    parts.append('s->%s = 5;' % MEM[j])
                else:
                    parts.append('s->%s = v%d;' % (MEM[j], fi))
                    fi += 1
            emit('%s\nvoid f(S* s, %s) { %s }\n' % (S, args, ' '.join(parts)))

    # 7. The TAIL x run length. The hoisted `li` sits ahead of a body whose
    #    epilogue is free; `return <other formal>` re-bases the stores and must
    #    refuse at every length.
    T = ('struct T { int a; int b; int c; T& r2(); T* p2(); void v3(); '
         'T(); T(int); };')
    emit('%s\nT& T::r2() { a = 0; b = 0; return *this; }\n' % T)
    emit('%s\nT* T::p2() { a = 0; b = 0; return this; }\n' % T)
    emit('%s\nvoid T::v3() { a = 0; b = 0; c = 0; }\n' % T)
    emit('%s\nT::T() { a = 0; b = 0; c = 0; }\n' % T)
    emit('%s\nT::T(int) : a(1), b(1), c(1) { }\n' % T)
    emit('%s\nS* f(S* s) { s->a = 4; s->b = 4; return s; }\n' % S)
    emit('%s\nint f(int u, S* s) { s->a = 4; s->b = 4; return u; }\n' % S)

    # 8. TWO bases and THREE, with one value: c2 keeps every store and does not
    #    reorder, which is the property that must NOT leak to the multi-value
    #    case. Every interleaving of two bases over three statements.
    for pat in ['sst', 'sts', 'tss', 'tts', 'tst', 'stt']:
        body = ' '.join('%s->%s = 6;' % ('s' if c == 's' else 't', MEM[j])
                        for j, c in enumerate(pat))
        emit('%s\nvoid f(S* s, S* t) { %s }\n' % (S, body))

    # 9. Statement order against offset order, with one value — the axis that
    #    separates "source order" from "offset order", which agree on every
    #    ascending case and on every case where the value is the same. Included
    #    precisely because they agree: a hoist that also sorted would be
    #    invisible to case 1.
    import itertools as _it
    for perm in _it.permutations(range(4)):
        body = ' '.join('s->%s = 3;' % MEM[i] for i in perm)
        emit('%s\nvoid f(S* s) { %s }\n' % (S, body))

    # 10. A literal run and a formal run in the SAME TU (must both be in class,
    #     independently) and in the same BODY (must refuse). The first is the
    #     regression the hoist could cause: it is detected off the whole `ops`
    #     stream, so a second function's stream must not be able to reach it.
    emit('%s\nvoid f(S* s) { s->a = 2; s->b = 2; }\n'
         'void g(S* s, int u, int v) { s->a = u; s->b = v; }\n' % S)
    emit('%s\nvoid g(S* s, int u, int v) { s->a = u; s->b = v; }\n'
         'void f(S* s) { s->a = 2; s->b = 2; }\n' % S)
    emit('%s\nvoid f(S* s, int u) { s->a = 2; s->b = 2; s->c = u; }\n' % S)

    # 11. Sub-object designators under a one-value run: a run of byte-offset
    #     adds (W35's shared walk) and an inherited member (intrinsic 2117).
    NEST = ('struct In { int p; int q; }; struct Mid { int pad; In in; }; '
            'struct Out { int pad0; Mid mid; };')
    emit('%s\nvoid f(Out* o) { o->mid.in.p = 8; o->mid.in.q = 8; }\n' % NEST)
    emit('%s\nvoid f(Out* o, Out* p) { o->mid.in.p = 8; p->mid.in.q = 8; }\n' % NEST)
    BASE = ('struct B { int b0; int b1; }; '
            'struct D : B { int d0; void z2(); void z3(); };')
    emit('%s\nvoid D::z2() { b0 = 0; d0 = 0; }\n' % BASE)
    emit('%s\nvoid D::z3() { b0 = 0; b1 = 0; d0 = 0; }\n' % BASE)

    # 12. The dead-store pair with one value — c2 eliminates the second, so this
    #     must refuse even though every other property says "one value".
    emit('%s\nvoid f(S* s) { s->a = 1; s->a = 1; }\n' % S)
    emit('%s\nvoid f(S* s) { s->a = 1; s->b = 1; s->a = 1; }\n' % S)
    emit('%s\nvoid f(S* s) { s->j = 0; s->a = 0; }\n'
         % 'struct S { long long j; int a; };')

    # 13. Source lines and brace scopes between the statements — instance #1's
    #     axis, and the statement-boundary walk now runs under a hoisted `li`.
    emit('%s\nvoid f(S* s) {\n s->a = 1;\n\n\n s->b = 1;\n}\n' % S)
    emit('%s\nvoid f(S* s) { s->a = 1; { s->b = 1; } s->c = 1; }\n' % S)
    emit('%s\nvoid f(S* s) { { s->a = 1; s->b = 1; } }\n' % S)

    # 14. cv-qualification on the base and on the member, which changes no
    #     operator and no shape and is where the sweep has found live mis-emits
    #     before.
    emit('%s\nvoid f(S* const s) { s->a = 1; s->b = 1; }\n' % S)
    emit('%s\nvoid f(S* s) { s->a = 1; s->b = 1; }\n'
         % 'struct S { volatile int a; int b; };')
    emit('%s\nvoid f(S* s) { s->a = 1; s->b = 1; }\n'
         % 'struct S { int a; volatile int b; };')
    emit('#pragma pack(1)\n%s\nvoid f(S* s) { s->a = 0; s->j = 0; }\n'
         % 'struct S { char c; int a; long long j; };')
