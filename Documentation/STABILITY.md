# What 1.0 will promise

Written before 1.0 rather than after, so that the promise is a decision rather
than a description of whatever happened to be true on the day.

**Status: in force from 1.0.0.** These are promises now, not intentions. Five of
the nine `0.x` releases broke something — 0.1.1, 0.3.0, 0.4.0, 0.6.0 and the
release that became 1.0.0. None of the remaining `1.x` line may.

**Two of the exit criteria were not met, and 1.0 shipped anyway.** The
[road to 1.0](Proposals/ROAD-TO-1.0.md) made contact the gate: an external
implementation reproducing the conformance vectors, and a surface that had
survived meeting someone. Neither happened in three cycles of asking. The
criteria are left standing rather than rewritten, and what follows is a promise
about a design **no one outside this repository has ever used**. The section at
the end of this file, *What would make 1.0 dishonest*, still opens with
"shipping before contact", and it is left there on purpose.

---

## The floor: semver

Within `1.x`, no release removes a public item, changes a public signature, or
narrows what compiles. That is the ordinary contract and it is the least
interesting part of this file.

## Above the floor

Semver has no vocabulary for most of what this project actually claims. Six
promises, each with the mechanism that keeps it — because a promise with no
mechanism is a convention, and 0.5.0 was spent removing the last two of those.

### 1. `BIG_BANG_CLAIM` never acquires an arithmetic operator

The published identification of the origin is metadata. It is fully readable,
fully cited, carries an exact magnitude, and **cannot enter a computation**
(Rule Q.3). No `1.x` release adds an operator, a conversion, or an accessor that
would let it.

*Enforced by* three compile-fail cases —
`signed_window_as_operand`, `signed_window_into_delta`,
`signed_window_arithmetic` — which must fail to build. If one starts compiling,
the promise has been broken by the type system rather than by a person.

### 2. A rounding never becomes undeclared again

Values round when displayed, never when constructed, and always under a mode the
caller can name (Rule R). `--decimals` and `--round` reach every rendered
rational, and every rendering that cannot take them says why.

*Enforced by* the `rounding-is-declared` lint over the library crates, a
property test that no call site in `crates/ucal` formats a decimal without going
through `Value::quantity`, and the certification map that lists every rounding
in the output.

*History.* This was a convention until 0.4.0, and every alignment defect a
reader found in two cycles landed on it. That is why it is on this list.

### 3. No Earth unit appears outside an Earth context unless asked for

A Julian year is 365.25 of Earth's rotations; an SI second is an Earth unit.
Neither is used to describe something that is not of Earth without `--bridge`.
`to-civil` and `from-civil` are exempt because they *are* Earth calendar
commands, and `datum`'s provenance chain is exempt because §19.2 requires it and
it records where Earth entered.

*Enforced by* `no_earth_units.rs`, and structurally by `Value::Bridge`, which
makes "this is a foreign unit" a property of the value rather than something a
call site must remember.

### 4. `ucal-json/1` fields never change name, shape or meaning

New fields may appear — a consumer must ignore what it does not recognise. No
existing field is removed, renamed, or changed from a string to an object.

*Enforced by* `fixtures/json-surface.txt`, a committed baseline of every field
path and the JSON kind it serialises to, checked by `json_surface.rs`. A removed
path or a changed kind fails; an added path does not.

**Its limit, stated because a green run would otherwise imply more.** A field
that keeps its name and its kind and begins reporting something *else* passes
this check, and is exactly the breakage this promise is about. Nothing
mechanical reaches meaning. What the baseline gives is that the *shape* cannot
drift unnoticed, which is the part that can be automated.

### 5. The command line never aborts

A failure leaves through §19.5's table: an Appendix E code and a sentence on
stderr, nothing on stdout, and an exit status in `0–9`. No release makes a
diagnosable condition arrive as a panic, and none prints a Rust backtrace or
suggests `RUST_BACKTRACE` — neither is addressed to the person running the
program.

A defect is not the same as a failure and does not pretend to be: a panic that
reaches the top exits **70** (`EX_SOFTWARE`), outside §19.5's range on purpose,
and says that the input was not at fault.

