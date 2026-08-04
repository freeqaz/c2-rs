//! CRC-32 in its two initial values, which is the obvious way to get this
//! wrong: the aux section `CheckSum` is init `0`, the string COMDAT name hash is
//! init `0xFFFFFFFF` (JamCRC). Same polynomial, different seeds
//! (`docs/OBJ_DYNINIT_SHAPE.md` §2.3 vs §5).


/// Reflected CRC-32, polynomial `0xEDB88320`, **no final inversion**, over a
/// byte run — parameterised on the initial value, because c2 uses this same
/// loop twice with two different ones and getting them the wrong way round is
/// the documented way to implement this wrong (`docs/OBJ_DYNINIT_SHAPE.md`
/// §2.3, closing note):
///
/// | consumer | init | via |
/// |---|---|---|
/// | COFF aux section-def `CheckSum` | `0` | [`coff_checksum`] |
/// | the `??_C@…` string-literal name hash (JamCRC) | `0xFFFFFFFF` | [`jamcrc`] |
///
/// One loop with an argument, not two loops — two independent copies is exactly
/// how the swap happens, and it is invisible to every consistency check the port
/// has (both values are 32 bits and both look like noise).
pub(crate) fn crc32_reflected(init: u32, data: &[u8]) -> u32 {
    let mut c = init;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    c
}

/// The COFF aux section-def CheckSum algorithm — [`crc32_reflected`] with init
/// `0`. Used for `.pdata` (whose aux carries a real checksum even though it is
/// not a COMDAT) and for a string-literal `.rdata`; the fixed `.XBLD$W` COMDAT
/// checksums stay hardcoded above.
///
/// **Scope, corrected out-of-sample** (`docs/OBJ_DYNINIT_SHAPE.md` §2.3, held-out
/// prediction H9 refuted): the field is `0` for `.text$y?`, for `.text`, for
/// `.bss`, for `.CRT$XCU`, for `.drectve`/`.debug$S`, and — the refutation — for
/// an **FP-constant** `.rdata` COMDAT. It carries the real CRC for the two
/// `.XBLD$W`, for `.pdata`, and for a **string** `.rdata`, COMDAT or not.
pub(crate) fn coff_checksum(data: &[u8]) -> u32 {
    crc32_reflected(0, data)
}

/// JamCRC — [`crc32_reflected`] with init `0xFFFFFFFF`, no final XOR
/// (equivalently `!crc32(data)`). The hash inside a `??_C@…` string-literal
/// COMDAT name, over the literal's bytes **including the NUL**.
pub(crate) fn jamcrc(data: &[u8]) -> u32 {
    crc32_reflected(0xFFFF_FFFF, data)
}
