# The fuzzer

Three targets over the widest input surfaces the program has, run with
[`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz).

```
cargo +nightly fuzz run parse_instant seeds/parse_instant
cargo +nightly fuzz run parse_forms   seeds/parse_forms
cargo +nightly fuzz run decode_bytes  seeds/decode_bytes
```

| target | surface | property |
|---|---|---|
| `parse_instant` | `ucal::parse_instant` — what every command taking an instant calls | a value or a diagnosed rejection, never a panic |
| `parse_forms` | the §6 codec, both text forms | as above, **and** that a form and its re-rendering parse to the same value |
| `decode_bytes` | Rule B's 64-byte wire format | a decoded value re-encodes to the bytes it came from |

## Why it is not in CI

It needs a nightly toolchain and libFuzzer, and it is unbounded by nature: a
fuzz job either has a time budget, in which case it proves less each time the
code grows, or it does not, in which case it is not a CI job. The MSRV job pins
1.87 and this would drag nightly into the same build.

`fuzz/` is therefore outside the workspace — `exclude = ["fuzz"]` in the root
manifest — so `cargo build --workspace` never sees it.

What *is* in CI is `crates/ucal/tests/hostile_input.rs`, forty hand-chosen
invocations of the real binary, and `crates/ucal-core/tests/properties.rs`,
which runs the same invariants over ten thousand deterministically generated
inputs on both backends. Those catch the class; this finds the instance.

## Seeds, corpus, artefacts — three different things

**`seeds/` is committed.** Inputs the program already knows: a tick count, a
human form, a UCID, a digit form, and the two extremes of the byte encoding. A
fuzzer starting from nothing spends its first minutes rediscovering that a UC1
form begins with `UC1`.

**`corpus/` is not.** A run grows it to some fifteen hundred files and six
megabytes, and it is a cache — minutes of fuzzing rebuild it. Committing it
would put six megabytes into every clone of this repository forever for no gain
a reader could use.

**`artifacts/` is committed, and is the opposite case.** A crash artefact is an
input that broke something, and an input that broke something once must be
checked on every run after. It is currently empty, which is the honest state and
not an omission.

## What has been run, and what it found

Recorded rather than summarised as "fuzzed", because a clean fuzz run is weak
evidence presented as strong more often than almost anything else in software.

| target | executions | found |
|---|---|---|
| `parse_instant` | 37.6M in 121 s, then 214.2M in 601 s | nothing |
| `parse_forms` | 12.8M in 121 s | nothing |
| `decode_bytes` | 162.6M in 121 s | nothing |

Run on an Apple M5 Pro, seeded from `seeds/`, on the 1.3.0 tree.

**Nothing found is not the same as nothing there.** These ran for minutes on one
machine against three entry points; they say the parsers do not crash on
adversarial bytes, and say nothing about whether the values they produce are the
right ones. That is what the property tests and the conformance vectors are for.
