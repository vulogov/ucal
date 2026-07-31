# ucal

The command line.

Part of [**ucal**](https://github.com/vulogov/ucal), an implementation of RFC
UCAL-1: absolute time as an unsigned integer count of Planck-time units since a
stipulated datum, with a positional base-5 calendar over it.

The `ucal` binary, and the same commands available as a library so they can be
called directly rather than by spawning a process.

```
ucal now                      the current instant, offline
ucal datum                    what tick 0 is, and what is not being claimed
ucal explain <instant>        every form, every tier, the SI bridge
ucal from-civil / to-civil    civil labels in and out
ucal cal / show               derived and legacy calendars
ucal events / timeline        cited milestones against the tier ladder
ucal cosmo age / z / model    flat LambdaCDM enclosures
ucal doctor                   profile, backend, ceiling, provenance
```

`--json` gives stable, versioned output for all of them.

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
