//! `entry.bin` — the single-file container for one capture-cache entry.
//!
//! # Why this exists
//!
//! A cache entry used to be **eight files in a directory** (`key.bin`,
//! `meta.txt`, `out.obj`, and the five `_CL_*` IL streams) — nine inodes with
//! the directory itself. At 22.6 M entries that is ~203 M inodes, and the cost
//! is not bytes: btrfs inlines sub-`max_inline` files, so the 2026-08-04 cleanup
//! deleted 98.7 % of 4.94 M entries and returned ~17 GiB, not the ~266 GB the
//! naive arithmetic predicted. The cost is **metadata and traversal**: any
//! `find`/`du`/`rg`/`**` glob rooted at the repo walks the whole tree, which has
//! twice taken this machine down with the OOM killer (62 GB and 72 GB of anon
//! RSS in a `zsh`) and once wedged it for an hour at load 37.8.
//!
//! Folding the seven small files into one blob takes an entry from nine inodes
//! to three (dir + `entry.bin` + `out.obj`). `out.obj` cannot join them:
//! `cl.exe` runs under wibo and writes to a *path*, and that path is baked into
//! the obj as `S_OBJNAME` in `.debug$S`, where even its **length** shifts bytes.
//! The IL streams have no such constraint — [`IlBundle`] is fully in-memory and
//! replay materialises IL to a scratch directory, never to the cache — so they
//! fold and the obj does not.
//!
//! # Why the format lives in `c2-il` and not in the harness
//!
//! `c2-il` is a zero-dependency leaf that `c2-harness` already depends on, so
//! putting the codec here lets `tests/gl_alias_corpus.rs` read a blob without
//! inverting the dependency. That test scans cache entries for `_CL_*gl` files;
//! after the fold it would find none, and because it is env-gated it would
//! **silently report zero** rather than fail — absence read as success. A codec
//! it can call is the fix.
//!
//! # The format
//!
//! All integers little-endian. No `unsafe`; every read is a `try_into` on a
//! bounds-checked slice.
//!
//! ```text
//! off  size  field
//!   0     8  MAGIC     b"C2RSCAP\x02"
//!   8     4  VERSION   u32 = 2
//!  12     4  N_SECT    u32, <= MAX_SECTIONS
//!  16     8  TOTAL_LEN u64, whole-file length including this header
//!  24    32  DIGEST    32 ASCII hex = digest128(&file[HEADER_LEN..TOTAL_LEN])
//!  56          SECT[N] — 24 B each: TAG[8] NUL-padded ASCII, OFF u64, LEN u64
//! 56+24N       payloads, contiguous, in section-table order
//! ```
//!
//! Tags in the fixed order `key, meta, ex, gl, sy, in, db`; absent IL streams are
//! omitted, matching [`IlBundle::load_from_dir`]'s "missing files are skipped".
//!
//! **Canonical form is enforced on read**: strictly ascending tag ordinals (which
//! rules out duplicates and reordering in one comparison), payloads contiguous
//! from the end of the table to `TOTAL_LEN`, no padding. Any deviation is an
//! error, and every error is a cache **miss**. That makes [`encode_entry`] a
//! deterministic pure function of its inputs — testable, and free of a whole
//! class of valid-but-weird blobs that would otherwise need semantics.
//!
//! # What the blob is stronger than
//!
//! The directory layout's completion marker was "`meta.txt` exists", written
//! last. `std::fs::write` is create+truncate+write_all, not atomic, so a crash
//! after two lines landed left a *parseable* `base`+`arg` pair — a Hit with a
//! truncated argv. Here `TOTAL_LEN` catches the torn write before anything else
//! in the file is trusted, the digest catches interior corruption including a
//! corrupted section table, and `rename(2)` makes the whole entry appear at once.
//! The fold is a strict strengthening, not a neutral inode trade.

use std::fmt;

use crate::{IlBundle, IL_SUFFIXES};

