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
/// **CLOSED 2026-08-06, board #301, lane `w-rtti`:** a `Section { name: … }`
/// literal inside an emitter that **nothing calls** satisfies that test and
/// still inflates C. That is precisely what `container::bss_deferred_layout` was
/// — a `.bss` layout the differential had never graded one byte of, which
/// disagreed with reality on the walk *and* on the free list once a real caller
/// was written — and board **#278** deleted it.
/// `coff::tests::every_production_emitter_has_a_lib_rs_caller` now asserts that
/// every `pub fn emit_*` present in a non-test build is named by `lib.rs`, and
/// `emit_mvp_obj` below is `#[cfg(test)]` rather than exempted from it.
///
/// **Measured, not asserted** — `work/w-rtti/counterfactual.sh` breaks the tree
/// both ways and reports which guard catches which: BREAK 1 (the constant
/// alone) turns the vocabulary test red and leaves this one green; BREAK 2 (the
/// constant **plus** an uncalled emitter carrying a real `.rdata$r` `Section`)
/// leaves the vocabulary test **green** and turns this one red. Both restore to
/// `git status --porcelain crates/` = 0.
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
///
/// **Lane `w-rtti` was briefed to add it anyway, re-derived the price at this
/// master, and declined again** (`docs/rungs/2026-08-07-w-rtti.md`;
/// `OBJ_RDATA_R_SHAPE.md` §8.1). Two of the seven were re-measured directly and
/// both are still unpaid: `c2rs census` on the minimal source still reads
/// **`0/1 functions in class`** at `expr-op-0x27`, and `c2-il`'s `.gl`
/// data-record reader returns **0 of the 11 sections' 6 data records** on that
/// TU — against **2 of 2** on a `.data` control, so the zero is a reading and
/// not a broken probe. What did move is the *shape* of the reader's job: the
/// three class-layout integers `OBJ_RDATA_R_SHAPE.md` §4 calls irreducible are
/// spelled literally in `.in`, and the one thing standing between the existing
/// [`c2_il`] initializer reader and every RTTI record is **element tag `02`**,
/// the symbol-address element it currently refuses by design.
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
///
/// # `#[cfg(test)]` — board **#301**, closed by lane `w-rtti`
///
/// **Its only caller is [`super::tests`], and an emitter with no production
/// caller is the second half of the [`PORT_WRITER_SECTIONS`] hole.** A
/// `Section { name: … }` literal reached only from a test satisfies
/// `the_writer_vocabulary_is_every_section_name_this_file_emits` and still
/// inflates factor **C**, which is what `container::bss_deferred_layout` was
/// before board #278 deleted it. This one inflated nothing — every name it
/// emits is also emitted by a called emitter — and `w-rdata` §10 recorded that
/// as *"luck, not a guarantee"*.
///
/// Making it test-only turns the luck into a guarantee without deleting a
/// fixture eight tests use: `every_production_emitter_has_a_lib_rs_caller` can
/// now assert the property over the emitters that exist in a **release** build,
/// where this function does not.
#[cfg(test)]
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
///
/// # W-EXTDATA — the symbol is not always DATA
///
/// `_vswprintf_s_l` materializes the address of `_woutput_s_l`, a **function**,
/// to pass it as an argument. The relocation is this same REFHI/PAIR/REFLO/PAIR
/// quad, and the symbol record is a callee's: `Type` **0x0020**. Measured side
/// by side in one workload obj — `work/w-extdata/ref/vswprnc/dis.txt` symbol 18
/// reads `type=0x0020` where `work/w-extdata/ref/undname/dis.txt` symbols 15 and
/// 17 read `type=0x0000` for the same relocation shape.
///
/// [`Self::is_function`] carries that, and it is a field rather than a name test
/// because nothing about a mangled name distinguishes the two reliably (`_errno`
/// and `_nhandle` differ in no lexical way at all).
pub struct DataRef<'a> {
    pub hi_off: u32,
    pub lo_off: u32,
    pub name: &'a str,
    /// `true` when the target is a FUNCTION (`Type` 0x0020) rather than a data
    /// name (`Type` 0x0000). See the block above.
    pub is_function: bool,
}

