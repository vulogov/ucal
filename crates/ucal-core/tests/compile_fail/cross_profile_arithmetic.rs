//! Rule P / §21.3-11: cross-profile arithmetic must not compile.
//!
//! This is the attempt a person makes second. The first, `a.since(&b)`, is
//! refused by the language itself — an `Instant<UC1Prime>` is simply not an
//! `Instant<UC1>` — so the next move is to convert `b` and then subtract. That
//! is the door this test holds shut: there is no `From` between profiles, and
//! `Instant::rebase` is the only crossing, and it reports its shift. Widen
//! `since` to accept any profile and the `.into()` target becomes ambiguous,
//! which is a different sentence and still a failure of this test.
//!
//! Until 1.13.0 this file held the first attempt. Its refusal is E0308, and
//! the compiler's expected/found note for that error names `Instant` with or
//! without its crate depending on the compiler's version — `std::time::Instant`
//! shares the short name, and 1.99 began trimming it regardless. The refusal
//! was the same on every compiler; the sentence was not, and trybuild pins the
//! sentence. This form's E0277 reads the same on 1.95 and on 1.99.

mod support;

use support::UC1Prime;
use ucal_core::{Instant, UC1};

fn main() {
    let a: Instant<UC1> = Instant::zero();
    let b: Instant<UC1Prime> = Instant::zero();
    // Values from two profiles must not mix, even though both are 512-bit tick
    // counts: a tick count is meaningless without the datum it counts from.
    let _ = a.since(&b.into());
}
