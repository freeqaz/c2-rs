#!/usr/bin/env python3
"""featmap.py — the FEATURE-UNION ranking of the FRONTIER.

Lane w-frame. Measurement tooling, outside the std-only Rust workspace (same
status as `scripts/plot_perf.py` and `scripts/gt_dump.py`); nothing here is
linked into the port.

WHAT IT ANSWERS
---------------
`c2rs gap` prints the FRONTIER ranked by **blocked-function count**. Three
lanes (w-front, w-pair, w-cfgimpl) each picked a target off the head of that
list and each converted zero TUs, and each independently reported the same
cause: bucket size is not distance (`BOARD.md` #150 / #197 / #198).

This script ranks the same 17 TUs by the key those lanes said was missing:

    gap(TU) = | union of emission features over EVERY function the obj emits
                MINUS the feature vocabulary the port has DEMONSTRABLY emitted |

A TU matches only when the whole obj is byte-exact, so the union is taken over
every emitted function, not only the census-blocked ones.

`port_vocab` is **measured, never asserted**: it is the feature set of the objs
the port already reproduces byte-exact (the 102 `Port=Match` fixtures plus the
8 matching workload TUs). If the port emits a construct today, that construct
is in the vocabulary by construction and cannot inflate anyone's gap.

WHAT THE KEY IS NOT (registered in the prereg before any number existed)
-----------------------------------------------------------------------
* It UNDER-counts. One bucket can be several independent facts — w-cfgimpl §4.1
  showed a single `if`-fold is four (bool spine, constant materialization, mask
  derivation, destination allocation).
* It cannot see SCHEDULE. w-pair measured `xboxheap.cpp` diverging at
  instruction 0 on instruction ORDER with every instruction already in
  vocabulary. `gap == 0` is therefore **not** a conversion claim.

Both errors point the same way — toward making TUs look cheaper than they are.

Usage:
    featmap.py obj  <obj>                     one obj, human-readable
    featmap.py rank                           the full ranking (compiles ~127 TUs)
"""

import json
import os
import struct
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "scripts"))

import gt_dump  # noqa: E402  — the COFF reader and the llvm-mc disassembly seam

# ---------------------------------------------------------------------------
# toolchain
# ---------------------------------------------------------------------------


def _sibling(name):
    d = REPO
    while d != "/":
        c = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(c):
            return c
        d = os.path.dirname(d)
    return None


WIBO = os.environ.get("C2RS_WIBO") or os.path.join(
    _sibling("wibo") or "", "build", "release", "wibo"
)
CL = os.path.join(REPO, "compilers", "X360", "16.00.11886.00", "cl.exe")
DC3 = os.environ.get("C2RS_DC3") or _sibling("dc3-decomp")
OBJDIR = os.path.join(HERE, "obj")

WORKLOAD_FLAGS = open(os.path.join(REPO, "work", "dc3-workload", "flags.txt")).read().split()
FIXTURE_FLAGS = ["/Ox", "/GS-", "/c"]


def compile_obj(src, flags, cwd, out):
    """Real cl.exe 16.00.11886.00 / c2.dll under wibo. Returns the obj bytes.

    NOT `c2rs compile` — board #195: it hardcodes `/Ox /GS- /c` and drops
    `--flag`, so it cannot produce an obj at the workload's own profile, and
    w-cfg measured that the profile reaches the bytes (the per-function
    optimization word moves 0x00a00005 -> 0x00200005).
    """
    if os.path.exists(out):
        return open(out, "rb").read()
    os.makedirs(os.path.dirname(out), exist_ok=True)
    zout = "Z:" + os.path.abspath(out).replace("/", "\\")
    env = dict(os.environ, TMP=os.path.dirname(out), TEMP=os.path.dirname(out),
               WIBO_FS_CACHE="1")
    subprocess.run([WIBO, CL] + flags + ["/Fo" + zout, src],
                   cwd=cwd, env=env, capture_output=True)
    if not os.path.exists(out) or os.path.getsize(out) == 0:
        return None
    return open(out, "rb").read()


