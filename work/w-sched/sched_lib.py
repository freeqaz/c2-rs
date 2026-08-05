#!/usr/bin/env python3
"""sched_lib.py — lane w-sched.

Compile a generated probe TU through the REAL cl.exe/c2.dll under wibo at the
WORKLOAD's own flags, with /FAsc, and parse the listing into one instruction
sequence per probe function.

Measurement only. Nothing here is consulted by `crates/`; the oracle is real c2.

The unit of observation is the FULL EMITTED PERMUTATION of a probe function,
not a gap statistic. w-pair measured "gap"; a gap is a summary of the
permutation and three of its rules died on cells the permutation separates.
"""
import os
import re
import subprocess
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
WIBO = os.environ.get("C2RS_WIBO") or "/home/free/code/milohax/wibo/build/release/wibo"
CL = os.path.join(
    os.environ.get("C2RS_COMPILERS") or os.path.join(REPO, "compilers"),
    "X360", "16.00.11886.00", "cl.exe",
)
# The workload's own profile, read from the manifest rather than transcribed.
FLAGS = open(os.path.join(REPO, "work", "dc3-workload", "flags.txt")).read().split()
FLAGS = [f for f in FLAGS if not f.startswith("/I")]


def zpath(p):
    return "z:" + p.replace("/", "\\")


def compile_cod(src_path, cod_path, obj_path, extra=()):
    """Run the real compiler. Returns the listing text.

    Positive check with a printed count: raises if the listing is absent or
    contains no PROC. `gt_capture.sh` exits 0 on SKIP, which is exactly the
    absence-read-as-success shape this project has paid for 16 times.
    """
    if not os.path.exists(CL):
        raise SystemExit("SKIP: toolchain absent (cl.exe at %s)" % CL)
    if not os.access(WIBO, os.X_OK):
        raise SystemExit("SKIP: toolchain absent (wibo at %s)" % WIBO)
    for p in (cod_path, obj_path):
        if os.path.exists(p):
            os.remove(p)
    env = dict(os.environ, WIBO_FS_CACHE="1", TMP=os.path.dirname(obj_path),
               TEMP=os.path.dirname(obj_path))
    cmd = ([WIBO, CL] + FLAGS + list(extra) +
           ["/FAsc", "/Fa" + zpath(cod_path), "/Fo" + zpath(obj_path),
            zpath(src_path)])
    r = subprocess.run(cmd, capture_output=True, env=env, cwd=REPO)
    if not os.path.exists(cod_path):
        sys.stderr.write(r.stdout.decode("utf8", "replace"))
        sys.stderr.write(r.stderr.decode("utf8", "replace"))
        raise SystemExit("FAIL: no listing at %s" % cod_path)
    txt = open(cod_path, encoding="utf8", errors="replace").read()
    n = txt.count("PROC NEAR")
    if n == 0:
        raise SystemExit("FAIL: listing has 0 PROC NEAR (compile produced nothing)")
    return txt


PROC_RE = re.compile(r"^(\S+)\s+PROC NEAR\s+;\s*([^,]+), COMDAT")
INSN_RE = re.compile(r"^  ([0-9a-f]{5})\t([0-9a-f]{8})\t (\S+)\s*(.*?)\s*$")


def parse_cod(txt):
    """listing -> {plain_name: [(idx, mnemonic, operands), ...]}"""
    out = {}
    cur = None
    for line in txt.splitlines():
        m = PROC_RE.match(line)
        if m:
            cur = m.group(2).strip()
            out[cur] = []
            continue
        if line.endswith("ENDP"):
            cur = None
            continue
        if cur is not None:
            m = INSN_RE.match(line)
            if m:
                # the listing appends `\t\t; <comment>` to some operand fields
                ops = m.group(4).split("\t")[0].split(";")[0].strip()
                out[cur].append((len(out[cur]), m.group(3), ops))
    return out


STORE_RE = re.compile(r"^r(\d+),(-?[0-9A-Fa-f]+[hH]?)\((r\d+)\)$")
LOAD_RE = re.compile(r"^r(\d+),(-?[0-9A-Fa-f]+[hH]?)\((r\d+)\)$")


def masm_int(s):
    """MASM literal: `0Ch` is hex, `12` is decimal, `-8` decimal."""
    s = s.strip()
    neg = s.startswith("-")
    if neg:
        s = s[1:]
    v = int(s[:-1], 16) if s[-1] in "hH" else int(s, 10)
    return -v if neg else v


def classify(seq):
    """Annotate one function's instruction sequence.

    Returns a list of dicts, one per instruction, with a coarse role:
      store  : st{b,h,w,d} rS, off(rB)
      li     : li rD, k
      addi   : addi rD, rA, k
      other  : anything else (mr, bl, mflr, blr, ...)
    """
    ann = []
    for idx, mn, ops in seq:
        d = {"i": idx, "mn": mn, "ops": ops, "role": "other"}
        if mn in ("stw", "stb", "sth", "std"):
            m = STORE_RE.match(ops)
            if m:
                d.update(role="store", src="r" + m.group(1),
                         off=masm_int(m.group(2)), base=m.group(3))
        elif mn == "li":
            p = ops.split(",")
            d.update(role="li", dst=p[0], imm=masm_int(p[1]))
        elif mn == "addi":
            p = ops.split(",")
            d.update(role="addi", dst=p[0], base=p[1], imm=masm_int(p[2]))
        elif mn in ("lwz", "lbz", "lhz", "ld"):
            m = LOAD_RE.match(ops)
            if m:
                d.update(role="load", dst="r" + m.group(1),
                         off=masm_int(m.group(2)), base=m.group(3))
        elif mn == "mr":
            p = ops.split(",")
            d.update(role="mr", dst=p[0], src=p[1])
        elif re.match(r"^r\d+,", ops):
            # any other reg-writing op (mulli, rlwinm/slwi, add, nor, ...)
            d.update(role="alu", dst=ops.split(",")[0])
        ann.append(d)
    return ann
