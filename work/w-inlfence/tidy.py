#!/usr/bin/env python3
"""w-inlfence — drop the vestigial `needle` column from inline_fence.rs.

The assertions moved from name lookup to key counting when the opaque twins
turned out not to pair their `.gl` names; this removes the tuple field they no
longer read, so the test does not carry an unused discriminator that a reader
would take for a live one.
"""
p = "crates/c2-harness/tests/inline_fence.rs"
s = open(p).read()

s = s.replace(
    """/// Capture one source and return `(mangled name, census key)` per `.ex` body,
/// plus whether `PortC2` accepts the whole TU.
///
/// The names come from the census's own `reported_name`, which is `None` when
/// the positional pairing is not meaningful; every cell below is a TU small
/// enough to pair, and a `None` would show up as an unmatched needle rather
/// than as a silently skipped assertion.""",
    """/// Capture one source and return `(mangled name, census key)` per `.ex` body,
/// plus whether `IlBundle::functions` — the port's own acceptance path —
/// accepts the whole TU.
///
/// The name is the census's own `reported_name` and is empty when the
/// positional pairing is not meaningful. It is carried for the assertion
/// messages only; every claim below is counted over the KEYS, for the reason
/// [`fenced`] states.""",
)

s = s.replace(
    """    // (tag, needle, local source, opaque source)
    let pairs: [(&str, &str, &str, &str); 4] = [
        (
            "tail",
            "?wif_t_use@@",
""",
    """    // (tag, local source, opaque source)
    let pairs: [(&str, &str, &str); 4] = [
        (
            "tail",
""",
)
for n in ('"?wif_s_use@@",\n', '"?SplitMs@",\n', '"?wif_i_use@@",\n'):
    s = s.replace("            " + n, "", 1)
s = s.replace("    for (tag, _needle, local, opaque) in pairs {",
              "    for (tag, local, opaque) in pairs {")
open(p, "w").write(s)
print("ok")
