#!/usr/bin/env python3
"""seed.py — print the `.gl` compiler-label seed of every bundle under a dir.

`c2_il::label_counter` is `gl[7..11]` little-endian behind a 7-byte header
prefix; this is a transcription of that reader so a probe cell's seed can be
read without a build. Lane w-main2.
"""
import os
import struct
import sys


def seed_of(path):
    d = open(path, 'rb').read()
    if len(d) < 11:
        return None
    return struct.unpack_from('<I', d, 7)[0]


for root in sys.argv[1:]:
    for dirpath, _dirs, files in os.walk(root):
        for f in sorted(files):
            if f.endswith('.gl'):
                p = os.path.join(dirpath, f)
                print('%-60s %d' % (p, seed_of(p)))
