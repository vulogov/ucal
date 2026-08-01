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
  include "chapters/00b-the-rules.typ",

  part(number: "I", title: "Foundations", elapsed: 0.0,
    blurb: [Definitional. Nothing later in the book may redefine these, and a
            reader who stops here must not leave with a false impression.]),
  include "chapters/01-intent.typ",
  include "chapters/02-the-tick.typ",
  include "chapters/03-the-zero-of-time.typ",
  include "chapters/04-the-universe-second.typ",

  part(number: "II", title: "What was built", elapsed: 0.13,
    blurb: [An engineer may stop at the end of this part and have received a
            complete technical article.]),
  include "chapters/05-the-domain.typ",
  include "chapters/06-notation.typ",
  include "chapters/07-the-bridge.typ",
  include "chapters/08-derived-calendars.typ",

  part(number: "III", title: "What implementation refused", elapsed: 0.25,
    blurb: [Design as something that loses arguments.]),
  include "chapters/09-divergence.typ",
  include "chapters/10-the-97-400-correction.typ",

  part(number: "IV", title: "The datum", elapsed: 0.31,
    blurb: [Why it cannot be measured, and what was done instead.]),
  include "chapters/11-why-it-cannot-be-measured.typ",
  include "chapters/12-what-was-done-instead.typ",
  include "chapters/13-florenskys-radius.typ",

  part(number: "V", title: "Any celestial body", elapsed: 0.41,
    blurb: [How far the approach generalises, and precisely where it fails.]),
  include "chapters/14-universal-baselines.typ",
  include "chapters/15-deriving-a-calendar.typ",
  include "chapters/16-where-it-breaks.typ",
  include "chapters/17-the-one-qualification.typ",

  part(number: "VI", title: "Nine readings", elapsed: 0.53,
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

  part(number: "VII", title: "The instrument as research tool", elapsed: 0.81,
    blurb: [What a program can do that an argument cannot: it can be run against
            material its author did not choose, and return an answer he did not
            want.]),
  include "chapters/27-six-samples.typ",
  include "chapters/28-null-results.typ",
  part(number: "VIII", title: "The claim", elapsed: 0.88),
  include "chapters/29-why-a-program.typ",
  include "chapters/30-uselessness-restated.typ",
  include "chapters/31-what-is-not-claimed.typ",
  include "chapters/32-kants-moon.typ",
  include "chapters/a-sources.typ",
  include "chapters/b-glossary.typ",
  include "chapters/c-diagnostics.typ",
  include "chapters/d-rules.typ",

  include "chapters/99-about-the-author.typ",
))
