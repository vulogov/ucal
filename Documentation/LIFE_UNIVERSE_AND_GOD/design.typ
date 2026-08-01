// Life, the Universe, and God — design tokens and page chrome.
//
// PAGE-BREAK POLICY. Every framed block is `breakable: false`: term, callout,
// claim, conflict, terminal, recap, and raw code blocks. A framed block that
// splits leaves its label at the foot of one page and its content on the next,
// which reads as two things rather than one — and for `claim` and `conflict`
// the label is the whole point, since it marks where the book stops asserting
// fact. The cost is slack at the foot of a page when a block will not fit; that
// is the right trade for a book whose apparatus carries meaning.
//
// Figures are wrapped in `block(breakable: false)` at each call site, so a
// caption cannot orphan from its figure.
//
// SLOT-VOICE is filled from the companion volumes (blackInkhaven/Book/POETRY):
// the same measure-and-report discipline, the same warm paper and cool ink,
// pointed now at an instrument rather than at verse. Where this book departs
// from its companions it is for one reason only — Rule M, which requires every
// interpretive claim to be marked, and which is therefore typographic here
// rather than merely editorial.

#import "@preview/cetz:0.4.2"
#import "slots.typ": *

#let book_title    = "Life, the Universe, and God"
#let book_subtitle = "A Software Engineer's Instrument for the Immeasurable"
#let book_author   = SLOT_AUTHOR.name
#let book_year     = "2026"
#let running_title = "An Instrument for the Immeasurable"

// ── Palette — warm paper, cool ink, restrained accents ──────────────
#let ink_black   = rgb("#1a1a1a")
#let ink_gray    = rgb("#5d5d5d")
#let ink_faint   = rgb("#9a9a9a")
#let ink_rule    = rgb("#c6c0b5")
#let ink_accent  = rgb("#7a4a2f")            // burnt sienna — chapter numbers
#let ink_smoke   = rgb("#7d736a")            // muted brown — cover eyebrow
#let ink_paper   = rgb("#fdfaf3")            // warm cream — cover ground
#let ink_term    = rgb("#2f5d7a")            // slate blue — term definitions
#let ink_code_bg = rgb("#f3eee4")
#let ink_call_bg = rgb("#f6f1e6")
#let ink_term_bg = rgb("#eef3f7")
#let ink_recap   = rgb("#3f6b4a")
#let ink_recap_bg = rgb("#e9f3ea")

// Rule M's colours. Interpretation and resonance are the two kinds of claim the
// book is not entitled to assert, so they get the two marked grounds. Conflict
// is Rule C's obligation and earns a third.
#let ink_interp    = rgb("#5b4a7a")          // muted violet — interpretation
#let ink_interp_bg = rgb("#f1eef6")
#let ink_reson     = rgb("#7a6a2f")          // olive — resonance
#let ink_reson_bg  = rgb("#f5f2e6")
#let ink_conflict  = rgb("#8a3a3a")          // oxide red — conflict
#let ink_conflict_bg = rgb("#f8eded")

#let body_family = ("Libertinus Serif", "New Computer Modern")
#let mono_family = ("DejaVu Sans Mono",)

#let book_page = (
  paper: "iso-b5",
  margin: (inside: 26mm, outside: 20mm, top: 27mm, bottom: 24mm),
  numbering: "1",
)


// ── The mark, drawn at a given elapsed fraction ─────────────────────
//
// The identity's sector is elapsed time (see Documentation/logo/README.md). A
// part divider that draws it at the fraction of the book already read is the
// mark measuring the reading — which is what the mark is for. Redrawn here
// rather than imported because the asset's sector is fixed and this one is not.
#let dial(elapsed: 0.0, size: 16mm) = {
  let n = 40
  cetz.canvas(length: size / 2, {
    import cetz.draw: *
    circle((0, 0), radius: 1.0, stroke: 0.9pt + ink_black)
    for i in range(0, 5) {
      let a = 90deg - i * 72deg
      line((calc.cos(a) * 0.84, calc.sin(a) * 0.84),
           (calc.cos(a) * 1.0,  calc.sin(a) * 1.0), stroke: 0.9pt + ink_black)
    }
    if elapsed > 0.001 {
      arc((0, 0), start: 90deg, stop: 90deg - elapsed * 360deg, radius: 0.78,
        anchor: "origin", mode: "PIE", fill: ink_black, stroke: none)
    }
    circle((0, 0), radius: 0.13, fill: white, stroke: 0.8pt + ink_black)
    line((-0.07, 1.06), (0.07, 1.06), (0, 1.2), close: true, fill: ink_black,
      stroke: none)
  })
}

