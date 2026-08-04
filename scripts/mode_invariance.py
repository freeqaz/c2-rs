#!/usr/bin/env python3
"""**How much of the flag axis can the generated corpus actually see?**

`scripts/expr_sweep.sh` grades 14,635 generated cases at exactly one profile
(`/Ox /GS- /c`, hardcoded in `c2rs diff`). `scripts/gate.sh` grades 228
hand-written fixtures at the 12 profiles in `scripts/lanes.txt`. Their product
is ungraded, and **both wrong emits found on 2026-08-04 live in it** — board
#232 needed an implicit destructor *and* the packed path, and w-order's Y-a
needed an empty-bodied unwind target *and* `/EHsc`, live at `/O1 /EHsc` and
invisible at `/Ox`.

Crossing them naively is 14,635 x 12 = 175,620 gradings per gate run. Before
paying that, measure how much of it is *not redundant*.

# The redundancy criterion — a proof, not an observation

For one case `c` and two profiles `p`, `q`:

* The **port's** whole input is `(IlBundle, obj_name, gy)`. It never reads the
  cl flags: `gap.rs` derives `gy` with
  `PortC2::flags_imply_function_level_linking` and hands it in, and the
  optimization mode arrives *inside* the IL as `IlFunction::opt_word`.
  `obj_name` comes from the source name and is equal at both profiles.
* The **reference's** output is measured here directly, at a **fixed** `-Fo`
  path so that the `S_OBJNAME` c2 bakes into `.debug$S` is constant and the two
  objs are byte-comparable.

so

    IL(c,p) == IL(c,q)  and  gy(p) == gy(q)  and  refobj(c,p) == refobj(c,q)
        ==>  grading c at q after p establishes NOTHING.

Both halves are needed and neither alone is sound:

* IL alone is not: `/Ox /Gy` and `/Ox` produce **identical IL** (the front end
  does not encode `/Gy`) and different objs, so an IL-only key would call them
  redundant while c2 lays the sections out differently.
* the obj alone is not: two profiles can agree on c2's output while handing the
  *port* different IL, and the port is the thing under test.

`scripts/lanes.txt` already records the weaker trap this avoids: `/O1 /EHsc` and
`/O1` differ in **0 verdict rows** over the fixture corpus while producing
genuinely different objs. **Verdict-identical is not redundant.**

# Direction of the risk

A fragment excluded as invariant is a fragment nothing will ever grade again at
the excluded profiles. So the exclusion is never a written-down constant: it is
re-derived by running this script, and `scripts/mode_cross.sh` re-derives it on
a sample of every fragment on every run, so a fragment whose collapse *changed*
(a new case, an edited generator) is reported rather than quietly trusted.

# Instrument self-check

`--verify K` re-captures every Kth cell and requires both hashes to reproduce.
This is the control that can go red: if any run-varying byte (a temp path, a
timestamp) leaked into either hash, every profile would look distinct, the
script would report "nothing collapses", and that reads as a maximally
conservative result while being pure noise. It is on by default.

Usage:

    scripts/mode_invariance.py --out work/w-modes/inv                 # 8/fragment
    scripts/mode_invariance.py --out DIR --per-fragment 25 --jobs 8
    scripts/mode_invariance.py --out DIR --only 63-emit               # one fragment

Writes `<out>/cells.tsv` (one row per case x lane) and prints the per-fragment
class table plus the corpus-wide cost consequence. Without a toolchain it prints
`SKIP: toolchain absent` and exits 0.
"""

import argparse
import hashlib
import os
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import sweep_gen  # noqa: E402  (the ONE fragment loader; see its module docs)

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
IL_SUFFIXES = ("ex", "gl", "sy", "in", "db")

# `PortC2::flags_imply_function_level_linking` (crates/c2-core/src/lib.rs), the
# ONE port input that does not arrive through the IL. This is a mirror of that
# predicate and mirrors drift; it is used only to make the partition FINER (two
# lanes with different `gy` are never merged), so drift can cost coverage and
# cannot create a false merge in the other direction — every merge still requires
# the IL *and* the reference obj to be byte-identical as well.
GY_FLAGS = {"/gy", "-gy", "/o1", "-o1", "/o2", "-o2"}


def gy_of(flags):
    return any(f.lower() in GY_FLAGS for f in flags)


