#!/usr/bin/env python3
"""w-r8idiom, obj side: WHAT is the `mr r8,r8` population?

`ref/P_OPATTR.md` §6 (`w-tailread`) measured **3,792** `mr r8,r8` self-moves in
**1,206 of 120,000** objs -- all naming `r8`, branch-adjacent, `/Ox`, with no
relocation covering them -- and *deliberately declined to guess what they are*.
This probe characterises that population instead of guessing: where each one
sits, what is on either side of it, which symbol (if any) is bound to its
address, and which source produced it.

`probe_selfmove.py` is the sibling that FOUND them; it answers "do they exist".
This one answers "what are they", and adds the two things a population claim
needs and a refutation does not: **a denominator** and **a control**.

WHAT MAKES EACH NUMBER CAPABLE OF BEING WRONG

  * `--census` prints, beside every "N of the self-moves are X", the count of
    NON-self-move sites that are also X.  A predicate true of 100 % of the
    self-moves is worthless if it is also true of 100 % of everything else.
  * `--control` runs the inverse population: objs that have the enabling
    feature (C++ EH) and NO self-move.  If that set is empty the feature is
    necessary AND sufficient on this corpus; if it is large the feature is at
    best necessary, and the record must say so.
  * VACUITY is checked and reported as its own outcome (exit 2), never as a
    pass -- board #3341, "0 SKIP lines" is not evidence.  The refutation check
    runs FIRST, because a corpus whose only move form is a self-move has a zero
    denominator and would otherwise be called vacuous
    (`probe_selfmove.py`'s own fence bug, found by testing it).

THE CORPUS FENCE, AND WHY IT IS TIED TO THE PINNED IMAGE

An obj-side tool has no image to hash, so "sha256-fenced" would normally mean
nothing here.  It does not have to.  Every capture-cache entry records the
compiler that produced it:

    tool c2.dll 1347072:79b0c1d78becf1d8b9496949aaeb1326

and that string is `len(bytes) : digest128(bytes)` over the c2.dll file
(`crates/c2-il/src/cachefmt.rs` -- FNV-1a-64 forward, then FNV-1a-64 over the
reversed bytes seeded with the first).  So this probe:

  1. sha256-verifies the image it is handed against the pin, and refuses
     otherwise;
  2. RECOMPUTES that tool line from those very bytes; and
  3. requires every obj it measures to record exactly that line.

An obj built by any other c2 is counted as `foreign` and excluded, and a corpus
that is entirely foreign is a REFUSE, not a pass.  The corpus is therefore
fenced to the same image the disassembly side reads, by construction and not by
assumption.  (`digest128` is not cryptographic and is not relied on as such --
here it only has to distinguish one shipped DLL from another.)

WHAT IT CANNOT DO
  * An obj is POST-EVERYTHING.  It cannot say which c2 pass minted a word.
  * The capture cache is a FIXTURE corpus written by many lanes.  Source
    attribution therefore describes the fixtures, not C++ at large, and
    `--sources` prints the generator share so that limit cannot be dropped.

Usage:
    python3 docs/whitebox/scripts/probe_r8idiom.py <c2.dll> --census
    python3 docs/whitebox/scripts/probe_r8idiom.py <c2.dll> --sources
    python3 docs/whitebox/scripts/probe_r8idiom.py <c2.dll> --control
    python3 docs/whitebox/scripts/probe_r8idiom.py <c2.dll> --show <objdir>
      [--cache DIR] [--limit N] [--skip N]
"""

import collections
import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, "scripts"))

from gt_dump import Obj  # noqa: E402

PINNED_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

SELF_MR_WORD = 0x7D084378          # `or r8,r8,r8`, i.e. `mr r8,r8`
M64 = (1 << 64) - 1


# ------------------------------------------------------------------ the fence

def fnv1a64(seed, b):
    h = seed
    for x in b:
        h ^= x
        h = (h * 0x100000001B3) & M64
    return h


