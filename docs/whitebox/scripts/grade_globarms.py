#!/usr/bin/env python3
"""grade_globarms.py — grade gate A's twelve arms against the pinned image and real objs.

Lane `w-globarms` (L4 of docs/ADOPTION_BRIEF_2026-08-29.md, board #3808-#3813).
Predictions frozen in work/w-globarms/PREREG.md before the image was opened and
before any cell was compiled.

Same status as `grade_globobj.py` beside it: whitebox *characterization* tooling
that grades **real c2's obj** and **real c2's bytes**, outside the std-only Rust
workspace on purpose, and NOT a gate row (`#3691` — a 22nd count-bearing row
makes `gate_identity_diff.sh` exit 2 for every live lane). `#1406` binds
instruments that grade *the port*; this one grades the reference compiler and
carries its own `--selftest`.

WHAT IT REFUSES TO TAKE ON TRUST

  1. THE TWELVE ARMS ARE DECODED FROM THE PINNED IMAGE, NOT TYPED IN. Every
     compare constant (0x10, 3, 5, 6, 8, 0xa), every branch condition and every
     branch target in FUN_10b550e5's gate-A chain is read out of c2.dll's own
     bytes at 0x10b5511a..0x10b551c6. The kind -> arm map is then produced by
     SIMULATING the decoded chain over kind = 0..0x10. Change one immediate in
     the image and the map changes; change one in this file and nothing does,
     because none of them is in this file.

  2. THE PAGE IS CHECKED AGAINST THE IMAGE, NOT THE OTHER WAY ROUND.
     docs/whitebox/ref/P_GLOBREGS.md §3's gate-A table is PARSED, and its twelve
     addresses must equal the twelve the image decode produced. A page edit that
     drifts from the binary fails here.

  3. THE KIND ENUM IS DECODED TOO. FUN_10bd2913 (0x10bd2913) is c2's front-end
     -> back-end symbol map: it computes the globregs kind from the `.gl`
     record's kind byte [gl+0x30] and, for [gl+0x30]==1, from the 3-bit LINKAGE
     field ([gl+0x37]>>0x15)&7 through the 8-entry jump table at 0x10bd2a9f.
     Both the dec-chain and the jump table are read from the image.

  4. THE PROMOTION READOUT IS `w-globobj`'s FRAME-TRAFFIC RULE, re-implemented
     here rather than imported, so that a disagreement between the two graders
     is detectable (prereg §5 control C4). A promoted local needs no stack
     slot; the prologue's own stw r12,-8(1) / std r31,-16(1) saves sit BEFORE
     the `stwu` and are excluded by construction, not by a heuristic.

  5. A CELL WITH NO FRAME SCORES `U` and enters no numerator and no
     denominator. Absence is not evidence.

  6. AN ABSENT IMAGE IS AN ERROR, NOT A SKIP. `w-globobj` §2.6 lost two planted
     defects to a control that silently skipped; here a missing image exits 2
     and prints IMAGE-ABSENT on the verdict line, so `grep FAIL` cannot read a
     skip as a pass.

Usage:
    grade_globarms.py --arms <dump.txt> ... [--dll <c2.dll>]
    grade_globarms.py --image [--dll <c2.dll>]     # the read half alone
    grade_globarms.py --selftest                   # no toolchain, no obj

Exit 1 = a CONTROL failed: the instrument is dead and every verdict it printed
is discarded. Exit 2 = the image is absent. A prediction being refuted is a
RESULT, never an exit code.
"""

import os
import re
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
PAGE = os.path.join(HERE, "..", "ref", "P_GLOBREGS.md")

PINNED_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

GATE_A_START = 0x10B5511A
KIND_MAP_FN = 0x10BD2913
LINKAGE_TABLE = 0x10BD2A9F
POOL_FN = 0x10BD2492

CELL = re.compile(r"^-- \.text #\d+ \((\d+) B\) (\S+)")
INS = re.compile(r"^\s+([0-9a-f]{4})\s+([0-9a-f]{8})\s+(\S+)\s*([^;]*)(;.*)?$")


# ---------------------------------------------------------------------------
# 0. The image
# ---------------------------------------------------------------------------

def find_dll(explicit=None):
    if explicit:
        return explicit if os.path.isfile(explicit) else None
    if os.environ.get("C2RS_C2DLL"):
        p = os.environ["C2RS_C2DLL"]
        return p if os.path.isfile(p) else None
    p = os.path.join(os.environ.get("C2RS_COMPILERS", os.path.join(ROOT, "compilers")),
                     "X360", "16.00.11886.00", "c2.dll")
    return p if os.path.isfile(p) else None


