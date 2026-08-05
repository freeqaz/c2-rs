#!/usr/bin/env python3
"""loopshape.py — from `leaf-ptrwalk` to `?HashString@@YAHPBDH@Z`, one
construct at a time.

Lane **w-hash**. Control: `work/w-hash/PREREG.md` (`1630f70`).

`w-tu1`'s technique, which is the one that has ever produced a conversion:
build a byte-exact base, then add ONE construct at a time and grade each
independently against real `c2` before the next. This is the grading half —
it prints the reference bytes for each step of the ladder so a rung can be
written against a difference of one construct rather than against 80 bytes at
once.

The ladder ends at the workload's own function, transcribed verbatim from
`src/system/math/Sort.cpp`, so the last row must reproduce `work/w-hash/Sort.obj`'s
`.text` exactly. That row is the **anchor control**: if it does not reproduce,
the probe construction differs from the workload's and nothing above it is a
reading of the target.

    work/w-hash/loopshape.py                 # /O1 (the workload's mode)
    work/w-hash/loopshape.py --dis <cell>
"""

import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G  # noqa: E402

sys.path.insert(0, HERE)
import importlib.util as _ilu  # noqa: E402
_spec = _ilu.spec_from_file_location("w_hash_divgrid", os.path.join(HERE, "divgrid.py"))
_dg = _ilu.module_from_spec(_spec)
_spec.loader.exec_module(_dg)


HASHSTRING = """int HashString(const char *str, int i) {
    int ret = 0;
    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {
        ret = (*u + ret * 0x7F) % i;
    }
    return ret;
}"""

LADDER = [
    # --- the induction variable alone -------------------------------------
    ("L0-count", "int P(const unsigned char* s){ int n=0;"
     " for (const unsigned char* u=s; *u; u++) n=n+1; return n; }",
     "pointer walk to a sentinel, body does not use *u"),
    ("L1-sum", "int P(const unsigned char* s){ int r=0;"
     " for (const unsigned char* u=s; *u; u++) r=r+*u; return r; }",
     "the body USES *u -- does the peeled `lbz`+`lbzu` pair appear?"),
    ("L1-sumc", "int P(const char* s){ int r=0;"
     " for (const char* u=s; *u; u++) r=r+*u; return r; }",
     "signed char element -- lbzu or lbz+extsb?"),
    ("L1-cast", "int P(const char* s){ int r=0;"
     " for (unsigned char* u=(unsigned char*)s; *u != 0; u++) r=r+*u; return r; }",
     "Sort.cpp's own induction spelling (cast + explicit != 0), body simplified"),

    # --- the accumulate ----------------------------------------------------
    ("L2-mul", "int P(const char* s){ int r=0;"
     " for (unsigned char* u=(unsigned char*)s; *u != 0; u++) r=*u+r*127; return r; }",
     "+ mulli. The whole RHS but no modulo"),
    ("L2-mul2", "int P(const char* s){ int r=0;"
     " for (unsigned char* u=(unsigned char*)s; *u != 0; u++) r=*u+r*2; return r; }",
     "the same with a power-of-two multiplier -- strength reduction inside a loop?"),

    # --- the modulo --------------------------------------------------------
    ("L3-modk", "int P(const char* s){ int r=0;"
     " for (unsigned char* u=(unsigned char*)s; *u != 0; u++) r=(*u+r*127)%7; return r; }",
     "literal modulo -- no twi (divgrid R3), so this isolates the loop from the traps"),
    ("L3-modv", HASHSTRING.replace("HashString", "P"),
     "the workload function, renamed"),
    ("L4-anchor", HASHSTRING,
     "ANCHOR: must reproduce work/w-hash/Sort.obj's .text byte for byte"),

    # --- counterfactuals on the loop's own shape ---------------------------
    ("X-while", "int P(const char* s){ int r=0; unsigned char* u=(unsigned char*)s;"
     " while (*u != 0) { r=(*u+r*127)%9; u++; } return r; }",
     "the same loop written as a `while` -- same bytes?"),
    ("X-idx", "int P(const char* s,int n){ int r=0;"
     " for (int k=0;k<n;k++) r=(s[k]+r*127)%9; return r; }",
     "an INDEX induction rather than a pointer -- does the peel survive?"),
    ("X-nomod", "int P(const char* s){ int r=0;"
     " for (unsigned char* u=(unsigned char*)s; *u != 0; u++) r=r+*u*3; return r; }",
     "a different body length -- does the peel depend on the body?"),
    ("X-store", "void P(const char* s,int* o){"
     " for (unsigned char* u=(unsigned char*)s; *u != 0; u++) *o=*o+*u; }",
     "a store in the body"),
    ("X-two", "int P(const char* s){ int r=0;"
     " for (unsigned char* u=(unsigned char*)s; *u != 0; u++) { r=r+*u; r=r+*u; } return r; }",
     "TWO uses of *u -- one peeled load feeding both?"),
]


def run(mode, wd, only=None, dis=False):
    ref = None
    bad = 0
    for name, src, note in LADDER:
        if only and name not in only:
            continue
        o = G.capture(src + "\n", mode, wd, name.replace("-", "_"))
        print("== %s  --  %s" % (name, note))
        if o is None:
            print("   CAPTURE FAILED")
            bad += 1
            continue
        r = _dg.render(o)
        print("   %d B: %s" % (4 * len(r), " ".join(m for _, _, m in r)))
        if dis or name in ("L4-anchor",):
            path = os.path.join(wd, "%s.obj" % name.replace("-", "_"))
            open(path, "wb").write(o.d)
            subprocess.run([sys.executable, os.path.join(REPO, "scripts", "gt_dump.py"), path])
        if name == "L4-anchor":
            ref = bytes()
            for s in o.sections:
                if s["name"] == ".text":
                    ref = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
        print()
    if ref is not None and not only:
        want = open(os.path.join(HERE, "Sort.text.bin"), "rb").read() \
            if os.path.exists(os.path.join(HERE, "Sort.text.bin")) else None
        if want is None:
            print("  (no Sort.text.bin to compare -- write it with --save-anchor)")
        elif want != ref:
            print("!! ANCHOR CONTROL FAILED: L4 does not reproduce Sort.obj's .text")
            bad += 1
        else:
            print("  anchor control: L4 reproduces Sort.obj's .text, %d B" % len(ref))
    print("controls failed: %d" % bad)
    return bad


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    dis = "--dis" in argv
    only = [a for a in argv[1:] if not a.startswith("--")]
    wd = tempfile.mkdtemp(prefix="whashls")
    print("mode: %s   workdir: %s" % (mode, wd))
    print()
    return 1 if run(mode, wd, only or None, dis) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
