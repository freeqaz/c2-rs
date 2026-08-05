#!/usr/bin/env python3
"""sygrid.py — what does a `.sy` automatic-local record say for a POINTER?

Lane w-hash. `SyBlock::int_locals` admits plain `int` automatics only (kind
0x01, size 4, tid 0x74), which is exactly right for `assign.rs`'s
value-substituting parse and leaves the pointer-walk loop's induction variable
with no positive local test at all.

Widening `.sy` is the one thing its own module docs warn hardest about — six
widths were wrong there and every one of them "agreed on small, simple,
hand-written declarations". So this grid varies the local's TYPE one axis at a
time and prints the raw record, and the predicate that ships is read off the
table rather than from two witnesses in one file.
"""
import os, re, subprocess, sys, tempfile
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))

C2RS = os.path.join(REPO, "target", "release", "c2rs")

CELLS = [
    ("int",        "int P(int a){ int x = a; x = x + 1; return x; }"),
    ("uint",       "int P(int a){ unsigned x = a; x = x + 1; return x; }"),
    ("const-int",  "int P(int a){ const int x = a; return x; }"),
    ("short",      "int P(int a){ short x = (short)a; return x; }"),
    ("char",       "int P(int a){ char x = (char)a; return x; }"),
    ("uchar",      "int P(int a){ unsigned char x = (unsigned char)a; return x; }"),
    ("bool",       "int P(int a){ bool x = a != 0; return x; }"),
    ("longlong",   "int P(int a){ long long x = a; return (int)x; }"),
    ("float",      "int P(int a){ float x = (float)a; return (int)x; }"),
    ("double",     "int P(int a){ double x = a; return (int)x; }"),
    ("ptr-uchar",  "int P(const char* s){ unsigned char* u = (unsigned char*)s; return *u; }"),
    ("ptr-char",   "int P(const char* s){ const char* u = s; return *u; }"),
    ("ptr-int",    "int P(int* s){ int* u = s; return *u; }"),
    ("ptr-void",   "int P(void* s){ void* u = s; return u != 0; }"),
    ("ptr-cuchar", "int P(const char* s){ const unsigned char* u = (const unsigned char*)s; return *u; }"),
    ("ptr-ptr",    "int P(char** s){ char** u = s; return **u != 0; }"),
    ("ptr-fn",     "typedef int (*F)(int); int P(F f){ F g = f; return g(1); }"),
    ("array",      "int P(int a){ int v[4]; v[0]=a; return v[0]; }"),
    ("struct",     "struct S{int a;int b;}; int P(int a){ S s; s.a=a; return s.a; }"),
    ("addrtaken",  "int q(int*); int P(int a){ int x = a; return q(&x); }"),
    ("ptr-addrtaken","int q(char**); int P(char* s){ char* u = s; return q(&u); }"),
]

REC = re.compile(rb"")

def dump(name, src, wd):
    cpp = os.path.join(wd, name.replace("-", "_") + ".cpp")
    open(cpp, "w").write(src + "\n")
    d = os.path.join(wd, "il_" + name.replace("-", "_"))
    os.makedirs(d, exist_ok=True)
    r = subprocess.run([C2RS, "capture", cpp, "--keep-il", d], capture_output=True, text=True)
    sy = [f for f in os.listdir(d) if f.endswith(".sy")]
    if not sy:
        print("%-14s CAPTURE FAILED %s" % (name, r.stderr.strip()[:80])); return
    b = open(os.path.join(d, sy[0]), "rb").read()
    print("%-14s %s" % (name, b.hex(" ")))

def main(argv):
    wd = tempfile.mkdtemp(prefix="whashsy")
    only = [a for a in argv[1:] if not a.startswith("--")]
    for name, src in CELLS:
        if only and name not in only: continue
        dump(name, src, wd)
    return 0
sys.exit(main(sys.argv))