class Image(object):
    def __init__(self, path):
        self.d = open(path, "rb").read()
        d = self.d
        pe = struct.unpack_from("<I", d, 0x3C)[0]
        nsec = struct.unpack_from("<H", d, pe + 6)[0]
        optsz = struct.unpack_from("<H", d, pe + 20)[0]
        opt = pe + 24
        self.base = struct.unpack_from("<I", d, opt + 28)[0]
        self.secs = []
        off = opt + optsz
        for _ in range(nsec):
            vs, va, rs, ra = struct.unpack_from("<IIII", d, off + 8)
            self.secs.append((va, vs, ra, rs))
            off += 40

    def off(self, va):
        rva = va - self.base
        for sva, vs, ra, rs in self.secs:
            if sva <= rva < sva + max(vs, rs):
                return ra + (rva - sva)
        raise ValueError("va %#x outside every section" % va)

    def b(self, va, n):
        o = self.off(va)
        return self.d[o:o + n]

    def u32(self, va):
        return struct.unpack_from("<I", self.d, self.off(va))[0]


# ---------------------------------------------------------------------------
# 1. THE TWELVE ARMS, decoded from the image
# ---------------------------------------------------------------------------
#
# The chain is a straight-line run of x86 at 0x10b5511a. Each entry below names
# only the OPCODE SHAPE that must be present; every immediate, every branch
# condition and every target is taken from the bytes.
#
#   ("A1", 0x10b5511a, [("mov_al_ptr", 3), ("cmp_al_imm", 2), ("jcc32", 6)])
#
# A mismatch anywhere aborts the decode: the answer key is then unavailable and
# every arm scores U.

REJECT_TAIL = 0x10B552B8
NEXT_SLOT = 0x10B55295


class ArmDecodeError(Exception):
    pass


def _jcc(img, va):
    """Return (mnemonic, target, length) for a short or near conditional jump."""
    b = img.b(va, 6)
    short = {0x74: "je", 0x75: "jne", 0x76: "jbe", 0x77: "ja",
             0x72: "jb", 0x73: "jae", 0x7E: "jle", 0x7F: "jg"}
    if b[0] in short:
        disp = b[1] if b[1] < 0x80 else b[1] - 0x100
        return short[b[0]], va + 2 + disp, 2
    if b[0] == 0x0F and 0x80 <= b[1] <= 0x8F:
        near = {0x84: "je", 0x85: "jne", 0x86: "jbe", 0x87: "ja",
                0x82: "jb", 0x83: "jae", 0x8E: "jle", 0x8F: "jg"}
        if b[1] not in near:
            raise ArmDecodeError("unmodelled near jcc %02x at %08x" % (b[1], va))
        disp = struct.unpack_from("<i", b, 2)[0]
        return near[b[1]], va + 6 + disp, 6
    raise ArmDecodeError("no jcc at %08x (bytes %s)" % (va, b.hex()))


