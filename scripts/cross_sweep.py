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
* **Representatives are discovered by compiling, not written down.** The whole
  `scripts/sweep.d/` corpus is generated and graded, and a family's
  representative is a *matched* TU whose in-class functions are all of that one
  family — preferring the smallest, and preferring one per fragment so the `k`
  representatives of a family come from structurally different generators.
  A family with **no** representative is a hole in the sweep corpus and this
  lane **fails**, by name (that is how `call-sequence`, `call-sequence-value`
  and `call-sequence-lit` were found to have no single-function case anywhere,
  which is what `scripts/sweep.d/71-call-sequence.py` fixes).
* **Tier A — the pairwise cross.** Every ordered pair of representatives,
  *including both orders and the diagonal*, at four mode lanes: `/Ox` packed,
  `/Ox /Gy`, `/O1`, `/O2`. Order matters (the label counter is consumed in `.ex`
  order and `_fltused` is placed after the *first* FP-touching function), so
  both orders are separate configurations, not one.
* **Tier B — the arity axis.** #13's rule is "one slot per function plus one for
  the TU if anything touches floating point", and at n = 1 that is
  indistinguishable from "two slots per FP function". So each family is also
  graded at n = 1, 2, 3, 4 copies of itself, alone and with a framed observer
  before and after it — a framed function is the only thing that *renders* the
  counter, so a TU without one cannot grade it however many copies it has.
* **Tier C — triples over the TU-external families.** Every TU-level external
  takes a slot in the same compiler-label sequence, so pairs cannot separate
  "one slot per external" from "one slot per external-bearing function" once
  two externals come from one function. The external families are therefore
  crossed three deep in all orders, and again with a stride-1 integer leaf
  inserted at each of the four positions — a counter error an adjacent function
  absorbs is invisible without a separator.
* **Tier W — the wrapping check.** Every representative is also compiled ALONE
  inside a namespace. If a namespace by itself pushed a shape out of class, the
  whole lane would grade refusals and report a green that means nothing.
* **Deliberately NOT graded**, and this is not a footnote:
  - **triples of three distinct NON-external families.** Tier C is restricted to
    the families whose representative carries a TU-level external; the full
    `R³` is not run, so a three-way interaction among plain leaves would not be
    caught.
  - **the intra-family parameter space.** A family is represented by `k`
    (default 3) TUs out of the hundreds the sweep generates. Operand order,
    widths, offsets and argument positions are swept *within* a family by the
    per-axis fragments and are **not** crossed against another family here.
  - **flags beyond the four lanes** (`/Od`, `/EHsc`, `/GS`, `/GR`, `/Zi`, `/Oi`).
  - **any pair the port refuses at the TU level.** Those are compiled and
    counted and named, but no bytes were compared, so they are *ungraded*, not
    green. The FP-beside-framed pair is currently in this set — which is #13's
    "a gate that hides a wrong rule is a debt, not a fix", still outstanding.

