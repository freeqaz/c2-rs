//! **Proof of legibility for the opaque middle's two interfaces** — lane
//! `w-ildecode`, boards **#3357**–**#3360**.
//!
//! Findings doc: `docs/whitebox/WB_MIDDLE_INTERFACES.md`.
//! Registered predictions: `docs/whitebox/WB_MIDDLE_PREREG.md`.
//!
//! # What this file is, and what it is NOT
//!
//! `docs/ARCH_REVIEW_2026-08-21.md` finding 3 priced two unbudgeted
//! prerequisites at 3–9 engineer-months: a general op-level IL decode and a
//! general lowering to `coff::Function`. **This file is neither.** It is the
//! smallest runnable thing that shows the two interfaces are *legible* — that
//! the documented field correspondence is a fact about c2 and not an
//! assertion. Every claim below is graded by OUTPUT equality (live tap rows,
//! real obj bytes); nothing here compares c2's own instruction bytes, and the
//! port stays I/O-behavioral.
//!
//! | test | grades |
//! |---|---|
//! | [`the_opcode_space_is_c2s_own_mnemonic_table`] | **Interface 0.** Every real-instruction tuple's `+0x4` indexes `0x10b1b260`; every structural tuple's does not |
//! | [`the_il_subset_decoder_reproduces_the_tuple_rows`] | **Interface 1.** IL record → tuple, row-for-row, on the closed subset |
//! | [`the_final_tuple_order_reproduces_the_text_words`] | **Interface 2.** Final tuple order + operand window → the real `.text` COMDAT bytes, 32 bits of 32 |
//! | [`the_operand_window_never_moves_the_obj`] | the new tap field's own required-zero |
//!
//! # Why these fixtures
//!
//! `mvp_add3.cpp` (one function, `a+b+c`, three words) is the traced worked
//! example, and `mvp_two.cpp` (`add2` = one `add`, `add4` = three) is the
//! *predictive* witness: the interface-1 rule says an n-leaf pure additive
//! chain becomes n−1 machine `add` tuples, and 1 and 3 are two counts the
//! transcription of `mvp_add3`'s 2 cannot produce. Both are `Port=Match`
//! (byte-exact) today, so the lane's own subject matter is inside the shipped
//! class and the required-zero cannot be moved by anything here.
//!
//! `crates/c2-reference/tests/stage.rs` BANS `add3.cpp` as its control, for a
//! reason that does not apply here and is worth stating so the two files do not
//! look contradictory: there, a call-free single-region fixture would make a
//! zero region count look like a property of the mechanism. Here the region
//! count is not the observable — the *tuple rows* are, and a trivially small
//! function is precisely what makes a row-for-row identity readable by a human.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use c2_obj::{ObjDiff, ObjImage};
use c2_reference::stage::STAGE_SITES;
use c2_reference::Toolchain;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// The workload's own profile is `/O1 /Oi /EHsc /GS- /c`; this lane uses the
/// **capture default** `/Ox /GS- /c` instead, and deliberately, because that is
/// the profile under which `c2rs diff` reports `Port=Match` on both fixtures.
/// Grading the tuple stream at one profile and the byte-exactness at another
/// would be two measurements presented as one.
const FLAGS: [&str; 3] = ["/Ox", "/GS-", "/c"];

// ---------------------------------------------------------------------------
// The three per-opcode tables, read out of the pinned image at run time.
//
// Read at RUN TIME rather than transcribed into this file on purpose: the only
// things this test adopts from the disassembly are the three table addresses
// and their strides (DISCLOSURE.md rows W-MID-1..W-MID-3), never 660 table
// entries. A transcription would also be a snapshot that could silently rot
// against a different c2.dll; a live read cannot.
// ---------------------------------------------------------------------------

/// Mnemonic table: stride 12, `[+0] char* name`, `[+4] operand format`.
/// `P_DAG.md` §2.1 named it; this lane pins its base and its `_last` sentinel.
const MNEMONIC_TABLE: u32 = 0x10b1_b260;
const MNEMONIC_STRIDE: u32 = 12;
/// Index of the `_last` sentinel — the last machine opcode is `0x294`.
const MNEMONIC_LAST: u32 = 0x295;
/// Base-encoding table: stride 4, one 32-bit PPC word per machine opcode.
/// Sole reader is the encoder `FUN_10bf9f15` @ `0x10bf9f3c`.
const BASE_WORD_TABLE: u32 = 0x10c3_a578;
/// Encode-form table: stride 4, the arm index the encoder dispatches on
/// (`0x10bf9f43`, then `jmp [ (form-1)*4 + 0x10bfae2d ]`).
const FORM_TABLE: u32 = 0x10c3_9b18;