/// FNV-1a 64, hand-rolled (the workspace is std-only by hard constraint).
fn fnv1a64(seed: u64, bytes: &[u8]) -> u64 {
    let mut h = seed;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// 128-bit content digest: FNV-1a-64 forward, and a second FNV-1a-64 over the
/// reversed bytes seeded with the first. Two passes with different orders make
/// the pair far harder to collide jointly than either alone.
///
/// Lives here rather than in the harness because the blob header stores one and
/// the cache key is one; a second implementation of this beside the first is the
/// "one rule, two implementations" shape this project keeps recording. Not a
/// cryptographic hash and not relied on as one — a cache hit is decided by
/// comparing the stored key material **byte-for-byte**, never by the digest.
pub fn digest128(bytes: &[u8]) -> String {
    let h1 = fnv1a64(FNV_OFFSET, bytes);
    let rev: Vec<u8> = bytes.iter().rev().copied().collect();
    let h2 = fnv1a64(h1 ^ 0x9E37_79B9_7F4A_7C15, &rev);
    format!("{h1:016x}{h2:016x}")
}

/// Container magic. The trailing byte is the format generation, so a v1-shaped
/// or future blob is refused by the very first check.
pub const MAGIC: &[u8; 8] = b"C2RSCAP\x02";

/// Format version. Bump with [`MAGIC`]'s last byte.
pub const VERSION: u32 = 2;

/// Fixed header length; also the first byte covered by [`digest128`].
pub const HEADER_LEN: usize = 56;

/// Bytes per section-table row: `TAG[8] OFF[8] LEN[8]`.
pub const SECT_LEN: usize = 24;

/// A ceiling far above the seven tags that exist, so a garbage `N_SECT` cannot
/// make the reader allocate or loop.
pub const MAX_SECTIONS: usize = 16;

/// Section tags in canonical order. `key` and `meta` are mandatory; the five IL
/// suffixes follow [`IL_SUFFIXES`] and any of them may be absent.
pub const TAGS: [&str; 7] = ["key", "meta", "ex", "gl", "sy", "in", "db"];

/// Position of `tag` in [`TAGS`], or `None` if it is not a tag we define.
fn ordinal(tag: &str) -> Option<usize> {
    TAGS.iter().position(|t| *t == tag)
}

/// Why a blob could not be decoded.
///
/// Every variant is a cache **miss** at the call site — the reason is carried
/// only so `c2rs cache show` and the tests can say which check fired, not so
/// callers can treat some as recoverable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobError {
    /// Shorter than the fixed header.
    TooShort,
    /// Not a `C2RSCAP` blob of this generation.
    BadMagic,
    /// Magic matched but `VERSION` did not.
    BadVersion,
    /// `TOTAL_LEN` disagrees with the actual file length — a torn write.
    LengthMismatch,
    /// `N_SECT` exceeds [`MAX_SECTIONS`].
    TooManySections,
    /// The section table does not fit inside the file.
    TableOverflow,
    /// A section's `OFF..OFF+LEN` runs past the end of the file.
    SectionOutOfBounds,
    /// Payloads are not contiguous from the end of the table to `TOTAL_LEN`.
    NotContiguous,
    /// Tags are not strictly ascending in [`TAGS`] order (reordered, or a
    /// duplicate).
    NotCanonical,
    /// A tag that is not in [`TAGS`].
    UnknownTag,
    /// The stored digest does not match the body.
    DigestMismatch,
    /// No `key` section.
    MissingKey,
    /// No `meta` section.
    MissingMeta,
    /// The `meta` section is not valid UTF-8.
    MetaNotUtf8,
}

impl fmt::Display for BlobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BlobError::TooShort => "shorter than the header",
            BlobError::BadMagic => "bad magic",
            BlobError::BadVersion => "unsupported version",
            BlobError::LengthMismatch => "TOTAL_LEN != file length (torn write)",
            BlobError::TooManySections => "N_SECT above the ceiling",
            BlobError::TableOverflow => "section table does not fit",
            BlobError::SectionOutOfBounds => "section runs past the end",
            BlobError::NotContiguous => "payloads are not contiguous",
            BlobError::NotCanonical => "sections are not in canonical order",
            BlobError::UnknownTag => "unknown section tag",
            BlobError::DigestMismatch => "digest does not match the body",
            BlobError::MissingKey => "no key section",
            BlobError::MissingMeta => "no meta section",
            BlobError::MetaNotUtf8 => "meta section is not UTF-8",
        };
        f.write_str(s)
    }
}