def tool_line_for(path):
    """The exact `tool c2.dll …` line a capture written by THIS image carries.

    Mirrors `c2-il::cachefmt::digest128` and `c2-harness::capture_cache::
    file_digest`.  Ported rather than shared because this file may not import
    from `crates/`; the port is checked against a real entry.bin by --selftest.
    """
    b = open(path, "rb").read()
    if hashlib.sha256(b).hexdigest() != PINNED_SHA256:
        return None, b
    h1 = fnv1a64(0xCBF29CE484222325, b)
    h2 = fnv1a64(h1 ^ 0x9E3779B97F4A7C15, b[::-1])
    return "tool c2.dll %d:%016x%016x" % (len(b), h1, h2), b


# --------------------------------------------------------------- obj plumbing

def be32(b, o):
    return int.from_bytes(b[o:o + 4], "big")


def mnemonic(w):
    """A coarse name for one big-endian PPC word -- enough to bucket a
    neighbourhood.  Deliberately NOT a disassembler: only the forms this
    question turns on are named, everything else is `op<primary>`."""
    op = w >> 26
    if op == 18:
        return "bl" if (w & 1) else "b"
    if op == 16:
        return "bcl" if (w & 1) else "bc"
    if op == 19:
        xo = (w >> 1) & 0x3FF
        if xo == 16:
            return "blrl" if (w & 1) else "blr"
        if xo == 528:
            return "bctrl" if (w & 1) else "bctr"
        return "op19"
    if op == 31 and ((w >> 1) & 0x3FF) == 444:
        rs, ra, rb = (w >> 21) & 31, (w >> 16) & 31, (w >> 11) & 31
        if rs == rb:
            return "mr.self" if ra == rs else "mr"
        return "or"
    if op == 24 and w == 0x60000000:
        return "nop"
    return "op%d" % op


def is_self_mr(w):
    op = w >> 26
    if op != 31 or ((w >> 1) & 0x3FF) != 444:
        return False
    rs, ra, rb = (w >> 21) & 31, (w >> 16) & 31, (w >> 11) & 31
    return rs == rb == ra


def parse_entry(path):
    """-> (meta_text, kv) from a capture-cache entry.bin.

    The container is `C2RSCAP\\x02` + header + sections; the `meta` section is a
    newline-keyed preamble followed by NUL-separated key/value pairs.  Nothing
    here needs the section table: the preamble is located by its own magic
    string and the pairs are read from where it ends.
    """
    try:
        b = open(path, "rb").read()
    except OSError:
        return "", {}
    i = b.find(b"c2rs-capture-cache")
    if i < 0:
        return "", {}
    j = b.find(b"\0", i)
    if j < 0:
        return b[i:].decode("latin1"), {}
    # OFF-BY-ONE, CAUGHT BY --show PRINTING `src-arg: ?` ON AN OBJ THAT PLAINLY
    # HAS ONE.  The newline-keyed preamble is NOT NUL-terminated: the first NUL
    # in the blob terminates the first KEY, which therefore sits at the tail of
    # `head`.  Splitting the remainder into pairs without it pairs every value
    # with the NEXT key and silently yields an empty `src-arg`.  Take the last
    # preamble line as key zero.
    head_raw = b[i:j].decode("latin1")
    nl = head_raw.rfind("\n")
    head, first_key = head_raw[:nl], head_raw[nl + 1:]
    parts = [first_key.encode("latin1")] + b[j + 1:].split(b"\0")
    kv = {}
    for k in range(0, len(parts) - 1, 2):
        key = parts[k].decode("latin1")
        if not key or "\n" in key:
            break
        kv[key] = parts[k + 1].decode("latin1")
    return head, kv


def iter_entries(root, limit, skip):
    n = seen = 0
    for dirpath, _dirnames, filenames in os.walk(root):
        if "out.obj" not in filenames:
            continue
        seen += 1
        if seen <= skip:
            continue
        yield dirpath
        n += 1
        if limit and n >= limit:
            return


# ----------------------------------------------------------------- the census

