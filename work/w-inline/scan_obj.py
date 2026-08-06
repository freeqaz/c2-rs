#!/usr/bin/env python3
"""scan_obj.py — read INLINE-P's inputs, and its one-sided falsifier, out of a
reference obj.

Lane w-inline measurement tooling. **Read-only with respect to `crates/`.**

WHAT THIS IS FOR
----------------
`work/w-inline/PREREG.md` §1 states the incumbent `/O1` inline predicate as one
function of the *callee alone*, transcribed from `docs/LABEL_COUNTER.md`
§6.15-§6.20. Every input it needs is in the reference obj:

    s          the callee's own emitted `.text` COMDAT size          (§6.5)
    linkage    the COFF storage class of the callee's symbol         (§6.17.3)
    inline     PROPOSED: the COMDAT selection of its section         (unverified)
    nparams    from the mangled name, `this` included                (§6.17.6)
    leaf       no call in the emitted body, helpers not counted      (§6.19.6/7)
    varargs    from the mangled name                                 (§6.18.5)

and the *verdict* is in the obj too, by §6.15's own instrument:

> **An inlined call leaves no trace in the caller's relocation table; a
> declined one leaves exactly one REL24 against the callee's symbol.**

THE FALSIFIER, AND WHY IT IS ONE-SIDED
--------------------------------------
From an obj alone we can see the DECLINES (a surviving REL24) and not the
INLINES (nothing). So exactly one direction is falsifiable without knowing the
source's call sites:

    INLINE-P says "inlined at every site"  AND  a REL24 to G survives
        => the rule is WRONG about G.

The other direction — "never inlined" and no REL24 — is indistinguishable from
"the source never called G", so it is counted into an `unresolvable` bucket and
never scored. A grader that scored it would read every uncalled function in the
TU as a hit and report a precision that is a function of the corpus's dead code.
That is `docs/STATUS.md` trap 5's shape ("absence reads as success") and this
file refuses it by construction.

`#644`: no positional readers — every field is found by walking the structure.
`#843`: graded from obj bytes, never from a listing.

Usage:
    scan_obj.py <obj> [<obj>...] [--tsv PATH] [--summary]
    scan_obj.py --selftest
"""

import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "scripts"))
from gt_dump import Obj  # noqa: E402  — the project's one COFF reader

REL24 = 0x0006
IMAGE_SYM_CLASS_EXTERNAL = 2
IMAGE_SYM_CLASS_STATIC = 3
IMAGE_SYM_DTYPE_FUNCTION = 0x20

# COMDAT selection numbers (winnt.h).
SELECT = {
    1: "NODUPLICATES",
    2: "ANY",
    3: "SAME_SIZE",
    4: "EXACT_MATCH",
    5: "ASSOCIATIVE",
    6: "LARGEST",
}

# §6.19.6 — a REL24 to the out-of-line register-save/restore helpers is NOT a
# call. The shipped `callee_is_leaf()` counted them and was wrong by 48 bytes,
# which is six bands of SCHEDULE D.
HELPER = re.compile(r"^__(save|rest)(gprlr|fpr|vmx|gpr)_\d+$")


def _is_bcctr(w):
    """`bcctr`/`bcctrl` — an indirect transfer through CTR.

    §6.19.7: an indirect call counts, **and so does an indirect TAIL call** —
    the shipped predicate tested the LK bit and therefore called a `bctr`
    callee a leaf. The LK bit is ignored here on purpose.
    """
    return (w >> 26) == 19 and ((w >> 1) & 0x3FF) == 528


def _is_blr(w):
    return (w >> 26) == 19 and ((w >> 1) & 0x3FF) == 16


class Fn:
    __slots__ = (
        "name", "sec", "size", "sc", "selection", "words", "rel24", "has_indirect",
        "nparams", "varargs", "demangled", "parse_ok",
    )


