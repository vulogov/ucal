#import "../design.typ": *

#chapter(number: 19, title: "Jewish")

This chapter contains the hardest sentence in the book, and it arrives in the conflict
section. Everything before it is preparation for being able to write it honestly.

#section("What the direction holds")

#claim("tradition")[
  The Hebrew calendar's computational epoch is *molad tohu* — "the new moon of chaos" —
  fixed by the mnemonic BaHaRaD: day 2, hour 5, 204 *ḥalakim*. By rabbinic reckoning it
  falls roughly a year *before* the creation it anchors, because the calendar's
  arithmetic needs a molad to count from and the first actual molad is not it.

  Time is subdivided by the *ḥelek*: 1/1080 of an hour, about 3⅓ seconds. The number
  1080 was not chosen for convenience of size. It was chosen because it divides — by 2,
  3, 4, 5, 6, 8, 9, 10, 12 and more — so that the fractions the calendar needs come out
  whole.

  Intercalation follows a nineteen-year cycle: seven leap years in nineteen, fixing the
  lunar year against the solar.

  And chronology is not held to be independent of interpretation. *Seder Olam Rabbah*,
  the second-century chronology that underlies the traditional Anno Mundi count,
  derives its dates from scripture read as a coherent whole.
]

#section("Which rule it illuminates")

Rule Q, on stipulated epochs. Rule Q.4, on provenance as data. D-3, on divisibility.
And Appendix I.2, on the Metonic cycle.

#section("The convergences")

#subsection("Molad tohu is Rule Q, nineteen centuries early")

This is the closest precedent for the datum that this book has found anywhere.

The epoch is *computational*. It is placed before the event it anchors. It is not
claimed as an observation. And — the part that no modern epoch matches — it is
*named for its own emptiness*. Tohu is the word from Genesis 1:2, formlessness, the
void before ordering.

#claim("interpretation")[
  Compare the honesty of the two namings.

  This project calls its epoch "the datum" and then spends four chapters and a type
  system explaining that it is stipulated rather than observed. The Hebrew calendar
  calls its epoch *the new moon of chaos*, and the explanation is in the name.

  Whoever fixed BaHaRaD understood exactly what they were doing: an exact arithmetic
  needs an exact origin, the origin cannot be the thing itself, so you place a
  computational fiction at a convenient distance and you say in the name that it is
  one.

  Rule Q's entire content is in that phrase. The project did not know this when the
  rule was written.
]

#subsection("The ḥelek and SECOND were chosen the same way")

Chapter 7 noted that `SECOND` is a multiple of $10^30$, and that the choice makes whole
seconds land with thirty trailing base-5 zeros and whole nanoseconds with twenty-one.

The reason is divisibility: pick a constant with many factors and the subdivisions you
need come out exact.

The *ḥelek* is the same move with the same reasoning, made around the ninth century. A
thousand and eighty parts to the hour, because 1080 has a lot of divisors and lunar
arithmetic needs thirds and fifths and eighths of an hour to come out whole.

Both are a designer looking at a unit and asking not *how big should this be* but *what
must it divide by*.

#subsection("The nineteen-year cycle is convergent 6")

Chapter 15 showed the mechanism deriving Earth's grouping cycles and producing 235/19 —
the Metonic cycle — as the sixth convergent, from Earth's two periods with nothing else
supplied.

The Hebrew calendar's nineteen-year intercalation is that cycle. So the mechanism
recovers, unaided, the structure this tradition has used for well over a millennium.

#claim("interpretation")[
  As with Meton in chapter 15, this is not anticipation. 235/19 is where the good
  approximation is, and anyone with an accurate ratio and a motive lands near it.

  What it does establish is that the derived/declared distinction of chapter 8 cuts
  *for* this tradition on this point. The nineteen-year cycle is derivable. The
  Gregorian leap rule is not. Whatever else is true, the intercalation here is the kind
  of thing the mechanism finds rather than the kind it has to be told.
]

#subsection("Two epochs, and why Rule P exists")

The Anno Mundi count is text-dependent. The Masoretic text yields a creation around
3761 BCE. The Septuagint's longer genealogies yield the Byzantine reckoning, 5509 BCE.
A gap of 1,748 years, from the same procedure applied to different manuscript
traditions.