*Enforced by* the `no-panic-in-cli` lint over `crates/ucal/src`, a corpus of
malformed invocations in `hostile_input.rs` that asserts all of the above
against the real binary, and `panic_handler.rs`, which induces a panic and
checks what comes out.

**Its limit, and it is a large one: this promise is about the binary, not the
libraries.** A caller who links `ucal-core`, `ucal-civil`, `ucal-events` or
`ucal-cosmo` gets neither the lint nor the handler. Those crates carry twenty-two `expect` calls on invariants they have just established — `self >=
other` immediately after comparing — and a caller who trips one gets a panic in
their process, with no diagnostic and no exit code, because there is no process
of ours to exit.

That is stated here rather than left for a reader to infer from "the command
line never aborts", because the natural reading of a stability document that
promises no aborts is that the library does not abort either, and it does not
promise that.

Why the libraries are not held to the same rule: rewriting a provably
unreachable branch into a `Result` gives every caller an error case that no
input can produce, on a path where the alternative is not a wrong answer but no
answer. A caller who wants the guarantee anyway can have it today by catching
unwinds at their own boundary, which is where the decision belongs.

**What a `1.x` release will not do** is make a *reachable* condition arrive as a
panic in any crate. An `expect` guarding an established invariant is allowed to
stay; an `expect` on something a caller's input can reach is a defect, and gets
fixed rather than documented.

The lint's own limits are narrower: it covers `crates/ucal/src` and not the
integration tests, and the hostile corpus is forty hand-chosen invocations
rather than a fuzzer.

### 6. A certified enclosure never narrows silently

An enclosure claims the true value provably lies inside it. Within `1.x`, an
enclosure may widen — a correction always may — and may narrow only when the
model or the depth changed and the release notes say so.

*Enforced by* the outward-rounding tests within a version: every step of the
quadrature widens, the two accumulator snaps are directed apart, and quantising
to ticks floors the lower bound and ceils the upper.

**Its limit.** Those checks run against one version. Nothing compares an
enclosure across releases, so this promise is the weakest on the list — it rests
on the release-notes discipline that has caught a `Breaking` omission twice, not
on a mechanism. It is listed anyway, because a promise stated with its weakness
is worth more than one silently omitted.

---

## The public surface, and why it is shaped as it is

Thirty-nine public types carried public fields or variants and no
`#[non_exhaustive]` when 0.6.0 opened. Each is now a recorded decision, and the
`public-type-is-classified` lint refuses to let a new one arrive undecided.

**Twenty-eight record types are `#[non_exhaustive]`.** The crates construct
them, callers read them, and they will gain fields — an anchor gains a
determination detail, an event gains a citation field, a model gains a
parameter. Marking them means that costs nobody a major version.

**Eleven vocabularies stay open**, because an exhaustive match on them is a
*feature* rather than an oversight:

| type | why closed |
|---|---|
| `Rounding` | a caller must handle every mode; a fifth changes what correct rounding means |
| `Form` | §6 names exactly these text forms; another is a specification change |
| `Scale` | §8.1 names exactly three time scales |
| `CivilCalendar` | §8.5 names exactly two, and both are legacy (§8.6) |
| `Kind` | §19.4's binary distinction: derived, or declared tables |
| `Precision` | the complete disjunction of Rule T |
| `Sign` | closed by arithmetic |
| `IntervalOrdering` | the complete lattice of interval comparison, including indeterminate |
| `Provenance` (body) | Rule C's binary: measured, or derived from something measured |
| `ColorChoice` | auto, always, never |
| `Frame` | already `#[non_exhaustive]`; a second profile may declare a different frame, which is the point of declaring it (Rule F) |

The list lives in `xtask/src/lint.rs` as `CLOSED_VOCABULARIES`, with these
reasons, so the record and the check are the same object.

### Two types callers construct

`#[non_exhaustive]` forbids a struct literal, so a type a caller must *build*
needs another way in. Two did, and the empirical test was simply marking
everything and seeing what stopped compiling.

**`Citation`** gained a `const fn new`. Every citation in this workspace is
declared in a `const` item and a third-party body, profile or event set must be
able to do the same.

