//! **THE CANDIDATE SET** — `globregs.c`'s promotion policy (`FUN_10b550e5`
//! gate A, `FUN_10bd7d24` gate B) and the one function that decides its input
//! (`FUN_10bd2913`), written as executable code with every decision it makes
//! exposed as a **named, settable parameter whose default reproduces the read**.
//!
//! Lane `w-globset`, wave 20 L3 (`docs/WAVE20_BRIEF_2026-08-29.md` §2).
//! Prereg `work/w-globset/PREREG.md`. Write-up
//! `docs/rungs/2026-08-29-w-globset.md`. Board **#3831**–**#3837**.
//!
//! # Why this module exists, in one sentence
//!
//! `[globregs]`'s **order** half is settled and obj-confirmed — definition
//! order, `[O]` on 42/42 cells with seven rivals refuted
//! (`docs/whitebox/ref/P_GLOBREGS.md` §7.1, board `#3774`). Its **set** half
//! was read by `w-globarms` (`#3808`–`#3810`) and was expressed nowhere. That
//! lane's own handoff names the shape this module has:
//!
//! > **the parameter to expose is linkage class, not variable kind. Kinds 4
//! > and 5 are linkage 1 and 3, the no-COFF-record classes; everything with a
//! > COFF record is 7/8/9. The escape flag `sym+0x05 & 2` is the second
//! > parameter and it is per-symbol, not per-function.**
//! > — `docs/whitebox/WB_GLOBARMS_FINDINGS.md` §7
//!
//! # The model, as read
//!
//! ```text
//!   .gl record  --FUN_10bd2913-->  kind byte  --gate A-->  eligible / reject
//!   [gl+0x30]       0x10bd2a1d      sym+0x04    0x10b5511a       |
//!   ([gl+0x37]                                                   v
//!    >>0x15)&7 ---> 8-entry jump table @ 0x10bd2a9f          gate B (type)
//!                                                            0x10b551d4
//! ```
//!
//! **Gate A is a statement about COFF linkage.** A6's kinds 4 and 5 are
//! linkage 1 and 3, which `P_SYMBOL.md` §3 reads at `0x10b28bb4` as *"a linkage
//! class that is suppressed outright"* — **no COFF record at all**. Every
//! symbol that does get a COFF record arrives at A8/A9 as kind 7, 8 or 9.
//!
//! # The five parameters
//!
//! `docs/rungs/README.md` § "Lane kinds", THE DECISION-SURFACE CLAUSE: a
//! general layer ships its arbitrary choices as named, enumerable parameters
//! whose default reproduces c2, because a named decision point serves the
//! permuter and the training pipeline at the same correctness cost as a baked
//! constant.
//!
//! | # | parameter | default | what it makes runnable |
//! |---|---|---|---|
//! | P1 | [`KindMap::table`] | the 8 arms at `0x10bd2a9f` | a table read at the wrong stride; entry 0 treated as reachable |
//! | P2 | [`KindMap::kind_for_gl2`] &c. | the `dec`-chain at `0x10bd2926` | — |
//! | P3 | [`GateA`]'s seven bounds | `0x10b5511a`…`0x10b5514e` | A6's bound `5` → `7`, the mutant the image grader itself plants |
//! | P4 | [`AliasingPolicy`] | [`AliasingPolicy::EscapesToOpaqueCallee`] | **`AddressTaken`, refuted by `gb_addr_local`**; `Never`; `Always` |
//! | P5 | [`TypeClassPolicy`] | the 30-byte table at `0x10b18b28` | `AllPromotable`, `NonePromotable`, a stride shift |
//!
//! **P4 is the brief's "A6 needs TWO parameters, not one".** The escape bit's
//! sole setter is `FUN_10bd2db7`, which walks the **leader's `+0x0c` chain** —
//! so it is a property of a symbol *group*, and [`SymbolGroup`] is what the
//! policy takes. And it is **not** "address taken": `gb_addr_local`'s
//! `int *q = &x;` with no escape is **PROMOTED**, which is why
//! [`AliasingPolicy::AddressTaken`] ships as a **refuted rival** rather than as
//! a plausible reading.
//!
//! # The four things this module is NOT
//!
//! * **It has no production caller, by construction.** The port has no symbol
//!   arena, no `.gl` records and no tuple list; [`crate::PortC2::build`] does
//!   not reach this module, no byte the judge grades can move, and no refusal
//!   consults it. That is the construct-rung corollary in `rungs/README.md`,
//!   and it is why the byte delta cannot carry this lane's grade —
//!   [`tests::the_fail_axis`] and the registered decision surface do.
//! * **It defines no `ported` numerator** — decision 21 §4, board `#3809`,
//!   `#3505`. Six of gate A's twelve arms are `CONSTR` for one shared
//!   structural reason (every rejecting arm branches to `0x10b552b8`), so a
//!   12-arm ratio carries a 6/12 ceiling on day one; and the arms are not
//!   equally weighted — A6 and A8 cover every symbol a C++ compiland declares
//!   while A1 covers **one record per compilation**. [`tests`] measures
//!   *separating power* over an existing obj population and publishes its
//!   zeros; that is not a coverage ratio and no percentage of c2 is claimed
//!   from it.
//! * **It is not a register allocator** (decision 20 §2, `P_REGALLOC` §7: F5
//!   is not separable from F0).
//! * **It is not a judge.** Nothing here is in `scripts/gate.sh`'s verdict and
//!   nothing here licenses an emit (`docs/FUNCTION_BYTE_MATCH.md` §0).
//!
//! # What is READ, what is OBSERVED, and what is INFERRED
//!
//! The distinction is load-bearing and the marks are the reference pages':
//!
//! * `[R]` — the kind map, the jump table, gate A's twelve arms and their
//!   addresses, gate B's 30-byte table. Read at the addresses cited per item.
//! * `[O]` — A6 and its internal escape test (`gb_pair_yescape` /
//!   `gb_pair_xescape` / `gb_pair_none` / `gb_addr_local` / `gb_addr_escape`,
//!   18 verdicts, 0 `U`, both profiles); A4 and A11's accept side (`ga_temp`,
//!   `ga_temp3`); A3's consequence (`ga_structmix`).
//! * `[I]` — **[`CandidateSet::verdict`]'s last step**, "in the aliasing set ⇒
//!   MEMORY at the observable". `DAT_10c2e3e8` membership is `[R]`; the frame
//!   traffic is `[O]`; the *arrow between them* is this module's inference and
//!   is marked as one wherever it is used. `WB_GLOBARMS_FINDINGS.md` §1.1 puts
//!   it the same way: *"`[O]` on the partition; `[I]` on calling the bit
//!   escape"*.
//!
//! PROV-BLOCK[R] DISCLOSURE `W-GLOBSET-1` — the kind map's jump table and its
//! five arm shapes, gate A's seven kind bounds, and gate B's not-promotable
//! class set, adopted as source literals. The read is `w-globarms`' and
//! `w-globobj`'s and was **not re-taken** by this lane.

