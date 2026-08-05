#!/usr/bin/env python3
"""hashgrid.py — is `HashString`'s emission a FUNCTION of the class parameters?

Lane w-hash. The decisive measurement before any emitter is written: vary the
pointer formal's position, the divisor formal's position, the multiplier, the
accumulator's initial value and the element/return types, and ask whether the
twenty words move as a function of those alone (register numbers substituted)
or whether the allocator re-plans.

If it re-plans on any axis, that axis is out of class and the refusal is
positive. `w-tu3` §6 and `w-sched` both declined to fit a schedule; this grid
is what makes a narrow class honest rather than fitted -- every cell it accepts
is graded by real c2.
"""
import os, sys, tempfile
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G
import importlib.util as _ilu
_s = _ilu.spec_from_file_location("wh_dg", os.path.join(HERE, "divgrid.py"))
_dg = _ilu.module_from_spec(_s); _s.loader.exec_module(_dg)

BASE = ("int P(const char *str, int i) {\n"
        "    int ret = %(k0)s;\n"
        "    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {\n"
        "        ret = (*u + ret * %(K)s) %% i;\n"
        "    }\n"
        "    return ret;\n}\n")

CELLS = []
# axis K -- the multiplier
for K in ["0x7F", "3", "5", "31", "1000", "32767", "2", "1", "0", "-3", "100000"]:
    CELLS.append(("K=%s" % K, BASE % {"k0": "0", "K": K}))
# axis k0 -- the accumulator's initial value
for k0 in ["0", "1", "7", "-1", "1000"]:
    CELLS.append(("k0=%s" % k0, BASE % {"k0": k0, "K": "0x7F"}))
# axis: formal order / arity
CELLS += [
    ("swap", "int P(int i, const char *str) {\n int ret = 0;\n"
     " for (unsigned char *u = (unsigned char *)str; *u != 0; u++) ret = (*u + ret*0x7F) % i;\n"
     " return ret; }\n"),
    ("p3", "int P(int q, int w, const char *str, int i) {\n int ret = 0;\n"
     " for (unsigned char *u = (unsigned char *)str; *u != 0; u++) ret = (*u + ret*0x7F) % i;\n"
     " return ret; }\n"),
    ("divfirst", "int P(int i, int q, const char *str) {\n int ret = 0;\n"
     " for (unsigned char *u = (unsigned char *)str; *u != 0; u++) ret = (*u + ret*0x7F) % i;\n"
     " return ret; }\n"),
    ("uptr", "int P(const unsigned char *str, int i) {\n int ret = 0;\n"
     " for (const unsigned char *u = str; *u != 0; u++) ret = (*u + ret*0x7F) % i;\n"
     " return ret; }\n"),
    ("nocast", "int P(unsigned char *u, int i) {\n int ret = 0;\n"
     " for (; *u != 0; u++) ret = (*u + ret*0x7F) % i;\n"
     " return ret; }\n"),
    ("truthy", "int P(const char *str, int i) {\n int ret = 0;\n"
     " for (unsigned char *u = (unsigned char *)str; *u; u++) ret = (*u + ret*0x7F) % i;\n"
     " return ret; }\n"),
    ("commuted", "int P(const char *str, int i) {\n int ret = 0;\n"
     " for (unsigned char *u = (unsigned char *)str; *u != 0; u++) ret = (ret*0x7F + *u) % i;\n"
     " return ret; }\n"),
    ("udiv", "unsigned P(const char *str, unsigned i) {\n unsigned ret = 0;\n"
     " for (unsigned char *u = (unsigned char *)str; *u != 0; u++) ret = (*u + ret*0x7Fu) % i;\n"
     " return ret; }\n"),
    ("divk", "int P(const char *str) {\n int ret = 0;\n"
     " for (unsigned char *u = (unsigned char *)str; *u != 0; u++) ret = (*u + ret*0x7F) % 9;\n"
     " return ret; }\n"),
    ("noacc", "int P(const char *str, int i) {\n int ret = 0;\n"
     " for (unsigned char *u = (unsigned char *)str; *u != 0; u++) ret = (*u) % i;\n"
     " return ret; }\n"),
    ("twofn", "int gz(int);\nint P(const char *str, int i) {\n int ret = 0;\n"
     " for (unsigned char *u = (unsigned char *)str; *u != 0; u++) ret = (*u + ret*0x7F) % i;\n"
     " return ret; }\nint z9(int a){ return gz(a)+7; }\n"),
]

def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i+1]; del argv[i:i+2]
    only = [a for a in argv[1:] if not a.startswith("--")]
    wd = tempfile.mkdtemp(prefix="whashhg")
    print("mode: %s\n" % mode)
    for name, src in CELLS:
        if only and name not in only: continue
        o = G.capture(src, mode, wd, name.replace("=","_").replace("-","_"))
        if o is None:
            print("%-10s CAPTURE FAILED" % name); continue
        # first .text only (the probe function is first)
        r = _dg.render(o)
        print("%-10s %3dB  %s" % (name, 4*len(r), " ".join(m for _,_,m in r)))
        if "--dis" in argv:
            for off, w, m in r:
                print("        %04x  %08x  %s" % (off, w, m))
    return 0
sys.exit(main(sys.argv))
