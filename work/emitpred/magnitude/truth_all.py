#!/usr/bin/env python3
"""truth_all.py — ground truth E(t) for workload TUs: the COMDAT leader symbol
set of every *code* section of the real obj.

Section selection is by the **IMAGE_SCN_CNT_CODE (0x20)** characteristic, never
by a `.text` name prefix (this project has twice been burned by name-as-proxy).
The harness's own `.text`-prefix rule is computed alongside and any
disagreement is reported per TU rather than silently reconciled.

Runs the real `cl.exe` (front end + c2) under wibo with the unmodified workload
flag line.  THIS SCRIPT RUNS c2 — it must never be pointed at a quarantined TU.

    usage: truth_all.py <outroot> <tulist.txt> [jobs]
    writes <outroot>/<slug>.txt   (one symbol per line, sorted; code+COMDAT)
           <outroot>/<slug>.meta  (json: counts, name-rule disagreement)
"""
import json
import os
import shutil
import struct
import subprocess
import sys
import concurrent.futures as cf

WIBO = os.environ.get("C2RS_WIBO", "<home>/code/milohax/wibo/build/release/wibo")
CL = os.environ.get(
    "C2RS_CL_EXE",
    "<repo>/compilers/X360/16.00.11886.00/cl.exe",
)
DC3 = os.environ.get("C2RS_DC3", "<home>/code/milohax/dc3-decomp")
HERE = os.path.dirname(os.path.abspath(__file__))
FLAGS = open(os.path.join(HERE, "..", "..", "dc3-workload", "flags.txt")).read().split()

COFF_HEADER_LEN = 20
SECTION_HEADER_LEN = 40
SYMBOL_LEN = 18
IMAGE_SCN_CNT_CODE = 0x20
IMAGE_SCN_LNK_COMDAT = 0x1000
IMAGE_SYM_CLASS_STATIC = 3


def comdat_leaders(b, mode):
    """mode='code' -> CNT_CODE|LNK_COMDAT ; mode='name' -> name.startswith('.text')|LNK_COMDAT.
    Otherwise a verbatim port of ObjImage::text_comdat_entries."""
    if len(b) < COFF_HEADER_LEN:
        return None
    nsec = struct.unpack_from("<H", b, 2)[0]
    psym = struct.unpack_from("<I", b, 8)[0]
    nsym = struct.unpack_from("<I", b, 12)[0]
    sec_end = COFF_HEADER_LEN + nsec * SECTION_HEADER_LEN
    sym_end = psym + nsym * SYMBOL_LEN
    if sec_end > len(b) or psym < sec_end or sym_end + 4 > len(b):
        return None
    strtab = b[sym_end:]

    def str_at(i):
        if i >= len(strtab):
            return None
        e = strtab.find(b"\0", i)
        return None if e < 0 else strtab[i:e].decode("utf-8", "replace")

    sel = [False] * nsec
    for i in range(nsec):
        o = COFF_HEADER_LEN + i * SECTION_HEADER_LEN
        raw = b[o:o + 8]
        if raw[0:1] == b"/":
            try:
                idx = int(raw[1:].rstrip(b"\0").strip())
            except ValueError:
                return None
            name = str_at(idx)
            if name is None:
                return None
        else:
            name = raw.rstrip(b"\0").decode("utf-8", "replace")
        chars = struct.unpack_from("<I", b, o + 36)[0]
        hit = (chars & IMAGE_SCN_CNT_CODE) != 0 if mode == "code" else name.startswith(".text")
        sel[i] = hit and (chars & IMAGE_SCN_LNK_COMDAT) != 0
    claimed = [False] * nsec
    out = []
    i = 0
    while i < nsym:
        o = psym + i * SYMBOL_LEN
        naux = b[o + 17]
        secnum = struct.unpack_from("<h", b, o + 12)[0]
        sclass = b[o + 16]
        if 1 <= secnum <= nsec:
            s = secnum - 1
            is_secdef = sclass == IMAGE_SYM_CLASS_STATIC and naux == 1
            if sel[s] and not claimed[s] and not is_secdef:
                if b[o:o + 4] == b"\0\0\0\0":
                    at = struct.unpack_from("<I", b, o + 4)[0]
                    name = str_at(at)
                    if name is None:
                        return None
                else:
                    name = b[o:o + 8].rstrip(b"\0").decode("utf-8", "replace")
                claimed[s] = True
                out.append(name)
        i = i + 1 + naux
        if i > nsym:
            return None
    if any(t and not c for c, t in zip(claimed, sel)):
        return None
    return sorted(set(out))


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def one(src, outroot):
    dst = os.path.join(outroot, slug(src) + ".txt")
    if os.path.exists(dst):
        return (src, "cached")
    tmp = os.path.join(outroot, "_t_" + slug(src))
    shutil.rmtree(tmp, ignore_errors=True)
    os.makedirs(tmp, exist_ok=True)
    obj = os.path.join(tmp, "x.obj")
    argv = [WIBO, CL] + FLAGS + ["/Fo" + obj, src]
    env = dict(os.environ, WIBO_FS_CACHE="1", TMP=tmp, TEMP=tmp)
    try:
        subprocess.run(argv, cwd=DC3, env=env, stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL, timeout=900)
    except subprocess.TimeoutExpired:
        shutil.rmtree(tmp, ignore_errors=True)
        return (src, "TIMEOUT")
    if not os.path.exists(obj):
        shutil.rmtree(tmp, ignore_errors=True)
        return (src, "NOOBJ")
    b = open(obj, "rb").read()
    code = comdat_leaders(b, "code")
    name = comdat_leaders(b, "name")
    shutil.rmtree(tmp, ignore_errors=True)
    if code is None:
        return (src, "COFF-REJECT")
    open(dst, "w").write("\n".join(code) + ("\n" if code else ""))
    json.dump({"src": src, "n_code": len(code),
               "n_name": -1 if name is None else len(name),
               "name_rule_ok": name is not None and set(name) == set(code),
               "code_only": sorted(set(code) - set(name or []))[:20]},
              open(os.path.join(outroot, slug(src) + ".meta"), "w"))
    return (src, "ok")


def main():
    outroot, tulist = sys.argv[1], sys.argv[2]
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 16
    srcs = [l.strip() for l in open(tulist) if l.strip()]
    os.makedirs(outroot, exist_ok=True)
    bad = 0
    with cf.ThreadPoolExecutor(jobs) as ex:
        for i, (src, st) in enumerate(ex.map(lambda s: one(s, outroot), srcs)):
            if st not in ("ok", "cached"):
                bad += 1
                print("%s %s" % (st, src), flush=True)
            if (i + 1) % 100 == 0:
                print("... %d/%d (%d bad)" % (i + 1, len(srcs), bad), flush=True)
    print("DONE %d TUs, %d failures" % (len(srcs), bad), flush=True)


if __name__ == "__main__":
    main()
