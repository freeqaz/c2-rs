#!/usr/bin/env python3
"""Do fragments 88 and 89 generate any case in common? Lane w-gen2 evidence."""
import os
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
sys.path.insert(0, os.path.join(REPO, 'scripts'))
import sweep_gen  # noqa: E402

d = os.path.join(REPO, 'scripts/sweep.d')
_, a = sweep_gen.fragment_cases(d, '88-store-run-call.py')
_, b = sweep_gen.fragment_cases(d, '89-store-run-live-arg.py')
print('88: %d cases (%d distinct)' % (len(a), len(set(a))))
print('89: %d cases (%d distinct)' % (len(b), len(set(b))))
print('intersection: %d' % len(set(a) & set(b)))