/// A decoded blob, borrowing from the buffer it was read out of.
#[derive(Debug)]
pub struct EntryBlob<'a> {
    /// The cache key material, verbatim. Compared byte-for-byte by the caller.
    pub key: &'a [u8],
    /// The `meta.txt` text, grammar unchanged by the fold.
    pub meta: &'a str,
    /// IL payloads, indexed by position in [`IL_SUFFIXES`].
    il: [Option<&'a [u8]>; IL_SUFFIXES.len()],
}

impl<'a> EntryBlob<'a> {
    /// Raw bytes of one IL stream by suffix (`"ex"`, `"gl"`, …).
    pub fn il(&self, suffix: &str) -> Option<&'a [u8]> {
        let i = IL_SUFFIXES.iter().position(|s| *s == suffix)?;
        self.il[i]
    }

    /// Rebuild the [`IlBundle`]. `base` comes from the caller's parse of
    /// [`Self::meta`] — the base name is metadata, not a section, exactly as it
    /// was when the streams were files named after it.
    pub fn bundle(&self, base: &str) -> IlBundle {
        let mut b = IlBundle::new(base.to_string());
        for (i, suffix) in IL_SUFFIXES.iter().enumerate() {
            if let Some(bytes) = self.il[i] {
                b.set(*suffix, bytes.to_vec());
            }
        }
        b
    }
}

