# Conformance

An independent implementation of [UCAL-1.1](UCAL-1.1.md) can be checked against
this one without reading this one, using `fixtures/vectors.json`.

## What the vectors cover

| group | contents |
|---|---|
| `constants` | every Appendix A constant, as decimal strings |
| `provenance` | the §2.2 chain: `AGE_s`, `AGE_ticks`, whole beats, residual |
| `alignment` | the §2.4 invariants — `v5(SECOND)`, `v5(NANOSECOND)`, `v5(ORIGIN_OFFSET)` |
| `tiers` | the `5^(5k)` ladder with exact tick counts |
| `fixtures` | Appendix C tick fixtures, UCIDs, and text forms at the precision printed |
| `continued_fractions` | Appendix I intercalation expansions and convergents |
| `mars_satellites` | Appendix I.4 synodic periods |
| `deltas` | which corrections the vectors embody |

Values are decimal strings, not numbers. JSON numbers are IEEE 754 doubles in
most parsers, and a 203-bit integer does not survive one — Rule E applies to the
interchange format as much as to the library.

The `rfc` and `profile` fields say which specification and profile the vectors
are for. A vector file that does not name them is not usable as a conformance
artefact, because "these numbers" is not a claim until you know what they are
numbers *for*.

## Checking a checkout

```
cargo run -p xtask -- verify-vectors
```

This re-derives every vector along **two independent integer routes** —
`bnum::U512` and `num_bigint::BigUint` — and checks the result against
`fixtures/SHA256SUMS`.

It answers *"does this checkout produce the vectors it claims to"*. It does not
answer *"did anyone vouch for this digest"*. Those are different questions and
the tool reports them separately: an unsigned run prints `UNSIGNED` rather than
a bare `ok`, because a digest proves a file was not corrupted and only a
signature proves who stood behind it. Reporting the first as though it were the
second is exactly the overclaim Rule Q exists to prevent.

## Checking a different implementation

Load `vectors.json`, compute each quantity your own way, and compare the decimal
strings. No dependency on this codebase is required or wanted — an oracle you
have to trust to check a thing is not an oracle.

The vectors are deliberately *derivations*, not just outputs. `provenance`
carries every intermediate of the §2.2 chain, so a mismatch localises to the
step that diverged instead of reporting only that the final constant differs.

## Signing a release

§20 asks for a **signed** vector file. `fixtures/SHA256SUMS` is the artefact to
sign; signing needs a key, which is a release-process step and deliberately not
something the harness does.

**Current status: signed**, from 0.5.0.

```
key ID           D0E4E5A9439E54CC
public key       RWTMVJ5DqeXk0HgeN+BIdnQaamRTdzkjITkdprOPLVsGWP8R/2HYIj0r
signed digest    1f99cf6280f8b5dce88c6558e7c73769a40f93f7fcd3d3091f27c2658389f1f0
```

Verify:

```
minisign -Vm fixtures/SHA256SUMS \
  -P RWTMVJ5DqeXk0HgeN+BIdnQaamRTdzkjITkdprOPLVsGWP8R/2HYIj0r
```

The trusted comment carries the digest and is signed along with the file, so a
signature cannot later be presented as vouching for a different one.

### Procedure

[minisign](https://jedisct1.github.io/minisign/) — small, one file in, one file
out, no web of trust to reason about.

```
# once, per maintainer
minisign -G                                  # writes minisign.key / minisign.pub

# at each release, after `cargo run -p xtask` regenerates the vectors
cargo run -p xtask -- verify-vectors         # confirm the digest first
minisign -Sm fixtures/SHA256SUMS -t "ucal vX.Y.Z conformance vectors"

# commit the signature and publish the public key
git add fixtures/SHA256SUMS.minisig
```

`verify-vectors` picks up `fixtures/SHA256SUMS.minisig` automatically once it
exists and prints the verification command.

A verifier needs the public key from somewhere other than this repository — a
signature checked against a key stored beside it proves only that the two files
were made together. Publish it in the release announcement, or anywhere with a
different trust path.

### What signing does and does not establish

It establishes that the holder of one key vouched for one digest at one time.

It does **not** establish that the vectors are correct. That comes from the two
independent derivation routes agreeing, from the 376 tests, and from the fact
that every Appendix A constant reproduces bit-exactly — none of which a
signature can substitute for.

Sign because it lets someone who does not trust the transport check that they
have what was published. Not because it makes the numbers truer.

### Key custody, stated so that nobody infers more than exists

A signature invites an assumption about the infrastructure behind it. Here there
is very little, and that is the honest description rather than an apology:

- **One key, held by one maintainer**, on one machine, with an offline backup.
- **No rotation procedure.** There is no schedule and no successor key.
- **No revocation path.** If the key were compromised there is no mechanism to
  announce it beyond amending this file and saying so in a release.
- **No timestamping authority.** The signature says a key vouched for a digest;
  nothing independent attests to *when*.

What follows from that is narrow and worth stating plainly. A verifier who
checks this signature learns that the holder of `D0E4E5A9439E54CC` vouched for
this digest. They do not learn that the key is still under its holder's control,
and they cannot learn it from anything in this repository.

The offline backup exists so that losing the laptop does not silently end the
signing line — which would otherwise be discovered at the next release, by which
time re-establishing trust in a new key costs more than keeping a copy did.

### Where the key is published, and what that does not establish

```
RWTMVJ5DqeXk0HgeN+BIdnQaamRTdzkjITkdprOPLVsGWP8R/2HYIj0r
```

It appears in `fixtures/ucal.pub`, in this file, in the repository's README, in
[`Documentation/CONTACT.md`](../Documentation/CONTACT.md), and in the READMEs of
the `ucal` and `ucal-core` crates — which means it is also rendered on
**crates.io** and on **docs.rs**, from a package tarball that cannot be changed
once a version is published.

**That is not five independent trust paths.** One person placed all five, and a
reader who does not already know that person is no better off against a forgery
than with one copy. Multiplying copies under one authority multiplies nothing.

What it does establish is narrower and worth having. Two of the copies are
beyond the author's own reach once released: the crates.io tarball for a
published version cannot be edited, only yanked. So a repository whose key was
**changed** — by a compromise, by a rewrite of history, by coercion — can be
contradicted by an artefact nobody involved can quietly edit. That is
tamper-evidence after the fact, not authentication on first contact.

**First contact remains trust on first use**, and no number of copies fixes it.
Fixing it needs an authority that is not this project: a DNS record under a
domain with its own registrar, a third party who signs this key with theirs, or
publication somewhere with an editorial record. None of those exists yet, and
this section says so rather than letting five copies look like five reasons.

*Enforced by* the `signing-key` check in `cargo run -p xtask -- check-docs`,
which fails if any declared publication has dropped the key, and if any document
in the tree carries a key that is not this one — a truncated paste or a
transposed character being the realistic failure rather than a forgery. A reader
who checks a mistyped key learns nothing and believes they learned something.
