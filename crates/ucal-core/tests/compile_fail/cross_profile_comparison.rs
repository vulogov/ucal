//! Rule P: cross-profile *comparison* must not compile either.
//!
//! `a == b` is refused by the language — the two are different types — so the
//! second attempt is to convert and then compare. There is no conversion; the
//! only crossing is `Instant::rebase`, which reports the shift it applies. Add
//! a cross-profile `PartialEq` and the `.into()` target becomes ambiguous,
//! which is a different sentence and still a failure of this test.
//!
//! The first attempt is not kept here for the reason given in
//! `cross_profile_arithmetic.rs`: its E0308 note is worded differently by
//! different compilers, and trybuild pins the wording.

mod support;

use support::UC1Prime;
use ucal_core::{Instant, UC1};

fn main() {
    let a: Instant<UC1> = Instant::zero();
    let b: Instant<UC1Prime> = Instant::zero();
    let _ = a == b.into();
}
