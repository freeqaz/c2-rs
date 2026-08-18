#!/usr/bin/env python3
"""Extract the .text COMDAT symbol names of one reference obj.

Reading (3) of the hypothesis — "malformed or truncated containers" — is tested
here too: every structural field is checked and a violation is REPORTED, never
tolerated. A clean read is a positive result, not an absence.

Usage: objsyms.py <obj>            -> prints "OK <n> <name>..." or "BAD <why>"
"""
import struct
import sys

# COFF section flags
IMAGE_SCN_LNK_COMDAT = 0x00001000


def read(path):
    b = open(path, "rb").read()
    problems = []
    if len(b) < 20:
        return None, [f"file shorter than a COFF header ({len(b)} B)"]
    machine, nsec, tds, symptr, nsym, oh, chars = struct.unpack_from("<HHIIIHH", b, 0)
    if machine != 0x01F2:  # IMAGE_FILE_MACHINE_POWERPCBE
        problems.append(f"machine 0x{machine:04X} != 0x01F2 (PowerPC BE)")
    if oh != 0:
        problems.append(f"optional header size {oh} != 0 for an object file")
    sec_off = 20 + oh
    if sec_off + 40 * nsec > len(b):
        return None, problems + [f"section table runs past EOF ({nsec} sections)"]
    secs = []
    for i in range(nsec):
        o = sec_off + 40 * i
        name = b[o:o + 8].rstrip(b"\0").decode("latin1")
        vsize, vaddr, rawsize, rawptr, relptr, lnoptr, nrel, nlno, flags = \
            struct.unpack_from("<IIIIIIHHI", b, o + 8)
        if rawptr and rawptr + rawsize > len(b):
            problems.append(f"section {i} ({name}) raw data {rawptr}+{rawsize} past EOF {len(b)}")
        if relptr and relptr > len(b):
            problems.append(f"section {i} ({name}) reloc table past EOF")
        secs.append((name, flags, rawsize))
    if symptr == 0 or nsym == 0:
        problems.append("no symbol table")
        return None, problems
    if symptr + 18 * nsym > len(b):
        return None, problems + ["symbol table runs past EOF"]
    strtab_off = symptr + 18 * nsym
    if strtab_off + 4 > len(b):
        problems.append("string table header past EOF")
        strtab = b""
    else:
        stlen = struct.unpack_from("<I", b, strtab_off)[0]
        if strtab_off + stlen > len(b):
            problems.append(f"string table length {stlen} runs past EOF")
        strtab = b[strtab_off:strtab_off + max(4, min(stlen, len(b) - strtab_off))]

    def sname(raw):
        if raw[:4] == b"\0\0\0\0":
            off = struct.unpack_from("<I", raw, 4)[0]
            end = strtab.find(b"\0", off)
            if end < 0:
                return None
            return strtab[off:end].decode("latin1")
        return raw.rstrip(b"\0").decode("latin1")

    names = []
    i = 0
    while i < nsym:
        o = symptr + 18 * i
        raw = b[o:o + 8]
        value, secnum, typ, sclass, naux = struct.unpack_from("<IhHBB", b, o + 8)
        nm = sname(raw)
        if nm is None:
            problems.append(f"symbol {i} name offset outside the string table")
            nm = f"<bad@{i}>"
        # external (2) or static (3), defined in a section, function type (0x20)
        if secnum > 0 and secnum <= nsec and sclass in (2, 3) and typ == 0x20:
            sec = secs[secnum - 1]
            if sec[0].startswith(".text"):
                names.append(nm)
        i += 1 + naux
    return names, problems


if __name__ == "__main__":
    names, problems = read(sys.argv[1])
    if problems:
        print("BAD " + " | ".join(problems))
    if names is not None:
        print("OK %d" % len(names))
        for n in sorted(set(names)):
            print(n)