/// The highest **type class** the 13-entry nibble table at `0x10bd7cf0` can
/// produce, and therefore the last index of gate B's table at `0x10b18b28`.
///
/// A class above this is not a value `FUN_10bd7c10` can return, so
/// [`TypeClassPolicy::promotable`] **refuses** rather than guessing — the
/// domain runs two classes past it on purpose, which is what makes naming this
/// a `guards` entry of the registered surface a claim that can be tested
/// (`#3746`: two of the registry's seven original guard entries were false and
/// moved zero domain lines).
///
/// PROV[R] DISCLOSURE `W-GLOBSET-1` — `P_GLOBREGS.md` §3 gate B: the top
/// nibble maps through the 13-entry table at `0x10bd7cf0`, *"each arm resolving
/// the low 12 bits to a **type class `0x00…0x1d`**"*.
pub const TYPE_CLASS_MAX: u8 = 0x1d;

// ---------------------------------------------------------------------------
// P1/P2 — the front end → back end kind map, `FUN_10bd2913`
// ---------------------------------------------------------------------------

/// The `.gl` record, reduced to **exactly** the five fields `FUN_10bd2913`
/// reads.
///
/// Deliberately not a model of c2's record: the port lays out no `.gl` record
/// and dereferences no offset. The offsets are carried as *field names* so the
/// disassembly and this struct can be read side by side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlRecord {
    /// `[gl+0x30]` — the front end's own kind byte. `P_SYMBOL.md` §1:
    /// **1 data, 3 function, 4 extern/alias**.
    pub kind_byte: u8,
    /// `([gl+0x37] >> 0x15) & 7` — **the 3-bit COFF linkage field**, and the
    /// index into [`KindMap::table`]. The same field `P_SYMBOL.md` §3 reads at
    /// `0x10b28bb4` to decide that linkage `∈ {1,3}` produces no COFF record.
    pub linkage: u8,
    /// Whether `([gl+0x37] & 0x1e0) == 0x80` — the test the linkage-2/6 arm
    /// makes.
    pub storage_bits_hit: bool,
    /// The storage kind the linkage-4/7 arm switches on: `1,2 → 7`, `4 → 8`,
    /// anything else `→ 9`.
    pub storage_kind: u8,
    /// `((gl+0x20) >> 4) & 2` — the bit the linkage-5 arm ORs into `5`,
    /// yielding kind 5 or kind 7.
    pub alias_bit: bool,
}

impl GlRecord {
    /// A data record (`[gl+0x30] == 1`) at the given linkage, with every other
    /// field at its quietest value.
    pub fn data(linkage: u8) -> GlRecord {
        GlRecord {
            kind_byte: 1,
            linkage,
            storage_bits_hit: false,
            storage_kind: 0,
            alias_bit: false,
        }
    }
}

/// One entry of the **8-entry jump table at `0x10bd2a9f`**, indexed by the
/// linkage field.
///
/// Five shapes, and the fact that there are five rather than eight is the
/// content of the read: two pairs of linkage values share an arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkageArm {
    /// **Entry 0 is a NULL slot.** Linkage 0 is unreachable by invariant and
    /// c2 would jump to address 0 if it ever arose
    /// (`WB_GLOBARMS_FINDINGS.md` §7). The port **refuses** rather than
    /// inventing a kind, which is the whole difference between modelling an
    /// invariant and papering over it.
    NullSlot,
    /// A fixed kind. Linkage 1 → 4, linkage 3 → 5 — **the two no-COFF-record
    /// classes, which are exactly A6's kinds.**
    Kind(u8),
    /// Linkage 2 and 6: kind `hit` when `([gl+0x37] & 0x1e0) == 0x80`, else
    /// `miss`.
    StorageBits {
        /// c2: 8.
        hit: u8,
        /// c2: 7.
        miss: u8,
    },
    /// Linkage 4 and 7: storage kind `1,2 → 7`, `4 → 8`, else `9`.
    StorageKind,
    /// Linkage 5: `((gl+0x20) >> 4) & 2 | 5` — kind 5 or kind 7.
    AliasBit,
}