# ---------------------------------------------------------------------------
# the classifier
# ---------------------------------------------------------------------------

# Section names that carry no emission decision: every obj cl produces has them
# and the port already writes all four.
SHELL_SECTIONS = {".drectve", ".debug$S", ".XBLD$W", ".text"}


# Symbol FAMILIES. A section name is far too coarse on its own: the port has
# emitted `.rdata` and `.pdata` in fixtures, so `Main.cpp` — whose whole
# distance is the EH record set (`__ehfuncinfo$main`, `__unwindtable$main`, an
# `__unwind$` funclet, a `$T` IP-to-state map and a `$M` label pair, w-pair §2)
# — scored ONE missing token on the instruction axis alone. Each prefix below
# is a distinct emission production, so each is its own token.
SYM_FAMILIES = [
    ("__ehfuncinfo$", "eh-funcinfo"),
    ("__unwindtable$", "eh-unwindtable"),
    ("__unwind$", "eh-funclet"),
    ("__catchsym$", "eh-catchsym"),
    ("__CxxFrameHandler", "eh-personality"),
    ("__savegprlr_", "helper-savegprlr"),
    ("__restgprlr_", "helper-restgprlr"),
    ("__savefpr_", "helper-savefpr"),
    ("__restfpr_", "helper-restfpr"),
    ("__savevmx_", "helper-savevmx"),
    ("__restvmx_", "helper-restvmx"),
    ("??_R", "rtti"),
    ("??_7", "vftable"),
    ("??_C@", "string-literal"),
    ("??__E", "dyninit-ctor"),
    ("??__F", "dyninit-dtor"),
    ("$T", "label-T"),
    ("$M", "label-M"),
]

# Compiler runtime helpers c2 CHOOSES to call — each is a lowering decision, not
# a user call. Matched on the undefined-external name.
HELPER_PREFIXES = ("_alldiv", "_aulldiv", "_allmul", "_allshl", "_allshr",
                   "_aullshr", "_allrem", "_aullrem", "__rt_", "memcpy",
                   "memset", "memmove", "_fltused", "__security")


def _sym_family(name):
    for pfx, fam in SYM_FAMILIES:
        if name.startswith(pfx):
            return fam
    return None


def _mnemonic(line):
    """Base mnemonic from an llvm-mc line. `bf\t26, .+12` -> `bf`."""
    line = line.strip()
    if line.startswith("<"):
        return "undecodable"
    for sep in ("\t", " "):
        if sep in line:
            return line.split(sep, 1)[0]
    return line