class Census:
    def __init__(self):
        self.nobj = self.nbad = self.nforeign = 0
        self.nself = 0
        self.self_objs = set()
        self.eh_objs = set()
        self.eh_objs_no_self = set()
        self.reg = collections.Counter()
        self.prev = collections.Counter()      # instruction BEFORE a self-move
        self.next = collections.Counter()      # instruction AFTER a self-move
        self.prev_all = collections.Counter()  # ... and for every OTHER word
        self.next_all = collections.Counter()
        self.symbol_at = collections.Counter()  # symbol class bound to the addr
        self.symkind = collections.Counter()
        self.sec = collections.Counter()
        self.reloc_cover = 0
        self.bl_total = 0
        self.bl_followed = 0
        self.bl_in_eh_obj = 0
        self.bl_in_eh_followed = 0
        self.nonself_words = 0
        self.src = collections.Counter()
        self.flags = collections.Counter()
        self.optarg = collections.Counter()
        self.src_bearing = collections.Counter()
        self.eh_bearing_flagcheck = collections.Counter()
        self.examples = []
        self.run_len = collections.Counter()
        # runs, and what brackets them -- the structural question
        self.run_shape = collections.Counter()   # (before, after) mnemonics
        self.run_class = collections.Counter()   # pre-call / post-call / other
        self.pair_len = collections.Counter()    # (len before bl, len after bl)
        # per-obj correlations
        self.corr_catch = collections.Counter()  # (n_self, n_catch_sym)
        self.corr_bl = collections.Counter()     # (n_self, n_bl)
        self.per_obj = []                        # (n_self, n_catch, n_bl, n_try)
        self.maxrun_catch = collections.Counter()  # (max run length, n_catch)
        self._obj_maxrun = 0

    def add_obj(self, dirpath, o, kv):
        self.nobj += 1
        names = [s["name"] for s in o.symbols]
        has_eh = any(n.startswith("__ehfuncinfo$") or n == "__CxxFrameHandler"
                     for n in names)
        if has_eh:
            self.eh_objs.add(dirpath)
        n_catch = sum(1 for n in names if n.startswith("__catch$"))
        n_try = sum(1 for n in names if n.startswith("__tryblocktable$"))
        n_bl_obj = 0
        found_here = 0
        for s in o.sections:
            if not s["name"].startswith(".text"):
                continue
            try:
                raw = o.raw(s)
            except Exception:
                continue
            reloc_offs = {r[0] for r in o.relocs(s)}
            # symbol -> offset map for this section index
            symat = collections.defaultdict(list)
            for sym in o.symbols:
                if sym["sec"] == s["idx"]:
                    symat[sym["value"]].append(sym)
            nw = len(raw) // 4
            run = 0
            for i in range(nw):
                off = i * 4
                w = be32(raw, off)
                pv = mnemonic(be32(raw, off - 4)) if i else "<start>"
                nx = mnemonic(be32(raw, off + 4)) if i + 1 < nw else "<end>"
                if mnemonic(w) == "bl":
                    self.bl_total += 1
                    n_bl_obj += 1
                    if has_eh:
                        self.bl_in_eh_obj += 1
                    if i + 1 < nw and is_self_mr(be32(raw, off + 4)):
                        self.bl_followed += 1
                        if has_eh:
                            self.bl_in_eh_followed += 1
                if not is_self_mr(w):
                    self.nonself_words += 1
                    self.prev_all[pv] += 1
                    self.next_all[nx] += 1
                    if run:
                        self.run_len[run] += 1
                        run = 0
                    continue
                run += 1
                found_here += 1
                self.nself += 1
                self.reg[(w >> 21) & 31] += 1
                self.prev[pv] += 1
                self.next[nx] += 1
                self.sec[s["name"]] += 1
                if off in reloc_offs:
                    self.reloc_cover += 1
                here = symat.get(off, [])
                if not here:
                    self.symbol_at["<none>"] += 1
                else:
                    for sym in here:
                        self.symbol_at[sym_class(sym["name"])] += 1
                        self.symkind[sym["sc"]] += 1
                if len(self.examples) < 6:
                    self.examples.append((dirpath, s["name"], off,
                                          [x["name"] for x in here], pv, nx))
            if run:
                self.run_len[run] += 1
            self._runs(raw, nw)
        self.per_obj.append((found_here, n_catch, n_bl_obj, n_try))
        if found_here:
            self.corr_catch[(found_here, n_catch)] += 1
            self.corr_bl[(found_here, n_bl_obj)] += 1
            self.maxrun_catch[(self._obj_maxrun, n_catch)] += 1
        self._obj_maxrun = 0
        if found_here:
            self.self_objs.add(dirpath)
            self.src_bearing[src_key(kv)] += 1
            self.flags[kv.get("flags", "?").replace("\x1f", " ").strip()] += 1
        elif has_eh:
            self.eh_objs_no_self.add(dirpath)
        self.src[src_key(kv)] += 1


    def _runs(self, raw, nw):
        """Maximal runs of consecutive self-moves, and what brackets each.

        A per-instruction neighbour histogram cannot see this: inside a run of
        three, two of the three neighbours are self-moves and the bracketing
        instruction is invisible.  `w-tailread`'s `self|self x634` row is that
        blind spot, and this is what replaces it.
        """
        i = 0
        pend = None            # a run that ended at a `bl`, awaiting its mate
        while i < nw:
            if not is_self_mr(be32(raw, i * 4)):
                i += 1
                continue
            j = i
            while j < nw and is_self_mr(be32(raw, j * 4)):
                j += 1
            before = mnemonic(be32(raw, (i - 1) * 4)) if i else "<start>"
            after = mnemonic(be32(raw, j * 4)) if j < nw else "<end>"
            self.run_shape[(before, after)] += 1
            self._obj_maxrun = max(self._obj_maxrun, j - i)
            if after == "bl":
                self.run_class["run then `bl`"] += 1
                pend = j - i
            elif before == "bl":
                self.run_class["`bl` then run"] += 1
                if pend is not None:
                    self.pair_len[(pend, j - i)] += 1
                    pend = None
            else:
                self.run_class["not adjacent to a `bl`"] += 1
                pend = None
            i = j


