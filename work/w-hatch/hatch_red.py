#!/usr/bin/env python3
"""Make every `hatch.py` refusal go RED on purpose, and print it verbatim.

    python3 work/w-hatch/hatch_red.py            # runs every arm
    python3 work/w-hatch/hatch_red.py --list     # names the arms, runs nothing

Board **#1380**, lane `w-hatch`. `w-instr`'s `work/w-instr/hatch_red.py` is the
prototype and this is the same instrument aimed at `revert` instead of `apply`,
plus `apply`'s two arms carried across so the whole file is covered by one run.

A guard nobody has seen fire is a guard nobody has tested. That rule has caught
three defects on this project in two days, so every arm below prints the FULL
verbatim refusal and the report is written from what the arms printed.

# The two mutation traps, and how this file forecloses each

**Trap A — an early guard can make a later assertion unreachable.** If the
mutation that is supposed to fire gate N also trips gate N-1, gate N never runs
and its arm passes on somebody else's refusal.

> Every DIRTY arm here is produced by writing a **genuine foreign edit into a
> real `crates/` file**, never by breaking the un-apply machinery — so the
> quantities the earlier gates read are held fixed and only the one under test
> moves. The two guards that stand *after* the destructive step
> (`HATCH-CHECKOUT-FAILED`, `HATCH-RESIDUE`) are structurally unreachable on a
> well-formed tree; they are fired through `hatch.py`'s one declared seam
> (`_checkout`) and are printed as **INJECTED**. A guard fired through a seam is
> weaker evidence than one fired by a real defect and the report says which.

**Trap B — a shared message prefix lets a later gate's refusal satisfy an
earlier case's expectation.** A lane had **two of six mutations pass silently**
for exactly this reason.

> Three things, and the third is the one that would have caught it:
>   1. every refusal in `hatch.py` leads with its own word;
>   2. this file asserts on the **leading whitespace-token of a line**, never on
>      a substring anywhere in the output;
>   3. every red arm additionally asserts that **every other arm's word is
>      ABSENT**, unless the co-occurrence is declared in the arm's own `also`
>      set. A collapse back to a shared prefix fails every arm at once instead
>      of silently passing two of them.

# What is NOT touched

Nothing under `crates/` is left modified: every arm restores the tree with
`git checkout --` in a `finally`, and the run refuses to report success if the
final `git diff` is non-empty. Nothing here is ever committed into `crates/`.

# 2026-08-08, lane w-hatchroot — THIS FILE HAD #1460 TOO, AND WORSE (#1494)

`ROOT` here was `dirname²(dirname(__file__))` — the same `__file__`-derived
root board **#1460** found in `hatch.py`, in the harness written to test
`hatch.py`. It is the *more* dangerous of the two: `hatch.py revert` refuses on
a tree carrying anything that is not the hatch, and `restore()` below is an
unconditional `git checkout -- crates/`. Run from a worktree by relative path,
this file would have discarded the **main** repository's unstaged `crates/`
work — #1380's incident, aimed by #1460's mechanism, from the instrument built
to prevent #1380.

Resolved the same way `hatch.py` now resolves it, with its **own** leading word
(`HATCHRED-FOREIGN-ROOT`, `HATCHRED-NOREPO`) deliberately kept **out** of
`ALL_WORDS`: this refusal aborts the whole run before any arm exists, and a word
an arm could match is a word that could satisfy an arm's expectation (trap B).
"""

import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
SELF = os.path.abspath(__file__)


