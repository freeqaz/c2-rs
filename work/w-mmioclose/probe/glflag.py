#!/usr/bin/env python3
"""w-mmioclose — print the `.gl` DEFINED-function record tail for every mangled
name in a captured bundle, so an attribute's bit can be read off a grid instead
of guessed at.

`c2_il::func::gl::gl_defined_names_framed` binds a record by scanning for
`codec::gl_offset_framed` and reading a LE32 at `p+1`; the record's NAME is the
last symbol run ending at or before `p`.  This script does the same walk in the
other direction — find each NUL-terminated mangled run, then print the bytes
that follow it — and it makes no claim about what any byte MEANS.  The grid
does that.

Usage:  glflag.py <stem-without-extension> [n_tail_bytes]
"""
import sys, re

stem = sys.argv[1]
n = int(sys.argv[2]) if len(sys.argv) > 2 else 26
gl = open(stem + ".gl", "rb").read()

pat = rb"[?A-Za-z_][A-Za-z0-9_?@$.:\-]{1,}?\x00" if len(sys.argv) > 3 else rb"[?][ -~]{2,}?\x00"
for m in re.finditer(pat, gl):
    name = m.group(0)[:-1].decode("ascii", "replace")
    nul = m.end() - 1
    tail = gl[nul + 1 : nul + 1 + n]
    print("%-28s @%04x  %s" % (name, nul, " ".join("%02x" % b for b in tail)))
