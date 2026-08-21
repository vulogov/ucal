//! Rule F / D-A25: `FRAME_BRIDGE_CLAIM` must not reach arithmetic either.
//!
//! It returns the same `SignedWindow` as `big_bang_claim`, so it inherits that
//! type's inertness — and inheritance is the kind of guarantee that quietly
//! stops holding when a return type is changed for an unrelated reason. This
//! fixture is what notices.
//!
//! If it ever compiles, a bound on the *interpretation* of a tick count could be
//! added to a tick count, which is the failure `BIG_BANG_CLAIM`'s own fixtures
//! exist to prevent, in the one place where the number is small enough to look
//! harmless.

use ucal_core::{Instant, Profile, UC1};

fn main() {
    let t: Instant<UC1> = Instant::zero();
    let claim = UC1::frame_bridge_claim();
    let _ = t.checked_add(&claim);
}
