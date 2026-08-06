#import "../design.typ": *

#chapter(number: 14, title: "Universal baselines")

The claim this part examines is that the approach works for any body, not just Earth.
That is a strong claim, and the next three chapters test it: this one says why it
should be true, chapter 15 shows it working, and chapter 16 says where it fails — at
no less length than chapter 15, because a capability section longer than its limits
section is a sales document.

#section("Why it generalises")

Three properties, each established in Part I or II, and together they are the whole
argument.

*The tick is not an Earth artifact.* It is composed from $G$, $planck$ and $c$. A
Martian, a Titanian, or anyone else running the same physics arrives at the same
unit — with the caveat from chapter 2 that its *length in seconds* is a metrological
convention, and the caveat matters less here than it did there, because nobody
counting ticks needs the conversion.

*The tier ladder has no body content.* It is the powers $5^(5k)$. There is no rung
called "day" and none called "year", and the spacing was chosen for digit packing.
Nothing on the ladder would have been different if Earth had a thirty-hour rotation.

*The arithmetic never references a rotation, an orbit, or a civil calendar.* This is
the one that can be checked rather than argued, and chapter 7 described the check: a
workspace lint fails the build if any identifier in the core crate names a foreign
unit outside the single bridge declaration.

#claim("interpretation")[
  The third property is doing more work than the first two, and it is worth separating
  them.

  The first two are claims about *choices*: the tick and the ladder were selected to be
  body-independent, and one could take that on trust or verify it by reading the
  definitions.

  The third is a claim about *what the code cannot contain*, enforced mechanically.
  Body independence in this system is not a design intention that survives as long as
  contributors remember it. It is a property the build checks.
]

#section("One mechanism, and Earth as an instance")

Chapter 8 gave the shape: every calendar is (Body, Anchor, LeapRule, Cycles). What
matters for this part is that there is exactly *one* implementation of that shape.

There is no Earth path and no Mars path. There is no `if body == earth`. There is no
crate named after a body — the crate is `ucal-body`, and Earth, Mars, Titan, the
Moon, Mercury, Venus and Jupiter are entries in a data table it reads.

#terminal(caption: "ucal cal list — seven calendars, one mechanism")[
```
earth-d:    kind  derived — Rule K    body  earth
mars-d:     kind  derived — Rule K    body  mars
titan-d:    kind  derived — Rule K    body  titan
luna-d:     kind  derived — Rule K    body  luna
mercury-d:  kind  derived — Rule K    body  mercury
venus-d:    kind  derived — Rule K    body  venus
jupiter-d:  kind  derived — Rule K    body  jupiter
```
]

The list grew from three to seven without the mechanism changing, which is the
only kind of evidence this claim can have. Adding a body is adding a row of cited
parameters; nothing else moved.

The test that matters here constructs `earth-d` and `mars-d` through the identical
generic path, from data alone, and asserts that neither required a special case. If
someone adds a branch that treats Earth differently, that test is what notices.

#callout(label: "Why this needed enforcing at all")[
  Because the failure it prevents is not a bug, it is a *drift*.

  Nobody sets out to make Earth the template. What happens is that Earth is
  implemented first, because it is the one you can check against a wall calendar. A
  convenience gets added for it. A constant gets a default that happens to be Earth's.
  A test asserts something true only of bodies with one large moon. None of these
  breaks anything, and after enough of them the mechanism is an Earth calendar with
  parameters.

  Failure mode F9 is that drift, and Rule K is the response: one mechanism, Earth as
  an ordinary instance, checked by a test that builds two bodies the same way.
]

#section("The Copernican point, stated at its actual size")

There is a temptation here that should be named and declined.

It would be easy to write that this system dethrones Earth the way Copernicus
dethroned it — that a calendar with no privileged body is a philosophical achievement
of the same kind. The sentence writes itself, and it would be an overclaim.

#claim("interpretation")[
  What was actually done is narrower and it is worth stating precisely, because the
  narrow version is defensible and the grand one is not.

  Earth was moved from being the *template* of the mechanism to being an *instance*
  of it. That is a change in software architecture with a philosophical shape, and its
  entire content is: the structure that generates calendars does not contain a planet.

  It is not a claim about the universe. It does not show that Earth is unimportant, or
  that a body-independent reckoning is more true than a local one. Chapter 22 will
  argue, from a tradition that has thought about it longer than this project has, that
  a calendar is by definition an instrument of the created order — which cuts against
  reading any of this as transcendence.

  And the privilege was not abolished. It was *relocated*, from a body to a power-of-5
  grid, and a grid is not nothing. Chapter 23 is where that gets pressed properly, by
  a tradition that puts a governing body exactly where this system puts an abstract
  ladder.
]

#section("What generalising does not include")

One boundary, stated here so chapter 15 cannot be read as claiming more than it shows.

The mechanism generalises over *bodies*. It does not generalise over *physics*. There
is no relativistic model here: no time dilation, no worldline, no frame transformation
beyond the single comoving frame the profile declares. Two clocks in different
gravitational potentials are not modelled as running at different rates, because
nothing in this system models rates at all — it counts.

That is a real limit and chapter 16 lists it among the others. It is mentioned here
because "works for any celestial body" is the kind of phrase that expands while nobody
is watching.

#recap((
  [The tick is built from constants, the tier ladder is a power ladder, and neither contains a body.],
  [The arithmetic cannot reference a rotation, an orbit, or a civil calendar — checked by a lint, not by intention.],
  [One mechanism, no per-body path, no crate named after a body; a test builds Earth and Mars identically from data alone.],
  [The failure this prevents is drift rather than a bug: Earth becomes the template one convenience at a time.],
  [Earth moved from template to instance. That is the claim, at its actual size — the privilege was relocated to a grid, not abolished.],
  [The mechanism generalises over bodies, not over physics. There is no relativistic model.],
))