def sym_class(name):
    """Bucket a COFF symbol name by the compiler-internal family it belongs to."""
    if name.startswith("$M"):
        return "$M#### (EH state marker)"
    if name.startswith("$LN"):
        return "$LN#### (code label)"
    if name.startswith("$T"):
        return "$T#### (data label)"
    if name.startswith("__catch$"):
        return "__catch$####"
    if name.startswith("."):
        return "section symbol"
    return "other/user"


def src_key(kv):
    p = kv.get("src-arg", "?")
    return p.replace("\\", "/").rsplit("/", 1)[-1]


def is_generated(name):
    """A capture-cache source that came out of a corpus generator rather than a
    tracked fixture.  The generator names its files `NN-family-NNNN.cpp`."""
    stem = name[:-4] if name.endswith(".cpp") else name
    bits = stem.split("-")
    return (len(bits) >= 3 and bits[0].isdigit() and bits[-1].isdigit()
            and len(bits[-1]) == 4)


# ------------------------------------------------------------------- printing

def pct(a, b):
    return "%.2f%%" % (100.0 * a / b) if b else "n/a"


def run(cache, limit, skip, tool_line):
    c = Census()
    for dirpath in iter_entries(cache, limit, skip):
        head, kv = parse_entry(os.path.join(dirpath, "entry.bin"))
        if tool_line and tool_line not in head:
            c.nforeign += 1
            continue
        try:
            o = Obj(open(os.path.join(dirpath, "out.obj"), "rb").read())
        except Exception:
            c.nbad += 1
            continue
        c.add_obj(dirpath, o, kv)
    return c


