"""COFF reader — .text COMDAT leader extraction.

Verbatim port of `crates/c2-obj/src/lib.rs::ObjImage::text_comdat_entries`,
carried over from the w-phase7plan lane's known-answer-gated
`work/lane-c/readers.py` (gated on App.cpp 38/158 and TextFile.cpp 674/70/32-30).
Do not "improve" it: it must stay bit-for-bit equivalent to the harness rule.
"""
import struct

COFF_HEADER_LEN = 20
SECTION_HEADER_LEN = 40
SYMBOL_LEN = 18
IMAGE_SCN_LNK_COMDAT = 0x1000
IMAGE_SYM_CLASS_STATIC = 3


def text_comdat_entries(b):
    """Port of ObjImage::text_comdat_entries. Returns list of (name, secidx) or None."""
    if len(b) < COFF_HEADER_LEN:
        return None
    nsec = struct.unpack_from('<H', b, 2)[0]
    psym = struct.unpack_from('<I', b, 8)[0]
    nsym = struct.unpack_from('<I', b, 12)[0]
    sec_end = COFF_HEADER_LEN + nsec * SECTION_HEADER_LEN
    sym_end = psym + nsym * SYMBOL_LEN
    if sec_end > len(b) or psym < sec_end or sym_end + 4 > len(b):
        return None
    strtab = b[sym_end:]

    def str_at(i):
        if i >= len(strtab):
            return None
        e = strtab.find(b'\0', i)
        if e < 0:
            return None
        return strtab[i:e].decode('utf-8', 'replace')

    is_text = [False] * nsec
    for i in range(nsec):
        o = COFF_HEADER_LEN + i * SECTION_HEADER_LEN
        raw = b[o:o + 8]
        if raw[0:1] == b'/':
            try:
                idx = int(raw[1:].rstrip(b'\0').strip())
            except ValueError:
                return None
            name = str_at(idx)
            if name is None:
                return None
        else:
            name = raw.rstrip(b'\0').decode('utf-8', 'replace')
        chars = struct.unpack_from('<I', b, o + 36)[0]
        is_text[i] = name.startswith('.text') and (chars & IMAGE_SCN_LNK_COMDAT) != 0
    claimed = [False] * nsec
    out = []
    i = 0
    while i < nsym:
        o = psym + i * SYMBOL_LEN
        naux = b[o + 17]
        secnum = struct.unpack_from('<h', b, o + 12)[0]
        sclass = b[o + 16]
        if 1 <= secnum <= nsec:
            s = secnum - 1
            is_secdef = (sclass == IMAGE_SYM_CLASS_STATIC and naux == 1)
            if is_text[s] and not claimed[s] and not is_secdef:
                if b[o:o + 4] == b'\0\0\0\0':
                    at = struct.unpack_from('<I', b, o + 4)[0]
                    name = str_at(at)
                    if name is None:
                        return None
                else:
                    name = b[o:o + 8].rstrip(b'\0').decode('utf-8', 'replace')
                claimed[s] = True
                out.append((name, s))
        i = i + 1 + naux
        if i > nsym:
            return None
    if any(t and not c for c, t in zip(claimed, is_text)):
        return None
    return out