// ── Part divider ────────────────────────────────────────────────────
#let part(number: "I", title: "", blurb: none, elapsed: 0.0) = {
  pagebreak(weak: true)
  hide(heading(level: 1, numbering: none, outlined: true, bookmarked: true,
    [Part #number — #title]))
  v(5.4cm)
  align(center)[
    #dial(elapsed: elapsed, size: 17mm)
    #v(9mm)
    #text(font: body_family, size: 11pt, tracking: 3pt, fill: ink_gray,
      upper("Part " + number))
    #v(6mm)
    #line(length: 36%, stroke: 0.5pt + ink_rule)
    #v(6mm)
    #text(font: body_family, size: 26pt, weight: "bold", fill: ink_black, title)
    #if blurb != none {
      v(8mm)
      block(width: 66%)[
        #set par(justify: false)
        #text(font: body_family, size: 10.5pt, style: "italic", fill: ink_gray, blurb)
      ]
    }
  ]
}

// ── Chapter opening ─────────────────────────────────────────────────
#let chapter(number: 0, title: "") = {
  pagebreak(weak: true)
  hide(heading(level: 1, numbering: none, outlined: true, bookmarked: true,
    [#str(number) — #title]))
  v(1.6cm)
  align(left)[
    #text(font: body_family, size: 9pt, tracking: 2pt, fill: ink_gray,
      upper("Chapter " + str(number)))
    #v(1mm)
    #text(font: body_family, size: 84pt, weight: "bold", fill: ink_accent, str(number))
    #v(-6mm)
    #text(font: body_family, size: 25pt, weight: "regular", fill: ink_black, title)
  ]
  v(1cm)
  line(length: 100%, stroke: 0.5pt + ink_rule)
  v(8mm)
}

#let appendix(letter: "A", title: "") = {
  pagebreak(weak: true)
  hide(heading(level: 1, numbering: none, outlined: true, bookmarked: true,
    [Appendix #letter — #title]))
  v(1.6cm)
  align(left)[
    #text(font: body_family, size: 9pt, tracking: 2pt, fill: ink_gray,
      upper("Appendix " + letter))
    #v(1mm)
    #text(font: body_family, size: 84pt, weight: "bold", fill: ink_accent, letter)
    #v(-6mm)
    #text(font: body_family, size: 25pt, weight: "regular", fill: ink_black, title)
  ]
  v(1cm)
  line(length: 100%, stroke: 0.5pt + ink_rule)
  v(8mm)
}

#let section(title) = {
  hide(heading(level: 2, numbering: none, outlined: true, title))
  block(sticky: true, above: 8mm, below: 3.2mm,
    text(font: body_family, size: 15pt, weight: "bold", fill: ink_black, title))
}

#let subsection(title) = {
  block(sticky: true, above: 4.5mm, below: 2mm,
    text(font: body_family, size: 11.5pt, weight: "bold", fill: ink_black, title))
}

// ── Term box ────────────────────────────────────────────────────────
#let term(name, body) = {
  v(2mm)
  block(fill: ink_term_bg, stroke: (left: 2pt + ink_term),
    inset: (left: 9pt, right: 9pt, top: 7pt, bottom: 7pt),
    width: 100%, radius: 1pt, breakable: false, {
      text(font: body_family, size: 8pt, weight: "bold", fill: ink_term,
        tracking: 1pt, "TERM")
      h(6pt)
      text(font: body_family, size: 11pt, weight: "bold", fill: ink_term, name)
      v(2mm)
      body
    })
  v(2mm)
}