def report_census(c):
    print("corpus: %d objs measured, %d foreign (other c2), %d unreadable"
          % (c.nobj, c.nforeign, c.nbad))
    print("self-moves: %d in %d objs (%s)"
          % (c.nself, len(c.self_objs), pct(len(c.self_objs), c.nobj)))
    print("registers named: %s"
          % ", ".join("r%d x%d" % (r, n) for r, n in c.reg.most_common()))
    print("relocations covering a self-move: %d" % c.reloc_cover)
    print()
    print("-- WHAT IS IMMEDIATELY BEFORE ONE (self-move sites vs every other word)")
    print("%-14s %10s %8s   %12s %8s" % ("prev", "self", "share", "all-other", "share"))
    for k, n in c.prev.most_common(8):
        print("%-14s %10d %8s   %12d %8s"
              % (k, n, pct(n, c.nself), c.prev_all[k], pct(c.prev_all[k],
                                                           c.nonself_words)))
    print()
    print("-- WHAT IS IMMEDIATELY AFTER ONE")
    print("%-14s %10s %8s   %12s %8s" % ("next", "self", "share", "all-other", "share"))
    for k, n in c.next.most_common(8):
        print("%-14s %10d %8s   %12d %8s"
              % (k, n, pct(n, c.nself), c.next_all[k], pct(c.next_all[k],
                                                           c.nonself_words)))
    print()
    print("-- RUN LENGTHS (consecutive self-moves)")
    for k in sorted(c.run_len):
        print("  %d in a row: %d runs" % (k, c.run_len[k]))
    print()
    print("-- RUNS, AND WHAT BRACKETS THEM  (a run is a maximal consecutive group)")
    for k, n in c.run_class.most_common():
        print("  %-26s %8d" % (k, n))
    print("  top bracketings (before | run | after):")
    for (b, a), n in c.run_shape.most_common(8):
        print("    %-10s | run | %-10s  %8d" % (b, a, n))
    print()
    print("-- LENGTH OF THE RUN BEFORE A `bl` vs THE RUN AFTER IT")
    print("     (before, after) : occurrences")
    eq = sum(n for (x, y), n in c.pair_len.items() if x == y)
    tot = sum(c.pair_len.values())
    for k, n in sorted(c.pair_len.items()):
        print("     %-14s : %d" % (str(k), n))
    print("  EQUAL on %d of %d bracketed calls (%s)" % (eq, tot, pct(eq, tot)))
    print()
    print("-- PER-OBJ: self-move count vs `__catch$` symbol count")
    print("     (n_self, n_catch) : objs")
    for k, n in sorted(c.corr_catch.items())[:16]:
        print("     %-18s : %d" % (str(k), n))
    same = sum(n for (s_, c_), n in c.corr_catch.items() if s_ == 2 * c_)
    tot2 = sum(c.corr_catch.values())
    print("  n_self == 2 * n_catch on %d of %d bearing objs (%s)"
          % (same, tot2, pct(same, tot2)))
    print()
    print("-- PER-OBJ: LONGEST RUN vs `__catch$` symbol count")
    print("     (max_run, n_catch) : objs")
    for k, n in sorted(c.maxrun_catch.items()):
        print("     %-18s : %d" % (str(k), n))
    eqr = sum(n for (r_, c_), n in c.maxrun_catch.items() if r_ == c_)
    print("  max_run == n_catch on %d of %d bearing objs (%s)"
          % (eqr, tot2, pct(eqr, tot2)))
    print()
    print("-- SYMBOL BOUND TO THE SELF-MOVE'S OWN ADDRESS")
    for k, n in c.symbol_at.most_common():
        print("  %-28s %8d  %s" % (k, n, pct(n, c.nself)))
    print()
    print("-- SECTION")
    for k, n in c.sec.most_common(6):
        print("  %-20s %8d" % (k, n))
    print()
    print("-- THE DENOMINATOR AND THE CONTROL")
    print("  objs with C++ EH (__ehfuncinfo$/__CxxFrameHandler): %d (%s)"
          % (len(c.eh_objs), pct(len(c.eh_objs), c.nobj)))
    print("  ... of those, carrying >=1 self-move:              %d (%s)"
          % (len(c.eh_objs & c.self_objs),
             pct(len(c.eh_objs & c.self_objs), len(c.eh_objs))))
    print("  ... of those, carrying NONE:                       %d (%s)"
          % (len(c.eh_objs_no_self),
             pct(len(c.eh_objs_no_self), len(c.eh_objs))))
    print("  self-move objs WITHOUT EH:                         %d"
          % len(c.self_objs - c.eh_objs))
    print("  `bl` sites, whole corpus:      %d, of which followed by a"
          " self-move: %d (%s)" % (c.bl_total, c.bl_followed,
                                   pct(c.bl_followed, c.bl_total)))
    print("  `bl` sites in EH-bearing objs: %d, of which followed: %d (%s)"
          % (c.bl_in_eh_obj, c.bl_in_eh_followed,
             pct(c.bl_in_eh_followed, c.bl_in_eh_obj)))
    print()
    print("-- first %d sites" % len(c.examples))
    for d, sec, off, syms, pv, nx in c.examples:
        print("  %s %s+%#x  syms=%s  [%s | SELF | %s]"
              % (os.path.basename(d), sec, off, syms or "-", pv, nx))