def read_obj(path):
    """Every function COMDAT of one obj, with its relocations resolved to names."""
    data = open(path, "rb").read()
    o = Obj(data)

    # Section-definition aux records carry the COMDAT selection. They hang off
    # the *section symbol* (name == section name, sc == STATIC), not off the
    # function symbol, so the map is built first and looked up by index.
    selection = {}
    for s in o.symbols:
        if s["sc"] == IMAGE_SYM_CLASS_STATIC and s["naux"] >= 1 and s["sec"] > 0:
            sec = o.sections[s["sec"] - 1]
            if s["name"] == sec["name"]:
                aux = s["aux"][0]
                selection[s["sec"]] = aux[14]

    fns = {}
    by_sec = {}
    for s in o.symbols:
        if s["sec"] <= 0 or s["sc"] not in (IMAGE_SYM_CLASS_EXTERNAL, IMAGE_SYM_CLASS_STATIC):
            continue
        sec = o.sections[s["sec"] - 1]
        if sec["name"] != ".text":
            continue
        if (s["type"] & 0xF0) != IMAGE_SYM_DTYPE_FUNCTION:
            continue
        if s["name"] == sec["name"]:
            continue
        f = Fn()
        f.name = s["name"]
        f.sec = s["sec"]
        f.size = sec["rawsize"]
        f.sc = s["sc"]
        f.selection = selection.get(s["sec"])
        raw = o.raw(sec)
        f.words = [int.from_bytes(raw[i:i + 4], "big") for i in range(0, len(raw) - 3, 4)]
        f.rel24 = []
        f.has_indirect = any(_is_bcctr(w) for w in f.words)
        f.nparams = None
        f.varargs = None
        f.demangled = None
        f.parse_ok = False
        fns[f.name] = f
        by_sec[s["sec"]] = f

    # Relocations, resolved to target NAMES. `#644`: the reloc's symbol index is
    # followed into the symbol table rather than assumed to sit at a position.
    for sec in o.sections:
        if sec["name"] != ".text" or sec["idx"] not in by_sec:
            continue
        f = by_sec[sec["idx"]]
        for (_va, symidx, ty) in o.relocs(sec):
            if (ty & 0x00FF) != REL24:
                continue
            t = o.sym_by_index(symidx)
            if t is not None:
                f.rel24.append(t["name"])
    return fns


# ---------------------------------------------------------------------------
# nparams / varargs, from the mangled name via llvm-undname.
# ---------------------------------------------------------------------------

_QUALS = (" const", " volatile", " __ptr64", " __restrict", " &", " &&", " __unaligned")


def _param_list(dm):
    """The top-level parameter list of a demangled MSVC signature, or None.

    Scans from the END for the matching `(` of the last top-level `)`, after
    stripping trailing cv/ref qualifiers. A positional guess (`dm.index("(")`)
    would land in a template argument on every STL name in this workload.
    """
    s = dm.rstrip()
    changed = True
    while changed:
        changed = False
        for q in _QUALS:
            if s.endswith(q):
                s = s[: -len(q)].rstrip()
                changed = True
    if not s.endswith(")"):
        return None
    depth = 0
    for i in range(len(s) - 1, -1, -1):
        c = s[i]
        if c == ")":
            depth += 1
        elif c == "(":
            depth -= 1
            if depth == 0:
                return s[i + 1 : len(s) - 1], s[:i]
    return None


def _split_top(args):
    out, depth, cur = [], 0, ""
    for c in args:
        if c in "(<[":
            depth += 1
        elif c in ")>]":
            depth -= 1
        if c == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += c
    if cur.strip():
        out.append(cur)
    return [a.strip() for a in out]


def annotate_params(fns):
    """Fill `nparams`, `varargs`, `demangled`. Unparseable names stay `parse_ok=False`."""
    names = list(fns)
    if not names:
        return
    p = subprocess.run(
        ["llvm-undname"], input="\n".join(names) + "\n",
        capture_output=True, text=True,
    )
    # llvm-undname echoes the input line, then the demangling, then a blank.
    cur = None
    for line in p.stdout.splitlines():
        if line in fns:
            cur = line
            continue
        if cur is None or not line.strip():
            continue
        f = fns[cur]
        if f.demangled is not None:
            continue
        f.demangled = line
        pl = _param_list(line)
        if pl is None:
            cur = None
            continue
        args, head = pl
        parts = _split_top(args)
        if parts == ["void"] or parts == []:
            n = 0
        else:
            n = len(parts)
        f.varargs = bool(parts) and parts[-1] == "..."
        if f.varargs:
            n -= 1
        # `this` is an ordinary parameter 0 to the back end (§6.17.3), so it is
        # counted. A member is any name whose demangling carries an access
        # specifier; a `static` member has one too and has no `this`.
        is_member = head.startswith(("public:", "private:", "protected:"))
        if is_member and " static " not in head + " ":
            n += 1
        f.nparams = n
        f.parse_ok = True
        cur = None


# ---------------------------------------------------------------------------
# INLINE-P itself — every constant carries its section number.
# ---------------------------------------------------------------------------

UNBOUNDED = 10 ** 9


def is_leaf(f, defined):
    """No call in the emitted body (§6.18.6), helpers excluded (§6.19.6),
    indirect transfers counted with LK ignored (§6.19.7)."""
    if f.has_indirect:
        return False
    for t in f.rel24:
        if not HELPER.match(t):
            return False
    return True


def sched_index(f, leaf):
    """§6.17.3/§6.17.4/§6.17.5/§6.17.6 + §6.18.6."""
    if f.sc == IMAGE_SYM_CLASS_STATIC:
        idx = f.size
    else:
        inl = 1 if f.selection == 2 else 0          # SELECT_ANY  (PROPOSED)
        np = f.nparams if f.parse_ok else 1
        idx = f.size - 4 * (np - 1) - 8 * inl
    if leaf:
        idx -= 48                                    # §6.18.6
    return idx


