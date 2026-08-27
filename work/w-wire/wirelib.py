#!/usr/bin/env python3
"""wirelib.py — lane w-wire.

Compile a generated probe TU through the REAL cl.exe/c2.dll under wibo, with
/FAsc, at an ARBITRARY optimization mode, and parse the listing into one
instruction sequence per probe function.

Measurement only. Nothing here is consulted by `crates/`; the oracle is real c2.

Why this is not `work/w-alloc/alloc_lib.py` verbatim: every prior lane's grid
was compiled at the WORKLOAD's flags, which are `/O1 /Oi /EHsc`. The fixture
gate runs `/Ox`. This lane ships an EMITTER, so it has to know whether ALLOC
and ORDER hold at `/Ox` too — and no lane has ever asked.

`REPO` deliberately resolves to the MAIN repo, not the worktree: the workload
manifest is gitignored and only exists there. `C2RS_WORKLOAD` overrides it.
"""
import os
import re
import subprocess
import sys

WORK = os.path.dirname(os.path.abspath(__file__))
# The worktree is `<repo>/.claude/worktrees/<lane>`; the workload manifest is
# gitignored, so it lives in the main repo only.
REPO = os.environ.get("C2RS_REPO") or "<repo>"
WORKLOAD = os.environ.get("C2RS_WORKLOAD") or os.path.join(REPO, "work", "dc3-workload")
WIBO = os.environ.get("C2RS_WIBO") or "<home>/code/milohax/wibo/build/release/wibo"
CL = os.path.join(
    os.environ.get("C2RS_COMPILERS") or os.path.join(REPO, "compilers"),
    "X360", "16.00.11886.00", "cl.exe",
)

# The workload's own profile, read from the manifest rather than transcribed.
BASE = open(os.path.join(WORKLOAD, "flags.txt")).read().split()
BASE = [f for f in BASE if not f.startswith("/I")]


def flags_for(mode):
    """The workload's flags with the optimization word swapped.

    A positive check rather than a string edit that silently matches nothing:
    `/O1` must be PRESENT in the manifest or this raises.
    """
    if "/O1" not in BASE:
        raise SystemExit("FAIL: manifest flags carry no /O1: %r" % (BASE,))
    if mode == "O1":
        return list(BASE)
    if mode == "Ox":
        return ["/Ox" if f == "/O1" else f for f in BASE]
    raise SystemExit("FAIL: unknown mode %r" % mode)


def zpath(p):
    return "z:" + p.replace("/", "\\")


def compile_cod(src_path, cod_path, obj_path, mode="O1", extra=()):
    """Run the real compiler at `mode`. Returns the listing text."""
    if not os.path.exists(CL):
        raise SystemExit("SKIP: toolchain absent (cl.exe at %s)" % CL)
    if not os.access(WIBO, os.X_OK):
        raise SystemExit("SKIP: toolchain absent (wibo at %s)" % WIBO)
    for p in (cod_path, obj_path):
        if os.path.exists(p):
            os.remove(p)
    env = dict(os.environ, WIBO_FS_CACHE="1", TMP=os.path.dirname(obj_path),
               TEMP=os.path.dirname(obj_path))
    cmd = ([WIBO, CL] + flags_for(mode) + list(extra) +
           ["/FAsc", "/Fa" + zpath(cod_path), "/Fo" + zpath(obj_path),
            zpath(src_path)])
    r = subprocess.run(cmd, capture_output=True, env=env, cwd=REPO)
    if not os.path.exists(cod_path):
        sys.stderr.write(r.stdout.decode("utf8", "replace"))
        sys.stderr.write(r.stderr.decode("utf8", "replace"))
        raise SystemExit("FAIL: no listing at %s" % cod_path)
    txt = open(cod_path, encoding="utf8", errors="replace").read()
    if txt.count("PROC NEAR") == 0:
        raise SystemExit("FAIL: listing has 0 PROC NEAR (compile produced nothing)")
    return txt


# `, COMDAT` is OPTIONAL: `/O1` implies function-level linking on this
# compiler and `/Ox` does not, so the same probe TU comes back COMDAT at one
# mode and packed at the other. Requiring the suffix made every `/Ox` listing
# parse to zero functions — which the `0 PROC` check caught, and which is the
# first difference between the two modes this lane measured.
PROC_RE = re.compile(r"^(\S+)\s+PROC NEAR\s+;\s*([^,\n]+?)(?:,\s*COMDAT)?\s*$")
INSN_RE = re.compile(r"^  ([0-9a-f]{5})\t([0-9a-f]{8})\t (\S+)\s*(.*?)\s*$")


def parse_cod(txt):
    """listing -> {plain_name: [(word, mnemonic, operands), ...]}"""
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
        if cur is None:
            continue
        m = INSN_RE.match(line)
        if m:
            out[cur].append((m.group(2), m.group(3), m.group(4)))
    return out


STORE_MN = {"stb", "sth", "stw", "std", "stfs", "stfd"}
LOADIMM_MN = {"li", "lis", "ori", "addi", "addis"}
OFF_RE = re.compile(r"^r(\d+),\s*(-?\d+)\(r(\d+)\)")
DST_RE = re.compile(r"^r(\d+)")


def seq(insns):
    """Render one function as a token sequence: `P<reg>=<k>` / `S<off>@<reg>`.

    `blr` is dropped. Anything unrecognised is rendered verbatim so a shape this
    lane did not anticipate shows up as noise rather than as a silent pass.
    """
    toks = []
    for _w, mn, ops in insns:
        if mn == "blr":
            continue
        if mn in STORE_MN:
            m = OFF_RE.match(ops)
            if m:
                toks.append("S%s@r%s" % (m.group(2), m.group(1)))
            else:
                toks.append("S?%s" % ops)
        elif mn in LOADIMM_MN:
            m = DST_RE.match(ops)
            toks.append("P%s:r%s" % (mn, m.group(1) if m else "?"))
        else:
            toks.append("%s %s" % (mn, ops))
    return toks
