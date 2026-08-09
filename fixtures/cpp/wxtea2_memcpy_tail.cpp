// W-XTEA2 — the body that is NOTHING BUT a `memcpy` into the receiver, which c2
// lowers as a TAIL BRANCH rather than a framed `bl`.
// `?SetKey@XTEABlockEncrypter@@QAAXPBE@Z` is the workload instance
// (`src/system/utl/EncryptXTEA.cpp`, 12 bytes, one of that TU's four blocked
// bodies).
//
// Compile at the workload's own profile (`/O1 /Oi /EHsc /GR`). At `/Ox` this
// file is deliberately 0/4 in class: the mode gate lives in the PARSER
// (board #1638) and in the emitter, and `scripts/mode_lane.sh` compiles every
// fixture at both.
//
// What this fixture grades that no earlier one can:
//
//  * **A MINTED external on a LEAF.** `w-ifn` already grades a minted `memcpy`
//    — but only on a FRAMED user, where `CEILING.md` §11's NC-1 item 7 records
//    its symbol landing after that function's `$T`. These users have no `$T` at
//    all, and both of this lane's obj readings put `memcpy` in the CALLEE
//    REGION instead (`work/w-xtea2/ref/xtea.dump` `[16] ?SetKey · [17] memcpy ·
//    [18] .text`). So this is the cell that separates the two placements: a
//    writer that used `w-ifn`'s answer here would resolve every relocation and
//    move two symbol indices.
//  * **THREE users of one minted name, and only one symbol.** `?wx2_set_key`,
//    `?wx2_set_nonce` and `wx2_free` all branch to `memcpy`; the reference obj
//    carries exactly one `memcpy` symbol and three relocations against it.
//  * **The conditional first word.** `?wx2_set_key` copies into a member at
//    offset 16 and emits `addi r3,r3,16`; `?wx2_set_nonce` copies into the
//    member at offset 0 and emits **no `addi` at all** — 8 bytes against 12.
//    Both spellings in one obj, so a port that emitted `addi r3,r3,0` is one
//    word long in a section whose relocation still resolves.
//  * **The label channel, downstream.** The TU's first `memcpy`-minting
//    function takes one extra counter slot before any later function's `$M`
//    triple, and `?wx2_after` at the end is what makes that slot observable.
//    `LABEL_COUNTER.md` §7.6 step 5 and board #2305: a charge on the LAST
//    function of a TU moves nothing, so a cell that puts the subject last
//    cannot fail.

extern "C" void *memcpy(void *, const void *, unsigned long);

// The layout is `EncryptXTEA.h`'s, and the two offsets are what make the two
// spellings different bodies: `mNonce` at 0 and `mKey` at 16.
struct wx2_block {
    unsigned long long mNonce[2]; // 0x00
    unsigned int mKey[4];         // 0x10
    void set_key(const unsigned char *uc);
    void set_nonce(const unsigned char *uc);
};

// THE SUBJECT — `?SetKey@XTEABlockEncrypter`'s shape, word for word.
//   addi r3,r3,16 ; li r5,16 ; b memcpy
void wx2_block::set_key(const unsigned char *uc) { memcpy(mKey, uc, 0x10); }

// The same class at destination offset 0: NO `addi`.
//   li r5,16 ; b memcpy
void wx2_block::set_nonce(const unsigned char *uc) { memcpy(mNonce, uc, 0x10); }

// …and with no `this` at all, so the two argument registers are two formals
// rather than the receiver and one. A different length, so the `li`'s immediate
// is graded as a field and not as a constant.
//
// **Not `extern "C"`, deliberately.** An undecorated definition stops
// `gl_defined_names_framed` at `INLINE_NAME_MAX` and the whole TU reads
// `gl-stop-name-not-mangled` — measured on the first draft of this file, which
// graded `vocab-gap` with the gate never reaching a body. That is `w-front5`
// #2621's mechanism arriving in a fixture, and the repair that would bind it is
// a measured net regression (#2622), so the cell is spelled the way the
// workload's own TU is.
void wx2_free(unsigned char *d, const unsigned char *s) {
    memcpy(d, s, 0x20);
}

// The downstream fence: an ORDINARY framed call, LAST. It is what makes the
// once-per-TU `memcpy` label slot land on an observable `$M`/`$M`/`$T` triple,
// and it grades both external placements in one obj — `?wx2_gz` is IL-named and
// sits between this function's two `$M`s, while `memcpy` is minted and sits in
// the first subject's callee region.
int wx2_gz(int);
int wx2_after(int a) { return wx2_gz(a) + 7; }