def report_sources(c):
    print("corpus: %d objs measured, %d foreign, %d unreadable"
          % (c.nobj, c.nforeign, c.nbad))
    print("self-move-bearing objs: %d, over %d distinct sources"
          % (len(c.self_objs), len(c.src_bearing)))
    print()
    tot = sum(c.src_bearing.values())
    gen = sum(n for s, n in c.src_bearing.items() if is_generated(s))
    print("generator-produced share of bearing objs: %d of %d (%s)"
          % (gen, tot, pct(gen, tot)))
    print("generator-produced share of the WHOLE corpus: %s"
          % pct(sum(n for s, n in c.src.items() if is_generated(s)),
                sum(c.src.values())))
    print()
    print("-- bearing objs by source (top 25), with that source's TOTAL objs")
    cum = 0
    for i, (s, n) in enumerate(c.src_bearing.most_common(25)):
        cum += n
        print("  %-42s %6d  of %6d captured  cum %s"
              % (s[:42], n, c.src[s], pct(cum, tot)))
    print()
    print("-- family (the generator's NN-family- prefix) over bearing objs")
    fam = collections.Counter()
    for s, n in c.src_bearing.items():
        stem = s[:-4] if s.endswith(".cpp") else s
        bits = stem.split("-")
        fam["-".join(bits[:-1]) if is_generated(s) else "(not generated)"] += n
    for k, n in fam.most_common(20):
        print("  %-42s %6d  %s" % (k[:42], n, pct(n, tot)))
    print()
    print("-- flags on bearing objs")
    for k, n in c.flags.most_common(10):
        print("  %-42s %6d" % (k[:42], n))


def report_control(c):
    """The inverse population, printed on its own so it cannot be skipped."""
    print("corpus: %d objs measured, %d foreign" % (c.nobj, c.nforeign))
    print()
    print("EH-bearing objs with NO self-move: %d" % len(c.eh_objs_no_self))
    print("EH-bearing objs WITH one:          %d" % len(c.eh_objs & c.self_objs))
    print("non-EH objs with a self-move:      %d" % len(c.self_objs - c.eh_objs))
    print()
    print("If the middle line is the whole of the first two, C++ EH is")
    print("necessary AND sufficient on this corpus.  If the first line is")
    print("large, EH is at best NECESSARY and something narrower selects.")
    print()
    print("`bl` followed by a self-move, inside EH-bearing objs: %d of %d (%s)"
          % (c.bl_in_eh_followed, c.bl_in_eh_obj,
             pct(c.bl_in_eh_followed, c.bl_in_eh_obj)))
    print("`bl` followed by a self-move, whole corpus:           %d of %d (%s)"
          % (c.bl_followed, c.bl_total, pct(c.bl_followed, c.bl_total)))


