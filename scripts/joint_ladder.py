#!/usr/bin/env python3
"""joint_ladder.py — THE COMPLETE-BLOCKER-SET LADDER, committed.

    scripts/joint_ladder.py --tus work/<lane>/near_tus.txt --out work/<lane>/ladder \\
                            [--cwd ../dc3-decomp] [--flags work/dc3-workload/flags.txt]

# Why a ladder and not a histogram

`crates/c2-il/src/func/census.rs` returns `FnVerdict::Blocked(Block)` — **one**
block — and its own doc says "Blocked at the *first* unmodeled feature". The
port stops at the first refusal **BY DESIGN**, so every blocked body reports
exactly one blocker no matter how many it has (board **#3131**). A first-blocker
key is therefore **not a distance**, a histogram of them is not a distance
distribution, and the standing rule that a lane may not dispatch off a
blocked-key size ranking (bound five times, `#3505`) follows from that fact and
not from taste.

The only route to a body's **complete** blocker set is to **lift the clause that
raised the head key and look again**. That is what this does. Per **function
slot** (slot identity is why `c2rs census --tsv` exists — the histogram at the
bottom of that command throws it away), the sequence of head keys across rungs
IS that slot's enumerated blocker set, in the order the parser meets them.

# The two lift mechanisms, and why the reach is declared rather than discovered

* **`relax:N`** — the **shipped** relaxation switch, `c2_il::Relax`, driven
  through `c2rs census --relax N`. Level 1 (`name-from-gl`) supplies a
  placeholder where a callee or data symbol did not resolve through `.gl`, so a
  body blocked only on a NAME reaches the composition. This needs no scratch
  tree: it is a named, settable decision point that already ships.

* **`src:<name>`** — a **scratch-tree source lift**. Each is one named clause in
  `census.rs`, neutralized by prefixing its match-arm guard with `false &&`.
  That transform is deliberately the weakest one that works: it cannot delete a
  binding, cannot reorder arms, and cannot change any other arm's behaviour —
  the arm simply never fires and the chain falls through to the next gate, which
  is exactly "lift this clause". The anchor for each lift is asserted to occur
  **exactly once** in the file before it is applied; a lift whose anchor moved is
  a hard failure, never a silently-skipped rung.

  The scratch tree is `<out>/scratch/`, a copy of the committed worktree. **The
  lifted state never ships**: this tool never writes to the committed tree, and
  it checks after every rung that the committed tree's `crates/` is clean.

**The ladder's reach is NOT total, and that is declared up front rather than
discovered at the end.** Both mechanisms lift **post-parse** gates — clauses
that run on a body which already parsed end to end. A head key raised *inside*
`parse_segment_detail` (`expr-op-0xNN`, `expr-load-type-XXXX`,
`call-ref-cflow-jump`, …) cannot be lifted without writing the decoder it names,
and this tool does not write one. Such a slot is reported **UNLIFTABLE** with
its terminal key, and is never reported as "closed" or as "no movement".

# The empty-rung rule (board #3470, sharpened for a ladder)

`repo_root()` is `CARGO_MANIFEST_DIR/../..` **baked at compile time**, so a
binary built in a scratch tree resolves `compilers/` relative to *that* tree,
finds none, prints `SKIP: toolchain absent`, degrades cleanly as `CLAUDE.md`
requires — and **exits 0**.

For a scan pair an empty arm looks like a broken run. **For a ladder it does
not**: every rung's output is a judgement of the form *"did lifting this clause
move anything?"*, and a rung that graded nothing produces the same observable as
a genuinely inert clause — **no movement**. That is not an obvious error, it is
a substantive and completely wrong conclusion. So, per rung:

* `C2RS_COMPILERS` / `C2RS_WIBO` are exported for **every** invocation;
* the **denominator triple** prints beside every numerator — TUs graded,
  function slots walked, distinct head keys emitted;
* a zero on any component, any `SKIP: toolchain absent`, or a nonzero probe exit
  is a **VOID**: the tool refuses and exits **4**, the same code
  `scripts/scan_pair.sh` uses for the same family. It is never a data point.
* `skips=0` is printed **positively** (`w-permute`'s pattern), never assumed.

Rung 0 is the identity control: a scratch build with zero lifts must reproduce
the committed tree slot for slot. That doubles as the toolchain proof, because a
scratch binary that could not see the toolchain makes rung 0 **red** rather than
silently green.

# What it prints

**Subset structure, never a ranking.** Per slot: the ordered sequence of head
keys and how it terminated (`CLOSED` / `UNLIFTABLE` / `EXHAUSTED`). Per TU: the
set of constructs its closure requires and whether that set is bounded. Plus the
count of **discriminating cells** — slots whose key moved at any rung — because
"nothing moved" must be a loud result and not a silent pass.

std-library Python only.
"""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys

