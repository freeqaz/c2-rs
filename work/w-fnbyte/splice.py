import io
p = "crates/c2-core/src/lib.rs"
lines = open(p, encoding="utf-8").read().split("\n")
# lines[] is 0-based; file line N -> lines[N-1]
assert lines[516].strip() == "for &fi in &order {", lines[516]
assert lines[652].strip() == "}", lines[652]
new = """            // **The per-function COMDAT body comes from `comdat::comdat_function_body`,
            // which is the ONE composition** (board #322). It used to be this
            // loop's inline `match`, reachable only from here — so the standing
            // per-function alarm (FUNCTION BYTE MATCH) could not ask the port
            // for a `Tail`/`Framed`/`Seq`/`CondPair` body at all and declined to
            // grade 9,375 emitted functions. Lifting it changes no byte: the
            // arms below moved verbatim, and `crates/c2-core/src/comdat.rs`
            // carries the reason the harness must call this and never a copy.
            for &fi in &order {
                let f = &funcs[fi];
                let body = comdat::comdat_function_body(f, mode)?;
                placed.push(coff::Function {
                    name: &f.mangled_name,
                    text_offset: 0,
                    calls: body.calls,
                    is_float: f.touches_floating_point(),
                    fp_refs: Vec::new(),
                    data_refs: body.data_refs,
                    frame: body.frame,
                    label_lead: leads[fi],
                });
                texts.push(body.text);
            }"""
out = lines[:516] + new.split("\n") + lines[653:]
open(p, "w", encoding="utf-8").write("\n".join(out))
print("spliced", len(lines), "->", len(out))