/// Serialise one entry. Pure and byte-stable: the same inputs always give the
/// same bytes, which is what lets a test mutate what production actually writes
/// instead of hand-rolling a second implementation of the format.
pub fn encode_entry(key: &[u8], meta: &str, bundle: &IlBundle) -> Vec<u8> {
    // Gather payloads in canonical tag order.
    let mut sections: Vec<(&str, &[u8])> = Vec::with_capacity(TAGS.len());
    sections.push(("key", key));
    sections.push(("meta", meta.as_bytes()));
    for suffix in IL_SUFFIXES {
        if let Some(bytes) = bundle.get(suffix) {
            sections.push((suffix, bytes));
        }
    }

    let table_len = SECT_LEN * sections.len();
    let body_off = HEADER_LEN + table_len;
    let payload_len: usize = sections.iter().map(|(_, b)| b.len()).sum();
    let total = body_off + payload_len;

    let mut out = vec![0u8; HEADER_LEN];
    out[0..8].copy_from_slice(MAGIC);
    out[8..12].copy_from_slice(&VERSION.to_le_bytes());
    out[12..16].copy_from_slice(&(sections.len() as u32).to_le_bytes());
    out[16..24].copy_from_slice(&(total as u64).to_le_bytes());
    // DIGEST at [24..56] is filled in once the body exists.

    let mut off = body_off;
    for (tag, bytes) in &sections {
        let mut row = [0u8; SECT_LEN];
        row[..tag.len()].copy_from_slice(tag.as_bytes());
        row[8..16].copy_from_slice(&(off as u64).to_le_bytes());
        row[16..24].copy_from_slice(&(bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(&row);
        off += bytes.len();
    }
    for (_, bytes) in &sections {
        out.extend_from_slice(bytes);
    }
    debug_assert_eq!(out.len(), total);

    let digest = digest128(&out[HEADER_LEN..]);
    out[24..56].copy_from_slice(digest.as_bytes());
    out
}

/// Parse a blob, enforcing canonical form. Order matters: `TOTAL_LEN` is checked
/// before anything else in the file is trusted, so a truncated write is rejected
/// by a length comparison rather than by whatever the table happens to say.
pub fn decode_entry(bytes: &[u8]) -> Result<EntryBlob<'_>, BlobError> {
    if bytes.len() < HEADER_LEN {
        return Err(BlobError::TooShort);
    }
    if &bytes[0..8] != MAGIC {
        return Err(BlobError::BadMagic);
    }
    if u32::from_le_bytes(bytes[8..12].try_into().unwrap()) != VERSION {
        return Err(BlobError::BadVersion);
    }
    let n_sect = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let total = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    // The torn-write catch, before the table is read.
    if total != bytes.len() as u64 {
        return Err(BlobError::LengthMismatch);
    }
    if n_sect > MAX_SECTIONS {
        return Err(BlobError::TooManySections);
    }
    let table_len = SECT_LEN * n_sect; // bounded by MAX_SECTIONS, cannot overflow
    let body_off = HEADER_LEN + table_len;
    if body_off > bytes.len() {
        return Err(BlobError::TableOverflow);
    }
    // Covers the section table too, so a corrupted offset cannot survive.
    if digest128(&bytes[HEADER_LEN..]) != String::from_utf8_lossy(&bytes[24..56]) {
        return Err(BlobError::DigestMismatch);
    }

    let mut key: Option<&[u8]> = None;
    let mut meta: Option<&[u8]> = None;
    let mut il: [Option<&[u8]>; IL_SUFFIXES.len()] = Default::default();
    let mut prev_ord: Option<usize> = None;
    let mut expect_off = body_off;

    for i in 0..n_sect {
        let row = &bytes[HEADER_LEN + i * SECT_LEN..HEADER_LEN + (i + 1) * SECT_LEN];
        let tag_end = row[..8].iter().position(|b| *b == 0).unwrap_or(8);
        let Ok(tag) = std::str::from_utf8(&row[..tag_end]) else {
            return Err(BlobError::UnknownTag);
        };
        // Any NUL padding must be trailing; a tag with an interior NUL would
        // otherwise alias a shorter one.
        if row[tag_end..8].iter().any(|b| *b != 0) {
            return Err(BlobError::UnknownTag);
        }
        let Some(ord) = ordinal(tag) else {
            return Err(BlobError::UnknownTag);
        };
        // Strictly ascending: rules out reordering and duplicates at once.
        if prev_ord.is_some_and(|p| ord <= p) {
            return Err(BlobError::NotCanonical);
        }
        prev_ord = Some(ord);

        let off = u64::from_le_bytes(row[8..16].try_into().unwrap());
        let len = u64::from_le_bytes(row[16..24].try_into().unwrap());
        let (Ok(off), Ok(len)) = (usize::try_from(off), usize::try_from(len)) else {
            return Err(BlobError::SectionOutOfBounds);
        };
        let Some(end) = off.checked_add(len) else {
            return Err(BlobError::SectionOutOfBounds);
        };
        if end > bytes.len() {
            return Err(BlobError::SectionOutOfBounds);
        }
        if off != expect_off {
            return Err(BlobError::NotContiguous);
        }
        expect_off = end;

        let payload = &bytes[off..end];
        match tag {
            "key" => key = Some(payload),
            "meta" => meta = Some(payload),
            _ => {
                // `ordinal` already accepted it, so it is one of IL_SUFFIXES.
                let i = IL_SUFFIXES.iter().position(|s| *s == tag).unwrap();
                il[i] = Some(payload);
            }
        }
    }
    // No trailing slack: the payloads must reach exactly TOTAL_LEN.
    if expect_off != bytes.len() {
        return Err(BlobError::NotContiguous);
    }

    let Some(key) = key else {
        return Err(BlobError::MissingKey);
    };
    let Some(meta) = meta else {
        return Err(BlobError::MissingMeta);
    };
    let Ok(meta) = std::str::from_utf8(meta) else {
        return Err(BlobError::MetaNotUtf8);
    };
    Ok(EntryBlob { key, meta, il })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Vec<u8>, String, IlBundle) {
        let mut b = IlBundle::new("_CL_deadbeef");
        b.set("ex", vec![0x5B, 0x80, 0x54, 0x0A, 1, 2, 3]);
        b.set("gl", vec![9; 40]);
        b.set("db", vec![7; 5]);
        (
            b"context\x00src-arg\x00foo.cpp".to_vec(),
            "c2rs-capture-cache/v2\nbase _CL_deadbeef\narg -Fo\n".to_string(),
            b,
        )
    }

    fn encoded() -> Vec<u8> {
        let (k, m, b) = fixture();
        encode_entry(&k, &m, &b)
    }

    #[test]
    fn encode_is_byte_stable_and_decode_is_its_inverse() {
        let (k, m, b) = fixture();
        assert_eq!(encode_entry(&k, &m, &b), encode_entry(&k, &m, &b));
        let bytes = encode_entry(&k, &m, &b);
        let blob = decode_entry(&bytes).expect("round trip");
        assert_eq!(blob.key, &k[..]);
        assert_eq!(blob.meta, m);
        let back = blob.bundle("_CL_deadbeef");
        assert_eq!(back.base_name, b.base_name);
        assert_eq!(back.files, b.files);
    }

    #[test]
    fn an_absent_il_stream_stays_absent() {
        let (k, m, b) = fixture();
        let bytes = encode_entry(&k, &m, &b);
        let blob = decode_entry(&bytes).unwrap();
        // The fixture omits `sy` and `in`; they must not reappear as empty.
        assert!(blob.il("sy").is_none());
        assert!(blob.il("in").is_none());
        assert!(!blob.bundle("x").files.contains_key("sy"));
        assert_eq!(blob.il("ex").unwrap().len(), 7);
    }

    #[test]
    fn an_empty_bundle_still_round_trips() {
        let b = IlBundle::new("_CL_0");
        let bytes = encode_entry(b"k", "m", &b);
        let blob = decode_entry(&bytes).unwrap();
        assert_eq!(blob.key, b"k");
        assert_eq!(blob.meta, "m");
        assert!(blob.bundle("_CL_0").files.is_empty());
    }

    #[test]
    fn truncation_reads_as_a_length_mismatch_not_as_garbage() {
        let bytes = encoded();
        for frac in [1, 2, 3] {
            let cut = bytes.len() * frac / 4;
            assert_eq!(
                decode_entry(&bytes[..cut]).unwrap_err(),
                if cut < HEADER_LEN {
                    BlobError::TooShort
                } else {
                    BlobError::LengthMismatch
                },
                "truncated to {cut} of {}",
                bytes.len()
            );
        }
    }

    #[test]
    fn a_flipped_payload_byte_fails_the_digest() {
        let mut bytes = encoded();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        assert_eq!(decode_entry(&bytes).unwrap_err(), BlobError::DigestMismatch);
    }

    #[test]
    fn a_corrupted_section_table_cannot_survive_the_digest() {
        // The digest covers the table, so an attacker-shaped offset edit is
        // caught before the bounds checks even matter.
        let mut bytes = encoded();
        bytes[HEADER_LEN + 8] ^= 0x01;
        assert_eq!(decode_entry(&bytes).unwrap_err(), BlobError::DigestMismatch);
    }

    #[test]
    fn bad_magic_and_bad_version_are_distinguished() {
        let mut bytes = encoded();
        bytes[0] = b'X';
        assert_eq!(decode_entry(&bytes).unwrap_err(), BlobError::BadMagic);

        let mut bytes = encoded();
        bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(decode_entry(&bytes).unwrap_err(), BlobError::BadVersion);
    }

    #[test]
    fn a_lying_total_len_is_refused() {
        let mut bytes = encoded();
        let n = bytes.len() as u64 + 1;
        bytes[16..24].copy_from_slice(&n.to_le_bytes());
        assert_eq!(decode_entry(&bytes).unwrap_err(), BlobError::LengthMismatch);
    }

    #[test]
    fn an_absurd_section_count_is_refused_before_any_allocation() {
        let mut bytes = encoded();
        bytes[12..16].copy_from_slice(&(u32::MAX).to_le_bytes());
        assert_eq!(
            decode_entry(&bytes).unwrap_err(),
            BlobError::TooManySections
        );
    }

    /// Rebuilt by hand rather than by flipping bytes, because a flip would fail
    /// the digest first and prove nothing about the ordering check.
    #[test]
    fn reordered_and_duplicated_sections_are_refused() {
        for rows in [
            vec![("meta", &b"m"[..]), ("key", &b"k"[..])],
            vec![("key", &b"k"[..]), ("key", &b"k2"[..])],
        ] {
            let bytes = hand_encode(&rows);
            assert_eq!(
                decode_entry(&bytes).unwrap_err(),
                BlobError::NotCanonical,
                "rows {:?}",
                rows.iter().map(|(t, _)| *t).collect::<Vec<_>>()
            );
        }
        assert_eq!(
            decode_entry(&hand_encode(&[("key", &b"k"[..]), ("zz", &b"?"[..])])).unwrap_err(),
            BlobError::UnknownTag
        );
        assert_eq!(
            decode_entry(&hand_encode(&[("meta", &b"m"[..])])).unwrap_err(),
            BlobError::MissingKey
        );
        assert_eq!(
            decode_entry(&hand_encode(&[("key", &b"k"[..])])).unwrap_err(),
            BlobError::MissingMeta
        );
    }

    #[test]
    fn a_gap_between_payloads_is_refused() {
        let mut bytes = hand_encode(&[("key", &b"k"[..]), ("meta", &b"m"[..])]);
        // Push `meta` one byte later and grow the file to match, so the only
        // broken invariant is contiguity.
        let row = HEADER_LEN + SECT_LEN;
        let off = u64::from_le_bytes(bytes[row + 8..row + 16].try_into().unwrap());
        bytes[row + 8..row + 16].copy_from_slice(&(off + 1).to_le_bytes());
        bytes.insert(bytes.len() - 1, 0);
        let n = bytes.len() as u64;
        bytes[16..24].copy_from_slice(&n.to_le_bytes());
        let d = digest128(&bytes[HEADER_LEN..]);
        bytes[24..56].copy_from_slice(d.as_bytes());
        assert_eq!(decode_entry(&bytes).unwrap_err(), BlobError::NotContiguous);
    }

    #[test]
    fn a_section_running_past_the_end_is_refused() {
        let mut bytes = hand_encode(&[("key", &b"k"[..]), ("meta", &b"m"[..])]);
        let row = HEADER_LEN;
        bytes[row + 16..row + 24].copy_from_slice(&u64::MAX.to_le_bytes());
        let d = digest128(&bytes[HEADER_LEN..]);
        bytes[24..56].copy_from_slice(d.as_bytes());
        assert_eq!(
            decode_entry(&bytes).unwrap_err(),
            BlobError::SectionOutOfBounds
        );
    }

    #[test]
    fn the_digest_covers_the_body_and_not_the_header() {
        let bytes = encoded();
        assert_eq!(
            String::from_utf8_lossy(&bytes[24..56]),
            digest128(&bytes[HEADER_LEN..])
        );
    }

    /// A second encoder, used ONLY to build blobs `encode_entry` refuses to
    /// emit. Everything a valid blob asserts is checked against the real one.
    fn hand_encode(rows: &[(&str, &[u8])]) -> Vec<u8> {
        let body_off = HEADER_LEN + SECT_LEN * rows.len();
        let total: usize = body_off + rows.iter().map(|(_, b)| b.len()).sum::<usize>();
        let mut out = vec![0u8; HEADER_LEN];
        out[0..8].copy_from_slice(MAGIC);
        out[8..12].copy_from_slice(&VERSION.to_le_bytes());
        out[12..16].copy_from_slice(&(rows.len() as u32).to_le_bytes());
        out[16..24].copy_from_slice(&(total as u64).to_le_bytes());
        let mut off = body_off;
        for (tag, b) in rows {
            let mut row = [0u8; SECT_LEN];
            row[..tag.len()].copy_from_slice(tag.as_bytes());
            row[8..16].copy_from_slice(&(off as u64).to_le_bytes());
            row[16..24].copy_from_slice(&(b.len() as u64).to_le_bytes());
            out.extend_from_slice(&row);
            off += b.len();
        }
        for (_, b) in rows {
            out.extend_from_slice(b);
        }
        let d = digest128(&out[HEADER_LEN..]);
        out[24..56].copy_from_slice(d.as_bytes());
        out
    }

    #[test]
    fn digest_is_deterministic_and_input_sensitive() {
        assert_eq!(digest128(b"abc"), digest128(b"abc"));
        assert_ne!(digest128(b"abc"), digest128(b"abd"));
        assert_ne!(digest128(b""), digest128(b"\0"));
        assert_eq!(digest128(b"abc").len(), 32);
    }
}
