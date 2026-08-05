//! Rule F: the reference frame is stated, not assumed.
//!
//! `Profile::FRAME` is a required associated constant with no default, so a
//! profile that declines to declare a frame does not exist — it fails to
//! compile. Rule F's entry in `spec/RULES.md` called itself convention-enforced
//! until 0.5.0; this is the part of it that was never convention.
//!
//! What remains convention is narrower and irreducible: that the frame declared
//! is the frame the numbers were computed in. No type can check that, and Rule F
//! does not ask one to — it requires the frame to be *stated*, not to be true.

use ucal_core::profile::{Bridge, Citation, Profile, Provenance};
use ucal_core::value::SignedWindow;
use ucal_core::{Ticks, UC1};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FramelessProfile;

impl Profile for FramelessProfile {
    const TAG: &'static str = "NOFRAME";
    // No `const FRAME`. Everything else is delegated, so this is the only thing
    // wrong with it.
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

fn main() {}
