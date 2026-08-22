#!/usr/bin/env python3
"""candid_c1.py -- control C1 for whitebox read R1 (docs/whitebox/WB_CANDID_*).

Preregistered in docs/whitebox/WB_CANDID_PREREG.md section 6 and committed
BEFORE it was run.  It asks the obj -- the sole judge -- whether a function's
emitted bytes can depend on how many register-allocation candidates the
functions BEFORE it in the same TU minted.

  C1     a tie-sensitive probe P compiled alone, and again after N fillers.
  C1-pos the POSITIVE CONTROL: a P perturbed by one operand MUST come back
         DIFFERENT.  If it does not, the instrument is dead and C1's green is
         discarded rather than published -- "absence must never read as
         success" (docs/STATUS.md).
  C1b    every filler F0..F(N-1) is a CHARACTER-IDENTICAL body distinguished
         only by its name.  Under a function-scoped candidate counter every Fi
         must equal F0.  Under a compilation-global one, Fi's ids are shifted
         by about i*k and wrap the 1024-bucket hash at 0x10c43b80 once
         i*k > 1024, permuting 0x10b316b1's bucket walk.

RED (exit 1) = some body differs by position.  That refutes function-scoping.

Outside the std-only Rust workspace on purpose -- measurement tooling, same
status as scripts/gt_dump.py and scripts/plot_perf.py.  Degrades cleanly to
"SKIP: toolchain absent" (exit 2) with no compilers/ present.

Usage:
    scripts/candid_c1.py              # C1 at N=120, then the C1b ladder
    scripts/candid_c1.py 40           # C1 only, N=40
"""

import os, struct, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCRATCH = os.path.join(os.environ.get("C2RS_WORK", os.path.join(ROOT, "work")),
                       "w-read-r1", "scratch")

PROBE = """
extern "C" void P(int a, int b, int c, int d) {
    g0 = a + b;
    g1 = c + d;
    g2 = a * 3;
    g3 = b << 2;
    g4 = c - d;
    g5 = a ^ b;
    g6 = c | d;
    g7 = a - c;
    g8 = b + d;
    g9 = a | d;
}
"""

PROBE_PERTURBED = """
extern "C" void P(int a, int b, int c, int d) {
    g0 = a + b;
    g1 = c + d;
    g2 = a * 5;
    g3 = b << 2;
    g4 = c - d;
    g5 = a ^ b;
    g6 = c | d;
    g7 = a - c;
    g8 = b + d;
    g9 = a | d;
}
"""

HEAD = "extern int g0,g1,g2,g3,g4,g5,g6,g7,g8,g9;\n"


def filler(i):
    # each filler is written to force a healthy number of global-register
    # candidates: many simultaneously-live values across a loop.
    return """
extern "C" int F%d(int a, int b, int c, int d, int e) {
    int v0=a+1, v1=b+2, v2=c+3, v3=d+4, v4=e+5;
    int v5=a^b, v6=b^c, v7=c^d, v8=d^e, v9=e^a;
    int s=0;
    for (int i=0;i<e;i++) {
        s += v0+v1+v2+v3+v4+v5+v6+v7+v8+v9+i;
        v0+=i; v1-=i; v2^=i; v3+=v0; v4+=v1; v5+=v2; v6+=v3; v7+=v4; v8+=v5; v9+=v6;
    }
    return s+v0+v1+v2+v3+v4+v5+v6+v7+v8+v9;
}
""" % i


def compile_obj(name, src):
    os.makedirs(SCRATCH, exist_ok=True)
    p = os.path.join(SCRATCH, name + ".cpp")
    with open(p, "w") as f:
        f.write(src)
    obj = os.path.join(SCRATCH, name + ".obj")
    env = dict(os.environ)
    env.setdefault("C2RS_WIBO", os.path.join(ROOT, "..", "wibo", "build", "release", "wibo"))
    env["GT_OUT"] = obj
    r = subprocess.run([os.path.join(ROOT, "scripts", "gt_capture.sh"), p, "/O1", "/GS-", "/c"],
                       capture_output=True, text=True, env=env)
    out = r.stdout.strip()
    if not out or not os.path.exists(obj):
        sys.stderr.write("compile failed for %s (rc=%d)\n%s\n" % (name, r.returncode, r.stderr[-2000:]))
        return None
    return obj