/// What the kind map produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappedKind {
    /// A back-end kind byte, the value gate A tests as `sym+0x04`.
    Kind(u8),
    /// The null table slot. Not a kind; a state c2's own invariant says cannot
    /// arise.
    Unreachable,
}

/// **P1 + P2 — `FUN_10bd2913`, as a settable object.**
///
/// PROV[R] DISCLOSURE `W-GLOBSET-1` — the kind write is `0x10bd2a1d`, the
/// `dec`-chain `0x10bd2926`, the jump table `0x10bd2a9f`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindMap {
    /// **P1** — the jump table, indexed by [`GlRecord::linkage`].
    pub table: [LinkageArm; 8],
    /// **P2** — `[gl+0x30] == 2`. c2: kind 4.
    pub kind_for_gl2: u8,
    /// **P2** — `[gl+0x30] == 3`, a function. c2: kind `0xb`.
    pub kind_for_function: u8,
    /// **P2** — `[gl+0x30] == 4` (extern/alias) **and every other value**.
    /// c2: kind `0xa`.
    pub kind_for_extern: u8,
}

impl KindMap {
    /// **c2's map, and the default everywhere in this module.**
    ///
    /// PROV[R] DISCLOSURE `W-GLOBSET-1` — read at `0x10bd2a9f`;
    /// `WB_GLOBARMS_FINDINGS.md` §0 is the decode and
    /// `work/w-globarms/GRADE.txt` is its grader's own print-out of the same
    /// eight rows, which [`tests::the_fail_axis`] parses rather than trusts.
    pub const C2: KindMap = KindMap {
        table: [
            LinkageArm::NullSlot,                       // 0 — unreachable
            LinkageArm::Kind(4),                        // 1 — an auto
            LinkageArm::StorageBits { hit: 8, miss: 7 }, // 2
            LinkageArm::Kind(5),                        // 3 — an auto
            LinkageArm::StorageKind,                    // 4
            LinkageArm::AliasBit,                       // 5
            LinkageArm::StorageBits { hit: 8, miss: 7 }, // 6 = 2
            LinkageArm::StorageKind,                    // 7 = 4
        ],
        kind_for_gl2: 4,
        kind_for_function: 0xb,
        kind_for_extern: 0xa,
    };

    /// `FUN_10bd2913` itself.
    pub fn kind_of(&self, gl: &GlRecord) -> MappedKind {
        match gl.kind_byte {
            1 => self.linkage_kind(gl),
            2 => MappedKind::Kind(self.kind_for_gl2),
            3 => MappedKind::Kind(self.kind_for_function),
            _ => MappedKind::Kind(self.kind_for_extern),
        }
    }

    /// The table arm alone, for a data record.
    pub fn linkage_kind(&self, gl: &GlRecord) -> MappedKind {
        match self.table[(gl.linkage & 7) as usize] {
            LinkageArm::NullSlot => MappedKind::Unreachable,
            LinkageArm::Kind(k) => MappedKind::Kind(k),
            LinkageArm::StorageBits { hit, miss } => {
                MappedKind::Kind(if gl.storage_bits_hit { hit } else { miss })
            }
            LinkageArm::StorageKind => MappedKind::Kind(match gl.storage_kind {
                1 | 2 => 7,
                4 => 8,
                _ => 9,
            }),
            LinkageArm::AliasBit => MappedKind::Kind(if gl.alias_bit { 7 } else { 5 }),
        }
    }

    /// **Does a symbol of this linkage reach the object file at all?**
    ///
    /// `P_SYMBOL.md` §3: linkage `∈ {1,3}` is suppressed outright. Composed
    /// with the table above, that is the one-sentence model of gate A —
    /// [`tests::a6s_kinds_are_exactly_the_linkage_classes_with_no_coff_record`]
    /// is that sentence as an executed claim rather than a quoted one.
    pub fn has_coff_record(&self, linkage: u8) -> bool {
        !matches!(linkage & 7, 1 | 3)
    }
}

impl Default for KindMap {
    fn default() -> Self {
        KindMap::C2
    }
}

// ---------------------------------------------------------------------------
// P4 — the escape bit, `sym+0x05 & 2`
// ---------------------------------------------------------------------------

/// A **symbol group** — a leader and the `+0x0c` chain hanging off it.
///
/// The escape bit's sole setter is `FUN_10bd2db7`, **which walks the leader's
/// `+0x0c` chain**, so the property is a group's and not a symbol's. Gate A's
/// A3 arm is the other half of the same fact: only a group leader is
/// considered, and a member is reached through the leader's chain instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SymbolGroup {
    /// The group's address is taken somewhere in the function — `int *q = &x;`.
    pub address_taken: bool,
    /// The address **escapes to an opaque callee**. This is the one that moves
    /// the map: `gb_addr_local` takes an address without escaping and is
    /// **PROMOTED**; `gb_addr_escape` escapes and is **MEMORY**.
    pub address_escapes: bool,
}

