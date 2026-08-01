#!/usr/bin/env python3
"""**The cross-product lane** — grade every accepted shape family *beside* every
other one, mechanically.

Run it through `scripts/cross_sweep.sh`, which builds `c2rs` first.

Why this exists
---------------
`docs/GAPS.md` §6 #12: two branches were each fully green — an FP-store rung and
a many-call framed rung — and the *merge* mis-emitted. A compiler-label counter
read from a per-function method that was carrying a per-TU fact, so a framed
function downstream came out six bytes wrong in an obj that still links.
**Neither branch's corpus could contain the case**: the FP rung's fixtures have
no framed function and the framed rung's have no floating point, and the label
counter has an observable effect only when a framed function follows. It existed
only in the merge. #13 then found that the *repair* was also wrong one row
further out, because a per-function quantity and a per-TU one are
indistinguishable at n = 1.

The rule those two wrote down is: **a merge of two independently-green branches
is a new corpus, and the shapes only it contains have never been graded by
anyone.** Until now that grading was manual and depended on whoever was merging
thinking of it. This lane makes it mechanical.

What it grades, and what it does not
------------------------------------
* **Families are asked of the port, not listed here.** The accepted shape
  families are the `FnVerdict::InClass("…")` labels in
  `crates/c2-il/src/func/census.rs` — the port's own classifier — extracted by a
  paren-matched scan so a family added by a future rung is picked up without
  anyone editing this file. A hand-written list is exactly what drifts.
* **Representatives are discovered by compiling, not written down — and they are
  keyed on SHAPE, not on the family's name.** The whole `scripts/sweep.d/`
  corpus is generated and graded; a candidate is a *matched* TU whose in-class
  functions are all of one family; every candidate's obj is emitted and its
  **shape** read out of it — the masked opcode sequence of `.text`, registers
  and immediates removed. A family's representatives are then one per *distinct
  shape*, most-populated bucket first, capped at `C2RS_CROSS_REPS` (8).

  Keying on the label was a real hole and it was measured: a rung landing
  **+5,507 functions of new accepted shape** produced numbers byte-identical to
  the pre-merge run, because the new shapes were absorbed into the existing
  `multiarg-tail-call` label and the three label-sampled representatives all
  came from older fragments. Under the shape key, `multiarg-tail-call` has
  **8** distinct emitted shapes and all 8 are crossed. A family with **no**
  representative is a hole in the sweep corpus and this lane **fails**, by name
  (that is how `call-sequence`, `call-sequence-value` and `call-sequence-lit`
  were found to have no single-function case anywhere, which is what
  `scripts/sweep.d/71-call-sequence.py` fixes).
* **Tier A — the pairwise cross.** Every ordered pair of representatives,
  *including both orders and the diagonal*, at **every lane in
  `scripts/lanes.txt`**. Order matters (the label counter is consumed in `.ex`
  order and `_fltused` is placed after the *first* FP-touching function), so
  both orders are separate configurations, not one.
* **Tier B — the arity axis.** #13's rule is "one slot per function plus one for
  the TU if anything touches floating point", and at n = 1 that is
  indistinguishable from "two slots per FP function". So each family is also
  graded at n = 1, 2, 3, 4 copies of itself, alone and with a framed observer
  before and after it — a framed function is the only thing that *renders* the
  counter, so a TU without one cannot grade it however many copies it has.
* **Tier C — triples over the external-bearing families.** These are the
  families most likely to disturb the compiler-label counter, so they are
  crossed three deep in all orders and again with a stride-1 integer leaf
  inserted at each of the four positions — a counter error an adjacent function
  absorbs is invisible without a separator. Pairs reach n = 2, which is the
  smallest n at which a per-function rule and a per-TU one can disagree at all;
  the triples reach n = 3. The selector is a HEURISTIC and not a rule: "one slot
  per TU-level external" was refuted (`docs/LABEL_COUNTER.md` §2.1) and the
  measured model is a per-function surcharge table (§1.1), which charges +2 for
  a newly pooled FP constant that mints no external and 0 for a string literal
  that mints one. See the module's `docs/CROSS_PRODUCT.md` for what that leaves
  unselected.
* **Tiers S and W — the wrapping check and its control.** Every representative
  is compiled ALONE (S) and ALONE INSIDE A NAMESPACE (W), at every lane. If a
  namespace by itself pushed a shape out of class, the whole lane would grade
  refusals and report a green that means nothing. The alarm is the *difference*
  — W refuses where S matched — because `/Od` is in the registry and refuses
  both, on purpose.
* **The mode lanes are `scripts/lanes.txt`, not a list in this file.** Until
  2026-08-01 this file carried its own four (packed, `/Gy`, `/O1`, `/O2`), which
  compile **no `/EH` at any invocation**, on a workload whose every TU is
  compiled `/EHsc` and which has 35,964 in-class `eh-bare` functions whose
  markers appear only under it. That was the last surviving instance of the
  un-enumerated-lane defect, in the one instrument that exists to find
  cross-shape mis-emits.
* **Deliberately NOT graded**, and this is not a footnote:
  - **triples of three distinct NON-external families.** Tier C is restricted to
    the families whose representative carries a TU-level external; the full
    `R³` is not run, so a three-way interaction among plain leaves would not be
    caught.
  - **the shapes beyond the cap.** A family is represented by up to
    `C2RS_CROSS_REPS` (8) of its distinct measured shapes. Families with more
    are named in the run's output with the count they left out — `store-run`
    has 173 measured shapes and gets 8 — so the residue is a number, not a
    silence. Operand order, widths and offsets *within* one shape are swept by
    the per-axis fragments and are still not crossed here.
  - **flags beyond the registry** (`/GS`, `/GR`, `/Zi`, and every combination
    of what is there).
  - **any pair the port refuses at the TU level.** Those are compiled and
    counted and named, but no bytes were compared, so they are *ungraded*, not
    green. The FP-beside-framed pair is currently in this set — which is #13's
    "a gate that hides a wrong rule is a debt, not a fix", still outstanding.

A MISMATCH here is an ALARM: the port emitted bytes for a combination and they
were wrong. `NotImplemented` / `vocab-gap` is an honest refusal. `capture-fail`
is neither: it is the toolchain producing no reference, and the lane exits
non-zero on it rather than counting it as a graded configuration.
"""

