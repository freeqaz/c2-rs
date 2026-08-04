//! One emitted function: its name, its `.text` offset, its call sites and its
//! data references, plus the FP-constant pool those drag in.

use super::*;

/// Build the complete MVP `.obj` image bytes.
///
/// * `obj_name` — the `-Fo` output-path string exactly as the reference saw it
///   (e.g. `Z:\tmp\anat\mvp.obj`); embedded verbatim in `.debug$S` S_OBJNAME.
/// **The port's COFF writer vocabulary** — every section name any emitter in
/// this module can put in an obj, and therefore the whole of what **factor C**
/// (`docs/ROADMAP.md` §10.19) admits.
///
/// **This is the published home of the list.** `c2-harness`'s `gap.rs` used to
/// carry a hand-written mirror of it, with its own doc comment stating that the
/// list "should be `c2-core`" and was duplicated only because this crate
/// belonged to another lane. W-R1c owns this crate, and the mirror stopped being
/// accurate the moment `emit_dyninit_obj` acquired a caller: the port can now
/// emit `.text$yc`, `.bss` and `.CRT$XCU`, none of which the six-name mirror
/// listed.
///
/// It is a **vocabulary, not a section list** — `.XBLD$W` is emitted twice (the
/// C1 and C2 watermarks) and appears once here.
///
/// The control that can turn this red is in the scan, not here: every
/// byte-exact TU's obj must fall inside factor C, and a `match` obj *is* the
/// port's own output — so a list that is too small makes a matching TU fall
/// outside C and the scan says so. That control cannot catch the opposite error
/// (a name here that no writer emits, which would inflate C), which is why this
/// stays a transcription of the `Section { name: … }` tables below rather than a
/// generalization of them.
pub const PORT_WRITER_SECTIONS: [&str; 9] = [
    ".drectve",
    ".debug$S",
    ".XBLD$W",
    ".text",
    ".pdata",
    ".rdata",
    ".text$yc",
    ".bss",
    ".CRT$XCU",
];

/// * `mangled_name` — the function's mangled symbol (from `.gl`), e.g.
///   `?add3@@YAHHHH@Z`.
/// * `text` — the `.text` bytes from codegen (12 for `add3`).
pub fn emit_mvp_obj(obj_name: &str, mangled_name: &str, text: &[u8]) -> Vec<u8> {
    // Label counter unused: a `Function::plain` has no frame, so no `$M`/`$T`.
    emit_obj(obj_name, &[Function::plain(mangled_name, 0)], text, 0)
}

/// A relative-branch (REL24) relocation for a tail call: the callee's mangled
/// name and the `.text` byte offset of the branch instruction to patch.
pub struct Call<'a> {
    pub reloc_offset: u32,
    pub callee: &'a str,
}

/// **WR1 — one reference to a NAMED DATA SYMBOL's address**: the `.text` byte
/// offset of the `lis rS,sym@ha` that opens it, plus the symbol's mangled name.
///
/// **The two halves are NOT adjacent, and that is the one place this differs from
/// [`crate::codegen::FpConstRef`].** The `lis` is hoisted to the top of the body
/// while the `addi rD,rS,sym@l` takes its own argument slot's turn in the
/// descending setup walk, so a literal slot above it lands *between* them —
/// MEASURED (`work/wr1/probes/p4.cpp`, `void a7(){ gsp(&gI, 7); }`):
/// `lis r11 · li r4,7 · addi r3,r11,0 · b`, with REFHI at the function's start
/// and REFLO **eight** bytes later, not four. Carrying one offset and adding 4
/// was a live wrong-bytes emit on exactly that body, caught by the differential
/// before it left this worktree.
///
/// Four relocation records: REFHI + PAIR at `hi_off`, REFLO + PAIR at `lo_off`,
/// both PAIRs against symbol index 0.
///
/// The symbol itself is an **undefined external DATA** symbol — `Type` 0x0000,
/// where a callee carries 0x0020 — emitted in this function's group after its
/// callee externals. MEASURED (`work/wr1/probes/p1.cpp`): `void f(){ gso(&gI); }`
/// gives `?f5@@YAXXZ`, `?gso@@YAXPAH@Z`, `?gI@@3HA`, in that order, with the
/// callee ahead of the data symbol because its `26` push precedes the argument's
/// (`docs/IL_CALL_IN_EXPR.md` §17.2 item 6).
pub struct DataRef<'a> {
    pub hi_off: u32,
    pub lo_off: u32,
    pub name: &'a str,
}

