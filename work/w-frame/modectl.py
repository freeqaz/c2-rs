#!/usr/bin/env python3
"""modectl.py — the MODE control on port_vocab.

`port_vocab` is extracted from fixture objs compiled at the fixture profile
(`/Ox /GS- /c`); the 17 frontier TUs are compiled at the WORKLOAD profile
(`/O1 /Oi /EHsc /GR ...`). If c2 emits a construct at /O1 that it does not emit
at /Ox, the vocabulary is under-measured and EVERY gap in the ranking is
inflated -- in the direction that makes the frontier look further away.

`scripts/gate.sh` already grades the port byte-exact on these fixtures in a /O1
lane, so an /O1 fixture obj is a legitimate member of the vocabulary. This
recompiles every matched fixture at the workload flags and reports what the
vocabulary GAINS.
"""
import os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import featmap

fixdir = os.path.join(featmap.REPO, "fixtures", "cpp")
fixtures = [l.strip() for l in open(os.path.join(HERE, "match_fixtures.txt")) if l.strip()]
# the workload profile minus its project /I paths, which no fixture needs
flags = [f for f in featmap.WORKLOAD_FLAGS if not f.startswith("/I")]
flags = [f for f in flags if f != "src"]

ox, o1, n_ox, n_o1 = set(), set(), 0, 0
for fx in fixtures:
    b = featmap.compile_obj(fx, featmap.FIXTURE_FLAGS, fixdir,
                            os.path.join(featmap.OBJDIR, "fix", fx + ".obj"))
    if b:
        ox |= featmap.obj_features(b)[1]
        n_ox += 1
    b = featmap.compile_obj(fx, flags, fixdir,
                            os.path.join(featmap.OBJDIR, "fix_o1", fx + ".obj"))
    if b:
        o1 |= featmap.obj_features(b)[1]
        n_o1 += 1

print("fixtures compiled: /Ox %d   workload-profile %d" % (n_ox, n_o1))
print("vocab /Ox            : %d tokens" % len(ox))
print("vocab workload-profile: %d tokens" % len(o1))
print("gained by adding /O1 : %d   %s" % (len(o1 - ox), " ".join(sorted(o1 - ox))))
print("lost (in /Ox only)   : %d   %s" % (len(ox - o1), " ".join(sorted(ox - o1))))
