// GRID-N n04 — A BODY REFUSED FOR A DIFFERENT REASON, and the one that matters:
// `body-0x67`, the virtual-dispatch production. On the workload it holds **5,154**
// no-effect chain stops, and w-memset #1056 records why that is a hazard and not
// headroom: it is exactly the refusal that keeps E safe from an INDIRECT call
// site (INLINE_PREDICATE.md §1.3, board #921 — `f10_virtual_ptr`), and E does not
// model the site.
//
// Registered: the new reader admits NOTHING here. `fnbyte-nothing-rows` must not
// count a `body-0x67` row, `?use` stays an honest differ, and if this cell ever
// converts the lane STOPS (PREREG §3 clause 6) — that is #232's shape.
struct B { virtual void f(); };

inline void vcall(B* p) { p->f(); }

void use(B* p) { vcall(p); }