/// One function placed in `.text`: its mangled name (from `.gl`), byte offset
/// within the concatenated `.text`, and one relocation per call it makes.
pub struct Function<'a> {
    pub name: &'a str,
    pub text_offset: u32,
    /// Every REL24 site this function contributes, in ascending `.text` offset —
    /// a tail call's `b`, a framed call's `bl`, or one `bl` per call of a Class A
    /// many-call body. **A list, not an `Option`**: the shipped framed class had
    /// exactly one call site, and every "the" in this file's relocation and
    /// symbol code was that constant.
    ///
    /// Duplicates are expected and are not an error. `void f(){ g(); h(); g(); }`
    /// has three sites and **two** external symbols: c2 emits one undefined
    /// external per distinct callee and relocates every later site against that
    /// same index (measured — both `?g1` relocations in the three-call probe point
    /// at symbol 16).
    pub calls: Vec<Call<'a>>,
    /// True iff this function's body **touches floating point** in any way. The
    /// obj then carries an undefined external `_fltused`, emitted immediately
    /// after the FIRST such function's symbol group — the CRT's float-support
    /// hook. Verified: a pure FP leaf changes the obj shell by exactly this one
    /// symbol (`docs/CODEGEN_W13_FLOAT.md` §4).
    ///
    /// **"Touches FP" and "is a float leaf" are two facts**, and they shared this
    /// field until the FP store leaf pulled them apart:
    /// `void f(S* s, float v){ s->f = v; }` needs the marker and is a store leaf
    /// with a label stride of 1, not 2. The producer is
    /// [`c2_il::IlFunction::touches_floating_point`]; the stride is
    /// [`c2_il::IlFunction::label_slots`]. One field, two readers, and the
    /// mismatch it caused was 14 out of 14 objs short by one symbol.
    pub is_float: bool,
    /// W13b: this function's floating-point constant reference sites, in
    /// emission order, with `hi_off` already rebased to the whole `.text`.
    pub fp_refs: Vec<crate::codegen::FpConstRef>,
    /// **WR1**: this function's named-data-symbol address references, in emission
    /// order, with `hi_off` already rebased to the whole `.text`. At most one in
    /// the class the parser admits; a `Vec` because the relocation and symbol code
    /// below is written over a list either way and a "the" here would be the same
    /// constant [`Function::calls`]' own comment records having been.
    pub data_refs: Vec<DataRef<'a>>,
    /// `Some` iff this function establishes a stack frame, carrying the two
    /// lengths its `.pdata` record and its two `$M` labels need. `None` for a
    /// leaf — c2 emits no unwind record for one, so this field alone decides
    /// whether the obj has a `.pdata` section at all.
    pub frame: Option<Frame>,
    /// **Compiler-label counter slots this function takes BEFORE its own `$M`
    /// triple** — 0 for every class but WCR's signed two-call comparator, which
    /// takes 2.
    ///
    /// A *leading* count and not merely a bigger stride: it moves this
    /// function's own `$M`/`$M`/`$T` numbers up as well as every later
    /// function's, which is the placement `docs/CODEGEN_FRAMED_CALLS.md` §4.4
    /// records for the `__savegprlr_N`/`__restgprlr_N` pair and
    /// `docs/LABEL_COUNTER.md` §1.1 tabulates as a surcharge. Producer:
    /// [`c2_il::IlFunction::label_lead`]; the total stride it feeds is
    /// `c2_il::IlFunction::label_slots`, and the two are separate because
    /// [`plan_labels`] needs to add them at different points. Moving the same
    /// two slots to *after* the triple is 119 mismatches in
    /// `scripts/sweep.d/98-cmp-order.py`, i.e. the placement is graded and not
    /// merely the total.
    pub label_lead: u32,
}

impl<'a> Function<'a> {
    /// A function with no call, no constant pool and no frame — the common case.
    pub fn plain(name: &'a str, text_offset: u32) -> Function<'a> {
        Function {
            name,
            text_offset,
            calls: Vec::new(),
            is_float: false,
            fp_refs: Vec::new(),
            data_refs: Vec::new(),
            frame: None,
            label_lead: 0,
        }
    }

    /// The callees this function introduces to the symbol table, in the order
    /// their symbols are **emitted**: distinct names in **reverse first-reference
    /// order**.
    ///
    /// Measured (`docs/OBJ_GY_SHAPES.md` §3.3 as extended, byte evidence in
    /// `docs/CODEGEN_FRAMED_CALLS.md` §4.1). `f(){ g1(); g2(); g3(); }` puts `?g3`
    /// at index 15, `?g2` at 16 and `?g1` at 17 — and the mirrored source
    /// `g3(); g2(); g1();` puts `?g1` at 15, which is what refutes both
    /// "alphabetical" and "declaration order". `g1(); g2(); g1();` emits two
    /// symbols, not three, and its repeat relocates against the first.
    ///
    /// This is the same LIFO the `.rdata` constant pool uses within one function
    /// (§2.3) and it has the same failure mode: a naive append emits every index
    /// swapped and **every relocation still resolves**, so the obj is wrong in a
    /// way no linker complains about.
    pub(crate) fn introduced_callees(&self) -> Vec<&'a str> {
        let mut first_ref: Vec<&'a str> = Vec::with_capacity(self.calls.len());
        for c in &self.calls {
            if !first_ref.contains(&c.callee) {
                first_ref.push(c.callee);
            }
        }
        first_ref.reverse();
        first_ref
    }
}

/// `.rdata` COMDAT characteristics for a pooled FP constant:
/// CNT_INITIALIZED_DATA | LNK_COMDAT | ALIGN_4/8 | MEM_READ. The alignment field
/// is the only difference between the `float` and `double` pools.
pub(crate) const CH_RDATA_F32: u32 = 0x4030_1040;
pub(crate) const CH_RDATA_F64: u32 = 0x4040_1040;


/// The mangled symbol name c2 gives a pooled FP constant: `__real@` followed by
/// the big-endian IEEE bit pattern in lowercase hex — 8 digits for a `float`,
/// 16 for a `double`.
pub(crate) fn real_symbol_name(bits: u64, double: bool) -> String {
    if double {
        format!("__real@{bits:016x}")
    } else {
        let v = f64::from_bits(bits) as f32;
        format!("__real@{:08x}", v.to_bits())
    }
}

/// The pooled constant's raw `.rdata` bytes: big-endian IEEE-754, narrowed to
/// binary32 for a `float`. The narrowing is exactness-checked in codegen before
/// the reference is ever recorded.
pub(crate) fn real_raw_bytes(bits: u64, double: bool) -> Vec<u8> {
    if double {
        bits.to_be_bytes().to_vec()
    } else {
        (f64::from_bits(bits) as f32).to_be_bytes().to_vec()
    }
}

/// The CRT float-support marker symbol.
pub(crate) const NAME_FLTUSED: &str = "_fltused";
