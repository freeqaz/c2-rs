#!/usr/bin/env python3
"""ctl_ob0.py — THE CONTROL THE /Ob0 SITE ENUMERATOR NEEDS.

Lane w-inline measurement tooling. Read-only with respect to `crates/`.

`grade_pair.py` rests entirely on one claim: **at `/Ob0` a source-level call to
a same-TU function leaves a REL24.** If it does not — if some elision survives
`/Ob0` — then every `INLINED-ALL` this lane reports is a call that was never
there, and the enumerator is measuring its own blind spot.

So the claim is tested on hand probes whose answer is known in advance, at the
workload's own flags, with and without `/Ob0`, and **it can go red**: p4 is a
non-empty callee that `/Ob0` must restore, and if it does not the enumerator is
broken and this file says so rather than the lane reporting a number.

    p2  `void g() {} void f() { g(); }`      — w-fnbyte §5.2's own probe
    p3  a trivial destructor over a trivial base
    p4  `int g(int a){return a+1;} int f(int a){return g(a);}`  — the POSITIVE
        control: a callee with a body, which /Ob0 must leave as a real call

Usage: ctl_ob0.py [outdir]
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
from scan_obj import read_obj  # noqa: E402

PROBES = {
    "p2": "void g() {}\nvoid f() { g(); }\n",
    "p3": "struct A { ~A() {} };\nstruct B : A { ~B() {} };\n"
          "void mk(B *p) { p->~B(); }\n",
    "p4": "int g(int a) { return a + 1; }\nint f(int a) { return g(a); }\n",
    "p5": "struct A { ~A() { } int x; };\nstruct B : A { ~B() { } int y; };\n"
          "void mk(B *p) { p->~B(); }\n",
    # THE DISCRIMINATOR. `g` here is NOT empty in source -- it returns its
    # argument -- but it EMITS exactly `blr`, because the PPC ABI already has
    # the value in r3. If the call is dropped, the rule reads the emitted body;
    # if it survives, the rule reads the source.
    "p6": "int g(int a) { return a; }\nint f(int a) { return g(a) + 0; }\n",
    # …and its control: the same shape one instruction bigger.
    "p7": "int g(int a) { return a + 1; }\nint f(int a) { return g(a) + 0; }\n",
    # A callee that is empty in source and takes an argument it ignores.
    "p8": "void g(int a) { }\nvoid f(int a) { g(a); }\n",
}


def sib(name):
    d = REPO
    while d != "/":
        if os.path.isdir(os.path.join(d, "..", name)):
            return os.path.abspath(os.path.join(d, "..", name))
        d = os.path.dirname(d)
    return None


def main(argv):
    out = argv[0] if argv else os.path.join(HERE, "ctl")
    os.makedirs(out, exist_ok=True)
    wibo = os.environ.get("C2RS_WIBO") or os.path.join(sib("wibo") or "", "build/release/wibo")
    cl = os.path.join(REPO, "compilers/X360/16.00.11886.00/cl.exe")
    if not (os.path.exists(wibo) and os.path.exists(cl)):
        print("SKIP: toolchain absent")
        return 3
    flags = open(os.path.join(REPO, "work/dc3-workload/flags.txt")).read().split()
    bad = 0
    for p, text in PROBES.items():
        src = os.path.join(out, p + ".cpp")
        open(src, "w").write(text)
        seen = {}
        for tag, extra in (("o1", []), ("ob0", ["/Ob0"])):
            obj = os.path.join(out, f"{p}_{tag}.obj")
            zin = "Z:" + os.path.abspath(src).replace("/", "\\")
            zout = "Z:" + os.path.abspath(obj).replace("/", "\\")
            subprocess.run(
                [wibo, cl] + flags + extra + ["/Fo" + zout, zin],
                capture_output=True, cwd=out,
                env={**os.environ, "TMP": out, "TEMP": out, "WIBO_FS_CACHE": "1"})
            if not os.path.exists(obj):
                print(f"{p} {tag}: NO OBJ")
                bad += 1
                continue
            fns = read_obj(obj)
            seen[tag] = {n: (f.size, [t for t in f.rel24 if t in fns])
                         for n, f in fns.items()}
            for n, (sz, rel) in sorted(seen[tag].items()):
                print(f"  {p:3s} {tag:3s} {n:34s} {sz:4d} B  internal REL24 -> {rel}")
        # THE POSITIVE CONTROL. p4's `f` must hold no internal REL24 at /O1 (c2
        # inlines a 8-byte callee) and exactly one at /Ob0.
        if p == "p4" and seen:
            f_o1 = next((v for k, v in seen.get("o1", {}).items() if k.startswith("?f@")), None)
            f_b0 = next((v for k, v in seen.get("ob0", {}).items() if k.startswith("?f@")), None)
            ok = f_o1 is not None and f_b0 is not None and not f_o1[1] and len(f_b0[1]) == 1
            print(f"  ==> POSITIVE CONTROL p4: {'PASS' if ok else 'FAIL'} "
                  f"(/O1 internal rel24 {f_o1[1] if f_o1 else '?'}, "
                  f"/Ob0 {f_b0[1] if f_b0 else '?'})")
            bad += 0 if ok else 1
    print("ctl_ob0:", "PASS" if bad == 0 else f"FAIL ({bad})")
    return 0 if bad == 0 else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
