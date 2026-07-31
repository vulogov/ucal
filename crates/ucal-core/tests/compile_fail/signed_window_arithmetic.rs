//! Rule Q.3: `SignedWindow` has no arithmetic operators at all.

use ucal_core::{Profile, UC1};

fn main() {
    let a = UC1::big_bang_claim();
    let b = UC1::big_bang_claim();
    let _ = a + b;
}