/// **P4 — what sets `sym+0x05 & 2`.**
///
/// The default is `[O]` on the partition and `[I]` on the name
/// (`WB_GLOBARMS_FINDINGS.md` §1.1). The other three arms are instrument
/// states; [`AliasingPolicy::AddressTaken`] in particular is a **refuted**
/// reading and is kept because a rival that has been run is worth more than a
/// rival that has been asserted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasingPolicy {
    /// **c2's.** The bit is set when the group's address escapes to an opaque
    /// callee.
    EscapesToOpaqueCallee,
    /// **REFUTED** by `gb_addr_local` at both profiles: `int x = p[0]; int *q =
    /// &x; … return *q;` takes the address, does not escape it, and is
    /// PROMOTED. Kept as the rival it is.
    AddressTaken,
    /// Nothing ever joins the aliasing set. Refuted by `ga_escape` /
    /// `gb_addr_escape`.
    Never,
    /// Everything joins it. Refuted by `ga_int` / `gb_pair_none`.
    Always,
}

impl AliasingPolicy {
    /// Whether this policy sets the bit for the given group.
    pub fn escape_bit(&self, g: &SymbolGroup) -> bool {
        match self {
            AliasingPolicy::EscapesToOpaqueCallee => g.address_escapes,
            AliasingPolicy::AddressTaken => g.address_taken,
            AliasingPolicy::Never => false,
            AliasingPolicy::Always => true,
        }
    }
}

impl Default for AliasingPolicy {
    fn default() -> Self {
        AliasingPolicy::EscapesToOpaqueCallee
    }
}

// ---------------------------------------------------------------------------
// P5 — gate B, `FUN_10bd7d24`
// ---------------------------------------------------------------------------

/// **P5 — gate B's 30-byte promotability table at `0x10b18b28`.**
///
/// PROV[R] DISCLOSURE `W-GLOBSET-1` — `P_GLOBREGS.md` §3: *"Not promotable:
/// classes `0x00`, `0x12`, `0x13`, `0x18`, `0x1d`. The other 25 are."*
/// Independently re-derived out of the pinned image by
/// `docs/whitebox/scripts/grade_globobj.py`, whose answer-key line
/// [`tests::the_fail_axis`] parses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeClassPolicy {
    /// The classes the table marks not promotable.
    pub not_promotable: &'static [u8],
    /// The back-end kinds that reach gate B **at all**.
    ///
    /// `WB_GLOBARMS_FINDINGS.md` §2.2 refuted `P_GLOBREGS` §3's "gate A, then
    /// gate B" sequencing: **kind 10 never reaches gate B.** The A10 path
    /// jumps straight to `0x10b55295` and `t+0x20 == 4` is kind 10's
    /// *substitute* for the type gate, not an addition.
    pub kinds_reaching_gate_b: &'static [u8],
}

impl TypeClassPolicy {
    /// **c2's table.**
    pub const C2: TypeClassPolicy = TypeClassPolicy {
        not_promotable: &[0x00, 0x12, 0x13, 0x18, 0x1d],
        kinds_reaching_gate_b: &[3, 4, 5, 7, 8],
    };

    /// Every class promotable — an instrument state, and the widening the
    /// registered surface exists to make loud.
    pub const ALL_PROMOTABLE: TypeClassPolicy =
        TypeClassPolicy { not_promotable: &[], kinds_reaching_gate_b: &[3, 4, 5, 7, 8] };

    /// A **one-byte stride shift** of the table — the exact defect
    /// `grade_globobj.py`'s planted control 1 makes, kept executable here.
    pub const STRIDE_SHIFTED: TypeClassPolicy = TypeClassPolicy {
        not_promotable: &[0x01, 0x13, 0x14, 0x19, 0x1e],
        kinds_reaching_gate_b: &[3, 4, 5, 7, 8],
    };

    /// Whether a class is promotable. A class above [`TYPE_CLASS_MAX`] is not
    /// a value the nibble table can produce, and is **refused**.
    pub fn promotable(&self, class: u8) -> Option<bool> {
        if class > TYPE_CLASS_MAX {
            return None;
        }
        Some(!self.not_promotable.contains(&class))
    }

    /// Whether this kind is type-gated at all.
    pub fn reaches_gate_b(&self, kind: u8) -> bool {
        self.kinds_reaching_gate_b.contains(&kind)
    }
}

impl Default for TypeClassPolicy {
    fn default() -> Self {
        TypeClassPolicy::C2
    }
}

// ---------------------------------------------------------------------------
// P3 — gate A, `FUN_10b550e5`
// ---------------------------------------------------------------------------

