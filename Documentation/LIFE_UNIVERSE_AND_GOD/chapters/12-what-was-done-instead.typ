#import "../design.typ": *
#import "@preview/cetz:0.4.2"

#chapter(number: 12, title: "What was done instead")

Three moves. Stipulate the datum. Declare the physical claim separately. Make the
claim impossible to compute with.

The first two are ordinary good practice. The third is the one this book is about,
and it is the only place where a code listing earns its space.

#section("The exhibit")

`BIG_BANG_CLAIM` is a value of type `SignedWindow`. Here is the type's entire
definition:

```rust
pub struct SignedWindow {
    lo: Signed,
    hi: Signed,
}
```

Two fields. What matters is everything that is absent, and the source says so
explicitly:

#claim("tradition")[
  This type deliberately has:

  - no arithmetic operators of any kind,
  - no `From<SignedWindow>` for `Delta`, `Instant` or `Window`,
  - no method returning any of those types.

  It cannot be added to an `Instant` and cannot be widened into one. Attempting to use
  it as an operand is `UCAL-E0025`, and the type system is what makes that unreachable
  rather than a runtime check.
]

The claim is fully available. You can read its bounds, render it, print its citation,
compare it to itself. What you cannot do is *arithmetic* with it. There is no
addition, no conversion, no escape hatch — and no runtime guard either, because a
runtime guard is a check that can be forgotten, disabled, or bypassed by a code path
nobody thought about.

#section("The tests that prove the absence")

An absence is hard to test. You cannot write an assertion that a method does not
exist, because the file would not compile if you named it.

So the project uses compile-fail tests: programs that are *required to fail to
build*, checked on every run.

#terminal(caption: "tests/compile_fail/signed_window_arithmetic.rs")[
```rust
use ucal_core::{Profile, UC1};

fn main() {
    let a = UC1::big_bang_claim();
    let b = UC1::big_bang_claim();
    let _ = a + b;
}
```
]

#terminal(caption: "tests/compile_fail/signed_window_as_operand.rs")[
```rust
use ucal_core::{Instant, Profile, UC1};

fn main() {
    let t: Instant<UC1> = Instant::zero();
    let claim = UC1::big_bang_claim();
    // A SignedWindow is not a Delta and must never become one.
    let _ = t.checked_add(&claim);
}
```
]

#terminal(caption: "tests/compile_fail/signed_window_into_delta.rs")[
```rust
use ucal_core::{Delta, Profile, UC1};

fn main() {
    let claim = UC1::big_bang_claim();
    let _d: Delta = claim.into();
}
```
]

Each of these is a person trying, in the most natural way available, to use the
uncertainty in the age of the universe as a number. Each fails to compile. The test
suite passes precisely because they do.

#claim("interpretation")[
  This is the whole thesis in nine lines of Rust, and it is worth spelling out why it
  is different in kind from a comment saying *do not do this*.

  A comment addresses a reader who is paying attention. A runtime check addresses a
  program that reaches a particular line. A type addresses *everyone who ever writes
  code against this library*, including the author in three years, including someone
  who has not read a word of this book, including someone who actively disagrees with
  the distinction being enforced.

  They will not be persuaded. They will be refused, by a machine, in under a second,
  with an error message. That is a different relationship between an argument and its
  audience than any essay can have.
]

#section("Why the type is signed at all")

A detail that is easy to pass over and is load-bearing.

The window is *signed* — it can express values before the datum — while `Instant` is
unsigned and cannot. That is not an inconsistency. It is the reason the type had to be
separate.

The FLRW limit may lie before tick zero. The published uncertainty is symmetric:
±0.020 Gyr around the datum, so its lower bound is negative. A type that could not
express that would have forced the claim to be truncated at zero, which would have
misreported the measurement.

