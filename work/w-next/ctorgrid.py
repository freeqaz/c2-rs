#!/usr/bin/env python3
"""ctorgrid.py — grade real `c2.dll` over the CONSTRUCTOR-INIT class.

The question this exists to answer, and nothing else:

    `xboxheap.cpp`'s one function places three producers at instruction slots
    0, 2 and 5 among six field stores.  Is that placement a CONSTANT of the
    shape (so a transcription can carry it) or a FUNCTION of the immediate
    fields (so it cannot)?

`w-hash` §5.1's discipline applies: this grid VARIES ONE AXIS AT A TIME and
carries an ANCHOR CONTROL — a cell that must reproduce `xboxheap`'s own twenty
words — so a harness that silently stopped grading is visible as a failed
anchor rather than as a clean sheet.

Outside the std-only Rust workspace on purpose: measurement tooling, never
linked into the port (same status as `scripts/gt_dump.py`).

Usage:
    ctorgrid.py [--jobs N] [--out DIR] [--only NAME]
"""

import os
import subprocess
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")
def _sib(name):
    """The sibling checkout, found by walking UP from the repo root.

    No absolute path is written in this file (CLAUDE.md): this tree may be the
    main repo or a worktree under `.claude/worktrees/<lane>/`, and those differ
    by three levels. Same locator as `work/w-frame/refobj.sh`.
    """
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")


# --- the shape under test -----------------------------------------------------
#
# `xboxheap`'s constructor, reduced to its skeleton.  Every cell below is this
# text with the marked fields substituted; the STRUCTURE (which store needs a
# producer, and in what source order) is held fixed except where a row says
# otherwise.
#
#   struct L { L* mNext; L* mPrev; };
#   struct H {
#       H* mFreeHead;   //  0   <- this
#       H* mUsedHead;   //  4   <- this
#       L  mListHead;   //  8   <- &mListHead (this+8), twice
#       unsigned mSize; // 16   <- formal
#       unsigned mCount;// 20   <- literal
#       H(unsigned a, unsigned b);
#       void AllocatePageBlock(unsigned);
#   };

TEMPLATE = """\
struct L%(tag)s { L%(tag)s* mNext; L%(tag)s* mPrev; };
struct H%(tag)s {
%(fields)s
    H%(tag)s(unsigned a, unsigned b);
    void* AllocatePageBlock(unsigned);
};
H%(tag)s::H%(tag)s(unsigned a, unsigned b) {
%(body)s
    AllocatePageBlock(%(callarg)s);
}
"""


def cell(tag, fields, body, callarg="a"):
    return TEMPLATE % dict(tag=tag, fields=fields, body=body, callarg=callarg)


# The anchor: byte-for-byte the layout and body order of `xboxheap.cpp`.
ANCHOR_FIELDS = """\
    H%(tag)s* mFreeHead;
    H%(tag)s* mUsedHead;
    L%(tag)s  mListHead;
    unsigned  mSize;
    unsigned  mCount;"""

ANCHOR_BODY = """\
    mSize = b;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;
    L%(tag)s& listHead = mListHead;
    listHead.mNext = &listHead;
    listHead.mPrev = &listHead;"""


