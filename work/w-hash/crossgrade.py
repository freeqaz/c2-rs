#!/usr/bin/env python3
"""crossgrade.py — the ptr-walk loop class, graded cell by cell by REAL c2.

Lane w-hash. The emitter is a twenty-word transcription with two free fields,
so the two places it can be wrong in a way the single fixture cannot see are
exactly `<K0>` and `<K>` — and they must be graded over their CROSS PRODUCT,
not two rows through the origin. The single-cell trap has fired five times on
this project (`!=`->`>` at exactly 63 burners; a 32768 bound; unsigned k=0
emitting a bare blr; a mask collapse that moved block layout as well as the
instruction; C = 0x249b0000 wrong on 29 of 32 columns of its own row).

It grades BOTH directions, which is the point:

  * every cell the class ACCEPTS must come back `match` — byte-exact obj
    against real c2.dll under wibo, TimeDateStamp zeroed;
  * every cell it REFUSES must come back `vocab-gap` or `codegen-gap`, never
    `mismatch`. A refusal that turns out to emit is the alarm.

Run:  work/w-hash/crossgrade.py [--jobs N]
Exit non-zero on ANY mismatch, on any accept cell that did not match, and on
any refuse cell that did.
"""
import os, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
C2RS = os.path.join(REPO, "target", "release", "c2rs")
FLAGS = os.path.join(REPO, "work", "dc3-workload", "flags.txt")

BASE = """int P%(tag)s(const char *str, int i) {
    int ret = %(k0)d;
    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {
        ret = (*u + ret * %(K)d) %% i;
    }
    return ret;
}
"""

# The two free fields, FULL cross product.
K0S = [0, 1, 7, -1, 1000, -32768, 32767]
KS_IN = [3, 5, 31, 127, 1000, 32767, 999]
# Out of class, each with the instruction c2 emits instead (divgrid/mulgrid).
KS_OUT = {2: "rlwinm", 4: "rlwinm", 8: "rlwinm", 32768: "rlwinm",
          1: "identity", 0: "li 0", -1: "neg", -3: "mulli, and the add becomes subf",
          65535: "addis/ori/mullw", 100000: "addis/ori/mullw"}

# Spelling variants that must be INSIDE the class (same IL, same bytes) and
# structural variants that must be OUTSIDE it (c2 re-plans the whole block).
SPELL_IN = {
    "truthy":   "int P%(tag)s(const char *str, int i){ int ret=0;\n"
                " for (unsigned char *u=(unsigned char *)str; *u; u++) ret=(*u+ret*127)%%i;\n"
                " return ret; }\n",
    "commuted": "int P%(tag)s(const char *str, int i){ int ret=0;\n"
                " for (unsigned char *u=(unsigned char *)str; *u!=0; u++) ret=(ret*127+*u)%%i;\n"
                " return ret; }\n",
    "uptr":     "int P%(tag)s(const unsigned char *str, int i){ int ret=0;\n"
                " for (const unsigned char *u=str; *u!=0; u++) ret=(*u+ret*127)%%i;\n"
                " return ret; }\n",
}
SPELL_OUT = {
    "swap":     "int P%(tag)s(int i, const char *str){ int ret=0;\n"
                " for (unsigned char *u=(unsigned char *)str; *u!=0; u++) ret=(*u+ret*127)%%i;\n"
                " return ret; }\n",
    "p3":       "int P%(tag)s(int q,int w,const char *str,int i){ int ret=0;\n"
                " for (unsigned char *u=(unsigned char *)str; *u!=0; u++) ret=(*u+ret*127)%%i;\n"
                " return ret; }\n",
    "nocast":   "int P%(tag)s(unsigned char *u, int i){ int ret=0;\n"
                " for (; *u!=0; u++) ret=(*u+ret*127)%%i;\n return ret; }\n",
    "divk":     "int P%(tag)s(const char *str){ int ret=0;\n"
                " for (unsigned char *u=(unsigned char *)str; *u!=0; u++) ret=(*u+ret*127)%%9;\n"
                " return ret; }\n",
    "udiv":     "unsigned P%(tag)s(const char *str, unsigned i){ unsigned ret=0;\n"
                " for (unsigned char *u=(unsigned char *)str; *u!=0; u++) ret=(*u+ret*127u)%%i;\n"
                " return ret; }\n",
    "divop":    "int P%(tag)s(const char *str, int i){ int ret=0;\n"
                " for (unsigned char *u=(unsigned char *)str; *u!=0; u++) ret=(*u+ret*127)/i;\n"
                " return ret; }\n",
    "stride2":  "int P%(tag)s(const char *str, int i){ int ret=0;\n"
                " for (unsigned char *u=(unsigned char *)str; *u!=0; u+=2) ret=(*u+ret*127)%%i;\n"
                " return ret; }\n",
    "shortelem":"int P%(tag)s(const short *str, int i){ int ret=0;\n"
                " for (const short *u=str; *u!=0; u++) ret=(*u+ret*127)%%i;\n"
                " return ret; }\n",
    "acc-wide": "int P%(tag)s(const char *str, int i){ int ret=100000;\n"
                " for (unsigned char *u=(unsigned char *)str; *u!=0; u++) ret=(*u+ret*127)%%i;\n"
                " return ret; }\n",
    "ltzero":   "int P%(tag)s(const char *str, int i){ int ret=0;\n"
                " for (unsigned char *u=(unsigned char *)str; *u>0; u++) ret=(*u+ret*127)%%i;\n"
                " return ret; }\n",
}


