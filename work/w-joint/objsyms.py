#!/usr/bin/env python3
"""objsyms.py — the extended truth capture: every DEFINED symbol of an obj,
classified, with the invariants that grade the instrument itself.

w-skip's §8 item 4 names this lane's first task:

    "the 850-TU version needs a truth capture that records DEFINED DATA
     SYMBOLS, which `work/w-emit/truth` does not — it holds code COMDAT
     leaders only."

**The oracle cannot grade a correspondence.**  The compiler judges obj bytes;
it cannot tell you whether census row R is symbol S.  So a binding instrument
has to be graded on its own invariants, and this module publishes four:

  INJECTIVITY  within one obj, a defined name defines exactly one entity.
               Conflicts are counted AND named, never merged.
  TOTALITY     every symbol-table ENTITY lands in exactly one bucket; the
               residue is printed with its names on every run.  Residue 0 is
               NOT a control on its own (STATUS trap 4) — moving an entity from
               one bucket to another satisfies it exactly.
  ARITY        so the residue cannot go quiet while contents rot:
                 A1  sum(1 + naux) over entities == the header's NumberOfSymbols
                 A2  aux-record count == nsym - entity count
                 A3  every long name resolves inside the string table, and the
                     bytes it consumes are counted
               Residue counts entities; arity counts their contents.  A lane
               here once removed a `DUP` expansion and left totality silent at
               residue 0 while an arity check went 22 red.
  AGREEMENT    the code-COMDAT-leader set recomputed here must equal w-emit's
               independently captured `truth/<slug>.txt`, on every TU.  That is
               the one place the oracle HAS graded the symbol table, and it is
               the only external check a correspondence can get.

Section selection is by CHARACTERISTIC, never by name prefix — this project has
twice been burned by name-as-proxy — and the name-prefix rule is computed
alongside so any disagreement is reported rather than reconciled.

stdlib only; reads objs, runs nothing.
"""
import struct

COFF_HDR = 20
SEC_HDR = 40
SYM = 18

SCN_CNT_CODE = 0x00000020
SCN_CNT_INITIALIZED_DATA = 0x00000040
SCN_CNT_UNINITIALIZED_DATA = 0x00000080
SCN_LNK_COMDAT = 0x00001000

SYM_CLASS_EXTERNAL = 2
SYM_CLASS_STATIC = 3
SYM_CLASS_LABEL = 6
SYM_CLASS_FILE = 103
SYM_CLASS_SECTION = 104
SYM_CLASS_WEAK_EXTERNAL = 105


class ObjSyms(object):
    """One obj's sections and symbol-table entities, with the arity counters."""

    def __init__(self, b):
        self.b = b
        self.ok = True
        self.err = None
        if len(b) < COFF_HDR:
            self.ok, self.err = False, "short"
            return
        self.nsec = struct.unpack_from("<H", b, 2)[0]
        self.psym = struct.unpack_from("<I", b, 8)[0]
        self.nsym = struct.unpack_from("<I", b, 12)[0]
        sec_end = COFF_HDR + self.nsec * SEC_HDR
        sym_end = self.psym + self.nsym * SYM
        if sec_end > len(b) or self.psym < sec_end or sym_end + 4 > len(b):
            self.ok, self.err = False, "bounds"
            return
        self.strtab = b[sym_end:]
        self.strtab_len = struct.unpack_from("<I", b, sym_end)[0] \
            if len(self.strtab) >= 4 else 0
        self.sections = []
        for i in range(self.nsec):
            o = COFF_HDR + i * SEC_HDR
            raw = b[o:o + 8]
            if raw[0:1] == b"/":
                try:
                    nm = self.str_at(int(raw[1:].rstrip(b"\0").strip()))
                except ValueError:
                    nm = None
            else:
                nm = raw.rstrip(b"\0").decode("utf-8", "replace")
            if nm is None:
                self.ok, self.err = False, "secname"
                return
            self.sections.append({
                "name": nm,
                "chars": struct.unpack_from("<I", b, o + 36)[0],
                "size": struct.unpack_from("<I", b, o + 16)[0],
            })
        self.entities = None
        self.arity = None
        self._read_symbols()

    # -- string table -------------------------------------------------
    def str_at(self, i):
        if i >= len(self.strtab):
            return None
        e = self.strtab.find(b"\0", i)
        return None if e < 0 else self.strtab[i:e].decode("utf-8", "replace")

    # -- the symbol table, as ENTITIES (record + its aux records) ------
    def _read_symbols(self):
        b, ents = self.b, []
        n_aux = 0
        n_rec = 0
        strbytes = 0
        long_fail = 0
        i = 0
        while i < self.nsym:
            o = self.psym + i * SYM
            naux = b[o + 17]
            secnum = struct.unpack_from("<h", b, o + 12)[0]
            sclass = b[o + 16]
            value = struct.unpack_from("<I", b, o + 8)[0]
            longname = b[o:o + 4] == b"\0\0\0\0"
            if longname:
                at = struct.unpack_from("<I", b, o + 4)[0]
                name = self.str_at(at)
                if name is None:
                    long_fail += 1
                    name = ""
                else:
                    strbytes += len(name) + 1
            else:
                name = b[o:o + 8].rstrip(b"\0").decode("utf-8", "replace")
            ents.append({"name": name, "sec": secnum, "cls": sclass,
                         "val": value, "naux": naux, "long": longname,
                         "idx": i})
            n_rec += 1
            n_aux += naux
            i += 1 + naux
            if i > self.nsym:
                self.ok, self.err = False, "aux-overrun"
                return
        self.entities = ents
        self.arity = {
            "nsym_header": self.nsym,
            "records_consumed": n_rec + n_aux,   # A1: must equal nsym_header
            "entities": n_rec,
            "aux": n_aux,                        # A2
            "aux_check": self.nsym - n_rec,
            "long_names": sum(1 for e in ents if e["long"]),
            "long_name_bytes": strbytes,         # A3
            "long_name_unresolved": long_fail,
            "strtab_len_field": self.strtab_len,
            "strtab_actual": len(self.strtab),
        }

    # -- section predicates -------------------------------------------
    def is_code(self, s):
        return (self.sections[s]["chars"] & SCN_CNT_CODE) != 0

    def is_comdat(self, s):
        return (self.sections[s]["chars"] & SCN_LNK_COMDAT) != 0

    def is_data(self, s):
        c = self.sections[s]["chars"]
        return (c & SCN_CNT_CODE) == 0 and (
            c & (SCN_CNT_INITIALIZED_DATA | SCN_CNT_UNINITIALIZED_DATA)) != 0


