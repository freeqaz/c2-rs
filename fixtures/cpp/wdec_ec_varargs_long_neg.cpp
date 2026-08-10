// w-decouple `_neg` — the same refusal on a name the INCUMBENT walk already
// binds, which makes this cell a repair of a PRE-EXISTING hole rather than a
// guard on this lane's own widening.
//
// `wdec_ec_varargs_neg.cpp`'s name is three bytes, so the incumbent
// `INLINE_NAME_MAX` clause refused it for an unrelated reason and the varargs
// question never arose. This name is sixteen. It has bound since W-EXTDATA
// replaced `looks_mangled` with a length test (#1721) — an undecorated name in
// the string table, exactly the encoding path that clause says is modeled — and
// `mangled_is_varargs` has never been able to answer for it, because there is no
// `@@` and no trailing `ZZ` in an `extern "C"` name of any length.
//
// The clause is guarded on `!looks_mangled`, not on the name's length, so it
// covers this cell too. `_vswprintf_s_l` (14 bytes, `extern "C"`, and the name
// W-EXTDATA's rung is written about) is the shape that made the hole reachable.

extern "C" int v_long_name_here(int a, ...) { return a + 1; }