# ---------------------------------------------------------------------------
# THE LIFT CATALOGUE — named, enumerated, and asserted before it is applied.
#
# Every entry is one clause of the post-parse gate chain in `census.rs`. The
# `anchor` must occur EXACTLY ONCE in the file; the transform prefixes the
# match-arm guard with `false &&` so the arm can never fire.
#
# `raises` names the census key the clause produces. It is documentation for a
# reader and a cross-check for the tool: a lift whose key never appears as a
# head key on the population is reported as *not exercised on this population*,
# which is a different statement from "moved nothing".
# ---------------------------------------------------------------------------
CENSUS_RS = "crates/c2-il/src/func/census.rs"

SRC_LIFTS = {
    "opt-mode": {
        "file": CENSUS_RS,
        "anchor": "if opt_word_mode(opt_word).is_none() =>",
        "lifted": "if false && opt_word_mode(opt_word).is_none() =>",
        "raises": "opt-mode-*",
    },
    "ptr-walk-loop-not-o1": {
        "file": CENSUS_RS,
        "anchor": "if f.ptr_walk_loop().is_some()",
        "lifted": "if false && f.ptr_walk_loop().is_some()",
        "raises": "ptr-walk-loop-not-o1",
    },
    "ptr-walk-chain-loop-not-o1": {
        "file": CENSUS_RS,
        "anchor": "if f.ptr_walk_chain_loop().is_some()",
        "lifted": "if false && f.ptr_walk_chain_loop().is_some()",
        "raises": "ptr-walk-chain-loop-not-o1",
    },
    "callee-defined-in-tu": {
        "file": CENSUS_RS,
        "anchor": "if callee_defined_here(&f, defined).is_some()",
        "lifted": "if false && callee_defined_here(&f, defined).is_some()",
        "raises": "callee-defined-in-tu",
    },
    "data-sym-strlit-fenced": {
        "file": CENSUS_RS,
        "anchor": "if f.data_syms\n",
        "lifted": "if false && f.data_syms\n",
        "raises": "data-sym-strlit-fenced",
    },
    "fn-varargs": {
        "file": CENSUS_RS,
        "anchor": "let verdict = if varargs {",
        "lifted": "let verdict = if false && varargs {",
        "raises": "fn-varargs",
    },
}

# Head keys raised INSIDE `parse_segment_detail`. Neither mechanism can lift
# these; a slot terminating on one is UNLIFTABLE, and saying so is the point.
# The test is structural rather than a name list: a key is liftable iff some
# rung in the ladder is able to target it.
PARSE_LAYER_NOTE = (
    "raised inside parse_segment_detail; lifting it means writing the decoder "
    "it names, which this tool does not do"
)


def die(msg, code=4):
    sys.stderr.write("joint_ladder: VOID — %s\n" % msg)
    sys.exit(code)


def run(cmd, env, cwd=None):
    return subprocess.run(cmd, env=env, cwd=cwd, capture_output=True, text=True)


def read_tsv(path):
    """One rung's per-slot verdicts for one TU: {index: (in_class, key, completeness)}."""
    slots = {}
    with open(path) as fh:
        for line in fh:
            if line.startswith("#") or line.startswith("index\t"):
                continue
            p = line.rstrip("\n").split("\t")
            if len(p) < 4:
                continue
            slots[int(p[0])] = (p[1] == "1", p[2], p[3])
    return slots


def build(tree, env, log):
    r = run(["cargo", "build", "--release", "-p", "c2-harness", "--bin", "c2rs"], env, cwd=tree)
    with open(log, "w") as fh:
        fh.write(r.stdout + r.stderr)
    if r.returncode != 0:
        die("cargo build failed in %s (see %s)" % (tree, log), 2)
    exe = os.path.join(tree, "target", "release", "c2rs")
    if not os.access(exe, os.X_OK):
        die("no c2rs at %s after a successful build" % exe, 2)
    return exe