import json
import os
import re
import struct
import subprocess
import sys
import threading

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
C2RS = os.environ.get("C2RS_BIN") or os.path.join(REPO, "target/release/c2rs")
JOBS = os.environ.get("C2RS_JOBS", "16")
NREPS = int(os.environ.get("C2RS_CROSS_REPS", "8"))
CENSUS_RS = os.path.join(REPO, "crates/c2-il/src/func/census.rs")
LANES = os.environ.get("C2RS_LANES") or os.path.join(REPO, "scripts/lanes.txt")

# The profile the representative pool is DISCOVERED at — the same `/Ox /GS- /c`
# that `c2rs diff` and `scripts/expr_sweep.sh` hardcode, and the same profile the
# shape signatures below are measured at. It is one profile because discovery has
# to be reproducible; it is asserted to be one of the registry's own lanes
# (`assert_discovery_is_a_lane`), so the representatives are never chosen at a
# configuration that nothing then grades.
DISCOVERY_FLAGS = ["/Ox", "/GS-", "/c"]

# Every TU class `c2rs gap` can report that means BYTES WERE COMPARED, plus the
# port's honest refusals. `capture-fail` is deliberately absent: it is the
# toolchain failing to produce a reference at all, which is an instrument failure
# and must never be counted as a graded configuration.
GRADED = ("match", "mismatch", "codegen-gap", "vocab-gap", "port-error")

# Markers a *translation unit* carries because some function in it needed them.
# Used only to CHOOSE tier-C representatives: three of the four are surcharges in
# the measured compiler-label model (`docs/LABEL_COUNTER.md` §1.1 — `_fltused`
# +1 on the first FP-touching function, each distinct helper width +2, `.pdata`
# marking the framed base), and that counter is the mechanism behind every bug
# this lane exists to find. It is a heuristic, not a derivation: "one slot per
# TU-level external" is REFUTED (§2.1 — a newly pooled FP constant costs +2 and
# mints no external; a string literal mints one and costs 0). Read out of the
# representative's own obj, never assumed from the family's name.
TU_EXTERNALS = (b"_fltused", b"__savegprlr", b"__restgprlr", b".pdata")


def wibo_path(p):
    """`/tmp/x/y.cpp` -> `z:\\tmp\\x\\y.cpp`, the form `cl.exe` under wibo takes."""
    return "z:" + os.path.abspath(p).replace("/", "\\")


# ---------------------------------------------------------------------------------
# The mode lanes — READ FROM `scripts/lanes.txt`, never listed here.
# ---------------------------------------------------------------------------------
#
# This lane used to carry its own four: packed, `/Gy`, `/O1`, `/O2`. That was the
# last surviving instance of the defect `scripts/lanes.txt` was written to close —
# **a lane that is not enumerated is a lane that does not run** — and here it had
# the worst possible shape, because those four compile **no `/EH` at any
# invocation** while every TU of the dc3 workload is compiled `/EHsc` and 35,964
# already-in-class `eh-bare` functions carry markers that appear only under it. The
# lane whose entire purpose is finding mis-emits the hand-written corpus cannot was
# blind to the axis the workload is built under, and its green read exactly like a
# green that had verified those flags.
#
# Reading the registry also makes this lane INHERIT the assertions that already
# guard it (`crates/c2-harness/tests/lane_registry.rs`): that the shipped list
# still carries an `/EH` lane, still *varies* `/Oi` where it is not already
# implied, and still names `/O1 /EHsc` even though its verdict rows are identical
# to `/O1`'s — verdict-identical is not redundant, the reference obj is a different
# obj. None of that had to be restated here, and restating it would have been a
# second implementation of one rule.


def lane_flags(fields):
    """Splice a registry row's flags EXACTLY the way `scripts/mode_lane.sh` does.

    `mode_lane.sh` writes `"$mode /GS- /c $*"` — mode first, then `/GS- /c`, then
    any remaining flags. `gate.sh` invokes it as `mode_lane.sh $flags`, so
    `Ox-Gy  /Ox /Gy` grades at `/Ox /GS- /c /Gy`. Reproduced verbatim rather than
    normalised: the flag string is part of the capture cache key and part of what
    `cl.exe` is handed, so "the same lane" has to mean the same characters in the
    same order, or this lane and the fixture gate are grading two things under one
    name.
    """
    return [fields[0], "/GS-", "/c"] + list(fields[1:])


