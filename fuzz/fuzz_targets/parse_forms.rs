//! The §6 text-form codec, against arbitrary text, in both forms.
//!
//! `codec::parse` is stricter than the CLI's dispatcher and has more structure
//! to get wrong: a profile tag, group separators, a sub-beat introducer, group
//! values bounded at 3124, and a tier count bounded by the grid.
//!
//! Where a parse succeeds this re-renders and re-parses, so the fuzzer is
//! looking for a string that parses to a value whose rendering parses to a
//! *different* value — a round-trip break, which no crash would reveal.
#![no_main]
use libfuzzer_sys::fuzz_target;
use ucal_core::codec::{self, Fmt};
use ucal_core::UC1;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    for fmt in [Fmt::human(), Fmt::digit5()] {
        if let Ok((t, _)) = codec::parse::<UC1>(s, &fmt) {
            if let Ok(rendered) = codec::render(&t, &fmt) {
                if let Ok((again, _)) = codec::parse::<UC1>(&rendered, &fmt) {
                    assert_eq!(
                        again.ticks(),
                        t.ticks(),
                        "a form parsed to one value and its rendering to another"
                    );
                }
            }
        }
    }
});
