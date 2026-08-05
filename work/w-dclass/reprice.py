#!/usr/bin/env python3
"""reprice.py — the FRONTIER ranked by DISTINCT UNMODELED CONSTRUCTS.

Board **#198** ("the frontier is ranked by blocked-function count, and that key
is wrong" — found independently by w-front, w-pair and w-cfgimpl) asks for this
ranking and nobody had built it. `docs/rungs/2026-08-04-w-conv.md` §2 hand-counted
it over the then-17 frontier, but its dump `work/w-conv/frontier_dis.txt` was
**never committed**, so none of its numbers is re-derivable from the tree. This
script is the re-derivable version, and it is deliberately re-runnable so that
the next mechanism to land **invalidates it visibly instead of silently**.

    work/w-dclass/reprice.py --regen      # rebuild the 19 objs + censuses first
    work/w-dclass/reprice.py              # price and rank
    work/w-dclass/reprice.py --selftest   # pin the taxonomy, no toolchain needed
    work/w-dclass/reprice.py --tu mmio    # one TU, per-function detail

**The population is BLOCKED EMITTED FUNCTIONS**, said here once and repeated on
every table this prints. It is `emit-emitted − emit-in-class` per TU, the same
quantity `GapReport::factor_frontier` sorts on. There is a different, larger
population called "blocked functions" (over the whole `.ex`, most of whose bodies
c2 never emits) and two lanes have already been burned mixing them. On all
nineteen FRONTIER TUs the two happen to have the same *denominator* —
`fn_total == emit-emitted` on every one — but that is a fact about these TUs, not
about the corpus.

## What "unmodeled" means here, and why the price has two halves

Grounded in `crates/`, never in another lane's prose. A construct is:

* **HARD** — the port has no mechanism at all. No encoder outside `#[cfg(test)]`
  (`encode.rs`), or a shape `frame.rs`/`labels.rs` refuses **by name**.
* **SOFT** — the mechanism exists and is reachable, but no *production* in
  `select.rs`'s ordered dispatch puts this function's shape through it.

The split is the whole point of re-running this after W11 and `labels.rs`.
w-conv's two dearest mechanisms — *"a real label→offset map"* (14 of 17 TUs) and
*"the intra-section unconditional `b`"* (10 of 17) — were **HARD** when it
counted and are **not refusals at all** now for a forward-only layout:
`labels.rs` mints, references, defines and resolves, and `calls.rs` drives it
with `Form::B` to a named epilogue. A **backward** reference is still refused by
name and still counts HARD, and `labels.rs` says why in its header (the
compiler-label counter charges ≥ +1 in 11 of 11 measured cells, at four distinct
magnitudes with no rule that survives them).

There is a third axis, and it is the one the disassembly cannot show: **IL** —
one token per distinct census blocker key, i.e. per parse production that does
not exist (`crates/c2-il/src/func/bundle.rs:699`). A refusal at parse time never
reaches an instruction, so a byte-level count alone misses it entirely.

    price = |IL| + |HARD| + |SOFT|

distinct tokens, unioned over the TU's blocked emitted functions.

**It is a LOWER BOUND, and it is low in a known direction**: it counts what the
bytes and the blocker key show, and it cannot see a *selection*, *allocation* or
*scheduling* decision. `?GetXAllocAttributes` is the calibrating case — w-cfgimpl
priced it at four independent facts with a 10-cell grid and all four are one
mnemonic here. `xboxheap.cpp` is the extreme: every one of its instructions is in
vocabulary, it has no branch and no frame surprise, and it still diverges at
**instruction 0 on order**. Its price of 2 is a floor meaning *"one blocker key
plus one uncharacterized scheduler"*, and it must be read as w-conv read it —
**unpriceable, not cheapest**.

## Collapse rule

The project rule is *if one quantity governs several boundaries, that is one
refusal*. Applied explicitly, and each collapse is named in `COLLAPSE` below so
a reader can undo it: a CTR loop implies `mtctr`+`bdnz`, an indirect call implies
`mtctr`+`bctrl`, and every cr0 producer (record form or a `cmp` against cr0) is
one refusal because one constant — `CR_COMPARE = 6`, hardwired at all nine
emission sites — governs them all.

Outside the std-only Rust workspace on purpose, same status as
`scripts/gt_dump.py`. **Read-only with respect to `crates/`.**
"""

import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from parse import FRONTIER, REPO, pair, read_dump  # noqa: E402

DIS = os.path.join(REPO, "work/w-dclass/dis")
CEN = os.path.join(REPO, "work/w-dclass/census")
REF = os.path.join(REPO, "work/w-dclass/ref")

# ---------------------------------------------------------------------------
# THE PORT'S CAPABILITY TABLE — every row cites where it lives in `crates/`
# ---------------------------------------------------------------------------