/// A loaded PE with a VA → file-offset map built from its own section table.
struct Image {
    blob: Vec<u8>,
    base: u32,
    /// `(vaddr, vsize, rawptr, rawsize)`
    sections: Vec<(u32, u32, u32, u32)>,
}

impl Image {
    fn open(path: &Path) -> Option<Image> {
        let blob = std::fs::read(path).ok()?;
        let lfanew = u32::from_le_bytes(blob.get(0x3c..0x40)?.try_into().ok()?) as usize;
        if blob.get(lfanew..lfanew + 4)? != b"PE\0\0" {
            return None;
        }
        let coff = lfanew + 4;
        let nsec = u16::from_le_bytes(blob.get(coff + 2..coff + 4)?.try_into().ok()?) as usize;
        let opt_size =
            u16::from_le_bytes(blob.get(coff + 16..coff + 18)?.try_into().ok()?) as usize;
        let opt = coff + 20;
        let base = u32::from_le_bytes(blob.get(opt + 28..opt + 32)?.try_into().ok()?);
        let sect = opt + opt_size;
        let mut sections = Vec::new();
        for i in 0..nsec {
            let o = sect + i * 40;
            let g = |k: usize| -> Option<u32> {
                Some(u32::from_le_bytes(blob.get(o + k..o + k + 4)?.try_into().ok()?))
            };
            sections.push((g(12)?, g(8)?.max(g(16)?), g(20)?, g(16)?));
        }
        Some(Image { blob, base, sections })
    }

    fn off(&self, va: u32) -> Option<usize> {
        let rva = va.checked_sub(self.base)?;
        for &(vaddr, vsize, rawptr, rawsize) in &self.sections {
            if rva >= vaddr && rva < vaddr + vsize {
                let d = rva - vaddr;
                if d >= rawsize {
                    return None; // zero-fill tail
                }
                return Some((rawptr + d) as usize);
            }
        }
        None
    }

    fn u32_at(&self, va: u32) -> Option<u32> {
        let o = self.off(va)?;
        Some(u32::from_le_bytes(self.blob.get(o..o + 4)?.try_into().ok()?))
    }

    fn cstr(&self, va: u32) -> Option<String> {
        let o = self.off(va)?;
        let end = self.blob[o..].iter().position(|&b| b == 0)? + o;
        String::from_utf8(self.blob[o..end].to_vec()).ok()
    }

    fn mnemonic(&self, op: u32) -> Option<String> {
        let p = self.u32_at(MNEMONIC_TABLE + op * MNEMONIC_STRIDE)?;
        if p == 0 {
            return None;
        }
        self.cstr(p)
    }

    fn base_word(&self, op: u32) -> Option<u32> {
        self.u32_at(BASE_WORD_TABLE + op * 4)
    }

    fn form(&self, op: u32) -> Option<u32> {
        self.u32_at(FORM_TABLE + op * 4)
    }

