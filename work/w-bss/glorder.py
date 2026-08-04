#!/usr/bin/env python3
"""Lane w-bss: is the .bss/.data address order a function of the IL `.gl`
symbol-record order?

For each probe source: capture the IL, read the `.gl` name records in FILE
order, compile the same source to an obj with the real c2, read the .bss/.data
symbol offsets, and compare.  No hash is fitted; this is a pure correspondence
test between an INPUT the port already has and an OUTPUT it must reproduce.
"""
import os, re, sys, glob, shutil, subprocess
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj

W = os.path.dirname(os.path.abspath(__file__))
R = os.path.dirname(os.path.dirname(W))
C2RS = os.path.join(R, "target/release/c2rs")
FLAGS = os.path.join(W, os.environ.get("WBSS_FLAGS", "flags-w.txt"))

SHELL_NAMES = ('.XBLD$W', '__C1_11886', '__C2_11886', '@comp.id')


def demangle(n):
    """`?x@@3DA` (external linkage) and `$x` (internal linkage, .gl form) -> `x`."""
    if n.startswith("?") and "@@" in n:
        return n[1:n.index("@@")]
    if n.startswith("$") and len(n) > 1:
        return n[1:]
    return n


def gl_order(src, tag):
    """The `.gl` name records, in file order, shell records dropped."""
    cpp = os.path.join(W, "g_%s.cpp" % tag)
    out = os.path.join(W, "il", tag)
    open(cpp, "w").write(src)
    shutil.rmtree(out, ignore_errors=True)
    r = subprocess.run([C2RS, "capture", os.path.basename(cpp), "--keep-il", out],
                       capture_output=True, cwd=W)
    g = glob.glob(os.path.join(out, "*.gl"))
    if not g:
        raise RuntimeError("no .gl for %s: %s" % (tag, r.stderr.decode()[:300]))
    d = open(g[0], "rb").read()
    names = []
    for m in re.finditer(rb'[ -~]{2,}', d):
        s = m.group().decode()
        if s in SHELL_NAMES or '\\' in s or '/' in s or s.startswith('.'):
            continue
        if not (s[0].isalpha() or s[0] in '?_$'):
            continue
        names.append(s)
    return names


def obj_order(src, tag, secname):
    cpp = os.path.join(W, "g_%s.cpp" % tag)
    obj = os.path.join(W, "g_%s.obj" % tag)
    open(cpp, "w").write(src)
    r = subprocess.run([C2RS, "compile", os.path.basename(cpp), "--cwd", W,
                        "--flags-file", FLAGS, "--keep-obj", obj], capture_output=True)
    if not os.path.exists(obj):
        raise RuntimeError("compile failed %s: %s" % (tag, r.stderr.decode()[:300]))
    o = Obj(open(obj, "rb").read())
    idx = [s["idx"] for s in o.secs if s["name"] == secname]
    if not idx:
        return []
    i = idx[0]
    got = sorted((sy["val"], sy["name"]) for sy in o.syms
                 if sy["sec"] == i and sy["naux"] == 0)
    return [n for _, n in got]


def cell(label, src, tag, secname=".bss"):
    gl = [demangle(x) for x in gl_order(src, tag)]
    ob = [demangle(x) for x in obj_order(src, tag, secname)]
    glf = [x for x in gl if x in set(ob)]
    verdict = ("== .gl order" if ob == glf else
               "== REVERSE(.gl)" if ob == glf[::-1] else "NEITHER")
    print("  %-26s %s" % (label, verdict))
    print("      .gl        : %s" % " ".join(glf))
    print("      %-10s : %s" % (secname, " ".join(ob)))
    return verdict


if __name__ == "__main__":
    print("FLAGS: %s\n" % open(FLAGS).read().strip())
    N = ['p1', 'p2', 'p3', 'p4', 'd1', 'd2', 'd3', 'd4']
    M = ['alpha', 'bravo', 'charlie', 'delta', 'echo', 'foxtrot']

    print("1. UNINITIALIZED namespace-scope objects -> .bss")
    cell("8 externs",        "".join("char %s;\n" % n for n in N), 'e8')
    cell("6 externs (words)", "".join("char %s;\n" % n for n in M), 'e6')
    cell("8 statics+fnref",
         "".join("static char %s;\n" % n for n in N) +
         "".join("char* f%d(){return &%s;}\n" % (i, n) for i, n in enumerate(N)), 'st8')

    print("\n2. INITIALIZED namespace-scope objects -> .data")
    cell("8 externs =1",     "".join("char %s=1;\n" % n for n in N), 'i8', ".data")
    cell("6 externs (words)", "".join("int %s=7;\n" % n for n in M), 'i6', ".data")

    print("\n3. DYNAMIC-INITIALIZER objects -> .bss")
    cell("8 static L(1)",
         "struct L{L(int);};\n" + "".join("static L %s(1);\n" % n for n in N), 'dy8')
    cell("6 static L(1)",
         "struct L{L(int);};\n" + "".join("static L %s(1);\n" % n for n in M), 'dy6')

    print("\n4. MIXED eager + deferred in one .bss")
    cell("4 plain + 4 dyninit",
         "struct L{L(int);};\n" + "".join("char p%d;\n" % i for i in range(1, 5)) +
         "".join("static L d%d(1);\n" % i for i in range(1, 5)), 'mx')

    print("\n5. DECLARATION-ORDER INVARIANCE (same names, permuted source order)")
    import random
    rnd = random.Random(11)
    base = None
    for t in range(4):
        d = rnd.sample(N, len(N))
        gl = [demangle(x) for x in gl_order("".join("char %s;\n" % n for n in d), 'dp%d' % t)]
        ob = [demangle(x) for x in obj_order("".join("char %s;\n" % n for n in d), 'dp%d' % t, ".bss")]
        glf = [x for x in gl if x in set(ob)]
        if base is None:
            base = ob
        print("    decl %-32s .gl=%-32s .bss=%-32s bss==gl:%s  bss==first:%s"
              % (" ".join(d), " ".join(glf), " ".join(ob), ob == glf, ob == base))
