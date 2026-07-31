# ucal-core

Ticks, tiers, text and binary forms, UCIDs, exact rationals and certified intervals.

Part of [**ucal**](https://github.com/vulogov/ucal), an implementation of RFC
UCAL-1: absolute time as an unsigned integer count of Planck-time units since a
stipulated datum, with a positional base-5 calendar over it.

The foundation crate: the tick type, the base-5 tier grid, the two text forms
and the fixed 64-byte binary form, UCIDs, exact rational and interval arithmetic,
and profile `UC-1`.

Two interchangeable integer backends sit behind one trait — `bnum::U512` by
default (stack-allocated, `Copy`, const-constructible) and `num_bigint::BigUint`
under `--features bigint`. Both accept and reject exactly the same values.

Builds `no_std` with **no allocator at all**; what goes with the allocator is
radix formatting — the text codecs and the continued-fraction expansions. The
tick type, all of the checked arithmetic, `Ratio`, `RatInterval`, the tier grid
and the binary codec remain.

```rust
use ucal_core::{Instant, UC1, Profile};

let t = UC1::bridge_epoch()?;
println!("{}", t.ticks());
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
