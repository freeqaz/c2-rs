#!/usr/bin/env python3
"""P0.6(a) length-plasticity probe — does c2 accept a `.ex` IL stream whose
BYTE LENGTH differs from what the front-end emitted?

P0.2 proved c2 consumes *same-length* edited IL (operand retarget, opcode
substitution) and re-optimizes it, with no cross-file checksum. It also
showed that truncating `.ex` mid-stream SIGSEGVs — something is length-bearing.
This probe bounds every insert/delete/grow IL-space move and the K3 codec
grain by answering: is `.ex` length plastic, and under what re-emit
obligations?

Battery (cheapest / most-diagnostic first). Each edit is applied in a fresh
copy of a captured bundle and replayed through the P0.1 `c2host` stub to a
FIXED `-Fo` path (so the only obj-baked path is constant; any obj delta is the
edit's true effect). Semantic-no-op edits are byte-compared to the unedited
baseline; genuine insert/delete edits are byte-compared to a DIRECT capture of
the equivalent source.

  A  varint-widen, single fn      1-byte literal -> `80`+LE32 (+4B), only fn
  B  varint-narrow, single fn     the reverse (-4B), synthetic (FE never emits)
  C  widen fn1 of 3, .gl UNPATCHED shifts fns 2,3; leaves .gl offsets stale
  D  widen fn1 of 3, .gl PATCHED   +4 on the fn2/fn3 `.gl` body-start offsets
  E  genuine INSERT, single fn     splice `LIT 5; ADD` -> (a+5)+5 (+6B)
  F  genuine DELETE, single fn     drop `LOAD c; ADD` -> a+b (-7B)
  G  whole-function delete         drop the last fn's `4F 1F` segment (.ex only)

Key structural fact this probe establishes: the per-function `.ex` body-start
offset is carried in `.gl` as `80 <LE32>` (0x0A54, 0x0AC1, ... == the `4F 1F`
marker offsets), NOT in the `.ex` header. So a length change that shifts a
downstream function's start must re-emit that `.gl` offset table or c2 seeks a
stale offset and SIGSEGVs.

Paths are env-driven (same `C2RS_*` convention as `p0_2_edit_tolerance.py`),
whose capture/replay/mutate machinery this reuses. Run from anywhere:
    python3 probes/p0_6a_length_plasticity.py

Requires: wibo (release), the DC3 X360 toolchain (cl.exe + c2.dll), strace,
and the built c2host stub. All degrade to a clear error if absent.
"""
import os, sys, tempfile, binascii

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import p0_2_edit_tolerance as p02  # reuse capture/replay/mutate/obj_facts

# Redirect the reused machinery to a P0.6-specific work root (script dir stays
# SEPARATE from the work root — a prior probe rmtree'd its own script).
WORK = p02.env("C2RS_P06A_WORK", os.path.join(tempfile.gettempdir(), "c2rs-p06arun"))
SRC = os.path.join(WORK, "src")   # synthetic single-fn sources captured at runtime
p02.ROOT = WORK
p02.FIXED_OBJ = os.path.join(WORK, "fixed_out.obj")

hx = lambda b: binascii.hexlify(b).decode()
B = bytes.fromhex


def wsrc(name, code):
    open(os.path.join(SRC, name + ".cpp"), "w").write(code)


def cap(name, fix=None):
    """Capture `name`.cpp. `fix` overrides the fixture dir (repo fixtures by
    default; SRC for the synthetic single-fn sources)."""
    saved = p02.FIX
    if fix:
        p02.FIX = fix
    try:
        bd, hsh, args, ref = p02.capture(name)
    finally:
        p02.FIX = saved
    return bd, hsh, args, ref


def norm_eq(a, b):
    return a is not None and b is not None and p02.norm(a) == p02.norm(b)


def replay_edit(bd, hsh, args, tag, fn):
    """Copy the bundle, apply `fn(paths)`, replay. Returns (verdict, obj, tail)."""
    w = p02.mutate_dir(bd, hsh, tag, fn)
    return p02.replay(w, hsh, args, tag)


def show(tag, desc, verdict, obj, ref, ref_label):
    """Print a one-line verdict with a byte-exact comparison against `ref`."""
    cmp = ""
    if obj is not None and ref is not None:
        cmp = "BYTE-EXACT==" + ref_label if norm_eq(obj, ref) else "DIFF vs " + ref_label
        if not norm_eq(obj, ref):
            t1, _ = p02.obj_facts(obj)
            t2, _ = p02.obj_facts(ref)
            if t1 != t2:
                cmp += f" (.text {t2} -> {t1})"
    print(f"[{tag:22}] {desc}\n    -> {verdict}   {cmp}")
    return verdict, obj


