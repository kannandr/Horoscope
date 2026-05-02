# Rust Parity Strategy

`panchang-core` is the only Panchang calculation engine in the platform. The
Python reference implementation has been retired; parity is now maintained
through the Rust **golden** crate (`rust/crates/panchang-golden`).

Golden coverage should include:

- Common temple-calendar locations: Livermore, Bengaluru, Chennai.
- UTC, Asia/Kolkata, America/Los_Angeles, and a DST transition day.
- New moon, full moon, nakshatra wrap, sunrise/sunset edge cases.
- Both `meeus` and `surya_mean`; `lahiri`, `lahiri_alt_stub`, and `raman`.

Tolerance targets:

- Julian day transition boundaries: ≤ 10 seconds for current-era dates.
- Longitudes: ≤ 0.02 degrees for Sun and Moon against the locked golden fixtures.
- Label parity: exact for tithi, nakshatra, yoga, karana, rashi, vaara.

Cross-language fixtures (e.g. comparing against a third-party almanac PDF for a
specific city/day) live next to the golden crate so they ship with the engine
and not with any UI surface.
