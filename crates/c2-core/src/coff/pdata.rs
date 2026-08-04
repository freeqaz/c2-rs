//! `.pdata` — the Xbox 360 unwind record (there is no `.xdata`) and the frame
//! description that generates it.


/// `.pdata` section characteristics: CNT_INIT_DATA | ALIGN_8 | MEM_READ.
pub(crate) const CH_PDATA: u32 = 0x4040_0040;


/// The unwind facts one framed function contributes: the two lengths that go
/// into its `.pdata` record and, as it happens, the values of its two `$M`
/// labels. Both in **bytes**; both must be word multiples.
///
/// A **leaf** contributes nothing — c2 emits no `.pdata` record for a function
/// that establishes no frame, and "establishes a frame" is exactly what the
/// emitter knows (it wrote the prologue). Measured: a leaf with a 400-byte local
/// array addresses it below `r1` in the red zone (`addi r10,r1,-400`) and gets no
/// record; make the array 70,000 bytes so the prologue has to move `r1` and the
/// record appears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Prologue length in bytes — the offset one past the last prologue
    /// instruction, i.e. the value of the `$M(n)` label.
    pub prolog_len: u32,
    /// Function length in bytes, **excluding** any inter-function padding —
    /// the value of the `$M(n+1)` label.
    pub func_len: u32,
}

/// `IMAGE_COMDAT_SELECT_ASSOCIATIVE` — the selection a per-function `.pdata`
/// COMDAT carries under `/Gy`, tying it to its `.text` COMDAT.
pub(crate) const COMDAT_SELECT_ASSOCIATIVE: u8 = 5;

/// `.pdata` COMDAT characteristics under `/Gy`: [`CH_PDATA`] plus
/// `IMAGE_SCN_LNK_COMDAT` (0x1000).
pub(crate) const CH_PDATA_COMDAT: u32 = 0x4040_1040;

/// Build the 8-byte X360 `RUNTIME_FUNCTION` for one framed function:
/// `BeginAddress` (patched by an ADDR32 relocation against the function's own
/// symbol, so the raw value is the addend — 0 for every record the port emits)
/// followed by the packed unwind word, both **big-endian** (like `.text`,
/// unlike every COFF header field).
///
/// The unwind word is a bitfield, established from c2's own output rather than
/// from any x64 `.pdata` documentation — the Xbox 360 form has no `.xdata` and
/// no unwind-code array at all, the whole record is these 8 bytes:
///
/// ```text
///   bits  7..0   PrologLen   prologue length in INSTRUCTIONS
///   bits 29..8   FuncLen     function length in INSTRUCTIONS
///   bit  30      ThirtyTwoBit  1 in every record c2 emitted across the probes
///   bit  31      ExceptionFlag 1 iff the function has EH data
/// ```
///
/// Witnesses, each read straight out of a reference obj (source in
/// `docs/OBJ_FORMAT_MVP.md` §7):
///
/// ```text
///   0x40000903  9 words / prolog 3   return g(a)+1        .text 0x24, $M @ 0x0c
///   0x40001205 18 words / prolog 5   two calls, r30/r31   .text 0x48, $M @ 0x14
///   0x40001607 22 words / prolog 7   100 KB local + calls .text 0x58, $M @ 0x1c
///   0x40002203 34 words / prolog 3   6 args via __savegprlr_25
///   0x40000f06 15 words / prolog 6   leaf with a 70 KB frame (still framed)
///   0xc0001306 19 words / prolog 6   a body with a destructor, /EHsc
/// ```
///
/// so `FuncLen` and `PrologLen` are the only fields that move, they are exactly
/// the two `$M` label values divided by four, and bit 31 is the one thing that
/// takes the record outside the class this port emits (EH also splits a function
/// into **several** records — a `try`/`catch` body produced two, the catch
/// funclet's first, with a non-zero `BeginAddress` addend).
pub fn pdata_record(begin_addend: u32, frame: &Frame) -> [u8; 8] {
    debug_assert_eq!(frame.func_len % 4, 0, "function length is a word multiple");
    debug_assert_eq!(frame.prolog_len % 4, 0, "prologue length is a word multiple");
    let unwind = UNWIND_THIRTY_TWO_BIT | ((frame.func_len / 4) << 8) | (frame.prolog_len / 4);
    let mut r = [0u8; 8];
    r[..4].copy_from_slice(&begin_addend.to_be_bytes());
    r[4..].copy_from_slice(&unwind.to_be_bytes());
    r
}

/// Bit 30 of the unwind word — set in every record c2 emitted across every
/// probe. Named rather than folded into a magic constant because bit 31 beside
/// it is the EH flag, and the port refuses that case.
pub(crate) const UNWIND_THIRTY_TWO_BIT: u32 = 0x4000_0000;

/// The `.pdata` raw section for a run of framed functions, records concatenated
/// in `.text` order. Under `/Gy` this is called once per function (one record);
/// packed, once for the whole TU.
pub(crate) fn build_pdata(frames: &[&Frame]) -> Vec<u8> {
    let mut b = Vec::with_capacity(frames.len() * 8);
    for f in frames {
        b.extend_from_slice(&pdata_record(0, f));
    }
    b
}
