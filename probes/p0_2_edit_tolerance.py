#!/usr/bin/env python3
"""P0.2 edit-tolerance probe — does standalone c2 accept MODIFIED IL bundles?

The remaining half of il-witness Gate G0 (P0.1 proved *verbatim* replay is
byte-exact; this asks whether c2 consumes *edited* IL). Capture one bundle +
the exact c2 argv (`/Bd` + strace-inject-unlink keeps the temp bundle), then
run a battery of mutations, each in a fresh copy of the bundle, replaying
through the P0.1 `c2host` stub. Every replay writes to a FIXED `-Fo` path so
the only path MSVC bakes into the obj (`.debug$S` S_OBJNAME) is constant —
any obj difference is then the true effect of the mutation.

Classification per mutation: COMPILES(rc) / EMPTY-OBJ(rc) / ERROR(rc) /
CRASH(sig) / TIMEOUT. For COMPILES, decode `.text` + the function symbol to
show the *semantic* change (not just "bytes differ").

Paths are env-driven (same `C2RS_*` convention as crate `c2-reference`), with
relative-to-repo defaults. Run from anywhere:
    python3 probes/p0_2_edit_tolerance.py [fixture-stem]   # default mvp_add3

Requires: wibo (release), the DC3 X360 toolchain (cl.exe + c2.dll), strace,
and the built c2host stub. All degrade to a clear error if absent.
"""
import os, re, shutil, subprocess, glob, struct, sys, tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def env(key, default):
    return os.environ.get(key, default)

WIBO   = env("C2RS_WIBO",   os.path.join(REPO, "../wibo/build/release/wibo"))
DC3    = env("C2RS_DC3_ROOT", os.path.join(REPO, "../dc3-decomp"))
TC     = env("C2RS_TC", os.path.join(DC3, "build/compilers/X360/16.00.11886.00"))
CL     = env("C2RS_CL_EXE", os.path.join(TC, "cl.exe"))
C2     = env("C2RS_C2_DLL", os.path.join(TC, "c2.dll"))
C2HOST = env("C2RS_C2HOST", os.path.join(REPO, "target/c2host/c2host.exe"))
FIX    = os.path.join(REPO, "fixtures/cpp")
ROOT   = env("C2RS_P02_WORK", os.path.join(tempfile.gettempdir(), "c2rs-p02run"))
FIXED_OBJ = os.path.join(ROOT, "fixed_out.obj")

def zpath(h): return "Z:" + os.path.abspath(h).replace("/", "\\")

def run(argv, env_extra=None, cwd=None, timeout=90):
    e = dict(os.environ); e["WIBO_FS_CACHE"] = "1"
    if env_extra: e.update(env_extra)
    return subprocess.run(argv, env=e, cwd=cwd, capture_output=True, text=True, timeout=timeout)

def norm(b):
    b = bytearray(b)
    if len(b) >= 8: b[4:8] = b"\0\0\0\0"   # zero the COFF TimeDateStamp
    return bytes(b)

def capture(name, tries=4):
    """Compile `name`.cpp with /Bd, keep the IL bundle, return
    (bundle_dir, base_hash, c2_argv_template, reference_obj_bytes)."""
    last = ""
    for attempt in range(tries):
        d = os.path.join(ROOT, f"cap_{name}"); shutil.rmtree(d, ignore_errors=True); os.makedirs(d)
        shutil.copy(os.path.join(FIX, f"{name}.cpp"), os.path.join(d, f"{name}.cpp"))
        obj = os.path.join(d, "out.obj")
        strace = ["strace","-f","-e","trace=unlink,unlinkat",
                  "-e","inject=unlink,unlinkat:retval=0","-o","/dev/null"]
        cl_argv = strace + [WIBO, CL, "/Bd","/Ox","/GS-","/c",
                            f"/Fo{zpath(obj)}", zpath(os.path.join(d, f'{name}.cpp'))]
        try:
            r = run(cl_argv, env_extra={"TMP":d,"TEMP":d})
        except subprocess.TimeoutExpired:
            last = "timeout"; continue
        echo = r.stdout + r.stderr
        m = [ln for ln in echo.splitlines() if "c2.dll" in ln.lower() and "-il" in ln]
        exs = glob.glob(os.path.join(d, "_CL_*ex"))
        if m and os.path.exists(obj) and exs:
            c2line = m[0].strip().lstrip("`").rstrip("'")
            ref = open(obj,"rb").read()
            hsh = os.path.basename(exs[0][:-2])
            args = c2line.split("c2.dll",1)[1].strip().split(" ")
            return d, hsh, args, ref
        last = f"attempt {attempt}: c2_echo={bool(m)} obj={os.path.exists(obj)} ex={bool(exs)}"
    raise RuntimeError(f"capture failed after {tries} tries: {last}")

