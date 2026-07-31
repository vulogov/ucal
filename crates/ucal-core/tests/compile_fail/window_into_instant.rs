//! Rule U: "Windows MUST NOT be silently collapsed; `Window::midpoint(Rounding)`
//! must be called explicitly."
//!
//! A `From<Window> for Instant` would make the collapse implicit and pick a
//! rounding mode on the caller's behalf, which is the whole thing the rule
//! forbids.

use ucal_core::{Instant, Window, UC1};

fn main() {
    let w: Window<UC1> = Window::exact(Instant::zero());
    let _collapsed: Instant<UC1> = w.into();
}
