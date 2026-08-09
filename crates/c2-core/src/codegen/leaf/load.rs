//! The indirect-load leaf: `return p->m;` / `return *p;` / `return p[k];`
//! as one `lwz`/`lbz`/`lhz`/`ld` at a folded displacement, plus its sign
//! extension. Consumer of the sub-object designator
//! (`c2-il/src/func/body/shapes/designator.rs`).

use c2_il::{IlFunction, IlOp};
use crate::BackendError;
use crate::codegen::encode::{
    encode_blr,
    encode_extsb,
    encode_lbz,
    encode_ld,
    encode_lhz,
    encode_lwz,
};
use crate::codegen::select::{ARG_REGS, RET_REG, SCRATCH_REG, out_of_class};

/// Lower an **indirect-load leaf** — `return *p;` / `return s->m;` /
/// `return p[k];` / `return mMember;` — to one load + `blr`.
///
/// Recognized by an **exact** two-op stream `[Load(base), LoadInd { off }]` or
/// `[Load(base), LoadIndSized { … }]`, which `c2_il::try_parse_indirect_load_leaf`
/// is the only producer of. Returns `None` for anything else so the ordinary
/// selector keeps its behaviour unchanged; the pattern is deliberately not a prefix
/// match, because c2 does *not* lower a load that feeds arithmetic this way —
/// `*p + 1` puts the loaded value in the scratch register
/// (`lwz r11,0(r3) ; addi r3,r11,1`, and for a `char*`
/// `lbz r11 ; extsb r11,r11 ; addi r3,r11,1`) and `*p * 3` is strength-reduced.
///
/// The measured lowering table (`/Ox /GS-` and the workload's `/O1`, identical
/// unless noted; `docs/IL_LOAD_TYPES.md` §3 plus this project's own re-capture):
///
/// ```text
///   T f(T*)                          int f(T*)   (an IL `2C … 00` to int)
///   char/schar   lbz r3              lbz r11 ; extsb r3,r11   <- the r11 rule
///   uchar/bool   lbz r3              lbz r3      (the widening is free)
///   short        lhz r3, NEVER lha    /O1: lha r3   /Ox,/O2: lhz r11 ; extsh r3,r11
///   ushort/wchar lhz r3              lhz r3
///   int/unsigned lwz r3              lwz r3
///   long long    ld r3 (DS-form)     — not captured
/// ```
///
/// The signed-halfword widening is the one row this function cannot emit: it is the
/// only shape in the table whose *instruction count* depends on the optimization
/// mode, and this path takes no mode. The parser refuses it, so it never arrives
/// here; the `Err` below is the second lock, not the primary one.
///
/// `func.params` maps the base token to its incoming argument register by
/// declaration order, with a member function's `this` already at index 0.
pub fn indirect_load_text(func: &IlFunction) -> Option<Result<Vec<u8>, BackendError>> {
    let (base_tok, off, width, sext) = match func.ops.as_slice() {
        [IlOp::Load(t), IlOp::LoadInd { off }] => (*t, *off, 4u8, false),
        [IlOp::Load(t), IlOp::LoadIndSized { off, width, sext }] => (*t, *off, *width, *sext),
        _ => return None,
    };
    let d = match i16::try_from(off) {
        Ok(d) => d,
        // The parser gates this; if it ever changed, refuse rather than truncate.
        Err(_) => return Some(Err(out_of_class("indirect load offset exceeds a 16-bit displacement"))),
    };
    let base = match func.params.iter().position(|&t| t == base_tok) {
        Some(i) if i < ARG_REGS.len() => ARG_REGS[i],
        _ => {
            return Some(Err(out_of_class(
                "indirect load whose base is not a register argument",
            )))
        }
    };
    let mut text = Vec::with_capacity(12);
    // A load that feeds a sign-extension targets r11 and the `exts*` produces r3;
    // an unextended load targets r3 directly.
    let dest = if sext { SCRATCH_REG } else { RET_REG };
    match (width, sext) {
        (1, _) => text.extend_from_slice(&encode_lbz(dest, base, d)),
        (2, false) => text.extend_from_slice(&encode_lhz(dest, base, d)),
        (4, false) => text.extend_from_slice(&encode_lwz(dest, base, d)),
        (8, false) if d % 4 == 0 => text.extend_from_slice(&encode_ld(dest, base, d)),
        (8, false) => {
            return Some(Err(out_of_class(
                "8-byte indirect load whose offset is not a multiple of 4 (ld is DS-form)",
            )))
        }
        // Only `width == 1` is ever sign-extended here (see `IlOp::LoadIndSized`).
        _ => {
            return Some(Err(out_of_class(
                "indirect load of an unmodeled width/extension combination",
            )))
        }
    }
    if sext {
        text.extend_from_slice(&encode_extsb(RET_REG, dest));
    }
    text.extend_from_slice(&encode_blr());
    Some(Ok(text))
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;
    #[test]
    fn indirect_load_text_is_one_lwz_and_a_blr() {
        let mut f = IlFunction {
            inlinable: None,
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?ld_p@@YAHPAH@Z".into(),
            source_path: None,
            params: vec![0xEE09],
            ops: vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 0 }],
            tail_call: None,
            framed_call: None,
        call_seq: None,
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
        ptr_walk_loop: None,
            if_call_join: None,
            ptr_walk_chain_loop: None,
        div_mod_leaf: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
        fp_arg_sources: None,
            arg_sources: None,
            data_syms: Vec::new(),
            fn_addr_sym: None,
            data_def: None,
            static_scan_loop: None,
            counted_accum_loop: None,
            guard_chain_shared_tail: None,
        alloc_init_or_fail: None,
        osf_handle_guard: None,
        guard_ret_chain: None,
        memcpy_tail: None,
            nonce_add_run: None,
            xtea_round_loop: None,
            xtea_encrypt_loop: None,
        fp_store_diamond: None,
        ctor_forward_call: None,
        xlrc_create_guard: None,
        json_utf8_copy: None,
            pool_free_list: None,
            pool_ctor_chain: None,
        float_walk_loop: None,
    };
        assert_eq!(
            indirect_load_text(&f).unwrap().unwrap(),
            vec![0x80, 0x63, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20]
        );
        // The base's register comes from its position in `params`, which is where a
        // member function's `this` sits at index 0.
        f.params = vec![0x1234, 0xEE09];
        f.ops = vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 4 }];
        assert_eq!(
            indirect_load_text(&f).unwrap().unwrap(),
            vec![0x80, 0x64, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20]
        );
        // Anything that is not EXACTLY `[Load, LoadInd]` is not this shape: c2 does
        // not lower a load that feeds arithmetic as a destination-register load.
        f.ops = vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 0 }, IlOp::Lit(1), IlOp::Add];
        assert!(indirect_load_text(&f).is_none());
        // …and the affine selector must refuse it rather than pick a register.
        assert!(select_text(&f, OptMode::Ox).is_err());
    }

    #[test]
    fn narrow_indirect_load_text_matches_the_captured_bodies() {
        let f = |ops: Vec<IlOp>, params: Vec<u32>| IlFunction {
            inlinable: None,
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?g@@YADPAD@Z".into(),
            source_path: None,
            params,
            ops,
            tail_call: None,
            framed_call: None,
        call_seq: None,
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
        ptr_walk_loop: None,
            if_call_join: None,
            ptr_walk_chain_loop: None,
        div_mod_leaf: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
        fp_arg_sources: None,
            arg_sources: None,
            data_syms: Vec::new(),
            fn_addr_sym: None,
            data_def: None,
            static_scan_loop: None,
            counted_accum_loop: None,
            guard_chain_shared_tail: None,
        alloc_init_or_fail: None,
        osf_handle_guard: None,
        guard_ret_chain: None,
        memcpy_tail: None,
            nonce_add_run: None,
            xtea_round_loop: None,
            xtea_encrypt_loop: None,
        fp_store_diamond: None,
        ctor_forward_call: None,
        xlrc_create_guard: None,
        json_utf8_copy: None,
            pool_free_list: None,
            pool_ctor_chain: None,
        float_walk_loop: None,
    };
        let blr = [0x4E, 0x80, 0x00, 0x20];
        let body = |ops: Vec<IlOp>, params: Vec<u32>| {
            indirect_load_text(&f(ops, params)).unwrap().unwrap()
        };
        let sized = |width, sext, off| {
            vec![IlOp::Load(0xEE09), IlOp::LoadIndSized { off, width, sext }]
        };
        // `char g_c_c(char* p){return *p;}`  ->  lbz r3,0(r3) ; blr
        assert_eq!(
            body(sized(1, false, 0), vec![0xEE09]),
            [&[0x88, 0x63, 0x00, 0x00][..], &blr].concat()
        );
        // `int g_i_c(char* p){return *p;}`   ->  lbz r11,0(r3) ; extsb r3,r11 ; blr
        // The load targets the SCRATCH register and the extension produces r3 —
        // the r11-then-r3 rule. `lbz r3 ; extsb r3,r3` is the plausible wrong emit.
        assert_eq!(
            body(sized(1, true, 0), vec![0xEE09]),
            [&[0x88 + 1, 0x63, 0x00, 0x00][..], &[0x7D, 0x63, 0x07, 0x74], &blr].concat()
        );
        // `int g_i_c2(int a,char* p){return *p;}` -> base r4, destination still r11
        assert_eq!(
            body(sized(1, true, 0), vec![0x1234, 0xEE09]),
            [&[0x89, 0x64, 0x00, 0x00][..], &[0x7D, 0x63, 0x07, 0x74], &blr].concat()
        );
        // `short g_s_s(short* p){return *p;}` ->  lhz r3,0(r3) — never `lha`
        assert_eq!(
            body(sized(2, false, 0), vec![0xEE09]),
            [&[0xA0, 0x63, 0x00, 0x00][..], &blr].concat()
        );
        // `short m_h(S* s){return s->h;}`    ->  lhz r3,6(r3)
        assert_eq!(
            body(sized(2, false, 6), vec![0xEE09]),
            [&[0xA0, 0x63, 0x00, 0x06][..], &blr].concat()
        );
        // `long long m_q(S* s){return s->q;}` -> ld r3,16(r3)
        assert_eq!(
            body(sized(8, false, 16), vec![0xEE09]),
            [&[0xE8, 0x63, 0x00, 0x10][..], &blr].concat()
        );
        // The 4-byte load keeps its own variant and its own bytes.
        assert_eq!(
            body(vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 4 }], vec![0xEE09]),
            [&[0x80, 0x63, 0x00, 0x04][..], &blr].concat()
        );
        // An 8-byte load whose offset is not a multiple of 4 cannot be a DS-form
        // displacement: c2 emits `li r11,3 ; ldx r3,r3,r11` instead (measured on a
        // `#pragma pack(1)` member, `fixtures/cpp/w12_narrow_neg.cpp`). The parser
        // refuses it; this is the second lock.
        assert!(indirect_load_text(&f(sized(8, false, 3), vec![0xEE09]))
            .unwrap()
            .is_err());
        // Sign extension is only ever modeled at width 1 — a signed halfword
        // widening is mode-dependent and refused upstream.
        assert!(indirect_load_text(&f(sized(2, true, 0), vec![0xEE09]))
            .unwrap()
            .is_err());
        assert!(indirect_load_text(&f(sized(8, true, 0), vec![0xEE09]))
            .unwrap()
            .is_err());
        // A narrow load feeding arithmetic is not this shape at all (c2 extends in
        // place — `extsb r11,r11` — and the leaf extends across registers).
        assert!(indirect_load_text(&f(
            vec![IlOp::Load(0xEE09), IlOp::LoadIndSized { off: 0, width: 1, sext: true }, IlOp::Lit(1), IlOp::Add],
            vec![0xEE09]
        ))
        .is_none());
    }

}