    /// **The fail-closed identity check.**
    ///
    /// A sha256 would need a hand-rolled digest (std only) and would answer a
    /// weaker question than this does: what the tests below depend on is that
    /// these three tables are at these addresses *with these contents*, and a
    /// content probe says so directly. Six independently-derived cells, three
    /// tables, no two of which could survive a wrong base together.
    fn tables_are_the_pinned_ones(&self) -> Result<(), String> {
        let cells: [(&str, Option<String>, &str); 4] = [
            ("mnemonic[0x001]", self.mnemonic(0x001), "add"),
            ("mnemonic[0x00b]", self.mnemonic(0x00b), "addi"),
            ("mnemonic[0x285]", self.mnemonic(0x285), "blr"),
            ("mnemonic[0x284]", self.mnemonic(0x284), "ret"),
        ];
        for (what, got, want) in cells {
            if got.as_deref() != Some(want) {
                return Err(format!("{what} = {got:?}, expected {want:?}"));
            }
        }
        let words: [(&str, Option<u32>, u32); 4] = [
            ("base[add]", self.base_word(0x001), 0x7c00_0214),
            ("base[addi]", self.base_word(0x00b), 0x3800_0000),
            ("base[lis]", self.base_word(0x271), 0x3c00_0000),
            ("base[blr]", self.base_word(0x285), 0x4c00_0020),
        ];
        for (what, got, want) in words {
            if got != Some(want) {
                return Err(format!("{what} = {got:x?}, expected {want:#010x}"));
            }
        }
        if self.mnemonic(MNEMONIC_LAST).as_deref() != Some("_last") {
            return Err(format!(
                "mnemonic[{MNEMONIC_LAST:#x}] is not the _last sentinel"
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// The tap's rows
// ---------------------------------------------------------------------------

/// One `TU <idx> <opcode> <cat> <flags> <cc>` row, decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tuple {
    opcode: u32,
    cat: u8,
    flags: u8,
    /// `+0xa & 0x1f`. `stagetap.c` calls this a condition code; PREREG P1.4
    /// predicts it is the operand size in bytes. Scored in the findings doc.
    cc: u8,
}

impl Tuple {
    fn parse(row: &str) -> Option<Tuple> {
        let t: Vec<&str> = row.split_whitespace().collect();
        Some(Tuple {
            opcode: u32::from_str_radix(t.get(1)?, 16).ok()?,
            cat: u8::from_str_radix(t.get(2)?, 16).ok()?,
            flags: u8::from_str_radix(t.get(3)?, 16).ok()?,
            cc: u8::from_str_radix(t.get(4)?, 16).ok()?,
        })
    }
    /// `+0x9` bit 0 — "this tuple is a real machine instruction".
    fn is_instruction(self) -> bool {
        self.flags & 1 != 0
    }
}

/// One `OPD <idx> <dst> <src1> <src2>` row: hardware register numbers, or
/// `ABSENT` where the operand link was not a plausible pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Operands {
    dst: u32,
    src1: u32,
    src2: u32,
}

const ABSENT: u32 = 0xffff_ffff;

impl Operands {
    fn parse(row: &str) -> Option<Operands> {
        let t: Vec<&str> = row.split_whitespace().collect();
        Some(Operands {
            dst: u32::from_str_radix(t.get(1)?, 16).ok()?,
            src1: u32::from_str_radix(t.get(2)?, 16).ok()?,
            src2: u32::from_str_radix(t.get(3)?, 16).ok()?,
        })
    }
}

/// The obj's `.text` functions as `(name, big-endian words)`, in address order.
///
/// **Why this is not `ObjImage::text_comdat_functions_with_bytes`.** These
/// fixtures compile to a `.text` with `Characteristics = 0x60400020` — no
/// `IMAGE_SCN_LNK_COMDAT` (`0x1000`) bit — so the COMDAT walk correctly returns
/// nothing and a test that used it would have graded zero functions while
/// printing a pass. Measured, not anticipated: the first run of this file said
/// *"only 0 COMDATs"*. Slicing on the function symbols' `Value` is the general
/// form and works for both.
fn text_functions(obj: &ObjImage) -> Vec<(String, Vec<u32>)> {
    let b = obj.as_bytes();
    let g16 = |o: usize| u16::from_le_bytes(b[o..o + 2].try_into().unwrap());
    let g32 = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
    let nsec = g16(2) as usize;
    let symptr = g32(8) as usize;
    let nsym = g32(12) as usize;
    let opt = g16(16) as usize;
    let sect = 20 + opt;
    let mut text: Option<(usize, usize, usize)> = None; // (index1, rawptr, rawsize)
    for i in 0..nsec {
        let o = sect + i * 40;
        let name = String::from_utf8_lossy(&b[o..o + 8]).trim_end_matches('\0').to_string();
        if name == ".text" {
            text = Some((i + 1, g32(o + 20) as usize, g32(o + 16) as usize));
        }
    }
    let Some((idx, rawptr, rawsize)) = text else { return Vec::new() };
    let strtab = symptr + nsym * 18;
    let mut syms: Vec<(u32, String)> = Vec::new();
    let mut i = 0usize;
    while i < nsym {
        let o = symptr + i * 18;
        let naux = b[o + 17] as usize;
        let secnum = g16(o + 12) as usize;
        let sclass = b[o + 16];
        let typ = g16(o + 14);
        if secnum == idx && sclass == 2 && typ == 0x20 {
            let name = if g32(o) == 0 {
                let so = strtab + g32(o + 4) as usize;
                let end = b[so..].iter().position(|&c| c == 0).unwrap_or(0) + so;
                String::from_utf8_lossy(&b[so..end]).to_string()
            } else {
                String::from_utf8_lossy(&b[o..o + 8]).trim_end_matches('\0').to_string()
            };
            syms.push((g32(o + 4), name));
        }
        i += 1 + naux;
    }
    syms.sort_by_key(|(v, _)| *v);
    let mut out = Vec::new();
    for (k, (val, name)) in syms.iter().enumerate() {
        let start = *val as usize;
        let end = syms.get(k + 1).map(|(v, _)| *v as usize).unwrap_or(rawsize);
        let bytes = &b[rawptr + start..rawptr + end];
        out.push((
            name.clone(),
            bytes
                .chunks_exact(4)
                .map(|c| u32::from_be_bytes(c.try_into().unwrap()))
                .collect(),
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// Harness plumbing
// ---------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(name: &str) -> PathBuf {
    repo_root().join("fixtures/cpp").join(name)
}

fn work(tag: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-middle-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn require() -> bool {
    std::env::var_os("C2RS_REQUIRE_TOOLCHAIN").is_some()
}

/// `Some(…)` when everything this file needs is present; `None` after printing
/// a SKIP — or a panic, under `C2RS_REQUIRE_TOOLCHAIN`, because a skipped
/// legibility proof reads exactly like a passing one.
fn ready(tag: &str) -> Option<(Toolchain, Image)> {
    let Some(tc) = Toolchain::locate() else {
        assert!(!require(), "{tag}: C2RS_REQUIRE_TOOLCHAIN set but no toolchain");
        eprintln!("SKIP: toolchain absent");
        return None;
    };
    if tc.strace.is_none() || tc.mingw.is_none() {
        assert!(
            !require(),
            "{tag}: C2RS_REQUIRE_TOOLCHAIN set but strace/mingw missing"
        );
        eprintln!("SKIP: toolchain absent (strace/mingw)");
        return None;
    }
    let Some(img) = Image::open(&tc.c2_dll) else {
        assert!(!require(), "{tag}: C2RS_REQUIRE_TOOLCHAIN set but c2.dll unreadable");
        eprintln!("SKIP: toolchain absent (c2.dll unreadable)");
        return None;
    };
    // FAIL-CLOSED, never skip: the DLL is present and parsed. If its tables are
    // not the pinned ones, every number below would be a decode of the wrong
    // image, and that is a failure, not an absence.
    img.tables_are_the_pinned_ones()
        .unwrap_or_else(|e| panic!("{tag}: c2.dll is not the pinned image — {e}"));
    Some((tc, img))
}

/// One capture + one tapped replay with the operand window on, for one fixture.
/// Returns `(per-phase blocks, the obj that replay produced)`.
fn snap(
    tc: &Toolchain,
    cpp: &Path,
    tag: &str,
) -> (Vec<c2_reference::stage::StageBlock>, ObjImage) {
    let w = work(tag);
    let abs = cpp.canonicalize().unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| (*s).to_string()).collect();
    let cap = tc
        .capture_reference_with(
            &c2_reference::to_wibo_path(&abs),
            &w.join("cap"),
            &flags,
            None,
        )
        .unwrap_or_else(|e| panic!("{tag}: capture failed: {e}"));
    let out = cap.ref_obj_path.clone();
    let (obj, rep) = tc
        .replay_tapped_operands(&cap, &w.join("il"), &out, STAGE_SITES)
        .unwrap_or_else(|e| panic!("{tag}: tapped replay failed: {e}"));
    // POSITIVE CHECK before any row is read. An unarmed run yields an empty
    // block list, and "0 rows agreed with 0 rows" is the vacuous green this
    // whole family of instruments exists not to print.
    assert!(
        rep.armed_and_fired(),
        "{tag}: the tap did not arm and fire (armed={:?} refused={:?} hits={})",
        rep.armed,
        rep.refused,
        rep.total_hits()
    );
    assert!(
        rep.walk_refusals.is_empty(),
        "{tag}: the payload is TRUNCATED ({:?}) — every row below would be a floor",
        rep.walk_refusals
    );
    assert!(!rep.operands.is_empty(), "{tag}: the operand window emitted nothing");
    assert_eq!(
        rep.operands.len(),
        rep.tuples.len(),
        "{tag}: the operand window and the tuple stream are not in lockstep"
    );
    (rep.blocks, obj)
}

/// The FULL tuple list at one phase for one function — the FIRST region block,
/// which walks from the list head to `next == 0` and therefore covers the whole
/// list. (`ARCH_REVIEW` §1: 65.1% of the payload is suffix re-reads of exactly
/// this list; taking block 0 and no other is how this file avoids counting a
/// suffix twice.)
fn full_list(
    blocks: &[c2_reference::stage::StageBlock],
    phase: &str,
    func: u32,
) -> Option<(Vec<Tuple>, Vec<Operands>)> {
    let b = blocks.iter().find(|b| b.phase == phase && b.func == func)?;
    let tuples: Vec<Tuple> = b.tuples.iter().filter_map(|r| Tuple::parse(r)).collect();
    let ops: Vec<Operands> = b.operands.iter().filter_map(|r| Operands::parse(r)).collect();
    if tuples.len() != b.tuples.len() || ops.len() != b.operands.len() {
        return None;
    }
    Some((tuples, ops))
}

// ---------------------------------------------------------------------------
// INTERFACE 0 — the opcode-number space
// ---------------------------------------------------------------------------

/// **PREREG P0.1 / P0.2 / P0.3 / P0.4 — and P0.1 is REFUTED as written.**
///
/// The registered sentence was *"every tuple whose flag byte has bit 0 set
/// carries a machine opcode"*. It is false before the lowering band: on
/// `mvp_add3` the `sched1` list carries opcode `0x2f8` with bit 0 **set**, and
/// `0x2f8` is past the mnemonic table's `_last` sentinel. The repaired
/// statement — asserted here, and the more useful one — is:
///
/// * **at `sched0`** (after the lowering band) every real-instruction tuple is
///   a machine opcode with a mnemonic; and
/// * **before it** the same list contains at least one real-instruction tuple
///   that is *not*, which is precisely what the lowering band is for.
///
/// Structural tuples (bit 0 clear) are above the machine space at every phase,
/// so P0.2 survives unchanged.
#[test]
fn the_opcode_space_is_c2s_own_mnemonic_table() {
    let Some((tc, img)) = ready("interface-0") else { return };
    let mut instrs = 0usize;
    let mut structural = 0usize;
    let mut pseudo_pre = 0usize;
    let mut graded_final = 0usize;
    for name in ["mvp_add3.cpp", "mvp_two.cpp", "mvp_call.cpp", "il_stmt_seq.cpp"] {
        let (blocks, _) = snap(&tc, &fixture(name), "i0");
        assert!(!blocks.is_empty(), "{name}: no region blocks");
        for b in &blocks {
            for row in &b.tuples {
                let t = Tuple::parse(row).unwrap_or_else(|| panic!("{name}: bad row {row:?}"));
                if t.is_instruction() {
                    instrs += 1;
                    let machine = t.opcode < MNEMONIC_LAST;
                    if b.phase == "sched0" {
                        graded_final += 1;
                        assert!(
                            machine,
                            "{name} sched0: real-instruction tuple opcode {:#x} is at or \
                             past the mnemonic table's _last sentinel — after the lowering \
                             band there should be no pseudo-op left",
                            t.opcode
                        );
                        let m = img.mnemonic(t.opcode);
                        assert!(
                            m.is_some() && m.as_deref() != Some("_last"),
                            "{name} sched0: opcode {:#x} has no mnemonic",
                            t.opcode
                        );
                    } else if !machine {
                        pseudo_pre += 1;
                    }
                } else {
                    structural += 1;
                    assert!(
                        t.opcode > MNEMONIC_LAST + 2,
                        "{name} {}: structural tuple opcode {:#x} is INSIDE the machine \
                         opcode space — P0.2 refuted",
                        b.phase,
                        t.opcode
                    );
                }
            }
        }
    }
    // A green over zero rows is not a green.
    assert!(instrs >= 20, "only {instrs} real-instruction tuples seen");
    assert!(structural >= 20, "only {structural} structural tuples seen");
    assert!(graded_final >= 8, "only {graded_final} sched0 instruction tuples");
    // The LIVENESS half: without this, "no pseudo-ops at sched0" could be true
    // because there are never any pseudo-ops anywhere.
    assert!(
        pseudo_pre > 0,
        "no pre-lowering pseudo-op tuple seen at all — the sched0 assertion above is \
         then vacuous and says nothing about the lowering band"
    );
    eprintln!(
        "interface-0: {instrs} real-instruction tuples ({graded_final} at sched0, all machine \
         opcodes), {structural} structural (all above the machine space), {pseudo_pre} \
         pre-lowering pseudo-op instruction tuples"
    );
}

// ---------------------------------------------------------------------------
// INTERFACE 1 — IL record → tuple, on the closed subset
// ---------------------------------------------------------------------------

/// The `.ex` operand-class table (`DAT_10b25e48`, `WB_READER_FINDINGS.md` §3.1)
/// as this decoder uses it: only the four classes the traced subset needs.
///
/// Read from the image, never transcribed — the class byte for each opcode is
/// `image[0x10b25e48 + opcode]`.
const EX_CLASS_TABLE: u32 = 0x10b2_5e48;

/// Width of one `.ex` token, for the closed subset, using c2's own class byte.
/// Returns `None` for any opcode outside the subset — the decoder REFUSES
/// rather than guessing, which is what keeps a green from meaning "it skipped
/// what it did not know".
fn ex_token_width(img: &Image, body: &[u8], p: usize) -> Option<usize> {
    let op = *body.get(p)?;
    let class = *img.blob.get(img.off(EX_CLASS_TABLE + op as u32)?)?;
    // TYPE word: 1/2/3 bytes (WB_READER_FINDINGS.md §3.2 / DISCLOSURE W-EXT-1),
    // followed by the globally gated LEB skip, which IS present in these
    // captures (every `86 41 74` in the corpus is word + one skip byte).
    let type_len = |q: usize| -> Option<usize> {
        let b1 = *body.get(q)?;
        let word = if b1 & 0x80 == 0 {
            1
        } else if b1 & 0x40 != 0 {
            3
        } else {
            2
        };
        // the LEB continuation run
        let mut k = q + word;
        loop {
            let b = *body.get(k)?;
            k += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        Some(k - q)
    };
    // varU: 2 bytes LE, 4 if bit 15 of the second byte is set.
    let varu_len = |q: usize| -> Option<usize> {
        let hi = *body.get(q + 1)?;
        Some(if hi & 0x80 != 0 { 4 } else { 2 })
    };
    match class {
        0x00 => Some(1),                            // payload-free
        0x01 => Some(1 + type_len(p + 1)?),         // one TYPE
        0x02 => Some(1 + varu_len(p + 1)?),         // one symbol token
        0x18 => {
            // varU symbol, then TYPE  (`B9 <sym> <TYPE>`)
            let v = varu_len(p + 1)?;
            Some(1 + v + type_len(p + 1 + v)?)
        }
        0x0d => {
            // i32c: 1 byte, or 5 when the first is 0x80
            Some(1 + if *body.get(p + 1)? == 0x80 { 5 } else { 1 })
        }
        _ => None,
    }
}

/// **The interface-1 decoder for the closed subset.**
///
/// Walks one `.ex` function body with c2's own operand-class table and emits
/// the machine-tuple rows it predicts. Every rule is marked DERIVED (it follows
/// from a table in the image) or TRANSCRIBED (it was observed once on
/// `mvp_add3` and is not derived from anything) — because a decoder whose rules
/// are all transcriptions proves nothing, and one that hides which is which
/// proves less.
///
/// * DERIVED — token widths, from `DAT_10b25e48` + the TYPE-word rule.
/// * DERIVED — `0x02` (class `00`, payload-free) is a binary add and becomes
///   exactly one `add` (machine opcode `0x001`) tuple. The count is a
///   *prediction*: n−1 for an n-leaf chain, tested at n = 2, 3 and 4.
/// * DERIVED — `0xB9 <sym> <TYPE>` (a parameter load) becomes no tuple.
/// * TRANSCRIBED — the `0x41 <TYPE>` return-value token becomes one tuple with
///   opcode `0x2f8` / category `0x15`, and the four-row structural tail
///   `0x30f/0x17`, `0x309/0x1a`, `0x30b/0x19`, `0x309/0x1a`. Neither is derived
///   from any table; both are read off one snapshot.
/// * DERIVED — the `cc` column is the operand size in bytes from the IL TYPE
///   word's size index `(v >> 9) & 7` (PREREG P1.4).
///
/// Returns `None` — refuses — on any token outside the subset.
fn decode_body_to_tuples(img: &Image, body: &[u8]) -> Option<Vec<Tuple>> {
    let mut out: Vec<Tuple> = Vec::new();
    let mut p = 0usize;
    // The operand size, carried from the last TYPE word seen. add3's operands
    // are all `86 41 74` → word 0x641 → size index (0x641 >> 9) & 7 = 3 → 4.
    let mut size = 0u8;
    while p < body.len() {
        let op = body[p];
        // The trailing metadata records and the end-of-stream marker end the
        // walk BEFORE their width is asked for: `0x4F` is operand class `0x0C`,
        // whose payload is a sub-record the subset does not read, and asking
        // for its width would make this decoder refuse a body it has in fact
        // finished decoding.
        if op == 0x4f || op == 0x4d {
            break;
        }
        let w = ex_token_width(img, body, p)?;
        match op {
            0xb9 => {
                // parameter load: read its TYPE for the size, emit no tuple
                let v = if body.get(p + 2)? & 0x80 != 0 { 4 } else { 2 };
                let b1 = *body.get(p + 1 + v)?;
                let word: u32 = if b1 & 0x80 == 0 {
                    b1 as u32
                } else if b1 & 0x40 != 0 {
                    let b2 = *body.get(p + 2 + v)? as u32;
                    let b3 = *body.get(p + 3 + v)? as u32;
                    ((b2 & 0x7f) << 16) | (((b1 as u32) & 0x7f) << 8) | b3
                } else {
                    (((b1 as u32) & 0x7f) << 8) | *body.get(p + 2 + v)? as u32
                };
                size = match (word >> 9) & 7 {
                    1 => 1,
                    2 => 2,
                    3 => 4,
                    4 => 8,
                    _ => 0,
                };
            }
            0x02 => out.push(Tuple { opcode: 0x001, cat: 0x0d, flags: 0x01, cc: size }),
            0x41 => {
                out.push(Tuple { opcode: 0x2f8, cat: 0x15, flags: 0x01, cc: size });
                // the structural tail (TRANSCRIBED)
                out.push(Tuple { opcode: 0x30f, cat: 0x17, flags: 0, cc: 0 });
                out.push(Tuple { opcode: 0x309, cat: 0x1a, flags: 0, cc: 0 });
                out.push(Tuple { opcode: 0x30b, cat: 0x19, flags: 0, cc: 0 });
                out.push(Tuple { opcode: 0x309, cat: 0x1a, flags: 0, cc: 0 });
            }
            0x3a | 0x54 | 0x29 | 0x53 => {} // jump to the epilogue, scope close, label
            _ => return None,               // outside the subset: REFUSE, never skip
        }
        p += w;
    }
    Some(out)
}

/// Locate one function body in the `.ex`: from its `4C 4F 11 53` marker to the
/// next marker or the `4D` end-of-stream.
fn body_segments(ex: &[u8]) -> Vec<&[u8]> {
    let mut starts: Vec<usize> = Vec::new();
    let mut i = 0usize;
    while i + 4 <= ex.len() {
        if ex[i] == 0x4c && ex[i + 1] == 0x4f && ex[i + 2] == 0x11 && ex[i + 3] == 0x53 {
            starts.push(i + 4);
            i += 4;
        } else {
            i += 1;
        }
    }
    let mut out = Vec::new();
    for (k, &s) in starts.iter().enumerate() {
        let e = starts.get(k + 1).map(|n| n - 4).unwrap_or(ex.len());
        out.push(&ex[s..e]);
    }
    out
}

/// **PREREG P1.1 / P1.2 / P1.3 / P1.4 / P1.6.**
///
/// Row-for-row equality between the decoder's output and the live tap's
/// `sched1` list, on three functions across two fixtures whose additive chains
/// have 2, 3 and 4 leaves.
#[test]
fn the_il_subset_decoder_reproduces_the_tuple_rows() {
    let Some((tc, img)) = ready("interface-1") else { return };
    // (fixture, function ordinal, .ex body ordinal, expected `add` count)
    let cells: [(&str, u32, usize, usize); 3] = [
        ("mvp_add3.cpp", 1, 0, 2),
        ("mvp_two.cpp", 1, 0, 1),
        ("mvp_two.cpp", 2, 1, 3),
    ];
    let mut graded = 0usize;
    for (name, func, body_ix, adds) in cells {
        let w = work("i1cap");
        let abs = fixture(name).canonicalize().unwrap();
        let flags: Vec<String> = FLAGS.iter().map(|s| (*s).to_string()).collect();
        let cap = tc
            .capture_reference_with(
                &c2_reference::to_wibo_path(&abs),
                &w.join("cap"),
                &flags,
                None,
            )
            .unwrap_or_else(|e| panic!("{name}: capture failed: {e}"));
        let ex = cap
            .bundle
            .get("ex")
            .unwrap_or_else(|| panic!("{name}: the captured bundle has no .ex"))
            .to_vec();
        let bodies = body_segments(&ex);
        assert!(
            bodies.len() > body_ix,
            "{name}: only {} function bodies in the .ex",
            bodies.len()
        );
        let predicted = decode_body_to_tuples(&img, bodies[body_ix]).unwrap_or_else(|| {
            panic!("{name} body {body_ix}: the subset decoder REFUSED — a token outside the closed subset")
        });
        let (blocks, _) = snap(&tc, &fixture(name), "i1");
        let (observed, _) = full_list(&blocks, "sched1", func)
            .unwrap_or_else(|| panic!("{name} fn{func}: no sched1 block"));
        assert_eq!(
            predicted, observed,
            "{name} fn{func}: the decoder's rows differ from the tap's\n  predicted: {predicted:?}\n  observed:  {observed:?}"
        );
        // The count is the *predictive* half — a transcription of add3's 2
        // cannot produce 1 or 3.
        let got = observed.iter().filter(|t| t.opcode == 0x001).count();
        assert_eq!(got, adds, "{name} fn{func}: expected {adds} `add` tuples, saw {got}");
        // P1.4: the cc column is the operand size in bytes, 4 for `int`.
        for t in observed.iter().filter(|t| t.is_instruction()) {
            assert_eq!(t.cc, 4, "{name} fn{func}: real-instruction tuple cc = {}", t.cc);
        }
        graded += 1;
    }
    assert_eq!(graded, 3);
    eprintln!("interface-1: {graded} functions, row-for-row equality, chains of 2/3/4 leaves");
}

// ---------------------------------------------------------------------------
// INTERFACE 2 — final tuple order → COFF `.text`
// ---------------------------------------------------------------------------

/// Encode one real-instruction tuple, for the closed form subset.
///
/// `base_word[opcode] | <fields>`, exactly as `FUN_10bf9f15` composes it. Only
/// the two forms the traced subset reaches are implemented; anything else
/// REFUSES.
fn encode(img: &Image, t: Tuple, o: Operands) -> Option<u32> {
    let base = img.base_word(t.opcode)?;
    let form = img.form(t.opcode)?;
    match form {
        // The three-register arm (0x10bf9f91): RT | RA | RB, 5 bits each, at
        // bits 21 / 16 / 11.
        0x03 => {
            if o.dst == ABSENT || o.src1 == ABSENT || o.src2 == ABSENT {
                return None;
            }
            Some(base | ((o.dst & 0x1f) << 21) | ((o.src1 & 0x1f) << 16) | ((o.src2 & 0x1f) << 11))
        }
        // `ret`/`blr` (form 0x1e): no register operands; the BO field is the
        // form's own constant 20 ("branch always").
        0x1e => Some(base | (20 << 21)),
        _ => None,
    }
}

/// **PREREG P2.1 / P2.2 / P2.3 / P2.5 — the lowering byte check.**
///
/// Given the `sched0`-entry tuple order and the operand window, reproduce the
/// `.text` COMDAT bytes of one function, all 32 bits of every word, and compare
/// against the obj the same run produced.
#[test]
fn the_final_tuple_order_reproduces_the_text_words() {
    let Some((tc, img)) = ready("interface-2") else { return };
    // (fixture, function ordinal, COMDAT ordinal)
    let cells: [(&str, u32, usize); 3] = [
        ("mvp_add3.cpp", 1, 0),
        ("mvp_two.cpp", 1, 0),
        ("mvp_two.cpp", 2, 1),
    ];
    let mut words_graded = 0usize;
    for (name, func, comdat) in cells {
        let (blocks, obj) = snap(&tc, &fixture(name), "i2");
        let (tuples, ops) = full_list(&blocks, "sched0", func)
            .unwrap_or_else(|| panic!("{name} fn{func}: no sched0 block"));
        assert_eq!(tuples.len(), ops.len(), "{name}: tuple/operand rows out of step");
        let funcs = text_functions(&obj);
        assert!(
            funcs.len() > comdat,
            "{name}: only {} .text functions in the obj",
            funcs.len()
        );
        let (fname, real) = &funcs[comdat];

        // P2.1: the real-instruction tuples, in order, are the emitted words.
        let insns: Vec<(Tuple, Operands)> = tuples
            .iter()
            .zip(ops.iter())
            .filter(|(t, _)| t.is_instruction())
            .map(|(t, o)| (*t, *o))
            .collect();
        assert_eq!(
            insns.len(),
            real.len(),
            "{name} fn{func} ({fname}): {} real-instruction tuples at sched0 entry \
             but {} emitted words",
            insns.len(),
            real.len()
        );

        // P2.2 / P2.3: every word, all 32 bits.
        for (i, (t, o)) in insns.iter().enumerate() {
            let got = encode(&img, *t, *o).unwrap_or_else(|| {
                panic!(
                    "{name} fn{func} word {i}: no encode rule for opcode {:#x} ({:?}) \
                     form {:?} operands {o:?} — the subset REFUSES rather than guessing",
                    t.opcode,
                    img.mnemonic(t.opcode),
                    img.form(t.opcode)
                )
            });
            assert_eq!(
                got, real[i],
                "{name} fn{func} ({fname}) word {i}: encoded {:#010x} from tuple {t:?} \
                 operands {o:?} ({:?}), obj has {:#010x}",
                got,
                img.mnemonic(t.opcode),
                real[i]
            );
            words_graded += 1;
        }
    }
    assert!(words_graded >= 9, "only {words_graded} words graded");
    eprintln!("interface-2: {words_graded} .text words reproduced from the final tuple order, 32 bits of 32");
}

// ---------------------------------------------------------------------------
// The new tap field's own required-zero
// ---------------------------------------------------------------------------

/// **The operand window must not move a single obj byte.**
///
/// It is the first thing in this tap that dereferences a pointer *out of* the
/// tuple record, so its neutrality is not inherited from the walk's — it has to
/// be measured on its own. Same shape as `stage.rs`'s G1: one disarmed replay,
/// one armed-with-operands replay, byte compare.
#[test]
fn the_operand_window_never_moves_the_obj() {
    let Some((tc, _img)) = ready("operand-neutrality") else { return };
    let w = work("i2n");
    let abs = fixture("mvp_add3.cpp").canonicalize().unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| (*s).to_string()).collect();
    let cap = tc
        .capture_reference_with(&c2_reference::to_wibo_path(&abs), &w.join("cap"), &flags, None)
        .expect("capture");
    let out = cap.ref_obj_path.clone();
    let (disarmed, rep0) = tc
        .replay_tapped_raw(&cap, &w.join("il0"), &out, &[], false, 0)
        .expect("disarmed replay");
    assert!(rep0.lines.is_empty(), "the DISARMED leg printed stage-tap output");
    let (armed, rep1) = tc
        .replay_tapped_operands(&cap, &w.join("il1"), &out, STAGE_SITES)
        .expect("armed replay");
    assert!(
        rep1.armed_and_fired(),
        "byte identity is FREE unless the tap armed and fired (armed={:?} hits={})",
        rep1.armed,
        rep1.total_hits()
    );
    assert!(!rep1.operands.is_empty(), "the operand window emitted nothing");
    assert_eq!(
        ObjImage::diff(&disarmed, &armed),
        ObjDiff::Identical,
        "THE OPERAND WINDOW MOVED THE OBJ — the oracle is grading a different compiler"
    );
}
