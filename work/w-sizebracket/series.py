#!/usr/bin/env python3
"""series.py — the SIZE-axis series: vary one callee's size monotonically and ask
real `c2.dll` whether it inlined the call.

One cell = one (family, n, profile).  For each cell:

  * generate the .cpp,
  * `c2rs capture --keep-il`  -> the `.gl`, from which `glsize.py` reads the
    callee's SIZE field (`[sym+0x50]`, the quantity `0x10b5fc86` tests),
  * `c2rs compile --keep-obj` -> real c2's own obj,
  * read the obj: the callee's own `.text` COMDAT size, and whether the CALLER's
    `.text` COMDAT carries a REL24 naming the callee.

`kept`    the caller's relocations name the callee  -> c2 DECLINED to inline
`inlined` they do not, and the callee's body is not reachable from the caller
`absent`  the callee has no `.text` COMDAT of its own at all

Derived from the logs, never accumulated (`docs/rungs/README.md` probe rule 2):
every cell writes a JSON line and the tables are re-derived from it.
"""

import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
import glsize  # noqa: E402

C2RS = os.path.join(ROOT, "target", "release", "c2rs")


# ---------------------------------------------------------------- COFF reader
def u16(b, o):
    return int.from_bytes(b[o : o + 2], "little")


def u32(b, o):
    return int.from_bytes(b[o : o + 4], "little")


class Obj:
    def __init__(self, data):
        self.d = data
        self.nsec = u16(data, 2)
        self.symptr = u32(data, 8)
        self.nsym = u32(data, 12)
        self.strtab = self.symptr + 18 * self.nsym
        self.sections = []
        for i in range(self.nsec):
            o = 20 + 40 * i
            self.sections.append(
                {
                    "index": i + 1,
                    "name": data[o : o + 8].rstrip(b"\0").decode("latin1"),
                    "raw": u32(data, o + 16),
                    "rawptr": u32(data, o + 20),
                    "relptr": u32(data, o + 24),
                    "nrel": u16(data, o + 32),
                }
            )
        self.syms = []
        i = 0
        while i < self.nsym:
            o = self.symptr + 18 * i
            raw = data[o : o + 8]
            if raw[:4] == b"\0\0\0\0":
                off = u32(data, o + 4)
                e = data.index(b"\0", self.strtab + off)
                name = data[self.strtab + off : e].decode("latin1")
            else:
                name = raw.rstrip(b"\0").decode("latin1")
            naux = data[o + 17]
            self.syms.append(
                {"i": i, "name": name, "sec": u16(data, o + 12), "naux": naux}
            )
            i += 1 + naux

    def comdat_of(self, name):
        """The section index whose EXTERNAL symbol is `name`."""
        for s in self.syms:
            if s["name"] == name and s["sec"] not in (0, 0xFFFF):
                return s["sec"]
        return None

    def sec(self, idx):
        return self.sections[idx - 1]

    def relocation_targets(self, secidx):
        s = self.sec(secidx)
        out = []
        for k in range(s["nrel"]):
            o = s["relptr"] + 10 * k
            symidx = u32(self.d, o + 4)
            for sy in self.syms:
                if sy["i"] == symidx:
                    out.append(sy["name"])
                    break
        return out


# ---------------------------------------------------------------- generators
def body_arith(n):
    """n straight-line arithmetic statements.  ~2 emitted instructions each."""
    L = ["  int s = a;"]
    for i in range(n):
        L.append(f"  s = s * {2 * i + 3} + {i + 1};")
    L.append("  return s;")
    return "\n".join(L)


def body_mix(n):
    """n statements mixing multiply/xor/shift — a wider emitted word per stmt."""
    L = ["  int s = a;"]
    for i in range(n):
        L.append(f"  s = (s * {2 * i + 3} + {i + 1}) ^ (s >> {(i % 7) + 1});")
    L.append("  return s;")
    return "\n".join(L)


