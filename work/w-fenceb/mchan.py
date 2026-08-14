#!/usr/bin/env python3
"""mchan.py — the CONTROL: does the charge reach the obj, and can a wrong one
be seen?

Lane **w-backedge**, `PREREG.md` §4, registered before the first probe.

# Why this is not another stride table

A stride is a **difference**, and a constant error in the charge cancels out of
it. The most likely wrong-by-one error is therefore invisible in
`labelil.py`'s table. This script builds the one TU shape where the charge is
not a difference — board **#747**'s *"the corpus cannot express the shape a
backward-branch guard would break on"*:

```text
    int P(int a){ <a loop> }        a LEAF with a back edge
    int F(int a){ return ga(a)+1; } a FRAMED function, downstream
```

`coff::plan_labels` mints `$M`/`$T` only for a framed function, so P's charge
lands on **F's** three symbol records — six bytes of the reference obj's symbol
table, in an obj that would still link. Everything here is scored against
**those bytes**, not against the port's own model.

# The predictor, and the three mutants

`plan_labels`, transcribed for this two-function TU at `/O1` (COMDAT):

```text
    $M(F) = u32le(.gl[7..11]) + LABEL_SEED_GAP(9) + 3*nfuncs(6) + charge(P) + 1
                                                                             ^ P is a leaf
```

* **M0** the charge from R1 (`PREREG.md` §3)
* **M1** charge + 1        the most likely error, upward
* **M2** charge − 1        the most likely error, downward
* **M3** charge **0**      what `coff::plan_labels` charges TODAY, i.e. exactly
                           what the port would emit if invariant 4 were lifted
                           with no model at all

**M1/M2/M3 must go RED.** A cell where a mutant agrees with M0 could not have
disagreed; it is **vacuous** and is counted separately. The discriminating
count is printed for every mutant, so "no cells disagreed" is a loud failure
rather than a silent pass — a lane here once reported two "0 disagree" results
that no cell could have made disagree.

    work/w-backedge/mchan.py
    work/w-backedge/mchan.py --mode '/Ox /GS- /c'
"""

import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
sys.path.insert(0, HERE)
import gt_label_stride as G  # noqa: E402
from gt_dump import Obj  # noqa: E402
import labelil as L  # noqa: E402

LABEL_SEED_GAP = 9  # crates/c2-core/src/coff/label.rs

# The probes are grid1 cells, by name: the control is about the CHANNEL and the
# byte, not about fitting, so re-using fitted cells is correct here. `none` is
# the zero-charge control — the row that proves the predictor is right about
# everything OTHER than the charge, so a red row elsewhere is the charge.
CELLS = ["a-none", "a-if", "a-while", "a-dowhile", "a-for", "a-forever",
         "a-goto-back", "b-for2", "c-nest2", "b-dowhile2", "b-while2"]


def build(body):
    return ("int ga(int);\n"
            "int P(int a){ %s }\n"
            "int F(int a){ return ga(a)+1; }\n" % body)


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    grid = {c[0]: c[2] for c in L.load_grid(1)}
    wd = tempfile.mkdtemp(prefix="wbem")
    print("mode: %s   TU = [ P (leaf, the loop) , F (framed) ]" % mode)
    print("  predicted $M(F) = counter + %d + 3*2 + charge(P) + 1" % LABEL_SEED_GAP)
    print()
    print("%-16s %8s %7s %8s %8s   %-5s %-5s %-5s %-5s"
          % ("cell", "counter", "charge", "pred$M", "real$M",
             "M0", "M1", "M2", "M3"))
    red = {"M1": 0, "M2": 0, "M3": 0}
    disc = {"M1": 0, "M2": 0, "M3": 0}
    m0_green = m0_total = 0
    bad = 0
    for name in CELLS:
        body = grid[name]
        src = build(body)
        tag = name.replace("-", "_")
        cpp = os.path.join(wd, tag + ".cpp")
        open(cpp, "w").write(src)
        r = subprocess.run([os.path.join(REPO, "scripts", "gt_capture.sh"), cpp]
                           + mode.split(), capture_output=True, text=True)
        objp = r.stdout.strip()
        if not objp or not os.path.exists(objp):
            print("%-16s  CAPTURE FAILED" % name); bad += 1; continue
        o = Obj(open(objp, "rb").read())
        gs = G.groups(o)
        F = None
        for g in gs:
            if g["name"].startswith("?F@@"):
                F = g
        if F is None or not F["labels"]:
            print("%-16s  no $M on F (%s)" % (name, [g["name"] for g in gs]))
            bad += 1
            continue
        real = min(F["labels"])

        out = os.path.join(wd, "il_" + tag)
        os.makedirs(out, exist_ok=True)
        fl = os.path.join(wd, "flags.txt")
        open(fl, "w").write("/nologo " + mode + "\n")
        c = subprocess.run([os.path.join(REPO, "target", "release", "c2rs"),
                            "capture", cpp, "--keep-il", out, "--flags-file", fl],
                           capture_output=True, text=True)
        if c.returncode != 0:
            print("%-16s  IL CAPTURE FAILED" % name); bad += 1; continue
        got = {f.rsplit(".", 1)[-1]: os.path.join(out, f) for f in os.listdir(out)}
        gl = open(got["gl"], "rb").read()
        ex = open(got["ex"], "rb").read()
        counter = int.from_bytes(gl[7:11], "little")
        segs = L.ex_segments(ex)
        if len(segs) != 2:
            print("%-16s  IL SHAPE: %d segments" % (name, len(segs)))
            bad += 1
            continue
        f = L.ex_cflow(segs[0])          # P is the FIRST function here
        charge = 2 * f["bwd_uncond"] + 1 * f["bwd_cond"]
        base = counter + LABEL_SEED_GAP + 6 + 1
        preds = {"M0": base + charge, "M1": base + charge + 1,
                 "M2": base + charge - 1, "M3": base + 0}
        verdict = {}
        for k, v in preds.items():
            verdict[k] = "GREEN" if v == real else "red"
        m0_total += 1
        if verdict["M0"] == "GREEN":
            m0_green += 1
        for k in ("M1", "M2", "M3"):
            if preds[k] != preds["M0"]:
                disc[k] += 1            # this cell COULD have disagreed
                if verdict[k] == "red":
                    red[k] += 1
        print("%-16s %8d %7d %8d %8d   %-5s %-5s %-5s %-5s"
              % (name, counter, charge, preds["M0"], real,
                 verdict["M0"], verdict["M1"], verdict["M2"], verdict["M3"]))

    print()
    print("M0 (R1)          GREEN on %d of %d constructed cells" % (m0_green, m0_total))
    for k, what in (("M1", "charge + 1"), ("M2", "charge - 1"),
                    ("M3", "charge = 0 — what plan_labels charges TODAY")):
        print("%s (%-42s) red on %d of %d DISCRIMINATING cells"
              % (k, what, red[k], disc[k]))
        if disc[k] == 0:
            print("     *** VACUOUS: no cell could have disagreed. "
                  "This is a FAILED control, not a passing one.")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
