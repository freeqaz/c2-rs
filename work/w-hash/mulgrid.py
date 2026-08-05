#!/usr/bin/env python3
"""mulgrid.py — `a * k`, the whole constant axis, both modes.

Lane w-hash. `chain.rs:85` refuses `expr-out-of-class-mul-by-lit` with the
stated reason *"multiply by a constant strength-reduces to shifts and adds"*.
`divgrid`'s `s-mul-k127` reads a single `mulli`, so the stated reason is at
best incomplete. This grid is the full cross product the claim needs: it is
exactly the population `#644` warns about (a grid that silently held the
literal to one word).
"""
import os, sys, tempfile
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G
import importlib.util as _ilu
_s = _ilu.spec_from_file_location("wh_dg", os.path.join(HERE, "divgrid.py"))
_dg = _ilu.module_from_spec(_s); _s.loader.exec_module(_dg)

KS = [0,1,2,3,4,5,6,7,8,9,10,15,16,17,63,64,65,100,127,128,255,256,
      1000,32767,32768,65535,65536,100000,-1,-2,-3,-4,-7,-8,-127,-128,-32768,-32769]
def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i+1]; del argv[i:i+2]
    wd = tempfile.mkdtemp(prefix="whashmul")
    print("mode: %s" % mode)
    print("%8s  %-6s %-30s | %-6s %s" % ("k", "a*k", "", "k*a", ""))
    for k in KS:
        o1 = G.capture("int P(int a){ return a*(%d); }\n" % k, mode, wd, "m%s" % (str(k).replace('-','n')))
        o2 = G.capture("int P(int a){ return (%d)*a; }\n" % k, mode, wd, "n%s" % (str(k).replace('-','n')))
        f = lambda o: (" ".join(m for _,_,m in _dg.render(o)) if o else "FAIL")
        print("%8d  %-6d %-30s | %-6d %s" % (k, 4*len(_dg.render(o1)) if o1 else 0, f(o1),
                                             4*len(_dg.render(o2)) if o2 else 0, f(o2)))
    return 0
sys.exit(main(sys.argv))
