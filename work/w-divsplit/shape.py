#!/usr/bin/env python3
"""What the division population IS, once split.py has said what it is not.

split.py answers board #783 (the int/float split). This answers the question the
rung has to answer next: what would an embedded-division emit actually have to
do? Enumerated from the bytes, the way w-loop and w-hash enumerated Sort.cpp.

Everything here is a COUNT WITH A DENOMINATOR. Nothing is a ranking.

Usage:  shape.py rows.tsv
"""
import sys
import collections

from split import read_type, read_lit_payload, decode_operand_ending_at, CLASS

rows = []
for line in open(sys.argv[1]):
    f = line.rstrip("\n").split("\t")
    if len(f) < 13:
        continue
    rows.append(f)

n = len(rows)
names = collections.Counter()
emitted_names = collections.Counter()
divisor_val = collections.Counter()
after = collections.Counter()
prod = collections.Counter()
disp = collections.Counter()
cflow = collections.Counter()
eh = collections.Counter()
frame = collections.Counter()
ptr_in_window = 0
sub_before = 0
pow2 = 0
emitted_n = 0
srcs = collections.Counter()
dividend = collections.Counter()

for f in rows:
    key, is_em, mark = f[2], f[3] == "EMITTED", int(f[11])
    b = bytes(int(x, 16) for x in f[12].split())
    names[f[4]] += 1
    srcs[f[0]] += 1
    if is_em:
        emitted_n += 1
        emitted_names[f[4]] += 1
    prod[f[9]] += 1
    disp[f[8]] += 1
    cflow[f[6]] += 1
    eh[f[7]] += 1
    frame[f[5]] += 1
    # The divisor's VALUE: re-decode the literal that ends at the opcode.
    hits = [h for h in decode_operand_ending_at(b, mark) if h[1] == "lit"]
    if hits:
        j = min(hits, key=lambda h: h[0])[0]
        t = read_type(b, j + 1)
        v = read_lit_payload(b, j + 1 + t[3])[0]
        divisor_val[v] += 1
        if v > 0 and (v & (v - 1)) == 0:
            pow2 += 1
    # What consumes the quotient (the token right after the opcode).
    after["%02x" % b[mark + 1] if mark + 1 < len(b) else "END-OF-WINDOW"] += 1
    # Is the DIVIDEND a subtraction, and of WHAT? `03` immediately before the
    # divisor's literal means the dividend is `x - y`; the token ending at that
    # `03` is `y`, and its TYPE says whether this is a pointer difference or an
    # integer one. Decoded with the same reader, never assumed.
    if hits:
        j = min(hits, key=lambda h: h[0])[0]
        if j > 0 and b[j - 1] == 0x03:
            sub_before += 1
            sub_hits = decode_operand_ending_at(b, j - 1)
            typed = [h for h in sub_hits if h[3] is not None]
            if not typed:
                dividend["UNDECODABLE" if not sub_hits else "op-result"] += 1
            else:
                k = min(typed, key=lambda h: h[0])[3]
                dividend[CLASS.get(k & 0x0F, "unknown-%x" % (k & 0x0F))] += 1
        else:
            dividend["no-sub-before-the-divisor"] += 1
    for i in range(len(b)):
        t = read_type(b, i)
        if t is not None and (t[1] & 0x0F) == 0x3:
            ptr_in_window += 1
            break

print("rows %d   emitted %d   distinct source TUs %d" % (n, emitted_n, len(srcs)))
print()
print("DISTINCT MANGLED NAMES (the replication question witness.rs raises):")
print("  distinct names, all rows      : %d of %d rows" % (len(names), n))
print("  distinct names, EMITTED rows  : %d of %d rows" % (len(emitted_names), emitted_n))
print("  top names by row count:")
for k, v in names.most_common(8):
    print("    %5d  %s" % (v, k[:96]))
print()
print("DIVISOR LITERAL VALUE (denominator %d):" % n)
print("  power-of-two divisors : %d  (%.1f%%)" % (pow2, 100.0 * pow2 / n))
for k, v in divisor_val.most_common(20):
    print("    /%-8d %5d  %5.1f%%" % (k, v, 100.0 * v / n))
print("  distinct divisor values: %d" % len(divisor_val))
print()
print("DIVIDEND TYPE -- the operand of the `03` SUB the division consumes (denominator %d):" % n)
for k, v in dividend.most_common():
    print("    %-30s %5d  %5.1f%%" % (k, v, 100.0 * v / n))
print()
print("DIVIDEND is a `03` SUB directly before the divisor:")
print("  %d of %d  (%.1f%%)" % (sub_before, n, 100.0 * sub_before / n))
print("  rows with any pointer-class TYPE in the 40-byte window: %d of %d" % (ptr_in_window, n))
print()
print("TOKEN IMMEDIATELY AFTER the division opcode (denominator %d):" % n)
for k, v in after.most_common(8):
    print("    %-14s %5d  %5.1f%%" % (k, v, 100.0 * v / n))
print()
for label, c in (("frame class", frame), ("cflow", cflow), ("eh", eh),
                 ("dispatch", disp), ("production", prod)):
    print("%s (denominator %d):" % (label.upper(), n))
    for k, v in c.most_common(6):
        print("    %-58s %5d  %5.1f%%" % (k[:58], v, 100.0 * v / n))
    print("    distinct values: %d" % len(c))
    print()
