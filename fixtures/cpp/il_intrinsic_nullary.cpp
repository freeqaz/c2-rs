// **Negative** — the `0x40` intrinsic-call token with an EMPTY argument list.
// The three intrinsics must keep refusing; the fixture exists to pin the *shape*
// of the token, not to be lowered. (`n_lwsync` is deliberately in class: it is
// an ordinary tail call, which is the whole point of it being here.)
//
// This is the fixture that decides between the two readings of `0x40`. The
// competing hypothesis was `40 <TYPE> <varint>` — the shape `2C` (convert),
// `99`/`9B` (member bind) and `5C` all have, and the one an earlier session
// assumed. A one-argument intrinsic cannot separate them: the varint would just
// eat the first byte of the argument and the parse would fail somewhere
// downstream. A **zero-argument** one can, because then the `4C` apply sits
// immediately after the result type and a varint would swallow it:
//
//   n_break     33 86 41 74 80 1f 02 00 00 | 40 82 07 03       | 4c 4b
//                 selector 543 (int)         token, ret void     apply, discard
//     -> 0fe00016   twi 31,r0,22            (one instruction)
//
//   n_retaddr   33 86 41 74 80 e5 00 00 00 | 40 86 43 83 08    | 4c
//                 selector 229              token, ret void*     apply
//               41 86 43 83 08 3a … (the ordinary int-return plumbing follows)
//     -> 7c6802a6   mflr r3
//
//   n_mftb      33 86 41 74 80 9c 07 00 00 | 40 88 82 23       | 4c
//                 selector 1948             token, ret u64       apply
//     -> 7c6c42e6   mftb r3   (mfspr r3,268)
//
// `n_retaddr` is the load-bearing one: its `4C` is followed by a `41 <TYPE>`
// result annotation, so the apply is bracketed on both sides by fixed markers
// and cannot be anything else. So the production is
//
//   33 86 41 74 <id>  40 <TYPE result>  ( <expr> 55 <TYPE> )*  4C
//
// with **no trailing field** and a possibly-empty argument list. `1948` was
// UNKNOWN in docs/IL_CAST_CONVERT.md §1.5 ("a nullary clock/counter read"); it
// is `__mftb`.
//
// `n_notintrinsic` and `n_lwsync` are the separating negatives for the *other*
// guess — that c1xx turns any zero-argument `extern "C"` declaration whose name
// it does not know into a `0x40`. It does not: the intrinsic set is a fixed
// internal table, and these two compile to ordinary `26 <tok> BD … 4C` calls
// with a REL24 relocation (`b <_AddressOfReturnAddress>`), so the id space
// cannot be enumerated by inventing names. That is why the port allow-lists ids
// pinned by capture instead of pattern-matching the token.

extern "C" {
void __debugbreak(void);
void *_ReturnAddress(void);
unsigned __int64 __mftb(void);
void *_AddressOfReturnAddress(void);
void __lwsync(void);
}

void n_break() { __debugbreak(); }
void *n_retaddr() { return _ReturnAddress(); }
unsigned __int64 n_mftb() { return __mftb(); }

void *n_notintrinsic() { return _AddressOfReturnAddress(); }
void n_lwsync() { __lwsync(); }