/// Gate A's twelve arms, by the names `w-globarms` gave them.
///
/// The names are this repo's, not c2's; the **addresses** are c2's and are on
/// each variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    /// `0x10b5511a` — `sym+0x04 == 0x10`, the symbol table's **sentinel**
    /// (minted once per compilation at `0x10bd339c`). Skips **without**
    /// running the reject tail.
    A1,
    /// `0x10b55129` — `sym+0x08 != sym`: only a group **leader** is considered.
    A3,
    /// `0x10b55134` — kind `== 3`, a compiler-generated temporary, dispatched
    /// to A11/A12.
    A4,
    /// `0x10b55138` — kind `< 3`: reject. Kind 1 is a physical register and
    /// kind 2 is the candidate record itself.
    A5,
    /// `0x10b5513e` — kind `∈ {4,5}`: **the autos**, eligible.
    A6,
    /// `0x10b55142` — kind `== 6`: reject. A by-name runtime symbol.
    A7,
    /// `0x10b5514a` — kind `∈ {7,8}`: eligible, and **always** joins the
    /// aliasing set.
    A8,
    /// `0x10b5514e` — kind `!= 10`: reject. Covers kind 9 and `0xb`…`0xf`,
    /// and `0xb` is a **function**.
    A9,
    /// `0x10b55156`–`0x10b5516b` — kind 10 needs `*(sym)+0x37 & 0x400` set and
    /// `& 0x200000` clear.
    A10,
    /// `0x10b551b3` — a kind-3 temporary needs `sym+0x14 == 0`.
    A11,
    /// `0x10b551bc` — …and `sym+0x07 & 0x40` clear.
    A12,
}

impl Arm {
    /// The arm's name, as `w-globarms`' `ARMS.tsv` and `GRADE.txt` spell it.
    pub fn name(&self) -> &'static str {
        match self {
            Arm::A1 => "A1",
            Arm::A3 => "A3",
            Arm::A4 => "A4",
            Arm::A5 => "A5",
            Arm::A6 => "A6",
            Arm::A7 => "A7",
            Arm::A8 => "A8",
            Arm::A9 => "A9",
            Arm::A10 => "A10",
            Arm::A11 => "A11",
            Arm::A12 => "A12",
        }
    }

    /// The address the arm is read at.
    ///
    /// PROV[R] DISCLOSURE `W-GLOBSET-1` — `P_GLOBREGS.md` §3's table, decoded
    /// out of the image by `docs/whitebox/scripts/grade_globarms.py`.
    pub fn addr(&self) -> u32 {
        match self {
            Arm::A1 => 0x10b5511a,
            Arm::A3 => 0x10b55129,
            Arm::A4 => 0x10b55134,
            Arm::A5 => 0x10b55138,
            Arm::A6 => 0x10b5513e,
            Arm::A7 => 0x10b55142,
            Arm::A8 => 0x10b5514a,
            Arm::A9 => 0x10b5514e,
            Arm::A10 => 0x10b55156,
            Arm::A11 => 0x10b551b3,
            Arm::A12 => 0x10b551bc,
        }
    }
}

/// What gate A did with a slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A1/A3 — the slot is skipped, **and the reject tail does not run**.
    /// The distinction is real in the binary and, `w-globarms` §3.3 measured,
    /// **unobservable in an obj**: the reject tail's counter `DAT_10c2e454` is
    /// written and never read, so nothing downstream can tell a silent skip
    /// from a charged reject.
    Skip,
    /// The reject tail at `0x10b552b8`: bump `DAT_10c2e454`, clear
    /// `+0x34`/`+0x38` on every sub-symbol.
    Reject,
    /// Eligible, and **not** in the `DAT_10c2e3e8` aliasing set.
    Eligible,
    /// Eligible, and in the aliasing set.
    EligibleAliased,
    /// A10's accept side: kind 10 indexes only its **sub-symbols**, and it
    /// never reaches gate B (§2.2).
    IndexSubSymbols,
}

impl Outcome {
    /// Does the symbol survive gate A as a candidate?
    pub fn is_eligible(&self) -> bool {
        matches!(self, Outcome::Eligible | Outcome::EligibleAliased | Outcome::IndexSubSymbols)
    }
}

/// **P3 — gate A's seven kind bounds, as a settable object.**
///
/// Every field is one `cmp` in `FUN_10b550e5`. The names say what the value
/// means; the defaults say what c2 compares against.
///
/// PROV[R] DISCLOSURE `W-GLOBSET-1` — decoded from the image at
/// `0x10b5511a`–`0x10b551c6`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GateA {
    /// A1 — the sentinel kind that is skipped without the reject tail.
    pub sentinel_kind: u8,
    /// A4 — the temporary kind that dispatches to A11/A12.
    pub temp_kind: u8,
    /// A5 — kinds `<=` this are rejected.
    pub reject_at_or_below: u8,
    /// A6 — kinds `<=` this (and above [`Self::reject_at_or_below`]) are the
    /// **autos**. Moving this from 5 to 7 is the mutant
    /// `grade_globarms.py --selftest` plants in the image itself.
    pub auto_at_or_below: u8,
    /// A7 — kinds `<=` this are rejected.
    pub reject2_at_or_below: u8,
    /// A8 — kinds `<=` this are eligible **and always aliased**.
    pub coff_at_or_below: u8,
    /// A9 — the only kind above [`Self::coff_at_or_below`] that is not
    /// rejected.
    pub extern_kind: u8,
}

impl GateA {
    /// **c2's bounds, and the default everywhere in this module.**
    pub const C2: GateA = GateA {
        sentinel_kind: 0x10,
        temp_kind: 3,
        reject_at_or_below: 3,
        auto_at_or_below: 5,
        reject2_at_or_below: 6,
        coff_at_or_below: 8,
        extern_kind: 0xa,
    };

