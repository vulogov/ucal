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

## Verifying what you downloaded

`fixtures/vectors.json` carries every declared constant of profile UC-1 and its
derivation. Its digest is signed:

```
minisign -Vm fixtures/SHA256SUMS -P RWTMVJ5DqeXk0HgeN+BIdnQaamRTdzkjITkdprOPLVsGWP8R/2HYIj0r
```

The public key is `RWTMVJ5DqeXk0HgeN+BIdnQaamRTdzkjITkdprOPLVsGWP8R/2HYIj0r`,
minisign key ID `D0E4E5A9439E54CC`.

That key is printed here, in this crate's README on crates.io, and in
`spec/CONFORMANCE.md`. Those are copies placed by one person, not independent
authorities — but a crates.io version cannot be altered once published, so a key
changed *in this repository* can be contradicted by one nobody can edit.
[`spec/CONFORMANCE.md`](https://github.com/vulogov/ucal/blob/main/spec/CONFORMANCE.md)
states the rest of what a signature here does and does not establish.

## Licence

Mozilla Public License 2.0 — see [LICENSE](LICENSE).