#claim("interpretation")[
  So the design faced a genuine dilemma. The claim needs a representation the timeline
  cannot have; the timeline needs a discipline the claim would break.

  Splitting them into two types with no bridge is not a compromise between those
  requirements — it satisfies both exactly. The claim keeps its sign and its full
  symmetric range. The timeline keeps its unsigned domain. And the absence of any
  conversion is what stops the first from contaminating the second.
]

#v(3mm)
#block(breakable: false, width: 100%)[
#align(center, cetz.canvas({
  import cetz.draw: *
  let claim = rgb("#8a3a3a")
  // the tick axis
  line((-1.2, 0), (7.2, 0), stroke: 0.8pt, mark: (end: "straight"))
  content((7.35, 0), anchor: "west", text(size: 8pt, "ticks"))
  // datum
  line((1.6, -0.28), (1.6, 0.28), stroke: 1.2pt)
  content((1.6, -0.55), text(size: 8pt, weight: "bold", "tick 0"))
  content((1.6, -0.85), text(size: 7pt, fill: luma(100), "the datum"))
  // unsigned domain
  line((1.6, 0.62), (7.0, 0.62), stroke: 0.7pt)
  line((1.6, 0.52), (1.6, 0.72), stroke: 0.7pt)
  content((4.3, 0.9), text(size: 8pt, "Instant — unsigned, representable"))
  // the claim, straddling zero
  rect((0.72, -0.18), (2.48, 0.18), fill: claim.lighten(70%), stroke: 0.6pt + claim)
  content((1.6, 1.55), text(size: 8pt, weight: "bold", fill: claim, "BIG_BANG_CLAIM"))
  content((1.6, 1.28), text(size: 7.5pt, fill: claim, "±0.020 Gyr = ±141.53 drifts"))
  line((1.6, 1.15), (1.6, 0.25), stroke: (thickness: 0.4pt, paint: claim))
  // the part that is not representable
  content((0.55, -0.55), anchor: "east", text(size: 7.5pt, fill: claim, "not a tick (N12)"))
  line((0.6, -0.5), (1.05, -0.14), stroke: (thickness: 0.4pt, paint: claim))
  // the barrier
  content((4.3, -1.35), text(size: 8pt, "no operator, no conversion, no method"))
  content((4.3, -1.68), text(size: 7.5pt, fill: luma(100),
    "crossing this line is UCAL-E0025 — and does not compile"))
  line((0.7, -1.05), (7.0, -1.05), stroke: (thickness: 0.6pt, dash: "dashed"))
}))
#v(1mm)
#figcap[5][
  `BIG_BANG_CLAIM` against the timeline. The claim straddles the datum and extends
  where no tick exists; the timeline is unsigned. Nothing converts between them.
]
]

#section("The provenance chain")

The second move — declaring the claim separately — would be worth little if the
declaration were prose. It is data, and it re-executes.

#terminal(caption: "ucal datum — the chain")[
```
datum_provenance:
  input     13.787 Gyr +/- 0.020 Gyr (age_of_universe)
  citation  Planck 2018 results VI, A&A 641, A6 (2020)
  unit_defs:
    Gyr = 10^9 x 31 557 600 s (Julian years, exact)
  chain:
    AGE_s     = 13 787 000 000 x 31 557 600
              = 435 084 631 200 000 000 s        (exact)
    AGE_ticks = AGE_s x SECOND
              = 8070204002895596516263200000...  (exact)
    beats     = round_half_even(AGE_ticks / BEAT)
              = 9 304 311 741 502 590 385
    ORIGIN_OFFSET
              = beats x BEAT
  residual_ticks     -3188569143643628194695338...
  residual_rendered  -0.017190364 s
  rationale  a whole-beat datum makes all sub-beat
             digits of the bridge epoch zero (2.4)
```
]

Every step is there: the cited input, the unit definitions, each exact
multiplication, the rounding to a whole beat, and — the part that matters most — the
*residual that the rounding discarded*.

Seventeen milliseconds. The datum is not the published age; it is the published age
rounded to a whole beat, and it differs from the input by 0.017190364 seconds. The
system prints that number rather than absorbing it.

