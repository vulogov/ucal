//! `CalendarIdentity` must stay implementable from outside this crate.
//!
//! X3. `cargo semver-checks` does not catch a required method added to a public
//! trait — V1 Finding 6, confirmed against 0.50.0 with a real external crate —
//! so for each publicly implementable trait the mechanism is a downstream
//! implementor kept in the test suite. A trybuild fixture is compiled as its own
//! crate depending on `ucal-core`, which is that position exactly.
//!
//! This one is cheap to implement and therefore cheap to break: two required
//! methods and a defaulted third. `Kind` is a closed vocabulary, so an outsider
//! can name a variant; if it ever became `#[non_exhaustive]` this fixture would
//! fail, which is the right outcome and not an accident.

use ucal_core::qualified::CalendarIdentity;
use ucal_core::Kind;

pub struct Downstream;

impl CalendarIdentity for Downstream {
    fn id(&self) -> &str {
        "downstream"
    }
    fn kind(&self) -> Kind {
        Kind::Derived
    }
}

fn main() {
    assert_eq!(Downstream.id(), "downstream");
    // The defaulted method: a legacy calendar has no anchor revision, and an
    // implementor that does not care must not have to say so.
    assert_eq!(Downstream.revision(), None);
}
