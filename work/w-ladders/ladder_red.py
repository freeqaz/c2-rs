#!/usr/bin/env python3
"""Make `ladder.py`'s NEW guards go RED on purpose, and print them verbatim.

    python3 work/w-ladders/ladder_red.py           # every arm
    python3 work/w-ladders/ladder_red.py --list    # names the arms, runs nothing

Lane `w-ladders`. `work/w-hatch/hatch_red.py` is the model and this is the same
discipline aimed at the two rules this lane added to `work/w-front3/ladder.py`:

  * `pinned_opcodes()` — derives `chain_skip_form`'s width table FROM THE TREE
    and refuses (`LADDER-NOWIDTHTABLE`) rather than returning an empty set;
  * `is_rename()` — a grant of an opcode that table has not pinned cannot
    advance the stream one byte, so it is a RENAME and not a rung.

# Why this is a SEPARATE file and not two more arms in `hatch_red.py`

`hatch_red.py` is a `gate.sh` row (board **#1435**) whose arm count the gate reads
from its own `--list`, and every one of its arms writes into `crates/` and
restores it. These arms touch **no** `crates/` file and test a different
instrument. Folding them in would put a new failure mode into a live gate row
**while two other lanes are mid-wave**, for no coverage this file does not give.

**The cost is stated rather than hidden: this file is NOT a `gate.sh` row and is
run by hand**, which is exactly the open second half of board **#1406**. It is
one row's work to wire in and the next lane should; it is not taken here for the
reason above.

# The traps, carried across verbatim from `hatch_red.py`

* **Trap A — an early guard can make a later assertion unreachable.** Each arm
  perturbs exactly ONE thing and the others are held fixed.
* **Trap B — a shared message prefix lets a later gate's refusal satisfy an
  earlier case's expectation.** Every refusal leads with its own word, every
  assertion is on the LEADING TOKEN of a line, and every red arm additionally
  asserts that no other arm's word appears.
* **A green control that cannot go red is not a control**, so the counterfactual
  (`--master`) runs the same arms against `ladder.py` as of the base commit,
  where every red arm is expected to FAIL to fire.
"""

import importlib.util
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
LADDER = os.path.join(ROOT, "work/w-front3/ladder.py")
SNAP = os.path.join(HERE, ".ladder_master_snapshot.py")

ALL_WORDS = ["LADDER-NOWIDTHTABLE"]
RESULTS = []


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def arm(name, run, word, note, also=()):
    """One arm. `word` is None for a GREEN control."""
    print("-" * 74)
    print("ARM %s — %s" % (name, note))
    try:
        out = run()
        rc, text = 0, str(out)
    except SystemExit as e:
        rc, text = 1, str(e.code)
    except Exception as e:                                # noqa: BLE001
        rc, text = 2, "%s: %s" % (type(e).__name__, e)
    print("  rc=%d\n  %s" % (rc, text.replace("\n", "\n  ")))
    lead = {ln.split()[0] for ln in text.splitlines() if ln.split()}
    if word is None:
        ok = rc == 0 and not (set(ALL_WORDS) & lead)
        print("  => %s" % ("GREEN as required" if ok
                           else "*** CONTROL FIRED — FAILED ***"))
    else:
        fired = word in lead or text.startswith(word)
        others = [w for w in ALL_WORDS
                  if w != word and w not in also and (w in lead or w in text)]
        ok = rc != 0 and fired and not others
        print("  => %s%s" % ("RED as expected" if ok else "*** ARM FAILED ***",
                             ("  (LEAKED: %s)" % others) if others else ""))
    RESULTS.append((name, ok, word or "(green)"))


ARMS = [("W1 NO-TABLE-FILE", "LADDER-NOWIDTHTABLE"),
        ("W2 EMPTY-TABLE", "LADDER-NOWIDTHTABLE"),
        ("W3 TRUNCATED-TABLE", "LADDER-NOWIDTHTABLE"),
        ("G1 REAL-TABLE", "(green)"),
        ("G2 RENAME-CLASSIFIES", "(green)")]


