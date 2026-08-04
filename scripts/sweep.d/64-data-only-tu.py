# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **THE FUNCTIONLESS TU** (board #174, lane w-sect). A translation unit that
# defines no functions and *does* define namespace-scope storage. `c2` gives it a
# `.bss`, a `.data`, a `.rdata`, a `.tls$` or a COMDAT; `PortC2::build` gave it
# the bare four-section shell, because the arm's precondition was
# `is_empty_module`, which is a property of `.ex` alone and cannot see `.gl`.
#
# Eight of eleven hand-written probes were live `Port=Mismatch @ offset 2` — a
# wrong SECTION COUNT — and **no standing instrument could raise the alarm**:
#
#   * `expr_sweep.sh`'s 47 other fragments all emit *expressions*; not one of
#     them writes a bare declaration, so the class was unrepresentable here.
#   * `differential.rs` names a fixed list of three fixtures.
#   * the 878-TU dc3 workload contains **ZERO** TUs whose section set is the
#     shell plus data (measured on `work/w-bss/census/sections.jsonl`: 8 objs
#     lack `.text`, and all 8 are shell-only or `??__E`), so `c2rs gap` read
#     `mismatch 0` over a defect it cannot represent.
#
# Three instruments green, one live wrong-emit family. That is `docs/STATUS.md`
# trap 5 — *absence reads as success unless something forbids it* — and this
# fragment is the something.
#
# ---- the axes are STRUCTURAL, and the values are deliberately boring ---------
#
# This project's grids have varied *values* exhaustively three times and missed
# on arity, register position and structural counts. So the initializer values
# here are almost all `1`, and the cross is over the shape:
#
#   A. OBJECT COUNT per non-COMDAT section — 1, 2, 3, 5, 8. The writer's class
#      bound is `<= 2` (`OBJ_DATA_BSS_SHAPE.md` §8.1: a `.bss` with exactly two
#      objects is exact on 47 of 48 real sections and 38 of 62 above that), so
#      the rows on both sides of the bound are what say the bound is real.
#   B. SECTION KIND — `.bss` alone, `.data` alone, both together, and the three
#      that must refuse: `.rdata` (a `const` pool or a string literal), `.tls$`,
#      and a COMDAT.
#   C. LINKAGE — every object `extern`, every object `static`, and mixed. Rule
#      Y1 (§6.2) says the `.bss` symbol table is externals-in-reverse-`.gl` then
#      statics-in-declaration-order, so a single-linkage grid cannot see it.
#   D. ALIGNMENT PADDING present vs absent. Ten of the 64 real sections whose
#      walk needs NO padding are still wrong (§8.1), so the no-padding rows are
#      the ones that isolate walk order from arithmetic.
#   E. SIZE, only at the promotion thresholds — `align = max(natural, 1 if n<2
#      else 4 if n<64 else 8)` steps at n=2 and n=64, and the size varint's own
#      escape steps at 128.
#   F. DECLARATION ORDER vs `.gl` ORDER. `.bss` walks `.gl` file order and
#      `.data` walks declaration order (§5.2/§5.3), and they are different
#      permutations of the same names — so the name multisets below are reused
#      across several declaration orders on purpose.
#   G. THE CONTROLS — TUs that genuinely are the bare four-section shell. A
#      fragment with only positive rows would pass with a rule that refuses
#      everything, which is the failure mode a refusal-shaped fix invites.
#
# NotImplemented is fine and expected on most rows. A MISMATCH is the alarm.

# Six names whose declaration order, sorted order and `.gl` order are pairwise
# different — `OBJ_DATA_BSS_SHAPE.md` §5.3 uses this set for exactly that reason.
WORDS = ("zulu", "alpha", "mike", "bravo", "yankee", "charlie")
SHORT = ("s9", "s1", "s7", "s3", "s5", "s2")


