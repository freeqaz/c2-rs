//! **W-R1c — the `.in` initializer stream, read for the first time.**
//!
//! `.in` has been carried by [`crate::IlBundle`] since the container model was
//! written and parsed by nothing: `grep IL_SUFFIXES` finds it in the suffix list
//! and in two round-trip tests, and there was no reader. This module adds
//! exactly one — the **string-literal** record — because that is the one input
//! the `??__E` dynamic-initializer obj needs and `.gl` does not carry in a form
//! this port can use.
//!
//! It is deliberately not a general `.in` parser. Everything that is not a
//! string-literal record is skipped without being interpreted, and a record that
//! does not frame exactly is refused rather than partially believed.

use super::readers::read_token_var;

/// The record tag that introduces a string-literal payload: `00 03`, immediately
/// after the operand token.
///
/// MEASURED, one row per capture, `<token> 00 03 <len> <bytes> 07`:
///
/// ```text
///   fixture /Ox   ef 09 · 00 03 · 04       · 61 62 63 00                  · 07
///   TomCrypt      fc 09 · 00 03 · 1a       · system/src/synth/tomcrypt\0  · 07
///   Zlib          fc 09 · 00 03 · 10       · system/src/zlib\0            · 07
///   p_biglit      f0 09 · 00 03 · 80 86 00 · 0123…ABC\0   (134 bytes)     · 07
/// ```
///
/// `<len>` counts the trailing NUL, so the payload is the literal's bytes
/// verbatim — exactly what `c2_core::coff::StringLiteral::bytes` wants and what
/// `string_comdat_name` hashes.
const LITERAL_TAG: [u8; 2] = [0x00, 0x03];

/// The byte that closes a string-literal record.
const RECORD_END: u8 = 0x07;

/// Read the `.in` **length** field: a single byte below `0x80`, else `0x80`
/// followed by a **little-endian u16**.
///
/// **This is a third varint and not either of the two already in the crate.**
/// [`super::readers::read_varint`] escapes with `80` + LE**32** and
/// [`read_token_var`] is a 2-or-4-byte token; reusing either here reads the
/// wrong width. The 134-byte probe is the witness: `80 86 00` is `0x0086` = 134,
/// and the literal's first byte follows immediately at +3. A `read_varint` here
/// would have consumed `86 00 30 31` as a 32-bit length and desynchronized every
/// literal of 128 bytes or more.
///
/// The mis-read is not hypothetical-only: with two target TUs at 26 and 16 bytes
/// both encodings agree, so nothing in this lane's own corpus would have caught
/// it. The probe exists because the rung doc asked for it.
fn read_len(inb: &[u8], p: &mut usize) -> Option<u32> {
    let b0 = *inb.get(*p)?;
    if b0 < 0x80 {
        *p += 1;
        return Some(b0 as u32);
    }
    if b0 != 0x80 {
        // A short form with the high bit set would be a negative signed byte in
        // the statement varint's alphabet. Nothing measured spells a length that
        // way, so it fails closed rather than being sign-extended into a huge
        // length.
        return None;
    }
    let v = u16::from_le_bytes([*inb.get(*p + 1)?, *inb.get(*p + 2)?]);
    *p += 3;
    Some(v as u32)
}