def classify(o):
    """Bucket every entity.  Returns (buckets, residue, injectivity, notes).

    Buckets partition the entities; `residue` holds anything that fell through,
    with its names, so absence can never read as success.
    """
    B = {k: [] for k in (
        "code_comdat_leader",   # a COMDAT code section's first non-secdef sym
        "code_other",           # any other defined symbol in a code section
        "data_comdat_leader",   # a COMDAT data section's first non-secdef sym
        "data_other",           # any other defined symbol in a data section
        "defined_other_sec",    # defined in a section that is neither
        "section_def",          # the STATIC/naux==1 section definition record
        "undefined",            # secnum 0, value 0
        "common",               # secnum 0, value != 0
        "absolute",             # secnum -1
        "debug",                # secnum -2 (the .file records live here)
        "file",                 # IMAGE_SYM_CLASS_FILE
    )}
    residue = []
    claimed = [False] * o.nsec
    names_defined = {}
    conflicts = []

    for e in o.entities:
        s, cls = e["sec"], e["cls"]
        if cls == SYM_CLASS_FILE:
            B["file"].append(e)
            continue
        if s == -2:
            B["debug"].append(e)
            continue
        if s == -1:
            B["absolute"].append(e)
            continue
        if s == 0:
            (B["common"] if e["val"] else B["undefined"]).append(e)
            continue
        if not (1 <= s <= o.nsec):
            residue.append(e)
            continue
        si = s - 1
        if cls == SYM_CLASS_STATIC and e["naux"] == 1:
            B["section_def"].append(e)
            continue
        # a real definition
        prev = names_defined.get(e["name"])
        if prev is not None:
            conflicts.append((e["name"], prev, e["idx"]))
        else:
            names_defined[e["name"]] = e["idx"]
        lead = not claimed[si]
        claimed[si] = True
        if o.is_code(si):
            B["code_comdat_leader" if (lead and o.is_comdat(si))
              else "code_other"].append(e)
        elif o.is_data(si):
            B["data_comdat_leader" if (lead and o.is_comdat(si))
              else "data_other"].append(e)
        else:
            B["defined_other_sec"].append(e)

    unclaimed_comdat = [o.sections[i]["name"] for i in range(o.nsec)
                        if o.is_comdat(i) and not claimed[i]]
    return B, residue, conflicts, unclaimed_comdat


def sets(o):
    """The name sets this lane consumes, widest-reading first.

    `D_all` is deliberately the widest — every symbol with a real section
    number — because that is the definition `w-skip/mutate_owner.py` used when
    it measured 10/10 against 0/10 through real c2, and a narrower one here
    would let a decoder's blind spot look like a filter.
    """
    B, residue, conflicts, unclaimed = classify(o)
    defined = (B["code_comdat_leader"] + B["code_other"]
               + B["data_comdat_leader"] + B["data_other"]
               + B["defined_other_sec"])
    return {
        "E": sorted(set(e["name"] for e in B["code_comdat_leader"])),
        "D_all": sorted(set(e["name"] for e in defined if e["name"])),
        "D_data": sorted(set(e["name"] for e in
                             B["data_comdat_leader"] + B["data_other"]
                             if e["name"])),
        "D_lead": sorted(set(e["name"] for e in B["data_comdat_leader"]
                             if e["name"])),
        "U_undef": sorted(set(e["name"] for e in B["undefined"] if e["name"])),
        "buckets": {k: len(v) for k, v in B.items()},
        "residue": [e["name"] for e in residue],
        "conflicts": conflicts,
        "unclaimed_comdat": unclaimed,
        "arity": o.arity,
        "sections": len(o.sections),
        "secnames": sorted(set(s["name"] for s in o.sections)),
    }


def name_rule_E(o):
    """The `.text`-prefix rule, for the disagreement report only."""
    claimed = [False] * o.nsec
    out = []
    for e in o.entities:
        s = e["sec"]
        if not (1 <= s <= o.nsec):
            continue
        if e["cls"] == SYM_CLASS_STATIC and e["naux"] == 1:
            continue
        si = s - 1
        lead = not claimed[si]
        claimed[si] = True
        if lead and o.sections[si]["name"].startswith(".text") \
                and o.is_comdat(si):
            out.append(e["name"])
    return sorted(set(out))
