// UCAL-A1 Rule L — author-supplied material fails the build when unfilled.
//
// This mirrors `UCAL-E0062` in the software: an absent anchor is an error and
// never a fallback. A book about an instrument that refuses to default its way
// past missing data should not default its way past missing data either.

// SLOT-VOICE — the author's formatting and voice sample.
// Filled from ../../../blackInkhaven/Book/POETRY: the companion volumes'
// design language, adapted in design.typ. Recording the provenance rather than
// the word "filled", so the derivation is checkable.
#let SLOT_VOICE = (
  source: "blackInkhaven/Book/POETRY — design.typ, chapters/00-introduction.typ, chapters/99-about-the-author.typ",
  register: "second person, plain, says the hard thing on the first page",
  chrome: "iso-b5, Libertinus Serif, warm cream ground, burnt-sienna accent",
)

// SLOT-LOGO — identity assets, per Documentation/logo/README.md.
#let SLOT_LOGO_DIR = "assets/logo"

// SLOT-AUTHOR — About the author. Adapted from the companion volume's
// afterword; the biography is the author's own and is not invented here.
#let SLOT_AUTHOR = (
  name: "Vladimir Ulogov",
  source: "blackInkhaven/Book/POETRY — chapters/99-about-the-author.typ",
)

#let assert-slots() = {
  if SLOT_VOICE == none {
    panic("UCAL-A1 / Rule L: SLOT-VOICE unfilled. Supply the voice and format " +
          "sample before typesetting. No default styling is permitted.")
  }
  if SLOT_LOGO_DIR == none {
    panic("UCAL-A1 / Rule L: SLOT-LOGO unfilled. Supply identity asset paths.")
  }
  if SLOT_AUTHOR == none {
    panic("UCAL-A1 / Rule L: SLOT-AUTHOR unfilled. Supply About the Author. " +
          "No biography may be invented.")
  }
}