def fn_bytes(obj, want):
    """Return the raw bytes of function `want` from a COFF obj, by symbol
    offset, bounded by the next function symbol in the same section."""
    d = open(obj, "rb").read()
    nsec = struct.unpack_from("<H", d, 2)[0]
    symptr, nsym = struct.unpack_from("<II", d, 8)
    secs = []
    for i in range(nsec):
        o = 20 + i * 40
        nm = d[o:o + 8]
        rawsize, rawptr = struct.unpack_from("<II", d, o + 16)
        secs.append((nm, rawsize, rawptr))
    strtab = symptr + nsym * 18
    syms = []
    i = 0
    while i < nsym:
        o = symptr + i * 18
        raw = d[o:o + 8]
        val, secnum, typ, cls, naux = struct.unpack_from("<IhHBB", d, o + 8)
        if raw[:4] == b"\x00\x00\x00\x00":
            off = struct.unpack_from("<I", raw, 4)[0]
            e = d.index(b"\x00", strtab + off)
            nm = d[strtab + off:e].decode("latin1")
        else:
            nm = raw.rstrip(b"\x00").decode("latin1")
        syms.append((nm, val, secnum, typ, cls))
        i += 1 + naux
    hit = [s for s in syms if s[0] == want or s[0] == "_" + want]
    if not hit:
        return None, "symbol %s absent (have %s)" % (want, [s[0] for s in syms][:40])
    nm, val, secnum, typ, cls = hit[0]
    same = sorted({s[1] for s in syms if s[2] == secnum and s[3] == 0x20 and s[1] > val})
    _, rawsize, rawptr = secs[secnum - 1]
    end = same[0] if same else rawsize
    return d[rawptr + val:rawptr + end], None


def main():
    nfill = int(sys.argv[1]) if len(sys.argv) > 1 else 120
    solo = compile_obj("solo", HEAD + PROBE)
    after = compile_obj("after_%d" % nfill,
                        HEAD + "".join(filler(i) for i in range(nfill)) + PROBE)
    pert = compile_obj("pert", HEAD + PROBE_PERTURBED)
    if not (solo and after and pert):
        print("SKIP: toolchain absent or compile failed")
        return 2
    bs, e1 = fn_bytes(solo, "P")
    ba, e2 = fn_bytes(after, "P")
    bp, e3 = fn_bytes(pert, "P")
    for e in (e1, e2, e3):
        if e:
            print("INSTRUMENT FAILURE:", e)
            return 3
    print("fillers            : %d" % nfill)
    print("P bytes solo       : %d" % len(bs))
    print("P bytes after      : %d" % len(ba))
    print("P bytes perturbed  : %d" % len(bp))
    print("solo[0:32]         : %s" % bs[:32].hex())
    print("after[0:32]        : %s" % ba[:32].hex())
    pos_ok = (bp != bs)
    print("C1-pos (must be DIFFERENT): %s" % ("DIFFERENT -> instrument LIVE" if pos_ok else "IDENTICAL -> INSTRUMENT DEAD"))
    if not pos_ok:
        print("RESULT: DISCARDED — positive control did not go red")
        return 4
    same = (bs == ba)
    print("C1 (solo vs after) : %s" % ("IDENTICAL" if same else "DIFFERENT"))
    print("RESULT: %s" % ("GREEN — consistent with function-scoped" if same
                          else "RED — preceding functions reach P; function-scoping refuted"))
    return 0 if same else 1




def ladder(n):
    """C1b: every Fi has a character-identical body; only the name differs."""
    src = HEAD + "".join(filler(i) for i in range(n)) + PROBE
    obj = compile_obj("ladder_%d" % n, src)
    if not obj:
        return None
    b0, e = fn_bytes(obj, "F0")
    if e:
        print("INSTRUMENT FAILURE:", e)
        return None
    diffs = []
    for i in range(1, n):
        bi, e = fn_bytes(obj, "F%d" % i)
        if e:
            print("INSTRUMENT FAILURE:", e)
            return None
        if bi != b0:
            diffs.append(i)
    bp, _ = fn_bytes(obj, "P")
    solo = compile_obj("solo", HEAD + PROBE)
    bs, _ = fn_bytes(solo, "P")
    return len(b0), diffs, (bp == bs)


if __name__ == "__main__":
    rc = main()
    if rc != 0 or len(sys.argv) > 1:
        sys.exit(rc)
    print()
    print("-- C1b ladder (identical filler bodies at different TU positions)")
    worst = 0
    for n in (2, 40, 120, 400):
        r = ladder(n)
        if r is None:
            print("N=%-4d SKIP" % n)
            continue
        sz, diffs, psame = r
        print("N=%-4d  F0 size=%-4d  differing from F0: %-4d %s  P==solo: %s"
              % (n, sz, len(diffs), ("first=%s" % diffs[:5]) if diffs else "", psame))
        if diffs or not psame:
            worst = 1
    print("RESULT: %s" % ("GREEN -- no body depends on its TU position" if not worst
                          else "RED -- position reaches the emitted bytes"))
    sys.exit(worst)
