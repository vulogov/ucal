//! Rule K.6 / §8.6: "`LegacyCalendar` and `BodyCalendar` are distinct traits with
//! no blanket conversion; a function requiring a derived calendar MUST NOT accept
//! a legacy one."
//!
//! `require_derived` takes a `&dyn CalendarIdentity` and checks at runtime, which
//! is the fallback for erased types. The primary defence is this: a function
//! generic over a derivation must not accept `Gregorian` at all.

use ucal_civil::legacy::Gregorian;
use ucal_core::qualified::Kind;

/// Stands in for the `BodyCalendar` bound that arrives with UC-P11. The point is
/// the marker trait, which no legacy calendar implements.
trait DerivedCalendarMarker {
    const KIND: Kind = Kind::Derived;
}

fn needs_a_derivation<C: DerivedCalendarMarker>(_c: C) {}

fn main() {
    needs_a_derivation(Gregorian);
}
