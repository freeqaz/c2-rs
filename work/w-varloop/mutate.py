#!/usr/bin/env python3
"""mutate.py — **the must-fail mutations**, run against real `c2`.

Lane **w-varloop**. The six mutations were registered in
`work/w-varloop/PREREG.md` §4 at `e6ab626`, **before the emitter existed**.

This lane ships an accept path, so w-rotate's escape — *"no mutation, and here
is why"* — is not available to it. Each mutation below perturbs **one** rule by
**one** and must turn the oracle grid **red**. A rate that survives its own
mutation is measuring nothing.

Every mutation is applied to the real source, the workspace is rebuilt, the grid
is re-run against real `c2.dll`, and the source is restored — with the restore
in a `finally`, and verified by `git diff --quiet` afterwards, so a crashed run
cannot leave a mutated tree behind and be mistaken for the intact one.

Usage:
    work/w-varloop/mutate.py            # all six
    work/w-varloop/mutate.py M3 M6
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))

EMIT = "crates/c2-core/src/codegen/ptr_walk_chain_loop.rs"
IL = "crates/c2-il/src/func/mod.rs"
BUILD = "crates/c2-core/src/lib.rs"

# Each mutation: (file, old, new, grids to grade, what it must break)
MUTATIONS = {
    # ---- M1: S1, the induction load's slot -------------------------------
    "M1": (EMIT,
           "    let a = usize::from(!(m <= 2 && pv == 0 && !same));",
           "    let a = (usize::from(!(m <= 2 && pv == 0 && !same)) + 1).min(m);",
           "A B C D", "one more chain word before the load"),
    # ---- M2: S2, the record form's slot ----------------------------------
    "M2": (EMIT,
           "        (a + 2..=m + 1)",
           "        (a + 3..=m + 1)",
           "A B C D", "the record slot's load-use distance 2 becomes 3"),
    # ---- M3: S3m, the regime threshold -----------------------------------
    "M3": (IL,
           "        self.pv() == Some(0) && self.producers() >= 4",
           "        self.pv() == Some(0) && self.producers() >= 3",
           "A B C D", "the regime threshold M>=4 becomes M>=3"),
    # ---- M4: S4r, the two chain temps exchanged --------------------------
    "M4": (EMIT,
           """        } else if slot_of(i, a, rec) > rec {
            R_T2
        } else {
            R_T1
        });""",
           """        } else if slot_of(i, a, rec) > rec {
            R_T1
        } else {
            R_T2
        });""",
           "A B C D", "T1 and T2 exchanged"),
    # ---- M5: S5, the commutative operand order ---------------------------
    "M5": (EMIT,
           "            let (first, second) = if i == 0 { (R_CHAR, prev) } else { (prev, R_CHAR) };",
           "            let (first, second) = if i == 0 { (prev, R_CHAR) } else { (R_CHAR, prev) };",
           "A B C D", "the commutative operand order reversed"),
    # ---- M6: BOARD #747 --------------------------------------------------
    #
    # "The body length is fixed at the first loop's M." This is w-sched2's
    # sentence — *a model that hard-codes one interleave is wrong about at least
    # one function of an obj that links* — implemented literally, and it is
    # per-TU by construction because it is applied where the TU's functions are
    # laid out rather than inside the per-function emitter.
    #
    # It must turn `f-1-3`, `f-2-6`, `f-3-8` and `f-4-1` red and leave `f-same`
    # and EVERY single-loop cell green. If it breaks a single-loop cell it has
    # not isolated #747's shape, and the rung says so.
    "M6": (BUILD,
           "        if self.fn_level_linking {",
           """        // MUTATION M6 (board #747): every chain loop in this TU is
        // emitted with the FIRST one's operation list.
        let funcs: Vec<c2_il::IlFunction> = {
            let first = funcs
                .iter()
                .find(|f| f.ptr_walk_chain_loop.is_some())
                .and_then(|f| f.ptr_walk_chain_loop.clone());
            funcs
                .into_iter()
                .map(|mut f| {
                    if f.ptr_walk_chain_loop.is_some() {
                        f.ptr_walk_chain_loop = first.clone();
                    }
                    f
                })
                .collect()
        };
        if self.fn_level_linking {""",
           "A F", "the body length fixed at the first loop's M"),
}


def sh(cmd, **kw):
    return subprocess.run(cmd, shell=True, cwd=REPO, capture_output=True,
                          text=True, **kw)


def apply(path, old, new):
    full = os.path.join(REPO, path)
    s = open(full).read()
    n = s.count(old)
    if n != 1:
        raise SystemExit("ANCHOR NOT UNIQUE in %s: %d occurrences.\n%s"
                         % (path, n, old))
    open(full, "w").write(s.replace(old, new, 1))


def restore(path):
    r = sh("git checkout -- %s" % path)
    if r.returncode:
        raise SystemExit("RESTORE FAILED for %s: %s" % (path, r.stderr))


def grid(grids):
    r = sh("python3 work/w-varloop/vargrid.py --jobs 8 --grid %s" % grids)
    out = r.stdout + r.stderr
    fails = passes = None
    for line in out.splitlines():
        if line.startswith("pass"):
            parts = line.split()
            passes, fails = int(parts[1]), int(parts[3])
    # Only per-cell rows, never the summary line -- a summary that matched its
    # own filter would put "MISMATCHES:" in the list of cells that turned red,
    # which is a small instance of the instrument reporting itself.
    red = [l.split()[0] for l in out.splitlines()
           if l.startswith("  ") and (" FAIL " in l or "MISMATCH  <-" in l)]
    # HOW they failed: wrong bytes is strictly stronger evidence than a refusal.
    wrong = sum(1 for l in out.splitlines() if "MISMATCH  <-" in l)
    return passes, fails, red, out, wrong


def main():
    want = [a.upper() for a in sys.argv[1:]] or list(MUTATIONS)
    # The intact baseline, so "red" is a CHANGE and not a reading.
    print("=== building the INTACT tree")
    r = sh("cargo build --release -p c2-harness --bin c2rs")
    if r.returncode:
        raise SystemExit("intact build failed:\n" + r.stderr[-3000:])
    base = {}
    for g in ["A B C D", "A F"]:
        p, f, red, _, w = grid(g)
        base[g] = (p, f, red)
        print("    intact  grids [%s]  pass %s  fail %s  wrong-bytes %d  red %s" % (g, p, f, w, red))
        if f:
            raise SystemExit("the INTACT tree is not green — nothing below means anything")

    rows = []
    for name in want:
        path, old, new, grids, what = MUTATIONS[name]
        print("\n=== %s — %s   (%s)" % (name, what, path))
        try:
            apply(path, old, new)
            r = sh("cargo build --release -p c2-harness --bin c2rs")
            if r.returncode:
                print("    BUILD FAILED under the mutation:")
                print("    " + r.stderr.strip().splitlines()[-1][:200])
                rows.append((name, what, grids, "build-failed", "-", [], 0))
                continue
            p, f, red, out, wrong = grid(grids)
            bp, bf, _ = base[grids]
            print("    intact pass %d fail %d   ->   mutated pass %s fail %s"
                  % (bp, bf, p, f))
            print("    of the %d red, %d are WRONG BYTES (mismatch) and %d a refusal"
                  % (f, wrong, f - wrong))
            print("    turned red: %s" % (red,))
            rows.append((name, what, grids, "%s/%s" % (p, p + f),
                         "%s/%s" % (bp, bp + bf), red, wrong))
        finally:
            restore(path)
    # The tree must be intact again, checked rather than assumed.
    r = sh("git diff --quiet -- crates/")
    print("\n%s" % ("=" * 68))
    print("tree restored: %s" % ("YES" if r.returncode == 0 else "NO -- CHECK THE TREE"))
    print("\n| mutation | what | grids | intact | mutated | of the red, wrong BYTES |")
    print("|---|---|---|---:|---:|---:|")
    survived = []
    for name, what, grids, mut, intact, red, wrong in rows:
        print("| %s | %s | %s | %s | %s | %d |" % (name, what, grids, intact, mut, wrong))
        if mut == intact:
            survived.append(name)
    print("\nSURVIVED ITS OWN MUTATION (a rate that does is measuring nothing): %s"
          % (survived or "none"))
    sh("cargo build --release -p c2-harness --bin c2rs")
    return 1 if survived else 0


if __name__ == "__main__":
    sys.exit(main())