#let callout(label: "Note", body) = {
  v(2mm)
  block(fill: ink_call_bg, stroke: (left: 2pt + ink_accent),
    inset: (left: 9pt, right: 9pt, top: 7pt, bottom: 7pt),
    width: 100%, radius: 1pt, breakable: false, {
      text(font: body_family, size: 8pt, weight: "bold", fill: ink_accent,
        tracking: 1.5pt, upper(label))
      v(2mm)
      body
    })
  v(2mm)
}

// ── Rule M, made typographic ────────────────────────────────────────
//
// The book marks where it stops asserting fact, exactly as the software marks
// where it stops computing. `#claim("code")` and `#claim("history")` pass
// through unmarked — they are checkable. The other three are not, and they are
// ruled off so a reader can see the boundary without being told about it.
//
// kind ∈ ("code", "history", "tradition", "interpretation", "resonance")
#let claim(kind, body) = {
  let marked = (
    interpretation: (ink_interp, ink_interp_bg, "INTERPRETATION"),
    resonance:      (ink_reson,  ink_reson_bg,  "RESONANCE"),
  )
  if kind in marked {
    let (fg, bg, tag) = marked.at(kind)
    v(2mm)
    block(fill: bg, stroke: (left: 2pt + fg),
      inset: (left: 9pt, right: 9pt, top: 7pt, bottom: 7pt),
      // Not breakable. A marked block split across a page leaves its tag
      // stranded at the foot of one page and its argument on the next, which
      // is exactly the boundary Rule M exists to make visible.
      width: 100%, radius: 1pt, breakable: false, {
        text(font: body_family, size: 8pt, weight: "bold", fill: fg,
          tracking: 1.5pt, tag)
        v(2mm)
        body
      })
    v(2mm)
  } else if kind == "tradition" {
    // A tradition's own position, reported. Not the book's claim, but not
    // interpretation either — so a rule, not a ground.
    v(1.5mm)
    block(inset: (left: 10pt), stroke: (left: 1pt + ink_faint), body)
    v(1.5mm)
  } else {
    body
  }
}

// ── Rule C — every Part VI chapter names its conflict ───────────────
#let conflict(body) = {
  v(3mm)
  block(fill: ink_conflict_bg, stroke: (left: 3pt + ink_conflict),
    inset: (left: 10pt, right: 10pt, top: 8pt, bottom: 8pt),
    width: 100%, radius: 1pt, breakable: false, {
      text(font: body_family, size: 8.5pt, weight: "bold", fill: ink_conflict,
        tracking: 1.5pt, "THE CONFLICT")
      v(2.5mm)
      body
    })
  v(3mm)
}

// ── Terminal — verbatim CLI output, captured at the pinned commit ───
#let terminal(caption: "", body) = {
  v(2mm)
  block(breakable: false, width: 100%, {
    block(fill: ink_smoke, inset: (left: 8pt, right: 8pt, top: 3pt, bottom: 3pt),
      width: 100%, radius: (top-left: 2pt, top-right: 2pt), {
        text(font: mono_family, size: 8pt, fill: ink_paper, "● ● ●")
        h(6pt)
        text(font: body_family, size: 8.5pt, style: "italic", fill: ink_paper, caption)
      })
    block(fill: ink_code_bg, stroke: 0.5pt + ink_rule, inset: 8pt, width: 100%,
      radius: (bottom-left: 2pt, bottom-right: 2pt),
      text(font: mono_family, size: 8pt, body))
  })
  v(2mm)
}

#let recap(items) = {
  v(7mm)
  block(fill: ink_recap_bg, stroke: (left: 2pt + ink_recap),
    inset: (left: 9pt, right: 9pt, top: 8pt, bottom: 8pt),
    width: 100%, radius: 1pt, breakable: false, {
      text(font: body_family, size: 9pt, weight: "bold", fill: ink_recap,
        tracking: 1.5pt, "WHAT THIS CHAPTER ESTABLISHED")
      v(2mm)
      list(..items)
    })
}