/// **W-DATA — one data object this TU DEFINES, together with the reference the
/// owning function makes to it.**
///
/// The object and the reference travel in ONE carrier, deliberately, and that is
/// the whole difference from [`DataRef`]. `DataRef` names an **undefined
/// external**: the obj gets a symbol and nothing else, so the reference is the
/// only fact there is. A *defined* object is a whole extra section, a defined
/// symbol **and** a relocation, and board **#232**'s direction is exactly what
/// happens when those three are three fields a dispatch can set independently —
/// a `.data` section with no relocation, or a relocation with no section. There
/// is nothing here for an ordering to get wrong.
///
/// # Why `lo_offs` is a list and [`DataRef::lo_off`] is not
///
/// **MEASURED on `src/system/math/Primes.cpp`'s own obj**: one `REFHI` at
/// `.text+0x00` and **two** `REFLO`s, at `+0x08` and `+0x0c`, all three against
/// `?primes@?1??NextHashPrime@@YAHH@Z@4PAHA`. `c2` materializes the high half
/// once and spends it twice — `addi r9,r10,sym@l` for the base and
/// `lwz r10,sym@l(r10)` for the peeled element — so a 1:1 carrier cannot spell
/// the body at all. Reproduced on three one-line cells (`work/w-data/attr/`
/// a1, a4, a5), so it is a property of the *shape* and not of this one TU.
///
/// `DataRef` is left 1:1 on purpose: its class is graded, `emit_dyninit_obj`
/// and FUNCTION BYTE MATCH both read it, and widening a field that six graded
/// call sites already agree about buys nothing this carrier does not.
pub struct DataDef<'a> {
    /// The COFF symbol name, already final — decorated for a function-local
    /// `static` (`?primes@?1??NextHashPrime@@YAHH@Z@4PAHA`).
    pub symbol: &'a str,
    /// `sizeof` the object; also the length of [`Self::bytes`].
    pub size: u32,
    /// The object's natural alignment from the `.gl` TYPE tag — **not** derived
    /// from the size. The section's alignment nibble is
    /// [`super::placement_align`] of the two, which is why a 248-byte `int[62]`
    /// takes ALIGN_8 (`0xC0401040`) where a 16-byte `int[4]` takes ALIGN_4
    /// (`0xC0301040`): both were read off c2's own obj.
    pub natural_align: u32,
    /// The initializer, in the obj's byte order, `bytes.len() == size`.
    pub bytes: &'a [u8],
    /// The `.text` byte offset of the `lis rS,sym@ha` that opens the reference.
    pub hi_off: u32,
    /// Every `.text` byte offset carrying a low half against this symbol, in
    /// ascending order. At least one.
    pub lo_offs: Vec<u32>,
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
    /// True iff this function is the reason the obj carries the undefined
    /// external **`memcpy`** — i.e. its body calls the block-copy intrinsic,
    /// which arrives in the IL as a `40` selector with no `.gl` record at all,
    /// so the name is minted by the emitter rather than resolved.
    ///
    /// Like [`Self::is_float`] this is a per-function fact that
    /// [`plan_labels`] turns into a **once-per-TU** label slot: the counter
    /// advances one extra time before the FIRST such function's `$M` triple and
    /// not at all for any later one. Measured, three cells at the workload's own
    /// flags (`work/w-ifn/probe/lab_x.cpp`, `lab_y.cpp`, `lab_z.cpp`):
    ///
    /// ```text
    ///   [framed, sub(memcpy)]                    stride 6   the slot
    ///   [framed, sub1(memcpy), sub2(memcpy),
    ///                          framed]           strides 6, 5, 5   once per TU
    ///   [sub(memcpy), framed]                    stride 5   INVISIBLE — the
    ///                                            slot is taken before the FIRST
    ///                                            function's own triple
    /// ```
    ///
    /// **The third row is the trap and it cost this lane a wrong `LABEL_LEAD.md`
    /// section.** A lead on the first function of a TU moves that function's own
    /// labels and every later one's *equally*, so every in-TU stride reads the
    /// plain framed constant and the surcharge is invisible. That is
    /// `w-blockir` board #2305's "cell that could not fail" in a mirror: there a
    /// wrong charge on the LAST function moved nothing after it, here a real
    /// charge on the FIRST function moves nothing measurable by strides. It was
    /// caught by the differential, not by the counterfactual.
    ///
    /// **Not unified with [`Self::helper_externals`], though the shape is the
    /// same** — `docs/LABEL_COUNTER.md` §1.1's `gpr3-dup` row measures that a
    /// second function reusing a `__savegprlr_N` width pays no surcharge and
    /// emits no second symbol, which is exactly this rule. They are two fields
    /// because W-XLR's surcharge is already charged per function through
    /// `c2_il::IlFunction::label_lead`, and folding it in here would count it
    /// twice.
    pub mints_memcpy: bool,
    /// W13b: this function's floating-point constant reference sites, in
    /// emission order, with `hi_off` already rebased to the whole `.text`.
    pub fp_refs: Vec<crate::codegen::FpConstRef>,
    /// **WR1**: this function's named-data-symbol address references, in emission
    /// order, with `hi_off` already rebased to the whole `.text`. At most one in
    /// the class the parser admits; a `Vec` because the relocation and symbol code
    /// below is written over a list either way and a "the" here would be the same
    /// constant [`Function::calls`]' own comment records having been.
    pub data_refs: Vec<DataRef<'a>>,
    /// **W-DATA**: the data objects this function DEFINES and references — a
    /// function-local `static`, which c2 gives its own COMDAT section placed
    /// **after** the code groups. See [`DataDef`].
    ///
    /// Empty for every function the port emitted before this field existed, and
    /// [`super::emit_obj`] (the packed `/Ox` writer) **refuses** a non-empty one
    /// rather than dropping it: the packed layout's section order and symbol
    /// indices were never measured with a COMDAT `.data` in them, and a dropped
    /// section is a wrong section count at file offset 2.
    pub data_defs: Vec<DataDef<'a>>,
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
    /// **W-XLR — undefined externals whose symbol records c2 places AFTER this
    /// function's `$T` label**, in emission order.
    ///
    /// Exactly one population today: the `__savegprlr_N` / `__restgprlr_N` pair
    /// of a Class C frame. They are unlike every name in
    /// [`Self::introduced_externals`] in two ways at once — the IL never names
    /// them (they are minted from the frame's `saved_gprs`), and c2 emits their
    /// records *after* the `.pdata` group rather than between the two `$M`s.
    /// `docs/CODEGEN_FRAMED_CALLS.md` §2.3a is the witnessed group:
    ///
    /// ```text
    ///   .text+aux · ?f1 · $M(end) · ?g · $M(prologue) · .pdata+aux · $T
    ///                                              · __restgprlr_29 · __savegprlr_29
    /// ```
    ///
    /// Their relocation sites still live in [`Self::calls`], because that is
    /// what `comdat::text_reloc_plan` reads and a REL24 is a REL24 — so the two
    /// lists overlap by name and [`Self::introduced_externals`] subtracts this
    /// one. Emitting them in the callee region instead resolves every
    /// relocation and moves four symbol indices, which is `docs/GAPS.md` §6's
    /// silent shape exactly.
    ///
    /// **The order is reverse first-reference**, the same rule
    /// `introduced_externals` applies, computed there rather than hardcoded as a
    /// pair: the save site is the prologue's and the restore site is the last
    /// word of the function, so the restore's symbol comes first.
    pub helper_externals: Vec<&'a str>,
}