def main():
    for tool, path in [("wibo", p02.WIBO), ("cl.exe", p02.CL),
                       ("c2.dll", p02.C2), ("c2host", p02.C2HOST)]:
        if not os.path.exists(path):
            print(f"MISSING {tool}: {path}\n(set C2RS_* env or build c2host first)")
            return 2
    os.makedirs(SRC, exist_ok=True)
    print("=== P0.6(a) length-plasticity probe ===\n")

    # Synthetic single-function sources (written at runtime; not repo fixtures).
    wsrc("p6_addk",  "int addk(int a){ return a + 5; }\n")
    wsrc("p6_addk2", "int addk(int a){ return a + 5 + 5; }\n")   # FE emits TWO lit-5 adds
    wsrc("p6_add3",  "int add3(int a, int b, int c){ return a + b + c; }\n")
    wsrc("p6_ab",    "int add3(int a, int b, int c){ return a + b; }\n")  # c unused

    # ---- single-fn base captures + their direct targets -------------------
    # Target objs are obtained by REPLAY of the direct capture through the same
    # FIXED `-Fo` path (not the capture's own reference obj, whose distinct -Fo
    # path leaks into `.debug$S` S_OBJNAME and would spuriously fail the compare).
    bd_k, h_k, a_k, _ = cap("p6_addk", SRC)
    _, base_k, _ = p02.replay(bd_k, h_k, a_k, "base_k")
    bd_k2, h_k2, a_k2, _ = cap("p6_addk2", SRC)    # a+5+5  (== a+10)
    _, ref_addk2, _ = p02.replay(bd_k2, h_k2, a_k2, "ref_addk2")
    bd3, h3, a3, _ = cap("p6_add3", SRC)
    bd_ab, h_ab, a_ab, _ = cap("p6_ab", SRC)       # a+b (c unused)
    _, ref_ab, _ = p02.replay(bd_ab, h_ab, a_ab, "ref_ab")
    bt, bn = p02.obj_facts(base_k)
    print(f"single-fn base p6_addk: obj={len(base_k)}B fn={bn} .text={bt}\n")

    # === A: varint-widen the only literal (pure +4B length, semantic no-op) ==
    LIT_N = B("3386417405")        # LIT int 5, 1-byte form
    LIT_W = B("338641748005000000")  # LIT int 5, wide `80`+LE32 form

    def widen_only(p):
        b = p02.rd(p["ex"]); i = b.find(LIT_N)
        assert i >= 0, "narrow literal not found"
        b[i:i + len(LIT_N)] = LIT_W; p02.wr(p["ex"], b)

    vA, oA, _ = replay_edit(bd_k, h_k, a_k, "A_widen", widen_only)
    show("A widen (single-fn)", "1-byte lit 5 -> wide 80+LE32 (+4B), only fn",
         vA, oA, base_k, "baseline")
    # determinism
    vA2, oA2, _ = replay_edit(bd_k, h_k, a_k, "A_widen2", widen_only)
    print(f"    determinism: {vA2}  {'obj==first' if norm_eq(oA, oA2) else 'obj DIFFERS!'}")

    # === B: varint-narrow the widened form back (-4B) — synthetic shrink =====
    def narrow_widened(p):
        b = p02.rd(p["ex"]); i = b.find(LIT_N)
        b[i:i + len(LIT_N)] = LIT_W                       # first widen
        j = b.find(LIT_W); b[j:j + len(LIT_W)] = LIT_N    # then narrow back
        p02.wr(p["ex"], b)
    vB, oB, _ = replay_edit(bd_k, h_k, a_k, "B_narrow", narrow_widened)
    show("B narrow (synthetic)", "wide lit -> 1-byte (-4B); FE never emits wide-small",
         vB, oB, base_k, "baseline")

    # === C / D: length change in a NON-last fn (mvp_lit fn1 of 3) ============
    # mvp_lit funcs at .ex 0x0A54, 0x0ABD, 0x0B26; .gl carries these as 80+LE32.
    bd_l, h_l, a_l, _ = cap("mvp_lit")
    _, base_l, _ = p02.replay(bd_l, h_l, a_l, "base_l")
    F1 = B("b9e309864174") + LIT_N + B("02")   # addk: LOAD a, LIT 5, ADD

    def widen_f1(p):
        b = p02.rd(p["ex"]); i = b.find(F1)
        assert i >= 0, "fn1 addk stream not found"
        lp = i + len(B("b9e309864174"))
        b[lp:lp + len(LIT_N)] = LIT_W; p02.wr(p["ex"], b)

    vC, oC, tC = replay_edit(bd_l, h_l, a_l, "C_nopatch", widen_f1)
    show("C widen-f1 .gl UNPATCHED", "shift fns 2,3; leave .gl offsets stale",
         vC, oC, base_l, "baseline")
    if oC is None:
        print(f"    (c2 tail: {tC[:90]})")

    def widen_f1_patch_gl(p):
        widen_f1(p)
        g = p02.rd(p["gl"])
        for old, new in [(0x0ABD, 0x0AC1), (0x0B26, 0x0B2A)]:   # +4 each downstream
            ob = b"\x80" + old.to_bytes(4, "little")
            nb = b"\x80" + new.to_bytes(4, "little")
            k = g.find(ob); assert k >= 0, f".gl offset {hex(old)} not found"
            g[k:k + 5] = nb
        p02.wr(p["gl"], g)
    vD, oD, _ = replay_edit(bd_l, h_l, a_l, "D_patch", widen_f1_patch_gl)
    show("D widen-f1 .gl PATCHED", "same +4B, but re-emit fn2/fn3 .gl offsets",
         vD, oD, base_l, "baseline")

    # === E: genuine INSERT (grow the stream) — single fn, no downstream ======
    INS = B("3386417405") + B("02")   # extra LIT 5 ; ADD  -> (a+5)+5
    def insert_stmt(p):
        b = p02.rd(p["ex"]); anchor = B("b9e309864174") + LIT_N + B("02")
        i = b.find(anchor); assert i >= 0, "addk stream not found"
        pos = i + len(anchor)
        b[pos:pos] = INS; p02.wr(p["ex"], b)      # duplicate LIT5;ADD in place
    vE, oE, _ = replay_edit(bd_k, h_k, a_k, "E_insert", insert_stmt)
    show("E insert (+6B)", "splice LIT 5;ADD -> (a+5)+5; target = direct a+5+5",
         vE, oE, ref_addk2, "direct(a+5+5)")
    if oE is not None:
        te, _ = p02.obj_facts(oE); print(f"    grown-IL .text = {te}")
    vE2, oE2, _ = replay_edit(bd_k, h_k, a_k, "E_insert2", insert_stmt)
    print(f"    determinism: {vE2}  {'obj==first' if norm_eq(oE, oE2) else 'obj DIFFERS!'}")

    # === F: genuine DELETE (shrink the stream) — single fn ===================
    LOADC = B("b9e509864174") + B("02")   # LOAD c ; ADD  (the +c limb)
    def delete_stmt(p):
        b = p02.rd(p["ex"]); i = b.find(LOADC)
        assert i >= 0, "LOAD c;ADD not found"
        del b[i:i + len(LOADC)]; p02.wr(p["ex"], b)
    vF, oF, _ = replay_edit(bd3, h3, a3, "F_delete", delete_stmt)
    show("F delete (-7B)", "drop LOAD c;ADD -> a+b; target = direct a+b (c unused)",
         vF, oF, ref_ab, "direct(a+b)")
    if oF is not None:
        tf, _ = p02.obj_facts(oF); print(f"    shrunk-IL .text = {tf}")

    # === G: whole-function delete (last fn's .ex segment) ====================
    # mvp_two = add2 (0x0A54) + add4 (0x0AC1). Drop add4's 4F 1F..module-end
    # segment from .ex ONLY (leave .gl/.sy over-describing 2 fns): does c2
    # tolerate .ex carrying FEWER functions than .gl lists?
    bd_t, h_t, a_t, _ = cap("mvp_two")
    MARK = B("4f1f800500a0004f2080fe00")
    MODEND = B("4f0220004f01")   # module-end trailer prefix (…NN 4D)

    def drop_last_fn_ex(p):
        b = p02.rd(p["ex"])
        marks = []
        s = 0
        while True:
            j = b.find(MARK, s)
            if j < 0:
                break
            marks.append(j); s = j + 1
        assert len(marks) >= 2, f"expected >=2 fns, got {len(marks)}"
        me = b.rfind(MODEND)
        assert marks[-1] < me, "module-end not after last fn"
        # excise [last 4F1F marker .. module-end) ; keep module-end trailer
        del b[marks[-1]:me]; p02.wr(p["ex"], b)
    vG, oG, tG = replay_edit(bd_t, h_t, a_t, "G_dropfn", drop_last_fn_ex)
    print(f"[{'G drop-last-fn .ex':22}] excise add4 4F1F segment; .gl/.sy untouched\n"
          f"    -> {vG}")
    if oG is not None:
        tg, ng = p02.obj_facts(oG); print(f"    obj fn={ng} .text={tg}")
    else:
        print(f"    (c2 tail: {tG[:90]})")

    print("\n=== done ===")
    return 0


if __name__ == "__main__":
    sys.exit(main())