    /// A6's kind bound, moved. `grade_globarms.py --selftest` plants exactly
    /// this in the **image** — patching `cmp al,5` to `cmp al,7` at
    /// `0x10b5513f` moves kinds 6 and 7 into A6 — and asserts that its own
    /// kind→arm map follows. This is that mutation on this side of the seam.
    ///
    /// **A function rather than a `GateA::A6_BOUND_7` constant, and the reason
    /// is a finding about the registry's own screen.** `surface.rs`'s E4 finds
    /// boundary constants **by name**, so a `const` called `A6_BOUND_7` trips
    /// it — correctly, on the letter — while the boundary this module actually
    /// carries, the field [`GateA::auto_at_or_below`], is invisible to it
    /// because a struct field is not a `const`. E4's own doc says it is *"a
    /// ratchet on a hole, not a proof that the hole is closed"*; **every one of
    /// this module's five parameters is a field**, so the hole is this module's
    /// whole surface and [`crate::surface`]'s registered domain is what covers
    /// it instead.
    pub fn with_auto_bound(self, bound: u8) -> GateA {
        GateA { auto_at_or_below: bound, ..self }
    }
}

impl Default for GateA {
    fn default() -> Self {
        GateA::C2
    }
}

/// A symbol slot, reduced to **exactly** the fields gate A tests.
///
/// Not a model of c2's `0x60`-stride record: the port lays out no symbol and
/// dereferences no offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Symbol {
    /// `sym+0x04` — the kind byte [`KindMap`] produced.
    ///
    /// **Note the collision** `WB_GLOBARMS_FINDINGS.md` §2.5 records: `+0x04`
    /// of the *`.gl`* record is a name pointer, `+0x04` of the *globregs*
    /// record is this kind byte. Two record types, one offset.
    pub kind: u8,
    /// A3 — `sym+0x08 == sym`, i.e. this slot is its group's **leader**.
    pub is_group_leader: bool,
    /// A11 — `sym+0x14 == 0`, on a kind-3 temporary.
    pub temp_slot_clear: bool,
    /// A12 — `sym+0x07 & 0x40` **clear**, on a kind-3 temporary.
    pub temp_flag_clear: bool,
    /// A10 — `*(sym)+0x37 & 0x400` set **and** `& 0x200000` clear, on a
    /// kind-10 extern.
    pub extern_indexable: bool,
    /// `sym+0x05 & 2` — the aliasing bit [`AliasingPolicy`] decides.
    pub escaped: bool,
    /// The gate-B type class of the symbol's type word (`sym+0x10`, resolved
    /// through `0x10bd7c10`).
    pub type_class: u8,
}

impl Symbol {
    /// A leader of the given kind with every conditional bit at the value that
    /// lets the arm through, which is the configuration
    /// `grade_globarms.py`'s kind→arm simulation prints.
    pub fn leader(kind: u8) -> Symbol {
        Symbol {
            kind,
            is_group_leader: true,
            temp_slot_clear: true,
            temp_flag_clear: true,
            extern_indexable: true,
            escaped: false,
            type_class: 0x01,
        }
    }
}

/// The end-to-end verdict: is the symbol a register candidate, and if not, why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// A candidate. At the observable this is *no frame traffic*.
    Promoted,
    /// Not a candidate, with the reason named. The reason is the port's, not
    /// c2's — c2 produces the identical `sym+0x34 = 0` for every one of them,
    /// which is why six of the twelve arms are `CONSTR`.
    NotPromoted(&'static str),
}

/// **The candidate-set policy** — the five parameters, bound together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CandidateSet {
    /// P1 + P2.
    pub kinds: KindMap,
    /// P3.
    pub gate_a: GateA,
    /// P4.
    pub aliasing: AliasingPolicy,
    /// P5.
    pub gate_b: TypeClassPolicy,
}

impl CandidateSet {
    /// **c2's policy**: every parameter at the value the read gives it.
    pub const C2: CandidateSet = CandidateSet {
        kinds: KindMap::C2,
        gate_a: GateA::C2,
        aliasing: AliasingPolicy::EscapesToOpaqueCallee,
        gate_b: TypeClassPolicy::C2,
    };

