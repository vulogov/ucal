// A second profile, so that cross-profile misuse can be exercised. It delegates
// every constant to UC1 — the point is only that it is a *distinct type*.
use ucal_core::profile::{Bridge, Citation, Frame, Profile, Provenance};
use ucal_core::{Ticks, UC1};
use ucal_core::value::SignedWindow;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UC1Prime;

impl Profile for UC1Prime {
    const TAG: &'static str = "UC1P";
    const FRAME: Frame = Frame::FlrwComoving;
    fn beat() -> Ticks { UC1::beat() }
    fn origin_offset() -> Ticks { UC1::origin_offset() }
    fn domain_max() -> Ticks { UC1::domain_max() }
    fn bridge() -> Bridge { UC1::bridge() }
    fn big_bang_claim() -> SignedWindow { UC1::big_bang_claim() }
    fn big_bang_claim_citation() -> Citation { UC1::big_bang_claim_citation() }
    fn datum_provenance() -> ucal_core::error::Result<&'static Provenance> {
        UC1::datum_provenance()
    }
}
