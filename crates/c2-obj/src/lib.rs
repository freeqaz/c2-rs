//! COFF object handling for the differential compare.
//!
//! The oracle criterion is `port(IL) == c2(IL)` byte-exact — **except** the
//! 4-byte COFF `TimeDateStamp` at file offset 4..8, which is the only field
//! that varies between otherwise-identical rebuilds of the same source. So the
//! comparison always runs on *normalized* bytes with those four zeroed.
//!
//! (Reference: `msvc-src/docs/IL_CHANNEL_PROBE.md` — COFF TimeDateStamp
//! determinism note, offset 4-7. Verified empirically: identical source →
//! byte-identical `.obj` apart from those four bytes.)

/// A COFF `.obj` image: just its raw bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjImage(pub Vec<u8>);

/// Result of comparing two [`ObjImage`]s on their normalized bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjDiff {
    /// Normalized bytes are identical.
    Identical,
    /// Normalized bytes differ. `first_offset` is the first differing byte
    /// index (or `min(a_len, b_len)` when one is a prefix of the other).
    Differs {
        first_offset: usize,
        a_len: usize,
        b_len: usize,
    },
}

/// Byte offset of the COFF `TimeDateStamp` field.
const TIMESTAMP_OFFSET: usize = 4;
const TIMESTAMP_END: usize = 8;

impl ObjImage {
    pub fn new(bytes: Vec<u8>) -> Self {
        ObjImage(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The raw COFF `TimeDateStamp` (LE u32 at offset 4), or `None` if the
    /// image is too short to contain it.
    pub fn timestamp(&self) -> Option<u32> {
        if self.0.len() >= TIMESTAMP_END {
            Some(u32::from_le_bytes([
                self.0[4], self.0[5], self.0[6], self.0[7],
            ]))
        } else {
            None
        }
    }

    /// A clone of the bytes with the 4-byte `TimeDateStamp` zeroed. Guards the
    /// length: images shorter than 8 bytes are returned unchanged.
    pub fn normalized(&self) -> Vec<u8> {
        let mut v = self.0.clone();
        if v.len() >= TIMESTAMP_END {
            for b in &mut v[TIMESTAMP_OFFSET..TIMESTAMP_END] {
                *b = 0;
            }
        }
        v
    }

    /// Compare two images on their normalized (timestamp-zeroed) bytes.
    pub fn diff(a: &ObjImage, b: &ObjImage) -> ObjDiff {
        let na = a.normalized();
        let nb = b.normalized();
        let common = na.len().min(nb.len());
        for i in 0..common {
            if na[i] != nb[i] {
                return ObjDiff::Differs {
                    first_offset: i,
                    a_len: na.len(),
                    b_len: nb.len(),
                };
            }
        }
        if na.len() != nb.len() {
            return ObjDiff::Differs {
                first_offset: common,
                a_len: na.len(),
                b_len: nb.len(),
            };
        }
        ObjDiff::Identical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_obj() -> Vec<u8> {
        // A plausible-ish COFF header prefix: machine word (POWERPCBE 0x01F2),
        // section count, then a timestamp, then arbitrary payload.
        let mut v = vec![0xF2, 0x01, 0x03, 0x00]; // machine + nsections
        v.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // TimeDateStamp
        v.extend_from_slice(&[0x10, 0x20, 0x30, 0x40, 0x50, 0x60]); // payload
        v
    }

    #[test]
    fn timestamp_reads_offset_4_le() {
        let obj = ObjImage::new(base_obj());
        assert_eq!(obj.timestamp(), Some(0xDDCCBBAA));
        assert_eq!(ObjImage::new(vec![0, 1, 2]).timestamp(), None);
    }

    #[test]
    fn normalized_zeroes_timestamp_only() {
        let obj = ObjImage::new(base_obj());
        let n = obj.normalized();
        assert_eq!(&n[0..4], &[0xF2, 0x01, 0x03, 0x00]);
        assert_eq!(&n[4..8], &[0, 0, 0, 0]);
        assert_eq!(&n[8..], &[0x10, 0x20, 0x30, 0x40, 0x50, 0x60]);
        // Original untouched.
        assert_eq!(&obj.as_bytes()[4..8], &[0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn differ_only_in_timestamp_compares_identical() {
        let a = ObjImage::new(base_obj());
        let mut bb = base_obj();
        bb[4] = 0x11;
        bb[5] = 0x22;
        bb[6] = 0x33;
        bb[7] = 0x44;
        let b = ObjImage::new(bb);
        assert_eq!(ObjImage::diff(&a, &b), ObjDiff::Identical);
    }

    #[test]
    fn difference_elsewhere_reports_offset() {
        let a = ObjImage::new(base_obj());
        let mut bb = base_obj();
        bb[9] = 0x99; // offset 9 is in the payload
        let b = ObjImage::new(bb);
        match ObjImage::diff(&a, &b) {
            ObjDiff::Differs { first_offset, .. } => assert_eq!(first_offset, 9),
            ObjDiff::Identical => panic!("expected a difference at offset 9"),
        }
    }

    #[test]
    fn length_mismatch_reports_at_common_len() {
        let a = ObjImage::new(base_obj());
        let mut bb = base_obj();
        bb.push(0x77);
        let b = ObjImage::new(bb);
        match ObjImage::diff(&a, &b) {
            ObjDiff::Differs {
                first_offset,
                a_len,
                b_len,
            } => {
                assert_eq!(first_offset, a.len());
                assert_eq!(a_len, a.len());
                assert_eq!(b_len, a.len() + 1);
            }
            ObjDiff::Identical => panic!("expected a length difference"),
        }
    }
}