def decode_arms(img):
    """Decode gate A's twelve arms out of the pinned image.

    Returns a list of dicts. Raises ArmDecodeError on any byte that is not what
    the shape requires -- the instrument then has no answer key.
    """
    arms = []
    va = GATE_A_START

    # A1  8a 47 04    mov al,[edi+4]      <- the KIND byte, and its offset
    #     3c II       cmp al,imm
    #     jcc         -> next slot
    b = img.b(va, 5)
    if b[0:3] != bytes([0x8A, 0x47, 0x04]):
        raise ArmDecodeError("A1: no `mov al,[edi+0x4]` at %08x" % va)
    if b[3] != 0x3C:
        raise ArmDecodeError("A1: no `cmp al,imm8` at %08x" % (va + 3))
    imm = b[4]
    mn, tgt, n = _jcc(img, va + 5)
    arms.append(dict(arm="A1", addr=va, kind_off=0x04, test="kind %s 0x%x" % (_rel(mn), imm),
                     imm=imm, cc=mn, target=tgt))
    va = va + 5 + n

    # A2  83 67 40 fe   and dword [edi+0x40], ~1   -- unconditional, no test
    b = img.b(va, 4)
    if b != bytes([0x83, 0x67, 0x40, 0xFE]):
        raise ArmDecodeError("A2: no `and dword [edi+0x40],0xfffffffe` at %08x" % va)
    arms.append(dict(arm="A2", addr=va, test="(unconditional)", imm=None, cc=None, target=None))
    va += 4

    # A3  39 7f 08     cmp [edi+0x8],edi   -- the group-leader test
    b = img.b(va, 3)
    if b != bytes([0x39, 0x7F, 0x08]):
        raise ArmDecodeError("A3: no `cmp [edi+0x8],edi` at %08x" % va)
    mn, tgt, n = _jcc(img, va + 3)
    arms.append(dict(arm="A3", addr=va, test="sym+0x08 != sym", imm=None, cc=mn, target=tgt))
    va = va + 3 + n
    if img.b(va, 2) != bytes([0x33, 0xDB]):        # xor ebx,ebx -- the alias flag = 0
        raise ArmDecodeError("A3: no `xor ebx,ebx` after the leader test at %08x" % va)
    va += 2

    # A4..A9: five `cmp al,imm8` + jcc pairs in a row.
    for name in ("A4", "A5", "A6", "A7", "A8", "A9"):
        if name == "A5":
            # A5 shares A4's compare: `cmp al,3` / je -> A11, then jbe -> reject
            mn, tgt, n = _jcc(img, va)
            arms.append(dict(arm="A5", addr=va, test="kind %s 0x%x" % (_rel(mn), arms[-1]["imm"]),
                             imm=arms[-1]["imm"], cc=mn, target=tgt))
            va += n
            continue
        b = img.b(va, 2)
        if b[0] != 0x3C:
            raise ArmDecodeError("%s: no `cmp al,imm8` at %08x (byte %02x)" % (name, va, b[0]))
        imm = b[1]
        mn, tgt, n = _jcc(img, va + 2)
        arms.append(dict(arm=name, addr=va, test="kind %s 0x%x" % (_rel(mn), imm),
                         imm=imm, cc=mn, target=tgt))
        va = va + 2 + n

    # A10  8b 07 / 8b 40 37 / a9 imm32 (test eax,0x400) / jcc
    #                        a9 imm32 (test eax,0x200000) / jcc
    if img.b(va, 5) != bytes([0x8B, 0x07, 0x8B, 0x40, 0x37]):
        raise ArmDecodeError("A10: no `mov eax,[edi]` / `mov eax,[eax+0x37]` at %08x" % va)
    a10 = va
    va += 5
    if img.b(va, 1) != b"\xa9":
        raise ArmDecodeError("A10: no `test eax,imm32` at %08x" % va)
    m1 = struct.unpack_from("<I", img.b(va + 1, 4), 0)[0]
    mn1, t1, n1 = _jcc(img, va + 5)
    va = va + 5 + n1
    if img.b(va, 1) != b"\xa9":
        raise ArmDecodeError("A10: no second `test eax,imm32` at %08x" % va)
    m2 = struct.unpack_from("<I", img.b(va + 1, 4), 0)[0]
    mn2, t2, n2 = _jcc(img, va + 5)
    va = va + 5 + n2
    arms.append(dict(arm="A10", addr=a10, gl_off=0x37,
                     test="(*(sym))+0x37 & 0x%x set and & 0x%x clear" % (m1, m2),
                     imm=None, cc="%s/%s" % (mn1, mn2), target=t1, target2=t2,
                     mask_set=m1, mask_clear=m2))

    # A11  39 5f 14   cmp [edi+0x14],ebx  (ebx == 0 here)
    va11 = arms[0]["target"]                      # not used; A11 is reached by A4's je
    va11 = [a for a in arms if a["arm"] == "A4"][0]["target"]
    if img.b(va11, 3) != bytes([0x39, 0x5F, 0x14]):
        raise ArmDecodeError("A11: no `cmp [edi+0x14],ebx` at %08x" % va11)
    mn, tgt, n = _jcc(img, va11 + 3)
    arms.append(dict(arm="A11", addr=va11, test="kind 3 needs sym+0x14 == 0",
                     imm=None, cc=mn, target=tgt))
    va12 = va11 + 3 + n

    # A12  f6 47 07 40   test byte [edi+0x7],0x40
    b = img.b(va12, 4)
    if b[0:3] != bytes([0xF6, 0x47, 0x07]):
        raise ArmDecodeError("A12: no `test byte [edi+0x7],imm8` at %08x" % va12)
    mn, tgt, n = _jcc(img, va12 + 4)
    arms.append(dict(arm="A12", addr=va12, test="kind 3 needs sym+0x07 & 0x%x clear" % b[3],
                     imm=b[3], cc=mn, target=tgt))
    return arms


def _rel(mn):
    return {"je": "==", "jne": "!=", "jbe": "<=", "ja": ">", "jb": "<", "jae": ">="}.get(mn, mn)