**`Fmt`** gained a builder — `Fmt::default().with_form(…).with_precision(…)`.
The alternative was leaving it open and accepting that any new rendering option
is a breaking change. A caller writing `Fmt { .., ..Fmt::default() }` would have
survived that; one writing the exhaustive literal would not, and which of the two
a caller writes is not something this crate can influence. The builder makes the
safe form the only form.

### One decision that cost something immediately

`StatedAs` describes how a *source* states an event — after the datum, or before
the bridge epoch. A source could state one a third way, by redshift for
instance, so it is `#[non_exhaustive]`.

That broke a match in the CLI on the same commit, which is the decision working.
The fallback says the form is unrecognised and points at `as_published` for the
source's own words, rather than guessing a label — putting a wrong description of
a source's words into the output is the one thing `as_published` exists to
prevent.

---

## What 1.0 does *not* promise

### The specification is not frozen

UCAL-1 was superseded, not finished. `spec/SPEC-DELTAS.md` carries seventeen
entries — sixteen standing and one withdrawn — and D-A17 was written in 0.9.0.
Two of them, D-A16 and D-A17, sat recorded but **unapplied** in the normative
text until 0.9.0 went looking; `check-docs` now fails if a standing delta is not
marked inline in `UCAL-1.1.md`, because a delta that is written and not applied
reads as decided while the normative document still says the old thing. **1.0 freezes the API; the
specification keeps its amendment procedure.** A delta that changes behaviour
still produces a breaking change and still needs a major bump — the two are
independent.

### The text output is not an interface

`--json` is the contract. The human rendering — tables, colour, wrapping, column
order, the certification block — may change in any release. A script that parses
the text output is relying on something this file does not protect, and `--json`
exists so that it does not have to.

### A fixed minimum Rust version

`rust-version = "1.87"`, and the policy is:

- **An MSRV increase is a minor-version change**, never a patch. Within `1.x` a
  patch release compiles on every toolchain the preceding minor release did.
- **It is stated in the release notes**, in `Changed`, with what forced it.
- **The floor is set by dependencies as much as by this code.** Today it is
  `bnum 0.14`, which the default `u512` backend requires. A caller using only
  `bigint` can build `ucal-core` on 1.85 — but `rust-version` describes the
  default configuration, which is what a caller gets by typing `cargo add`.
- **`ucal`'s non-default `tui` feature needs 1.88**, and is the one stated
  exception. `ratatui` reaches `instability` and `darling`, which declare 1.88,
  so `cargo install ucal --features tui` does not build on 1.87. Nothing in the
  default configuration is affected and no library crate is: the feature exists
  only in the binary and only for `ucal wallclock`. The published release
  binaries are built on stable and carry it.

  Recorded rather than papered over. Pinning the transitive dependencies back to
  a 1.87-compatible set was possible and was not done: it would hold only until
  the next lock refresh, and a promise that survives by being re-fought every
  quarter is a promise about the maintainer's attention rather than about the
  software. If `tui` ever becomes a default feature, that is an MSRV bump and a
  minor-version change, by the first rule above.

*Enforced by* a CI job pinned to exactly 1.87 that builds the workspace and all
targets, plus a check that the manifest still declares 1.87, plus a check that
the `tui` exception is exactly one feature wide — that job asserts `--features
tui` fails at 1.87 and that everything else succeeds, so the exception cannot
quietly widen. The version pin is a literal rather than read from the manifest:
a job that follows the manifest would prove nothing about a manifest lowered by
hand.

*History, and the reason this is enforced rather than stated.* It read `1.85`
until 0.6.0 and had been **false** since `bnum` was updated — a caller on 1.85
could not build the default configuration at all. Nothing checked it, so nobody
knew. This is the same shape as every other finding in 0.4.0 and 0.5.0: the
claim without a mechanism was the claim that was wrong.

### Deprecation

Within `1.x`, nothing public is removed — that is the semver floor. What can
happen is a `#[deprecated]` attribute, which is a signal and not a removal:

- an item is deprecated in a minor release, with the replacement named in the
  attribute;
- it keeps working for the whole of `1.x`;
- removal waits for `2.0`.