A MISMATCH here is an ALARM: the port emitted bytes for a combination and they
were wrong. `NotImplemented` / `vocab-gap` is an honest refusal.
"""

import json
import os
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
C2RS = os.environ.get("C2RS_BIN") or os.path.join(REPO, "target/release/c2rs")
JOBS = os.environ.get("C2RS_JOBS", "16")
NREPS = int(os.environ.get("C2RS_CROSS_REPS", "3"))
CENSUS_RS = os.path.join(REPO, "crates/c2-il/src/func/census.rs")

# The four lanes `scripts/mode_lane.sh` grades the fixtures at. "Packed" is the
# absence of `/Gy`; `/Gy` puts every function in its own COMDAT and changes the
# obj shell, which is where §6 #11 (`_fltused`) went wrong.
MODES = (
    ("Ox-packed", ["/Ox", "/GS-", "/c"]),
    ("Ox-Gy", ["/Ox", "/GS-", "/Gy", "/c"]),
    ("O1-packed", ["/O1", "/GS-", "/c"]),
    ("O2-packed", ["/O2", "/GS-", "/c"]),
)
BASE_MODE = MODES[0][1]

# TU-level externals: symbols a *translation unit* carries because some function
# in it needed them. Each one occupies a slot in the compiler-label sequence
# (`docs/CODEGEN_FRAMED_CALLS.md` §4.4, `docs/GAPS.md` §6 #13), which is the
# mechanism behind every bug this lane exists to find. Detected by reading the
# representative's own obj, never assumed from the family's name.
TU_EXTERNALS = (b"_fltused", b"__savegprlr", b"__restgprlr", b".pdata")


def wibo_path(p):
    """`/tmp/x/y.cpp` -> `z:\\tmp\\x\\y.cpp`, the form `cl.exe` under wibo takes."""
    return "z:" + os.path.abspath(p).replace("/", "\\")


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


def externals_of(cpp, work):
    """The TU-level externals a source's obj actually carries — measured.

    The port's obj is byte-identical to the reference's for a matched TU, so
    emitting the port's is the same evidence and costs no second capture.
    """
    obj = os.path.join(work, "probe.obj")
    if os.path.exists(obj):
        os.remove(obj)
    cmd = [C2RS, "prefilter", "--source", wibo_path(cpp), "--emit-obj", obj]
    for f in BASE_MODE:
        cmd += ["--flag", f]
    res = subprocess.run(cmd, capture_output=True, text=True)
    if not os.path.exists(obj):
        # Every representative is a matched TU, so this cannot happen — and an
        # empty answer would silently shrink tier C rather than fail, so it says
        # so instead of returning "no externals".
        raise SystemExit(
            "prefilter emitted no obj for the representative %s, so its TU-level "
            "externals cannot be measured:\n%s" % (cpp, res.stdout.strip())
        )
    blob = open(obj, "rb").read()
    return set(e.decode() for e in TU_EXTERNALS if e in blob)


def main():
    work = sys.argv[1] if len(sys.argv) > 1 else "/tmp/c2rs-cross-sweep"
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

    families = declared_families()
    print("the port declares %d accepted shape families (census.rs):" % len(families))
    print("  " + " ".join(families))
    print()

    print("generating the sweep corpus (the representative pool)")
    total = sweep_gen.write_cases(cases, os.path.join(REPO, "scripts/sweep.d"), quiet=True)
    pool = sorted(os.path.join(cases, f) for f in os.listdir(cases) if f.endswith(".cpp"))
    print("  %d cases from %d fragments" % (total, len(os.listdir(os.path.join(REPO, "scripts/sweep.d")))))

    # Bail out loudly rather than reporting a vacuous pass.
    probe, _ = run_gap(pool[:1], BASE_MODE, work, "probe", jobs="1")
    if "SKIP" in probe:
        print("SKIP: toolchain absent — the cross-product lane would be vacuous")
        return 0

    print("grading it to discover which family each case actually exercises")
    out, rows = run_gap(pool, BASE_MODE, work, "pool")
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
    _, frows = run_gap(fixtures, BASE_MODE, work, "fixtures")

    graded_before = set()
    for r in list(rows.values()) + list(frows.values()):
        if r["class"] == "match":
            graded_before |= pairs_of(families_of(r))

    # ---- representatives -------------------------------------------------------
    cands = {}
    for path, r in sorted(rows.items()):
        if r["class"] != "match" or len(r["fn_frames"]) != 1:
            continue
        fam = families_of(r).pop()
        stem = os.path.basename(path).rsplit("-", 1)[0]
        src = open(path).read()
        cands.setdefault(fam, []).append(
            (sum(r["fn_frames"].values()), len(src), stem, path, src)
        )

    reps = []          # (family, fragment, path, source)
    missing = []
    for fam in families:
        c = sorted(cands.get(fam, []))
        if not c:
            missing.append(fam)
            continue
        picked, seen_frag = [], set()
        for entry in c:                       # one per fragment first: diversity
            if entry[2] not in seen_frag:
                seen_frag.add(entry[2])
                picked.append(entry)
            if len(picked) == NREPS:
                break
        for entry in c:                       # …then fill from the rest
            if len(picked) == NREPS:
                break
            if entry not in picked:
                picked.append(entry)
        for entry in picked:
            reps.append((fam, entry[2], entry[3], entry[4]))

    print()
    print("REPRESENTATIVES (%d per family, discovered by grading — never listed here)"
          % NREPS)
    ext_of = {}
    for fam, frag, path, _src in reps:
        ext = externals_of(path, work)
        ext_of[(fam, path)] = ext
        print("  %-28s %-24s %-28s %s"
              % (fam, frag, os.path.basename(path), " ".join(sorted(ext)) or "-"))
    if missing:
        print()
        print("*** %d DECLARED FAMILY WITH NO REPRESENTATIVE: %s"
              % (len(missing), ", ".join(missing)))
        print("    The port accepts this shape and NO case in scripts/sweep.d/")
        print("    produces a TU made only of it, so nothing can cross it and this")
        print("    lane cannot claim to have graded it. Add a fragment.")
        return 2

    external_reps = [(f, p) for (f, _fr, p, _s) in reps if ext_of[(f, p)]]
    print()
    print("  %d of %d representatives carry a TU-level external (%s)"
          % (len(external_reps), len(reps),
             ", ".join(sorted(set(e for v in ext_of.values() for e in v)))))

    # ---- tier W: is the wrapping itself coverage-neutral? -----------------------
    # Every second-and-later half sits in a namespace, so if a namespace by itself
    # pushed a shape out of class the whole lane would grade refusals and report a
    # green that means nothing. Each representative is therefore also compiled
    # ALONE inside a namespace, and every one of them must still match.
    plan = []          # (name, [sources], tier, [families])
    for fam, _fr, pa, sa in reps:
        plan.append(("W-%s.%s" % (fam, os.path.basename(pa)[:-4]),
                     ["namespace c2x0 {\n%s}\n" % sa], "W", [fam]))

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

    # ---- tier C: three TU-level externals, in every order ----------------------
    # The bug class this lane exists for is a *per-TU* quantity read from a
    # per-function place, and every TU-level external takes a slot in the same
    # compiler-label sequence (`docs/CODEGEN_FRAMED_CALLS.md` §4.4). Pairs cannot
    # separate "one slot per external" from "one slot per external-bearing
    # function" once two externals come from one function, so the external
    # families are also crossed three deep, in all orders — and again with a
    # stride-1 integer leaf inserted at each position, because a counter error
    # that an adjacent function absorbs is invisible without a separator.
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
        meta[p] = (tier, fams)
    if len(set(paths)) != len(paths):
        raise SystemExit("two cross configurations claim one filename")

    seen_pairs = set()
    for _n, _s, tier, fams in plan:
        if tier == "A":
            seen_pairs |= pairs_of(fams)
    npairs_new = len(seen_pairs - graded_before)

    print()
    print("CONFIGURATIONS")
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
    print("  %d configurations x %d mode lanes = %d gradings"
          % (len(paths), len(MODES), len(paths) * len(MODES)))

    # ---- grade -----------------------------------------------------------------
    alarms, refused_pairs, wrap_lost = [], {}, []
    emitted_pairs, refused_only = set(), set()
    print()
    for label, flags in MODES:
        out, crows = run_gap(paths, flags, work, "cross-%s" % label)
        if not crows:
            print(out)
            raise SystemExit("the %s lane produced no rows" % label)
        counts, per_tier = {}, {}
        for p, r in crows.items():
            tier, fams = meta[p]
            counts[r["class"]] = counts.get(r["class"], 0) + 1
            per_tier.setdefault(tier, {})
            per_tier[tier][r["class"]] = per_tier[tier].get(r["class"], 0) + 1
            pairs = pairs_of(fams)
            if r["class"] == "mismatch":
                alarms.append((label, p, r["reason"], r["detail"]))
            elif r["class"] == "match":
                emitted_pairs |= pairs
            else:
                if tier == "W":
                    wrap_lost.append((label, os.path.basename(p), r["reason"]))
                refused_only |= pairs
                if tier == "A":
                    refused_pairs.setdefault((label, " + ".join(fams)), 0)
                    refused_pairs[(label, " + ".join(fams))] += 1
        print("  %-12s %s   [%s]" % (label, "  ".join(
            "%s=%d" % (k, counts[k]) for k in sorted(counts)),
            "  ".join("%s: %s" % (t, ",".join("%s=%d" % (k, v)
                                              for k, v in sorted(per_tier[t].items())))
                      for t in sorted(per_tier))))

    if wrap_lost:
        print()
        print("*** THE WRAPPING IS NOT COVERAGE-NEUTRAL: %d representatives stop"
              % len(wrap_lost))
        print("*** matching merely by being inside a namespace, so every pair whose")
        print("*** second half is one of them grades a refusal and this lane's green")
        print("*** would mean nothing:")
        for label, name, reason in wrap_lost[:20]:
            print("      [%s] %s — %s" % (label, name, reason))
        return 3

    ungraded = sorted(set(k[1] for k in refused_pairs))
    if ungraded:
        print()
        print("UNGRADED (tier A) — the port refuses these ordered pairs at the TU")
        print("level, so no bytes were compared. They are not green; they are")
        print("unmeasured:")
        for k in ungraded:
            print("    %s" % k)
    frontier = sorted(refused_only - emitted_pairs)
    if frontier:
        print()
        print("THE TU-LEVEL REFUSAL FRONTIER — %d unordered family pairs that never"
              % len(frontier))
        print("emitted in ANY configuration of this lane, at any arity or mode. This is")
        print("what the whole-TU gate hides, and `docs/GAPS.md` §6 #13 named it a debt:")
        for a, b in frontier:
            print("    %s + %s" % (a, b))
    print()
    print("  %d of the %d unordered family pairs emitted somewhere in this lane"
          % (len(emitted_pairs), len(families) * (len(families) + 1) // 2))

    print()
    if alarms:
        print("*** MISMATCH — the port emitted bytes for a COMBINATION and they were")
        print("*** wrong. This outranks everything else (docs/GAPS.md §6).")
        for label, p, reason, detail in alarms:
            print("  [%s] %s\n      %s %s" % (label, os.path.basename(p), reason, detail[:160]))
        return 1
    print("cross-product lane: %d configurations x %d lanes, 0 mismatches"
          % (len(paths), len(MODES)))
    if baseline_mismatch:
        print("BUT the representative pool itself reported %d mismatch — that is an "
              "alarm of its own" % baseline_mismatch)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