def build_c2_args(args, base_win, obj_win):
    out, i = [], 0
    while i < len(args):
        t = args[i]
        if t == "-il": out += ["-il", base_win]; i += 2; continue
        if t.startswith("-Fo"): out += ["-Fo"+obj_win]; i += 1; continue
        if t == "": i += 1; continue
        out.append(t); i += 1
    return out

def replay(bundle_dir, hsh, args, tag):
    w = os.path.join(ROOT, f"rt_{tag}"); shutil.rmtree(w, ignore_errors=True); os.makedirs(w)
    for f in glob.glob(os.path.join(bundle_dir, f"{hsh}*")):
        shutil.copy(f, os.path.join(w, os.path.basename(f)))
    if os.path.exists(FIXED_OBJ): os.remove(FIXED_OBJ)
    c2args = build_c2_args(args, zpath(os.path.join(w, hsh)), zpath(FIXED_OBJ))
    try:
        rr = run([WIBO, C2HOST, C2, C2] + c2args, cwd=w, timeout=60)
    except subprocess.TimeoutExpired:
        return "TIMEOUT", None, "timeout"
    tail = (rr.stderr or "").strip().splitlines()
    tail = tail[-1] if tail else ""
    rc = rr.returncode
    if os.path.exists(FIXED_OBJ):
        data = open(FIXED_OBJ,"rb").read()
        if len(data) == 0: return f"EMPTY-OBJ(rc={rc})", None, tail
        return f"COMPILES(rc={rc})", data, tail
    if rc is not None and rc < 0: return f"CRASH(sig{-rc})", None, tail
    return f"ERROR(rc={rc})", None, tail

def mutate_dir(bundle_dir, hsh, tag, fn):
    w = os.path.join(ROOT, f"mut_{tag}"); shutil.rmtree(w, ignore_errors=True); os.makedirs(w)
    paths = {}
    for f in glob.glob(os.path.join(bundle_dir, f"{hsh}*")):
        suf = os.path.basename(f)[len(hsh):]
        dst = os.path.join(w, f"{hsh}{suf}")
        shutil.copy(f, dst); paths[suf] = dst
    fn(paths)
    return w

def rd(p): return bytearray(open(p,"rb").read())
def wr(p,b): open(p,"wb").write(bytes(b))

def obj_facts(data):
    """Extract .text hex + function symbol name from a PPC COFF obj."""
    try:
        nsec = struct.unpack_from("<H", data, 2)[0]
        ptrsym, nsym = struct.unpack_from("<II", data, 8)
        text = None
        for i in range(nsec):
            off = 20 + 40*i
            if data[off:off+8].split(b"\0")[0] == b".text":
                size, praw = struct.unpack_from("<II", data, off+16)
                text = data[praw:praw+size]
        strtab = ptrsym + 18*nsym
        fname = None
        for s in range(nsym):
            off = ptrsym + 18*s
            _val, _sec, typ, _cls, _naux = struct.unpack_from("<IhHBB", data, off+8)
            if typ == 0x20:
                nm = data[off:off+8]
                if nm[:4] == b"\0\0\0\0":
                    stroff = struct.unpack_from("<I", nm, 4)[0]
                    end = data.index(b"\0", strtab+stroff)
                    fname = data[strtab+stroff:end].decode("latin1")
                else:
                    fname = nm.split(b"\0")[0].decode("latin1")
        return (text.hex() if text else None), fname
    except Exception as e:
        return f"<parse err {e}>", None

