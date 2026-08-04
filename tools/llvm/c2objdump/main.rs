//! c2objdump — print what `crates/c2-obj` sees in an obj, as TSV.
//!
//! `ObjImage` is the reader the correctness gate uses and the reader whose
//! `section_names()` feeds factor C in `gap.rs`. Everything it decodes is
//! printed here so `tools/llvm/xcheck.py` can hold it against `llvm-readobj`.
//!
//! Fail-closed is preserved and made *visible*: when `ObjImage` refuses an obj
//! (its walk returns `None`) this prints `REFUSED\t<path>` rather than an empty
//! section list, because an empty list read as agreement is exactly the
//! absence-as-success failure the cross-check exists to avoid.
//!
//! Output:
//!   OBJ\t<abs path>
//!   TS\t<timestamp>
//!   SEC\t<section name>        (one per section, in section order)
//!   FN\t<comdat leader>        (one per `.text*` COMDAT, in section order)

use c2_obj::ObjImage;
use std::io::Write;

fn main() {
    let out = std::io::stdout();
    let mut w = std::io::BufWriter::new(out.lock());
    let mut any = false;
    for arg in std::env::args().skip(1) {
        let abs = std::fs::canonicalize(&arg).unwrap_or_else(|_| std::path::PathBuf::from(&arg));
        let bytes = match std::fs::read(&arg) {
            Ok(b) => b,
            Err(e) => {
                let _ = writeln!(w, "REFUSED\t{}\t{}", abs.display(), e);
                continue;
            }
        };
        let img = ObjImage::new(bytes);
        let secs = img.section_names();
        let fns = img.text_comdat_functions();
        match (secs, fns) {
            (Some(secs), Some(fns)) => {
                any = true;
                let _ = writeln!(w, "OBJ\t{}", abs.display());
                if let Some(ts) = img.timestamp() {
                    let _ = writeln!(w, "TS\t{}", ts);
                }
                for s in secs {
                    let _ = writeln!(w, "SEC\t{}", s);
                }
                for f in fns {
                    let _ = writeln!(w, "FN\t{}", f);
                }
            }
            _ => {
                let _ = writeln!(w, "REFUSED\t{}", abs.display());
            }
        }
    }
    let _ = w.flush();
    if !any {
        // Nothing decoded: exit non-zero so a caller cannot read the silence
        // as "everything agreed".
        std::process::exit(2);
    }
}
