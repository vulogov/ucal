#import "../design.typ": *

#chapter(number: 22, title: "Orthodox")

This direction gives the book the strongest licence its central sentence receives
anywhere, the technical term the research had been circling since Cusanus, and a
conflict that puts Rule N on the wrong side of a dispute involving the two thinkers
this book most depends on.

#section("What the direction holds")

#claim("tradition")[
  *Essence and energies.* Gregory Palamas, defending hesychast practice, distinguishes
  God's *essence* — utterly unknowable, imparticipable — from the *energies*, the
  uncreated activities by which God is genuinely known and participated. The
  distinction is real and not merely conceptual: what is known is truly God, and the
  essence remains beyond knowing.

  *Diastema.* Gregory of Nyssa, against Eunomius, makes διάστημα — interval, extension,
  the spread between before and after — the mark of createdness. Everything created is
  διαστηματικός; God is ἀδιάστατος, without interval. The gap between creature and
  Creator is not one of degree along a scale but of *kind*: there is no interval in
  God for a measure to measure.

  *Three modes.* Maximus the Confessor distinguishes ἀϊδιότης, the eternity proper to
  God; αἰών, the mode of created beings in their fixed existence, a "before" that is
  not a "when"; and χρόνος, measured successive time.

  *The name.* The имяславие controversy of the early twentieth century turned on
  whether the divine name *participates* in what it names — whether "the Name of God is
  God" — or is a designation. The Russian Synod condemned the affirmative position in
  1913. Florensky and Losev defended it.
]

#section("Which rule it illuminates")

The thesis itself. Rule N, on names as display-only. UC-Θ. And §8.6's `LegacyCalendar`.

#section("The convergences")

#subsection("Palamas is the licence")

The book's thesis is that an instrument may point at what it cannot describe, provided
it declares that it is only pointing.

Palamas' distinction is the strongest theological warrant that sentence receives from
any direction surveyed. Essence unknowable; energies manifest and genuinely known. The
structure permits real knowledge of something whose essence is beyond knowing — which
is precisely what "pointing without describing" requires in order not to be empty.

#claim("interpretation")[
  I want to be careful here, because this is the place where a book like this most
  wants to overreach.

  Palamas is not talking about calendars, and the essence/energies distinction is a
  claim about God, not a general licence for instruments that gesture at what they
  cannot measure. Taking it as one would be quarrying — using a tradition's
  vocabulary to dignify a much smaller point.

  What can honestly be said: the *form* of the thesis has a precedent here in which
  the two clauses are held together rigorously rather than as a rhetorical balance.
  That is worth something, and it is less than a licence.
]

#subsection("Diastema is the word the research was looking for")

This is the most useful single term Part VI produced.

#claim("tradition")[
  If διάστημα is the mark of createdness, then anything that measures interval is by
  definition an instrument of the created order. Not accidentally, not because of a
  technical limitation, but because interval is what created things have and what God
  does not.
]

Everything this system measures is διάστημα. The tick count is an interval. A `Delta`
is an interval. A `Window` is an interval with uncertainty. The whole apparatus, top to
bottom, measures the spread between before and after.

#claim("interpretation")[
  So the limitation stated in this book's title is not a defect in the instrument. It
  is a *doctrine* about what the instrument is measuring.

  A calendar cannot reach past the created order, and the reason is not that calendars
  are crude. It is that a calendar measures interval, interval is the mark of the
  created, and there is no interval in what lies beyond it — nothing for the measure to
  be a measure *of*.

  Cusanus said no proportion holds between finite and infinite. Nyssa says why: they
  are not two magnitudes of different size but two conditions, one with διάστημα and
  one without.

  Chapter 30 owes this chapter a debt. "An instrument for the immeasurable" reads as a
  paradox in the title; here it stops being one and becomes a statement about
  categories.
]

#subsection("Maximus supplies what UC-Θ needs")

Chapter 12 described UC-Θ as requiring a "before" that is not a "when" — an origin of
ordering that is not itself a moment in the ordering.

αἰών is exactly that. It is not eternity and not time; it is the mode of created being
that has a beginning without that beginning being a temporal position.

Whether UC-Θ can actually be built on it is another question, and chapter 21's conflict
stands regardless. But the concept UC-Θ was reaching for exists, is technical, and is
seventh-century.