impl<'a> Function<'a> {
    /// A function with no call, no constant pool and no frame — the common case.
    pub fn plain(name: &'a str, text_offset: u32) -> Function<'a> {
        Function {
            name,
            text_offset,
            calls: Vec::new(),
            is_float: false,
            mints_memcpy: false,
            fp_refs: Vec::new(),
            data_refs: Vec::new(),
            data_defs: Vec::new(),
            frame: None,
            label_lead: 0,
            helper_externals: Vec::new(),
        }
    }

    /// **The reverse first-reference LIFO, measured on callees alone.** Kept as
    /// the record behind [`Self::introduced_externals`]'s key, because it is a
    /// different measurement from GRID A's and predates it. The rule itself now
    /// lives in `introduced_externals`, which generalizes it to the union; the
    /// callees-then-data-names spelling it used to feed survives only as
    /// `external_order_tests::two_loop_order`, the refuted rival.
    ///
    /// (`docs/OBJ_GY_SHAPES.md` §3.3 as extended, byte evidence in
    /// `docs/CODEGEN_FRAMED_CALLS.md` §4.1.) `f(){ g1(); g2(); g3(); }` puts
    /// `?g3` at index 15, `?g2` at 16 and `?g1` at 17 — and the mirrored source
    /// `g3(); g2(); g1();` puts `?g1` at 15, which is what refutes both
    /// "alphabetical" and "declaration order". `g1(); g2(); g1();` emits two
    /// symbols, not three, and its repeat relocates against the first.
    ///
    /// This is the same LIFO the `.rdata` constant pool uses within one function
    /// (§2.3) and it has the same failure mode: a naive append emits every index
    /// swapped and **every relocation still resolves**, so the obj is wrong in a
    /// way no linker complains about.
    ///
    /// **W-UNDNAME / board #1720 — every undefined external this function
    /// introduces, as ONE list in reverse first-reference order, kind ignored.**
    ///
    /// This is GRID A's rule, and it replaces the two loops
    /// (a callee loop then a `data_refs` loop) that both writers used to
    /// run. `work/w-extdata/GRID_A_RESULT.md` — five one-function TUs, four
    /// rivals, per-cell predictions frozen and separation asserted before the
    /// first `cl.exe`:
    ///
    /// | cell | `.text` reference order | symbol table from index 15 |
    /// |---|---|---|
    /// | a1 | `gI g1 g0` | `g0 g1 gI` |
    /// | **a2** | `g0 gI g1` | **`g1 gI g0`** |
    /// | **a3** | `gI g1 g0 gJ g2` | **`g2 gJ g0 g1 gI`** |
    /// | **a4** | `gI g1 gJ g2` | **`g2 gJ g1 gI`** |
    /// | a5 | `g0 g3` | `g3 g0` |
    ///
    /// One list over callees ∪ data names in reverse first-reference order is
    /// confirmed 5 of 5; the two-loop rule is refuted on three, declaration order
    /// on four and `.gl` order on one. Read a second way as the grid required —
    /// by the relocation targeting each index — a3's sequence down `.text` is
    /// `REFHI(19) · REL24(18) · REL24(17) · REFHI(16) · REL24(15)`, strictly
    /// descending index against ascending offset for both kinds alike. So the key
    /// is the first-reference OFFSET and not the `Type` or the name.
    ///
    /// # Why this could not ship until now, and what it costs
    ///
    /// It converts nothing by itself, and until a body whose externals interleave
    /// was in class there was **no cell** that could tell it from the two loops —
    /// `docs/STATUS.md` trap 0 exactly. Measured at `w-undname`'s base: all four
    /// GRID A cells carrying a data symbol read `0/1 functions in class`. The
    /// cell that exercises this arm is `?append@DName@@QAAXPAVDNameNode@@@Z`,
    /// whose externals are `data · callee · data`, plus the fixture that
    /// reproduces it.
    ///
    /// **It is byte-neutral on every obj emitted before it**, and the argument is
    /// a proof rather than a hope: `crate::check_external_order` (deleted in the
    /// same commit) refused every body in which a data reference followed a call,
    /// so on all of them every data reference precedes every call — and reverse
    /// order over the union then places all callees first and all data names
    /// last, which is where the two loops put them. Measured anyway, three ways:
    /// the 878-TU scan at both ends, the gate's per-lane `match` counts, and the
    /// fixture-verdict total.
    ///
    /// The `bool` is `true` for a name whose symbol record is a FUNCTION's
    /// (`Type 0x0020`) — a callee, or a `DataRef` whose [`DataRef::is_function`]
    /// is set. A name occurring as both is one symbol and takes the FUNCTION
    /// record, which is the same record either way, so nothing is decided here
    /// that a cell has not seen.
    pub(crate) fn introduced_externals(&self) -> Vec<(&'a str, bool)> {
        // (name, first-reference offset, is a FUNCTION record)
        let mut first: Vec<(&'a str, u32, bool)> = Vec::new();
        // **W-XLR — the frame helpers are subtracted, not skipped.** They are in
        // `calls` because their relocations are ordinary REL24s; they are not in
        // this list because c2 puts their symbols after the `$T` label. One
        // filter, applied to the merged list, so a helper that were ever also an
        // IL-named callee could not appear twice.
        let mut note = |name: &'a str, off: u32, is_fn: bool| {
            match first.iter_mut().find(|(n, _, _)| *n == name) {
                Some(e) => {
                    e.1 = e.1.min(off);
                    e.2 |= is_fn;
                }
                None => first.push((name, off, is_fn)),
            }
        };
        for c in &self.calls {
            if self.helper_externals.contains(&c.callee) {
                continue;
            }
            note(c.callee, c.reloc_offset, true);
        }
        for r in &self.data_refs {
            note(r.name, r.hi_off, r.is_function);
        }
        // Descending first-reference offset. A stable sort, though the key is
        // unique by construction: two references at one `.text` offset would be
        // one instruction relocated twice.
        first.sort_by(|a, b| b.1.cmp(&a.1));
        first.into_iter().map(|(n, _, f)| (n, f)).collect()
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

#[cfg(test)]
mod external_order_tests {
    use super::*;

    /// **The REFUTED rival, kept as code so the separation is executable.**
    ///
    /// Callees in reverse first-reference order, then data names in reference
    /// order — the two loops both writers ran until board #1720 shipped. It
    /// lives here and not in the writer because nothing may emit it, and it is
    /// not deleted because a rival that is only described in a rung doc cannot
    /// be asserted against.
    fn two_loop_order<'a>(f: &Function<'a>) -> Vec<&'a str> {
        let mut first: Vec<&'a str> = Vec::new();
        for c in &f.calls {
            if !first.contains(&c.callee) {
                first.push(c.callee);
            }
        }
        first.reverse();
        first.extend(f.data_refs.iter().map(|r| r.name));
        first
    }

    /// **GRID A's rule, on GRID A's own `a3` shape** — `data · callee · data`
    /// down `.text`, which is `?append@DName@@QAAXPAVDNameNode@@@Z`'s shape and
    /// the one no ordering of a callee loop and a data loop can produce.
    ///
    /// `work/w-extdata/GRID_A_RESULT.md` a3: `.text` reference order
    /// `gI g1 g0 gJ g2` gives a symbol table `g2 gJ g0 g1 gI` from index 15 —
    /// strictly descending index against ascending first-reference offset, for
    /// both kinds alike.
    #[test]
    fn the_undefined_externals_are_one_list_in_reverse_first_reference_order() {
        let mut f = Function::plain("?a3@@YAXXZ", 0);
        f.data_refs = vec![
            DataRef { hi_off: 0x00, lo_off: 0x08, name: "?gI@@3HA", is_function: false },
            DataRef { hi_off: 0x18, lo_off: 0x20, name: "?gJ@@3HA", is_function: false },
        ];
        f.calls = vec![
            Call { reloc_offset: 0x0c, callee: "?g1@@YAXPAH@Z" },
            Call { reloc_offset: 0x10, callee: "?g0@@YAXXZ" },
            Call { reloc_offset: 0x24, callee: "?g2@@YAXPAH@Z" },
        ];
        assert_eq!(
            f.introduced_externals(),
            vec![
                ("?g2@@YAXPAH@Z", true),
                ("?gJ@@3HA", false),
                ("?g0@@YAXXZ", true),
                ("?g1@@YAXPAH@Z", true),
                ("?gI@@3HA", false),
            ]
        );
        // The refuted rival, spelled out so the separation is in the test and
        // not only in the rung doc: callees first, then data names, gives a
        // DIFFERENT list — and every relocation resolves either way.
        let two_loops = two_loop_order(&f);
        assert_eq!(two_loops, vec!["?g2@@YAXPAH@Z", "?g0@@YAXXZ", "?g1@@YAXPAH@Z", "?gI@@3HA", "?gJ@@3HA"]);
        assert_ne!(
            two_loops,
            f.introduced_externals().into_iter().map(|(n, _)| n).collect::<Vec<_>>()
        );
    }

    /// The two rules **coincide** whenever every data reference precedes every
    /// call — which is the population the deleted `check_external_order` fence
    /// admitted, and therefore the proof that shipping A1 changed no obj the
    /// port had ever emitted.
    #[test]
    fn a_body_whose_data_refs_all_precede_its_calls_gets_the_same_list_either_way() {
        let mut f = Function::plain("?wr1@@YAXXZ", 0);
        f.data_refs = vec![DataRef {
            hi_off: 0x00,
            lo_off: 0x08,
            name: "?gI@@3HA",
            is_function: false,
        }];
        f.calls = vec![
            Call { reloc_offset: 0x0c, callee: "?g1@@YAXPAH@Z" },
            Call { reloc_offset: 0x10, callee: "?g0@@YAXXZ" },
        ];
        let merged: Vec<&str> = f.introduced_externals().into_iter().map(|(n, _)| n).collect();
        assert_eq!(merged, two_loop_order(&f));
    }

    /// A repeated callee is ONE symbol at its FIRST reference's rank — the same
    /// dedup `introduced_callees` had, kept across the merge.
    #[test]
    fn a_callee_named_twice_takes_its_first_references_rank_once() {
        let mut f = Function::plain("?rep@@YAXXZ", 0);
        f.calls = vec![
            Call { reloc_offset: 0x00, callee: "?g1@@YAXXZ" },
            Call { reloc_offset: 0x04, callee: "?g2@@YAXXZ" },
            Call { reloc_offset: 0x08, callee: "?g1@@YAXXZ" },
        ];
        assert_eq!(
            f.introduced_externals(),
            vec![("?g2@@YAXXZ", true), ("?g1@@YAXXZ", true)]
        );
    }

    /// A REFHI/REFLO whose target is a FUNCTION carries `Type 0x0020`, and the
    /// merged list has to carry that bit — the symbol record is the only place
    /// the two kinds differ, and every relocation resolves either way
    /// (`DataRef::is_function`, W-EXTDATA).
    #[test]
    fn a_function_addresss_reference_keeps_its_function_symbol_type() {
        let mut f = Function::plain("?fa@@YAXXZ", 0);
        f.data_refs = vec![DataRef {
            hi_off: 0x38,
            lo_off: 0x48,
            name: "_woutput_s_l",
            is_function: true,
        }];
        f.calls = vec![Call { reloc_offset: 0x4c, callee: "_vswprintf_helper" }];
        assert_eq!(
            f.introduced_externals(),
            vec![("_vswprintf_helper", true), ("_woutput_s_l", true)]
        );
    }
}
