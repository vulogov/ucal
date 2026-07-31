//! Rule T: a value carrying a precision must not be usable where a tick-precise
//! instant is required, or the precision would be silently discarded (F2).

use ucal_core::{Delta, Instant, Precision, Stated, Tier, UC1};

fn main() {
    let s: Stated<UC1> = Stated::new(Instant::zero(), Precision::Tier(Tier::BEAT));
    // A statement is not an instant, and must not be added to a duration.
    let _ = s.checked_add(&Delta::one_tick());
}
