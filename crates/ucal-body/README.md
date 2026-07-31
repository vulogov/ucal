# ucal-body

Cited body parameters, and calendars derived from them.

Part of [**ucal**](https://github.com/vulogov/ucal), an implementation of RFC
UCAL-1: absolute time as an unsigned integer count of Planck-time units since a
stipulated datum, with a positional base-5 calendar over it.

Rotation and orbital parameters for Earth, Mars and Titan, each recorded as
published with its citation and provenance, and calendars derived from them by
one generic path with no per-body special-casing.

Intercalation rules come out of continued-fraction expansion rather than
declaration: Earth reaches 31/128 and Mars 45/76 from the same default drift
bound, stated in each body's own local days and years. Earth is an instance,
not the template — this crate must not depend on `ucal-civil`, and a workspace
lint enforces it.

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