def classify(obj_bytes):
    """Return (per-function feature sets, TU-level feature set).

    A feature TOKEN is one of:
        insn:<mnemonic>     an instruction family, by base mnemonic
        frame:<class>       leaf / stwu / savegprlr / mflr
        reloc:<kind>        a relocation kind reaching a .text COMDAT
        sect:<name>         a section beyond the four every obj has
    """
    o = gt_dump.Obj(obj_bytes)

    # symbol name per .text section index (the COMDAT's EXTERNAL definition)
    names = {}
    for s in o.symbols:
        if s["sec"] > 0 and s["sc"] == 2 and not s["name"].startswith("."):
            names.setdefault(s["sec"], s["name"])

    tu = set()
    for sec in o.sections:
        if sec["name"] not in SHELL_SECTIONS:
            tu.add("sect:" + sec["name"])

    # symbol-family tokens, TU-level (a defined EH record is the TU's, and the
    # `.pdata`/`.rdata` COMDAT it lives in is not the function's own section)
    for s in o.symbols:
        fam = _sym_family(s["name"])
        if fam:
            tu.add("sym:" + fam)
        elif s["sec"] == 0 and s["name"].startswith(HELPER_PREFIXES):
            tu.add("sym:helper-" + s["name"].lstrip("_"))

    funcs = []
    for sec in o.sections:
        # Every CODE section, not just `.text`: a `??__E` dynamic initializer
        # lives in `.text$yc` and a `??__F` in `.text$yd`, and those are exactly
        # the bodies the two matching license TUs are made of. Walking only
        # `.text` gave them ZERO functions and made the leave-one-out control
        # vacuous rather than green.
        if not sec["name"].startswith(".text"):
            continue
        raw = o.raw(sec)
        words = [struct.unpack_from(">I", raw, i)[0] for i in range(0, len(raw) - 3, 4)]
        lines = gt_dump.disasm(words)
        mnem = [_mnemonic(l) for l in lines]

        f = set("insn:" + m for m in mnem)

        # frame class — the axis w-cfgimpl named as the frontier's wall
        if any(m == "stwu" for m in mnem):
            f.add("frame:stwu")
        if any(m == "mflr" for m in mnem):
            f.add("frame:mflr")
        savegpr = False
        for r in o.relocs(sec):
            sym = o.sym_by_index(r[1])
            if sym and sym["name"].startswith("__savegprlr"):
                savegpr = True
        if savegpr:
            f.add("frame:savegprlr")
        if not (f & {"frame:stwu", "frame:mflr", "frame:savegprlr"}):
            f.add("frame:leaf")

        for r in o.relocs(sec):
            f.add("reloc:" + gt_dump.RELOC.get(r[2], "0x%04x" % r[2]))
            sym = o.sym_by_index(r[1])
            if not sym:
                continue
            fam = _sym_family(sym["name"])
            if fam:
                f.add("sym:" + fam)
            elif sym["sec"] == 0 and sym["name"].startswith(HELPER_PREFIXES):
                f.add("sym:helper-" + sym["name"].lstrip("_"))

        funcs.append({
            "name": names.get(sec["idx"], "<sec %d>" % sec["idx"]),
            "size": len(raw),
            "features": sorted(f),
        })
        tu |= f

    return funcs, tu


def obj_features(path_or_bytes):
    b = path_or_bytes if isinstance(path_or_bytes, bytes) else open(path_or_bytes, "rb").read()
    return classify(b)


# ---------------------------------------------------------------------------
# drivers
# ---------------------------------------------------------------------------


def cmd_obj(path):
    funcs, tu = obj_features(path)
    for f in funcs:
        print("%-60s %4d B" % (f["name"], f["size"]))
        print("    " + " ".join(f["features"]))
    print("\nTU union (%d tokens):" % len(tu))
    print("  " + " ".join(sorted(tu)))


def _read_frontier(gap_txt):
    """The 17 FRONTIER TUs, read from a real `c2rs gap` run — never transcribed."""
    out = []
    seen = False
    for line in open(gap_txt):
        if line.startswith("  FRONTIER"):
            seen = True
            continue
        if seen:
            if "|" not in line:
                break
            parts = [p.strip() for p in line.split("|")]
            if len(parts) != 3:
                break
            out.append((int(parts[0]), int(parts[1]), parts[2]))
    return out


def _read_matches(gap_txt):
    for line in open(gap_txt):
        if "the joint is EXACTLY the match set" in line:
            return [s.strip() for s in line.split(":", 1)[1].split(",")]
    return []


def nearest_witness(feats, witnesses):
    """min over witnessed functions of |feats \\ witness| — and the argmin.

    WHY THIS EXISTS, beside the flat vocabulary. A flat token set is blind to
    COMBINATION: a body whose every instruction is in vocabulary can still be a
    shape nobody has emitted (a framed body that also stores and also compares).
    Subset-against-one-witness asks the sharper question — *has the port ever
    emitted a single function that already carries everything this one needs?*

    It is still a CEILING on cheapness, for the same two reasons the flat key
    is: it under-counts derivation (one `rlwinm` can be four independent facts)
    and it is blind to SCHEDULE entirely.
    """
    best, arg = None, None
    for wname, wf in witnesses:
        d = len(feats - wf)
        if best is None or d < best:
            best, arg = d, wname
            if d == 0:
                break
    return (best if best is not None else len(feats)), arg