def main(argv):
    jobs = "8"
    if "--jobs" in argv:
        jobs = argv[argv.index("--jobs") + 1]
    wd = tempfile.mkdtemp(prefix="whashcg")
    cells = []          # (name, relpath, expect)  expect in {"match","refuse"}
    n = 0
    for k0 in K0S:
        for K in KS_IN:
            n += 1
            name = "in_k0%s_K%s" % (str(k0).replace("-", "n"), K)
            cells.append((name, name + ".cpp", "match"))
            open(os.path.join(wd, name + ".cpp"), "w").write(
                BASE % {"tag": "", "k0": k0, "K": K})
    for K, why in KS_OUT.items():
        for k0 in (0, 7):
            name = "out_k0%s_K%s" % (k0, str(K).replace("-", "n"))
            cells.append((name + "  [c2: %s]" % why, name + ".cpp", "refuse"))
            open(os.path.join(wd, name + ".cpp"), "w").write(
                BASE % {"tag": "", "k0": k0, "K": K})
    for name, src in SPELL_IN.items():
        cells.append(("spell_" + name, "s_" + name + ".cpp", "match"))
        open(os.path.join(wd, "s_" + name + ".cpp"), "w").write(src % {"tag": ""})
    for name, src in SPELL_OUT.items():
        cells.append(("shape_" + name, "x_" + name + ".cpp", "refuse"))
        open(os.path.join(wd, "x_" + name + ".cpp"), "w").write(src % {"tag": ""})

    lst = os.path.join(wd, "files.txt")
    open(lst, "w").write("\n".join(c[1] for c in cells) + "\n")
    r = subprocess.run(
        [C2RS, "gap", "--list", lst, "--flags-file", FLAGS, "--cwd", wd,
         "--jobs", jobs, "--no-cache"],
        capture_output=True, text=True)
    verdict = {}
    for line in r.stdout.splitlines():
        s = line.strip()
        if s.startswith("[") and "]" in s:
            # `[n/m] <verdict> <path>  (<reason>)` — the reason is optional and
            # is words, so the path is field 1 and never the last field.
            rest = s.split("]", 1)[1].split()
            if len(rest) >= 2:
                verdict[rest[1]] = rest[0]
    bad = 0
    print("%-40s %-14s %-12s" % ("cell", "expected", "graded"))
    print("-" * 70)
    for name, rel, expect in cells:
        got = verdict.get(rel, "NO-RESULT")
        ok = (got == "match") if expect == "match" else (got in ("vocab-gap", "codegen-gap"))
        if got == "mismatch":
            ok = False
        if not ok:
            bad += 1
        print("%-40s %-14s %-12s %s" % (name, expect, got, "" if ok else "<== FAIL"))
    nmatch = sum(1 for _, r_, e in cells if e == "match")
    nref = len(cells) - nmatch
    print()
    print("graded %d cells: %d must-match, %d must-refuse" % (len(cells), nmatch, nref))
    print("mismatch cells: %d" % sum(1 for v in verdict.values() if v == "mismatch"))
    print("FAILED: %d" % bad)
    if r.returncode != 0:
        print("gap exit %d" % r.returncode)
        print(r.stderr[-2000:])
    return 1 if bad else 0


sys.exit(main(sys.argv))
