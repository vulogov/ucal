// Life, the Universe, and God — master file.
//
// Compile with:
//   typst compile Documentation/LIFE_UNIVERSE_AND_GOOD/BOOK.typ
//
// Structure follows RFC UCAL-A1 §14 (the eight parts), §16.2 (chapter
// specifications), and §17.2 (the nine readings). Chapters not yet drafted are
// carried as `unwritten()` stubs that state their specification, so the book's
// shape is visible and its length is not quietly understated — the same
// discipline as `UCAL-E0062` in the software it describes.

#import "design.typ": *
#import "chapters/zz-unwritten.typ": unwritten

#book((
  include "chapters/00-preface.typ",

  part(number: "I", title: "Foundations",
    blurb: [Definitional. Nothing later in the book may redefine these, and a
            reader who stops here must not leave with a false impression.]),
  include "chapters/01-intent.typ",
  include "chapters/02-the-tick.typ",
  include "chapters/03-the-zero-of-time.typ",
  include "chapters/04-the-universe-second.typ",

  part(number: "II", title: "What was built",
    blurb: [An engineer may stop at the end of this part and have received a
            complete technical article.]),
  include "chapters/05-the-domain.typ",
  include "chapters/06-notation.typ",
  include "chapters/07-the-bridge.typ",
  include "chapters/08-derived-calendars.typ",

  part(number: "III", title: "What implementation refused",
    blurb: [Design as something that loses arguments.]),
  include "chapters/09-divergence.typ",
  include "chapters/10-the-97-400-correction.typ",

  part(number: "IV", title: "The datum",
    blurb: [Why it cannot be measured, and what was done instead.]),
  unwritten(number: 11, title: "Why it cannot be measured", spec: "§16.2",
    contains: (
      [Rule Q's three reasons: exactness cannot come from measurement; the FLRW
       $t arrow.r 0$ limit is not an observable event; the extrapolation is
       model-dependent.],
      [Kant's fourth, which the specification does not state: the question
       presupposes a completed totality that is not a possible object of
       knowledge.],
      [The datum in ordinary company — TAI 1958, the Julian Day epoch, Unix —
       and the exact parallel to the SI second.],
    )),
  unwritten(number: 12, title: "What was done instead", spec: "§16.2",
    contains: (
      [`BIG_BANG_CLAIM` as a `SignedWindow` with no arithmetic implementations,
       `UCAL-E0025`, and the compile-fail test proving the type cannot reach an
       operand position — *the book's central exhibit*.],
      [`datum_provenance` as machine-readable, re-executable data, with its
       $-0.017 space 190 space 364$ s rounding residual.],
      [Kant's constitutive/regulative distinction: Kant policed the boundary
       with philosophical discipline; the crate polices it with a type.],
      [UC-Θ as unbuilt — the profile in which the datum is the beginning of time
       at organization, and what it would cost.],
    )),
  unwritten(number: 13, title: "Florensky's radius", spec: "§16.2",
    contains: (
      [*Мнимости в геометрии* (1922) as the closest precedent and the clearest
       cautionary tale in one text.],
      [The Dante geometry as genuine achievement; then §9, where a formal
       artifact is read as a physical place.],
      [The distance between the two is exactly one move — and Florensky had no
       rule against the second.],
      [Basil, *Hexaemeron* I.6, and the historical cost: persecution from 1922
       to execution in 1937.],
    )),

  part(number: "V", title: "Any celestial body",
    blurb: [How far the approach generalises, and precisely where it fails.]),
  unwritten(number: 14, title: "Universal baselines", spec: "§16.2",
    contains: (
      [Nothing in the arithmetic references a rotation, an orbit, or a civil
       calendar.],
      [One derivation mechanism — every calendar is (Body, Anchor, Cycles,
       LeapRule) — with Earth as an ordinary instance and no crate named after a
       body.],
    )),
  unwritten(number: 15, title: "Deriving a calendar", spec: "§16.2",
    contains: (
      [Earth: convergents 1/4, 7/29, 8/33, 31/128 — the Julian rule as convergent
       1, the Gregorian absent.],
      [Mars: 668.59 sols per year, convergent 16/27.],
      [Titan: tidally locked, handled with no special case — which is the point.],
      [The Metonic cycle 235/19 derived from Earth's periods unaided.],
    )),
  unwritten(number: 16, title: "Where it breaks", spec: "§16.2 — no shorter than ch. 15",
    contains: (
      [The anchor is empirical and cannot be derived.],
      [A body with no qualifying satellite has no month — Mars is the worked
       case, and the absence is the correct output.],
      [A rogue planet has no year.],
      [A tidally locked body's day equals its orbit.],
      [Relativistic environments are out of scope.],
      [Body parameters carry secular rates and validity windows and are wrong
       outside them — which matters most at exactly the deep-time scale this
       project targets.],
    )),
  unwritten(number: 17, title: "The one qualification", spec: "§16.2 — one page",
    contains: (
      [For a single planet this is a curiosity. For timekeeping across two or
       more bodies, a universal ladder with local overlays may be the only
       coherent arrangement.],
      [A claim about coherence, not a recommendation for adoption.],
    )),

  part(number: "VI", title: "Nine readings",
    blurb: [The traditions as readers of the artifact — neither validating it nor
            validated by it. Every chapter names its conflict.]),
  unwritten(number: 18, title: "Greek", spec: "§17.2 — A1",
    contains: (
      [Archimedes' *Sand Reckoner* builds a positional hierarchy to make a cosmic
       magnitude expressible, arriving at $tilde 10^63$.],
      [Euclid X.2's anthyphairesis *is* the intercalation algorithm.],
      [*Conflict:* Aristotle holds time a continuum and the instant a limit; the
       instrument is discrete. And *Physics* IV.14 makes the datum's objectivity
       depend on there being a counter.],
    )),
  unwritten(number: 19, title: "Jewish", spec: "§17.2 — A2",
    contains: (
      [*Molad tohu* — "the new moon of chaos" — a stipulated epoch placed before
       the event it anchors, nineteen centuries early.],
      [The *ḥelek* chosen for divisibility, exactly as `SECOND` was.],
      [*Conflict:* the compressed Persian period is provenance overruled by
       doctrine, and the crate's re-executable chain would expose it — so the
       instrument judges a tradition, which the book's own rules forbid the
       author from doing.],
    )),
  unwritten(number: 20, title: "Islamic", spec: "§17.2 — A3",
    contains: (
      [Ghazālī's *ʿāda* — divine custom yielding practical certainty without
       necessity — is what a validity window encodes.],
      [*Conflict:* under Premise 6 an orbital period is a habit, so a guaranteed
       drift bound over 400,000 years is meaningless. The crate's ontology is
       Rushdian and it took a side without arguing for it.],
    )),
  unwritten(number: 21, title: "Patristic and Latin", spec: "§17.2 — A4",
    contains: (
      [Basil's road-and-house image is Rule Q's content exactly.],
      [Augustine's answer is `UCAL-E0020`: an error, not a negative number.],
      [*Conflict:* *ex nihilo* is held firmly, so UC-Θ is heterodox by this
       standard — and the book must say so rather than soften it.],
    )),
  unwritten(number: 22, title: "Orthodox", spec: "§17.2 — A5",
    contains: (
      [Gregory of Nyssa's διάστημα makes a calendar an instrument of the created
       order by definition — the limitation is the doctrine, not a defect.],
      [Palamas' essence/energies distinction as the strongest licence the
       thesis receives from any tradition.],
      [*Conflict:* Rule N sides with the 1913 Synodal position against Florensky
       and Losev — the two thinkers this book most depends on.],
    )),
  unwritten(number: 23, title: "Latter-day Saint", spec: "§17.2 — A6",
    contains: (
      [D&C 130:4–5 states Rule K's thesis in 1843: reckoning is body-relative.],
      [Abraham 3:4 is a declared bridge constant with a profile tag on each side.],
      [*Conflict:* Kolob is a governing *body*, where Rule K.5 permits only an
       abstract ladder — the privilege is located differently.],
    )),
  unwritten(number: 24, title: "Modern philosophy", spec: "§17.2 — A7",
    contains: (
      [The First Antinomy supplies a fourth reason for Rule Q the specification
       does not state.],
      [Constitutive versus regulative, enforced by a type with no arithmetic:
       Kant policed the boundary by discipline, the crate by the compiler.],
      [*Conflict:* Kant's number is the schema of magnitude — monadic — which
       contradicts the eidetic number of A1 and A8, and Rule G asserts both
       readings of one integer.],
    )),
  unwritten(number: 25, title: "Russian", spec: "§17.2 — A8",
    contains: (
      [Fyodorov makes total addressability of the past an obligation.],
      [Vernadsky's process-relative times and Chizhevsky's solar-period dating
       as Rule K's unacknowledged ancestry, a century early.],
      [*Conflict:* Bugaev proposes discreteness *as worldview*, which the book
       declines — from the closest kin. And Losev's имя makes the name
       constitutive where Rule N makes it a locale-table entry.],
    )),
  unwritten(number: 26, title: "Method", spec: "§17.2 — A9",
    contains: (
      [Klein on *arithmos*; Sorabji on time, creation, and the continuum.],
      [*Conflict:* the method makes the code the invariant and the traditions the
       variables. That is itself a metaphysical choice, made without argument —
       and disclosing it is the last and most self-referential application of
       Rule M.],
    )),

  part(number: "VII", title: "The instrument as research tool",
    blurb: [What a program can do that an argument cannot: it can be run against
            material its author did not choose, and return an answer he did not
            want.]),
  unwritten(number: 27, title: "Six samples", spec: "§18",
    contains: (
      [S1 — comparative chronology on one axis: Seder Olam, Byzantine, Ussher,
       D&C 77, each a declared profile.],
      [S2 — calendar audit by convergent. Julian *is* convergent 1; Gregorian is
       not a convergent at any depth; Jalali 8/33 *is* convergent 3.],
      [S3 — uncertainty audit: cited versus stipulated.],
      [S4 — cross-body simultaneity: the demonstration that "now" is not a shared
       object.],
      [S5 — measuring διάστημα with no Earth content in the units.],
      [S6 — a revealed ratio evaluated as a bridge constant, refused as an anchor.],
    )),
  unwritten(number: 28, title: "Null results", spec: "§16.2",
    contains: (
      [Each sample's negative finding, stated as plainly as its positive one.],
      [Where a sample produced nothing, say so; where a result was weaker than
       hoped, say how much.],
      [This chapter is what distinguishes the book from a demonstration.],
    )),

  part(number: "VIII", title: "The claim"),
  unwritten(number: 29, title: "Why a program", spec: "§16.2",
    contains: (
      [An essay can assert that a distinction ought to be respected; a type system
       can make violating it fail to build.],
      [Why prose cannot be *run* by someone who disagrees.],
    )),
  unwritten(number: 30, title: "Uselessness restated", spec: "§16.2",
    contains: (
      [The thesis, not an apology. Part V's qualification restated as a limit
       rather than a rescue.],
    )),
  unwritten(number: 31, title: "What is not claimed", spec: "§16.2",
    contains: ([The negative inventory, assembled from Appendix C.],)),
  unwritten(number: 32, title: "Kant's moon", spec: "§16.2",
    contains: (
      [Transcendental illusion is not error but a natural appearance that
       persists after diagnosis — the astronomer knows the moon is not larger at
       the horizon and still sees it that way.],
      [Every reader who sees a 61-digit integer will read it as a fact about
       being, and no rule prevents that.],
      [`UCAL-E0025` does not cure the illusion; it refuses to compute with it,
       which is the only thing a specification can do.],
    )),

  include "chapters/99-about-the-author.typ",
))
