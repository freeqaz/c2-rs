#!/usr/bin/env python3
"""rules.py — THE CANDIDATE ROOT RULES, as one closed enumeration.

Every rule is a predicate over an `.in` initializer owner `d` and the truth-free
features `rootmodel.feat` reads from the `.gl`/`.in` streams.  None of them may
touch the reference obj, `D`, `E`, or anything derived from them; `assert_free()`
is the mechanical check and `sweep.py` calls it.

THE QUESTION each rule answers: *what makes a defined file-scope data object a
root when nothing references it?*

  DEFINITION ALONE
    ALLW      being an `.in` owner at all.  This is w-mark's unfiltered reading
              and w-db's `rd_all`, reproduced as this lane's degenerate case.
  NAME CLASS  (the mangling encodes the C++ entity kind)
    M3        `?x@@3<type><cv>` -- a mangled file-scope VARIABLE
    M3A       ... and the cv-modifier is `A` (NON-const)
    M3B       ... and the cv-modifier is `B` (const)
    UNDEC     an undecorated name -- `extern "C"` data
    M3A_UNDEC M3A or UNDEC
    NOTRTTI   every owner except `??_7`/`??_R` (vftable / RTTI)
  RECORD STRUCTURE  (the `.gl` header fields `glowner` decodes)
    TAG<t>    the record tag byte
    SC<k>     the storage-class byte
    F20_<m>   a single `+0x20` flag bit
  CONJUNCTIONS  (only where a single channel left residue)
    M3A_F4000 M3A and the `+0x20 & 0x4000` bit

`PHASE7_PLAN.md` section 2 root clause (5) -- "kept data definitions: external-
linkage data and non-const internal data (internal *const* data is dropped when
unreferenced)" -- and `OBJ_DATA_BSS_SHAPE.md` P6 both predict `M3A`-shaped
behaviour and predict that `M3B` is where the drop happens.  That is the
hypothesis this enumeration is built to be able to REFUTE, not to confirm.

stdlib only.
"""

RTTI = "vftable / RTTI"
UNDECORATED = "undecorated (extern \"C\" / CRT)"
VAR3 = "other (3)"


def _tag(t):
    return lambda f: f["tag"] == "0x%02x" % t


def _sc(k):
    return lambda f: f["sc"] == "0x%02x" % k


def _f20(bit):
    key = None
    for b in range(20):
        if (1 << b) == bit:
            key = "f20b%02d(0x%x)" % (b, bit)
    return lambda f, _k=key: f.get(_k) == "1"


RULES = {
    "NOROOT":    lambda f: False,
    "ALLW":      lambda f: True,
    "M3":        lambda f: f["cls"] == VAR3,
    "M3A":       lambda f: f["cls"] == VAR3 and f["cv"] == "A",
    "M3B":       lambda f: f["cls"] == VAR3 and f["cv"] == "B",
    "UNDEC":     lambda f: f["cls"] == UNDECORATED,
    "M3A_UNDEC": lambda f: (f["cls"] == VAR3 and f["cv"] == "A") \
                            or f["cls"] == UNDECORATED,
    "NOTRTTI":   lambda f: f["cls"] != RTTI,
    "TAG01":     _tag(0x01),
    "TAG02":     _tag(0x02),
    "TAG04":     _tag(0x04),
    "TAG0E":     _tag(0x0E),
    "F20_400":   _f20(0x400),
    "F20_1000":  _f20(0x1000),
    "F20_2000":  _f20(0x2000),
    "F20_4000":  _f20(0x4000),
    # bits 17 and 18 are NOT in `w-db/joint.py`'s twelve-rule enumeration
    # (0x80, 0x400, 0x480, 0x1000, 0x2000, 0x4000, 0x60) -- the fit-side probe
    # is what surfaced them, at W/(W+I) 0.337 and 0.462 against 0.0009 overall
    "F20_20000":  _f20(0x20000),
    "F20_40000":  _f20(0x40000),
    "M3A_F4000": lambda f: (f["cls"] == VAR3 and f["cv"] == "A"
                            and f.get("f20b14(0x4000)") == "1"),
    "M3A_F40000": lambda f: (f["cls"] == VAR3 and f["cv"] == "A"
                             and f.get("f20b18(0x40000)") == "1"),
    "M3A_OR_F40000": lambda f: ((f["cls"] == VAR3 and f["cv"] == "A")
                                or f.get("f20b18(0x40000)") == "1"),
    # the INITIALIZER-PROPERTY candidate: a big initializer, regardless of what
    # the object is called
    "NPTR_GE17": lambda f: f["nptr"] in ("17-64", "65+"),
    "NPTR_GE5":  lambda f: f["nptr"] in ("5-16", "17-64", "65+"),
}

# rules whose feature keys are storage classes get added once the sweep has seen
# which `sc` bytes actually occur; `sweep.py --sc` prints them.
ORDER = ["NOROOT", "ALLW", "M3", "M3A", "M3B", "UNDEC", "M3A_UNDEC",
         "NOTRTTI", "TAG01", "TAG02", "TAG04", "TAG0E",
         "F20_400", "F20_1000", "F20_2000", "F20_4000",
         "F20_20000", "F20_40000",
         "M3A_F4000", "M3A_F40000", "M3A_OR_F40000",
         "NPTR_GE17", "NPTR_GE5"]

TRUTH_KEYS = ("D", "D_all", "D_data", "E", "emitted", "obj", "out.obj",
              "truth", "dtruth")


def assert_free(featkeys):
    """No rule may key off a feature whose name smells of the reference obj."""
    bad = [k for k in featkeys if k in TRUTH_KEYS]
    if bad:
        raise SystemExit("TRUTH LEAK in the feature set: %s" % bad)
    return True
