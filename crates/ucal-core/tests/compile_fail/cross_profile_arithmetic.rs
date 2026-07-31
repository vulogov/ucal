//! Rule P / §21.3-11: cross-profile arithmetic must not compile.

mod support;

use support::UC1Prime;
use ucal_core::{Instant, UC1};

fn main() {
    let a: Instant<UC1> = Instant::zero();
    let b: Instant<UC1Prime> = Instant::zero();
    // Values from two profiles must not mix, even though both are 512-bit tick
    // counts: a tick count is meaningless without the datum it counts from.
    let _ = a.since(&b);
}