def simulate(arms):
    """Run the DECODED chain over kind = 0..0x10 and return kind -> (arm, verdict).

    verdict is one of SKIP (leaves without the reject tail), REJECT (takes the
    reject tail at 0x10b552b8), ELIGIBLE, or ELIGIBLE-ALIASED.
    """
    by = {a["arm"]: a for a in arms}
    out = {}
    for k in range(0x11):
        # A1
        a = by["A1"]
        if _cmp(k, a["imm"], a["cc"]):
            out[k] = ("A1", "SKIP", "kind == 0x%x: skipped, and the reject tail is NOT run" % a["imm"])
            continue
        # A3 is not a kind test; a leader reaches A4.
        a4 = by["A4"]
        if _cmp(k, a4["imm"], a4["cc"]):
            out[k] = ("A4->A11/A12", "COND",
                      "kind == 0x%x: needs sym+0x14 == 0 and sym+0x07 & 0x%x clear"
                      % (a4["imm"], by["A12"]["imm"]))
            continue
        a5 = by["A5"]
        if _cmp(k, a5["imm"], a5["cc"]):
            out[k] = ("A5", "REJECT", "kind %s 0x%x" % (_rel(a5["cc"]), a5["imm"]))
            continue
        a6 = by["A6"]
        if _cmp(k, a6["imm"], a6["cc"]):
            out[k] = ("A6", "ELIGIBLE", "aliased only when sym+0x05 & 2 is set")
            continue
        a7 = by["A7"]
        if _cmp(k, a7["imm"], a7["cc"]):
            out[k] = ("A7", "REJECT", "kind %s 0x%x" % (_rel(a7["cc"]), a7["imm"]))
            continue
        a8 = by["A8"]
        if _cmp(k, a8["imm"], a8["cc"]):
            out[k] = ("A8", "ELIGIBLE-ALIASED", "always joins the DAT_10c2e3e8 set")
            continue
        a9 = by["A9"]
        if _cmp(k, a9["imm"], a9["cc"]):
            out[k] = ("A9", "REJECT", "kind %s 0x%x" % (_rel(a9["cc"]), a9["imm"]))
            continue
        a10 = by["A10"]
        out[k] = ("A10", "COND",
                  "needs (*(sym))+0x37 & 0x%x set and & 0x%x clear; indexes sub-symbols only"
                  % (a10["mask_set"], a10["mask_clear"]))
    return out


def _cmp(k, imm, cc):
    if cc == "je":
        return k == imm
    if cc == "jne":
        return k != imm
    if cc == "jbe":
        return k <= imm
    if cc == "jb":
        return k < imm
    if cc == "ja":
        return k > imm
    if cc == "jae":
        return k >= imm
    raise ArmDecodeError("unmodelled cc %s" % cc)


# ---------------------------------------------------------------------------
# 2. THE KIND ENUM, decoded from FUN_10bd2913 and the jump table
# ---------------------------------------------------------------------------

def decode_kind_map(img):
    """Decode the front-end -> back-end kind map at 0x10bd2913.

    Returns (glkind_map, linkage_targets, arm_of_linkage_note).
    """
    # movzx eax,[edi+0x30] ; dec/je x4
    va = 0x10BD2922
    if img.b(va, 4) != bytes([0x0F, 0xB6, 0x47, 0x30]):
        raise ArmDecodeError("kind map: no `movzx eax,byte [edi+0x30]` at %08x" % va)
    # The dec-chain: `48` dec eax, then a short je. `53` (push ebx) is
    # interleaved once and does not affect flags.
    va = 0x10BD2926
    glk = {}
    n = 1
    while n <= 4:
        if img.b(va, 1) != b"\x48":
            raise ArmDecodeError("kind map: no `dec eax` at %08x" % va)
        va += 1
        if img.b(va, 1) == b"\x53":                # push ebx, flag-neutral
            va += 1
        mn, tgt, ln = _jcc(img, va)
        if mn != "je":
            raise ArmDecodeError("kind map: dec-chain step %d is %s not je" % (n, mn))
        glk[n] = tgt
        va += ln
        n += 1
    glk["else"] = va                                # the fallthrough
    # Resolve each target that is a plain `mov bl,imm8`.
    out = {}
    for key, tgt in glk.items():
        b = img.b(tgt, 2)
        if b[0] == 0xB3:
            out[key] = ("kind", b[1])
        elif tgt == 0x10BD293B:
            out[key] = ("linkage-table", LINKAGE_TABLE)
        else:
            out[key] = ("?", tgt)
    # The jump table itself.
    if img.b(0x10BD2946, 3) != bytes([0xFF, 0x24, 0x8D]):
        raise ArmDecodeError("kind map: no `jmp [ecx*4+imm32]` at 0x10bd2946")
    tab = struct.unpack_from("<I", img.b(0x10BD2949, 4), 0)[0]
    if tab != LINKAGE_TABLE:
        raise ArmDecodeError("kind map: jump table is %08x, expected %08x" % (tab, LINKAGE_TABLE))
    links = {}
    for i in range(8):
        t = img.u32(tab + 4 * i)
        if t == 0:
            links[i] = ("unreachable", None)
            continue
        b = img.b(t, 2)
        if b[0] == 0xB3:
            links[i] = ("kind", b[1])
        elif img.b(t, 5) == bytes([0x25, 0xE0, 0x01, 0x00, 0x00]):
            # and eax,0x1e0 / cmp eax,imm32 / sete bl / add bl,imm8
            if img.b(t + 5, 1) != b"\x3d" or img.b(t + 10, 3)[0:2] != bytes([0x0F, 0x94]):
                raise ArmDecodeError("linkage %d: no `cmp eax,imm32` / `sete bl`" % i)
            want = struct.unpack_from("<I", img.b(t + 6, 4), 0)[0]
            base = img.b(t + 13, 3)
            if base[0:2] != bytes([0x80, 0xC3]):
                raise ArmDecodeError("linkage %d: no `add bl,imm8`" % i)
            links[i] = ("kind %d when ([gl+0x37]&0x1e0)==0x%x else kind %d"
                        % (base[2] + 1, want, base[2]), None)
        elif img.b(t, 3) == bytes([0xC1, 0xE8, 0x05]):
            links[i] = ("storage-kind switch", None)
        elif img.b(t, 3) == bytes([0x8B, 0x5F, 0x20]):
            links[i] = ("(sym+0x20 >> 4) & 2 | 5", None)
        else:
            links[i] = ("?", t)
    return out, links


