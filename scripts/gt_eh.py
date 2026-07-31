#!/usr/bin/env python3
"""gt_eh.py — decode the C++ EH obj structure of an Xbox 360 COFF object.

Ground-truth measurement tooling for `docs/EH_RECORDS.md` §1-§4 (the obj
structure half of the EH rung).  Read-only; outside the std-only Rust
workspace on purpose, same status as `scripts/gt_dump.py`.

It answers, from bytes and never from a layout assumed off x86 MSVC:

  * item 1 — the two-word `{__CxxFrameHandler, __ehfuncinfo$F}` prefix and the
    function symbol's `Value`;
  * item 2 — every `.pdata` COMDAT of the function, its `BeginAddress` addend,
    the packed unwind word, and which funclet each record covers;
  * item 3 — the `Selection = 5` EH `.rdata`: the unwind map, the try-block
    map, the handler arrays, `FuncInfo`, and the IP-to-state map, each decoded
    field by field with its relocation target;
  * item 4 — the `.data` type-descriptor COMDATs;
  * the SECTION ORDER of the whole function group, which is the composition
    question `CODEGEN_FRAMED_CALLS.md` §5 left open for EH.

Usage:
    scripts/gt_eh.py <src.cpp> [--mode '<flags>']     # capture, then decode
    scripts/gt_eh.py --obj <file.obj>                 # decode an existing obj
    scripts/gt_eh.py <src.cpp> --text                 # also disassemble .text

Default `--mode` is the DC3 WORKLOAD profile, not the fixture profile:
`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`.  `docs/EH_RECORDS.md` §6.1
records a row that was sized at the fixture profile and is wrong at this one,
so the profile is printed on every run.

Env: C2RS_WIBO, C2RS_COMPILERS — as `scripts/gt_capture.sh`.
"""

import os
import struct
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gt_dump import Obj, disasm  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORKLOAD = "/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc"

FUNCINFO_FIELDS = [
    "magic",
    "maxState",
    "pUnwindMap",
    "nTryBlocks",
    "pTryBlockMap",
    "nIPMapEntries",
    "pIPtoStateMap",
    "pESTypeList",
    "EHFlags",
]


def be32(b, o):
    return struct.unpack_from(">I", b, o)[0]


def sbe32(b, o):
    return struct.unpack_from(">i", b, o)[0]


def capture(src, mode):
    env = dict(os.environ)
    out = os.path.splitext(os.path.abspath(src))[0] + ".obj"
    if os.path.exists(out):
        os.remove(out)
    env["GT_OUT"] = out
    cmd = [os.path.join(REPO, "scripts", "gt_capture.sh"), src] + mode.split()
    r = subprocess.run(cmd, capture_output=True, text=True, env=env)
    if not os.path.exists(out):
        sys.stderr.write(r.stderr)
        raise SystemExit("capture failed for %s" % src)
    return out


class EhObj:
    def __init__(self, path):
        self.path = path
        self.o = Obj(open(path, "rb").read())
        # symbol index -> symbol, and (section, value) -> names
        self.by_idx = {}
        for s in self.o.symbols:
            self.by_idx[s["idx"]] = s
        self.at = {}
        for s in self.o.symbols:
            if s["sec"] > 0 and s["naux"] == 0:
                self.at.setdefault((s["sec"], s["value"]), []).append(s["name"])

    def relmap(self, sec):
        """va -> target symbol name, for ADDR32 relocations only."""
        m = {}
        for va, sym, typ in self.o.relocs(sec):
            s = self.o.sym_by_index(sym)
            m.setdefault(va, []).append((typ, s["name"] if s else "?%d" % sym))
        return m

    def sec(self, i):
        return self.o.sections[i - 1]

    def sym_named(self, name):
        for s in self.o.symbols:
            if s["name"] == name:
                return s
        return None


def label(eh, sec_idx, off):
    names = eh.at.get((sec_idx, off), [])
    return ("  <- " + ", ".join(names)) if names else ""


