#!/usr/bin/env python3
"""mutate.py — KA-B / KA-C: the emit SEED bit, tested against the SOLE JUDGE.

Captures a real workload TU's IL with the workload flags (the harness's own
recipe: `strace -e inject=unlink…:retval=0` keeps the `_CL_*` bundle alive while
`/Bd` echoes the c2 argv), decodes it with `record.py`, then flips bit `0x20`
at the byte offset **my decoder reports** and replays the mutated bundle through
the real `c2.dll` under wibo via `c2host`.  The verdict is the obj, not a model.

    KA-B  clear 0x20 on a seeded ROOT leaf  -> its COMDAT must disappear
    KA-C  set   0x20 on an unseeded record  -> its COMDAT must appear

    usage: mutate.py <src.cpp> [n_clear] [n_set]
"""
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(REPO, "work", "emitpred", "pipeline"))
import coff    # noqa: E402
import record  # noqa: E402
import scan as scanmod  # noqa: E402

DC3 = os.environ.get("C2RS_DC3", os.path.join(REPO, "..", "dc3-decomp"))
CL = os.path.join(REPO, "compilers", "X360", "16.00.11886.00", "cl.exe")
C2 = os.path.join(REPO, "compilers", "X360", "16.00.11886.00", "c2.dll")
WIBO = os.environ.get("C2RS_WIBO", os.path.join(REPO, "..", "wibo", "build", "wibo"))
C2HOST = os.path.join(REPO, "target", "c2host", "c2host.exe")
WORK = os.path.join(HERE, "mut")


def zpath(p):
    return "Z:" + os.path.abspath(p).replace("/", "\\")


def capture(src):
    """-> (bundle_dir, base_name, c2_argv, baseline_obj_bytes)"""
    shutil.rmtree(WORK, ignore_errors=True)
    os.makedirs(WORK)
    obj = os.path.join(WORK, "out.obj")
    flags = open(os.path.join(REPO, "work", "dc3-workload", "flags.txt")).read().split()
    cmd = ["strace", "-f", "-e", "trace=unlink,unlinkat",
           "-e", "inject=unlink,unlinkat:retval=0", "-o", "/dev/null",
           WIBO, CL, "/Bd"] + flags + ["/Fo" + zpath(obj), src]
    env = dict(os.environ, TMP=WORK, TEMP=WORK, WIBO_FS_CACHE="1")
    r = subprocess.run(cmd, cwd=DC3, env=env, capture_output=True, text=True)
    blob = r.stdout + "\n" + r.stderr
    # Identical rule to c2-reference::parse_c2_argv: the first line mentioning
    # both `c2.dll` and `-il`, everything AFTER the first "c2.dll".
    line = next((ln for ln in blob.splitlines()
                 if "c2.dll" in ln.lower() and "-il" in ln.lower()), None)
    if line is None:
        raise SystemExit("no c2 argv echo in cl output:\n" + blob[-3000:])
    t = line.strip().lstrip("`").rstrip("'")
    argv = t[t.lower().index("c2.dll") + len("c2.dll"):].split()
    base = None
    for f in os.listdir(WORK):
        if f.startswith("_CL_") and f.endswith("ex"):
            base = f[:-2]
    if base is None or not os.path.exists(obj):
        raise SystemExit("no surviving bundle / obj in %s: %s" % (WORK, os.listdir(WORK)))
    return WORK, base, argv, open(obj, "rb").read()


def replay(bundle_dir, base, argv, out_obj):
    z_il = zpath(os.path.join(bundle_dir, base.rstrip(".")))
    out = []
    i = 0
    while i < len(argv):
        t = argv[i]
        if t == "-il":
            out += ["-il", z_il]
            i += 2
            continue
        if t.startswith("-Fo"):
            out.append("-Fo" + zpath(out_obj))
            i += 1
            continue
        out.append(t)
        i += 1
    os.makedirs(os.path.dirname(os.path.abspath(out_obj)), exist_ok=True)
    if os.path.exists(out_obj):
        os.remove(out_obj)
    r = subprocess.run([WIBO, C2HOST, C2, C2] + out, cwd=bundle_dir,
                       env=dict(os.environ, WIBO_FS_CACHE="1"),
                       capture_output=True, text=True)
    if not os.path.exists(out_obj):
        return None, r.stdout + r.stderr
    return open(out_obj, "rb").read(), ""