def n_max(f, leaf, drop_leaf_term=False):
    """The number of sites c2 will inline. `UNBOUNDED` means every site."""
    if f.varargs:
        return 0                                     # §6.18.5
    idx = sched_index(f, False if drop_leaf_term else leaf)
    if f.sc == IMAGE_SYM_CLASS_EXTERNAL:
        return UNBOUNDED if idx <= 64 else 0         # §6.17.4
    i = idx // 4                                     # §6.18.9, LAW Dc
    if i >= 65:
        return 0
    if i <= 16:
        return UNBOUNDED
    return min(9, 1 + 19 // (i - 16))


# ---------------------------------------------------------------------------

HDR = ("obj\tcallee\tsize\tlinkage\tselection\tnparams\tvarargs\tleaf\tindex\t"
       "nmax\tinternal_callers\tself_recursive\tverdict")


def grade(path, fns, drop_leaf_term=False):
    rows = []
    callers = {n: set() for n in fns}
    selfrec = {n: False for n in fns}
    for f in fns.values():
        for t in f.rel24:
            if t in fns:
                if t == f.name:
                    selfrec[t] = True
                else:
                    callers[t].add(f.name)
    base = os.path.basename(path)
    for name, f in fns.items():
        leaf = is_leaf(f, fns)
        nm = n_max(f, leaf, drop_leaf_term)
        nc = len(callers[name])
        if nm >= UNBOUNDED:
            verdict = "FALSIFIED" if nc > 0 else "unresolvable-always"
        elif nm == 0:
            verdict = "consistent-never" if nc > 0 else "unresolvable-never"
        else:
            verdict = "consistent-bounded" if nc <= nm else "FALSIFIED-bounded"
        rows.append((
            base, name, f.size,
            "STATIC" if f.sc == IMAGE_SYM_CLASS_STATIC else "EXTERNAL",
            SELECT.get(f.selection, str(f.selection)),
            f.nparams if f.parse_ok else -1,
            int(bool(f.varargs)), int(leaf),
            sched_index(f, False if drop_leaf_term else leaf),
            "inf" if nm >= UNBOUNDED else nm,
            nc, int(selfrec[name]), verdict,
        ))
    return rows


def selftest():
    """Pins the two decoders that have no obj-independent witness."""
    ok = True
    for w, want in ((0x4E800421, True), (0x4E800420, True), (0x4E800020, False),
                    (0x48000000, False), (0x4BFFFFF1, False)):
        got = _is_bcctr(w)
        if got != want:
            print(f"FAIL bcctr {w:08x} {got} != {want}")
            ok = False
    if not _is_blr(0x4E800020):
        print("FAIL blr")
        ok = False
    cases = [
        ("public: int & __cdecl stlpmtx_std::vector<int, class A<int>>::back(void)", 1, False),
        ("void __cdecl f(void)", 0, False),
        ("public: class HamMove * __cdecl DataArray::Obj<class HamMove>(int) const", 2, False),
        ("int __cdecl v(int, ...)", 1, True),
        ("public: static int __cdecl S::s(int, int)", 2, False),
        ("void __cdecl g(int (__cdecl *)(int, int), int)", 2, False),
    ]
    for dm, wantn, wantv in cases:
        pl = _param_list(dm)
        if pl is None:
            print(f"FAIL parse {dm}")
            ok = False
            continue
        args, head = pl
        parts = _split_top(args)
        n = 0 if parts in ([], ["void"]) else len(parts)
        va = bool(parts) and parts[-1] == "..."
        if va:
            n -= 1
        if head.startswith(("public:", "private:", "protected:")) and " static " not in head + " ":
            n += 1
        if (n, va) != (wantn, wantv):
            print(f"FAIL params {dm}: {(n, va)} != {(wantn, wantv)}")
            ok = False
    for name, want in (("__savegprlr_29", True), ("__restfpr_14", True),
                       ("?f@@YAXXZ", False), ("__savegprlr", False)):
        if bool(HELPER.match(name)) != want:
            print(f"FAIL helper {name}")
            ok = False
    print("SELFTEST:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


def main(argv):
    if "--selftest" in argv:
        return selftest()
    tsv = None
    drop_leaf = "--drop-leaf-term" in argv
    if "--tsv" in argv:
        tsv = argv[argv.index("--tsv") + 1]
        argv = [a for a in argv if a != tsv]
    objs = [a for a in argv if not a.startswith("--")]
    out = []
    for p in objs:
        try:
            fns = read_obj(p)
        except Exception as e:                        # noqa: BLE001
            print(f"UNREADABLE {p}: {e}", file=sys.stderr)
            continue
        annotate_params(fns)
        out.extend(grade(p, fns, drop_leaf))
    lines = [HDR] + ["\t".join(str(c) for c in r) for r in out]
    if tsv:
        open(tsv, "w").write("\n".join(lines) + "\n")
    else:
        print("\n".join(lines))
    tot = {}
    for r in out:
        tot[r[-1]] = tot.get(r[-1], 0) + 1
    print("objs graded:", len(objs), " functions:", len(out), file=sys.stderr)
    for k in sorted(tot):
        print(f"  {k:24s} {tot[k]}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