    /// **Gate A** — `FUN_10b550e5`'s twelve arms, in the order the binary
    /// tests them.
    ///
    /// The order is `CONSTR`: a gate-A rejection and a gate-B rejection
    /// produce the identical `sym+0x34 = 0`, so no obj can separate the arms'
    /// sequence (`P_GLOBREGS` §3's "Registered, but deliberately NOT
    /// upgraded"). It is written in the read order anyway, because that is
    /// what the disassembly says and a reader comparing the two should not
    /// have to re-derive it.
    pub fn gate_a(&self, sym: &Symbol) -> (Arm, Outcome) {
        let g = &self.gate_a;
        // A1 — the sentinel. Skips, and the reject tail does NOT run.
        if sym.kind == g.sentinel_kind {
            return (Arm::A1, Outcome::Skip);
        }
        // A2 is `sym+0x40 &= ~1`, unconditional, and chooses nothing. It is
        // not modelled and its absence is the point: no body exists in which
        // it does not happen, so nothing here can depend on it.
        //
        // A3 — only a group leader is considered.
        if !sym.is_group_leader {
            return (Arm::A3, Outcome::Skip);
        }
        // A4 — a temporary, dispatched forward to A11/A12.
        if sym.kind == g.temp_kind {
            if !sym.temp_slot_clear {
                return (Arm::A11, Outcome::Reject);
            }
            if !sym.temp_flag_clear {
                return (Arm::A12, Outcome::Reject);
            }
            return (Arm::A11, Outcome::Eligible);
        }
        // A5.
        if sym.kind <= g.reject_at_or_below {
            return (Arm::A5, Outcome::Reject);
        }
        // A6 — the autos, and the one arm whose internal test is `[O]`.
        if sym.kind <= g.auto_at_or_below {
            return (
                Arm::A6,
                if sym.escaped { Outcome::EligibleAliased } else { Outcome::Eligible },
            );
        }
        // A7.
        if sym.kind <= g.reject2_at_or_below {
            return (Arm::A7, Outcome::Reject);
        }
        // A8 — a COFF-record symbol, ALWAYS aliased.
        if sym.kind <= g.coff_at_or_below {
            return (Arm::A8, Outcome::EligibleAliased);
        }
        // A9.
        if sym.kind != g.extern_kind {
            return (Arm::A9, Outcome::Reject);
        }
        // A10.
        if !sym.extern_indexable {
            return (Arm::A10, Outcome::Reject);
        }
        (Arm::A10, Outcome::IndexSubSymbols)
    }

    /// **Gate B** — `FUN_10bd7d24` at `0x10b551d4`, for the kinds that reach
    /// it.
    ///
    /// `None` means the class is above [`TYPE_CLASS_MAX`] and the port
    /// refuses.
    pub fn gate_b(&self, sym: &Symbol) -> Option<bool> {
        if !self.gate_b.reaches_gate_b(sym.kind) {
            // Kind 10 substitutes `t+0x20 == 4` for the type gate (§2.2), and
            // nothing else gets here.
            return Some(true);
        }
        self.gate_b.promotable(sym.type_class)
    }

    /// The composition: **does this symbol become a register candidate?**
    ///
    /// The last step — *"in the aliasing set ⇒ MEMORY at the observable"* — is
    /// `[I]`, and it is the only inference in this module.
    /// `DAT_10c2e3e8` membership is `[R]` (`0x10b5513e`, `0x10b5514a`); the
    /// frame traffic is `[O]` (`gb_pair_yescape` against `gb_pair_xescape`,
    /// 18 verdicts, both profiles); the **arrow between them** is this
    /// module's, and a lane that reads a stronger mark off it has read it
    /// wrong.
    pub fn verdict(&self, sym: &Symbol) -> Verdict {
        let (_arm, outcome) = self.gate_a(sym);
        match outcome {
            Outcome::Skip => Verdict::NotPromoted("gate A skipped the slot"),
            Outcome::Reject => Verdict::NotPromoted("gate A rejected the slot"),
            // `[I]` — the aliasing set is the memory set at the observable.
            Outcome::EligibleAliased => Verdict::NotPromoted("in the aliasing set [I]"),
            Outcome::Eligible | Outcome::IndexSubSymbols => match self.gate_b(sym) {
                None => Verdict::NotPromoted("type class above TYPE_CLASS_MAX"),
                Some(false) => Verdict::NotPromoted("gate B: class not promotable"),
                Some(true) => Verdict::Promoted,
            },
        }
    }

    /// The whole pipeline from a front-end record: map the kind, set the
    /// escape bit from the **group**, run both gates.
    ///
    /// `None` when the kind map hit the null table slot — linkage 0, which c2's
    /// own invariant says cannot arise and which this port refuses rather than
    /// invents a kind for.
    pub fn verdict_for(
        &self,
        gl: &GlRecord,
        group: &SymbolGroup,
        type_class: u8,
    ) -> Option<Verdict> {
        let kind = match self.kinds.kind_of(gl) {
            MappedKind::Unreachable => return None,
            MappedKind::Kind(k) => k,
        };
        let mut sym = Symbol::leader(kind);
        sym.escaped = self.aliasing.escape_bit(group);
        sym.type_class = type_class;
        Some(self.verdict(&sym))
    }
}

// ---------------------------------------------------------------------------
// The registered decision surface
// ---------------------------------------------------------------------------

/// The `.gl` kind bytes the domain walks: the three the `dec`-chain names, plus
/// `0` and `7` for the "anything else" arm.
///
/// PROV[N] an instrument domain; reaches no emitted byte.
const SURFACE_GL_KINDS: [u8; 5] = [1, 2, 3, 4, 7];

/// The storage kinds the linkage-4/7 arm switches on, plus one that falls
/// through to kind 9.
///
/// PROV[N] an instrument domain; reaches no emitted byte.
const SURFACE_STORAGE_KINDS: [u8; 4] = [1, 2, 4, 9];