def decode_pdata(eh, sec, funcs):
    raw = eh.o.raw(sec)
    rm = eh.relmap(sec)
    lines = []
    for i in range(0, len(raw), 8):
        addend = be32(raw, i)
        w = be32(raw, i + 4)
        tgt = rm.get(i, [("?", "?")])[0][1]
        prolog = w & 0xFF
        nwords = (w >> 8) & 0x3FFFFF
        bit30 = (w >> 30) & 1
        bit31 = (w >> 31) & 1
        base = None
        fs = eh.sym_named(tgt)
        if fs:
            base = fs["value"] + addend
        cover = ""
        if base is not None:
            cover = "  covers .text 0x%x..0x%x (%d words)%s" % (
                base,
                base + 4 * nwords,
                nwords,
                label(eh, fs["sec"], base),
            )
        lines.append(
            "    +0x%02x  BeginAddress=%s%+d  word=0x%08x  "
            "[bit31=%d bit30=%d len=%d prolog=%d]%s"
            % (i, tgt, addend, w, bit31, bit30, nwords, prolog, cover)
        )
    return lines


def decode_ehrdata(eh, sec):
    """Decode an EH .rdata: unwind map, tryblock map, handler arrays,
    FuncInfo, ipmap.  Boundaries come from the STATIC symbols c2 itself
    placed in the section, never from a guessed layout."""
    raw = eh.o.raw(sec)
    rm = eh.relmap(sec)
    idx = sec["idx"]
    out = []

    def word(o):
        r = rm.get(o)
        if r:
            return "%s (reloc %s)" % (r[0][1], r[0][0])
        return "0x%08x (%d)" % (be32(raw, o), sbe32(raw, o))

    # named anchors
    anchors = {}
    for s in eh.o.symbols:
        if s["sec"] == idx and s["naux"] == 0:
            anchors.setdefault(s["value"], []).append(s["name"])

    fi = None
    for v, names in anchors.items():
        for n in names:
            if n.startswith("__ehfuncinfo$"):
                fi = v
    out.append("    anchors: " + ", ".join(
        "0x%02x=%s" % (v, "/".join(n)) for v, n in sorted(anchors.items())))

    # pHandlerArray target -> nCatches, read off the try-block table
    ncatches_for = {}
    if fi is not None:
        tb = None
        for v, names in anchors.items():
            for n in names:
                if n.startswith("__tryblocktable$"):
                    tb = v
        if tb is not None:
            for k in range(be32(raw, fi + 0x0C)):
                o = tb + 20 * k
                ha = rm.get(o + 16)
                if ha:
                    tgt = ha[0][1]
                    for v2, names2 in anchors.items():
                        if tgt in names2:
                            ncatches_for[v2] = be32(raw, o + 12)

    # unwind map
    for v, names in sorted(anchors.items()):
        for n in names:
            if n.startswith("__unwindtable$"):
                nent = 0
                if fi is not None:
                    nent = be32(raw, fi + 4)
                out.append("    __unwindtable$ @0x%02x  (maxState=%d)" % (v, nent))
                for k in range(nent):
                    o = v + 8 * k
                    out.append("      [%d] toState=%-4s action=%s"
                               % (k, word(o), word(o + 4)))
            elif n.startswith("__tryblocktable$"):
                ntb = be32(raw, fi + 0x0C) if fi is not None else 0
                out.append("    __tryblocktable$ @0x%02x  (nTryBlocks=%d)" % (v, ntb))
                for k in range(ntb):
                    o = v + 20 * k
                    out.append(
                        "      [%d] tryLow=%s tryHigh=%s catchHigh=%s nCatches=%s "
                        "pHandlerArray=%s"
                        % (k, word(o), word(o + 4), word(o + 8), word(o + 12),
                           word(o + 16)))
            elif n.startswith("__catchsym$"):
                # LENGTH COMES FROM nCatches IN THE TRY-BLOCK ENTRY THAT POINTS
                # HERE, never from walking until the relocations run out: with
                # two try blocks the arrays are adjacent and a walk runs
                # straight through the boundary (it did, on probe pC).
                ncat = ncatches_for.get(v, 0)
                out.append("    %s @0x%02x  (%d HandlerType)" % (n, v, ncat))
                for k in range(ncat):
                    o = v + 16 * k
                    out.append(
                        "      [%d] adjectives=%s pType=%s dispCatchObj=%s "
                        "addressOfHandler=%s"
                        % (k, word(o), word(o + 4), word(o + 8), word(o + 12)))

    if fi is not None:
        out.append("    __ehfuncinfo$ @0x%02x" % fi)
        for k, name in enumerate(FUNCINFO_FIELDS):
            out.append("      +0x%02x %-14s %s" % (4 * k, name, word(fi + 4 * k)))
        end = fi + 4 * len(FUNCINFO_FIELDS)
        nip = be32(raw, fi + 0x14)
        pip = None
        for v, names in anchors.items():
            for n in names:
                if n.startswith("$T"):
                    pip = v
        out.append("      FuncInfo size = %d B (%d dwords); ipmap at 0x%02x"
                   " (pad %d)" % (4 * len(FUNCINFO_FIELDS), len(FUNCINFO_FIELDS),
                                  pip if pip is not None else -1,
                                  (pip - end) if pip is not None else -1))
        if pip is not None:
            out.append("    ip2state @0x%02x  (nIPMapEntries=%d)" % (pip, nip))
            for k in range(nip):
                o = pip + 8 * k
                out.append("      [%d] ip=%-24s state=%s" % (k, word(o), word(o + 4)))
    return out