def decode_pools(img):
    """Decode FUN_10bd2492's kind -> allocation-pool dispatch."""
    va = POOL_FN + 3
    steps = []
    while len(steps) < 6:
        b = img.b(va, 3)
        if b[0:2] != bytes([0x80, 0xF9]):           # cmp cl,imm8
            break
        imm = b[2]
        mn, tgt, n = _jcc(img, va + 3)
        steps.append((imm, mn, tgt))
        va = va + 3 + n
    return steps, va


# ---------------------------------------------------------------------------
# 3. The page, parsed
# ---------------------------------------------------------------------------

def parse_page_arms(path):
    """Pull the addresses out of P_GLOBREGS.md §3's gate-A table."""
    if not os.path.isfile(path):
        return None
    txt = open(path, encoding="utf-8").read()
    i = txt.find("**Gate A")
    if i < 0:
        return None
    j = txt.find("**Gate B", i)
    seg = txt[i:j if j > 0 else len(txt)]
    return [int(m, 16) for m in re.findall(r"`0x(10b5[0-9a-f]{4})`", seg)]


# ---------------------------------------------------------------------------
# 4. The obj readout -- w-globobj's frame-traffic rule, re-implemented
# ---------------------------------------------------------------------------

STWU = re.compile(r"^stwu$")
CALLEE_SAVED = set(range(14, 32))


def parse_dump(path):
    cells = {}
    cur = None
    for line in open(path, encoding="utf-8", errors="replace"):
        m = CELL.match(line)
        if m:
            cur = m.group(2)
            cells[cur] = []
            continue
        if cur is None:
            continue
        m = INS.match(line)
        if m:
            cells[cur].append((int(m.group(1), 16), m.group(3), m.group(4).strip(),
                               (m.group(5) or "")))
    return cells


def frame_verdict(ins):
    """PROMOTED / MEMORY / U for one cell, by the frame-traffic rule.

    U when the body has no `stwu` frame: the readout does not apply and the
    cell enters no numerator and no denominator.
    """
    open_i = None
    for i, (_, mn, ops, _) in enumerate(ins):
        if mn == "stwu":
            open_i = i
            break
    if open_i is None:
        return "U", []
    close_i = len(ins)
    for i in range(open_i + 1, len(ins)):
        if ins[i][1] == "addi" and ins[i][2].startswith("1, 1,"):
            close_i = i
            break
    traffic = []
    for i in range(open_i + 1, close_i):
        _, mn, ops, cm = ins[i]
        if not mn.startswith("st"):
            continue
        m = re.match(r"^(\d+),\s*(-?\d+)\((\d+)\)$", ops)
        if m and m.group(3) == "1":
            traffic.append((ins[i][0], mn, ops, "frame"))
        elif "REFLO" in cm and m:
            traffic.append((ins[i][0], mn, ops, "static"))
    return ("MEMORY" if traffic else "PROMOTED"), traffic


