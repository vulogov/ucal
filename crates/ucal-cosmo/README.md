# ucal-cosmo

Flat lambdacdm, t <-> z, by certified integer interval quadrature.

Part of [**ucal**](https://github.com/vulogov/ucal), an implementation of RFC
UCAL-1: absolute time as an unsigned integer count of Planck-time units since a
stipulated datum, with a positional base-5 calendar over it.

Age from redshift and back, under a declared flat LambdaCDM model, with integer
arithmetic only. No float appears in any signature, field, constant or
intermediate, and no transcendental function is evaluated anywhere.

Every result is a certified enclosure carrying **two widths that are never
merged**: the quadrature's own error, and the propagated consequence of Planck
2018's published uncertainties. On these numbers they differ by two orders of
magnitude, so a single merged tolerance would hide the one that matters.

```rust
use ucal_cosmo::{LambdaCdm, DEFAULT_DEPTH, DEFAULT_SCALE};
use ucal_core::num::Ratio;

let m = LambdaCdm::planck2018();
let z = Ratio::from_decimal_str("1100")?;
let age = m.t_of_z(&z, DEFAULT_DEPTH, DEFAULT_SCALE)?;
// age.value is an enclosure; age.arithmetic_width and
// age.parameter_width are reported separately.
```

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