def read_registry(path):
    """`[(slug, [flag, ...]), ...]` — the same parse `gate.sh` does."""
    lanes = []
    with open(path) as fh:
        for line in fh:
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            parts = line.split()
            if len(parts) < 2:
                raise SystemExit(
                    "registry row %r carries a slug and no flags; gate.sh calls "
                    "this fatal and so does this" % line
                )
            lanes.append((parts[0], parts[1:]))
    if not lanes:
        raise SystemExit("registry %s defines no lanes" % path)
    slugs = [s for s, _ in lanes]
    if len(set(slugs)) != len(slugs):
        raise SystemExit("duplicate lane slug in %s" % path)
    return lanes


def fragment_digest(srcs):
    """Digest of one fragment's generated case set, in emission order.

    This is what makes the exclusion safe rather than merely current: a row in
    `mode_classes.txt` applies to the EXACT cases its measurement ran on. Edit a
    generator and the digest moves, the row stops applying, and that fragment
    falls back to all 12 lanes — loudly. The alternative, re-deriving on a
    sample, only catches staleness the sample happens to hit.
    """
    hh = hashlib.sha256()
    hh.update(b"c2rs-mode-classes/v1\x00")
    for s in srcs:
        hh.update(("%d\x00" % len(s)).encode())
        hh.update(s.encode())
    return hh.hexdigest()[:16]


def read_classes(path):
    """`{fragment: ([lane, ...], digest)}` from `scripts/mode_classes.txt`.

    One reader for the table — `scripts/mode_cross.sh` reaches it through
    `--assign`, never with a second parse. Two parses of one file is the "one
    rule, two implementations" shape `docs/GAPS.md` §6 keeps recording.
    """
    out = {}
    with open(path) as fh:
        for line in fh:
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            parts = line.split()
            if len(parts) != 3:
                raise SystemExit(
                    "malformed row in %s: %r (want `<fragment> <lanes> <digest>`)"
                    % (path, line))
            sl = [s for s in parts[1].split(",") if s]
            if not sl:
                raise SystemExit("row %s in %s names no lanes" % (parts[0], path))
            out[parts[0]] = (sl, parts[2])
    if not out:
        raise SystemExit("%s records no fragments" % path)
    return out


def h(data):
    return hashlib.sha256(data).hexdigest()[:16]


def obj_hash(path):
    """Hash the obj with the COFF `TimeDateStamp` (offset 4..8) zeroed — the
    same normalization the correctness rule uses (CLAUDE.md)."""
    with open(path, "rb") as fh:
        b = bytearray(fh.read())
    if len(b) >= 8:
        b[4:8] = b"\0\0\0\0"
    return h(bytes(b))


def il_hash(d):
    """Hash the bundle's contents in fixed suffix order. The bundle's *base
    name* is a per-run scratch nonce and is deliberately not hashed; its
    *contents* are byte-stable across runs (verified by `--verify`)."""
    names = sorted(os.listdir(d))
    parts = []
    for suf in IL_SUFFIXES:
        hit = [n for n in names if n.endswith("." + suf)]
        if not hit:
            parts.append(b"<absent>")
            continue
        with open(os.path.join(d, hit[0]), "rb") as fh:
            parts.append(fh.read())
    return h(b"\x00\x01".join(parts))


CLASSES_HEADER = """\
# THE PROFILE-CLASS TABLE — generated. Do not hand-edit.
#
#     scripts/mode_invariance.py --out DIR --per-fragment N --write-classes \\
#         scripts/mode_classes.txt
#
# One row per `scripts/sweep.d/` fragment: the lane slugs `scripts/mode_cross.sh`
# has to grade that fragment at. Every lane NOT listed produced a byte-identical
# IL bundle, a byte-identical reference obj and the same `gy` as one that is, at
# every sampled case — so grading it is the same computation twice. See
# `scripts/mode_invariance.py` for why those three together are a proof and why
# no two of them are.
#
# A fragment with NO ROW HERE is graded at ALL %d lanes. That is the fail-safe
# direction and it is deliberate: adding a `sweep.d/` fragment then costs more
# than it needs to and never less, and nothing is silently excluded by being
# forgotten. Run this script to buy the reduction back.
#
# THE THIRD FIELD IS A DIGEST of that fragment's generated case set, and it is
# what makes the exclusion safe rather than merely current. A row applies only
# while the digest matches; edit the generator and the row stops applying and the
# fragment falls back to all 12 lanes, loudly. A stale exclusion is the one
# failure mode that matters here — a fragment excluded as invariant is a fragment
# nothing will ever grade again at the excluded lanes — and this closes it by
# construction rather than by re-sampling and hoping the sample hits it.
# (`--check` still re-derives against a live measurement when you want one.)
#
# Measured: %d cases per fragment (strided), %d cases, %d cells, registry %s.
#   full cross          %7d gradings
#   class-reduced cross %7d gradings  (%.2fx smaller)
#
# format:  <fragment>  <lane>[,<lane>...]  <case-set digest>  # <equivalence classes>
"""


