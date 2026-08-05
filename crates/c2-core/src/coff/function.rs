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
///
/// # The opposite error is caught HERE, in two halves, and only one is closed
///
/// **Closed:** `coff::tests`'s
/// `the_writer_vocabulary_is_every_section_name_this_file_emits` reconciles this
/// constant against the `Section { name: … }` literals of every `coff/*.rs` that
/// can build one. A name added here with no `Section` behind it turns that test
/// red. **Measured, not asserted** (`docs/rungs/2026-08-04-w-rdata.md` §5):
/// adding `".rdata$r"` to this list and nothing else makes the test fail *and*
/// makes `c2rs gap` read **`factor-c 590`, `b-and-c 315`, `mismatch 0`** — the
/// scan happily reports the whole `+421` over a writer that emits nothing.
/// **The scan is not the control. This test is.**
///
/// **Open:** a `Section { name: … }` literal inside an emitter that **nothing
/// calls** satisfies that test and still inflates C. That is precisely what
/// `container::bss_deferred_layout` was — a `.bss` layout the differential had
/// never graded one byte of, which disagreed with reality on the walk *and* on
/// the free list once a real caller was written — and board **#278** deleted it.
/// `emit_mvp_obj` below is the live instance of the class today: its only caller
/// is a test. It inflates nothing, because every name it emits is also emitted
/// by a called emitter, and that is luck rather than a guarantee.
///
/// # What is NOT here, and what it would cost
///
/// The workload uses **13** names; these are 10. The three missing ones are the
/// factor-C ladder, and `c2rs gap` prints it every run:
///
/// | name | C if added | TUs it blocks |
/// |---|---:|---:|
/// | **`.rdata$r`** | **590** | 676 |
/// | `.text$yd` | 804 | 243 |
/// | `.xdata$x` | 871 | 67 |
///
/// # THE LADDER IS WORTH `+0` TU MATCH — ALL THREE STEPS, MEASURED
///
/// The table above is a **factor-C** table, and C is one of four terms. Lane
/// `w-joint2` closed the whole ladder transiently and read every downstream
/// joint (`docs/rungs/_2026-08-05-w-joint2.md`, board **#360**):
///
/// | writer vocabulary | C | `B∧C` | **`A∧B∧C`** | **`A∧B∧C∧(D∨E)`** | **FRONTIER** | **match** |
/// |---|---:|---:|---:|---:|---:|---:|
/// | today, these 10 names | 169 | 151 | **27** | **8** | **19** | **8** |
/// | `+ .rdata$r` | 590 | 315 | **27** | **8** | **19** | **8** |
/// | `+ .text$yd` | 804 | 324 | **27** | **8** | **19** | **8** |
/// | `+ .xdata$x` — C **closed** | **871** | 338 | **27** | **8** | **19** | **8** |
///
/// **C moves 169 → 871 and nothing else moves at all.** The reason is one
/// count: **`A∧B` is 27 and every one of the 27 is already inside C** — zero
/// TUs on this workload have a reachable emit set and a complete binding *and*
/// are blocked by a section name. So a section name cannot be any TU's binding
/// constraint, and `+421` of C is `+0` of everything downstream of it.
///
/// Two further counts make it unconditional rather than a snapshot:
///
/// * **`|D∨E| = 10`.** A byte-exact obj requires `A∧B∧C∧(D∨E)`, so **TU match
///   is capped at 10** by the port's codegen class alone, at *every* C, under
///   *every* emit-set model. Match is 8; the entire non-codegen headroom on
///   this workload is **2 TUs**.
/// * **0 of the 676 `.rdata$r` TUs are in `D∨E`, and 0 satisfy `A∧B`.** The
///   population this section name unlocks is exactly the population the port
///   has no accepted route to a single function of. The two convertible TUs
///   (`decomp_pch.cpp`, `vec.cpp`) carry **no** out-of-vocabulary section at
///   all — both are already inside C and fail only **A**.
///
/// So `.rdata$r` is a **reach** rung, not a match rung. `w-reach`'s `+91`
/// (board #302) is `|{model exact} ∩ B∧C|` and carries no `A` term and no
/// `D∨E` term; it is real as reach and converts nothing. Anyone pricing this
/// name against TU match should read `_2026-08-05-w-joint2.md` first.
///
/// **`.rdata$r` is MSVC RTTI and is specified in
/// [`docs/OBJ_RDATA_R_SHAPE.md`](../../../../docs/OBJ_RDATA_R_SHAPE.md)** —
/// measured on a 22-source hierarchy grid and 38 real workload objs, down to
/// the byte. Three things a lane picking it up should not re-derive:
///
/// * the trigger is a **vftable**, i.e. a constructor or destructor body of a
///   polymorphic class defined in this TU. `dynamic_cast` and `typeid` mint
///   **zero** `.rdata$r`; `__declspec(novtable)` takes an otherwise-complete
///   TU to five sections. So **every `.rdata$r` obj has at least one `.text`
///   COMDAT**, which puts the section out of `emit_data_obj`'s reach forever —
///   that emitter's whole class is "defines no functions";
/// * the record bytes are **derivable**: 3,337 of 3,570 real records rebuild
///   exactly from their own mangled symbol names, and all 3,570 do once three
///   class-layout integers (`??_R4.offset`, `??_R4.cdOffset`,
///   `??_R3.attributes`) are supplied. Their aux `CheckSum` is already
///   `coff_checksum` (`coff/checksum.rs`), 3,570 of 3,570;
/// * one rule — a DFS pre-order over the relocation graph — fixes both the
///   section order and the undefined-external order, exact on 25 of 25 grid
///   objs and 38 of 38 workload objs.
///
/// **The blocker is not in this crate.** The minimal case needs seven
/// independent facts and the first two are `c2-il`'s: the vfptr-store leaf body
/// (`expr-op-0x27`, which the parse refuses) and a reader for the `??_R*` record
/// graph. Lane `w-rdata` measured all of that and **declined to add the name**,
/// because a vocabulary entry with no caller behind it is +421 of reachability
/// the port does not have.
pub const PORT_WRITER_SECTIONS: [&str; 10] = [
    ".drectve",
    ".debug$S",
    ".XBLD$W",
    ".text",
    ".pdata",
    ".rdata",
    ".text$yc",
    ".bss",
    ".CRT$XCU",
    ".data",
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