#claim("interpretation")[
  A provenance chain that reports its own rounding error is doing something a
  citation cannot. A citation says *where the number came from*. This says where the
  number came from *and what happened to it on the way in* — which is the part that
  a reader checking the work would otherwise have to reconstruct.

  The chain is also re-executable, which means the claim "this constant follows from
  that measurement" is testable rather than assertable. The constants harness does
  exactly that on every run, along two independent integer routes.
]

#section("Kant policed it by discipline; the crate polices it by the compiler")

Chapter 11 borrowed Kant's First Antinomy. Here is the borrowing that goes deeper.

#claim("tradition")[
  Kant distinguishes *constitutive* principles, which tell you what objects are, from
  *regulative* ones, which tell you how to go on investigating. The idea of the
  world-whole is regulative: it directs reason to keep extending the series and never
  licenses the assertion that the completed series exists.

  Transcendental illusion, for Kant, is precisely the slide from the second to the
  first — taking a rule for how to proceed as a description of an object.
]

The datum is a regulative posit. It says *count from here*; it does not say *here is
where time began*. The physical claim is the constitutive-looking statement, and it is
the one that has been rendered inert.

#claim("interpretation")[
  Kant had no mechanism for this. He had argument, and vigilance, and the expectation
  that a careful reader would maintain the distinction under pressure. He says himself
  that the illusion does not go away once diagnosed.

  What this project has is not a better argument. It is a worse philosopher with a
  compiler: the distinction is maintained by something that does not get tired, does
  not skim, and does not want the flattering answer.

  Whether that counts as philosophical work or merely as engineering hygiene is the
  question Part VIII exists to argue. The claim made here is narrower and, I think,
  hard to dispute — that a distinction which previously depended on the reader's
  attention now does not.
]

#section("UC-Θ, unbuilt")

One thing this part must not do is imply the design is finished.

There is a second profile that has been specified and not built. Call it UC-Θ. In it,
the datum is not the FLRW limit but the beginning of time *at organization* — a
cosmology in which matter is organised rather than created from nothing, so that a
physical origin lies later than the reckoning's zero, and there is a "before" that is
not a *when*.

#claim("interpretation")[
  Two things make UC-Θ interesting rather than idle.

  The unsigned domain, which under UC-1 is a limitation to be defended, becomes *room*
  — the interval between the datum and the physical origin is representable rather
  than refused.

  And Rule X becomes inapplicable. The cosmology module derives ages under a declared
  ΛCDM model anchored on the FLRW limit; move the datum off that limit and the
  certified enclosures no longer mean what they meant. A whole subsystem would have to
  be re-argued rather than re-parameterised.

  That is what makes the profile mechanism more than versioning. Two profiles here are
  not two configurations of one system. They are two cosmologies, and Part VI, chapter
  21 has to say plainly that UC-Θ is heterodox by the standard of the tradition that
  supplied chapter 13's cautionary tale.
]

It is listed as unbuilt because it is unbuilt. If it is ever implemented, this section
becomes a worked second datum and the book will have to say what changed.

#recap((
  [`BIG_BANG_CLAIM` is a `SignedWindow`: two fields, and no arithmetic operators, conversions, or methods returning a computable type.],
  [Three compile-fail tests encode the absence — each is a natural attempt to use the claim as a number, and each must fail to build.],
  [The type is signed because the claim straddles the datum; `Instant` is unsigned. Splitting them satisfies both requirements exactly rather than compromising between them.],
  [`datum_provenance` re-executes: cited input, exact chain, and the −0.017190364 s residual that rounding to a whole beat discarded.],
  [Kant maintained the constitutive/regulative distinction by vigilance and said the illusion persists after diagnosis. Here it is maintained by something that does not get tired.],
  [UC-Θ is specified and unbuilt. Under it the unsigned domain becomes room, and Rule X becomes inapplicable — which is why profiles are cosmologies, not configurations.],
))
