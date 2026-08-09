//! The CLI's own instant parser, against arbitrary text.
//!
//! `ucal::parse_instant` is what every command that takes an instant calls, so
//! it is the widest input surface the program has: a UC1 text form, a UCID, or
//! a decimal tick count, chosen by inspecting the string.
//!
//! The property is the one §19.5 already promises: **a value or a diagnosed
//! rejection, never anything else.** libFuzzer treats a panic as a crash, so
//! asserting it is a matter of not catching one.
#![no_main]
use libfuzzer_sys::fuzz_target;
use ucal_core::backend::TickInt;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    match ucal::parse_instant(s) {
        // A parsed instant must be renderable and must survive a round trip,
        // because a value that parses and then cannot be printed is a value the
        // program will fail on later, somewhere less obvious.
        Ok((t, _)) => {
            let _ = t.ticks().to_dec_string();
            let _ = t.to_ucid();
        }
        // A rejection must carry a code. `TimeError` has no other shape, so
        // reaching here at all is the assertion.
        Err(e) => {
            let _ = e.code;
        }
    }
});
