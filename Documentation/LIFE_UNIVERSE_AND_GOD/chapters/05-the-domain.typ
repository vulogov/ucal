#import "../design.typ": *

#chapter(number: 5, title: "The domain")

Part I established what is counted. This part is about how the counting is
implemented, and it starts with the number that holds the count.

Three properties, and the third is the one that turns out to have consequences all
the way into the cosmology: the domain is *unsigned*, it is *closed*, and every
operation on it is *checked*.

#section("Unsigned, and why that is not a saving")

There is no tick $-1$. Chapter 3 said why in one line — nothing precedes the datum,
because the datum is where the count begins — and the type system is where that
line is enforced.

`Instant` has no `Sub` implementation returning a signed value. Subtracting a later
instant from an earlier one does not produce a negative `Delta`; it produces
`UCAL-E0020` and the operation fails. To get the distance between two instants
regardless of order you must ask for it by name, and the name says what you are
doing.

#callout(label: "The same discipline, applied to rationals")[
  `Ratio` — the exact rational type used throughout the derivations — carries the
  identical rule. `sub` refuses a negative result and directs the caller to
  `abs_diff`. That is Rule Z applied one level down, and it means the same class of
  mistake is impossible in the calendar derivations as in the timestamps.
]

#section("Closed: 512 bits, and what that buys")

The tick count is a 512-bit unsigned integer. The ceiling is $2^512 - 1$, which the
system will print for you:

#terminal(caption: "ucal doctor — the domain")[
```
domain_max_ticks  13407807929942597099574024998205846127479365820592393377
                  72356144372176403007354697680187429816690342769003185818
                  6486050853753882811946569946433649006084095
domain_bits       512
```
]

In years, that ceiling is about $2.29 times 10^103$.

To give that a scale: proton decay, if it happens at all, is expected somewhere
around $10^34$ years. The last black holes evaporate around $10^100$. The domain
outlasts both, with room left.

The present epoch sits at about $8.07 times 10^60$ ticks — which is
$6.0 times 10^(-94)$ of the range. The counter is, for all practical purposes,
still at zero.

#callout(label: "A number this book had to correct")[
  RFC UCAL-A1 attaches the figure $7 times 10^(-17)$ to this paragraph. That number
  is real but belongs to a different range: it is the present epoch as a fraction of
  the *UCID's* 256-bit space, not of the 512-bit domain. The domain fraction is
  $6.0 times 10^(-94)$.

  Rule S is why this correction is here rather than the original figure. The article
  RFC describes what the book intended to say; the source tree is what is true. This
  is the first place in the book where those two disagreed, and it will not be the
  last.
]

#claim("interpretation")[
  The width was not chosen to be impressive. It was chosen so that *the width never
  has to change*, because under Rule B the value width is a wire-format commitment:
  the canonical binary encoding is 64 bytes because the domain is 512 bits, and
  widening the domain later would invalidate every stored timestamp in existence.

  A system that might need to grow its integer has a migration in its future. This
  one bought its way out of that with sixty-four bytes, once.
]

#section("Checked: overflow is a typed error")

No operation on a time type wraps, and none saturates.

Every arithmetic method that could exceed the domain returns a `Result`. Overflow is
`UCAL-E0060`, an ordinary error you handle, not a silently wrong answer you ship. The
release profile sets `overflow-checks = true`, so the guarantee holds in optimised
builds and not only in debug ones — which is where this class of bug normally hides.

There is also a workspace lint that fails the build on `wrapping_*` or `saturating_*`
appearing on a time type. That is belt and braces, and it is there because the
failure mode it guards is the quietest one available: a wrapped tick count is not an
error message, it is a timestamp from the wrong end of eternity that looks entirely
plausible.

#section("Two backends, one domain")

The tick integer has two implementations. The default is a fixed 512-bit stack
integer; the alternative is an arbitrary-precision heap integer. They are selected by
feature flag and are mutually exclusive at compile time.