# Mnemonics `llvm-mc` prints that the port can emit today. Derived from the 52
# `encode_*` functions in `crates/c2-core/src/codegen/` that have a call site
# OUTSIDE a `mod tests` block — `encode_extsh` is defined and has none, so `extsh`
# is NOT here (leaf/load.rs says why: the signed-halfword widening is the one row
# whose instruction count depends on the optimization mode, and the parser refuses
# it). Extended mnemonics are listed under the encoder that produces them.
PORT_MNEM = {
    # encode_addi / encode_addis, rA=0 forms included
    "addi", "li", "addis", "lis",
    "add", "addic", "addze", "adde",
    "subf", "sub", "subfc", "subfe", "subfic", "subfze",
    "mullw", "neg", "cntlzw",
    "andc", "orc", "eqv", "ori", "xori", "nop",
    # encode_rlwinm — the general form, so every rlwinm extended mnemonic
    "rlwinm", "slwi", "srwi", "clrlwi", "rotlwi", "clrrwi", "extlwi",
    "inslwi", "clrlslwi", "rotlwmi",
    "srawi",
    "lbz", "lhz", "lwz", "ld", "lfs", "lfd",
    "stb", "sth", "stw", "std", "stfs", "stfd", "stwu",
    "mr", "extsb",
    "cmpwi", "cmplwi",
    # frame.rs's hardwired words
    "mflr", "mtlr",
    # branches: encode_blr / encode_bc / encode_b_intra / encode_call_branch /
    # encode_tail_branch. `bt`/`bf` are how llvm-mc prints a `bc` with BO 12/4.
    "blr", "b", "bl", "bt", "bf",
    # float leaves
    "fadds", "fadd", "fsubs", "fsub", "fmuls", "fmul", "fdivs", "fdiv",
    "fmr", "frsp",
}

# Why each absent mnemonic family is absent, for the report. Anything not named
# here still prices — the map is documentation, not a filter.
WHY = {
    "mtctr": "no encoder; CTR is not in the port's register model",
    "bdnz": "no encoder; the CTR loop is absent from the port AND from docs/",
    "bctrl": "no encoder; an indirect call has no production",
    "bcctrl": "no encoder; an indirect call has no production",
    "bclr": "no encoder for a CONDITIONAL return; only the fixed 0x4E800020",
    "cmpw": "no encoder — encode_cmpwi takes an i16 immediate, not a second GPR",
    "cmplw": "no encoder — encode_cmplwi takes a u16 immediate, not a second GPR",
    "rlwimi": "no encoder; board #199 — the mask is derived from the CONSTANT, unmeasured",
    "mulli": "no encoder", "divw": "no encoder", "divwu": "no encoder",
    "xor": "no encoder (encode_eqv is its complement, not it)",
    "and": "no encoder", "or": "no encoder (encode_mr is the rA==rB case only)",
    "nand": "no encoder", "nor": "no encoder",
    "twi": "no encoder; the divide-by-zero trap has no model",
    "tw": "no encoder",
    "extsh": "encode_extsh EXISTS but has no call site outside tests; leaf/load.rs refuses the shape",
    "lbzx": "no indexed-load encoder", "lhzx": "no indexed-load encoder",
    "lwzx": "no indexed-load encoder", "lfsx": "no indexed-load encoder",
    "stbx": "no indexed-store encoder", "sthx": "no indexed-store encoder",
    "stwx": "no indexed-store encoder", "stdx": "no indexed-store encoder",
    "stfsx": "no indexed-store encoder",
    "lbzu": "no update-form encoder", "lwzu": "no update-form encoder",
    "stbu": "no update-form encoder", "sthu": "no update-form encoder",
    "stwu": "", "stfsu": "no update-form encoder", "stdu": "no 64-bit frame allocation",
    "rldicl": "no 64-bit rotate encoder", "rldimi": "no 64-bit rotate encoder",
    "rldicr": "no 64-bit rotate encoder", "clrldi": "no 64-bit rotate encoder",
    "sldi": "no 64-bit rotate encoder", "srdi": "no 64-bit rotate encoder",
    "slw": "no variable-shift encoder", "srw": "no variable-shift encoder",
    "sraw": "no variable-shift encoder",
    "fnmsubs": "no fused-multiply encoder", "fmadds": "no fused-multiply encoder",
    "fneg": "no encoder", "fabs": "no encoder", "fcmpu": "no encoder",
    "mfcr": "no encoder", "mtcrf": "no encoder", "mfspr": "no encoder",
}

# Record forms: `mnem.` sets CR0. Collapsed into the single `cr0` token — see
# COLLAPSE. Every emission site in the port passes `CR_COMPARE` (= 6):
# cond_tail.rs:148/153/165 and calls.rs:317/321/324/387/392/418/423/433/435.
COLLAPSE = {
    "ctr-loop": ["mtctr", "bdnz"],
    "indirect-call": ["mtctr", "bctrl", "bcctrl"],
    "cr0": [],   # record forms and cr0 compares, handled in the extractor
    "cmp-reg-reg": ["cmpw", "cmplw", "cmpd", "cmpld"],
    "conditional-return": ["bclr", "bltlr", "bgtlr", "beqlr", "bnelr", "blelr",
                           "bgelr", "bnslr", "bsolr"],
}

# ---------------------------------------------------------------------------
# FEATURE EXTRACTION over one blocked emitted function's `.text` COMDAT
# ---------------------------------------------------------------------------

BC_MNEMS = {"bt", "bf", "bdnz", "bdz", "bdnzt", "bdnzf"}
COND_RET = {"bclr", "bltlr", "bgtlr", "beqlr", "bnelr", "blelr", "bgelr", "bnslr", "bsolr"}