def report(path, mode, show_text=False):
    eh = EhObj(path)
    o = eh.o
    print("== %s" % path)
    print("   mode: %s" % mode)
    print("-- section order")
    for s in o.sections:
        aux = ""
        for sym in o.symbols:
            if sym["sec"] == s["idx"] and sym["naux"] == 1 and sym["name"] == s["name"]:
                a = sym["aux"][0]
                aux = "cksum=0x%08x Number=%d Sel=%d" % (
                    struct.unpack_from("<I", a, 4)[0],
                    struct.unpack_from("<H", a, 12)[0],
                    a[14],
                )
                break
        print("  %2d %-9s raw=%-5d rel=%-2d chars=0x%08x  %s"
              % (s["idx"], s["name"], s["rawsize"], s["nrel"], s["chars"], aux))

    print("-- item 1: the handler prefix and the function symbol")
    for s in o.symbols:
        if s["sec"] > 0 and s["sc"] == 2 and o.sections[s["sec"] - 1]["name"] == ".text":
            print("   %-40s sec=%d Value=0x%x" % (s["name"], s["sec"], s["value"]))
    for si, s in enumerate(o.sections):
        if s["name"] != ".text":
            continue
        rm = eh.relmap(s)
        raw = o.raw(s)
        for va in sorted(rm):
            names = [n for _, n in rm[va]]
            if "__CxxFrameHandler" in names:
                w0 = be32(raw, va)
                w1 = be32(raw, va + 4) if va + 4 < len(raw) else None
                n1 = [n for _, n in rm.get(va + 4, [])]
                print("   prefix @ .text+0x%02x: [%08x -> %s] [%08x -> %s]%s"
                      % (va, w0, "__CxxFrameHandler", w1 or 0,
                         ",".join(n1), label(eh, s["idx"], va + 8)))

    print("-- item 2: .pdata")
    for s in o.sections:
        if s["name"] == ".pdata":
            print("  section %d (%d B)" % (s["idx"], s["rawsize"]))
            for l in decode_pdata(eh, s, None):
                print(l)

    print("-- item 3: the EH .rdata")
    for s in o.sections:
        if s["name"] != ".rdata":
            continue
        has_fi = any(sy["sec"] == s["idx"] and sy["name"].startswith("__ehfuncinfo$")
                     for sy in o.symbols)
        if not has_fi:
            print("  section %d (%d B): not an EH .rdata (no __ehfuncinfo$)"
                  % (s["idx"], s["rawsize"]))
            continue
        print("  section %d (%d B)" % (s["idx"], s["rawsize"]))
        for l in decode_ehrdata(eh, s):
            print(l)

    print("-- item 4: type descriptors")
    for s in o.sections:
        if s["name"] != ".data":
            continue
        names = [sy["name"] for sy in o.symbols
                 if sy["sec"] == s["idx"] and sy["naux"] == 0]
        raw = o.raw(s)
        rm = eh.relmap(s)
        print("  section %d (%d B) syms=%s  name=%r  relocs=%s"
              % (s["idx"], s["rawsize"], names,
                 raw[8:].rstrip(b"\0").decode("latin1", "replace"),
                 [(hex(v), rm[v][0][1]) for v in sorted(rm)]))

    print("-- labels in .text")
    for s in o.sections:
        if s["name"] != ".text":
            continue
        for sy in sorted((y for y in o.symbols if y["sec"] == s["idx"] and y["naux"] == 0),
                         key=lambda y: y["value"]):
            print("   0x%04x  %s" % (sy["value"], sy["name"]))

    if show_text:
        print("-- .text")
        for s in o.sections:
            if s["name"] != ".text":
                continue
            raw = o.raw(s)
            words = [be32(raw, i) for i in range(0, len(raw) & ~3, 4)]
            mn = disasm(words)
            rm = eh.relmap(s)
            for i, w in enumerate(words):
                va = 4 * i
                r = "  ; " + ", ".join("%s->%s" % (t, n) for t, n in rm[va]) if va in rm else ""
                print("   %04x  %08x  %-32s%s%s"
                      % (va, w, mn[i], label(eh, s["idx"], va), r))