def main():
    path = LADDER
    counterfactual = "--master" in sys.argv
    if "--list" in sys.argv:
        for n, w in ARMS:
            print("%-22s %s" % (n, w))
        return 0
    if counterfactual:
        base = sys.argv[sys.argv.index("--master") + 1]
        blob = subprocess.run(["git", "show", "%s:work/w-front3/ladder.py" % base],
                              cwd=ROOT, capture_output=True, text=True)
        if blob.returncode != 0:
            sys.exit("cannot read work/w-front3/ladder.py at %s" % base)
        open(SNAP, "w").write(blob.stdout)
        path = SNAP
        print("*** COUNTERFACTUAL: work/w-front3/ladder.py as of %s (the file "
              "BEFORE this lane). Every RED arm below is EXPECTED to fail — it "
              "has no guard to fire. ***\n" % base)

    print("ladder.py RED-TEST — lane w-ladders")

    def with_expr(text):
        """Run `pinned_opcodes` against a SUBSTITUTED `expr.rs`, in memory.

        The real file is never written to. `EXPR_RS` is repointed at a temp copy
        under `work/`, so a failing arm cannot leave a `crates/` file damaged —
        which is the one way this instrument could hurt a peer lane (#1380).
        """
        m = load(path, "ladder_under_test")
        if not hasattr(m, "pinned_opcodes"):
            raise SystemExit("no pinned_opcodes in this ladder.py")
        tmp = os.path.join(HERE, ".expr_probe.rs")
        if text is None:
            m.EXPR_RS = os.path.join(HERE, ".no-such-file.rs")
        else:
            open(tmp, "w").write(text)
            m.EXPR_RS = tmp
        return m.pinned_opcodes()

    real = open(os.path.join(ROOT, "crates/c2-il/src/func/body/expr.rs")).read()

    arm("W1 NO-TABLE-FILE", lambda: with_expr(None), "LADDER-NOWIDTHTABLE",
        "the width table's file is GONE — the set must not come back empty")
    arm("W2 EMPTY-TABLE",
        lambda: with_expr("fn chain_skip_form(b: u8) -> Option<SkipForm> {\n"
                          "    None\n}\n"),
        "LADDER-NOWIDTHTABLE",
        "the function is there and has NO arms — an empty set would make every "
        "rung read as a RENAME, which is trap 5 pointing at this lane's own "
        "finding")
    arm("W3 TRUNCATED-TABLE",
        lambda: with_expr("fn chain_skip_form(b: u8) -> Option<SkipForm> {\n"
                          "    Some(match b {\n"
                          "        0x41 => Bare,\n        0x4F => Line4F,\n"
                          "        _ => return None,\n    })\n}\n"),
        "LADDER-NOWIDTHTABLE",
        "TWO arms parse — a table that shrank by 44 entries is a DRIFT and must "
        "refuse, not silently reclassify 44 opcodes as unpinned")

    def g1():
        p = with_expr(real)
        assert len(p) >= 40, "parsed only %d" % len(p)
        assert 0xBD in p and 0x4C in p and 0x41 in p, "known pins missing"
        assert 0x00 not in p and 0x1C not in p, "0x00/0x1C must be UNPINNED"
        return "parsed %d pinned opcodes from the real tree" % len(p)

    arm("G1 REAL-TABLE", g1, None,
        "THE CONTROL. The tree's own table parses, and the two ends this lane's "
        "finding rests on are checked POSITIVELY in both directions")

    def g2():
        m = load(path, "ladder_rename")
        if not hasattr(m, "is_rename"):
            raise SystemExit("no is_rename in this ladder.py")
        m.PINNED = with_expr(real)
        cases = [(("sink", "op:00"), True), (("sink", "op:1C"), True),
                 (("sink", "op:13"), True), (("sink", "op:41"), False),
                 (("sink", "op:BD"), False), (("sink", "type"), False),
                 (("hatch", "param-width"), False)]
        bad = [(a, want) for a, want in cases if m.is_rename(*a) != want]
        assert not bad, "misclassified: %s" % bad
        return "7 cases classified, 3 RENAME / 4 rung"

    arm("G2 RENAME-CLASSIFIES", g2, None,
        "the rename rule itself, on both sides: the three bytes six FRONTIER "
        "ladders end on, and four grants that are real rungs")

    print("=" * 74)
    red = [r for r in RESULTS if r[2] != "(green)"]
    grn = [r for r in RESULTS if r[2] == "(green)"]
    for n, ok, w in RESULTS:
        print("  %-22s %-24s %s" % (n, w, "OK" if ok else "*** FAILED ***"))
    print("distinct leading words exercised: %d of %d"
          % (len({w for _, _, w in red}), len(ALL_WORDS)))
    print("=" * 74)
    bad = [n for n, ok, _ in RESULTS if not ok]
    if bad:
        print("FAILED: %s" % ", ".join(bad))
        return 1
    print("ALL %d ARMS PASS — %d red, %d green" % (len(RESULTS), len(red), len(grn)))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    finally:
        for p in (SNAP, os.path.join(HERE, ".expr_probe.rs")):
            if os.path.exists(p):
                os.remove(p)