def load_lanes(path=LANES):
    """`[(slug, flags), ...]` from the registry, by `scripts/gate.sh`'s own rule.

    Every failure here is fatal and named. An empty, short or unreadable registry
    is a lane list of length zero, and a cross product graded at zero modes is a
    run that grades nothing while exiting 0 — the precise failure this file's
    conversion to the registry exists to make impossible.
    """
    try:
        text = open(path).read()
    except OSError as e:
        raise SystemExit(
            "cannot read the lane registry at %s: %s\nThe registry is the list of "
            "configurations this lane grades at; without it the cross product has "
            "no modes and would grade nothing." % (path, e)
        )
    lanes, rows, malformed = [], 0, []
    for line in text.splitlines():
        row = line.split("#", 1)[0]
        if not row.strip():
            continue
        rows += 1
        fields = row.split()
        if len(fields) < 2:
            malformed.append(row.strip())
            continue
        lanes.append((fields[0], lane_flags(fields[1:])))
    if malformed:
        raise SystemExit(
            "%s has %d non-comment row(s) but only %d parse as lanes. A row needs a "
            "slug AND at least one flag; a row that does not parse must be an error, "
            "never a lane that silently vanishes. Offending row(s): %s"
            % (path, rows, len(lanes), " | ".join(malformed))
        )
    if not lanes:
        raise SystemExit(
            "the lane registry %s defines NO lanes. Every configuration below would "
            "be compiled at zero modes and this lane would report 0 mismatches over "
            "0 gradings." % path
        )
    dup = sorted(set(s for i, (s, _f) in enumerate(lanes)
                     if any(s == t for t, _g in lanes[:i])))
    if dup:
        raise SystemExit(
            "duplicate lane slug(s) in %s: %s. Two rows under one slug means one "
            "lane's results silently overwrite the other's while the table still "
            "shows the expected number of rows." % (path, ", ".join(dup)))
    if not any(f.startswith("/EH") for _s, fl in lanes for f in fl):
        raise SystemExit(
            "NO lane in %s compiles /EH, over %d lane(s). This lane was converted to "
            "the registry *because* it had never compiled /EH; a registry without one "
            "reopens the hole in the one instrument that finds cross-shape mis-emits."
            % (path, len(lanes)))
    return lanes


def assert_discovery_is_a_lane(lanes):
    """The profile the representatives are discovered at must itself be graded.

    Otherwise the pool is chosen at a configuration no lane compiles, and every
    "representative" is a claim about a profile this run never tested.
    """
    if not any(fl == DISCOVERY_FLAGS for _s, fl in lanes):
        raise SystemExit(
            "the discovery profile %s is not one of the registry's lanes (%s). The "
            "representative pool would be chosen at a configuration nothing here "
            "grades." % (" ".join(DISCOVERY_FLAGS),
                         " | ".join(" ".join(fl) for _s, fl in lanes)))


# ---------------------------------------------------------------------------------
# SHAPE, measured — what a "family" is, once you stop taking its name for it.
# ---------------------------------------------------------------------------------
#
# The lane used to key on the census LABEL, and a label is a NAME, not a shape.
# The consequence was measured on 2026-08-01: a rung landed +5,507 functions of
# genuinely new accepted shape (a trailing literal call argument, and the formal
# that moves beside it) and every number this lane prints came back byte-identical
# to the pre-merge run — 28 families, 84 representatives, 406 pairs, 388 emitted.
# The new shapes were absorbed into the existing `multiarg-tail-call` label, so
# they added no family, no pair, and — with three representatives sampled per
# family, all three drawn from older fragments — no representative either. The
# contrast is the tell: when an earlier rung genuinely added *labels*, the counter
# moved 18 -> 20 families and 171 -> 210 pairs with nobody typing anything.
#
# So the representatives are now chosen by SHAPE, and shape is read out of the
# compiler's own output: the opcode sequence of the emitted `.text`, with register
# fields and immediates masked off. Two cases are the same shape iff the same
# instructions come out in the same order. That is the only sense of "same shape"
# this project judges by (`CLAUDE.md`: the obj is the sole judge), it needs no
# agreement from the port about how it labels anything, and it moves the moment a
# rung emits an instruction it did not emit before — `74-lit-call-arg` adds an
# `li`, `75-moved-lit-call-arg` adds an `mr`, and both fall out as new shapes.
#
# Masking the operands is the deliberate cut, not an approximation: offsets, widths
# and operand order are swept WITHIN a family by the per-axis fragments and are
# explicitly not crossed here (see `docs/CROSS_PRODUCT.md`). Keying on raw bytes
# would make every immediate its own "shape" and the cross product would be
# thousands of near-identical TUs deep in the one axis this lane does not own.
#
# The label still names the family, because the pair matrix and the refusal
# frontier are reported per family and that reporting is continuous with every
# earlier run. What changed is which cases fill a family's representative slots.

# Primary opcodes whose real identity is in the extended field (bits 21..30) —
# `31` is the whole integer/logical/load-indexed bank, `63`/`59` the FP banks,
# `19` the branch-conditional-to-register bank, `4` the vector/AltiVec bank.
EXTENDED_PRIMARY = (4, 19, 31, 59, 63)


def text_of(blob):
    """Concatenated `.text*` raw data of a PE/COFF object, in section order."""
    if len(blob) < 20:
        return b""
    nsec = struct.unpack_from("<H", blob, 2)[0]
    out = []
    for i in range(nsec):
        o = 20 + 40 * i
        if o + 40 > len(blob):
            break
        name = blob[o:o + 8].rstrip(b"\0")
        size = struct.unpack_from("<I", blob, o + 16)[0]
        ptr = struct.unpack_from("<I", blob, o + 20)[0]
        if name.startswith(b".text") and ptr and size:
            out.append(blob[ptr:ptr + size])
    return b"".join(out)


def shape_of(blob):
    """The masked PPC opcode sequence of an obj's `.text` — this lane's "shape"."""
    t = text_of(blob)
    ops = []
    for i in range(0, len(t) - 3, 4):
        w = struct.unpack_from(">I", t, i)[0]
        pri = (w >> 26) & 0x3F
        if pri in EXTENDED_PRIMARY:
            ops.append((pri, (w >> 1) & 0x3FF))
        elif pri in (16, 18):
            ops.append((pri, w & 3))       # keep AA/LK: `b` and `bl` differ
        else:
            ops.append((pri, 0))
    return tuple(ops)