def show(objdir):
    o = Obj(open(os.path.join(objdir, "out.obj"), "rb").read())
    head, kv = parse_entry(os.path.join(objdir, "entry.bin"))
    print("src-arg: %s" % kv.get("src-arg", "?"))
    print("flags  : %s" % kv.get("flags", "?").replace("\x1f", " "))
    for s in o.sections:
        if not s["name"].startswith(".text"):
            continue
        raw = o.raw(s)
        symat = collections.defaultdict(list)
        for sym in o.symbols:
            if sym["sec"] == s["idx"]:
                symat[sym["value"]].append(sym["name"])
        print("-- %s (%d B)" % (s["name"], len(raw)))
        for i in range(len(raw) // 4):
            off, w = i * 4, be32(raw, i * 4)
            tag = "  <== SELF-MOVE" if is_self_mr(w) else ""
            lab = (" ; " + ",".join(symat[off])) if off in symat else ""
            print("  %04x  %08x  %-8s%s%s" % (off, w, mnemonic(w), lab, tag))


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    image = argv[0]
    mode = None
    cache = os.path.expanduser("~/.cache/c2rs/capture")
    limit, skip, target = 120000, 0, None
    i = 1
    while i < len(argv):
        a = argv[i]
        if a in ("--census", "--sources", "--control", "--selftest"):
            mode = a[2:]; i += 1
        elif a == "--show":
            mode, target = "show", argv[i + 1]; i += 2
        elif a == "--cache":
            cache = argv[i + 1]; i += 2
        elif a == "--limit":
            limit = int(argv[i + 1]); i += 2
        elif a == "--skip":
            skip = int(argv[i + 1]); i += 2
        else:
            print("REFUSE: unknown argument %r" % a)
            return 1
    if mode is None:
        print("REFUSE: pick a mode")
        return 1

    tool_line, blob = tool_line_for(image)
    if tool_line is None:
        print("REFUSE: sha256 %s is not the pinned image %s"
              % (hashlib.sha256(blob).hexdigest()[:16], PINNED_SHA256[:16]))
        return 1
    print("image OK: %s (%d B)" % (PINNED_SHA256[:16], len(blob)))
    print("corpus fence: %r" % tool_line)
    print()

    if mode == "show":
        return show(target)

    if not os.path.isdir(cache):
        print("SKIP: no capture cache at %s" % cache)
        return 0

    if mode == "selftest":
        # The ported digest must reproduce what a real entry.bin recorded, or
        # the fence is fencing nothing.  One entry is enough and the failure is
        # loud.
        n = 0
        for dirpath in iter_entries(cache, 3, 0):
            head, _ = parse_entry(os.path.join(dirpath, "entry.bin"))
            if tool_line not in head:
                print("FAIL: %s does not record %r" % (dirpath, tool_line))
                return 1
            n += 1
        if n == 0:
            print("VACUOUS: no entries to check the fence against")
            return 2
        print("PASS: the ported digest128 reproduces the recorded tool line on"
              " %d entries" % n)
        return 0

    c = run(cache, limit, skip, tool_line)

    # REFUTATION / VACUITY ORDER.  A corpus with zero decodable code is a
    # statement about the scan; say so before saying anything about c2.
    if c.nobj == 0:
        print("VACUOUS: %d objs measured (%d foreign).  NOT a pass."
              % (c.nobj, c.nforeign))
        return 2
    if c.bl_total == 0 and c.nonself_words == 0:
        print("VACUOUS: the scan decoded no instruction at all.  NOT a pass.")
        return 2
    if c.nself == 0:
        print("NO SELF-MOVES in this slice -- %d objs, %d words decoded."
              % (c.nobj, c.nonself_words))
        print("That is a measurement (the denominator is non-zero), but it")
        print("characterises nothing.  Widen --limit or move --skip.")
        return 3

    {"census": report_census, "sources": report_sources,
     "control": report_control}[mode](c)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