/// Every string literal `.in` defines, keyed by the operand token an `.ex` body
/// references it with.
///
/// **The read is self-checking**, which is what makes a length this reader got
/// wrong refuse instead of returning a prefix: the decoded length must land
/// exactly on a byte that is NUL, and the byte after it must be [`RECORD_END`].
/// A literal is NUL-terminated by construction, so both halves are real
/// structure and not a checksum invented for the occasion.
///
/// A token two records disagree about is dropped, the same third value
/// [`super::gl::gl_data_objects`] and `gl_symbol_index` give an ambiguous token.
pub(crate) fn in_string_literals(inb: &[u8]) -> std::collections::BTreeMap<u32, Vec<u8>> {
    let mut out: std::collections::BTreeMap<u32, Option<Vec<u8>>> =
        std::collections::BTreeMap::new();
    let mut i = 0usize;
    while i + LITERAL_TAG.len() < inb.len() {
        if inb[i..i + 2] != LITERAL_TAG {
            i += 1;
            continue;
        }
        // The token ends where the tag begins. Try the 4-byte form first and
        // require its decoded width to land exactly on the tag, the same
        // discipline `gl_symbol_index` applies.
        let mut matched = false;
        for w in [4usize, 2] {
            if i < w {
                continue;
            }
            let Some((tok, got)) = read_token_var(inb, i - w) else {
                continue;
            };
            if got != w {
                continue;
            }
            let mut p = i + LITERAL_TAG.len();
            let Some(len) = read_len(inb, &mut p) else {
                continue;
            };
            if len == 0 {
                continue;
            }
            let Some(end) = p.checked_add(len as usize) else {
                continue;
            };
            if end >= inb.len() {
                continue;
            }
            // Self-check: the payload's last byte is the literal's NUL and the
            // record closes on `07`. Either failing means the length was read
            // wrong, and a wrong length yields wrong `.rdata` bytes AND a wrong
            // COMDAT name.
            if inb[end - 1] != 0 || inb[end] != RECORD_END {
                continue;
            }
            let bytes = inb[p..end].to_vec();
            match out.get(&tok) {
                None => {
                    out.insert(tok, Some(bytes));
                }
                Some(Some(prev)) if *prev != bytes => {
                    out.insert(tok, None);
                }
                _ => {}
            }
            i = end + 1;
            matched = true;
            break;
        }
        if !matched {
            i += 1;
        }
    }
    out.into_iter().filter_map(|(t, b)| b.map(|b| (t, b))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four measured records, byte for byte. The `p_biglit` row is the one
    /// that fails if the length is read with the statement varint's LE32 escape.
    #[test]
    fn the_four_measured_literal_records_decode() {
        // fixture: `abc\0`, short-form length.
        let mut v = vec![0xef, 0x09, 0x00, 0x03, 0x04, b'a', b'b', b'c', 0x00, 0x07];
        let got = in_string_literals(&v);
        assert_eq!(got.get(&0xef09).map(|b| b.as_slice()), Some(&b"abc\0"[..]));

        // TomCrypt: 26 bytes including the NUL.
        v = vec![0xfc, 0x09, 0x00, 0x03, 0x1a];
        v.extend_from_slice(b"system/src/synth/tomcrypt\0");
        v.push(0x07);
        assert_eq!(
            in_string_literals(&v).get(&0xfc09).map(|b| b.as_slice()),
            Some(&b"system/src/synth/tomcrypt\0"[..])
        );

        // Zlib: 16 bytes.
        v = vec![0xfc, 0x09, 0x00, 0x03, 0x10];
        v.extend_from_slice(b"system/src/zlib\0");
        v.push(0x07);
        assert_eq!(
            in_string_literals(&v).get(&0xfc09).map(|b| b.as_slice()),
            Some(&b"system/src/zlib\0"[..])
        );
    }

    /// **The escape probe, and the reason it exists.** A 134-byte literal spells
    /// its length `80 86 00` — `0x80` then a little-endian **u16**. The
    /// statement varint would read `80` + LE32 and run off the end of the
    /// record.
    #[test]
    fn a_literal_past_127_bytes_uses_an_le16_escape_not_the_statement_varint() {
        let mut lit: Vec<u8> = Vec::new();
        while lit.len() < 133 {
            lit.push(b"0123456789"[lit.len() % 10]);
        }
        lit.push(0);
        assert_eq!(lit.len(), 134);

        let mut v = vec![0xf0, 0x09, 0x00, 0x03, 0x80, 0x86, 0x00];
        v.extend_from_slice(&lit);
        v.push(0x07);
        assert_eq!(
            in_string_literals(&v).get(&0xf009).map(|b| b.as_slice()),
            Some(lit.as_slice()),
            "the length escape is 0x80 + LE16"
        );

        // The LE32 reading would have taken `86 00` plus the literal's first two
        // bytes as the length; assert it is not silently also accepted.
        let mut wrong = vec![0xf0, 0x09, 0x00, 0x03, 0x80, 0x86, 0x00, 0x00, 0x00];
        wrong.extend_from_slice(&lit);
        wrong.push(0x07);
        assert_eq!(
            in_string_literals(&wrong).get(&0xf009),
            None,
            "an LE32-shaped length does not frame and must refuse"
        );
    }

    /// The self-check is the fence: a record whose length does not land on a NUL
    /// followed by `07` is refused, not truncated to a prefix.
    #[test]
    fn a_length_that_does_not_frame_refuses() {
        // Length says 4 but the payload's 4th byte is not a NUL.
        let v = vec![0xef, 0x09, 0x00, 0x03, 0x04, b'a', b'b', b'c', b'd', 0x07];
        assert_eq!(in_string_literals(&v).get(&0xef09), None);

        // Length frames onto a NUL but the record does not close on `07`.
        let v = vec![0xef, 0x09, 0x00, 0x03, 0x04, b'a', b'b', b'c', 0x00, 0x09];
        assert_eq!(in_string_literals(&v).get(&0xef09), None);

        // A zero length is not a literal — every literal carries its NUL.
        let v = vec![0xef, 0x09, 0x00, 0x03, 0x00, 0x07];
        assert_eq!(in_string_literals(&v).get(&0xef09), None);
    }

    /// A token two records disagree about is dropped rather than resolved to the
    /// first — the same third value the `.gl` readers give.
    #[test]
    fn an_ambiguous_token_is_dropped() {
        let mut v = vec![0xef, 0x09, 0x00, 0x03, 0x04, b'a', b'b', b'c', 0x00, 0x07];
        v.extend_from_slice(&[0xef, 0x09, 0x00, 0x03, 0x04, b'x', b'y', b'z', 0x00, 0x07]);
        assert_eq!(in_string_literals(&v).get(&0xef09), None);

        // …but two records that AGREE are not a conflict.
        let mut same = vec![0xef, 0x09, 0x00, 0x03, 0x04, b'a', b'b', b'c', 0x00, 0x07];
        same.extend_from_slice(&[0xef, 0x09, 0x00, 0x03, 0x04, b'a', b'b', b'c', 0x00, 0x07]);
        assert_eq!(
            in_string_literals(&same).get(&0xef09).map(|b| b.as_slice()),
            Some(&b"abc\0"[..])
        );
    }

    /// An empty or truncated `.in` yields nothing and does not panic — the CLI
    /// must degrade cleanly.
    #[test]
    fn a_truncated_stream_yields_nothing() {
        assert!(in_string_literals(&[]).is_empty());
        assert!(in_string_literals(&[0x00, 0x03]).is_empty());
        assert!(in_string_literals(&[0xef, 0x09, 0x00, 0x03]).is_empty());
        assert!(in_string_literals(&[0xef, 0x09, 0x00, 0x03, 0x40]).is_empty());
    }
}
