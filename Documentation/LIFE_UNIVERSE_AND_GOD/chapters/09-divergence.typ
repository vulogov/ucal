#import "../design.typ": *
#import "@preview/cetz:0.4.2"

#chapter(number: 9, title: "Divergence")

Parts I and II described a system. This part describes the arguments that system won
against the person who specified it.

That framing is deliberate. A specification written before implementation is a set of
predictions about what will turn out to be buildable, and some of them are wrong. The
usual practice is to quietly fix the specification afterwards so that it appears to
have been right all along. This chapter is the record of not doing that.

#section("What was checked, and how")

Before a line of the library was written, every constant and derivation in the
specification was recomputed independently — twice, by two separate integer routes
that share no code, and compared against what the document claimed.

That harness still runs:

#terminal(caption: "cargo run -p xtask")[
```
UC-P0 constants harness — RFC UCAL-1, profile UC-1

  ok    routes A and B agree on every field
  ok    §3.3 profile constants are const-constructible and correct
  ok    BEAT: 5^60 (computed) == literal
  ok    provenance: AGE_s
  ...
  96 passed, 0 failed
```
]

It produced fifteen findings. Fourteen stand; one was withdrawn, and the withdrawal is
the most instructive of the lot.

#section("The map")

Twenty-four named rules. Six were amended by contact with the implementation.

#block(breakable: false, width: 100%)[
#v(3mm)
#align(center, cetz.canvas({
  import cetz.draw: *
  let kept = rgb("#e4e0d6")
  let amended = rgb("#7a4a2f")
  let rules = (
    ("Q", false), ("A", false), ("Y", false), ("Z", false), ("M", false), ("F", false),
    ("P", false), ("W", false), ("O", false), ("E", false), ("R", false), ("G", false),
    ("N", false), ("T", true),  ("U", false), ("D", true),  ("S", false), ("B", true),
    ("I", false), ("L", false), ("K", true),  ("J", false), ("C", true),  ("X", true),
  )
  let cols = 8
  for (i, (name, amend)) in rules.enumerate() {
    let cx = calc.rem(i, cols) * 1.05
    let cy = -calc.floor(i / cols) * 1.05
    rect((cx, cy), (cx + 0.82, cy + 0.82),
      fill: if amend { amended } else { kept },
      stroke: 0.4pt, radius: 0.06)
    content((cx + 0.41, cy + 0.41),
      text(size: 11pt, weight: "bold",
        fill: if amend { white } else { luma(40) }, name))
  }
  // legend
  rect((0, -3.5), (0.35, -3.15), fill: kept, stroke: 0.4pt, radius: 0.04)
  content((0.5, -3.32), anchor: "west", text(size: 8pt, "survived unchanged — 18"))
  rect((4.2, -3.5), (4.55, -3.15), fill: amended, stroke: 0.4pt, radius: 0.04)
  content((4.7, -3.32), anchor: "west", text(size: 8pt, "amended — 6"))
}))
#v(1mm)
#figcap[10][
  The twenty-four rules by fate. None was dropped; six changed, and four of the six
  changed because the implementation proved the original could not be built as
  written.
]
]

#v(2mm)
#block(width: 100%)[
  #set text(size: 9pt)
  #table(
    columns: (auto, auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 4.5pt),
    [*rule*], [*delta*], [*what changed*],
    [T — truncation is uncertainty], [D-A8], [what a printed form *means*, and how each form anchors],
    [D — two text forms], [D-A8, D-A4], [human anchors at T0, digit at T32; the fixtures were truncated],
    [B — canonical binary], [D-A7], [full-width encode is 45 `divmod` steps, not 44],
    [K — one derivation mechanism], [D-A5], [grouping cycles are declared per body, not admitted by a global bound],
    [C — parameter provenance], [D-A11, D-A13], [obliquity cannot be a `RatedParam`; a drift bound is a rate, not a duration],
    [X — certified enclosures], [D-A14], [the cosmological integral cannot be quadratured as written],
  )
]

#section("Three kinds of losing")

The fourteen findings sort into three classes, and the classes matter more than the
count.

*Editorial* — five findings where the specification stated a number or a description
inaccurately, with no behavioural consequence. `ORIGIN_OFFSET` has 61 trailing base-5
zeros, not 62. Appendix B's seconds column was chained from neighbouring rows instead
of computed independently, and drifts in the fifth significant figure. These are the
cheap ones.

