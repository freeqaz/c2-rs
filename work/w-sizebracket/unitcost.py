#!/usr/bin/env python3
"""unitcost.py — the `.gl` SIZE cost of one statement, by kind.

SIZE is a per-function count carried in the IL; this measures its increment for
a single added statement so a ladder can be built that hits every integer near
a boundary.
"""
import os, subprocess, sys
HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE); import glsize
C2RS = os.path.join(ROOT, "target", "release", "c2rs")
KINDS = {
    "none":      [],
    "mul_add":   ["s = s * 3 + 1;"],
    "mix":       ["s = (s * 3 + 1) ^ (s >> 2);"],
    "xor_shift": ["s ^= (a >> 3);"],
    "xor_self":  ["s ^= (s >> 5);"],
    "xor_var":   ["s ^= a;"],
    "neg":       ["s = -s;"],
    "not":       ["s = ~s;"],
    "add_var":   ["s += a;"],
    "shl":       ["s = s << 3;"],
    "cmp":       ["if (s > 3) s = 1;"],
}
os.makedirs(os.path.join(HERE, "uc"), exist_ok=True)
for name, stmts in KINDS.items():
    src = "int callee(int a) {\n  int s = a;\n" + "".join(f"  {x}\n" for x in stmts) + "  return s;\n}\nint caller(int a) { return callee(a) + 7; }\n"
    cpp = os.path.join(HERE, "uc", name + ".cpp"); open(cpp, "w").write(src)
    ild = os.path.join(HERE, "uc", "il_" + name)
    r = subprocess.run([C2RS, "capture", os.path.relpath(cpp, ROOT), "--keep-il", os.path.relpath(ild, ROOT),
                        "--flags-file", "work/w-sizebracket/flags_O1.txt"],
                       capture_output=True, text=True, cwd=ROOT, env=dict(os.environ, C2RS_REQUIRE_TOOLCHAIN="1"))
    if r.returncode != 0:
        print(f"{name:>10}  FAILED {r.stderr[-160:]}"); continue
    gl = open(os.path.join(ild, [x for x in os.listdir(ild) if x.endswith('.gl')][0]), "rb").read()
    sz = {x["name"]: x["size"] for x in glsize.records(gl)}
    print(f"{name:>10}  SIZE={sz.get('?callee@@YAHH@Z')}")