class Cell:
    __slots__ = ("frag", "case", "slug", "il", "obj", "gy", "err")

    def __init__(self, frag, case, slug, il, obj, gy, err):
        self.frag, self.case, self.slug = frag, case, slug
        self.il, self.obj, self.gy, self.err = il, obj, gy, err


def capture_cell(c2rs, src, out_root, tag, slug, flags, keep=False):
    """One (case, lane) cell: `(il_hash, obj_hash, error_or_None)`.

    The obj path is a function of the CASE only, never of the lane, so the
    `-Fo` string c2 bakes into `.debug$S` is identical at all 12 profiles and
    the objs compare byte-for-byte. That is why this cannot use `c2rs compile`,
    whose scratch directory differs per invocation.
    """
    objdir = os.path.join(out_root, "obj")
    ildir = os.path.join(out_root, "il", "%s.%s" % (tag, slug))
    os.makedirs(objdir, exist_ok=True)
    if os.path.isdir(ildir):
        for n in os.listdir(ildir):
            os.unlink(os.path.join(ildir, n))
    os.makedirs(ildir, exist_ok=True)
    objp = os.path.join(objdir, tag + ".obj")

    env = dict(os.environ)
    env["GT_OUT"] = objp
    env["WIBO_FS_CACHE"] = "1"
    r = subprocess.run(
        [os.path.join(REPO, "scripts/gt_capture.sh"), src] + list(flags) + ["/GS-", "/c"],
        capture_output=True, env=env,
    )
    if not os.path.exists(objp):
        return None, None, "no obj at %s (%s)" % (slug, r.stderr.decode("utf-8", "replace")[-160:])
    oh = obj_hash(objp)

    fl = os.path.join(ildir, "flags.txt")
    with open(fl, "w") as fh:
        fh.write(" ".join(list(flags) + ["/GS-", "/c"]) + "\n")
    r = subprocess.run(
        [c2rs, "capture", src, "--flags-file", fl, "--keep-il", ildir],
        capture_output=True, env=env,
    )
    ih = None
    if any(n.endswith(".ex") for n in os.listdir(ildir)):
        ih = il_hash(ildir)
    if not keep:
        for n in os.listdir(ildir):
            os.unlink(os.path.join(ildir, n))
        os.rmdir(ildir)
        os.unlink(objp)
    if ih is None:
        return None, oh, "no IL bundle at %s (%s)" % (
            slug, r.stderr.decode("utf-8", "replace")[-160:])
    return ih, oh, None