*Correction* — six findings where the specification was wrong and the implementation
could not follow it. The synodic-period formula computed the wrong quantity. A drift
bound was typed as a duration when it is a rate. The cosmological integral, as
written, was improper and could not be certified at all.

*Amendment* — three findings where the specification was coherent, buildable, and
adopted differently on purpose after implementation revealed what it cost.

#claim("interpretation")[
  The amendments are the interesting category, because nothing forced them.

  D-A5 is the clearest. The specification admitted a satellite as a "grouping
  satellite" — a source of months — if its synodic period fell within a bracket of 5
  to 100 solar days. That is a perfectly implementable rule. It was replaced because
  the bracket is *calibrated on Earth's Moon*, which puts an Earth-derived constant
  inside the one mechanism whose entire purpose is to keep Earth from being the
  template.

  Nothing broke. The tests passed. The rule was changed because implementing it made
  visible that "month-like" is not a derivable predicate — *month-like* is an Earth
  predicate — and a bound tuned until Mars gave the expected answer would have been
  fitting the constant to a conclusion.
]

#section("What each change cost")

Honesty requires the costs alongside the corrections.

D-A5 cost a calendar the ability to have months by default: a body must now *name* a
grouping satellite, with a citation for why. Mars names none, so `mars-d` has no
months at all. That is the correct output, and it is also less than the specification
promised.

D-A13 cost a type. `max_drift: &Delta` became `DriftBound { days, per_years }` in the
body's own local units — which means the bound cannot be stated in seconds, which
means a caller who thinks in seconds has to convert first and think about what they
are doing.

D-A14 cost the cosmology module its shape. The specification's integral runs to
infinity; certified quadrature needs a compact interval. The substitution
$u = 1 \/ (1+z)$ fixes it, and finding that substitution was a day of work that the
specification's phrasing implied would not be necessary.

#section("The withdrawal")

One finding was raised and retracted, and it stays in the record.

The verification claimed that Appendix C's fixture for 15 March 44 BC was one day
late. It was not. The checking oracle was wrong: it applied an era adjustment that
exists to make *truncating* integer division behave like flooring, on top of a
language whose division already floors. The correction was correct for positive
years, which is why seven of the eight fixtures passed and only the one negative year
failed.

Appendix C is eight for eight.

#claim("interpretation")[
  A retracted claim is more useful in the record than an absent one, and the reason is
  not modesty.

  A verification process that only ever reports findings is indistinguishable from one
  that manufactures them. The retraction is the evidence that the oracle was itself
  checked — that when the specification and the harness disagreed, the harness was not
  automatically believed.

  It is the smallest of the fifteen entries and it does the most work.
]

#section("Did it diverge enough?")

The plan for this book set a condition on Part III existing at all: if the
implementation turned out to follow the specification closely, this part would
collapse into two pages inside Part II, because a book claiming that implementation
talks back has to show that it did.

The answer is yes, and here is the count. Fourteen standing findings. Six of
twenty-four rules amended. Six places where the specification was simply wrong. One
where its central method could not be executed as written. And separately from the
rules, four of the project's six gated experiments hit their kill criteria — including
one that fired before implementation started, when the integer library the
specification named changed its API mid-build.

#callout(label: "The count is not the argument")[
  Fourteen findings out of a document this size is not a scandal, and it is not
  presented as one. Specifications of this length routinely contain more.

  What Part III claims is narrower: that the findings were *recorded* rather than
  absorbed, that the record distinguishes what was wrong from what was merely changed,
  and that one entry is a retraction. Those three properties are what make the next
  chapter's claim believable, and the next chapter is why this part exists.
]

#recap((
  [Every constant and derivation was recomputed by two independent routes *before* the library was written; the harness still runs, 96 checks.],
  [Fifteen findings: five editorial, six corrections, three amendments — and one withdrawal.],
  [Six of twenty-four rules were amended. None was dropped.],
  [The amendments were not forced. D-A5 replaced a working rule because implementing it revealed an Earth constant inside the mechanism built to exclude Earth.],
  [Each change cost something, and the costs are recorded next to the corrections.],
  [The withdrawal stays in the record, because a process that only ever finds things is indistinguishable from one that invents them.],
))