#let figcap(n, body) = align(center, text(font: body_family, size: 9pt,
  fill: ink_gray, style: "italic", [Figure #n — #body]))

// ── Master document wrapper ─────────────────────────────────────────
#let book(pages, frontispiece: none) = {
  assert-slots()

  set document(title: book_title, author: book_author)
  set text(font: body_family, size: 11pt, fill: ink_black, lang: "en")
  set par(leading: 0.72em, justify: true, first-line-indent: 1em)

  show raw.where(block: true): it => block(
    fill: ink_code_bg, stroke: 0.5pt + ink_rule, inset: 7pt, radius: 2pt,
    width: 100%, breakable: false,
    text(font: mono_family, size: 8.5pt, it))
  show raw.where(block: false): it => box(
    fill: ink_code_bg, inset: (x: 2pt, y: 0pt), outset: (y: 1.5pt), radius: 1pt,
    text(font: mono_family, size: 9.5pt, it))

  // ── Cover ──────────────────────────────────────────────────────────
  set page(paper: book_page.paper, margin: 0pt, numbering: none, header: none,
    fill: ink_paper)
  block(width: 100%, height: 100%)[
    #place(top + left, dx: 12mm, dy: 12mm,
      rect(width: 100% - 24mm, height: 100% - 24mm, stroke: 1pt + ink_accent))
    #place(top + left, dx: 14mm, dy: 14mm,
      rect(width: 100% - 28mm, height: 100% - 28mm, stroke: 0.4pt + ink_accent))
    // The mark, in place of the companions' ornament row. The instrument
    // announces the book about the instrument.
    #place(top + center, dy: 30mm, image(SLOT_LOGO_DIR + "/ucal-mark.svg", width: 26mm))
    #place(top + center, dy: 68mm, block(width: 74%)[
      #set par(justify: false)
      #align(center)[
        #text(font: body_family, size: 11pt, tracking: 4pt, fill: ink_smoke,
          upper("Counting from the first tick"))
        #v(10mm)
        #text(font: body_family, size: 30pt, weight: "bold", fill: ink_black,
          book_title)
        #v(6mm)
        #line(length: 55%, stroke: 0.6pt + ink_accent)
        #v(6mm)
        #text(font: body_family, size: 12.5pt, style: "italic", fill: ink_smoke,
          book_subtitle)
      ]
    ])
    #place(bottom + center, dy: -30mm, align(center)[
      #text(font: body_family, size: 10pt, fill: ink_smoke, book_author)
      #v(2mm)
      #text(font: body_family, size: 9pt, fill: ink_smoke,
        book_year + " · built against the ucal source tree")
    ])
  ]
  pagebreak()

  set page(margin: book_page.margin, fill: white)

  // Frontispiece — a plate facing the contents, no caption, no folio.
  //
  // Filled with a dial whose centre is a clean circular void — the identity's
  // knocked-out core at plate scale. The earlier dial, whose centre is torn
  // open to show clockwork, went to chapter 29 instead: it asserts the opposite
  // of what ch. 4 says the mark means, which is the wrong thing to put on the
  // first page a reader sees.
  if frontispiece != none {
    set page(numbering: none, header: none)
    // A leading v(1fr) collapses at the top of a page, so centre with a
    // full-height block instead.
    block(width: 100%, height: 100%,
      align(center + horizon, image(frontispiece, width: 82%)))
    pagebreak()
  }

  text(font: body_family, size: 22pt, weight: "bold", fill: ink_black, "Contents")
  v(7mm)
  outline(title: none, indent: auto, depth: 2)
  pagebreak()

  set page(
    numbering: "1",
    number-align: center,
    header: context {
      if counter(page).get().first() > 1 {
        align(center, text(font: body_family, size: 8pt, fill: ink_faint,
          tracking: 1.5pt, upper(running_title)))
      }
    },
  )
  counter(page).update(1)
  for p in pages [ #p ]
}