A deprecation that removed something would be a breaking change wearing a
warning, which is worse than either.

### Coexistence with another library that chose the other backend

**It does not work, and 1.0 freezes that.** `u512` and `bigint` are mutually
exclusive. If two libraries in one dependency graph each depend on `ucal-core`
having chosen different backends, cargo unifies the features, the guard fires,
and *nothing in the graph builds*. Neither library author can fix it; only the
end user can, by making one of them change.

Demonstrated rather than assumed, in 0.9.0: two crates, one on each backend, and
an application depending on both fails with the guard's own message.

It also breaks tooling that expects features to be additive —
`cargo semver-checks` enables all features by default and cannot build this
crate without `--default-features`.

**Why it is here and not fixed.** Removing it needs either a generic integer
parameter through the whole API or the withdrawal of one backend from the public
surface. Both are breaking changes, and this is the release after which there
are none. It is recorded as a known limitation of `1.x` rather than left for a
downstream user to discover at link time.

### An enumerated set of feature combinations

**Supported**, and each built by CI on every push:

| crate | combinations |
|---|---|
| `ucal-core` | `u512`; `u512,alloc`; `u512,std`; `bigint`; `bigint,std` |
| `ucal-civil` | `u512,std`; `u512,std,hifitime` |
| `ucal-body`, `ucal-events`, `ucal-cosmo` | `u512,std` |
| `ucal` | `u512,std` plus any of `civil`, `body`, `events`, `cosmo` |

Either backend may be substituted for `u512` throughout; `bigint` implies
`alloc`, because a heap-backed integer cannot be built without one — which is
why GE-5's no-allocator build is a `u512` build by construction.

**`ucal-core` is the only crate that builds without an allocator.** The crates
above it are made of `Vec` and `String` and always will be.

**Refused, each with a stated reason rather than a cascade:**

| combination | what it says |
|---|---|
| no backend | *exactly one of `u512` or `bigint` must be enabled* |
| both backends | *mutually exclusive: Rule B makes the value width a wire-format commitment* |
| `ucal-body`/`events`/`cosmo` without `alloc` | *this crate requires the `alloc` feature* |

*Enforced by* `.github/workflows/features.yml`, which builds every supported
combination and asserts that each refused one **names its reason** — and, for
the two-backend case, that the reason is the *first* error. It previously
emitted "the name `imp` is defined multiple times" ahead of the guard, which is
the error a caller reads and the one that says nothing.

*What this changed.* Three combinations — `ucal-body`, `ucal-events` and
`ucal-cosmo` on `u512` alone — failed with twenty unresolved-`alloc` errors and
no explanation. They were unsupported by accident rather than by design, and the
difference is what a caller sees at 2am.

### Performance

The certified quadrature is slow on purpose — GE-1 measured depth-24 at hours
rather than seconds and the kill criterion fired. No release promises a runtime,
and a future release may be slower if being slower is more correct.

### The documents

The book, the Typst papers, and the proposals are not versioned with the crates
and carry no compatibility promise.

---

## What would make 1.0 dishonest

Recorded here because the pressure to ship a version number is real and arrives
without an argument attached.

**Shipping before contact — which is what happened.** The API has never been
used by anyone but its author. Every breaking change up to 1.0 cost nothing,
which means none of them tested whether the surface is *right* — only that it
could be changed. A promise made in that state is a promise about a guess.

This paragraph is kept, unedited in substance, because the argument did not stop
being true when the decision went the other way. 1.0.0 shipped with the gate
open, by the author's decision, after three cycles in which the asks in
[`CONTACT.md`](CONTACT.md) were stated, made cheap, and not taken up. What that
costs is specific and now unavoidable: **a finding that arrives from outside
needs a major version.** `2.0` is where it goes.

**Freezing a surface nobody has enumerated.** At the opening of 0.6.0, thirty-
nine public types carried public fields or variants and no `#[non_exhaustive]`.
Each is a decision 1.0 makes permanent in one direction or the other, and
shipping without deciding *is* deciding.

**Promising what is merely true today.** Every item above names a mechanism or
admits it has none. A list of things that happen to hold is not a promise; it is
a snapshot that a reader will mistake for one.