def main():
    for tool, path in [("wibo",WIBO),("cl.exe",CL),("c2.dll",C2),("c2host",C2HOST)]:
        if not os.path.exists(path):
            print(f"MISSING {tool}: {path}\n(set C2RS_* env or build c2host first)"); return 2
    os.makedirs(ROOT, exist_ok=True)
    for sub in glob.glob(os.path.join(ROOT, "*")):
        shutil.rmtree(sub, ignore_errors=True) if os.path.isdir(sub) else os.remove(sub)
    name = sys.argv[1] if len(sys.argv) > 1 else "mvp_add3"
    print(f"=== P0.2 edit-tolerance probe on {name} ===\n")
    bd, hsh, args, ref = capture(name)
    print(f"captured {hsh}, ref obj = {len(ref)}B\n")

    k, base_obj, err = replay(bd, hsh, args, "baseline")
    assert base_obj is not None, f"baseline replay failed: {k} {err}"
    BASE = norm(base_obj)
    btext, bname = obj_facts(base_obj)
    print(f"[baseline verbatim] {k}  {len(base_obj)}B  .text={btext}  fn={bname}\n")

    def report(tag, desc, k, obj):
        change = ""
        if obj is not None:
            n = norm(obj)
            if n == BASE:
                change = "obj==baseline (UNCHANGED)"
            else:
                text, fname = obj_facts(obj)
                deltas = []
                if text != btext: deltas.append(f".text {btext}->{text}")
                if fname != bname: deltas.append(f"fn {bname}->{fname}")
                if not deltas:
                    fo = next((i for i in range(min(len(n),len(BASE))) if n[i]!=BASE[i]), None)
                    deltas.append(f"first@{hex(fo) if fo is not None else 'len'}")
                change = "CHANGED: " + "; ".join(deltas)
        print(f"[{tag:11}] {desc}\n    -> {k}   {change}")

    # 1. Which of the 5 files does c2 require?
    for suf in ["ex","gl","sy","in","db"]:
        w = mutate_dir(bd, hsh, f"drop_{suf}", lambda p,s=suf: os.remove(p[s]))
        k,obj,_ = replay(w, hsh, args, f"drop_{suf}")
        report(f"drop .{suf}", f"remove the .{suf} file", k, obj)

    # 2. Rename the mangled name in .gl (?add3 -> ?zzz3, same length)
    def swap_name(p):
        b = rd(p["gl"]); i = b.find(b"?add3")
        if i>=0: b[i+1:i+4] = b"zzz"
        wr(p["gl"], b)
    k,obj,_ = replay(mutate_dir(bd, hsh, "gl_name", swap_name), hsh, args, "gl_name")
    report("gl name", "rename ?add3 -> ?zzz3 in .gl", k, obj)

    # 3. Edit an operand token in the .ex body (first LOAD e3->e5)
    def flip_ex_token(p):
        b = rd(p["ex"]); i = b.find(bytes([0xB9,0xE3,0x09]))
        if i>=0: b[i+1] = 0xE5
        wr(p["ex"], b)
    k,obj,_ = replay(mutate_dir(bd, hsh, "ex_tok", flip_ex_token), hsh, args, "ex_tok")
    report("ex token", "first LOAD token e3->e5 (semantic edit)", k, obj)

    # 4. Corrupt an opcode (first ADD 0x02 -> 0x2A)
    def flip_ex_op(p):
        b = rd(p["ex"]); m = re.search(bytes([0x86,0x41,0x74,0x02]), b)
        if m: b[m.start()+3] = 0x2A
        wr(p["ex"], b)
    k,obj,_ = replay(mutate_dir(bd, hsh, "ex_op", flip_ex_op), hsh, args, "ex_op")
    report("ex opcode", "first ADD 0x02 -> 0x2A (invalid)", k, obj)

    # 5. Truncate .ex (well-formedness)
    k,obj,_ = replay(mutate_dir(bd, hsh, "trunc_ex",
                     lambda p: wr(p["ex"], rd(p["ex"])[:-16])), hsh, args, "trunc_ex")
    report("trunc .ex", "drop last 16 bytes of .ex", k, obj)

    # 6. Bit-flip in the .ex header padding (offset 0x10)
    def flip_hdr(p):
        b = rd(p["ex"]); b[0x10] ^= 0xFF; wr(p["ex"], b)
    k,obj,_ = replay(mutate_dir(bd, hsh, "ex_hdr", flip_hdr), hsh, args, "ex_hdr")
    report("ex header", "XOR 0xFF at .ex offset 0x10", k, obj)

    # 7. Corrupt mid-.sy (cross-file consistency)
    def corrupt_sy(p):
        b = rd(p["sy"])
        if len(b) > 4: b[len(b)//2] ^= 0xFF
        wr(p["sy"], b)
    k,obj,_ = replay(mutate_dir(bd, hsh, "sy_corrupt", corrupt_sy), hsh, args, "sy_corrupt")
    report("sy corrupt", "XOR 0xFF mid-.sy", k, obj)

    # 8. Append garbage to .in
    def pad_in(p):
        wr(p["in"], rd(p["in"]) + b"\xde\xad\xbe\xef"*8)
    k,obj,_ = replay(mutate_dir(bd, hsh, "in_pad", pad_in), hsh, args, "in_pad")
    report("in pad", "append 32 garbage bytes to .in", k, obj)

    print("\n=== done ===")
    return 0

if __name__ == "__main__":
    sys.exit(main())
