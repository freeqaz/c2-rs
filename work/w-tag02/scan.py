#!/usr/bin/env python3
"""w-tag02 — INSTRUMENT 1: a byte-scanner for `.in`, with NO production parser.

Deliberately a forward whole-stream record parser written from the grammar as
stated in `crates/c2-il/src/func/ininit.rs`'s module docs, in a different
language, by a different pass. It is instrument 1 of the two the `w-divsplit`
discipline requires; instrument 2 is the production reader's own cursor. Neither
is allowed to be the other's witness, so this file imports nothing from the
crate and reads no Rust.

  record  := <token-var> 00 <element>* 07
  element := 01 <type> <width> <value>     scalar
           | 02 <token-var> 00 <n>         address of a symbol   <-- the subject
           | 03 <len> <bytes>              byte string

Usage:  work/w-tag02/scan.py <cell>...      (default: every captured cell)
"""
import glob
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
IL = os.path.join(ROOT, "work", "w-tag02", "il")


def read_token(b, p):
    """(value, width) — big-endian 2-byte unless b[p+1] has the high bit."""
    if p + 1 >= len(b):
        return None
    if b[p + 1] & 0x80 == 0:
        return ((b[p] << 8) | b[p + 1], 2)
    if p + 3 >= len(b):
        return None
    return ((b[p] << 24) | (b[p + 1] << 16) | (b[p + 2] << 8) | b[p + 3], 4)


def read_value(b, p, width):
    """The scalar value encoding: raw byte at width 1, else short form or 0x80 escape."""
    if p >= len(b):
        return None
    b0 = b[p]
    if width == 1:
        return (bytes([b0]), p + 1)
    if b0 < 0x80:
        return (bytes(width - 1) + bytes([b0]), p + 1)
    if b0 != 0x80:
        return None
    if p + 1 + width > len(b):
        return None
    return (bytes(reversed(b[p + 1:p + 1 + width])), p + 1 + width)


def parse_record(b, p):
    """Parse one record at p. Returns (token, [elements], end) or None."""
    t = read_token(b, p)
    if t is None:
        return None
    tok, tw = t
    q = p + tw
    if q >= len(b) or b[q] != 0x00:
        return None
    q += 1
    elems = []
    while True:
        if q >= len(b):
            return None
        tag = b[q]
        if tag == 0x07:
            return (tok, elems, q + 1)
        if tag == 0x01:
            if q + 2 >= len(b):
                return None
            ty, width = b[q + 1], b[q + 2]
            v = read_value(b, q + 3, width)
            if v is None:
                return None
            val, q2 = v
            elems.append(("01", {"type": ty, "width": width, "value": val,
                                 "span": b[q:q2].hex(" ")}))
            q = q2
        elif tag == 0x02:
            t2 = read_token(b, q + 1)
            if t2 is None:
                return None
            tgt, tw2 = t2
            q2 = q + 1 + tw2
            if q2 + 1 >= len(b):
                return None
            sep, n = b[q2], b[q2 + 1]
            elems.append(("02", {"target": tgt, "target_w": tw2, "sep": sep, "n": n,
                                 "span": b[q:q2 + 2].hex(" ")}))
            q = q2 + 2
        elif tag == 0x03:
            if q + 1 >= len(b):
                return None
            ln = b[q + 1]
            q2 = q + 2
            if ln == 0x80:
                if q + 3 >= len(b):
                    return None
                ln = b[q + 2] | (b[q + 3] << 8)
                q2 = q + 4
            if q2 + ln > len(b):
                return None
            elems.append(("03", {"len": ln, "bytes": bytes(b[q2:q2 + ln]),
                                 "span": b[q:q2].hex(" ") + " …"}))
            q = q2 + ln
        else:
            return None


def scan(b):
    """Walk the whole stream forwards. Returns (records, bytes_consumed, resyncs)."""
    out, p, resyncs = [], 0, 0
    while p < len(b):
        r = parse_record(b, p)
        if r is None:
            p += 1
            resyncs += 1
            continue
        out.append((p,) + r)
        p = r[2]
    return out, resyncs


def gl_names(glb):
    """Rough token->name binding: every `<len> <name>` run in `.gl`, for LABELS ONLY.

    Deliberately not used for any claim — the `.gl` reader in the crate is the
    binding of record. This exists so the dump is readable.
    """
    names = []
    i = 0
    while i < len(glb):
        n = glb[i]
        if 1 <= n <= 0x7F and i + 1 + n <= len(glb):
            s = glb[i + 1:i + 1 + n]
            if all(0x20 <= c < 0x7F for c in s):
                names.append((i, s.decode()))
                i += 1 + n
                continue
        i += 1
    return names


def main():
    cells = sys.argv[1:] or sorted(os.path.basename(d.rstrip("/"))
                                   for d in glob.glob(os.path.join(IL, "*/")))
    for cell in cells:
        ins = glob.glob(os.path.join(IL, cell, "*.in"))
        if not ins:
            print("%-22s NO CAPTURE" % cell)
            continue
        b = open(ins[0], "rb").read()
        recs, resyncs = scan(b)
        covered = sum(r[3] - r[0] for r in recs)
        n02 = sum(1 for r in recs for e in r[2] if e[0] == "02")
        print("=== %s  (.in %d B, %d records, %d B covered, %d resync bytes, %d tag-02 elements)"
              % (cell, len(b), len(recs), covered, resyncs, n02))
        for (at, tok, elems, end) in recs:
            if not any(e[0] == "02" for e in elems):
                continue
            print("  @%04x tok=%04x  %s" % (at, tok, b[at:end].hex(" ")))
            for kind, e in elems:
                if kind == "02":
                    print("      tag02 target=%04x tw=%d sep=%02x n=%02x   [%s]"
                          % (e["target"], e["target_w"], e["sep"], e["n"], e["span"]))
                elif kind == "01":
                    print("      tag01 type=%02x width=%d value=%s   [%s]"
                          % (e["type"], e["width"], e["value"].hex(), e["span"]))
                else:
                    print("      tag03 len=%d bytes=%r" % (e["len"], e["bytes"]))


if __name__ == "__main__":
    main()