Those are two profiles in this system's exact sense: same mechanism, different declared
constants, and values from one are not comparable to values from the other.

Rule P exists for precisely this — profiles named, versioned, type-bound, and tagged in
every serialised form, so that a timestamp from one cannot be silently compared with a
timestamp from the other. Failure mode F1 is what happens when they are.

#conflict[
  *The instrument judges a tradition, and the author may not.*

  *Seder Olam*'s chronology compresses the Persian period. Standard historical
  reconstruction gives roughly 207 years from the fall of Babylon to Alexander;
  *Seder Olam* gives roughly 52. Something on the order of 165 years is absent, and
  the compression is not random — it aligns the timeline with the seventy weeks of
  Daniel 9.

  That is provenance overruled by doctrine. And this system's provenance chain is
  *machine-readable and re-executable*: the whole point of Rule Q.4 is that you can
  run the chain and see whether the constant follows from the cited inputs.

  Encode *Seder Olam* as a profile and the chain would not close. The instrument would
  report, mechanically and without comment, that the declared epoch does not follow
  from the declared sources.

  So the artifact is capable of producing a finding about a religious tradition's
  chronology — and this book's own rules forbid its author from drawing the conclusion
  that finding invites.
]

#section("What it changes")

The line this chapter has to hold is: *the instrument may expose; the author may not
judge.* It sounds like evasion. Here is why it is not.

#claim("interpretation")[
  Start with what the instrument would actually report. Not "this chronology is
  false." It would report that a particular arithmetic does not close — that given
  these declared inputs and this stated procedure, you do not arrive at this stated
  output. That is a narrow, checkable, and entirely mechanical claim.

  Everything interesting lies in what follows from it, and none of that is mechanical.
  Whether *Seder Olam* was doing chronology in the sense a modern historian means.
  Whether a second-century text that derives dates from scripture read whole is
  answerable to the standards of a discipline that did not exist. Whether the
  compression is an error, a deliberate harmonisation, or a genre distinction the
  question misunderstands.

  The author has views on some of that and they are not evidence. This book's rules
  say no tradition is argued true and none is argued false, and the rule does not get
  suspended when the artifact hands over something that looks like ammunition.

  The discipline is exactly the one chapter 12 built into the type system, applied to
  the author instead of to a program. `BIG_BANG_CLAIM` is fully available — you can
  read it, print it, cite it — and it cannot be used as an operand. The *Seder Olam*
  finding is fully available: it can be computed, reported, and examined. What it
  cannot do is enter as a premise in an argument this book makes about whether a
  tradition is correct.

  There is a real cost, and pretending otherwise would be its own dishonesty. A reader
  who wants to know what the author thinks about the Persian period will not find out.
  That is the price of a rule that also prevents the artifact being used to validate
  the traditions its author finds congenial — and a rule that only binds in the
  uncomfortable direction is not a rule.
]

#callout(label: "Why this is the hardest sentence")[
  Because the alternative is available, respectable, and would make a better chapter.

  A book could reasonably say: here is a mechanism for auditing chronologies, here is
  what it finds, and the finding is what it is. That book would be more useful to some
  readers and more honest-seeming to others.

  It would also have turned the instrument into an apologetic — one pointed away from
  the author's own commitments rather than toward them, which is the direction that
  feels like rigour and is not.
]

#recap((
  [*Molad tohu* — "the new moon of chaos" — is a computational epoch placed before what it anchors and named for its own emptiness. Rule Q's content, nineteen centuries early, and better named.],
  [The *ḥelek* was chosen for divisibility exactly as `SECOND` was: not *how big*, but *what must it divide by*.],
  [The nineteen-year cycle is the Metonic convergent the mechanism derives unaided — so the derived/declared distinction cuts *for* this tradition here.],
  [Masoretic and Byzantine Anno Mundi are two profiles in this system's exact sense, and Rule P exists for that case.],
  [*Conflict:* the re-executable provenance chain would report that *Seder Olam*'s chronology does not close — a finding about a religious tradition that the book's rules forbid its author from concluding from.],
  [*What changes:* the instrument may expose, the author may not judge — the same discipline chapter 12 put in the type system, applied to the author. It has a real cost, and a rule that only binds in the comfortable direction is not a rule.],
))