def declared_families(path=CENSUS_RS):
    """Every `FnVerdict::InClass("…")` label, by paren-matched scan of the census.

    Not a `grep`: three of the labels live inside a nested `match`/`if` argument
    (`call-sequence*`, `float-leaf`/`double-leaf`) and a line-wise pattern misses
    them — which would silently under-enumerate the families and report full
    coverage of a subset. That is the failure mode §6 keeps recording, so the
    scan brackets the whole argument.
    """
    src = open(path).read()
    key = "FnVerdict::InClass("
    names, i = [], 0
    while True:
        j = src.find(key, i)
        if j < 0:
            break
        k, depth = j + len(key), 1
        while depth and k < len(src):
            if src[k] == "(":
                depth += 1
            elif src[k] == ")":
                depth -= 1
            k += 1
        names += re.findall(r'"([^"]*)"', src[j + len(key):k - 1])
        i = k
    if not names:
        raise SystemExit(
            "no FnVerdict::InClass labels in %s — the family enumeration is the "
            "whole basis of this lane and it just came back empty" % path
        )
    return sorted(set(names))


def run_gap(paths, flags, work, tag, jobs=None):
    """Grade `paths` with `c2rs gap`. Returns `(stdout, rows)`; rows keyed by path."""
    listing = os.path.join(work, "%s.list" % tag)
    flagfile = os.path.join(work, "%s.flags" % tag)
    jsonl = os.path.join(work, "%s.jsonl" % tag)
    with open(listing, "w") as fh:
        fh.write("".join(wibo_path(p) + "\n" for p in paths))
    with open(flagfile, "w") as fh:
        fh.write(" ".join(flags) + "\n")
    out = subprocess.run(
        [C2RS, "gap", "--list", listing, "--flags-file", flagfile,
         "--jobs", jobs or JOBS, "--jsonl", jsonl],
        capture_output=True, text=True,
    ).stdout
    rows = {}
    if os.path.exists(jsonl):
        for line in open(jsonl):
            r = json.loads(line)
            if r.get("record"):
                continue
            rows["/" + r["src"].replace("\\", "/")[2:].lstrip("/")] = r
    return out, rows


def families_of(row):
    """The in-class shape families a graded TU exhibits, as a set."""
    return set(k.split("|", 1)[1] for k in row["fn_frames"])


def pairs_of(fams):
    """Every unordered family pair a TU exhibiting `fams` covers, self-pairs included."""
    fs = sorted(set(fams))
    return set((a, b) for i, a in enumerate(fs) for b in fs[i:])


def splice(sources):
    """One TU holding every source, each after the first inside its own namespace.

    The FIRST half is byte-identical to the standalone case that was graded, so
    a pair that mismatches cannot be blamed on the wrapping of the thing that
    was already known good. Namespaces rather than identifier renaming: they
    cannot collide, they need no tokenizer, and the port reads names out of the
    IL so the extra mangling is not a variable.
    """
    parts = [sources[0]]
    for i, s in enumerate(sources[1:], 1):
        parts.append("namespace c2x%d {\n%s}\n" % (i, s))
    return "\n".join(parts)