def assign(args):
    """Write per-lane case lists from `scripts/mode_classes.txt`. No toolchain.

    A fragment with no row is assigned **every** lane — the fail-safe direction,
    so a newly added `sweep.d/` fragment is over-graded rather than skipped.
    """
    lanes = read_registry(args.registry)
    all_slugs = [s for s, _ in lanes]
    table = read_classes(args.classes)
    for frag, (sl, _dg) in table.items():
        bad = [s for s in sl if s not in all_slugs]
        if bad:
            raise SystemExit(
                "%s: fragment %s names lane(s) %s that are not in %s"
                % (args.classes, frag, ",".join(bad), args.registry))

    # The live digest of every fragment, from the SAME loader that generated the
    # cases. A row applies only when it matches.
    live = {stem: fragment_digest(srcs)
            for stem, srcs in sweep_gen.load_all(args.frag_dir)}

    cases_dir = os.path.abspath(args.assign)
    out = os.path.abspath(args.assign_out or os.path.join(cases_dir, "lanes"))
    os.makedirs(out, exist_ok=True)
    for n in os.listdir(out):
        if n.endswith(".list"):
            os.unlink(os.path.join(out, n))

    per = {s: [] for s in all_slugs}
    cases = sorted(n for n in os.listdir(cases_dir) if n.endswith(".cpp"))
    if not cases:
        raise SystemExit("no generated cases in %s" % cases_dir)
    no_row = set()
    stale = set()
    for n in cases:
        frag = n.rsplit("-", 1)[0]
        row = table.get(frag)
        if row is None:
            no_row.add(frag)
            sl = all_slugs
        elif live.get(frag) != row[1]:
            stale.add(frag)
            sl = all_slugs
        else:
            sl = row[0]
        for s in sl:
            per[s].append(os.path.join(cases_dir, n))

    cells = 0
    for s in all_slugs:
        # `cl.exe` runs under wibo, so the sources are named as `Z:\…` paths.
        with open(os.path.join(out, "%s.list" % s), "w") as fh:
            for p in per[s]:
                fh.write("z:%s\n" % p.replace("/", "\\"))
        cells += len(per[s])
    print("assigned %d cases over %d lanes = %d cells (full cross would be %d)"
          % (len(cases), len(all_slugs), cells, len(cases) * len(all_slugs)))
    rel = os.path.relpath(args.classes, REPO)
    if no_row:
        print("  %d fragment(s) have NO ROW in %s and are graded at ALL %d lanes:"
              % (len(no_row), rel, len(all_slugs)))
        for f in sorted(no_row):
            print("      %s" % f)
    if stale:
        print("  %d fragment(s) have a row in %s whose case DIGEST no longer matches;"
              % (len(stale), rel))
        print("  the generator changed under the measurement, so the row does not apply")
        print("  and they are graded at ALL %d lanes:" % len(all_slugs))
        for f in sorted(stale):
            print("      %-27s table %s  live %s"
                  % (f, table[f][1], live.get(f, "<gone>")))
        print("  Re-measure to buy the reduction back:")
        print("    scripts/mode_invariance.py --out DIR --per-fragment 24 \\")
        print("        --write-classes %s" % rel)
    for s in all_slugs:
        print("  %-14s %6d cases" % (s, len(per[s])))
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="")
    ap.add_argument("--per-fragment", type=int, default=8)
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--verify", type=int, default=7,
                    help="re-capture every Kth cell and require it to reproduce (0 = off)")
    ap.add_argument("--registry", default=os.path.join(REPO, "scripts/lanes.txt"))
    ap.add_argument("--only", default="", help="fragment-name substring filter")
    ap.add_argument("--frag-dir", default=os.path.join(REPO, "scripts/sweep.d"))
    ap.add_argument("--check", default="",
                    help="re-derive against an existing scripts/mode_classes.txt and "
                         "FAIL if what is observed is finer than what is recorded")
    ap.add_argument("--write-classes", default="",
                    help="write the per-fragment representative lane table "
                         "(scripts/mode_classes.txt) that mode_cross.sh reads")
    ap.add_argument("--assign", default="",
                    help="ASSIGNMENT MODE (no toolchain): given a directory of "
                         "generated cases, write one `<slug>.list` per lane into "
                         "--assign-out naming the cases that lane must grade")
    ap.add_argument("--assign-out", default="")
    ap.add_argument("--classes", default=os.path.join(REPO, "scripts/mode_classes.txt"))
    args = ap.parse_args()

    if args.assign:
        return assign(args)
    if not args.out:
        raise SystemExit("--out is required (or use --assign)")

    out = os.path.abspath(args.out)
    cases_dir = os.path.join(out, "cases")
    os.makedirs(cases_dir, exist_ok=True)
    for n in os.listdir(cases_dir):
        if n.endswith(".cpp"):
            os.unlink(os.path.join(cases_dir, n))

    lanes = read_registry(args.registry)
    total_cases = sweep_gen.write_cases(cases_dir, args.frag_dir, args.only, quiet=True)

    c2rs = os.environ.get("C2RS_BIN") or os.path.join(REPO, "target/release/c2rs")
    if not os.path.exists(c2rs):
        raise SystemExit("no harness binary at %s (cargo build --release)" % c2rs)

    # Group the generated cases by fragment, then take a STRIDE — never a
    # prefix. `head -n` over a name-sorted list is what made a 400-case budget
    # cover 1 of 47 fragments (see `expr_sweep.sh`), and the same trap applies
    # one level in: a prefix of one fragment's cases is its first axis value and
    # nothing else.
    byfrag = {}
    for n in sorted(os.listdir(cases_dir)):
        if not n.endswith(".cpp"):
            continue
        byfrag.setdefault(n.rsplit("-", 1)[0], []).append(n)
    picked = []
    for frag in sorted(byfrag):
        cs = byfrag[frag]
        k = max(1, (len(cs) + args.per_fragment - 1) // args.per_fragment)
        sel = cs[::k][: args.per_fragment]
        picked.extend((frag, c) for c in sel)

    print("registry: %d lanes from %s" % (len(lanes), args.registry))
    print("corpus:   %d fragments, %d generated cases" % (len(byfrag), total_cases))
    print("sampled:  %d cases (stride, <=%d per fragment) x %d lanes = %d cells"
          % (len(picked), args.per_fragment, len(lanes), len(picked) * len(lanes)))

    # Toolchain probe. Absent -> SKIP, exit 0 (CLAUDE.md); never a vacuous pass.
    probe = os.path.join(out, "probe")
    os.makedirs(probe, exist_ok=True)
    ih, oh, err = capture_cell(c2rs, os.path.join(cases_dir, picked[0][1]), probe,
                               "probe", lanes[0][0], lanes[0][1])
    if ih is None or oh is None:
        print("SKIP: toolchain absent — the measurement would be vacuous (%s)" % err)
        return 0

    lock = threading.Lock()
    rows = []
    failures = []
    verify_bad = []
    verify_done = [0]
    attempted = [0]

    def do_case(idx_frag_case):
        idx, (frag, case) = idx_frag_case
        src = os.path.join(cases_dir, case)
        tag = case[:-4]
        local = []
        for j, (slug, flags) in enumerate(lanes):
            ih, oh, err = capture_cell(c2rs, src, out, tag, slug, flags)
            local.append(Cell(frag, case, slug, ih, oh, gy_of(flags), err))
            if args.verify and ((idx * len(lanes) + j) % args.verify == 0):
                ih2, oh2, _ = capture_cell(c2rs, src, out, tag, slug, flags)
                with lock:
                    verify_done[0] += 1
                    if (ih2, oh2) != (ih, oh):
                        verify_bad.append((case, slug, ih, ih2, oh, oh2))
        with lock:
            attempted[0] += len(lanes)
            for c in local:
                rows.append(c)
                if c.err:
                    failures.append(c)

    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        list(ex.map(do_case, enumerate(picked)))

    # ---- positive counts first. An absent row must never read as agreement. ----
    want = len(picked) * len(lanes)
    print()
    print("cells attempted %d, recorded %d, capture failures %d"
          % (attempted[0], len(rows), len(failures)))
    if len(rows) != want or attempted[0] != want:
        print("FATAL: expected %d cells, attempted %d, recorded %d — a cell that was"
              % (want, attempted[0], len(rows)))
        print("  never captured is not a cell that agreed.")
        return 3
    # A case the reference REJECTS at every profile is not a measurement failure —
    # it is a generated case `cl.exe` will not compile, which is a finding of its
    # own (`expr_sweep.sh` grades such a case as clean; see the w-modes rung). It
    # is named, counted and excluded. A case rejected at SOME profiles and not
    # others is flag-dependent compilability and stays FATAL: that is a real
    # difference across the axis this whole script is measuring.
    uncompilable = set()
    if failures:
        byc = {}
        for c in failures:
            byc.setdefault(c.case, []).append(c)
        partial = {k: v for k, v in byc.items() if len(v) != len(lanes)}
        uncompilable = {k for k, v in byc.items() if len(v) == len(lanes)}
        if uncompilable:
            print("UNCOMPILABLE at every one of the %d profiles — %d case(s), excluded:"
                  % (len(lanes), len(uncompilable)))
            for k in sorted(uncompilable):
                print("    %s   %s" % (k, byc[k][0].err.split("\n")[1][:110]
                                       if "\n" in byc[k][0].err else byc[k][0].err[:110]))
        if partial:
            for k, v in list(partial.items())[:10]:
                print("  CAPTURE-FAIL %s at %d/%d profiles: %s"
                      % (k, len(v), len(lanes), v[0].err))
            print("FATAL: %d case(s) captured at some profiles and not others. That is"
                  % len(partial))
            print("  either a real flag-dependent compile failure or a broken cell;")
            print("  either way no partition below would be trustworthy.")
            return 3
        rows = [c for c in rows if c.case not in uncompilable]
        picked = [p for p in picked if p[1] not in uncompilable]
    if args.verify:
        print("verify: %d cells re-captured, %d disagreed" % (verify_done[0], len(verify_bad)))
        if verify_bad:
            for v in verify_bad[:5]:
                print("  UNSTABLE %s %s  il %s/%s  obj %s/%s" % v)
            print("FATAL: a cell did not reproduce. Some run-varying byte is in the")
            print("  hash, which makes every profile look distinct and the whole")
            print("  measurement read 'nothing collapses' — noise wearing the shape")
            print("  of a conservative result.")
            return 3
        if verify_done[0] == 0:
            print("FATAL: --verify was on and re-captured nothing.")
            return 3

    with open(os.path.join(out, "cells.tsv"), "w") as fh:
        fh.write("fragment\tcase\tlane\tgy\til\tobj\n")
        for c in rows:
            fh.write("%s\t%s\t%s\t%d\t%s\t%s\n" % (c.frag, c.case, c.slug, c.gy, c.il, c.obj))

    # ---- the partition ----------------------------------------------------------
    bycase = {}
    for c in rows:
        bycase.setdefault(c.case, {})[c.slug] = (c.il, c.obj, c.gy)

    frag_classes = {}     # fragment -> [partition per sampled case]
    frag_refine = {}      # fragment -> the COMMON REFINEMENT of those partitions
    frag_maxk = {}
    frag_agree = {}
    slugs = [s for s, _ in lanes]
    for frag in sorted(byfrag):
        if not any(c.frag == frag for c in rows):
            continue
        sigs = []
        for case in sorted({c.case for c in rows if c.frag == frag}):
            m = bycase[case]
            groups = {}
            for slug in slugs:
                groups.setdefault(m[slug], []).append(slug)
            sigs.append(frozenset(frozenset(v) for v in groups.values()))
        # Two lanes are equivalent for this fragment only if they were equivalent
        # in EVERY sampled case. Taking the coarsest case's partition instead
        # would drop a lane pair that one case separates — the exclusion error
        # that runs in the unsafe direction, since an excluded lane is one
        # nothing grades again.
        key = {}
        for slug in slugs:
            key[slug] = tuple(
                tuple(sorted(g))
                for sig in sigs
                for g in [next(x for x in sig if slug in x)]
            )
        ref = {}
        for slug in slugs:
            ref.setdefault(key[slug], []).append(slug)
        frag_refine[frag] = [sorted(v) for v in ref.values()]
        frag_classes[frag] = sigs
        frag_maxk[frag] = len(ref)
        frag_agree[frag] = len(set(sigs)) == 1

    nlanes = len(lanes)
    print()
    print("FRAGMENT                     cases  classes(min..max)  cases agree?  representative partition")
    print("---------------------------  -----  -----------------  ------------  ------------------------")
    tot_reps = 0
    tot_cases_corpus = 0
    reps_of = {}
    for frag in sorted(frag_classes):
        sigs = frag_classes[frag]
        ks = [len(s) for s in sigs]
        rep = sorted(g[0] for g in frag_refine[frag])
        reps_of[frag] = rep
        print("%-27s  %5d  %6d..%-10d  %-12s  %s"
              % (frag, len(sigs), min(ks), max(ks),
                 "yes" if frag_agree[frag] else "NO", ",".join(rep)))
        tot_reps += len(rep) * len(byfrag[frag])
        tot_cases_corpus += len(byfrag[frag])

    ks = [frag_maxk[f] for f in frag_classes]
    ks_sorted = sorted(ks)
    print()
    print("classes over %d lanes: min %d, median %d, max %d; %d of %d fragments collapse at all"
          % (nlanes, ks_sorted[0], ks_sorted[len(ks_sorted) // 2], ks_sorted[-1],
             sum(1 for k in ks if k < nlanes), len(ks)))
    print("fragments whose sampled cases DISAGREE about their partition: %d"
          % sum(1 for f in frag_classes if not frag_agree[f]))
    print("sampled cases the reference REJECTS at every profile: %d" % len(uncompilable))
    print()
    print("cost consequence, extrapolated over the whole corpus:")
    print("  full cross          %7d gradings (%d cases x %d lanes)"
          % (tot_cases_corpus * nlanes, tot_cases_corpus, nlanes))
    print("  class-reduced cross %7d gradings (%.2fx smaller)"
          % (tot_reps, (tot_cases_corpus * nlanes) / float(max(1, tot_reps))))
    print("  single profile      %7d gradings (what expr_sweep.sh does today)"
          % tot_cases_corpus)

    # ---- the staleness guard ---------------------------------------------------
    #
    # `mode_cross.sh` calls this on a small stride every run. The failure that
    # matters runs in ONE direction: the recorded table says two lanes are
    # equivalent, they are not, and the cross has stopped grading one of them.
    # So a partition FINER than the record is fatal; a coarser one is only a note
    # (the table is over-grading, which costs time and never coverage).
    if args.check:
        rec = read_classes(args.check)
        finer = []
        coarser = []
        for frag, reps in sorted(reps_of.items()):
            want = set(rec[frag][0] if frag in rec else [s for s, _ in lanes])
            got = set(reps)
            missing = got - want
            if missing:
                finer.append((frag, sorted(missing), sorted(want)))
            elif want - got:
                coarser.append((frag, sorted(want - got)))
        print()
        print("staleness check against %s" % args.check)
        for frag, extra in coarser:
            print("  note   %-27s table grades %s that this sample does not separate"
                  % (frag, ",".join(extra)))
        for frag, missing, want in finer:
            print("  STALE  %-27s observed a split the table does not carry: %s"
                  % (frag, ",".join(missing)))
            print("         table: %s" % ",".join(want))
        if finer:
            print()
            print("FATAL: %d fragment(s) separate lanes the table calls equivalent. The"
                  % len(finer))
            print("  cross is NOT grading those lanes, and a lane excluded as invariant")
            print("  is a lane nothing else grades either. Regenerate:")
            print("    scripts/mode_invariance.py --out DIR --per-fragment 24 \\")
            print("        --write-classes %s" % args.check)
            return 4
        print("  OK — %d fragments, no observed split is missing from the table."
              % len(reps_of))

    if args.write_classes:
        digests = {stem: fragment_digest(srcs)
                   for stem, srcs in sweep_gen.load_all(args.frag_dir)}
        with open(args.write_classes, "w") as fh:
            fh.write(CLASSES_HEADER % (
                len(lanes), args.per_fragment, len(picked), len(rows),
                args.registry.replace(REPO + "/", ""),
                tot_cases_corpus * nlanes, tot_reps,
                (tot_cases_corpus * nlanes) / float(max(1, tot_reps))))
            for frag in sorted(reps_of):
                groups = " ".join(
                    "+".join(g) for g in sorted(frag_refine[frag], key=lambda g: g[0]))
                fh.write("%-27s %-44s %s  # %s\n"
                         % (frag, ",".join(reps_of[frag]), digests[frag], groups))
        print()
        print("wrote %s (%d fragments)" % (args.write_classes, len(reps_of)))

    # ---- per-lane-pair: which flag axis actually buys anything ----------------
    print()
    print("PAIRWISE: fragments (of %d) on which the two lanes differ, by axis" % len(frag_classes))
    def pair_report(a, b):
        n_il = n_obj = n_any = 0
        for frag in frag_classes:
            cs = sorted({c.case for c in rows if c.frag == frag})
            dil = dobj = False
            for case in cs:
                m = bycase[case]
                if m[a][0] != m[b][0]:
                    dil = True
                if m[a][1] != m[b][1]:
                    dobj = True
            n_il += dil
            n_obj += dobj
            n_any += (dil or dobj)
        print("  %-14s vs %-14s  IL differs %2d   obj differs %2d   either %2d"
              % (a, b, n_il, n_obj, n_any))

    have = {s for s, _ in lanes}
    for a, b in (("O1", "O2"), ("O1", "Ox"), ("O1", "Od"), ("O1", "O1-Oi"),
                 ("Ox", "Ox-Gy"), ("O1", "O1-EHsc"), ("Ox", "Ox-EHsc"),
                 ("O2", "O2-EHsc"), ("Od", "Od-EHsc"), ("Ox-Gy", "Ox-Gy-EHsc"),
                 ("O1-Oi", "O1-Oi-EHsc")):
        if a in have and b in have:
            pair_report(a, b)
    return 0


if __name__ == "__main__":
    sys.exit(main())
