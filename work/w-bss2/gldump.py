#!/usr/bin/env python3
"""Lane w-bss2: dump the `.gl` bytes around every name record.

Purely descriptive — it slices the file at printable-name runs and prints the
raw bytes before/after each, so a designed one-axis probe pair can be diffed by
eye.  No field meaning is assumed here; §A of the doc records what the diffs
showed.
"""
import re, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cap

SHELL = ('.XBLD$W', '__C1_11886', '__C2_11886', '@comp.id')


def name_spans(d):
    """[(start, end, name)] for every NUL-terminated printable run >= 2 chars."""
    out = []
    for m in re.finditer(rb'[ -~]{2,}\x00', d):
        s = m.group()[:-1].decode()
        if '\\' in s or '/' in s:
            continue
        if not (s[0].isalpha() or s[0] in '?_$.@'):
            continue
        out.append((m.start(), m.end(), s))
    return out


def dump(d, pre=6, post=24, skip_shell=False):
    for s, e, n in name_spans(d):
        if skip_shell and n in SHELL:
            continue
        b = d[max(0, s - pre):s]
        a = d[e:e + post]
        print("  %-28s | pre %-18s | post %s"
              % (n, ' '.join('%02x' % c for c in b),
                 ' '.join('%02x' % c for c in a)))


def hexdump(d, base=0):
    for i in range(0, len(d), 16):
        ch = d[i:i + 16]
        print('%04x  %-47s  %s' % (base + i, ' '.join('%02x' % c for c in ch),
                                   ''.join(chr(c) if 32 <= c < 127 else '.' for c in ch)))


HERE = os.path.dirname(os.path.abspath(__file__))
import paths
FLAGS = paths.probe_flags()


def gl_of(src, tag):
    p = os.path.join(HERE, "scratch", "%s.cpp" % tag)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    open(p, "w").write(src)
    return cap.capture_il(cap.to_z(p), FLAGS)["gl"]


if __name__ == "__main__":
    for i, src in enumerate(sys.argv[1:]):
        print("=== %s" % src)
        dump(gl_of(src, "gd%d" % i), skip_shell=True)
