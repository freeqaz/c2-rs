#!/usr/bin/env python3
"""carve.py — cut a FRESH held-out partition out of the 850-TU fitting corpus.

THE SELECTION RULE, stated in full and computable by anyone from the committed
`cacheidx.tsv` alone:

    key(src)  = sha256(src.encode("utf-8")).hexdigest()
    heldout   = the first N of the 850 source paths sorted ascending by key(src)
    fit       = the remaining 850 - N

It reads **only the source path strings**.  It does not open a cache entry, an
obj, an IL blob, a truth file or a prediction — so it cannot see any answer, and
the partition is a deterministic function of the corpus listing that was
committed on 2026-08-08 by an earlier lane.

Why N = 200 (registered before any number was computed; see PREREG.md §2):

  * `w-quar` spent a 21-TU set and reported that its 95 % Clopper-Pearson
    interval was **17x wider** than the in-sample one -- a 21-sample can refute
    a rate and cannot estimate one.  At N = 200 the CP width at p ~ 0.36 is
    about +/- 0.067 against the in-sample +/- 0.033, i.e. ~2.1x, not 17x.
  * The verdict that actually matters is TU REACH (`B^C ^ exact`).  `B^C` is 151
    of 878 TUs (~0.17), so a 21-sample carries ~3.6 of them and cannot see reach
    at all; 200 carries ~34, which can.
  * The comparison against the incumbent is PAIRED (same TUs, same code), so the
    discriminating quantity is the discordant count.  At the sizing the rung
    hypothesises, 200 gives discordance in the tens.
  * 650 TUs remain to fit on, and the fit chooses among a handful of binary
    predicates -- negligible capacity, so the split costs the fit nothing.

    usage: carve.py <cacheidx.tsv> <outdir> [N]

stdlib only.
"""
import hashlib
import os
import sys

N_DEFAULT = 200


def main():
    idxp, outd = sys.argv[1], sys.argv[2]
    n = int(sys.argv[3]) if len(sys.argv) > 3 else N_DEFAULT
    srcs = []
    for ln in open(idxp):
        ln = ln.rstrip("\n")
        if not ln.strip():
            continue
        srcs.append(ln.split("\t")[0])
    if len(srcs) != len(set(srcs)):
        raise SystemExit("corpus listing has duplicate source paths")
    ordered = sorted(srcs, key=lambda s: hashlib.sha256(s.encode()).hexdigest())
    held, fit = ordered[:n], ordered[n:]
    os.makedirs(outd, exist_ok=True)

    def emit(name, names):
        # sorted by PATH in the file, so the file is readable and stable; the
        # SELECTION is by digest and is reproduced by re-running this script.
        body = "\n".join(sorted(names)) + "\n"
        p = os.path.join(outd, name)
        open(p, "w").write(body)
        return hashlib.sha256(body.encode()).hexdigest()

    sh_h = emit("heldout200.txt", held)
    sh_f = emit("fit650.txt", fit)
    corpus = "\n".join(sorted(srcs)) + "\n"
    sh_c = hashlib.sha256(corpus.encode()).hexdigest()

    print("corpus            %d TUs   sha256 %s" % (len(srcs), sh_c))
    print("HELD-OUT          %d TUs   sha256 %s   heldout200.txt" % (len(held), sh_h))
    print("FIT (remainder)   %d TUs   sha256 %s   fit650.txt" % (len(fit), sh_f))
    print("disjoint          %s" % (not (set(held) & set(fit))))
    print("covering          %s" % (set(held) | set(fit) == set(srcs)))
    print()
    print("selection rule    sort by sha256(src), take the first %d" % n)
    print("first 3 by key   ", [ordered[i] for i in range(3)])
    print("last 3 by key    ", [ordered[-i] for i in (3, 2, 1)])


if __name__ == "__main__":
    main()