# The probe corpus of docs/EH_RECORDS.md §8, embedded so the section is
# reproducible from tracked files alone (`work/` is gitignored, and captured
# objs are never committed).  `--write-probes <dir>` drops them all.
PROBES = {
    "eh1": "int g(int);\n"
           "int f(int a){ try { return g(a); } catch(int e) { return e+1; } }\n",
    "eh2": "struct S { S(); ~S(); int m; };\nint g(int);\n"
           "int f(int a){ S s; return g(a)+s.m; }\n",
    # catch AND destructor -- THE bit-31 test (§8.2a)
    "pA": "struct S { S(); ~S(); int m; };\nint g(int);\n"
          "int f(int a){ S s; try { return g(a)+s.m; } catch(int e){ return e+1; } }\n",
    "pB": "int g(int);\nint f(int a){ try { return g(a); } catch(int e){ return e+1; }"
          " catch(char* p){ return p?2:3; } }\n",
    "pC": "int g(int);\nint f(int a){ try { try { return g(a); }"
          " catch(int e){ return e+1; } } catch(char c){ return c+2; } }\n",
    # EH + a pooled FP constant -- the section order (§8.5a)
    "pD": "struct S { S(); ~S(); };\nfloat g(float);\n"
          "float f(float a){ S s; return g(a)*2.5f + a; }\n",
    "pE": "int g(int);\n"
          "int f(int a){ try { return g(a); } catch(...){ return 7; } }\n",
    "pF": "int g(int);\nint f1(int a){ try { return g(a); } catch(int e){ return e+1; } }\n"
          "int f2(int a){ try { return g(a)+1; } catch(int e){ return e+2; } }\n",
    "pG": "struct MemA { MemA(); ~MemA(); int x; };\n"
          "struct OneB { MemA a; void Fini(); ~OneB(); };\n"
          "OneB::~OneB(){ Fini(); }\n",
    # pG with a BIGGER frame -- the entry-SP base (§8.5c)
    "pH": "struct MemA { MemA(); ~MemA(); int x; };\n"
          "struct OneB { MemA a; void Fini(int*); ~OneB(); };\n"
          "OneB::~OneB(){ int buf[40]; Fini(buf); }\n",
    "pI": "int g(int);\nint f(int a){ try { return g(a); } catch(int e){ return e+1; }"
          " catch(char c){ return c+2; } catch(short s){ return s+3; } }\n",
    "pJ": "int g(int);\nint f(int a){\n"
          "  try { try { return g(a); } catch(int e){ return e+1; }"
          " catch(char c){ return c+2; } }\n"
          "  catch(short s){ return s+3; } catch(long l){ return (int)l+4; }\n}\n",
    # eh1 with a BIGGER frame -- dispCatchObj and the unwind-help word (§8.5c)
    "pK": "int g(int); int h(int*);\n"
          "int f(int a){ int buf[40]; try { return g(a)+h(buf); }"
          " catch(int e){ return e+1; } }\n",
    # the eh-bare / eh-plus-stmt boundary at the WORKLOAD profile (§7.2)
    "pL": "struct SE { SE(); ~SE(); int m; };\nint gp(int);\n"
          "void c1(){ gp(1); }\nvoid c2(){ SE s; }\nvoid c3(){ SE s; gp(1); }\n",
    "pM": "int g(int);\nint f(int a){\n"
          "  try { try { try { return g(a); } catch(int e){ return e+1; } }\n"
          "        catch(char c){ return c+2; } }\n"
          "  catch(short s){ return s+3; }\n}\n",
    "pN": "struct SE { SE(); ~SE(); int m; };\nint g(int);\n"
          "int f(int a){ SE s; try { return g(a)+s.m; } catch(int e){ return e+1; }"
          " catch(char c){ return c+2; } }\n",
    "pO": "int g(int);\nint f(int a){\n"
          "  try { try { return g(a); } catch(int e){ return e+1; } }\n"
          "  catch(char c){ return c+2; } catch(short s){ return s+3; }"
          " catch(long l){ return (int)l+4; }\n}\n",
    # the full cross: a pooled constant, a catch funclet AND a type descriptor
    "pP": "float g(float);\n"
          "float f(float a){ try { return g(a)*2.5f; } catch(int e){ return (float)e; } }\n",
    # adjectives, and the FALSIFIER for the 16-byte HandlerType
    "pQ": "struct E { E(); E(const E&); ~E(); int m; };\n"
          "struct F2 { F2(); F2(const F2&); ~F2(); int m; };\nint g(int);\n"
          "int f(int a){\n  try { return g(a); }\n"
          "  catch(E e2){ return e2.m+1; }\n  catch(const F2& e){ return e.m; }\n"
          "  catch(int& r){ return r+2; }\n"
          "  catch(const char* volatile p){ return p?4:5; }\n}\n",
    # TWO unwind funclets in one function
    "pR": "struct SE { SE(); ~SE(); int m; };\nint gp(int);\n"
          "int P(int a){ SE s; SE t; return gp(a)+s.m+t.m; }\n",
}


def main(argv):
    mode = WORKLOAD
    if "--write-probes" in argv:
        i = argv.index("--write-probes")
        d = argv[i + 1]
        os.makedirs(d, exist_ok=True)
        for n, src in sorted(PROBES.items()):
            open(os.path.join(d, n + ".cpp"), "w").write(src)
        print("wrote %d probes to %s" % (len(PROBES), d))
        return 0
    show_text = "--text" in argv
    argv = [a for a in argv if a != "--text"]
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    if "--obj" in argv:
        i = argv.index("--obj")
        report(argv[i + 1], "(pre-existing obj)", show_text)
        return 0
    if len(argv) < 2:
        print(__doc__)
        return 2
    for src in argv[1:]:
        report(capture(src, mode), mode, show_text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
