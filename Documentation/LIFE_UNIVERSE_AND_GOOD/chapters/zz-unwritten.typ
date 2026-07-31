#import "../design.typ": *

// Placeholder for chapters specified in RFC UCAL-A1 §16.2 and §17.2 but not yet
// drafted. It exists so that the book's structure is visible in the contents and
// the page count is honest, rather than the unwritten parts being invisible.
//
// This is the same discipline as `UCAL-E0062`: an absent thing is reported, not
// defaulted away.

#let unwritten(number: 0, title: "", spec: "", contains: ()) = {
  chapter(number: number, title: title)
  callout(label: "Not yet drafted")[
    This chapter is specified in RFC UCAL-A1 #spec and has not been written. Its
    outline is below so that the book's shape is visible and its length is not
    quietly understated.
  ]
  if contains.len() > 0 {
    subsection("Specified content")
    list(..contains.map(x => [#x]))
  }
}
