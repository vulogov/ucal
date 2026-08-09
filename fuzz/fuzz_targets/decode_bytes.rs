//! Rule B's wire format, against arbitrary bytes.
//!
//! The canonical encoding is exactly 64 bytes and the decoder must reject
//! anything else, and reject a 64-byte value at or above the domain ceiling
//! rather than wrapping into it — the property that makes the two backends
//! interchangeable.
#![no_main]
use libfuzzer_sys::fuzz_target;
use ucal_core::backend::TickInt;
use ucal_core::Ticks;

fuzz_target!(|data: &[u8]| {
    if data.len() != 64 {
        return;
    }
    let mut buf = [0u8; 64];
    buf.copy_from_slice(data);
    if let Some(v) = <Ticks as TickInt>::from_canonical_bytes(&buf) {
        // Whatever decodes must re-encode to the same bytes. A decoder that
        // accepted a value it could not reproduce would break Rule B's promise
        // that the encoding is canonical.
        assert_eq!(
            v.to_canonical_bytes(),
            buf,
            "a decoded value did not re-encode to the bytes it came from"
        );
    }
});