def build_cells():
    cells = {}

    def add(name, fields, body, callarg="a"):
        t = name.replace("-", "_")
        cells[name] = cell(t, fields % {"tag": t}, body % {"tag": t}, callarg)

    # ---- ANCHOR ----------------------------------------------------------
    add("anchor", ANCHOR_FIELDS, ANCHOR_BODY)

    # ---- AXIS 1: the LITERAL stored into mCount ---------------------------
    for lit in ("1", "7", "255", "32767", "65536", "-1"):
        add(
            "lit%s" % lit.replace("-", "neg"),
            ANCHOR_FIELDS,
            ANCHOR_BODY.replace("mCount = 0;", "mCount = %s;" % lit),
        )

    # ---- AXIS 2: which FORMAL feeds mSize ---------------------------------
    add("formal-a", ANCHOR_FIELDS, ANCHOR_BODY.replace("mSize = b;", "mSize = a;"))

    # ---- AXIS 3: the INTERIOR OFFSET — move mListHead in the layout -------
    # mListHead first  => &mListHead == this + 0, so the `addi` should VANISH.
    add(
        "interior0",
        """\
    L%(tag)s  mListHead;
    H%(tag)s* mFreeHead;
    H%(tag)s* mUsedHead;
    unsigned  mSize;
    unsigned  mCount;""",
        ANCHOR_BODY,
    )
    # mListHead last => a larger interior offset.
    add(
        "interior16",
        """\
    H%(tag)s* mFreeHead;
    H%(tag)s* mUsedHead;
    unsigned  mSize;
    unsigned  mCount;
    L%(tag)s  mListHead;""",
        ANCHOR_BODY,
    )

    # ---- AXIS 4: SOURCE ORDER of the six stores ---------------------------
    # Monotone in offset instead of xboxheap's 16,0,20,4,8,12.
    add(
        "order-monotone",
        ANCHOR_FIELDS,
        """\
    mFreeHead = this;
    mUsedHead = this;
    L%(tag)s& listHead = mListHead;
    listHead.mNext = &listHead;
    listHead.mPrev = &listHead;
    mSize = b;
    mCount = 0;""",
    )
    # Literal store FIRST — moves the `li` producer's consumer to slot 0.
    add(
        "order-lit-first",
        ANCHOR_FIELDS,
        """\
    mCount = 0;
    mSize = b;
    mFreeHead = this;
    mUsedHead = this;
    L%(tag)s& listHead = mListHead;
    listHead.mNext = &listHead;
    listHead.mPrev = &listHead;""",
    )
    # Interior stores FIRST — moves the `addi` producer's consumer to slot 0.
    add(
        "order-interior-first",
        ANCHOR_FIELDS,
        """\
    L%(tag)s& listHead = mListHead;
    listHead.mNext = &listHead;
    listHead.mPrev = &listHead;
    mSize = b;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;""",
    )

    # ---- AXIS 5: NUMBER of stores ----------------------------------------
    add(
        "n4",
        ANCHOR_FIELDS,
        """\
    mSize = b;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;""",
    )
    add(
        "n2",
        ANCHOR_FIELDS,
        """\
    mSize = b;
    mCount = 0;""",
    )
    add(
        "n8",
        """\
    H%(tag)s* mFreeHead;
    H%(tag)s* mUsedHead;
    L%(tag)s  mListHead;
    unsigned  mSize;
    unsigned  mCount;
    unsigned  mX;
    unsigned  mY;""",
        ANCHOR_BODY + """
    mX = 3;
    mY = b;""",
    )

    # ---- AXIS 6: the RETURN — no call, so no `mr 31,3` at all -------------
    cells["nocall"] = TEMPLATE.replace(
        "    AllocatePageBlock(%(callarg)s);\n", ""
    ) % dict(
        tag="nocall",
        fields=ANCHOR_FIELDS % {"tag": "nocall"},
        body=ANCHOR_BODY % {"tag": "nocall"},
        callarg="a",
    )

    # ---- AXIS 7: the call ARGUMENT ---------------------------------------
    add("callarg-b", ANCHOR_FIELDS, ANCHOR_BODY, callarg="b")
    add("callarg-lit", ANCHOR_FIELDS, ANCHOR_BODY, callarg="42")
    add("callarg-field", ANCHOR_FIELDS, ANCHOR_BODY, callarg="mSize")

    # ---- AXIS 8: BOARD #644 — a producer that is NOT ONE INSTRUCTION ------
    # `li` covers simm16; `lis` alone covers a clean high half; anything else
    # needs `lis`+`ori`, i.e. a TWO-WORD producer, and #644's standing warning is
    # that c2 SPLITS such a producer across other instructions.  If it does, the
    # schedule is not a constant under this axis and the class must exclude it.
    for lit in ("65537", "123456", "0x12345", "2147483647"):
        add(
            "wide%s" % lit.replace("0x", "h"),
            ANCHOR_FIELDS,
            ANCHOR_BODY.replace("mCount = 0;", "mCount = %s;" % lit),
        )

    # ---- AXIS 9: FIELD WIDTH — stw is not the only store ------------------
    add(
        "field-short",
        """\
    H%(tag)s* mFreeHead;
    H%(tag)s* mUsedHead;
    L%(tag)s  mListHead;
    unsigned short mSize;
    unsigned short mCount;""",
        ANCHOR_BODY,
    )
    add(
        "field-char",
        """\
    H%(tag)s* mFreeHead;
    H%(tag)s* mUsedHead;
    L%(tag)s  mListHead;
    unsigned char mSize;
    unsigned char mCount;""",
        ANCHOR_BODY,
    )
    add(
        "field-ll",
        """\
    H%(tag)s* mFreeHead;
    H%(tag)s* mUsedHead;
    L%(tag)s  mListHead;
    unsigned long long mSize;
    unsigned long long mCount;""",
        ANCHOR_BODY,
    )

    # ---- AXIS 10: a THIRD formal, and a POINTER formal --------------------
    cells["formal3"] = """\
struct Lformal3 { Lformal3* mNext; Lformal3* mPrev; };
struct Hformal3 {
    Hformal3* mFreeHead; Hformal3* mUsedHead; Lformal3 mListHead;
    unsigned mSize; unsigned mCount;
    Hformal3(unsigned a, unsigned b, unsigned c);
    void* AllocatePageBlock(unsigned);
};
Hformal3::Hformal3(unsigned a, unsigned b, unsigned c) {
    mSize = b; mFreeHead = this; mCount = c; mUsedHead = this;
    Lformal3& listHead = mListHead;
    listHead.mNext = &listHead; listHead.mPrev = &listHead;
    AllocatePageBlock(a);
}
"""

    # ---- AXIS 11: a BASE CLASS (this-adjust) and a VIRTUAL (vptr store) ---
    cells["base"] = """\
struct Lbase { Lbase* mNext; Lbase* mPrev; };
struct Bbase { int pad0; int pad1; };
struct Hbase : Bbase {
    Hbase* mFreeHead; Hbase* mUsedHead; Lbase mListHead;
    unsigned mSize; unsigned mCount;
    Hbase(unsigned a, unsigned b);
    void* AllocatePageBlock(unsigned);
};
Hbase::Hbase(unsigned a, unsigned b) {
    mSize = b; mFreeHead = this; mCount = 0; mUsedHead = this;
    Lbase& listHead = mListHead;
    listHead.mNext = &listHead; listHead.mPrev = &listHead;
    AllocatePageBlock(a);
}
"""
    cells["virtual"] = """\
struct Lvirtual { Lvirtual* mNext; Lvirtual* mPrev; };
struct Hvirtual {
    virtual void v();
    Hvirtual* mFreeHead; Hvirtual* mUsedHead; Lvirtual mListHead;
    unsigned mSize; unsigned mCount;
    Hvirtual(unsigned a, unsigned b);
    void* AllocatePageBlock(unsigned);
};
Hvirtual::Hvirtual(unsigned a, unsigned b) {
    mSize = b; mFreeHead = this; mCount = 0; mUsedHead = this;
    Lvirtual& listHead = mListHead;
    listHead.mNext = &listHead; listHead.mPrev = &listHead;
    AllocatePageBlock(a);
}
"""

    # ---- AXIS 12: TWO calls, and a FREE-FUNCTION call ---------------------
    add("call2", ANCHOR_FIELDS, ANCHOR_BODY + """
    AllocatePageBlock(b);""")
    cells["callfree"] = """\
struct Lcallfree { Lcallfree* mNext; Lcallfree* mPrev; };
void FreeFn(unsigned);
struct Hcallfree {
    Hcallfree* mFreeHead; Hcallfree* mUsedHead; Lcallfree mListHead;
    unsigned mSize; unsigned mCount;
    Hcallfree(unsigned a, unsigned b);
};
Hcallfree::Hcallfree(unsigned a, unsigned b) {
    mSize = b; mFreeHead = this; mCount = 0; mUsedHead = this;
    Lcallfree& listHead = mListHead;
    listHead.mNext = &listHead; listHead.mPrev = &listHead;
    FreeFn(a);
}
"""

    # ---- AXIS 13: the REFERENCE LOCAL spelled three other ways -------------
    # `w-hash` §4.2: three source spellings emitted byte-identical `.text` from
    # three DIFFERENT IL productions, and it refused all three. Same check here.
    add(
        "spell-ptr",
        ANCHOR_FIELDS,
        """\
    mSize = b;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;
    L%(tag)s* listHead = &mListHead;
    listHead->mNext = listHead;
    listHead->mPrev = listHead;""",
    )
    add(
        "spell-direct",
        ANCHOR_FIELDS,
        """\
    mSize = b;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;
    mListHead.mNext = &mListHead;
    mListHead.mPrev = &mListHead;""",
    )

    return cells


