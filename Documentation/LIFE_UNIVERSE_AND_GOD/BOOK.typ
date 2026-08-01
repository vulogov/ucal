// Life, the Universe, and God — master file.
//
// Compile with:
//   typst compile Documentation/LIFE_UNIVERSE_AND_GOD/BOOK.typ
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
  include "chapters/11-why-it-cannot-be-measured.typ",
  include "chapters/12-what-was-done-instead.typ",
  include "chapters/13-florenskys-radius.typ",

  part(number: "V", title: "Any celestial body",
    blurb: [How far the approach generalises, and precisely where it fails.]),
  include "chapters/14-universal-baselines.typ",
  include "chapters/15-deriving-a-calendar.typ",
  include "chapters/16-where-it-breaks.typ",
  include "chapters/17-the-one-qualification.typ",

  part(number: "VI", title: "Nine readings",
    blurb: [The traditions as readers of the artifact — neither validating it nor
            validated by it. Every chapter names its conflict.]),
  include "chapters/18-greek.typ",
  include "chapters/19-jewish.typ",
  include "chapters/20-islamic.typ",
  include "chapters/21-patristic-and-latin.typ",
  include "chapters/22-orthodox.typ",
  include "chapters/23-latter-day-saint.typ",
  include "chapters/24-modern-philosophy.typ",
  include "chapters/25-russian.typ",
  include "chapters/26-method.typ",

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