def cases(emit):
    # ---- A x B x C: object count x section kind x linkage --------------------
    #
    # `int` throughout so alignment padding is absent by construction: these rows
    # isolate the WALK from the arithmetic.
    for n in (1, 2, 3, 5, 8):
        names = [w for w in (WORDS + SHORT)[:n]]
        for stor in ("", "static "):
            # `.bss` alone — uninitialized.
            emit("".join("%sint %s;\n" % (stor, x) for x in names))
            # `.data` alone — initialized.
            emit("".join("%sint %s = 1;\n" % (stor, x) for x in names))
            # both, interleaved in source so the two sections' walks cross.
            emit("".join(
                "%sint %s%s;\n" % (stor, x, " = 1" if i % 2 else "")
                for i, x in enumerate(names)
            ))
        # MIXED linkage in one TU — the cell Rule Y1 needs and a single-linkage
        # grid cannot produce.
        emit("".join(
            "%sint %s;\n" % ("static " if i % 2 else "", x) for i, x in enumerate(names)
        ))
        emit("".join(
            "%sint %s = 1;\n" % ("static " if i % 2 else "", x) for i, x in enumerate(names)
        ))
        # …and the same names in the REVERSE declaration order. `.gl` order is
        # keyed on the name SET and not on position (§7.2), so these two rows
        # must produce the same `.bss` walk and a different `.data` one.
        emit("".join("int %s;\n" % x for x in reversed(names)))
        emit("".join("int %s = 1;\n" % x for x in reversed(names)))

    # ---- D x E: alignment padding, at the promotion thresholds ---------------
    #
    # `align = max(natural, 1 if n<2 else 4 if n<64 else 8)`. `char` is the only
    # type below the first step; `char[64]` is the only cheap way over the
    # second. Every pair below is written in both orders, because the padding a
    # walk needs depends on which object the cursor met first.
    TYPES = (
        ("char", "c"),          # n=1  -> align 1   (below the n<2 step)
        ("short", "h"),         # n=2  -> align 4
        ("int", "i"),           # n=4  -> align 4
        ("double", "d"),        # n=8  -> align 8   (natural beats implied)
    )
    ARRAYS = (
        ("char %s[3];\n", "a3"),    # 3  -> align 4, leaves a hole after a char
        ("char %s[63];\n", "a63"),  # 63 -> align 4  (just below the n<64 step)
        ("char %s[64];\n", "a64"),  # 64 -> align 8  (just above it)
        ("char %s[65];\n", "a65"),
        ("char %s[200];\n", "a200"),  # size varint escape (`80 c8 00 00 00`)
    )
    for i, (ta, na) in enumerate(TYPES):
        for tb, nb in TYPES[i:]:
            # uninitialized: both orders.
            emit("%s %s1;\n%s %s2;\n" % (ta, na, tb, nb))
            emit("%s %s2;\n%s %s1;\n" % (tb, nb, ta, na))
            # initialized: both orders. `double` is the FP row and must refuse
            # (§4.2.1 — a float's bytes are omitted from the aux CheckSum).
            emit("%s %s1 = 1;\n%s %s2 = 1;\n" % (ta, na, tb, nb))
            emit("%s %s2 = 1;\n%s %s1 = 1;\n" % (tb, nb, ta, na))
    for fmt, nm in ARRAYS:
        emit(fmt % nm)
        emit("char one;\n" + fmt % nm)
        emit((fmt % nm) + "char one;\n")
        emit("double eight;\n" + fmt % nm)
    # An over-aligned object: the section nibble has no encoding above ALIGN_8
    # in this writer, so it must REFUSE rather than round to something plausible.
    emit("__declspec(align(16)) char q1;\n")
    emit("__declspec(align(32)) int q2;\n")
    emit("__declspec(align(16)) char q1;\nchar q2;\n")

    # ---- B's refusals: the four section kinds that are NOT `.bss`/`.data` ----
    #
    # Each of these frames as a data record and lands somewhere else, and each
    # one was a live mismatch. They must refuse, and the refusal must survive
    # being written beside an ordinary object.
    REFUSE = (
        "extern const int ce = 9;\n",                    # -> .rdata, non-COMDAT
        "const char cs[4] = \"abc\";\nconst char* keep = cs;\n",
        "const char* s = \"hi\";\n",                      # -> .rdata COMDAT + reloc
        "__declspec(thread) int t1;\n",                  # -> .tls$
        "__declspec(thread) int t2 = 4;\n",              # -> .tls$
        "__declspec(thread) static int t3;\n",
        "__declspec(selectany) int sa = 3;\n",           # -> COMDAT .data
        "__declspec(selectany) int sb;\n",               # -> COMDAT .bss
        "int gi;\nint* gp = &gi;\n",                     # -> .data with a reloc
        "double fd = 1.0;\n",                            # -> the CheckSum exclusion
        "float ff = 1.0f;\n",
        "float fa[2] = {1.0f, 2.0f};\n",
        "struct S { int a; int b; };\nS s1 = {1, 2};\n",
        "struct P { char c; int i; };\nP p1 = {1, 2};\n",  # padding inside the object
        "long long ll = 1;\n",                            # 8-byte integer width
        "unsigned long long ul = 1;\n",
    )
    for src in REFUSE:
        emit(src)
        # …and beside an ordinary object, so a rule that only looks at the FIRST
        # record cannot pass.
        emit("int ordinary;\n" + src)
        emit(src + "int ordinary = 1;\n")

    # ---- G: THE CONTROLS. These are genuinely the bare four-section shell and
    # must stay `Match`. Without them a "refuse every functionless TU" fix passes
    # this fragment, and that fix would lose the six workload TUs whose obj IS
    # the shell (`GainEffect.cpp`, `PeakDetector.cpp`, …).
    for src in (
        "typedef int T;\n",
        "struct S;\n",
        "struct S; void f();\n",
        "extern int e;\n",
        "extern int e; extern int f;\n",
        "static const int k = 7;\n",       # folded away — no section
        "enum E { A, B };\n",
        "namespace N { }\n",
        "template <class T> struct X { T t; };\n",
        "#define M 1\n",
        "class C { public: int m; };\n",
        "typedef int T;\nextern T e;\nstruct S;\n",
    ):
        emit(src)

    # ---- I: data BESIDE a function. A different arm of `PortC2::build` (the
    # `.text` writers), which refuses today for its own reasons — these rows are
    # here so a future widening of that arm cannot quietly start emitting a TU
    # whose `.bss` nobody placed.
    for data in ("int gb;\n", "int gd = 1;\n", "int gb;\nint gd = 1;\n"):
        emit(data + "int f(int a){return a+1;}\n")
        emit("int f(int a){return a+1;}\n" + data)
        emit(data + "int u(){return gb_or_gd_unused();}\n".replace(
            "gb_or_gd_unused()", "0"))