def _target(ins):
    """Resolved intra-section byte target of a branch, or None if it is a
    relocated (external) branch or not a branch at all."""
    if any(n.startswith("REL24") for n in ins.notes):
        return None
    t = ins.ops.strip()
    # llvm-mc prints `.+88` / `.-24` for a self-relative displacement.
    if t.startswith(".+") or t.startswith(".-"):
        return ins.off + int(t[1:], 0)
    if "," in t:
        last = t.rsplit(",", 1)[1].strip()
        if last.startswith(".+") or last.startswith(".-"):
            return ins.off + int(last[1:], 0)
    return None


def features(c):
    """Every construct-relevant fact about one `.text` COMDAT, from its bytes."""
    f = {
        "mnems": set(), "record": set(), "crfields": set(),
        "regreg_cmp": False, "ctr": False, "indirect": False, "condret": False,
        "backedge": False, "b_to_nonepilogue": False, "shared_target": False,
        "multi_exit": False, "refhi": False, "reflo_on_load": False,
        "addr32_text": False, "savegprlr": False, "savefpr": False,
        "framed": False, "frame_size": 0, "saved_gpr_open": 0,
        "stack_local": False, "frame_pointer": False, "stdu": False,
        "calls": [], "call_result_used": False, "targets": {}, "ncond": 0,
    }
    ins = c.insns
    if not ins:
        return f
    # The epilogue starts at the first `addi 1,1,+F` (Class A teardown) or, for a
    # `__restgprlr` tail, at the `b __restgprlr_N`.
    epi = None
    for i in ins:
        if i.mnem == "addi" and i.ops.replace(" ", "").startswith("1,1,") \
           and not i.ops.replace(" ", "").startswith("1,1,-"):
            epi = i.off
            break
    for i in ins:
        m = i.mnem
        if m.endswith(".") and m != ".":
            f["record"].add(m)
            m = m[:-1]
        f["mnems"].add(m)
        ops = i.ops.replace("\t", " ")
        if m in ("cmpw", "cmplw", "cmpd", "cmpld"):
            f["regreg_cmp"] = True
        if m in ("cmpwi", "cmplwi", "cmpw", "cmplw", "cmpdi", "cmpldi", "cmpd", "cmpld"):
            # **`llvm-mc` drops the CR field when it is cr0.** `cmplwi 6, 3, 0` is
            # three operands against cr6; `cmplwi 3, 0` is TWO operands against
            # **cr0** — and reading `ops[0]` on the short form gives `3`, i.e. a
            # cr field that is not cr0 and not cr6, which silently loses the one
            # fact this row exists to find. Six sites across the nineteen are the
            # short form (`mmioClose` ×2, `undname` ×2, `vsnprnc`/`vswprnc`).
            parts = [p.strip() for p in ops.split(",")]
            f["crfields"].add(int(parts[0]) if len(parts) >= 3 and parts[0].isdigit() else 0)
        if m in ("bt", "bf"):
            f["ncond"] += 1
            bi = ops.split(",")[0].strip()
            if bi.isdigit():
                f["crfields"].add(int(bi) // 4)
        if m == "mtctr":
            f["ctr"] = True
        if m in ("bctrl", "bcctrl", "bctr", "bcctr"):
            f["indirect"] = True
        if m in COND_RET:
            f["condret"] = True
        if m == "stwu" and ops.replace(" ", "").startswith("1,-"):
            # llvm-mc prints `stwu 1, -96(1)`.
            f["framed"] = True
            o = ops.replace(" ", "")
            f["frame_size"] = -int(o[o.index(",") + 1:o.index("(")], 0)
        if m == "stdu" and ops.replace(" ", "").startswith("1,"):
            f["stdu"] = True
            f["framed"] = True
        if m == "std" and ops.replace(" ", "").endswith("(1)") and "-" in ops:
            f["saved_gpr_open"] += 1
        if m in ("addi", "mr") and ops.replace(" ", "").startswith("31,1"):
            f["frame_pointer"] = True
        # A stack LOCAL: a load/store based on r1 at a NON-negative displacement
        # (the LR slot and the register saves are negative; the outgoing-parameter
        # home area and the locals are positive).
        if m in ("lwz", "lbz", "lhz", "ld", "lfs", "lfd",
                 "stw", "stb", "sth", "std", "stfs", "stfd"):
            o = ops.replace(" ", "")
            if o.endswith("(1)"):
                d = o[o.rindex(",") + 1:-3]
                try:
                    if int(d, 0) >= 0:
                        f["stack_local"] = True
                except ValueError:
                    pass
        for n in i.notes:
            if n.startswith("REL24"):
                nm = n.split("]", 1)[1].strip() if "]" in n else n
                f["calls"].append(nm)
                if nm.startswith("__savegprlr") or nm.startswith("__restgprlr"):
                    f["savegprlr"] = True
                if nm.startswith("__savefpr") or nm.startswith("__restfpr"):
                    f["savefpr"] = True
            if n.startswith("REFHI"):
                f["refhi"] = True
            if n.startswith("REFLO") and m in ("lwz", "lbz", "lhz", "ld", "lfs", "lfd"):
                f["reflo_on_load"] = True
            if n.startswith("ADDR32"):
                f["addr32_text"] = True
        t = _target(i)
        if t is not None and m in BC_MNEMS | {"b"}:
            f["targets"].setdefault(t, []).append((i.off, m))
            if t <= i.off:
                f["backedge"] = True
            if m == "b" and epi is not None and t != epi:
                f["b_to_nonepilogue"] = True
            if m == "b" and epi is None:
                f["b_to_nonepilogue"] = True
    f["shared_target"] = any(len(v) > 1 for v in f["targets"].values())
    f["multi_exit"] = sum(1 for i in ins if i.mnem == "blr") > 1
    # A value with a HOME ACROSS A TRANSFER: a register written at X and read at
    # Y > X with a branch target or a `bl` strictly between them. The port models
    # exactly one instance of this — `plan_cond_pair`'s park, at **r11 and only
    # r11** (`cond_tail.rs`; `docs/CODEGEN_W6_COMPARE.md` §6 calls the descent to
    # r10 "demonstrably richer than a descending counter and not characterized").
    # This is the one selection-level fact that IS visible in the bytes, and it is
    # w-conv's `negate_test` rows 3 and 4 (`mr r10,r3`, the park that is not r11;
    # `li r11,0`, a local live across every block).
    barriers = sorted(set(f["targets"].keys())
                      | {i.off for i in ins if i.mnem in ("bl", "bt", "bf", "b")})
    wrote = {}
    for i in ins:
        ops = [p.strip() for p in i.ops.replace("\t", " ").split(",") if p.strip()]
        if not ops:
            continue
        for p in ops[1:]:
            r = p[p.index("(") + 1:p.index(")")] if "(" in p and ")" in p else p
            if r.isdigit() and int(r) in wrote:
                x = wrote[int(r)]
                if any(x < b < i.off for b in barriers):
                    f["cross_block_value"] = True
        if i.mnem in ("mr", "li", "lis", "addi", "add", "lwz", "lbz", "lhz", "ld"):
            d = ops[0]
            if d.isdigit():
                wrote[int(d)] = i.off
    f.setdefault("cross_block_value", False)
    # A call whose result is consumed: any instruction reading r3 after a `bl`
    # that is not the function's own `blr`-adjacent return.
    seen_bl = False
    for i in ins:
        if i.mnem == "bl":
            seen_bl = True
            continue
        if seen_bl and i.mnem in ("mr", "stw", "stb", "sth", "cmpwi", "cmplwi",
                                  "cmpw", "cmplw", "add", "addi"):
            src = [p.strip() for p in i.ops.replace("\t", " ").split(",")]
            if any(p == "3" for p in src[1:]) or "(3)" in i.ops.replace(" ", ""):
                f["call_result_used"] = True
    return f


# ---------------------------------------------------------------------------
# TOKEN ASSIGNMENT
# ---------------------------------------------------------------------------

def tokens(f, cen_row, obj):
    """-> (hard:set, soft:set). Each token is one distinct unmodeled construct.

    **This function is byte-level only, and byte-level pricing is systematically
    LOW.** It sees an instruction the port cannot encode and a block structure it
    cannot lay out; it cannot see a *selection* or *allocation* decision, because
    those are invisible in the mnemonic stream. `?GetXAllocAttributes` is the
    calibrating case: w-cfgimpl's 10-cell grid (rung §4.1) prices it at **four**
    independent facts — the bool spine differs by relation, the constant
    materializes three different ways, the `rlwimi` mask is derived from the
    constant and not the shift, and the destination register depends on operand
    overlap — plus the `lis`'s schedule slot. All five are one mnemonic here
    (`insn:rlwimi`). The parse-production axis in `il_tokens` is what covers the
    other side of that; neither axis alone is the price.
    """
    hard, soft = set(), set()

    suppressed = set()
    if f["ctr"] and any(m == "bdnz" for m in f["mnems"]):
        hard.add("ctr-loop")
        suppressed |= set(COLLAPSE["ctr-loop"])
    if f["indirect"]:
        hard.add("indirect-call")
        suppressed |= set(COLLAPSE["indirect-call"])
    if f["condret"]:
        # `encode_blr` is the fixed word 0x4E800020 and there is no conditional
        # form. One missing encoder governs every `b<cond>lr` spelling.
        suppressed |= set(COLLAPSE["conditional-return"])
    if f["regreg_cmp"]:
        # One missing encoder shape governs `cmpw` and `cmplw` alike — the two
        # `encode_cmp*i` take an immediate, not a second GPR. One refusal.
        suppressed |= set(COLLAPSE["cmp-reg-reg"])

    # 1. instruction vocabulary
    for m in sorted(f["mnems"]):
        if m in suppressed or m in PORT_MNEM:
            continue
        if m.startswith("<"):
            hard.add("insn:undecodable-word")
            continue
        hard.add("insn:" + m)

    # 2. CR — one refusal, because one constant (CR_COMPARE = 6) governs all of it
    if f["record"] or any(cf != 6 for cf in f["crfields"]):
        hard.add("cr0")
    if f["regreg_cmp"]:
        hard.add("cmp-reg-reg")

    # 3. frame
    if f["savegprlr"]:
        hard.add("frame-savegprlr")        # frame.rs:223, refused by name
    if f["savefpr"]:
        hard.add("frame-savefpr")          # frame.rs:226, refused by name
    if f["frame_size"] >= 5 * 4096:
        hard.add("frame-rtlcheckstack12")  # frame.rs:229, refused by name
    if f["stdu"]:
        hard.add("frame-stdu")
    if f["frame_pointer"]:
        hard.add("frame-pointer")
    if f["stack_local"]:
        # FrameLayout::locals exists and sizes the frame, but nothing outside
        # `mod tests` ever sets it non-zero, and no production reads or writes a
        # stack slot. Mechanism-shaped, production-absent -> SOFT.
        soft.add("frame-locals")
    if f["saved_gpr_open"] > 2:
        hard.add("frame-saved-gprs>2")

    # 4. control flow / layout
    if f["backedge"]:
        hard.add("backward-branch")        # labels.rs invariant 4, refused by name
    if f["condret"]:
        hard.add("conditional-return")
    if f["b_to_nonepilogue"]:
        # `labels.rs` can name a second target and `Form::B` exists; what is
        # missing is a production that lays out a join block. SOFT.
        soft.add("join-block")
    if f["multi_exit"]:
        soft.add("multiple-blr-exits")
    if f["shared_target"]:
        # W11 + W-SMALL already emit >= 2 references to one label, so this is no
        # longer a refusal on its own. Recorded, never charged. (w-conv charged
        # it on 14 of 17.)
        pass

    # 5. data addressing
    if f["reflo_on_load"]:
        # The port emits REFHI/REFLO for an FP pool constant and for WR1's
        # `lis`+`addi` ADDRESS of a data symbol; reading a data symbol's VALUE
        # has no production (straightline.rs:290 — "a named data symbol's
        # address only ever appears as a whole").
        soft.add("data-symbol-load")
    elif f["refhi"]:
        soft.add("data-symbol-address")

    # 6. calls
    if f["call_result_used"]:
        soft.add("call-result-captured")   # CallSeq discards every result it makes
    if f.get("cross_block_value"):
        # The park exists but at r11 and only r11, for one shape.
        soft.add("value-live-across-a-transfer")
    if f["addr32_text"]:
        hard.add("addr32-in-text")

    # 7. EH and body decode, off the census axes rather than the bytes.
    #
    # `eh-unknown` and a `cf-expr-*` control-flow class are ONE quantity — the
    # body decoder stopped at a byte, so neither axis could be read. Charging two
    # would double-count one refusal (`Biquad::?SetCoefficients`, 838 B, is the
    # only instance across the nineteen).
    if cen_row["cflow"].startswith("cf-expr-") or cen_row["eh"] == "eh-unknown":
        hard.add("il-body-undecoded")
    elif cen_row["eh"] not in ("eh-none", "eh-bare"):
        hard.add("eh:" + cen_row["eh"])

    return hard, soft


def il_tokens(cen_row):
    """The PARSE-PRODUCTION axis: one token per distinct census blocker key.

    The accept/refuse boundary lives in two places and this is the first of them
    — `IlBundle::functions()` (`crates/c2-il/src/func/bundle.rs:699`), whose
    refusal key names the IL construct that has no production. A key is a
    distinct unmodeled construct in exactly the sense board #198 asks for: it is
    one thing somebody has to build, and it is a thing the disassembly cannot
    show, because a refusal at parse time never reaches an instruction.

    w-conv counted this axis too and kept it separate from layout — its
    `negate_test` row #9 is *"`cflow-if-n` inside a framed body **as a parse
    production** … the recognizer, in a different crate from #1's layout. Both
    must exist; neither implies the other."* This follows that cut.
    """
    return {"il:" + cen_row["key"]}


# TU-level obligations: a section the obj carries that the port's writer has no
# production for on this shape. `.rdata` is emitted for a pooled FP constant, so
# it only counts when the TU has no float leaf reason for it.
BASE_SECS = {".drectve", ".debug$S", ".XBLD$W", ".text", ".pdata"}


def tu_sections(obj, has_float):
    out = set()
    for n in obj.raw_sec_names:
        if n in BASE_SECS:
            continue
        if n == ".rdata" and has_float:
            continue
        out.add("sect:" + n)
    return out


# ---------------------------------------------------------------------------

def price_all(verbose_tu=None):
    rows = []
    for src in FRONTIER:
        b = os.path.basename(src)[:-4]
        obj, pairs = pair(b, DIS, CEN)
        hard, soft, il, per_fn = set(), set(), set(), []
        has_float = False
        for r, c in pairs:
            if not r["blocked"]:
                continue
            f = features(c)
            if f["mnems"] & {"fadds", "fmuls", "fdivs", "fsubs", "lfs", "stfs"}:
                has_float = True
            h, s = tokens(f, r, obj)
            i = il_tokens(r)
            per_fn.append((r, c, f, h, s, i))
            hard |= h
            soft |= s
            il |= i
        secs = tu_sections(obj, has_float)
        hard |= secs
        rows.append({
            "src": src, "base": b, "blocked": len(per_fn),
            "emitted": len(pairs), "hard": hard, "soft": soft, "il": il,
            "price": len(hard) + len(soft) + len(il), "fns": per_fn,
            "sects": secs,
        })
        if verbose_tu and verbose_tu == b:
            print("== %s — %d blocked EMITTED of %d emitted" % (src, len(per_fn), len(pairs)))
            for r, c, f, h, s, i in per_fn:
                print("  %-52s %-26s %s" % (c.name[:52], r["key"], r["cflow"]))
                print("     IL   %s" % " ".join(sorted(i)))
                print("     HARD %s" % (" ".join(sorted(h)) or "-"))
                print("     SOFT %s" % (" ".join(sorted(s)) or "-"))
            print("  TU sections charged: %s" % (" ".join(sorted(secs)) or "-"))
            print("  TU price = %d IL + %d HARD + %d SOFT = %d"
                  % (len(il), len(hard), len(soft), len(il) + len(hard) + len(soft)))
    return rows


# w-conv's hand count, `docs/rungs/2026-08-04-w-conv.md` §2 / `work/w-conv/PREREG.md`
# §1, transcribed for COMPARISON ONLY — never used in a price. Its dump
# (`work/w-conv/frontier_dis.txt`) was never committed, so these are the only
# surviving form of that measurement. Its counts are lower bounds: it says
# outright that it "stopped counting each row at the point the decline clause had
# already fired", which is why the dear end is where this script is *higher*.
WCONV = {
    "xboxmem": 6, "Main": 6, "IPP_basicmath_xbox": 6, "Biquad": 7, "xlrcimpl": 7,
    "mmio": 7, "Sort": 7, "Pool": 7, "undname": 8, "vsnprnc": 8, "vswprnc": 8,
    "osfinfo": 8, "negate_test": 9, "jsonwriter": 10, "wordwrap": 12,
    "EncryptXTEA": 12, "xboxheap": None,   # "unpriceable"
}

# The two mechanisms w-conv's §3 ranked first and called absent, both of which
# landed AFTER it (`labels.rs`, and `calls.rs`'s `Form::B` arm). A TU w-conv
# charged for either is over-priced by this much — but only where its block
# structure reaches the ONE production that drives the map.
WCONV_MECHANISM_TUS = {
    "label-map": 14,          # of 17
    "intra-section-b": 10,    # of 17
}


def compare():
    rows = {r["base"]: r for r in price_all()}
    print("w-conv (2026-08-04, 17 TUs, hand-counted, dump never committed)")
    print("  vs this script (2026-08-05, 19 TUs, re-derivable)")
    print("population both sides: BLOCKED EMITTED functions")
    print()
    print("%-22s %8s %6s %7s  %s" % ("TU", "w-conv", "here", "delta", "note"))
    both = 0
    for b, w in sorted(WCONV.items(), key=lambda kv: (kv[1] is None, kv[1] or 0, kv[0])):
        r = rows[b]
        if w is None:
            print("%-22s %8s %6d %7s  %s" % (b, "unprice", r["price"], "-",
                  "every insn in vocabulary; the refusal is a SCHEDULE, invisible here"))
            continue
        both += 1
        print("%-22s %8d %6d %+7d  %s" % (b, w, r["price"], r["price"] - w, ""))
    new = [b for b in rows if b not in WCONV]
    print()
    print("NOT in w-conv (the frontier was 17, it is 19):")
    for b in sorted(new):
        print("  %-20s price %d   %s" % (b, rows[b]["price"], rows[b]["src"]))
    print()
    print("COUNTS: %d TUs compared, %d new since w-conv, %d TUs total"
          % (both, len(new), len(rows)))
    if both == 0 or len(rows) != 19:
        print("FAIL: comparison is empty or the frontier is not 19")
        return 1
    return 0


def claims():
    """The two prior-art claims this lane was asked to check, mechanically."""
    rows = {r["base"]: r for r in price_all()}
    bad = 0
    print("== CLAIM 1 — w-cfgimpl rung §6 item 2: 'All five single-blocked-function")
    print("   frontier TUs were disassembled (osfinfo, undname, vswprnc, xlrcimpl,")
    print("   negate_test) and every one of them is FRAMED, with data-symbol")
    print("   REFHI/REFLO pairs, cr0 record-form branches, stack locals,")
    print("   srawi/mulli/lwzx, or __savegprlr_26.'")
    print()
    hdr = ("TU", "framed", "pdata", "REFHI", "cr0", "rec", "local", "svgpr", "sr/ml/lx")
    print("%-14s %-7s %-6s %-6s %-6s %-6s %-6s %-6s %s" % hdr)
    framed_all = True
    disj = {}
    for b in ["osfinfo", "undname", "vswprnc", "xlrcimpl", "negate_test"]:
        obj, pairs = pair(b, DIS, CEN)
        for r, c in pairs:
            if not r["blocked"]:
                continue
            f = features(c)
            pd = c.name in obj.pdata_targets
            ar = bool(f["mnems"] & {"srawi", "mulli", "lwzx", "lbzx", "lhzx"})
            framed_all &= bool(f["framed"] and pd)
            d = disj.setdefault(b, False)
            disj[b] = d or f["refhi"] or (0 in f["crfields"]) or bool(f["record"]) \
                or f["stack_local"] or f["savegprlr"] or ar
            print("%-14s %-7s %-6s %-6s %-6s %-6s %-6s %-6s %s"
                  % (b, f["framed"], pd, f["refhi"], 0 in f["crfields"],
                     bool(f["record"]), f["stack_local"], f["savegprlr"], ar))
    print()
    print("  'every one is FRAMED': %s — %d of 5" % (framed_all, sum(
        1 for b in disj)))
    hits = [b for b, v in disj.items() if v]
    print("  'with <one of the five features>': holds on %d of 5 — %s"
          % (len(hits), ", ".join(sorted(hits))))
    miss = sorted(b for b, v in disj.items() if not v)
    if miss:
        bad += 1
        print("  REFUTED on: %s — none of the five named features is present."
              % ", ".join(miss))
    print()
    print("== CLAIM 2 — docs/CFG_SHAPE.md §0 item 1: an `if` in the IL usually does")
    print("   NOT become a branch; six of seven leaf probes fold to branchless")
    print("   arithmetic or `bclr`. Checked over the 52 blocked EMITTED functions:")
    print()
    over = under = exact = 0
    for src in FRONTIER:
        b = os.path.basename(src)[:-4]
        obj, pairs = pair(b, DIS, CEN)
        for r, c in pairs:
            if not r["blocked"]:
                continue
            nbc = sum(1 for i in c.insns if i.mnem in ("bt", "bf"))
            nb = sum(1 for i in c.insns if i.mnem == "b"
                     and not any(n.startswith("REL24") for n in i.notes))
            ndz = sum(1 for i in c.insns if i.mnem in ("bdnz", "bdz"))
            nlr = sum(1 for i in c.insns if i.mnem in COND_RET)
            v = None
            if r["cflow"].startswith("cflow-if") and nbc == 0 and nb == 0:
                v = "OVERSTATES — folded to %s" % ("bclr, band 2" if nlr else "branchless, band 1")
                over += 1
            elif r["cflow"] == "cflow-loop" and ndz == 0 and nbc == 0 and nb == 0:
                v = "OVERSTATES — the loop is gone"
                over += 1
            elif r["cflow"] == "cflow-straight" and (nbc + nb + ndz + nlr) > 0:
                v = "UNDERSTATES — %d real transfers in a 'straight' body" % (nbc + nb + ndz + nlr)
                under += 1
            if v:
                print("  %-20s %-38s %-14s %s" % (b, c.name[:38], r["cflow"], v))
            else:
                exact += 1
    print()
    print("  COUNTS: %d overstate, %d understate, %d neither, of %d blocked emitted"
          % (over, under, exact, over + under + exact))
    if over + under + exact != 52:
        print("  FAIL: expected 52 blocked emitted functions")
        return 1
    return 1 if bad and "--strict" in sys.argv else 0


def main(argv):
    if "--selftest" in argv:
        return selftest()
    if "--compare" in argv:
        return compare()
    if "--claims" in argv:
        return claims()
    if "--regen" in argv:
        regen()
    tu = None
    if "--tu" in argv:
        tu = argv[argv.index("--tu") + 1]
    rows = price_all(tu)
    if tu:
        return 0
    rows.sort(key=lambda r: (r["price"], r["blocked"], r["src"]))
    print("FRONTIER repriced by DISTINCT UNMODELED CONSTRUCTS")
    print("population: BLOCKED EMITTED functions (emit-emitted - emit-in-class)")
    print()
    print("  price = |IL| + |HARD| + |SOFT|, distinct tokens over the TU's blocked")
    print("  emitted functions.  IL   = parse productions absent (census blocker keys,")
    print("                             c2-il/src/func/bundle.rs:699)")
    print("                      HARD = no mechanism at all (no encoder outside tests,")
    print("                             or refused BY NAME in frame.rs / labels.rs)")
    print("                      SOFT = mechanism exists and is reachable, but no")
    print("                             production in select.rs puts this shape through it")
    print()
    print("%5s %4s %4s %4s %5s  %s" % ("price", "IL", "HARD", "SOFT", "blkd", "src"))
    for r in rows:
        print("%5d %4d %4d %4d %5d  %s" % (r["price"], len(r["il"]), len(r["hard"]),
                                           len(r["soft"]), r["blocked"], r["src"]))
    print()
    # POSITIVE CHECKS WITH PRINTED COUNTS. The project's most-repeated defect is
    # absence read as success; a table that prices nothing must not exit 0.
    n = len(rows)
    priced = sum(1 for r in rows if r["price"] > 0)
    fns = sum(r["blocked"] for r in rows)
    toks = len(set().union(*[r["hard"] | r["soft"] | r["il"] for r in rows])) if rows else 0
    print("COUNTS: %d TUs, %d blocked emitted functions, %d priced TUs, "
          "%d distinct construct tokens" % (n, fns, priced, toks))
    print("        min price %d (%s), max %d (%s)" % (
        rows[0]["price"], rows[0]["base"], rows[-1]["price"], rows[-1]["base"]))
    fires = sum(1 for r in rows if r["price"] >= 4)
    print("        board #269 decline clause (>= 4 independent refusals): "
          "fires on %d of %d" % (fires, n))
    if n == 0 or fns == 0 or priced == 0:
        print("FAIL: priced nothing — refusing to exit 0 on an empty measurement")
        return 1
    if n != 19:
        print("FAIL: expected 19 FRONTIER TUs, got %d — regenerate the frontier" % n)
        return 1
    return 0


def regen():
    env = dict(os.environ)
    env.setdefault("C2RS_COMPILERS", os.path.join(REPO, "compilers"))
    for d in (DIS, CEN, REF):
        os.makedirs(d, exist_ok=True)
    ok = 0
    for src in FRONTIER:
        b = os.path.basename(src)[:-4]
        o = os.path.join(REF, b + ".obj")
        r = subprocess.run([os.path.join(REPO, "work/w-frame/refobj.sh"), src, o],
                           cwd=REPO, env=env, capture_output=True, text=True)
        if r.returncode == 3:
            print("SKIP: toolchain absent — %s" % r.stdout.strip())
            sys.exit(3)
        if r.returncode != 0:
            print("FAIL refobj %s: %s%s" % (src, r.stdout, r.stderr))
            sys.exit(1)
        with open(os.path.join(DIS, b + ".txt"), "w") as fh:
            subprocess.run([sys.executable, os.path.join(REPO, "scripts/gt_dump.py"), o],
                           cwd=REPO, stdout=fh, stderr=subprocess.STDOUT, check=True)
        with open(os.path.join(CEN, b + ".txt"), "w") as fh:
            subprocess.run([os.path.join(REPO, "target/release/c2rs"), "census",
                            os.path.join(REPO, "../dc3-decomp", src),
                            "--flags-file", os.path.join(REPO, "work/dc3-workload/flags.txt"),
                            "--cwd", os.path.join(REPO, "../dc3-decomp")],
                           cwd=REPO, env=env, stdout=fh, stderr=subprocess.STDOUT, check=True)
        ok += 1
    print("regenerated %d/%d TUs" % (ok, len(FRONTIER)))
    if ok != len(FRONTIER):
        sys.exit(1)


def selftest():
    """Pin the taxonomy against hand-built instruction streams. No toolchain."""
    from parse import Comdat, Insn
    checks, failed = 0, 0

    def mk(lines):
        c = Comdat(5, "t", 4 * len(lines), ".text")
        for k, (m, ops, notes) in enumerate(lines):
            c.insns.append(Insn(4 * k, 0, m + " " + ops, notes))
        return c

    row = {"eh": "eh-none", "key": "k", "cflow": "cflow-straight"}
    obj = read_dump.__self__ if False else None

    cases = [
        # (name, lines, expected-in-HARD, expected-NOT-in-HARD|SOFT)
        ("ctr loop collapses mtctr+bdnz",
         [("mtctr", "11", []), ("bdnz", ".-4", [])], {"ctr-loop"}, {"insn:mtctr", "insn:bdnz"}),
        ("backward branch is HARD",
         [("bf", "26, .-8", [])], {"backward-branch"}, set()),
        ("forward bc alone is free",
         [("cmpwi", "6, 3, 0", []), ("bf", "26, .+8", []), ("blr", "", [])],
         set(), {"backward-branch", "cr0", "join-block"}),
        ("cr0 from a record form",
         [("clrlwi.", "10, 10, 31", []), ("bt", "2, .+8", []), ("blr", "", [])],
         {"cr0"}, {"insn:clrlwi"}),
        ("cr0 from a call-result compare",
         [("bl", ".-4", ["REL24 -> [9] g"]), ("cmpwi", "0, 3, 0", []),
          ("bt", "2, .+8", []), ("blr", "", [])], {"cr0"}, set()),
        ("register-register compare",
         [("cmplw", "6, 3, 11", [])], {"cmp-reg-reg"}, set()),
        ("indexed load has no encoder",
         [("lwzx", "10, 9, 10", [])], {"insn:lwzx"}, set()),
        ("rlwinm extended forms are IN vocabulary",
         [("slwi", "9, 11, 2", []), ("clrlwi", "11, 3, 27", []), ("blr", "", [])],
         set(), {"insn:slwi", "insn:clrlwi"}),
        ("savegprlr is refused by name",
         [("bl", ".-4", ["REL24 -> [9] __savegprlr_26"])], {"frame-savegprlr"}, set()),
        ("a stack local is SOFT, not HARD",
         [("stwu", "1, -112(1)", []), ("stw", "11, 80(1)", []), ("blr", "", [])],
         set(), {"frame-locals"}),
    ]
    for name, lines, want_hard, want_absent in cases:
        c = mk(lines)
        f = features(c)
        h, s = tokens(f, row, obj)
        checks += 1
        miss = want_hard - h
        extra = want_absent & (h | s)
        # "a stack local is SOFT" — assert it landed in SOFT, not that it is absent
        if name.startswith("a stack local"):
            extra = set()
            if "frame-locals" not in s:
                miss = miss | {"frame-locals in SOFT"}
        if miss or extra:
            failed += 1
            print("FAIL %-42s missing=%s unexpected=%s (hard=%s soft=%s)"
                  % (name, sorted(miss), sorted(extra), sorted(h), sorted(s)))
        else:
            print("ok   %s" % name)
    print("selftest: %d checks, %d failed" % (checks, failed))
    if checks == 0:
        print("FAIL: zero checks ran")
        return 1
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