/// **SURFACE[globregs.candidate_set]** — the registered decision surface's
/// domain (`crate::surface`, board **#3723**).
///
/// Four blocks, and the reason there are four is that the policy has four
/// places a widening can be spelled: the kind map, gate A, gate B, and the
/// composition. Every point here is **past what the corpus reaches** — this
/// module has no production caller at all, so no fixture, no gate row and no
/// identity-diff line can reach a single one of them. That is exactly the
/// condition `#3723` measured a required-zero byte delta to be blind to.
pub fn surface_rows() -> Vec<crate::surface::Row> {
    let p = CandidateSet::C2;
    let mut rows = Vec::new();

    // -- block K: the kind map, FUN_10bd2913 --------------------------------
    for gl_kind in SURFACE_GL_KINDS {
        for linkage in 0..8u8 {
            for bits in [false, true] {
                for storage_kind in SURFACE_STORAGE_KINDS {
                    for alias in [false, true] {
                        let gl = GlRecord {
                            kind_byte: gl_kind,
                            linkage,
                            storage_bits_hit: bits,
                            storage_kind,
                            alias_bit: alias,
                        };
                        let outcome = match p.kinds.kind_of(&gl) {
                            MappedKind::Unreachable => {
                                format!("{} linkage-0-null-slot", crate::surface::REFUSE)
                            }
                            MappedKind::Kind(k) => format!(
                                "kind=0x{k:02x},{}",
                                if p.kinds.has_coff_record(linkage) { "coff" } else { "no-coff" }
                            ),
                        };
                        rows.push(crate::surface::Row::new(
                            format!(
                                "kindmap  gl={gl_kind} link={linkage} sbits={} skind={storage_kind} alias={}",
                                bits as u8, alias as u8
                            ),
                            outcome,
                        ));
                    }
                }
            }
        }
    }

    // -- block A: gate A, FUN_10b550e5 --------------------------------------
    // Kinds run one past the sentinel: 0x11 is not a value c2 mints and the
    // domain says what the port would do with it anyway.
    for kind in 0..=0x11u8 {
        for leader in [false, true] {
            for slot_clear in [false, true] {
                for flag_clear in [false, true] {
                    for indexable in [false, true] {
                        for escaped in [false, true] {
                            let sym = Symbol {
                                kind,
                                is_group_leader: leader,
                                temp_slot_clear: slot_clear,
                                temp_flag_clear: flag_clear,
                                extern_indexable: indexable,
                                escaped,
                                type_class: 0x01,
                            };
                            let (arm, outcome) = p.gate_a(&sym);
                            let text = match outcome {
                                Outcome::Skip => {
                                    format!("{} {}-skip", crate::surface::REFUSE, arm.name())
                                }
                                Outcome::Reject => {
                                    format!("{} {}-reject-tail", crate::surface::REFUSE, arm.name())
                                }
                                Outcome::Eligible => format!("{},ELIGIBLE", arm.name()),
                                Outcome::EligibleAliased => {
                                    format!("{},ELIGIBLE-ALIASED", arm.name())
                                }
                                Outcome::IndexSubSymbols => {
                                    format!("{},INDEX-SUB-SYMBOLS", arm.name())
                                }
                            };
                            rows.push(crate::surface::Row::new(
                                format!(
                                    "gateA    kind=0x{kind:02x} leader={} slot={} flag={} idx={} esc={}",
                                    leader as u8,
                                    slot_clear as u8,
                                    flag_clear as u8,
                                    indexable as u8,
                                    escaped as u8
                                ),
                                text,
                            ));
                        }
                    }
                }
            }
        }
    }

    // -- block B: gate B, FUN_10bd7d24 --------------------------------------
    // Classes run TWO past TYPE_CLASS_MAX, which is what makes naming it a
    // `guards` entry a claim the control set can test.
    for kind in [3u8, 4, 5, 7, 8, 0x0a] {
        for class in 0..=(TYPE_CLASS_MAX + 2) {
            let outcome = if !p.gate_b.reaches_gate_b(kind) {
                "NOT-TYPE-GATED".to_string()
            } else {
                match p.gate_b.promotable(class) {
                    None => format!("{} class-above-TYPE_CLASS_MAX", crate::surface::REFUSE),
                    Some(false) => format!("{} gate-B-not-promotable", crate::surface::REFUSE),
                    Some(true) => "PROMOTABLE".to_string(),
                }
            };
            rows.push(crate::surface::Row::new(
                format!("gateB    kind=0x{kind:02x} class=0x{class:02x}"),
                outcome,
            ));
        }
    }

    // -- block V: the composition ------------------------------------------
    for linkage in 0..8u8 {
        for class in 0..=(TYPE_CLASS_MAX + 2) {
            for escapes in [false, true] {
                let gl = GlRecord::data(linkage);
                let group = SymbolGroup { address_taken: escapes, address_escapes: escapes };
                let outcome = match p.verdict_for(&gl, &group, class) {
                    None => format!("{} linkage-0-null-slot", crate::surface::REFUSE),
                    Some(Verdict::Promoted) => "PROMOTED".to_string(),
                    Some(Verdict::NotPromoted(why)) => {
                        format!("{} {}", crate::surface::REFUSE, why.replace(' ', "-"))
                    }
                };
                rows.push(crate::surface::Row::new(
                    format!(
                        "verdict  link={linkage} class=0x{class:02x} esc={}",
                        escapes as u8
                    ),
                    outcome,
                ));
            }
        }
    }

    rows
}

#[cfg(test)]
mod tests;
