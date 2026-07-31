//! Rule Q.3: no conversion path from `SignedWindow` into a usable magnitude.

use ucal_core::{Delta, Profile, UC1};

fn main() {
    let claim = UC1::big_bang_claim();
    let _d: Delta = claim.into();
}
