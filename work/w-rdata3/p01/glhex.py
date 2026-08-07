#!/usr/bin/env python3
"""glhex.py — hexdump a `.gl` around each named symbol run. Read-only.

Lane w-rdata3. Locates each NUL-terminated ASCII-graphic run whose text matches
one of the names given on the command line and prints the bytes AFTER its
terminating NUL, which is where `data_object_at` reads the record frame from.
"""
import sys

path = sys.argv[1]
names = sys.argv[2:]
gl = open(path, "rb").read()

i = 0
runs = []
while i < len(gl):
    if not (0x21 <= gl[i] <= 0x7E):
        i += 1
        continue
    s = i
    while i < len(gl) and 0x21 <= gl[i] <= 0x7E:
        i += 1
    if i < len(gl) and gl[i] == 0:
        runs.append((s, i, gl[s:i].decode("latin1")))

for s, e, txt in runs:
    for n in names:
        if n in txt:
            pre = gl[max(0, s - 8):s].hex(" ")
            post = gl[e:min(len(gl), e + 20)].hex(" ")
            print(f"{txt}")
            print(f"    run@{s} pre: {pre}")
            print(f"    NUL@{e} post: {post}")
            break
print(f"-- {len(runs)} NUL-terminated graphic runs total")