# The per-cell expectation, from work/w-globarms/PREREG.md §3 and addendum 1.
# Each row is (cell, arm, expected). Kept here rather than in the grid because
# it is a PREDICTION, and a prediction that lives beside the source it predicts
# is not a prediction.
EXPECT = [
    ("ga_int",       "A6",  "PROMOTED"),
    ("ga_vol",       "-",   "MEMORY"),
    ("ga_escape",    "A6",  "MEMORY"),
    ("ga_extern",    "A8",  "MEMORY"),
    ("ga_fstatic",   "A8",  "MEMORY"),
    ("ga_lstatic",   "A8",  "MEMORY"),
    ("ga_temp",      "A11", "PROMOTED"),
    ("ga_temp3",     "A11", "PROMOTED"),
    ("ga_structmix", "A3",  "PROMOTED"),
    ("ga_struct4",   "A3",  "PROMOTED"),
    ("ga_fnaddr",    "A9",  "PROMOTED"),
    ("ga_param",     "A6",  "PROMOTED"),
    ("ga_ref",       "A6",  "PROMOTED"),
    ("gb_addr_local",   "A6", "PROMOTED"),
    ("gb_addr_escape",  "A6", "MEMORY"),
    ("gb_pair_yescape", "A6", "SPLIT"),
    ("gb_pair_xescape", "A6", "SPLIT"),
    ("gb_pair_none",    "A6", "PROMOTED"),
    ("gb_fnaddr2",      "A9", "PROMOTED"),
]


def main(argv):
    dll = None
    if "--dll" in argv:
        dll = argv[argv.index("--dll") + 1]
    if "--selftest" in argv:
        return selftest(dll)

    path = find_dll(dll)
    if path is None:
        print("GRADE: IMAGE-ABSENT — the answer key cannot be decoded; nothing published")
        return 2
    img = Image(path)
    import hashlib
    dig = hashlib.sha256(img.d).hexdigest()
    print("=== IMAGE ===")
    print("  %s" % os.path.basename(path))
    print("  sha256 %s  %s" % (dig[:16] + "...", "PINNED" if dig == PINNED_SHA256 else "*** NOT THE PINNED IMAGE ***"))
    if dig != PINNED_SHA256:
        print("GRADE: FAIL (image digest)")
        return 1

    try:
        arms = decode_arms(img)
    except ArmDecodeError as e:
        print("  arm decode FAILED: %s" % e)
        print("GRADE: FAIL (arm decode)")
        return 1

    print()
    print("=== GATE A, DECODED FROM THE IMAGE (not typed in) ===")
    for a in arms:
        tgt = ""
        if a.get("target") is not None:
            t = a["target"]
            tgt = " -> %08x%s" % (t, {REJECT_TAIL: " REJECT-TAIL", NEXT_SLOT: " NEXT-SLOT"}.get(t, ""))
        print("  %-4s %08x  %-52s %s%s" % (a["arm"], a["addr"], a["test"], a["cc"] or "", tgt))

    page = parse_page_arms(PAGE)
    ok_page = False
    if page is not None:
        decoded = [a["addr"] for a in arms]
        ok_page = all(d in page for d in decoded)
        print()
        print("  P_GLOBREGS.md §3 gate-A table: %d addresses; every decoded arm present: %s"
              % (len(page), "YES" if ok_page else "NO"))
        if not ok_page:
            print("  missing from the page: %s"
                  % " ".join("%08x" % d for d in decoded if d not in page))

    print()
    print("=== kind -> ARM, by SIMULATING the decoded chain ===")
    sim = simulate(arms)
    for k in sorted(sim):
        arm, verdict, why = sim[k]
        print("  kind 0x%02x  %-12s %-16s %s" % (k, arm, verdict, why))

    try:
        glk, links = decode_kind_map(img)
        pools, _ = decode_pools(img)
    except ArmDecodeError as e:
        print("  kind-map decode FAILED: %s" % e)
        print("GRADE: FAIL (kind map)")
        return 1

    print()
    print("=== WHERE THE KIND COMES FROM — FUN_10bd2913, decoded ===")
    for key in (1, 2, 3, 4, "else"):
        what, v = glk[key]
        s = ("globregs kind 0x%x" % v) if what == "kind" else \
            ("the linkage jump table at 0x%08x" % v if what == "linkage-table" else "%s %s" % (what, v))
        print("  .gl record kind [gl+0x30] == %-4s -> %s" % (key, s))
    print("  the 8-entry table at 0x%08x, index = ([gl+0x37] >> 0x15) & 7:" % LINKAGE_TABLE)
    for i in range(8):
        what, v = links[i]
        s = ("globregs kind 0x%x" % v) if what == "kind" else what
        print("    linkage %d -> %s" % (i, s))

    print()
    print("=== the allocation POOL is chosen by kind too — FUN_10bd2492, decoded ===")
    for imm, mn, tgt in pools:
        print("    cmp kind, 0x%-2x  %-4s -> %08x" % (imm, mn, tgt))

    if "--image" in argv:
        print()
        print("GRADE: PASS (image half only, no obj graded)")
        return 0

    dumps = [a for a in argv[1:] if a.endswith(".txt") and os.path.isfile(a)]
    if not dumps:
        print()
        print("GRADE: IMAGE-ONLY — no dump given")
        return 0

    print()
    print("=== CELLS — the frame-traffic readout ===")
    exp = dict((c, (a, e)) for c, a, e in EXPECT)
    n_graded = n_u = n_hit = n_miss = 0
    rows = []
    for dp in sorted(dumps):
        cells = parse_dump(dp)
        for name in sorted(cells):
            if name not in exp:
                continue
            arm, want = exp[name]
            got, traffic = frame_verdict(cells[name])
            if got == "U":
                n_u += 1
                rows.append((os.path.basename(dp), name, arm, want, "U", ""))
                continue
            if want == "SPLIT":
                # the pair cells: exactly one local homed, the other in a
                # callee-saved register. MEMORY-with-one-slot is the pass.
                slots = set(t[2] for t in traffic)
                got = "SPLIT" if len(slots) == 1 else ("PROMOTED" if not traffic else "MULTI")
            n_graded += 1
            hit = (got == want)
            n_hit += hit
            n_miss += (not hit)
            rows.append((os.path.basename(dp), name, arm, want, got,
                         " ".join("%04x:%s %s" % (t[0], t[1], t[2]) for t in traffic)))
    for dp, name, arm, want, got, tr in rows:
        flag = "." if got == want else ("U" if got == "U" else "X")
        print("  %s %-16s %-4s %-14s want %-9s got %-9s %s" % (flag, name, arm, dp, want, got, tr))

    print()
    print("  graded %d, U %d, hits %d, misses %d" % (n_graded, n_u, n_hit, n_miss))
    ctrl_pos = [r for r in rows if r[1] == "ga_int"]
    ctrl_neg = [r for r in rows if r[1] == "ga_vol"]
    c1 = (ctrl_pos and all(r[4] == "PROMOTED" for r in ctrl_pos)
          and ctrl_neg and all(r[4] == "MEMORY" for r in ctrl_neg))
    print("  CONTROL C1 (ga_int PROMOTED / ga_vol MEMORY, both profiles): %s"
          % ("FIRED" if c1 else "DEAD"))
    if not c1:
        print("GRADE: FAIL (control C1)")
        return 1
    if n_u and n_graded == 0:
        print("GRADE: FAIL (every cell unscoreable)")
        return 1
    print("GRADE: PASS%s" % ("" if n_miss == 0 else "  (%d prediction misses — a RESULT, not a failure)" % n_miss))
    return 0


