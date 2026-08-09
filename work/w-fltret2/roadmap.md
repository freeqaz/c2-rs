
### 10.28.1 w-fltret2 — §10.28 REPLICATED to the byte by a second session that built the same rung and DECLINED to ship it (2026-08-09)

**Two sessions were dispatched on w-callprice's R2 and neither knew of the
other.** The peer landed first (`751351b6`, §10.28, board #2080–#2087). This is
the second run, re-landed as `w-fltret2`
([`rungs/2026-08-09-w-fltret2.md`](rungs/2026-08-09-w-fltret2.md), board
**#2088**–**#2096**); its own history is on branch `wt-w-fltret` and it ships **no
`crates/` and no `fixtures/` change** — `git diff master` is empty on both.

**The replication is exact and it is checked on the bytes, not on the counts.**
Both sessions re-derived the population, both wrote a member value-tail reader
from scratch, both ran it over 878 TUs. Compared per `(TU, emit_name)` against
one base scan:

| | this session's reader | §10.28's landed reader |
|---|--:|--:|
| newly-differing emitted functions | **444** | **444** |
| the two sets | **identical**, symmetric difference **0** | |
| port `.text` words, per function | **byte-identical on 444 of 444** | |

The implementations are genuinely different code — this one factors a shared
`eat_member_call_to_args` out of `eat_member_stmt_call`, carries the FP width on
`CallRet::Real` as an `Option<bool>` and admits the integer post-op
(`return s->get() + 3` → `addi r3,r3,3`); §10.28's takes a different cut and
leaves `CallValueFp` without an `add_k` field. Every published digit reproduces:
census **712,238** / emitted **39,644**, `fnbyte-exact` **36,228** unmoved,
`fnbyte-differs` **2,111 → 2,555**, family **423,905 / 35,576**, R2's population
**544 / 9**, `?SplitMs@Timer@@QAAMXZ` **434 emitted in 434 TUs**. A count
agreeing would prove nothing — 444 and 444 could be disjoint sets — which is why
the set and the bytes are the claim. Board **#2088**.

**And the two sessions reached opposite shipping decisions on those identical
numbers.** §10.28 shipped the class with its finding in its own merge headline.
This one **reverted**, on the ground that the emitted census gains **444 claims
the oracle grades wrong**, in the direction four lanes have spent the week
reversing: `w-empty` −1,373, `w-fix` −143, `w-splice` −723, `w-seed` −223 =
**−2,462**, against **+444** and the largest single-name block on record. **The
class stays shipped** and board **#2089** is the caution, not an override — but
the second reading is on the record as a decision rather than as a side effect.

**Four things this run adds that the first did not have.**

1. **The mechanism reduces to four lines and two words, and it has nothing to do
   with floating point.** For
   `struct T{ void s(){} int m(){return 7;} int both(){ s(); return m(); } }`,
   c2 emits **`38600007 4e800020`** — `li r3,7 ; blr`, a **two-word leaf** —
   where the port emits the same 13-word framed sequence it emits for
   `Timer::SplitMs`. Board **#2090**.
2. **The same defect is already live in `int-tail-call`, which shipped with the
   MVP.** In that same probe `?m3_call@@YAHPAUT@@@Z` is in class **at base** and
   the port emits `48000000` where c2 emits the inlined two words. So **the 2,111
   `fnbyte-differs` at base *is* this population**, and §10.28's rung is a new
   instance of a standing property of every call-bearing class the port has. That
   cuts both ways and is stated as cutting both ways: it is why #2089 is a
   caution rather than a decline, and it is why the hazard is bigger than one
   rung. Board **#2091**.
3. **Why the inherited price missed it, diagnosed.** w-callprice §7-R2
   hand-checked R2 on `float wcp_value_tail(O *o){ o->Poll(); return
   o->Level(); }` and read `bl Poll · bl Level` off c2's own `/FAsc` listing.
   **That listing is correct.** `Poll` and `Level` are declared and never defined
   in it, so c2 *cannot* inline them — and the construct it stands for is a
   header inline whose callees are header inlines. **#1148's "one line of C++
   nobody had written", at the level of linkage rather than source**; the listing
   seam narrates the compiler faithfully and cannot help, because what it
   narrates is the reproduction. Board **#2092**.
4. **The label lead, measured against the obj — which §10.28 did not do.** Seven
   cells in w-json's counterfactual form at two modes: the value tail charges
   **exactly** what w-mcall's statement sequence charges (+5 at `/O1`, +4 at
   `/Ox`), so `SeqTail::label_lead` is **0** and nothing was guessed; floating
   point costs **+1 at both modes**, which is `plan_labels`' existing per-TU
   `_fltused` slot; and the known-answer control — an FP leaf with a pooled
   constant at **+4 = 1 + 1 + 2** — reproduces `LABEL_COUNTER.md` §1's own table
   rather than quoting it. **Three must-fail mutations** price the fences at
   named offsets: `Mismatch @ 12` for the `_fltused` obligation, `@ 2587` for a
   label lead of 1, `@ 8` for admitting the `2C` result conversion. Boards
   **#2093**, **#2094**.

**The method finding both runs share, stated with a number.** Between them the
two PREREGs made **34 predictions and exactly one was about the byte judge** —
this run's P11, *"`fnbyte-exact` moves by the emitted delta and `fnbyte-differs`
by ZERO"*, registered at 0.70 with the words *"a non-zero `fnbyte-differs` delta
is a **failure**, not a finding"*. It missed by the whole population, and both
lanes' conversion predictions hit and were worthless. **A conversion count is not
a result unless it is crossed with the oracle: register the pair or neither.**
Board **#2095**.

**And the collision itself is recorded.** Both sessions took the lane name, the
rung filename, the board range `#2080`– and `work/w-fltret/`. The duplication was
not wasted — it produced the replication no single session can produce — but the
cheap prevention is a lane-name claim minted in the same commit as the PREREG,
which this board already requires to be the first commit. Board **#2096**.

[`rungs/2026-08-09-w-fltret2.md`](rungs/2026-08-09-w-fltret2.md).
