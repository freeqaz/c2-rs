#!/usr/bin/env python3
"""labgrid.py — the compiler-label channel of `EncryptXTEA.cpp`, measured by
`docs/LABEL_COUNTER.md` §7.6's IN-THE-MIDDLE procedure and NEVER the
counterfactual form.

§7.2: the counterfactual form (two source texts, lead read off a control)
measures **Δseed + Δcharge**, because c1xx and c2 share one symbol-id space and
two different source texts have two different seeds.  `work/w-xtea/PRICE.md`
§6.1 — the standing "+1 for this TU" that board #2340 calls the binding
constraint — was taken in exactly that form, over TUs "one body apart".  This
file re-takes it in the form §7.6 mandates:

    int ga(int);
    <decls>
    int a0(int a){ return ga(a)+1; }      anchor
    <the probe P — a REAL EncryptXTEA member, verbatim>
    int a1(int a){ return ga(a)+2; }      anchor
    int a2(int a){ return ga(a)+3; }      anchor / control

    base      = first(a2) - first(a1)        measured IN THIS OBJ; must be 5
    stride(P) = first(a1) - first(a0) - base  == the slots P consumes
    extra(P)  = first(P)  - first(a0) - base  (framed probes only)

`stride(P)` is exactly what `coff::plan_labels` has to advance for P and what
`IlFunction::label_slots` has to return, so the grid answers the port's
question directly rather than a proxy for it.

`minted` is read on every row — §4's "read the minted column" box: a probe that
obliges a helper pair or an intrinsic external pays a MINTED surcharge as well
as a control-flow one.

Everything lands under work/w-xtea2/out/ (gitignored). std-lib only.

    work/w-xtea2/labgrid.py [probe ...]
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
from gt_dump import Obj  # noqa: E402
from gt_label_stride import groups, minted  # noqa: E402

WORK = os.path.join(ROOT, "work", "w-xtea2", "out")
CAPTURE = os.path.join(ROOT, "scripts", "gt_capture.sh")

ANCHOR_DECL = "int ga(int);"
ANCHORS = [
    "int a0(int a){ return ga(a)+1; }",
    "int a1(int a){ return ga(a)+2; }",
    "int a2(int a){ return ga(a)+3; }",
]

# The real header, verbatim from src/system/utl/EncryptXTEA.h, plus <cstring>
# for `SetKey`'s memcpy.  Nothing here is a paraphrase: the probe bodies below
# are the shipping definitions, so the strides are this TU's own and not a
# lookalike's.
# `<cstring>` is not on this probe's include path (the workload's `/I` list is
# not passed to `gt_capture.sh`), so `memcpy` is declared rather than included.
# `/Oi` keys on the NAME, so the intrinsic recognition is unchanged — checked by
# `x-setkey`'s obj, which carries the same `b memcpy` REL24 the real TU does.
XTEA_DECL = """extern "C" void *memcpy(void *, const void *, unsigned long);
struct XTEABlock { unsigned long long mData[2]; };
class XTEABlockEncrypter {
private:
    unsigned long long mNonce[2];
    unsigned int mKey[4];
    unsigned long long Encipher(unsigned long long, unsigned int *);
public:
    XTEABlockEncrypter();
    void SetKey(const unsigned char *);
    void SetNonce(const unsigned long long *, unsigned int);
    void Encrypt(const XTEABlock *, XTEABlock *);
};"""

CTOR = """XTEABlockEncrypter::XTEABlockEncrypter() {
    mNonce[0] = 0;
    mNonce[1] = 0;
}"""

SETKEY = ("void XTEABlockEncrypter::SetKey(const unsigned char *uc)"
          " { memcpy(mKey, uc, 0x10); }")

SETNONCE = """void XTEABlockEncrypter::SetNonce(const unsigned long long *nonce, unsigned int shift) {
    mNonce[0] = nonce[0] + shift;
    mNonce[1] = nonce[1] + shift;
}"""

ENCIPHER = """unsigned long long XTEABlockEncrypter::Encipher(unsigned long long nonce, unsigned int *key) {
    unsigned long v1 = nonce & 0xFFFFFFFF;
    unsigned long v2 = nonce >> 32;
    unsigned int sum = 0;
    for (int i = 0; i < 4; i++) {
        v1 += (v2 + (v2 << 4 ^ v2 >> 5)) ^ sum + key[sum & 3];
        sum += 0x9E3779B9;
        v2 += (v1 + (v1 << 4 ^ v1 >> 5)) ^ sum + key[(sum >> 11) & 3];
    }
    return (static_cast<unsigned long long>(v2) << 32)
         | (static_cast<unsigned long long>(v1) & 0xFFFFFFFF);
}"""

ENCRYPT = """void XTEABlockEncrypter::Encrypt(const XTEABlock *in, XTEABlock *out) {
    unsigned int *key = mKey;
    unsigned long offset = (char *)out - (char *)in;
    for (int i = 0; i < 2; i++) {
        *(unsigned long long *)(offset + (char *)in) =
            *(unsigned long long *)in ^ Encipher(mNonce[i], key);
        mNonce[i] += 1;
        in = (const XTEABlock *)((char *)in + 8);
    }
}"""

# (name, decls, probe source, symbol-substring of P or None, note)
PROBES = [
    # ---- controls: the instrument is re-proved on every run ---------------
    ("ctl-plain", "int gp(int);", "int P(int a){ return gp(a)+1; }", "?P@@",
     "Class A framed; stride must be 5 == base"),
    ("ctl-leaf", "", "int P(int a){ return a+1; }", None,
     "plain int leaf; stride must be 1"),
    ("ctl-leaf-for", "",
     "int P(int a){ int s=0; for(int i=0;i<a;i++) s+=i; return s; }", None,
     "a LEAF `for` — w-loop's Q1 row, re-taken in the middle"),
    # ---- the five real bodies, one probe each -----------------------------
    ("x-ctor", XTEA_DECL, CTOR, None, "??0XTEABlockEncrypter, 16 B leaf"),
    ("x-setkey", XTEA_DECL, SETKEY, None, "?SetKey, 12 B leaf, tail b memcpy"),
    ("x-setnonce", XTEA_DECL, SETNONCE, None, "?SetNonce, 32 B leaf"),
    ("x-encipher", XTEA_DECL, ENCIPHER, None, "?Encipher, 116 B LEAF ctr loop"),
    ("x-encrypt", XTEA_DECL + "\n" + ENCIPHER, ENCRYPT, "?Encrypt@",
     "?Encrypt, 96 B FRAMED loop + call (Encipher leads)"),
    # ---- separations --------------------------------------------------------
    ("x-encrypt-alone", XTEA_DECL, ENCRYPT, "?Encrypt@",
     "same, with Encipher UNDEFINED — separates the same-TU callee"),
]

# The workload's own profile first; `/Ox /GS- /c` second, because §7.4 measured
# construct-additivity holding at /O1 and FAILING at /Ox, and `label_slots` has
# no mode parameter.
MODES = ["/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc",
         "/nologo /wd4355 /wd4164 /c /GR /Ox /Oi /EHsc"]
MODE_TAGS = {MODES[0]: "workload-O1", MODES[1]: "workload-Ox"}


def build(decls, probe):
    parts = [ANCHOR_DECL]
    if decls:
        parts.append(decls)
    parts.append(ANCHORS[0])
    parts.append(probe)
    parts.append(ANCHORS[1])
    parts.append(ANCHORS[2])
    return "\n".join(parts) + "\n"


def capture(src, mode, tag):
    os.makedirs(WORK, exist_ok=True)
    cpp = os.path.join(WORK, "%s.cpp" % tag)
    open(cpp, "w").write(src)
    r = subprocess.run([CAPTURE, cpp] + mode.split(),
                       capture_output=True, text=True)
    path = r.stdout.strip()
    if not path or not os.path.exists(path):
        sys.stderr.write(r.stderr[-2000:])
        return None
    return Obj(open(path, "rb").read())


def nums(g):
    out = []
    for (k, n) in g["entries"]:
        if k == "label":
            d = "".join(c for c in n if c.isdigit())
            if d:
                out.append(int(d))
    return out


def firsts(o, psub):
    out = {}
    for g in groups(o):
        nm = g["name"]
        ns = nums(g)
        for sfx in ("a0", "a1", "a2"):
            if ("?%s@@" % sfx) in nm:
                out[sfx] = (min(ns) if ns else None, minted(g))
        if psub and psub in nm:
            out["P"] = (min(ns) if ns else None, minted(g))
    return out


def run(name, decls, probe, psub, mode):
    tag = "%s_%s" % (name, MODE_TAGS[mode])
    o = capture(build(decls, probe), mode, tag)
    if o is None:
        return None
    f = firsts(o, psub)
    if not all(k in f and f[k][0] is not None for k in ("a0", "a1", "a2")):
        return {"err": "anchors missing/label-free: %s" % sorted(f)}
    base = f["a2"][0] - f["a1"][0]
    stride = f["a1"][0] - f["a0"][0] - base
    p = f.get("P", (None, None))
    extra = (p[0] - f["a0"][0] - base) if p[0] is not None else None
    return {"base": base, "stride": stride, "extra": extra, "minted": p[1],
            "a0": f["a0"][0], "a1": f["a1"][0], "a2": f["a2"][0], "P": p[0]}


def main(argv):
    want = [a for a in argv if not a.startswith("-")]
    print("%-18s %-12s %5s %7s %6s %7s  %s" %
          ("probe", "mode", "base", "stride", "extra", "minted", "a0/P/a1/a2"))
    rc = 0
    for (name, decls, probe, psub, _note) in PROBES:
        if want and name not in want:
            continue
        for mode in MODES:
            r = run(name, decls, probe, psub, mode)
            if r is None:
                print("%-18s %-12s  CAPTURE FAILED" % (name, MODE_TAGS[mode]))
                rc = 1
                continue
            if "err" in r:
                print("%-18s %-12s  %s" % (name, MODE_TAGS[mode], r["err"]))
                rc = 1
                continue
            # §7.6 step 3: base is 5 under /Gy and 4 packed. The workload's
            # `/O1` obj is per-function (base 5) and its `/Ox` obj packs into
            # one `.text` (base 4) — measured, not assumed, and the row is void
            # if it is neither.
            flag = "" if r["base"] in (4, 5) else "   <== CONTROL BROKEN"
            if flag:
                rc = 1
            print("%-18s %-12s %5d %7d %6s %7s  %s/%s/%s/%s%s" %
                  (name, MODE_TAGS[mode], r["base"], r["stride"],
                   r["extra"] if r["extra"] is not None else "-",
                   r["minted"] if r["minted"] is not None else "-",
                   r["a0"], r["P"], r["a1"], r["a2"], flag))
    return rc


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
