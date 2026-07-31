<p align="center">
  <img src="Documentation/logo/ucal-lockup.svg" alt="ucal" width="380">
</p>

# ucal — the Universe Calendar

An implementation of **RFC UCAL-1**: absolute time as an unsigned integer count
of Planck-time units since a stipulated datum, with a positional base-5 calendar
over it.

```
$ ucal now
ticks      8070205189128471254993117657693008777530466139316558837890625
human      UC1 0031·0687·2481·3000·1638·3018:0779·2671·2006·1837·2640·1833·1790·1250·0000·0000·0000·0000
ucid       0000000000050PM6K45MKCAVY5MPYAMHCJQ142JHE26A2ZAJ9FJ1
precision  T-12
```

Everything above is an exact integer. There is no floating-point value anywhere
in this workspace.

## Three properties

- **Time is unsigned.** Tick 0 is the datum and nothing precedes it. Subtraction
  that would go negative is an error, not a wrap.
- **No floats.** Every derived quantity is an exact rational or a certified
  interval. Rounding happens once, at display, under a mode the caller names.
- **Uncertainty is kept.** A value stated to a coarser tier *is* an interval,
  and the type carries it.

## Crates

| crate | contents |
|---|---|
| `ucal-core` | ticks, tiers, text and binary forms, UCIDs, exact rationals and intervals |
| `ucal-civil` | the SI bridge: TT, TAI, UTC, leap seconds, Gregorian and Julian |
| `ucal-body` | cited body parameters and calendars derived from them |
| `ucal-events` | interval-valued, cited milestones |
| `ucal-cosmo` | flat ΛCDM, `t ↔ z`, by certified integer quadrature |
| `ucal` | the command line |

## Build and run

```
cargo build --release
./target/release/ucal --help
./target/release/ucal datum
```

## Test

```
cargo test --workspace --release
cargo run -p xtask              # the constants harness, two independent routes
cargo run -p xtask -- lint      # workspace lints
```

## Status

Released: **0.1.1** on [crates.io](https://crates.io/crates/ucal), all six
crates. `main` carries the released line; `0.2.0` is where development happens.

The library and CLI are complete against RFC UCAL-1 and the suite is green. The
API is **not yet stable** — a `0.x` bump may break it.

Release notes are in
[`Documentation/Release_Notes`](Documentation/Release_Notes). Verification notes
and RFC errata are in [`spec/SPEC-DELTAS.md`](spec/SPEC-DELTAS.md). Fuller
documentation is still to be written.

## Licence

Mozilla Public License 2.0 — see [LICENSE](LICENSE).
