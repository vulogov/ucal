//! **`Profile` is an open extension point, and this is what says so.**
//!
//! `cargo semver-checks` does not catch a required method added to this trait.
//! Verified against 0.50.0 with a real external crate: adding
//! `fn frame_bridge_claim() -> SignedWindow;` as a *required* item stops an
//! outside implementor compiling with `not all trait items implemented`, and the
//! tool reports `no semver update required` for the same change.
//!
//! A trybuild fixture is compiled as its **own crate** depending on `ucal-core`,
//! which is exactly the position a downstream implementor is in. So this file
//! passing is the statement that an outsider can still implement `Profile`, and
//! this file failing is the statement that something was added they must now
//! write. `STABILITY.md` promises that within `1.x` nothing which compiled stops
//! compiling; for this trait, this is the only thing that checks it.
//!
//! Note what it does **not** construct. `Bridge` is `#[non_exhaustive]` with no
//! constructor, so no outsider can build one — an implementor must delegate
//! `bridge()` to a profile that already exists, as this does and as the
//! in-crate `UC1Prime` fixture does. That is a real constraint on what a second
//! profile can be, and it was nearly mistaken for a seal on the whole trait:
//! an outsider who *constructs* a `Bridge` cannot compile, and one who
//! *delegates* can.

use ucal_core::profile::{Bridge, Citation, Frame, Profile, Provenance};
use ucal_core::value::SignedWindow;
use ucal_core::{Ticks, UC1};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Downstream;

impl Profile for Downstream {
    const TAG: &'static str = "DOWN";
    const FRAME: Frame = Frame::FlrwComoving;
    fn beat() -> Ticks {
        UC1::beat()
    }
    fn origin_offset() -> Ticks {
        UC1::origin_offset()
    }
    fn domain_max() -> Ticks {
        UC1::domain_max()
    }
    fn bridge() -> Bridge {
        UC1::bridge()
    }
    fn big_bang_claim() -> SignedWindow {
        UC1::big_bang_claim()
    }
    fn big_bang_claim_citation() -> Citation {
        UC1::big_bang_claim_citation()
    }
    fn datum_provenance() -> ucal_core::error::Result<&'static Provenance> {
        UC1::datum_provenance()
    }
}

fn main() {
    // The point is that the impl above compiles. Touch it so it is not dead.
    assert_eq!(<Downstream as Profile>::TAG, "DOWN");
}