def census_rung(exe, tus, args, env, outdir, relax):
    """Run one rung over the whole population. Returns {src: {index: (...)}}.

    Enforces the empty-rung rule: any SKIP, any nonzero exit, or any zero
    denominator component is a VOID and exits 4.
    """
    os.makedirs(outdir, exist_ok=True)
    got, skips = {}, 0
    for src in tus:
        tsv = os.path.join(outdir, src.replace("/", "_") + ".tsv")
        cmd = [exe, "census", src, "--flags-file", os.path.abspath(args.flags),
               "--cwd", os.path.abspath(args.cwd), "--tsv", tsv]
        if relax is not None:
            cmd += ["--relax", str(relax)]
        r = run(cmd, env)
        blob = r.stdout + r.stderr
        if "SKIP: toolchain absent" in blob:
            skips += 1
        if r.returncode != 0:
            die("census exited %d on %s\n%s" % (r.returncode, src, blob[-2000:]), 4)
        if not os.path.exists(tsv):
            die("census wrote no --tsv for %s (graded nothing)" % src, 4)
        got[src] = read_tsv(tsv)
    tus_graded = len(got)
    slots = sum(len(v) for v in got.values())
    keys = {k for v in got.values() for (_, k, _) in v.values()}
    print("      denominator: TUs=%d slots=%d distinct-keys=%d skips=%d"
          % (tus_graded, slots, len(keys), skips))
    if skips:
        die("%d rung invocations printed `SKIP: toolchain absent` — exits 0 and grades "
            "NOTHING (#3470). A ladder cannot tell that from an inert clause." % skips, 4)
    if tus_graded == 0 or slots == 0 or len(keys) == 0:
        die("zero denominator component (TUs=%d slots=%d keys=%d) — a rung that graded "
            "nothing is not a rung that moved nothing" % (tus_graded, slots, len(keys)), 4)
    return got


def make_scratch(repo, scratch, lifts):
    """A fresh copy of the committed tree with `lifts` applied. Never writes to `repo`."""
    if os.path.exists(scratch):
        # Keep target/ across rungs so a rung is a ~15 s incremental build, not
        # a cold one. Only the sources are refreshed.
        for name in os.listdir(scratch):
            if name != "target":
                p = os.path.join(scratch, name)
                shutil.rmtree(p) if os.path.isdir(p) else os.remove(p)
    else:
        os.makedirs(scratch)
    r = subprocess.run("git archive --format=tar HEAD | tar -x -C %s" % scratch,
                       shell=True, cwd=repo, capture_output=True, text=True)
    if r.returncode != 0:
        die("git archive into scratch failed: %s" % r.stderr, 2)
    applied = []
    for name in lifts:
        spec = SRC_LIFTS[name]
        p = os.path.join(scratch, spec["file"])
        with open(p) as fh:
            txt = fh.read()
        n = txt.count(spec["anchor"])
        if n != 1:
            die("lift %r: anchor occurs %d times in %s, expected exactly 1 — the clause "
                "moved and the lift would be silently wrong" % (name, n, spec["file"]), 2)
        with open(p, "w") as fh:
            fh.write(txt.replace(spec["anchor"], spec["lifted"]))
        applied.append(name)
    return applied


def committed_tree_clean(repo):
    r = subprocess.run(["git", "status", "--porcelain", "--", "crates/"],
                       cwd=repo, capture_output=True, text=True)
    return [l for l in r.stdout.splitlines() if l.strip()]