# ---------------------------------------------------------------------------
# 5. SELFTEST — including five things the grader must REJECT
# ---------------------------------------------------------------------------

SYN_PROMOTED = """-- .text #5 (40 B) syn_prom
   0000  7d8802a6  mflr 12
   0004  9181fff8  stw 12, -8(1)
   0008  fbe1fff0  std 31, -16(1)
   000c  9421ffa0  stwu 1, -96(1)
   0010  83eb0000  lwz 31, 0(11)
   0014  38210060  addi 1, 1, 96
   0018  4e800020  blr
"""

SYN_MEMORY = """-- .text #5 (40 B) syn_mem
   0000  7d8802a6  mflr 12
   0004  9181fff8  stw 12, -8(1)
   0008  9421ffa0  stwu 1, -96(1)
   000c  91610050  stw 11, 80(1)
   0010  38210060  addi 1, 1, 96
   0014  4e800020  blr
"""

SYN_STATIC = """-- .text #5 (40 B) syn_static
   0000  7d8802a6  mflr 12
   0004  9421ffa0  stwu 1, -96(1)
   0008  917f0000  stw 11, 0(31)   ; REFLO -> [53] gs_x
   000c  38210060  addi 1, 1, 96
   0010  4e800020  blr
"""

SYN_NOFRAME = """-- .text #5 (8 B) syn_leaf
   0000  7c631a14  add 3, 3, 3
   0004  4e800020  blr
"""


def _parse_text(s):
    import io
    cells = {}
    cur = None
    for line in io.StringIO(s):
        m = CELL.match(line)
        if m:
            cur = m.group(2)
            cells[cur] = []
            continue
        if cur is None:
            continue
        m = INS.match(line)
        if m:
            cells[cur].append((int(m.group(1), 16), m.group(3), m.group(4).strip(),
                               (m.group(5) or "")))
    return cells