The interesting requirement is that they must accept and reject *exactly the same
values*. The arbitrary-precision backend is unbounded and would happily represent
$2^513$; it is made to enforce the same ceiling anyway, and the whole test suite runs
against both.

#claim("interpretation")[
  It would have been easier to let the unbounded backend be unbounded. The reason not
  to is that the two would then be different systems wearing the same name — a
  timestamp accepted by one and rejected by the other, with no way to tell from the
  type which you were holding.

  The rule that forbids this is one line long, and enforcing it cost a full second
  copy of the test suite in CI. That ratio — a cheap sentence, an expensive
  guarantee — is characteristic of the whole design.
]

#section("No floating point. Anywhere.")

This is the constraint with the largest blast radius, so it gets stated in its
strongest form: there is no floating-point value in any shipped crate of this
workspace. Not in a signature, not in a field, not in a constant, not in an
intermediate, and not in the rendering path that produces the human-readable output.

A lint scans the source and fails the build on any float token. It permits exactly
one exception — a float reference implementation used as a *test oracle* — and it
requires the exception to be marked, and it prints every exemption it honoured on
every run, so an escape hatch cannot quietly become a retreat.

#subsection("What it costs")

The cost is real and it is concentrated in one place: cosmology.

Computing the age of the universe at a given redshift means evaluating an integral.
In any normal library that is a call to a quadrature routine over `f64`, and it
returns a number with an error estimate. Here there is no `f64`, so it is certified
interval quadrature over exact rationals — thousands of panels, each bounded above
and below, with directed integer square roots underneath.

It is slower by orders of magnitude. The default subdivision depth takes about half a
second where a float routine would take microseconds.

#subsection("What it buys")

An *enclosure* rather than an estimate.

The float routine returns a number and a tolerance, and the tolerance is a
well-informed guess about accumulated error. The interval routine returns two numbers
and a proof that the true value lies between them. Not probably. Provably, given the
model.

#claim("interpretation")[
  For most software the float answer is the better engineering, and it is not close.
  A guess accurate to twelve digits, computed instantly, beats a proof accurate to
  four, computed slowly.

  This project takes the other option because its subject is a quantity whose
  uncertainty is the interesting part. Part IV is four chapters about an origin that
  cannot be measured; a cosmology module that reported a confident point estimate
  would be undermining the book's central claim in the one place where the claim is
  most testable.
]

#section("The two widths")

One consequence deserves its own note, because it is where the arithmetic discipline
becomes an argument.

Every cosmological result carries *two* uncertainty widths, and they are never merged:
the width contributed by the quadrature, and the width contributed by the model's own
measured parameters. Asking for a single combined tolerance requires calling a method
with a name that says you are combining them.

At recombination the quadrature width is about 251 years and the parameter width is
about 10,900 years. Merged into one number, the answer would be "about eleven thousand
years of uncertainty" — and the fact that forty-three forty-fourths of it comes from
the *measurement* rather than from the computation would be invisible.

That distinction is what tells you more computing power would buy nothing here. A
merged tolerance hides it. Keeping them apart is a line in the specification, and it
is the reason the specification has that line.

#recap((
  [The domain is unsigned: a result before the datum is `UCAL-E0020`, not a negative number. The same rule applies one level down, to `Ratio`.],
  [512 bits reaches $2.29 times 10^103$ years, past proton decay and black-hole evaporation. The present epoch is $6.0 times 10^(-94)$ of it — and the width never has to change, because it is a wire-format commitment.],
  [Overflow is a typed error, enforced in release builds and by a lint, because a wrapped tick count looks entirely plausible.],
  [Two backends, one domain: the unbounded one enforces the same ceiling, verified by running the whole suite twice.],
  [No floating point anywhere. It costs orders of magnitude in the cosmology and buys a certified enclosure instead of an estimated tolerance.],
))
