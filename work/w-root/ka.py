#!/usr/bin/env python3
"""ka.py — KNOWN-ANSWER: `rootmodel.model(st, roots={})` IS `JFP_ALIAS`.

Every claim this lane makes is a delta against `JFP_ALIAS`, so the incumbent has
to be reproduced from this lane's own code before any delta is meaningful.  This
loads `work/w-quar/predict.py` -- the model frozen at `e75f46ac`, digest
`15b9a571...` -- BY PATH, runs it and `rootmodel` over the same TUs, and compares
the predicted sets NAME FOR NAME (not by count).

    usage: ka.py <idx.tsv> [jobs]

Exits non-zero on the first disagreement.  stdlib only.
"""
import hashlib
import importlib.util
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ["C2RS_LANEROOT"]
sys.path.insert(0, HERE)
import rootmodel as rm   # noqa: E402

_p = os.path.join(MAIN, "work", "w-quar", "predict.py")
_s = importlib.util.spec_from_file_location("wquar_predict", _p)
wq = importlib.util.module_from_spec(_s)
sys.modules["wquar_predict"] = wq
_s.loader.exec_module(wq)


def sha(names):
    return hashlib.sha256(("\n".join(sorted(names)) + "\n").encode()).hexdigest()


def one(row):
    src, entry = row[0], row[1]
    ref = wq.one([src, entry])
    if ref.get("status") != "ok":
        return (src, "SKIP", "", "")
    st = rm.state(entry)
    mine = rm.model(st, roots=frozenset())
    return (src, "OK" if sha(mine) == ref["sha"]["JFP_ALIAS"] else "DIFFER",
            sha(mine), ref["sha"]["JFP_ALIAS"])


def main():
    rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1]) if l.strip()]
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    ok = differ = skip = 0
    with cf.ProcessPoolExecutor(max_workers=jobs) as ex:
        for src, v, a, b in ex.map(one, rows, chunksize=2):
            if v == "OK":
                ok += 1
            elif v == "SKIP":
                skip += 1
            else:
                differ += 1
                print("  DIFFER %s\n    rootmodel %s\n    w-quar    %s"
                      % (src, a, b))
    print("KA JFP_ALIAS by NAME:  identical %d   differ %d   skipped %d   of %d"
          % (ok, differ, skip, len(rows)))
    sys.exit(1 if differ else 0)


if __name__ == "__main__":
    main()