def text_words(objpath):
    """Return {section-name: [hexword,...]} for every .text COMDAT."""
    out = subprocess.run(
        [sys.executable, DUMP, objpath, "--text-only"],
        capture_output=True, text=True,
    ).stdout
    secs, cur = {}, None
    for line in out.splitlines():
        if line.startswith("-- .text"):
            cur = line.split(") ", 1)[-1].strip()
            secs[cur] = []
        elif cur and line.strip() and line.strip()[0].isdigit() and len(line) > 12:
            parts = line.split()
            if len(parts) >= 2 and len(parts[1]) == 8:
                secs[cur].append((parts[1], " ".join(parts[2:]).split(";")[0].strip()))
    return secs


def run_cell(args):
    name, src, outdir = args
    cpp = os.path.join(outdir, name + ".cpp")
    with open(cpp, "w") as f:
        f.write(src)
    rel = os.path.relpath(cpp, DC3)
    obj = os.path.join(outdir, name + ".obj")
    r = subprocess.run([REFOBJ, rel, obj], capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None, (r.stderr or r.stdout)[-200:]
    return name, text_words(obj), None


def main():
    jobs = 8
    outdir = os.path.join(HERE, "ctorgrid")
    only = None
    a = sys.argv[1:]
    for i, x in enumerate(a):
        if x == "--jobs":
            jobs = int(a[i + 1])
        elif x == "--out":
            outdir = a[i + 1]
        elif x == "--only":
            only = a[i + 1]
    os.makedirs(outdir, exist_ok=True)

    cells = build_cells()
    if only:
        cells = {k: v for k, v in cells.items() if only in k}

    work = [(n, s, outdir) for n, s in sorted(cells.items())]
    results = {}
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        for name, secs, err in ex.map(run_cell, work):
            results[name] = (secs, err)

    ok = fail = 0
    for name, (secs, err) in sorted(results.items()):
        if err:
            print("FAIL %-22s %s" % (name, err.replace("\n", " ")))
            fail += 1
            continue
        ok += 1
        for sec, words in secs.items():
            if "AllocatePageBlock" in sec:
                continue
            print("\n== %-20s %s  (%d words)" % (name, sec, len(words)))
            for i, (w, dis) in enumerate(words):
                print("   %2d  %s  %s" % (i, w, dis))
    print("\ncells graded %d, failed %d, of %d" % (ok, fail, len(results)))


if __name__ == "__main__":
    main()
