# ucal-civil

The si bridge: tt pivot, tai, utc, leap seconds, gregorian and julian.

Part of [**ucal**](https://github.com/vulogov/ucal), an implementation of RFC
UCAL-1: absolute time as an unsigned integer count of Planck-time units since a
stipulated datum, with a positional base-5 calendar over it.

The one door between absolute time and Earth's civil labels. The SI second is
a *foreign unit* here, reachable only across a declared bridge and never assumed.

Covers the exact TT pivot, TAI, UTC with the IERS leap-second table, the
1961-1972 rubber-second era (exactly representable, because every rate
coefficient is divisible by 27), and the Gregorian and Julian calendars — both
of which this crate labels *legacy*.

## Properties

- **Unsigned.** Tick 0 is the datum; nothing precedes it. Subtraction that would
  go negative is an error, not a wrap.
- **No floats.** Anywhere. Enforced by a workspace lint that also reports every
  exemption it honours.
- **Uncertainty kept.** A value stated to a coarser tier *is* an interval, and
  the type carries it.

## Status

Early — the API is not yet stable.

## Licence

Mozilla Public License 2.0 — see [LICENSE](LICENSE).