def leaders(objb):
    return set(n for n, _ in (coff.text_comdat_entries(objb) or []))


def main():
    src = sys.argv[1]
    n_clear = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    n_set = int(sys.argv[3]) if len(sys.argv) > 3 else 3

    bdir, base, argv, base_obj = capture(src)
    print("bundle", base, "argv", " ".join(argv)[:160])
    glp = os.path.join(bdir, base + "gl")
    exp = os.path.join(bdir, base + "ex")
    glb = open(glp, "rb").read()
    exb = open(exp, "rb").read()
    recs, st = record.scan(glb, exb)
    print("decode:", st)

    # the replayed baseline must reproduce the pipeline obj before anything else
    rb, err = replay(bdir, base, argv, os.path.join(bdir, "b", "out.obj"))
    if rb is None:
        raise SystemExit("baseline replay produced no obj: " + err[-2000:])
    b0 = bytearray(base_obj); b0[4:8] = b"\0\0\0\0"
    r0 = bytearray(rb);       r0[4:8] = b"\0\0\0\0"
    print("BASELINE replay == pipeline obj (TimeDateStamp zeroed):", bytes(b0) == bytes(r0))
    L0 = leaders(rb)
    print("baseline leaders:", len(L0))

    U = set(recs)
    Nf = {v["ex"]: k for k, v in recs.items()}
    ed = scanmod.edges26(glb, exb, Nf, U)
    called = set()
    for a, tg in ed.items():
        if a in L0:
            called |= tg
    seeds = [k for k, v in recs.items() if v["seed"]]
    # KA-B candidates: seeded, nothing emitted calls them, and they call nothing
    cb = sorted(k for k in seeds if k not in called and not ed.get(k))
    # KA-C candidates: unseeded, not emitted, calling nothing
    cc = sorted(k for k in U if k not in seeds and k not in L0 and not ed.get(k))

    def run(name, bit_set):
        r = recs[name]
        mg = bytearray(glb)
        if bit_set:
            mg[r["fpos"]] |= 0x20
        else:
            mg[r["fpos"]] &= ~0x20
        assert bytes(mg) != glb
        open(glp, "wb").write(bytes(mg))
        try:
            ob, err = replay(bdir, base, argv, os.path.join(bdir, "m", "out.obj"))
        finally:
            open(glp, "wb").write(glb)
        if ob is None:
            return "NO-OBJ", set(), set()
        L = leaders(ob)
        return "ok", L0 - L, L - L0

    print("\nKA-B  clear 0x20 on a seeded root leaf (%d candidates)" % len(cb))
    hits = 0
    for nm in cb[:n_clear]:
        st_, lost, gained = run(nm, False)
        good = (lost == {nm} and not gained)
        hits += good
        print("  %-5s lost=%d gained=%d exact=%s  %s"
              % (st_, len(lost), len(gained), good, nm[:80]))
        if lost and lost != {nm}:
            print("        lost:", sorted(lost)[:4])
    print("  KA-B %d/%d" % (hits, min(n_clear, len(cb))))

    print("\nKA-C  set 0x20 on an unseeded, unemitted record (%d candidates)" % len(cc))
    hits2 = 0
    for nm in cc[:n_set]:
        st_, lost, gained = run(nm, True)
        good = (gained == {nm} and not lost)
        hits2 += good
        print("  %-5s lost=%d gained=%d exact=%s  %s"
              % (st_, len(lost), len(gained), good, nm[:80]))
        if gained and gained != {nm}:
            print("        gained:", sorted(gained)[:4])
    print("  KA-C %d/%d" % (hits2, min(n_set, len(cc))))


if __name__ == "__main__":
    main()
