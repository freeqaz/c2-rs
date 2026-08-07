#!/usr/bin/env python3
"""Did the comment-only edit after the gate change the generated corpus?

`w-hash` §7.2 records the failure of quoting a gate from a tree that is not the
one being landed. The gate ran at `4784ea05`; the fragment's doc block changed at
`91950fb1`. Checked here rather than assumed: load the fragment from BOTH blobs
and compare the case lists, not their counts."""
import hashlib
import os
import subprocess
import sys
import tempfile

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
sys.path.insert(0, os.path.join(REPO, 'scripts'))
import sweep_gen  # noqa: E402

FRAG = 'scripts/sweep.d/89-store-run-live-arg.py'


def cases_at(rev):
    blob = subprocess.check_output(['git', '-C', REPO, 'show', '%s:%s' % (rev, FRAG)])
    d = tempfile.mkdtemp(prefix='w-gen2-treecheck-')
    p = os.path.join(d, os.path.basename(FRAG))
    with open(p, 'wb') as fh:
        fh.write(blob)
    _, cs = sweep_gen.fragment_cases(d, os.path.basename(FRAG))
    os.unlink(p)
    os.rmdir(d)
    return cs


a = cases_at(sys.argv[1])
b = cases_at(sys.argv[2])
ha = hashlib.sha256(''.join(a).encode()).hexdigest()[:16]
hb = hashlib.sha256(''.join(b).encode()).hexdigest()[:16]
print('%s: %d cases  sha256[:16]=%s' % (sys.argv[1], len(a), ha))
print('%s: %d cases  sha256[:16]=%s' % (sys.argv[2], len(b), hb))
print('IDENTICAL' if a == b else 'DIFFERENT — the gate does not apply to the tip')