def measure_objs(paths, work):
    """`{path: (shape, externals)}` for every candidate TU — one obj each, measured.

    Both facts come out of the SAME emitted object, so neither is inferred from a
    name: the shape is the masked opcode sequence of `.text` and the externals are
    the marker symbols actually present in the bytes. Every candidate is a
    *matched* TU, so the port's obj is byte-identical to the reference's and
    emitting the port's is the same evidence at no second capture.

    Parallel because there are thousands of candidates and the whole point is that
    the lane can afford to look at all of them: measured at 8,863 candidates in
    13 s on 24 threads, against the ~2 minutes the old one-representative-at-a-time
    loop spent to learn strictly less.
    """
    out, lock, idx = {}, threading.Lock(), [0]
    failed = []
    nthreads = max(1, min(int(JOBS), len(paths))) if paths else 1

    def worker(n):
        obj = os.path.join(work, "probe%d.obj" % n)
        scratch = os.path.join(work, "probe%d" % n)
        while True:
            with lock:
                if idx[0] >= len(paths):
                    return
                i = idx[0]
                idx[0] += 1
            p = paths[i]
            if os.path.exists(obj):
                os.remove(obj)
            cmd = [C2RS, "prefilter", "--source", wibo_path(p),
                   "--emit-obj", obj, "--work", scratch]
            for f in DISCOVERY_FLAGS:
                cmd += ["--flag", f]
            res = subprocess.run(cmd, capture_output=True, text=True)
            if not os.path.exists(obj):
                with lock:
                    failed.append((p, res.stdout.strip()[:200]))
                continue
            blob = open(obj, "rb").read()
            v = (shape_of(blob), frozenset(e.decode() for e in TU_EXTERNALS
                                           if e in blob))
            with lock:
                out[p] = v

    threads = [threading.Thread(target=worker, args=(n,)) for n in range(nthreads)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    if failed:
        # Every candidate is a matched TU, so this cannot happen — and a silently
        # short measurement would shrink both the shape census and tier C rather
        # than fail, which is the absence-reads-as-success shape this file exists
        # to refuse. It is named, not swallowed.
        raise SystemExit(
            "prefilter emitted no obj for %d of %d candidate TUs, so their shape "
            "and TU-level externals cannot be measured. First: %s\n%s"
            % (len(failed), len(paths), failed[0][0], failed[0][1]))
    return out


def main():
    # ABSOLUTE, at entry, before anything derives a path from it. `wibo_path`
    # absolutises what it hands `cl.exe` but the plan's own bookkeeping used the
    # string as given, so a RELATIVE workdir made the two disagree and the run died
    # with a bare `KeyError` deep in the grading loop. Same family as the failure
    # `docs/` keeps recording one layer out: a relative sweep outdir also yields
    # `z:work\…`, which `cl.exe` cannot open, so every case capture-fails and every
    # count parsed out of the report reads 0 and passes. Normalised here, and the
    # resolved directory is PRINTED, so the run says which tree it wrote to.
    work = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else "/tmp/c2rs-cross-sweep")
    print("workdir %s" % work)
    cases = os.path.join(work, "cases")
    cross = os.path.join(work, "cross")
    for d in (work, cases, cross):
        os.makedirs(d, exist_ok=True)
    for d in (cases, cross):
        for f in os.listdir(d):
            if f.endswith(".cpp"):
                os.remove(os.path.join(d, f))

    sys.path.insert(0, os.path.join(REPO, "scripts"))
    import sweep_gen

    lanes = load_lanes()
    assert_discovery_is_a_lane(lanes)
    print("MODE LANES — read from %s, not listed in this file (%d lanes, %d of them /EH)"
          % (os.path.relpath(LANES, REPO), len(lanes),
             sum(1 for _s, fl in lanes if any(f.startswith("/EH") for f in fl))))
    for slug, fl in lanes:
        print("  %-14s %s" % (slug, " ".join(fl)))
    print()

    families = declared_families()
    print("the port declares %d accepted shape families (census.rs):" % len(families))
    print("  " + " ".join(families))
    print()

    print("generating the sweep corpus (the representative pool)")
    total = sweep_gen.write_cases(cases, os.path.join(REPO, "scripts/sweep.d"), quiet=True)
    pool = sorted(os.path.join(cases, f) for f in os.listdir(cases) if f.endswith(".cpp"))
    print("  %d cases from %d fragments" % (total, len(os.listdir(os.path.join(REPO, "scripts/sweep.d")))))

    # Bail out loudly rather than reporting a vacuous pass.
    probe, _ = run_gap(pool[:1], DISCOVERY_FLAGS, work, "probe", jobs="1")
    if "SKIP" in probe:
        print("SKIP: toolchain absent — the cross-product lane would be vacuous")
        return 0

    print("grading it to discover which family each case actually exercises")
    out, rows = run_gap(pool, DISCOVERY_FLAGS, work, "pool")
    if not rows:
        print(out)
        raise SystemExit("the pool scan produced no rows")
    baseline_mismatch = sum(1 for r in rows.values() if r["class"] == "mismatch")
    print("  %d TUs graded, %d matched, %d mismatched"
          % (len(rows), sum(1 for r in rows.values() if r["class"] == "match"),
             baseline_mismatch))

    # ---- what the corpus ALREADY grades, so "new" can be a number ----------------
    # A pair counts as already graded only where the port EMITTED (`match`): a TU
    # the port refuses compared no bytes. Unordered, because `fn_frames` is a map
    # and cannot say which function came first — the cross grades both orders,
    # a distinction the baseline cannot even express.
    fixtures = sorted(
        os.path.join(REPO, "fixtures/cpp", f)
        for f in os.listdir(os.path.join(REPO, "fixtures/cpp")) if f.endswith(".cpp")
    )
    _, frows = run_gap(fixtures, DISCOVERY_FLAGS, work, "fixtures")

    graded_before = set()
    for r in list(rows.values()) + list(frows.values()):
        if r["class"] == "match":
            graded_before |= pairs_of(families_of(r))

    # ---- candidates: one family, all in class, the port emitted ------------------
    cands = {}
    for path, r in sorted(rows.items()):
        if r["class"] != "match" or len(r["fn_frames"]) != 1:
            continue
        fam = families_of(r).pop()
        cands.setdefault(fam, []).append(path)
    ncands = sum(len(v) for v in cands.values())
    print()
    print("measuring the EMITTED SHAPE of all %d single-family candidate TUs" % ncands)
    measured = measure_objs([p for v in cands.values() for p in v], work)
    print("  %d objs emitted and read; shape = the masked .text opcode sequence"
          % len(measured))

    # ---- representatives: one per DISTINCT SHAPE, not `k` per label -------------
    #
    # A family's candidates are bucketed by measured shape and the buckets are
    # taken most-populated-first (so the family's typical emission is always a
    # representative), each bucket contributing its smallest source. `NREPS` is a
    # CAP on the number of shapes, not a sample size: a family with fewer distinct
    # shapes than the cap is now COMPLETELY represented, and a family with more
    # reports exactly how many it left out. That residue is the honest replacement
    # for the previous silence — under the old rule `multiarg-tail-call` had 8
    # distinct emitted shapes, 3 representatives, and a report that could not say
    # so.
    shapes_of_family = {}      # fam -> [(shape, [paths…]) …], most-populated first
    for fam, paths in cands.items():
        buckets = {}
        for p in paths:
            buckets.setdefault(measured[p][0], []).append(p)
        shapes_of_family[fam] = sorted(
            buckets.items(),
            key=lambda kv: (-len(kv[1]), min(os.path.getsize(q) for q in kv[1])),
        )

    reps = []          # (family, fragment, path, source)
    missing = []
    for fam in families:
        buckets = shapes_of_family.get(fam)
        if not buckets:
            missing.append(fam)
            continue
        for _shape, paths in buckets[:NREPS]:
            path = min(paths, key=lambda q: (os.path.getsize(q), q))
            reps.append((fam, os.path.basename(path).rsplit("-", 1)[0], path,
                         open(path).read()))

    ext_of = {(fam, p): set(measured[p][1]) for fam, _fr, p, _s in reps}
    print()
    print("REPRESENTATIVES — one per distinct emitted shape, cap %d per family"
          % NREPS)
    print("  %-28s %-24s %-28s %s" % ("family", "fragment", "case", "externals"))
    for fam, frag, path, _src in reps:
        print("  %-28s %-24s %-28s %s"
              % (fam, frag, os.path.basename(path),
                 " ".join(sorted(ext_of[(fam, path)])) or "-"))
    if missing:
        print()
        print("*** %d DECLARED FAMILY WITH NO REPRESENTATIVE: %s"
              % (len(missing), ", ".join(missing)))
        print("    The port accepts this shape and NO case in scripts/sweep.d/")
        print("    produces a TU made only of it, so nothing can cross it and this")
        print("    lane cannot claim to have graded it. Add a fragment.")
        return 2

    # ---- the SHAPE RESIDUE, named -----------------------------------------------
    # What the cap leaves outside the cross, per family and in total. This number
    # is what makes a widened family visible: a rung that emits an instruction it
    # did not emit before adds a shape here whether or not it adds a label.
    shapes_total = sum(len(v) for v in shapes_of_family.values())
    shapes_covered = sum(min(NREPS, len(v)) for v in shapes_of_family.values())
    uncovered = sorted(
        ((len(v) - NREPS, fam) for fam, v in shapes_of_family.items()
         if len(v) > NREPS), reverse=True)
    print()
    print("  SHAPES: %d distinct emitted shapes over %d candidate TUs; %d of them are"
          % (shapes_total, ncands, shapes_covered))
    print("  a representative and are crossed below. The %d not crossed, by family:"
          % (shapes_total - shapes_covered))
    if uncovered:
        for n, fam in uncovered:
            print("      %-28s %3d of %3d shapes uncrossed"
                  % (fam, n, len(shapes_of_family[fam])))
    else:
        print("      (none — every measured shape of every family is a representative)")
    print("  %d of the %d families are COMPLETELY represented."
          % (sum(1 for v in shapes_of_family.values() if len(v) <= NREPS),
             len(shapes_of_family)))

    external_reps = [(f, p) for (f, _fr, p, _s) in reps if ext_of[(f, p)]]
    print()
    print("  %d of %d representatives carry a TU-level external (%s)"
          % (len(external_reps), len(reps),
             ", ".join(sorted(set(e for v in ext_of.values() for e in v)))))

    # ---- tiers S and W: is the wrapping itself coverage-neutral, AT THIS LANE? ---
    # Every second-and-later half sits in a namespace, so if a namespace by itself
    # pushed a shape out of class the whole lane would grade refusals and report a
    # green that means nothing. Each representative is therefore also compiled
    # ALONE inside a namespace (tier W).
    #
    # Tier S is that check's CONTROL, and it is what made the conversion to the
    # 12-lane registry safe. The old check was "every W must match", which is only
    # a statement about the wrapping at a lane where the representative matches
    # UNWRAPPED — and the registry contains `/Od`, where the port refuses
    # essentially everything on purpose (that lane's whole content is `mismatch 0`).
    # Asserted unconditionally, `/Od` would have reported "the wrapping is not
    # coverage-neutral" for all of them, which is false and would have blamed the
    # instrument for the mode. So the representative is compiled BOTH ways at every
    # lane and the alarm is the difference: wrapped refuses where standalone
    # matched.
    plan = []          # (name, [sources], tier, [families])
    for fam, _fr, pa, sa in reps:
        key = "%s.%s" % (fam, os.path.basename(pa)[:-4])
        plan.append(("S-%s" % key, [sa], "S", [fam]))
        plan.append(("W-%s" % key, ["namespace c2x0 {\n%s}\n" % sa], "W", [fam]))

    # ---- tier A: the pairwise cross --------------------------------------------
    for fa, fra, pa, sa in reps:
        for fb, frb, pb, sb in reps:
            plan.append((
                "A-%s.%s__%s.%s" % (fa, os.path.basename(pa)[:-4],
                                    fb, os.path.basename(pb)[:-4]),
                [sa, sb], "A", [fa, fb],
            ))

    # ---- tier B: the arity axis, with and without a framed observer -------------
    observers = [(f, s) for (f, _fr, _p, s) in reps if f == "framed-call"]
    first_by_family = {}
    for fam, _fr, _p, src in reps:
        first_by_family.setdefault(fam, src)
    if not observers:
        print("  (no framed-call representative: tier B has no observer)")
    obs = observers[0][1] if observers else None
    for fam, src in sorted(first_by_family.items()):
        for n in (1, 2, 3, 4):
            body = [src] * n
            plan.append(("B-%s.x%d" % (fam, n), body, "B", [fam]))
            if obs is not None:
                plan.append(("B-%s.x%d.obs-after" % (fam, n), body + [obs], "B",
                             [fam, "framed-call"]))
                plan.append(("B-%s.x%d.obs-before" % (fam, n), [obs] + body, "B",
                            ["framed-call", fam]))

    # ---- tier C: three external-bearing families, in every order ---------------
    # The bug class this lane exists for is a *per-TU* quantity read from a
    # per-function place, and the compiler-label counter is where it lives. The
    # families selected here are the ones whose representative carries a marker
    # that the measured surcharge table charges for (`docs/LABEL_COUNTER.md`
    # §1.1) — a heuristic, see TU_EXTERNALS. Crossed three deep in all orders,
    # and again with a stride-1 integer leaf inserted at each position, because a
    # counter error that an adjacent function absorbs is invisible without a
    # separator.
    # The representative used here must be one that ACTUALLY carries an
    # external, not merely the first of a family that has some such member:
    # `store-leaf` covers both the FP store (which brings `_fltused`) and the
    # integer one (which does not), and picking the wrong one would label the
    # tier honestly and grade something else.
    ext_rep = {}
    for fam, _fr, p, src in reps:
        if ext_of[(fam, p)]:
            ext_rep.setdefault(fam, src)
    ext_families = sorted(ext_rep)
    sep = first_by_family.get("straight-line")
    for a in ext_families:
        for b in ext_families:
            for c in ext_families:
                body = [ext_rep[a], ext_rep[b], ext_rep[c]]
                plan.append(("C-%s__%s__%s" % (a, b, c), body, "C", [a, b, c]))
                if sep is None:
                    continue
                for at in range(4):
                    plan.append((
                        "C-%s__%s__%s.sep%d" % (a, b, c, at),
                        body[:at] + [sep] + body[at:], "C",
                        [a, b, c] + ["straight-line"],
                    ))

    paths, meta = [], {}
    for name, srcs, tier, fams in plan:
        p = os.path.join(cross, "%s.cpp" % re.sub(r"[^A-Za-z0-9._-]", "_", name))
        with open(p, "w") as fh:
            fh.write(splice(srcs))
        paths.append(p)
        meta[p] = (tier, fams, name)
    if len(set(paths)) != len(paths):
        raise SystemExit("two cross configurations claim one filename")

    seen_pairs = set()
    for _n, _s, tier, fams in plan:
        if tier == "A":
            seen_pairs |= pairs_of(fams)
    npairs_new = len(seen_pairs - graded_before)

    print()
    print("CONFIGURATIONS")
    print("  tier S  %5d representatives compiled ALONE (the wrapping check's control)"
          % sum(1 for _n, _s, t, _f in plan if t == "S"))
    print("  tier W  %5d representatives compiled alone inside a namespace (the "
          "wrapping check)" % sum(1 for _n, _s, t, _f in plan if t == "W"))
    print("  tier A  %5d ordered pairs of representatives (both orders, diagonal included)"
          % sum(1 for _n, _s, t, _f in plan if t == "A"))
    print("  tier B  %5d arity/observer configurations" % sum(1 for _n, _s, t, _f in plan if t == "B"))
    print("  tier C  %5d ordered triples over the %d TU-external families, with and "
          "without a separator" % (sum(1 for _n, _s, t, _f in plan if t == "C"),
                                   len(ext_families)))
    print("  %d unordered family pairs reached; %d of them occur in NO matched TU of"
          % (len(seen_pairs), npairs_new))
    print("  the fixture corpus or the whole sweep corpus — nothing has ever graded them.")
    print("  %d configurations x %d mode lanes = %d gradings SUBMITTED"
          % (len(paths), len(lanes), len(paths) * len(lanes)))
    if not paths or not seen_pairs:
        raise SystemExit(
            "the plan is empty (%d configurations, %d pairs reached). A cross "
            "product over nothing grades nothing and would print 0 mismatches."
            % (len(paths), len(seen_pairs)))

    # ---- grade -----------------------------------------------------------------
    #
    # Every quantity below is a POSITIVE statement — submitted, graded, emitted —
    # and each one is checked against zero before the run is allowed to call itself
    # clean. A cross product whose captures all failed reports `mismatch 0` exactly
    # as a clean one does, and `mismatch 0` over `graded 0` is the shape that has
    # now fooled this repo's instruments nine times.
    alarms, wrap_lost = [], []
    emitted_pairs, refused_only = set(), set()
    per_lane = []          # (slug, flags, submitted, graded, capture_fail, counts)
    lane_ungraded = {}     # slug -> {pair-string: n}, tier A only
    total_graded = 0
    print()
    print("GRADING (submitted / graded — `graded` excludes capture-fail, which is the")
    print("toolchain failing to produce a reference at all and is never a verdict)")
    for slug, flags in lanes:
        out, crows = run_gap(paths, flags, work, "cross-%s" % slug)
        if not crows:
            print(out)
            raise SystemExit(
                "lane %s [%s] produced NO rows over %d submitted configurations"
                % (slug, " ".join(flags), len(paths)))
        stray = sorted(set(crows) - set(meta))
        if stray:
            # Named, never a `KeyError`. The scan came back describing a source
            # this run did not plan, which means the path this file writes and the
            # path `c2rs gap` reports have drifted apart — and every count below
            # would then be a count over the wrong set.
            raise SystemExit(
                "lane %s reported %d source path(s) that are not in this run's plan; "
                "the paths written and the paths graded have drifted. First: %s\n"
                "(plan holds %d paths, e.g. %s)"
                % (slug, len(stray), stray[0], len(meta), sorted(meta)[0]))
        counts, per_tier, std = {}, {}, {}
        for p, r in crows.items():
            tier, fams, name = meta[p]
            counts[r["class"]] = counts.get(r["class"], 0) + 1
            per_tier.setdefault(tier, {})
            per_tier[tier][r["class"]] = per_tier[tier].get(r["class"], 0) + 1
            pairs = pairs_of(fams)
            if tier == "S":
                std[name[2:]] = r["class"]
            if r["class"] == "mismatch":
                alarms.append((slug, p, r["reason"], r["detail"]))
            elif r["class"] == "match":
                emitted_pairs |= pairs
            elif r["class"] == "capture-fail":
                pass                       # counted, never a verdict — see below
            else:
                refused_only |= pairs
                if tier == "A":
                    key = " + ".join(fams)
                    lane_ungraded.setdefault(slug, {})
                    lane_ungraded[slug][key] = lane_ungraded[slug].get(key, 0) + 1
        # Tier W against its OWN control at THIS lane: a representative that
        # refuses standalone here says nothing about the wrapping.
        for p, r in crows.items():
            tier, _fams, name = meta[p]
            if tier == "W" and std.get(name[2:]) == "match" and r["class"] != "match":
                wrap_lost.append((slug, name[2:], r["reason"]))
        graded = sum(counts.get(k, 0) for k in GRADED)
        capfail = counts.get("capture-fail", 0)
        total_graded += graded
        per_lane.append((slug, " ".join(flags), len(paths), graded, capfail, counts))
        print("  %-14s %6d/%-6d  %s   [%s]"
              % (slug, graded, len(paths),
                 "  ".join("%s=%d" % (k, counts[k]) for k in sorted(counts)),
                 "  ".join("%s: %s" % (t, ",".join("%s=%d" % (k, v)
                                                   for k, v in sorted(per_tier[t].items())))
                           for t in sorted(per_tier))))

    # ---- vacuity, checked positively -------------------------------------------
    dead = [(s, f, sub) for s, f, sub, g, _cf, _c in per_lane if g == 0]
    if dead:
        print()
        print("*** VACUOUS LANE(S): %d of %d lanes submitted configurations and graded"
              % (len(dead), len(lanes)))
        print("*** NONE of them. That is the toolchain PRESENT and every capture")
        print("*** failing — a bad flag string, a relative outdir, an exhausted tmpfs")
        print("*** inode table (`df -i`, not `df -h`) — and it reports `mismatch 0`:")
        for s, f, sub in dead:
            print("      %-14s [%s]  %d submitted, 0 graded" % (s, f, sub))
        return 5
    capfails = [(s, f, cf) for s, f, _sub, _g, cf, _c in per_lane if cf]
    if capfails:
        print()
        print("*** CAPTURE-FAIL on %d lane(s): the reference compiler produced no obj,"
              % len(capfails))
        print("*** so those configurations were submitted and never graded. They are")
        print("*** not refusals and they are not green — they are missing:")
        for s, f, cf in capfails:
            print("      %-14s [%s]  %d capture-fail" % (s, f, cf))
        return 4
    if total_graded == 0:
        print()
        print("*** NOTHING WAS GRADED across %d lanes. Exiting non-zero rather than"
              % len(lanes))
        print("*** reporting 0 mismatches over 0 comparisons.")
        return 5

    if wrap_lost:
        print()
        print("*** THE WRAPPING IS NOT COVERAGE-NEUTRAL: %d (lane, representative)"
              % len(wrap_lost))
        print("*** pairs match compiled ALONE and stop matching merely by being inside")
        print("*** a namespace, so every cross whose second half is one of them grades")
        print("*** a refusal and this lane's green would mean nothing:")
        for slug, name, reason in wrap_lost[:20]:
            print("      [%s] %s — %s" % (slug, name, reason))
        return 3

    if lane_ungraded:
        print()
        print("UNGRADED (tier A), per lane — ordered pairs the port refuses at the TU")
        print("level, so no bytes were compared. They are not green; they are")
        print("unmeasured. Listed per lane as a count, because a refusal is a property")
        print("of the lane too: `/Od` refuses on purpose and its whole content is")
        print("`mismatch 0`, so its 400-row list would drown the modes that emit.")
        for slug, _f in lanes:
            d = lane_ungraded.get(slug, {})
            print("  %-14s %4d ordered pairs refused, %4d configurations"
                  % (slug, len(d), sum(d.values())))
        everywhere = sorted(set.intersection(
            *[set(lane_ungraded.get(s, {})) for s, _f in lanes]) or ())
        print("  %d ordered pair(s) refused at EVERY lane:" % len(everywhere))
        for k in everywhere:
            print("      %s" % k)

    frontier = sorted(refused_only - emitted_pairs)
    print()
    print("THE TU-LEVEL REFUSAL FRONTIER — %d unordered family pairs that never"
          % len(frontier))
    print("emitted in ANY configuration of this lane, at any arity or mode. This is")
    print("what the whole-TU gate hides, and `docs/GAPS.md` §6 #13 named it a debt:")
    for a, b in frontier:
        print("    %s + %s" % (a, b))
    if not frontier:
        print("    (none — every pair this lane reached emitted somewhere)")

    npairs_total = len(families) * (len(families) + 1) // 2
    print()
    print("WHAT THIS RUN GRADED, stated positively")
    print("  configurations submitted   %d" % len(paths))
    print("  mode lanes                 %d (%s)"
          % (len(lanes), os.path.relpath(LANES, REPO)))
    print("  gradings submitted         %d" % (len(paths) * len(lanes)))
    print("  gradings GRADED            %d" % total_graded)
    print("  representatives            %d, one per distinct emitted shape (cap %d)"
          % (len(reps), NREPS))
    print("  distinct shapes measured   %d; %d crossed, %d not (residue named above)"
          % (shapes_total, shapes_covered, shapes_total - shapes_covered))
    print("  family pairs reached       %d of the %d the census declares"
          % (len(seen_pairs), npairs_total))
    print("  family pairs EMITTED       %d of %d" % (len(emitted_pairs), npairs_total))
    print("  refusal-frontier residue   %d pairs, named above" % len(frontier))

    print()
    if alarms:
        print("*** MISMATCH — the port emitted bytes for a COMBINATION and they were")
        print("*** wrong. This outranks everything else (docs/GAPS.md §6).")
        for slug, p, reason, detail in alarms:
            print("  [%s] %s\n      %s %s" % (slug, os.path.basename(p), reason, detail[:160]))
        return 1
    if not emitted_pairs:
        print("*** NO family pair EMITTED anywhere across %d lanes. The port compared")
        print("*** bytes for nothing, and `0 mismatches` would be a statement about")
        print("*** an empty set.")
        return 5
    print("cross-product lane: %d configurations x %d lanes = %d gradings, "
          "%d of them graded, 0 mismatches"
          % (len(paths), len(lanes), len(paths) * len(lanes), total_graded))
    if baseline_mismatch:
        print("BUT the representative pool itself reported %d mismatch — that is an "
              "alarm of its own" % baseline_mismatch)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