def cmd_rank(gap_txt):
    frontier = _read_frontier(gap_txt)
    matches = _read_matches(gap_txt)
    assert len(frontier) == 17, "frontier is %d, not 17 — rerun the scan" % len(frontier)
    assert len(matches) == 8, "match set is %d, not 8" % len(matches)

    # --- port_vocab, half one: the fixtures the port reproduces byte-exact
    fixtures = [l.strip() for l in open(os.path.join(HERE, "match_fixtures.txt")) if l.strip()]
    fixdir = os.path.join(REPO, "fixtures", "cpp")
    vocab_fix, fix_ok = set(), 0
    wit_fix = []
    for fx in fixtures:
        b = compile_obj(fx, FIXTURE_FLAGS, fixdir, os.path.join(OBJDIR, "fix", fx + ".obj"))
        if b is None:
            continue
        fix_ok += 1
        fns, tu = obj_features(b)
        vocab_fix |= tu
        for f in fns:
            wit_fix.append((fx + ":" + f["name"], set(f["features"])))

    # --- port_vocab, half two: the 8 matching workload TUs, at workload flags
    vocab_wl, per_match, wit_wl = set(), {}, {}
    for src in matches:
        b = compile_obj(src, WORKLOAD_FLAGS, DC3,
                        os.path.join(OBJDIR, "wl", src.replace("/", "_") + ".obj"))
        if b is None:
            continue
        fns, tu = obj_features(b)
        per_match[src] = tu
        wit_wl[src] = [(src + ":" + f["name"], set(f["features"])) for f in fns]
        vocab_wl |= tu

    vocab = vocab_fix | vocab_wl
    witnesses = list(wit_fix) + [w for ws in wit_wl.values() for w in ws]

    # --- LEAVE-ONE-OUT control. A matching TU must score 0 on BOTH keys against
    # a vocabulary that EXCLUDES it; scoring it against a vocabulary that
    # includes it is a tautology, not a control.
    loo = {}
    for src, feats in per_match.items():
        others = set(vocab_fix)
        owit = list(wit_fix)
        for s2, f2 in per_match.items():
            if s2 != src:
                others |= f2
                owit += wit_wl[s2]
        loo[src] = {
            "lex": sorted(feats - others),
            "wit": max([nearest_witness(w[1], owit)[0] for w in wit_wl[src]] or [-1]),
        }

    # --- the ranking
    rows = []
    for blocked, emitted, src in frontier:
        b = compile_obj(src, WORKLOAD_FLAGS, DC3,
                        os.path.join(OBJDIR, "fr", src.replace("/", "_") + ".obj"))
        if b is None:
            rows.append({"src": src, "blocked": blocked, "emitted": emitted, "error": "no obj"})
            continue
        funcs, tu = obj_features(b)
        miss = sorted(tu - vocab)
        frows = []
        for f in funcs:
            fs = set(f["features"])
            d, arg = nearest_witness(fs, witnesses)
            frows.append({"name": f["name"], "size": f["size"],
                          "lex": sorted(fs - vocab), "wit": d, "nearest": arg})
        rows.append({
            "src": src, "blocked": blocked, "emitted": emitted,
            "nfunc": len(funcs), "union": len(tu),
            "gap": len(miss), "missing": miss,
            "wit_sum": sum(f["wit"] for f in frows),
            "wit_max": max(f["wit"] for f in frows),
            "funcs": frows,
        })

    out = {
        "vocab_fixture_objs": fix_ok,
        "vocab_fixture_tokens": len(vocab_fix),
        "vocab_workload_tokens": len(vocab_wl),
        "vocab_tokens": len(vocab),
        "witness_functions": len(witnesses),
        "vocab": sorted(vocab),
        "loo_control": loo,
        "rows": sorted(rows, key=lambda r: (r.get("wit_sum", 9999), r.get("gap", 999), r["src"])),
    }
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    if len(sys.argv) >= 3 and sys.argv[1] == "obj":
        cmd_obj(sys.argv[2])
    elif len(sys.argv) >= 2 and sys.argv[1] == "rank":
        cmd_rank(sys.argv[2] if len(sys.argv) > 2 else os.path.join(HERE, "gap_base.txt"))
    else:
        print(__doc__)
        sys.exit(2)