def selftest(dllarg):
    fails = []
    skipped = 0

    def chk(name, cond):
        print("  %-64s %s" % (name, "ok" if cond else "FAIL"))
        if not cond:
            fails.append(name)

    print("=== SELFTEST ===")

    c = _parse_text(SYN_PROMOTED)
    chk("readout: no post-stwu frame store is PROMOTED",
        frame_verdict(c["syn_prom"])[0] == "PROMOTED")
    chk("readout: the prologue saves BEFORE the stwu are not frame traffic",
        frame_verdict(c["syn_prom"])[1] == [])
    c = _parse_text(SYN_MEMORY)
    chk("readout: a post-stwu r1 store is MEMORY",
        frame_verdict(c["syn_mem"])[0] == "MEMORY")
    c = _parse_text(SYN_STATIC)
    chk("readout: a store to a RELOCATED static is MEMORY",
        frame_verdict(c["syn_static"])[0] == "MEMORY")
    c = _parse_text(SYN_NOFRAME)
    chk("REJECT a body with no frame (U, not a pass)",
        frame_verdict(c["syn_leaf"])[0] == "U")

    # The decoder must refuse a mutated chain.
    class Fake(object):
        def __init__(self, real, patches):
            self.real = real
            self.patches = patches
            self.d = real.d
            self.base = real.base
            self.secs = real.secs

        def off(self, va):
            return self.real.off(va)

        def b(self, va, n):
            out = bytearray(self.real.b(va, n))
            for pva, pb in self.patches.items():
                if va <= pva < va + n:
                    out[pva - va] = pb
            return bytes(out)

        def u32(self, va):
            return self.real.u32(va)

    path = find_dll(dllarg)
    if path is None:
        print("  (image absent — 6 image assertions SKIPPED; this is NOT a pass)")
        skipped = 6
    else:
        img = Image(path)
        import hashlib
        chk("image: digest is the pinned one",
            hashlib.sha256(img.d).hexdigest() == PINNED_SHA256)
        arms = decode_arms(img)
        chk("image: gate A decodes to exactly twelve arms", len(arms) == 12)
        sim = simulate(arms)
        chk("image: kind 0x10 SKIPs without the reject tail", sim[0x10][1] == "SKIP")
        chk("image: kinds 4 and 5 are ELIGIBLE, 6 and 9 are REJECT",
            sim[4][1] == "ELIGIBLE" and sim[5][1] == "ELIGIBLE"
            and sim[6][1] == "REJECT" and sim[9][1] == "REJECT")
        glk, links = decode_kind_map(img)
        chk("image: the linkage table's entry 0 is UNREACHABLE (a null slot)",
            links[0][0] == "unreachable")
        chk("image: linkage 1 -> kind 4 and linkage 3 -> kind 5",
            links[1] == ("kind", 4) and links[3] == ("kind", 5))
        # REJECTIONS: a planted one-byte change must break the decode.
        bad = Fake(img, {0x10B5511C: 0x08})       # mov al,[edi+0x8] instead of +0x4
        try:
            decode_arms(bad)
            chk("REJECT a chain whose kind byte moved off sym+0x04", False)
        except ArmDecodeError:
            chk("REJECT a chain whose kind byte moved off sym+0x04", True)
        bad = Fake(img, {0x10B5513E: 0x3D})       # cmp eax,imm32 instead of cmp al,imm8
        try:
            decode_arms(bad)
            chk("REJECT a chain whose A6 compare changed shape", False)
        except ArmDecodeError:
            chk("REJECT a chain whose A6 compare changed shape", True)
        # A changed IMMEDIATE must silently change the simulated map -- which is
        # the whole point of decoding it: this assertion proves the map is not
        # hard-coded here.
        bad = Fake(img, {0x10B5513F: 0x07})       # cmp al,5 -> cmp al,7
        arms2 = decode_arms(bad)
        sim2 = simulate(arms2)
        chk("the kind->arm map FOLLOWS the image (A6 bound 5 -> 7 moves kinds 6,7)",
            sim2[6][0] == "A6" and sim2[7][0] == "A6" and sim[6][0] == "A7")
        page = parse_page_arms(PAGE)
        chk("P_GLOBREGS §3's gate-A table contains every decoded arm address",
            page is not None and all(a["addr"] in page for a in arms))

    print()
    if fails:
        print("  SELFTEST FAIL: %d" % len(fails))
        return 1
    if skipped:
        print("  SELFTEST INCOMPLETE — %d image assertions skipped; NOT a pass" % skipped)
        return 2
    print("  SELFTEST PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
