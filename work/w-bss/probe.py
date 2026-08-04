#!/usr/bin/env python3
"""Generate a data-only TU from a name list, compile it with real c2, return the
.bss address order.  Lane w-bss scratch tooling."""
import os, subprocess, sys, hashlib
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from coffdump import Obj

W = os.path.dirname(os.path.abspath(__file__))
R = os.path.dirname(os.path.dirname(W))
C2RS = os.path.join(R, "target/release/c2rs")
FLAGS = os.path.join(W, os.environ.get("WBSS_FLAGS", "flags-w.txt"))

def compile_src(src, tag):
    cpp = os.path.join(W, "q_%s.cpp" % tag)
    obj = os.path.join(W, "q_%s.obj" % tag)
    open(cpp, "w").write(src)
    r = subprocess.run([C2RS, "compile", os.path.basename(cpp), "--cwd", W,
                        "--flags-file", FLAGS, "--keep-obj", obj],
                       capture_output=True)
    if r.returncode != 0 or not os.path.exists(obj):
        raise RuntimeError("compile failed %s: %s" % (tag, r.stderr.decode()[:400]))
    return obj

def bss_order(obj, secname=".bss"):
    o = Obj(open(obj, "rb").read())
    idx = [s["idx"] for s in o.secs if s["name"] == secname]
    if not idx:
        return None, o
    i = idx[0]
    got = sorted((sy["val"], sy["name"]) for sy in o.syms if sy["sec"] == i and sy["naux"] == 0)
    return got, o

def demangle(n):
    # `?name@@3DA` -> `name`
    if n.startswith("?") and "@@" in n:
        return n[1:n.index("@@")]
    return n

def order_extern(names, tag=None, typ="char"):
    """family C: uninitialized externals, one per name."""
    src = "".join("%s %s;\n" % (typ, n) for n in names)
    tag = tag or hashlib.md5(src.encode()).hexdigest()[:12]
    obj = compile_src(src, tag)
    got, o = bss_order(obj)
    assert got is not None, "no .bss in %s" % tag
    res = [demangle(n) for _, n in got]
    # CONTROL: every name must appear exactly once, section size == len(names)
    sec = [s for s in o.secs if s["name"] == ".bss"][0]
    assert sorted(res) == sorted(names), "probe %s lost names: %s" % (tag, set(names) ^ set(res))
    assert sec["size"] == len(names), "probe %s size %d != %d" % (tag, sec["size"], len(names))
    return res

if __name__ == "__main__":
    import string
    ns = ["s%d" % i for i in range(1, 9)]
    print(order_extern(ns, "chk8"))