def body_loop(n):
    """a loop-bodied callee — WB_INLINE_FINDINGS F9's separate class."""
    L = ["  int s = a;", f"  for (int i = 0; i < {n + 2}; ++i) {{"]
    for i in range(max(1, n // 4 + 1)):
        L.append(f"    s = s * {2 * i + 3} + i;")
    L.append("  }")
    L.append("  return s;")
    return "\n".join(L)



def body_fine(n):
    """A FINE ladder: the `mix` prefix at 6, then n cheap statements.

    `mix` steps SIZE by 12 per statement, which cannot resolve a boundary.  Each
    `s ^= (a >> j);` here is a smaller IL step, and `j` varies so no pair of them
    folds away.
    """
    L = ["  int s = a;"]
    for i in range(6):
        L.append(f"  s = (s * {2 * i + 3} + {i + 1}) ^ (s >> {(i % 7) + 1});")
    for i in range(n):
        L.append(f"  s ^= (a >> {(i % 15) + 1});")
    L.append("  return s;")
    return "\n".join(L)


def body_fine2(n):
    """The same fine ladder from a SMALLER prefix (mix at 5), to show the flip is
    a property of the callee's total size and not of the prefix."""
    L = ["  int s = a;"]
    for i in range(5):
        L.append(f"  s = (s * {2 * i + 3} + {i + 1}) ^ (s >> {(i % 7) + 1});")
    for i in range(n):
        L.append(f"  s ^= (a >> {(i % 15) + 1});")
    L.append("  return s;")
    return "\n".join(L)


FAMILIES = {
    "arith": (body_arith, ""),
    "mix": (body_mix, ""),
    "loop": (body_loop, ""),
    "static": (body_arith, "static "),
    "fine": (body_fine, ""),
    "fine2": (body_fine2, ""),
    "finestatic": (body_fine, "static "),
}


def cell_src(fam, n):
    body, qual = FAMILIES[fam]
    return (
        f"{qual}int callee(int a) {{\n{body(n)}\n}}\n"
        f"int caller(int a) {{ return callee(a) + 7; }}\n"
        + ("int sink(int a) { return callee(a); }\n" if qual else "")
    )


CALLEE_MANGLED = {k: "?callee@@YAHH@Z" for k in
                  ("arith", "mix", "loop", "static", "fine", "fine2", "finestatic")}
CALLER_MANGLED = "?caller@@YAHH@Z"


# ---------------------------------------------------------------- the cell
def run(fam, n, profile, flags, outdir):
    tag = f"{fam}_{n:03d}_{profile}"
    cpp = os.path.join(outdir, "cells", tag + ".cpp")
    os.makedirs(os.path.dirname(cpp), exist_ok=True)
    with open(cpp, "w") as f:
        f.write(cell_src(fam, n))
    ildir = os.path.join(outdir, "il", tag)
    obj = os.path.join(outdir, "obj", tag + ".obj")
    os.makedirs(os.path.dirname(obj), exist_ok=True)
    env = dict(os.environ, C2RS_REQUIRE_TOOLCHAIN="1")
    rel = lambda q: os.path.relpath(q, ROOT)
    r1 = subprocess.run(
        [C2RS, "capture", rel(cpp), "--keep-il", rel(ildir), "--flags-file", rel(flags)],
        capture_output=True, text=True, cwd=ROOT, env=env,
    )
    r2 = subprocess.run(
        [C2RS, "compile", rel(cpp), "--keep-obj", rel(obj), "--flags-file", rel(flags)],
        capture_output=True, text=True, cwd=ROOT, env=env,
    )
    rec = {"family": fam, "n": n, "profile": profile, "tag": tag}
    if r1.returncode != 0 or r2.returncode != 0:
        rec["error"] = (r1.stderr + r2.stderr)[-400:]
        return rec
    gls = [x for x in os.listdir(ildir) if x.endswith(".gl")]
    exs = [x for x in os.listdir(ildir) if x.endswith(".ex")]
    gl = open(os.path.join(ildir, gls[0]), "rb").read()
    rec["ex_total"] = os.path.getsize(os.path.join(ildir, exs[0]))
    for r in glsize.records(gl):
        if r["name"] == CALLEE_MANGLED[fam]:
            rec["gl_size"] = r["size"]
            rec["gl_size_form"] = r["size_form"]
            rec["gl_attr"] = r["attr"]
        if r["name"] == CALLER_MANGLED:
            rec["caller_gl_size"] = r["size"]
    o = Obj(open(obj, "rb").read())
    ci = o.comdat_of(CALLEE_MANGLED[fam])
    ai = o.comdat_of(CALLER_MANGLED)
    rec["callee_text"] = o.sec(ci)["raw"] if ci else None
    rec["caller_text"] = o.sec(ai)["raw"] if ai else None
    if ai is None:
        rec["arm"] = "no-caller"
    else:
        tg = o.relocation_targets(ai)
        rec["caller_relocs"] = tg
        rec["arm"] = "kept" if CALLEE_MANGLED[fam] in tg else (
            "inlined" if ci else "absent")
    return rec


def main(argv):
    outdir = HERE
    fams = argv[1].split(",") if len(argv) > 1 else ["arith"]
    ns = [int(x) for x in argv[2].split(",")] if len(argv) > 2 else list(range(0, 40))
    profs = argv[3].split(",") if len(argv) > 3 else ["O1", "Ox"]
    out = open(os.path.join(outdir, "series.jsonl"), "a")
    for fam in fams:
        for p in profs:
            flags = os.path.join(outdir, f"flags_{p}.txt")
            for n in ns:
                rec = run(fam, n, p, flags, outdir)
                out.write(json.dumps(rec) + "\n")
                out.flush()
                print(
                    f"{rec['tag']:>22}  glSIZE={rec.get('gl_size')!s:>5}"
                    f" ({rec.get('gl_size_form','-')})"
                    f"  text={rec.get('callee_text')!s:>5}"
                    f"  {rec.get('arm', rec.get('error', '?'))}",
                    flush=True,
                )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