#subsection("The Julian retention defends `LegacyCalendar`")

This is the convergence with the most practical force in the book.

Chapter 8 classified `earth-civil` as legacy — declared tables, four items of arbitrary
content, a leap rule that is not a convergent — and chapter 10 established just how far
`97/400` sits from the derived ladder.

The Orthodox churches that retain the Julian calendar retain a rule, `1/4`, that is the
*crudest* convergent available: one day of drift in 128 years. Every church using it
knows this. It has been debated for a century, and the calendar has been retained
anyway, for reasons of communion and continuity that have nothing to do with accuracy
against the tropical year.

#claim("interpretation")[
  That is the strongest available argument that *derivation is not a calendar's only
  criterion*, and it is an argument from practice rather than from theory.

  A calendar is a thing a community keeps together. Accuracy is one property of it. Not
  breaking communion is another, and there is no arithmetic that ranks them.

  So `LegacyCalendar` is not a holding pen for calendars that failed to be derived. It
  is a category for calendars whose authority comes from somewhere the mechanism cannot
  reach — and chapter 8's insistence that "legacy" is a classification rather than a
  judgement gets its clearest vindication here.
]

#conflict[
  *Two, and the second is the one that hurts.*

  The *ex nihilo* conflict of chapter 21 applies here in full. UC-Θ is heterodox by this
  direction's standard too, and for the same reasons.

  The second is Rule N. It says: *the canonical identity of a tier is its exponent;
  names are display-only.* Nothing in the system decides behaviour from a name; names
  live in a locale table alongside their Russian translations, and the tier is the
  exponent.

  That is a position on the relationship between a name and what it names. It says the
  name is a label attached to an identity that exists independently of it.

  In the имяславие dispute, that is the Synodal position of 1913 — the one Florensky
  and Losev opposed. Rule N sides against them.

  Chapter 13's cautionary tale is built on Florensky. Chapter 25 depends on Losev.
  These are the two thinkers this book most needs, and on the question of what a name
  *is*, the artifact holds the view they were persecuted for opposing.
]

#section("What it changes")

#claim("interpretation")[
  Two things, and the second is unusual because it is falsifiable.

  First, the title stops being a paradox. Διάστημα makes "an instrument for the
  immeasurable" a statement about what kind of thing an interval-measure is. Chapter 30
  should be written from here rather than from the engineering side.

  Second: *the Rule N conflict can be settled in code rather than on the page.*

  Rule N holds that a tier's identity is its exponent and its name is display. If that
  is true, then no behaviour anywhere in the system should ever depend on a name. The
  lint and the tests currently assume this.

  But it is an empirical claim about the software, and it can fail. If some future
  feature makes a tier name load-bearing — if a name ever determines behaviour rather
  than merely rendering it — then Florensky and Losev win the argument *on the machine*
  rather than in the commentary, and Rule N would have to be amended rather than
  defended.

  That is the strangest thing Part VI produced: a hundred-year-old theological dispute
  with a test condition in a Rust workspace. I do not think it is trivial. Rule N was
  written to prevent a display concern from acquiring semantics, and the имяславие
  position is precisely that display concerns *do* acquire semantics when what is
  named is the sort of thing a name can participate in.

  A tier is not that sort of thing, and the analogy should not be pressed further than
  the shared structure warrants. But the test condition is real, and if it ever fires
  this chapter has to be rewritten.
]

#recap((
  [Essence and energies; διάστημα as the mark of createdness; ἀϊδιότης / αἰών / χρόνος; the name as participating or designating.],
  [Palamas gives the thesis its strongest precedent — with the caution that it is a claim about God, not a general licence for instruments.],
  [Διάστημα is the term the research was circling: everything this system measures is interval, and interval is what created things have. The title's paradox dissolves into a statement about categories.],
  [The Julian retention — keeping the *crudest* convergent knowingly, for a century of debate — is the strongest available defence of `LegacyCalendar` as a classification rather than a judgement.],
  [*Conflict:* Rule N sides with the 1913 Synodal position against Florensky and Losev, the two thinkers this book most depends on.],
  [*What changes:* that conflict is falsifiable in code. If a tier name ever becomes load-bearing, they win on the machine and Rule N is amended rather than defended.],
))
