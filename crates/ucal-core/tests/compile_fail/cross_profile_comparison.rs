//! Rule P: cross-profile *comparison* must not compile either.

mod support;

use support::UC1Prime;
use ucal_core::{Instant, UC1};

fn main() {
    let a: Instant<UC1> = Instant::zero();
    let b: Instant<UC1Prime> = Instant::zero();
    let _ = a == b;
}
