# **FOUR-LEAF chains — the depth at which the intermediate-register rules
# diverge.**  `lane w-build`.
#
# `10-int-chains.py` enumerates `l1 o1 l2 o2 l3`: THREE leaves, so exactly ONE
# intermediate — and with one intermediate every candidate allocation rule puts
# it in r11. That fragment is therefore structurally incapable of separating
# them, which is why a live wrong-bytes emit sat on master under a sweep
# reporting `0 mismatch`:
#
#     int f(int a,int b,int c,int d) { return (a + b) * c * d; }
#     c2   /Ox   add r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6
#     port /Ox   add r11,r3,r4 ; mullw r11,r11,r5 ; mullw r3,r11,r6
#
# `il_accum4.cpp` records an 11,664-case four-leaf enumeration that found the
# per-chain accumulator bug — but it was a ONE-OFF, and nothing in
# `scripts/sweep.d/` carried the axis forward. This fragment is that axis, made
# standing. `docs/GAPS.md`'s lesson about a corpus that cannot separate the
# candidate rules applies to a generated corpus exactly as it does to a
# hand-written one, once the generator's own axes are too shallow.
#
# 72 cases: four distinct formals, three operator positions, the leading
# operator ranging over the whole binary vocabulary (so an `add` in the position
# whose consumer decides the allocation is always represented) and the trailing
# two over a spanning subset.


def cases(emit):
    lead = ['+', '-', '*', '&', '|', '^', '<<', '>>']
    rest = ['+', '*', '&']
    for o1 in lead:
        for o2 in rest:
            for o3 in rest:
                # **PARENTHESIZED, and that is not tidiness.** The first
                # revision of this fragment emitted `a o1 b o2 c o3 d` bare and
                # reported `mismatches=0` against the very master binary whose
                # mis-emit it was written to catch — because C++ precedence
                # reassociates `a + b * c * d` into `a + ((b*c)*d)`, a depth-3
                # tree the parser refuses outright. A grid that cannot contain
                # its own counterexample; caught by requiring this fragment to
                # REPRODUCE the known failure before it was committed.
                emit(
                    "int f(int a, int b, int c, int d) "
                    "{ return ((a %s b) %s c) %s d; }\n" % (o1, o2, o3)
                )