def _toplevel(d):
    if not os.path.isdir(d):
        return None
    r = subprocess.run(["git", "rev-parse", "--show-toplevel"], cwd=d,
                       stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    out = r.stdout.decode("utf-8", "replace").strip()
    if r.returncode != 0 or not out:
        return None
    return os.path.realpath(out)


def _resolve_root():
    """The checkout these arms are allowed to write into and restore — #1460.

    From the INVOKING CWD, never from this file's own path, and this file must
    belong to the same checkout. A lexical containment test is not enough here
    and the reason is this repository's own layout: worktrees live at
    `<main>/.claude/worktrees/<name>`, INSIDE the main repo's path, so
    `commonpath` calls a worktree's copy "inside" the main checkout.
    """
    cwd = os.getcwd()
    here = _toplevel(cwd)
    if here is None:
        sys.exit("HATCHRED-NOREPO — the invoking directory is not inside a git "
                 "checkout, and every arm here restores `crates/` with "
                 "`git checkout --`.\n  invoking cwd: %s\n  this script : %s"
                 % (cwd, SELF))
    there = _toplevel(os.path.dirname(SELF))
    if there != here:
        sys.exit(
            "HATCHRED-FOREIGN-ROOT — this script belongs to a DIFFERENT checkout "
            "from the one you\nare standing in, and `restore()` is an "
            "unconditional `git checkout -- crates/`.\n"
            "  you are standing in   : %s\n"
            "  this script belongs to: %s\n"
            "  this script           : %s\n\n"
            "  Board #1460 in the file written to test board #1380. Run the copy "
            "that lives\n  in the tree you mean to test." % (here, there, SELF))
    return here


ROOT = _resolve_root()
# `W_HATCH_TARGET` points the whole harness at a DIFFERENT `hatch.py`. It exists
# for exactly one purpose: running these arms against **master's unguarded
# version**, so the run that says every arm passes is accompanied by a run that
# says every arm fails. A red test nobody has watched fail on the pre-fix code is
# a red test that might be asserting nothing — the same argument, one level up,
# that makes this file exist at all.
HATCH = os.environ.get("W_HATCH_TARGET") or os.path.join(
    ROOT, "work", "w-front3", "hatch.py")

# #1380's own casualty. The incident was a routine `revert` discarding an
# unstaged fix to this exact file, so the reproduction uses it by name.
VICTIM = "crates/c2-il/src/func/body/shapes/calls.rs"
FOREIGN = "\n// w-hatch RED PROBE — a foreign unstaged edit. Not the hatch's.\n"

ALL_WORDS = [
    "HATCH-DIRTY", "HATCH-UNREADABLE", "HATCH-UNTRACKED",
    "HATCH-CHECKOUT-FAILED", "HATCH-RESIDUE", "HATCH-FORCED",
    "HATCH-DRIFT", "HATCH-PAID-MISSING",
    # lane w-hatchroot, board #1460 — the root resolution's two.
    "HATCH-FOREIGN-ROOT", "HATCH-NOREPO",
]

# Scratch trees the #1460 arms need. Created in their setups and removed in the
# module's `finally`, never under `crates/` and never inside this checkout: a
# `git init` inside the repo would be a nested repository that `git status`
# reports and that the next lane has to reason about.
SCRATCH = []


# --------------------------------------------------------------------------
# plumbing
# --------------------------------------------------------------------------
class Cap:
    """Tee stderr so the verbatim refusal is BOTH printed and inspectable."""

    def __init__(self):
        self.buf, self.real = [], sys.stderr

    def write(self, s):
        self.buf.append(s)
        self.real.write(s)

    def flush(self):
        self.real.flush()

    def getvalue(self):
        return "".join(self.buf)

    def reset(self):
        self.buf = []


CAP = Cap()


def load():
    """A FRESH module per arm, so one arm's mutation cannot leak into the next."""
    spec = importlib.util.spec_from_file_location("hatch_probe", HATCH)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def git(*args, **kw):
    return subprocess.run(["git"] + list(args), cwd=ROOT,
                          capture_output=True, text=True, **kw)


def crates_diff():
    return git("diff", "--name-only", "--", "crates/").stdout.split()


def restore():
    git("checkout", "--", "crates/")


def leading_words(text):
    """The set of `HATCH-*` tokens that START a line (after indentation).

    Substring matching is what trap B exploits: `HATCH-DIRTY` appearing inside
    a sentence in some other refusal's explanatory paragraph would satisfy an
    arm written for it. Only a LEADING token counts.
    """
    got = set()
    for ln in text.splitlines():
        tok = ln.strip().split(" ")[0] if ln.strip() else ""
        if tok in ALL_WORDS:
            got.add(tok)
    return got


def hatch_tree(m):
    """Apply the hatch, or die — a red run on a tree that is not hatched is not
    the control it claims to be."""
    try:
        m.apply()
    except SystemExit:
        sys.stderr.write("SETUP FAILED: `hatch.py apply` refused; cannot build "
                         "a hatched tree for this arm.\n")
        raise


def dirty_tree():
    with open(os.path.join(ROOT, VICTIM), "a") as fh:
        fh.write(FOREIGN)


def foreign_present():
    with open(os.path.join(ROOT, VICTIM)) as fh:
        return FOREIGN in fh.read()


# --------------------------------------------------------------------------
# the arms
# --------------------------------------------------------------------------
RESULTS = []


def arm(name, setup, run, want, also=(), injected=False, expect_rc=None,
        note="", check=None):
    """One arm. `want` is the leading word, or None for a GREEN control."""
    print("=" * 74)
    print("ARM %s — expect %s%s"
          % (name, ("leading word " + want) if want else "GREEN, no refusal word",
             "   [INJECTED through the _checkout seam]" if injected else ""))
    if note:
        print("     " + note)
    print("=" * 74)
    CAP.reset()
    real_stderr, sys.stderr = sys.stderr, CAP
    ok_extra, rc = True, 0
    try:
        restore()
        m = load()
        try:
            setup(m)
        except SystemExit as e:
            # A setup that cannot build the tree the arm needs is an ARM
            # FAILURE, not a reason to abandon the run: the counterfactual pass
            # over master's unguarded file must still produce a full table.
            sys.stderr.write("SETUP FAILED (%s) — arm cannot run\n" % (e.code,))
            sys.stderr = real_stderr
            restore()
            print("\n  SETUP FAILED — this arm could not build its tree.")
            print("  *** ARM FAILED ***\n")
            RESULTS.append((name, False, want or "(green)", injected))
            return False
        before = git("diff", "--", "crates/").stdout
        try:
            run(m)
        except SystemExit as e:
            rc = e.code if isinstance(e.code, int) else 1
        except Exception as e:                      # noqa: BLE001 - see below
            # A version without the guard raises rather than refusing (master's
            # `revert` takes no `force`). That is a DISTINCT outcome from a
            # refusal and is reported as one, never folded into `rc != 0`.
            rc = -1
            sys.stderr.write("RAISED %s: %s\n" % (type(e).__name__, e))
        after = git("diff", "--", "crates/").stdout
        if check is not None:
            ok_extra = check(m, before, after)
    finally:
        sys.stderr = real_stderr
        text = CAP.getvalue()
        restore()

    words = leading_words(text)
    if want is None:
        ok_word = not words
        ok_excl = True
    else:
        ok_word = want in words
        allowed = {want} | set(also)
        ok_excl = not (words - allowed)
    ok_rc = True if expect_rc is None else (rc == expect_rc)
    ok = ok_word and ok_excl and ok_rc and ok_extra

    print("\n  rc                      : %d%s"
          % (rc, "" if ok_rc else "  *** expected %d ***" % expect_rc))
    print("  leading HATCH-* words   : %s" % (", ".join(sorted(words)) or "NONE"))
    print("  expected word present   : %s"
          % ("YES" if ok_word else "NO — ARM FAILED"))
    print("  no OTHER arm's word     : %s"
          % ("YES" if ok_excl else "NO — ARM FAILED, prefix collapse (trap B)"))
    print("  arm postcondition       : %s"
          % ("YES" if ok_extra else "NO — ARM FAILED"))
    print("  crates/ clean afterwards: %s"
          % ("YES" if not crates_diff() else "NO — ARM FAILED"))
    print()
    RESULTS.append((name, ok, want or "(green)", injected))
    return ok


def preserved(what):
    """Postcondition: the arm refused and the tree is EXACTLY as it was."""
    def _c(m, before, after):
        same = before == after
        print("\n  [postcondition] working tree byte-identical across the "
              "refusal: %s" % ("YES — %d bytes of diff, unchanged" % len(after)
                               if same else "NO — THE TREE WAS WRITTEN"))
        if not foreign_present() and what == "foreign":
            print("  [postcondition] the foreign edit SURVIVED: NO — IT WAS EATEN")
            return False
        if what == "foreign":
            print("  [postcondition] the foreign edit SURVIVED: YES")
        return same
    return _c


def destroyed(m, before, after):
    gone = not foreign_present()
    print("\n  [postcondition] --force DESTROYED the foreign edit, deliberately:"
          " %s" % ("YES" if gone else "NO — --force did nothing"))
    print("  [postcondition] crates/ diff after the forced revert: %s"
          % (after.strip() or "EMPTY"))
    return gone and not after.strip()


def clean_after(m, before, after):
    print("\n  [postcondition] crates/ diff after the revert: %s"
          % (after.strip() or "EMPTY"))
    return not after.strip()


# ---- the #1460 root-resolution arms ---------------------------------------
# `resolve_root` takes the cwd and the script path as ARGUMENTS, so these arms
# hand it the exact pair the incident was produced by — cwd in one checkout,
# script in another — rather than monkeypatching an internal. That makes them
# real defect reproductions and not INJECTED ones, which is the distinction the
# `_checkout` seam arms have to declare.
PROBE = {}


def _scratch(git=True):
    d = tempfile.mkdtemp(prefix="w-hatchroot-")
    SCRATCH.append(d)
    if git:
        subprocess.run(["git", "init", "-q", d], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return d


def s_foreign_root(m):
    """A REAL second git repository, and it has to be real.

    Trap A in person: `HATCH-NOREPO` is checked FIRST, so building "a tree
    somewhere else" out of a bare temp directory fires that word instead and
    this arm would pass on somebody else's refusal. The cwd here is a checkout;
    the only thing that differs from the green control is WHICH checkout.
    """
    PROBE.clear()
    PROBE["cwd"] = _scratch(git=True)


def s_norepo(m):
    d = _scratch(git=False)
    if _toplevel(d) is not None:
        # Honest rather than convenient: if this box's $TMPDIR is itself inside
        # a checkout, the arm cannot build the tree it needs and says so.
        raise SystemExit("temp dir %s is inside a git checkout (%s) — this arm "
                         "cannot build a non-repository cwd here" % (d, _toplevel(d)))
    PROBE.clear()
    PROBE["cwd"] = d


def s_subdir(m):
    """THE GREEN CONTROL, and it is the one most likely to break.

    A guard that refuses a cross-tree invocation and ALSO refuses
    `cd crates && python3 ../work/w-front3/hatch.py check` is a guard the next
    lane deletes. `crates/` is a real subdirectory of the real checkout.
    """
    PROBE.clear()
    PROBE["cwd"] = os.path.join(ROOT, "crates")


def r_resolve(m):
    PROBE["got"] = m.resolve_root(script=HATCH, cwd=PROBE["cwd"])


def resolved_to_root(m, before, after):
    got = PROBE.get("got")
    ok = got is not None and os.path.realpath(got) == os.path.realpath(ROOT)
    print("\n  [postcondition] resolve_root(cwd=%s)" % PROBE.get("cwd"))
    print("  [postcondition]   -> %s" % got)
    print("  [postcondition] equals this checkout's root: %s"
          % ("YES" if ok else "NO — ARM FAILED, it resolved somewhere else"))
    return ok and before == after


# ---- setups ---------------------------------------------------------------
def s_nothing(m):
    pass


def s_dirty(m):
    dirty_tree()


def s_hatch(m):
    hatch_tree(m)


def s_hatch_dirty(m):
    hatch_tree(m)
    dirty_tree()


def s_unreadable(m):
    p = "crates/c2-il/src/func/body/shapes/NOT_A_FILE_w_hatch.rs"
    m.FILES = m.FILES + [p]


def s_untracked(m):
    p = "work/w-hatch/.untracked_probe.tmp"
    with open(os.path.join(ROOT, p), "w") as fh:
        fh.write("not tracked, not staged\n")
    m.FILES = m.FILES + [p]


def s_checkout_fails(m):
    class R:
        returncode, stderr = 128, "fatal: injected by work/w-hatch/hatch_red.py\n"
    m._checkout = lambda paths: R()


def s_residue(m):
    hatch_tree(m)

    class R:
        returncode, stderr = 0, ""
    m._checkout = lambda paths: R()      # a checkout that restores NOTHING


def s_drift(m):
    """#1322's shape, re-injected: a needle that is not in the tree. Placed
    FIRST so it cannot be confused with 'the last edit failed'."""
    c, f, n, r = m.EDITS[1]
    m.EDITS = [m.EDITS[0], (c, f, n + "\n// NOT IN THE TREE\n", r)] + m.EDITS[2:]


def s_paid_missing(m):
    m.RETIRED = [(e, p, "a witness string that is not in the file", a, note)
                 for e, p, w, a, note in m.RETIRED]


MASTER_SNAP = os.path.join(HERE, ".hatch_master_snapshot.py")


def main():
    global HATCH
    if "--list" in sys.argv:
        for n, w in ARMS:
            print("%-22s %s" % (n, w))
        return 0
    if "--master" in sys.argv:
        # THE COUNTERFACTUAL. Run these same arms against the version of
        # `hatch.py` this lane started from, so "every arm passes" is quoted
        # beside "every arm fails" on the unrepaired file. Expected to exit 1.
        base = sys.argv[sys.argv.index("--master") + 1] \
            if len(sys.argv) > sys.argv.index("--master") + 1 else "2b1c89da"
        blob = git("show", "%s:work/w-front3/hatch.py" % base)
        if blob.returncode != 0:
            sys.exit("cannot read work/w-front3/hatch.py at %s" % base)
        with open(MASTER_SNAP, "w") as fh:
            fh.write(blob.stdout)
        HATCH = MASTER_SNAP
        print("*** COUNTERFACTUAL: running against work/w-front3/hatch.py "
              "as of %s (the UNREPAIRED file). Every arm below is EXPECTED "
              "to fail. ***\n" % base)
    print("hatch.py RED-TEST — board #1380, lane w-hatch")
    print("tree state before any arm: %s\n"
          % (", ".join(crates_diff()) or "crates/ diff: EMPTY"))
    if crates_diff():
        sys.exit("REFUSING to run on a dirty tree — every arm writes to crates/ "
                 "and restores it, so a pre-existing edit would be destroyed by "
                 "this very script. Commit or stash first.")

    # ---- the RED arms ----------------------------------------------------
    arm("R1 DIRTY-NOHATCH", s_dirty, lambda m: m.revert(), "HATCH-DIRTY",
        expect_rc=3, check=preserved("foreign"),
        note="an unstaged foreign edit and NO hatch — #1380's incident exactly")
    arm("R2 DIRTY+HATCH", s_hatch_dirty, lambda m: m.revert(), "HATCH-DIRTY",
        expect_rc=3, check=preserved("foreign"),
        note="fully hatched AND dirty: the guard must separate the two")
    arm("R3 UNREADABLE", s_unreadable, lambda m: m.revert(), "HATCH-UNREADABLE",
        expect_rc=3, check=preserved("tree"),
        note="a file in FILES that cannot be read — must NOT read as DIRTY")
    arm("R4 UNTRACKED", s_untracked, lambda m: m.revert(), "HATCH-UNTRACKED",
        expect_rc=3, check=preserved("tree"),
        note="a file with no stage-0 index entry: `git checkout --` has nothing "
             "to restore")
    arm("R5 CHECKOUT-FAILED", s_checkout_fails, lambda m: m.revert(),
        "HATCH-CHECKOUT-FAILED", expect_rc=4, injected=True,
        note="unreachable on a well-formed tree; fired through the _checkout seam")
    arm("R6 RESIDUE", s_residue, lambda m: m.revert(), "HATCH-RESIDUE",
        expect_rc=5, injected=True,
        note="the POSTCONDITION: a checkout that silently restored nothing")
    arm("A1 DRIFT", s_drift, lambda m: m.apply(), "HATCH-DRIFT",
        expect_rc=2, check=preserved("tree"),
        note="apply's #1322 arm — its own word now, not a shared prefix")
    arm("A2 PAID-MISSING", s_paid_missing, lambda m: m.apply(),
        "HATCH-PAID-MISSING", expect_rc=2, check=preserved("tree"),
        note="a RETIRED entry whose paid_witness is gone — a DIFFERENT defect "
             "from DRIFT and it must say so")

    # ---- the ROOT-RESOLUTION arms (board #1460, lane w-hatchroot) ---------
    # #1460 IS THE INCIDENT, NOT A HYPOTHETICAL: a worktree lane ran the main
    # repository's `hatch.py` by relative path and hatched the MAIN tree, while
    # `git status` in the worktree read empty. The tell was `sha256sum` — the
    # "hatched" binary was byte-identical to the unhatched one — and without it a
    # climb would have published the unhatched ladder table as *"the hatch moved
    # nothing"*. Both arms hand `resolve_root` the real argument pair; neither is
    # INJECTED.
    arm("A3 FOREIGN-ROOT", s_foreign_root, r_resolve, "HATCH-FOREIGN-ROOT",
        expect_rc=6, check=preserved("tree"),
        note="cwd in a REAL second git checkout, script in this one — #1460 "
             "exactly. A lexical containment test does not catch this "
             "repository's own layout, where worktrees live INSIDE the main "
             "repo's path")
    arm("A4 NOREPO", s_norepo, r_resolve, "HATCH-NOREPO",
        expect_rc=7, check=preserved("tree"),
        note="cwd in no checkout at all — a DIFFERENT defect from FOREIGN-ROOT "
             "and it must say so, because it is checked FIRST and would "
             "otherwise satisfy that arm (trap A)")

    # ---- the FORCE arm ---------------------------------------------------
    arm("F1 FORCE", s_hatch_dirty, lambda m: m.revert(force=True),
        "HATCH-FORCED", also=("HATCH-DIRTY",), expect_rc=0, check=destroyed,
        note="the escape hatch: reverts anyway, names what it destroys first")

    # ---- the CONTROLS, which must NOT fire -------------------------------
    arm("C1 HATCH-ONLY", s_hatch, lambda m: m.revert(), None,
        expect_rc=0, check=clean_after,
        note="THE CONTROL. A fully hatched tree is not dirty and must revert "
             "silently")
    arm("C2 CLEAN", s_nothing, lambda m: m.revert(), None,
        expect_rc=0, check=clean_after,
        note="a clean tree: revert is a no-op and still must not fire")
    arm("C3 ROOT-FROM-SUBDIR", s_subdir, r_resolve, None,
        expect_rc=0, check=resolved_to_root,
        note="THE CONTROL FOR #1460's GUARD, and the one most likely to break: "
             "`cd crates && python3 ../work/w-front3/hatch.py` must still "
             "resolve to THIS checkout's root. A guard that refuses the "
             "ordinary invocation is a guard the next lane deletes")

    # ---- the report ------------------------------------------------------
    print("=" * 74)
    red = [(n, ok, w, inj) for n, ok, w, inj in RESULTS if w != "(green)"]
    grn = [(n, ok, w, inj) for n, ok, w, inj in RESULTS if w == "(green)"]
    print("RED arms (%d) — each fired its OWN word and no other arm's:" % len(red))
    for n, ok, w, inj in red:
        print("  %-22s %-24s %s%s" % (n, w,
              "RED as expected" if ok else "*** ARM FAILED ***",
              "   [INJECTED]" if inj else ""))
    print("\nGREEN controls (%d) — these must NOT fire:" % len(grn))
    for n, ok, w, inj in grn:
        print("  %-22s %-24s %s" % (n, "", "GREEN as required" if ok
                                    else "*** CONTROL FIRED — FAILED ***"))
    words = sorted({w for _, _, w, _ in red})
    print("\ndistinct leading words exercised: %d of %d (%s)"
          % (len(words), len(ALL_WORDS), ", ".join(words)))
    print("final crates/ diff: %s" % (", ".join(crates_diff()) or "EMPTY"))
    print("=" * 74)
    bad = [n for n, ok, _, _ in RESULTS if not ok]
    if bad or crates_diff():
        print("FAILED: %s" % ", ".join(bad or ["tree left dirty"]))
        return 1
    print("ALL %d ARMS PASS — %d red, %d green" % (len(RESULTS), len(red), len(grn)))
    return 0


ARMS = [("R1 DIRTY-NOHATCH", "HATCH-DIRTY"), ("R2 DIRTY+HATCH", "HATCH-DIRTY"),
        ("R3 UNREADABLE", "HATCH-UNREADABLE"), ("R4 UNTRACKED", "HATCH-UNTRACKED"),
        ("R5 CHECKOUT-FAILED", "HATCH-CHECKOUT-FAILED"),
        ("R6 RESIDUE", "HATCH-RESIDUE"), ("A1 DRIFT", "HATCH-DRIFT"),
        ("A2 PAID-MISSING", "HATCH-PAID-MISSING"),
        ("A3 FOREIGN-ROOT", "HATCH-FOREIGN-ROOT"),
        ("A4 NOREPO", "HATCH-NOREPO"), ("F1 FORCE", "HATCH-FORCED"),
        ("C1 HATCH-ONLY", "(green)"), ("C2 CLEAN", "(green)"),
        ("C3 ROOT-FROM-SUBDIR", "(green)")]

if __name__ == "__main__":
    try:
        sys.exit(main())
    finally:
        restore()
        for p in (os.path.join(ROOT, "work/w-hatch/.untracked_probe.tmp"),
                  MASTER_SNAP):
            if os.path.exists(p):
                os.remove(p)
        for d in SCRATCH:
            shutil.rmtree(d, ignore_errors=True)
