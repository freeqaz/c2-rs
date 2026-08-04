//! The little-endian byte sink and the COFF string table.


/// A little-endian byte sink.
pub(crate) struct Buf(pub(crate) Vec<u8>);
impl Buf {
    pub(crate) fn new() -> Self {
        Buf(Vec::new())
    }
    /// Pre-sized sink for the whole-obj emitters: the layout pass has already
    /// computed the symbol-table offset, so the final size is known to within
    /// the string table. Capacity is invisible in the output — this only
    /// removes the realloc-and-copy churn of growing from empty.
    pub(crate) fn with_capacity(n: usize) -> Self {
        Buf(Vec::with_capacity(n))
    }
    pub(crate) fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub(crate) fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(crate) fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(crate) fn i16(&mut self, v: i16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(crate) fn bytes(&mut self, v: &[u8]) {
        self.0.extend_from_slice(v);
    }
    /// 8-byte NUL-padded short name (`len <= 8`).
    pub(crate) fn name8(&mut self, name: &str) {
        let b = name.as_bytes();
        assert!(b.len() <= 8, "short name > 8 bytes: {name}");
        self.0.extend_from_slice(b);
        for _ in b.len()..8 {
            self.0.push(0);
        }
    }
}


/// COFF string table: `Size:u32(incl self)` + NUL-terminated names in
/// first-reference order. Offsets returned are from the table base (so the
/// first name is at offset 4).
pub(crate) struct StringTable {
    pub(crate) names: Vec<(String, u32)>,
    pub(crate) cursor: u32,
}
impl StringTable {
    pub(crate) fn new() -> Self {
        StringTable {
            names: Vec::new(),
            cursor: 4, // past the 4-byte size word
        }
    }
    /// Intern a name (append if new), returning its byte offset.
    pub(crate) fn intern(&mut self, name: &str) -> u32 {
        if let Some((_, off)) = self.names.iter().find(|(n, _)| n == name) {
            return *off;
        }
        let off = self.cursor;
        self.cursor += name.len() as u32 + 1; // + NUL
        self.names.push((name.to_string(), off));
        off
    }
    pub(crate) fn finish(self) -> Vec<u8> {
        let mut out = Buf::new();
        out.u32(self.cursor); // Size includes the size word itself
        for (name, _) in &self.names {
            out.bytes(name.as_bytes());
            out.u8(0);
        }
        out.0
    }
}
