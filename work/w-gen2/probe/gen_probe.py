#!/usr/bin/env python3
"""Scratch probe for lane w-gen2 — NOT the fragment. Writes candidate cells so the
live-argument axis can be designed against real `c2` bytes rather than guessed."""
import itertools
import os
import sys

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "cells")

PRE = (
    'struct BE { BE* mNext; BE* mPrev; };\n'
    'extern BE* f0();\n'
    'extern BE* f1(unsigned int);\n'
    'extern BE* f2(unsigned int, unsigned int);\n'
    'struct K {\n'
    '  K(unsigned int a, unsigned int b, unsigned int c);\n'
    '  void mv(unsigned int a, unsigned int b, unsigned int c);\n'
    '  BE* A0();\n'
    '  BE* A1(unsigned int);\n'
    '  BE* A2(unsigned int, unsigned int);\n'
    '  BE* A3(unsigned int, unsigned int, unsigned int);\n'
    '  K* p0; K* p1; BE mList;\n'
    '  unsigned int mA; unsigned int mB; unsigned int mC;\n'
    '  BE mSecond; unsigned int mD; unsigned int mE;\n'
    '};\n'
)

S = {
    'a': 'mA = a;',
    'b': 'mB = b;',
    'c': 'mC = c;',
    't': 'p0 = this;',
    'u': 'p1 = this;',
    'L': 'mD = 0;',
    'M': 'mE = 7;',
    'P': 'mList.mNext = &mList;',
}

CALLS = {
    'k0': 'A0()',
    'k1': 'A1(a)',
    'k2': 'A2(a, b)',
    'k3': 'A3(a, b, c)',
    'kn': None,
}


def main():
    os.makedirs(OUT, exist_ok=True)
    names = []
    for kname, call in CALLS.items():
        for run in ('Lba', 'Lab', 'ab', 'ba', 'Lbat', 'Ltba', 'abc', 'cba',
                    'Lbc', 'Lca', 'Ltu', 'Pba', 'Lbau'):
            for hdr, hn in (
                ('K::K(unsigned int a, unsigned int b, unsigned int c)', 'ctor'),
                ('void K::mv(unsigned int a, unsigned int b, unsigned int c)', 'void'),
            ):
                stmts = ''.join('  %s\n' % S[ch] for ch in run)
                tail = '' if call is None else '  %s;\n' % call
                name = '%s_%s_%s' % (kname, run, hn)
                with open(os.path.join(OUT, name + '.cpp'), 'w') as fh:
                    fh.write(PRE + '%s {\n%s%s}\n' % (hdr, stmts, tail))
                names.append(name)
    print('\n'.join(names))


if __name__ == '__main__':
    main()