def stamp(d):
    def g(*a):
        return subprocess.run(["git", "-C", d] + list(a),
                              capture_output=True, text=True).stdout
    h = g("rev-parse", "HEAD").strip()[:12] or "UNVERSIONED"
    por = hashlib.md5("".join(sorted(g("status", "--porcelain", "-uno").splitlines()))
                      .encode()).hexdigest()[:10]
    con = hashlib.md5(g("diff", "HEAD").encode()).hexdigest()[:10]
    return "%s+%s+%s" % (h, por, con)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tus", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--cwd", default="../dc3-decomp")
    ap.add_argument("--flags", default="work/dc3-workload/flags.txt")
    ap.add_argument("--repo", default=".")
    ap.add_argument("--rungs", default="",
                    help="comma-separated rung specs, e.g. 'relax:1,src:opt-mode+callee-defined-in-tu'. "
                         "Default: relax:1 then every src lift, cumulatively.")
    args = ap.parse_args()

    repo = os.path.abspath(args.repo)
    out = os.path.abspath(args.out)
    os.makedirs(out, exist_ok=True)
    tus = [l.strip() for l in open(args.tus) if l.strip()]
    if not tus:
        die("--tus names no TUs", 2)

    env = dict(os.environ)
    for k, need in (("C2RS_COMPILERS", True), ("C2RS_WIBO", True)):
        if not env.get(k):
            die("%s is unset. Both arms of every rung must resolve the toolchain from ONE "
                "explicit resolution (#3470); a scratch-tree binary resolves `compilers/` "
                "relative to ITS OWN tree and would SKIP silently." % k, 2)
        _ = need

    wl = os.path.join(repo, args.cwd)
    stamp_before = stamp(wl)

    if args.rungs:
        specs = args.rungs.split(",")
    else:
        cum, specs = [], ["relax:1"]
        for name in SRC_LIFTS:
            cum.append(name)
            specs.append("relax:1+src:" + "+".join(cum))

    print("== joint_ladder ==")
    print("  repo      %s" % repo)
    print("  workload  %s  %s" % (args.cwd, stamp_before))
    print("  TUs       %d (from %s)" % (len(tus), args.tus))
    print("  rungs     %d: %s" % (len(specs) + 1, ["rung0-identity"] + specs))

    # ---- rung 0: the identity control, in the SCRATCH tree with zero lifts ----
    scratch = os.path.join(out, "scratch")
    print("\n-- rung 0 (scratch tree, ZERO lifts) — the identity control --")
    make_scratch(repo, scratch, [])
    exe0 = build(scratch, env, os.path.join(out, "build_rung0.log"))
    base = census_rung(exe0, tus, args, env, os.path.join(out, "rung0"), None)

    print("-- committed-tree control (the incumbent binary) --")
    exe_c = os.path.join(repo, "target", "release", "c2rs")
    if not os.access(exe_c, os.X_OK):
        exe_c = build(repo, env, os.path.join(out, "build_committed.log"))
    comm = census_rung(exe_c, tus, args, env, os.path.join(out, "committed"), None)

    diffs = [(s, i) for s in tus for i in base.get(s, {})
             if base[s][i][1] != comm.get(s, {}).get(i, (None, None, None))[1]]
    print("  K1 identity (scratch rung 0 vs committed tree): %d slot(s) differ of %d"
          % (len(diffs), sum(len(v) for v in base.values())))
    if diffs:
        for s, i in diffs[:10]:
            print("     %s [%d] scratch=%s committed=%s" % (s, i, base[s][i][1], comm[s][i][1]))
        die("K1 RED — the scratch build does not reproduce the committed tree. The ladder "
            "is VOID: every later rung's movement would be unattributable.", 4)

    rows = {(s, i): [base[s][i]] for s in tus for i in base.get(s, {})}
    rung_names = ["rung0"]

    for n, spec in enumerate(specs, start=1):
        relax, lifts = None, []
        for part in spec.split("+"):
            if part.startswith("relax:"):
                relax = int(part.split(":", 1)[1])
            elif part.startswith("src:"):
                lifts = part.split(":", 1)[1].split("+")
            elif part in SRC_LIFTS:
                lifts.append(part)
        for name in lifts:
            if name not in SRC_LIFTS:
                die("unknown src lift %r; known: %s" % (name, ", ".join(SRC_LIFTS)), 2)
        print("\n-- rung %d: %s --" % (n, spec))
        applied = make_scratch(repo, scratch, lifts)
        print("     lifts applied to scratch: relax=%s src=%s" % (relax, applied or "none"))
        exe = build(scratch, env, os.path.join(out, "build_rung%d.log" % n))
        got = census_rung(exe, tus, args, env, os.path.join(out, "rung%d" % n), relax)
        dirty = committed_tree_clean(repo)
        print("     K10 committed-tree crates/ clean: %s" % ("YES" if not dirty else dirty))
        if dirty:
            die("the lifted state ESCAPED into the committed tree: %s" % dirty, 3)
        moved = 0
        for s in tus:
            for i in base.get(s, {}):
                cur = got.get(s, {}).get(i)
                if cur is None:
                    die("slot %s[%d] vanished at rung %d — the segmentation moved and the "
                        "slot sequence is not a sequence" % (s, i, n), 4)
                if cur[1] != rows[(s, i)][-1][1]:
                    moved += 1
                rows[(s, i)].append(cur)
        print("     DISCRIMINATING CELLS: %d slot(s) moved vs the previous rung, of %d walked"
              % (moved, len(rows)))
        rung_names.append(spec)

    stamp_after = stamp(wl)
    print("\n  workload stamp before %s" % stamp_before)
    print("  workload stamp after  %s" % stamp_after)
    if stamp_before != stamp_after:
        die("the workload moved MID-LADDER — every rung is against a different corpus", 3)

    with open(os.path.join(out, "ladder.json"), "w") as fh:
        json.dump({"rungs": rung_names, "stamp": stamp_before,
                   "slots": {"%s#%d" % k: v for k, v in rows.items()}}, fh, indent=1)
    print("\n  wrote %s/ladder.json (%d slots x %d rungs)"
          % (out, len(rows), len(rung_names)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
