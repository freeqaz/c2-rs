#!/usr/bin/env python3
"""One-shot: append this lane's `gl.rs` cells to that file's test module."""
ADD = r'''
    // ---------------------------------------------------------------- w-mmioclose
    //
    // `gl_function_attrs` — the `.gl` function record's attribute byte, and the
    // one bit of it two fences read.

    /// One measured `.gl` FUNCTION record, spelled field by field.
    ///
    /// The literal bytes are `src/xdk/nuispeech/mmio.cpp`'s own, so the fixture
    /// is a transcription and not an invention:
    ///
    /// ```text
    ///   mmioGetInfo\0  86 01 05 04 00 08 00  80 18 10 00 00 00 00  80 3d 0c 00 00  00  3d  68
    ///   mmioFlush\0    86 01 05 04 00 08 00  80 0f 10 00 00 00 00  80 5d 0f 00 00  80 4d 01 00 00  10  28
    /// ```
    ///
    /// `srcpos` is `Ok(b)` for the one-byte form and `Err(v)` for the
    /// `80 <LE32>` escape, which is the same shape as the offset field.
    fn fn_record(name: &str, off: u32, srcpos: Result<u8, u32>, size: u8, attr: u8) -> Vec<u8> {
        let mut v = vec![0x00];
        v.extend_from_slice(name.as_bytes());
        v.push(0x00);
        // TYPE, then the seven framing bytes `gl_offset_framed` requires.
        v.extend_from_slice(&[0x86, 0x01, 0x05, 0x04, 0x00, 0x08, 0x00]);
        v.extend_from_slice(&[0x80, 0x18, 0x10, 0x00, 0x00, 0x00, 0x00]);
        v.push(0x80);
        v.extend_from_slice(&off.to_le_bytes());
        match srcpos {
            Ok(b) => v.push(b),
            Err(w) => {
                v.push(0x80);
                v.extend_from_slice(&w.to_le_bytes());
            }
        }
        v.push(size);
        v.push(attr);
        v
    }

    /// The measured attribute bytes, named. Every one is a real record from
    /// `work/w-mmioclose/probe/glgrid.cpp` or from `mmio.cpp`.
    const ATTR_PLAIN: u8 = 0x68;
    const ATTR_NOINLINE: u8 = 0x28;
    const ATTR_INLINE_KEYWORD: u8 = 0xC8;
    const ATTR_STATIC: u8 = 0x48;
    const ATTR_STATIC_NOINLINE: u8 = 0x08;

    /// **The positive, and it is the target TU's own two records.**
    ///
    /// `mmioGetInfo` takes the one-byte source position and `mmioFlush` the
    /// `80 <LE32>` escape, so one fixture exercises both encodings — which is
    /// what the first version of this reader got wrong, and got wrong SILENTLY,
    /// by returning an attribute of `0x00` (bit clear, i.e. the permissive
    /// reading) for every escaped record.
    #[test]
    fn the_attribute_byte_separates_noinline_from_plain() {
        let mut gl = fn_record("mmioGetInfo", 0x0c3d, Ok(0x00), 0x3d, ATTR_PLAIN);
        gl.extend(fn_record("mmioFlush", 0x0f5d, Err(333), 0x10, ATTR_NOINLINE));
        let attrs = gl_function_attrs(&gl).expect("both records decode");
        assert_eq!(attrs.get("mmioGetInfo"), Some(&ATTR_PLAIN));
        assert_eq!(attrs.get("mmioFlush"), Some(&ATTR_NOINLINE));
        assert_eq!(
            gl_noinline_names(&gl).expect("decodes"),
            ["mmioFlush".to_string()].into_iter().collect(),
            "exactly the record whose FN_FLAG_INLINABLE is clear"
        );
    }

    /// **The must-fail mutation, and it is the one that matters**: flip the one
    /// bit and the set inverts. Without it the test above would pass just as
    /// happily against a reader keyed on the whole byte, on the size, or on the
    /// record's position.
    #[test]
    fn flipping_only_fn_flag_inlinable_inverts_the_answer() {
        let plain = fn_record("g", 0x10, Ok(0x00), 0x10, ATTR_PLAIN);
        let flipped = fn_record("g", 0x10, Ok(0x00), 0x10, ATTR_PLAIN & !FN_FLAG_INLINABLE);
        assert!(gl_noinline_names(&plain).expect("decodes").is_empty());
        assert_eq!(
            gl_noinline_names(&flipped).expect("decodes"),
            ["g".to_string()].into_iter().collect()
        );
        // …and the bit is the ONLY thing that moved: the two records differ in
        // exactly one byte, and in exactly one bit of it.
        let d: Vec<usize> = plain
            .iter()
            .zip(&flipped)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(d.len(), 1, "one byte");
        assert_eq!(plain[d[0]] ^ flipped[d[0]], FN_FLAG_INLINABLE, "one bit");
    }

    /// The other measured attribute bytes, so the reader is pinned on the axes
    /// that are NOT `noinline`. `inline` and `__forceinline` both read `0xC8`
    /// and `static` reads `0x48`; all three keep bit 6, which is the statement
    /// that the bit is the inliner's legality flag and not a general "this
    /// declaration carries an attribute" marker.
    #[test]
    fn the_inline_keyword_and_static_do_not_clear_the_bit() {
        for (name, attr) in [("g_inl", ATTR_INLINE_KEYWORD), ("g_static", ATTR_STATIC)] {
            let gl = fn_record(name, 0x10, Ok(0x00), 0x10, attr);
            assert!(
                gl_noinline_names(&gl).expect("decodes").is_empty(),
                "{name} keeps FN_FLAG_INLINABLE"
            );
        }
        // `static __declspec(noinline)` clears bit 6 on top of static's bit 5,
        // which is what makes the two axes independent rather than one field.
        let gl = fn_record("g_sn", 0x10, Ok(0x00), 0x10, ATTR_STATIC_NOINLINE);
        assert_eq!(
            gl_noinline_names(&gl).expect("decodes"),
            ["g_sn".to_string()].into_iter().collect()
        );
        assert_eq!(ATTR_STATIC & !FN_FLAG_INLINABLE, ATTR_STATIC_NOINLINE);
    }

    /// **A file with an unrecognised field encoding yields NOTHING**, not "the
    /// records I could read".
    ///
    /// This is `Bindings::per_record`'s standard (`w-inlfence`, #2220–#2227) and
    /// the direction is why: the consumer reads a clear bit as *"c2 keeps this
    /// call"*, so a record decoded at the wrong displacement is a permission
    /// granted from an unrelated byte. Each clause below trips a DIFFERENT one,
    /// and the good record in front of it is what proves the refusal is total
    /// rather than local.
    #[test]
    fn one_unreadable_record_refuses_the_whole_file() {
        let good = fn_record("good", 0x10, Ok(0x00), 0x10, ATTR_PLAIN);
        assert!(gl_function_attrs(&good).is_some(), "the control decodes");

        // (a) SRCPOS is >= 0x80 and is not the 0x80 escape.
        let mut a = good.clone();
        a.extend(fn_record("bad", 0x20, Ok(0x00), 0x10, ATTR_PLAIN));
        let n = a.len();
        a[n - 3] = 0x81;
        assert!(gl_function_attrs(&a).is_none(), "srcpos: unknown encoding");

        // (b) SIZE is >= 0x80 — the attribute would be one byte further along
        // and this reader would hand back an unrelated value.
        let mut b = good.clone();
        b.extend(fn_record("bad", 0x20, Ok(0x00), 0x80, ATTR_PLAIN));
        assert!(gl_function_attrs(&b).is_none(), "size: escaped");

        // (c) the record is truncated before its attribute byte.
        let mut c = good.clone();
        c.extend(fn_record("bad", 0x20, Ok(0x00), 0x10, ATTR_PLAIN));
        c.pop();
        assert!(gl_function_attrs(&c).is_none(), "no attribute byte");

        // (d) a framed offset with no name run near enough to be its record's.
        let mut d = good.clone();
        let mut orphan = fn_record("x", 0x30, Ok(0x00), 0x10, ATTR_PLAIN);
        let at = 3; // just past `00 x 00`
        for _ in 0..MAX_NAME_TO_OFFSET + 1 {
            orphan.insert(at, 0x00);
        }
        d.extend(orphan);
        assert!(gl_function_attrs(&d).is_none(), "name too far from the offset");

        // (e) one name, two records, two different attribute bytes.
        let mut e = good.clone();
        e.extend(fn_record("good", 0x20, Ok(0x00), 0x10, ATTR_NOINLINE));
        assert!(gl_function_attrs(&e).is_none(), "one name, two answers");
    }

    /// **`None` and the empty set are different facts**, and a consumer that
    /// collapsed them would read "this reader has nothing to say" as "nothing
    /// here is `noinline`" — which is the permissive direction.
    #[test]
    fn a_refused_file_is_not_an_empty_noinline_set() {
        let mut bad = fn_record("bad", 0x20, Ok(0x00), 0x80, ATTR_PLAIN);
        bad.push(0x00);
        assert_eq!(gl_noinline_names(&bad), None);
        let empty: &[u8] = &[];
        assert_eq!(
            gl_noinline_names(empty),
            Some(std::collections::BTreeSet::new()),
            "a .gl with no function record at all HAS an answer, and it is empty"
        );
    }
}'''

p = 'crates/c2-il/src/func/gl.rs'
s = open(p).read().rstrip()
assert s.endswith('}')
s = s[: s.rfind('}')] + ADD + '\n'
open(p, 'w').write(s)
print('done')
