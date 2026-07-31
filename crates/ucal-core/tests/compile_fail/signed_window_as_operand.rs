//! Rule Q.3 / §21.3-3: `BIG_BANG_CLAIM` must not reach arithmetic.
//!
//! `big_bang_claim()` returns a `SignedWindow`, which has no conversion into
//! `Delta`. If this ever compiles, the datum's ±0.020 Gyr uncertainty could leak
//! into timestamps — failure mode F11 — and the exactness of the arithmetic would
//! be silently traded for the accuracy of a measurement.

use ucal_core::{Instant, Profile, UC1};

fn main() {
    let t: Instant<UC1> = Instant::zero();
    let claim = UC1::big_bang_claim();
    // A SignedWindow is not a Delta and must never become one.
    let _ = t.checked_add(&claim);
}
