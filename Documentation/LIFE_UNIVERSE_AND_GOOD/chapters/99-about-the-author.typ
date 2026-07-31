#import "../design.typ": *

#pagebreak(weak: true, to: "odd")

#hide(heading(level: 1, numbering: none, outlined: true, bookmarked: true,
  "About the Author"))

#v(2cm)
#align(left)[
  #text(font: body_family, size: 9pt, tracking: 2pt, fill: ink_gray, upper("Afterword"))
  #v(4mm)
  #text(font: body_family, size: 36pt, weight: "regular", fill: ink_black,
    "About the author")
]
#v(1cm)
#line(length: 100%, stroke: 0.5pt + ink_rule)
#v(12mm)

Vladimir Ulogov has spent decades building infrastructure for distributed systems — the
kind of software that watches other software. Early in his career he worked on monitoring
and telemetry platforms; later years took him into federated observability, telemetry
buses, and the architecture of systems that have to make sense of millions of data points
without losing the thread.

Observability, in the end, is a discipline of *coherence* — of never reporting a state the
system cannot account for, of insisting that every signal follow from something real. It
is not an accident that a calendar built to point at an origin it cannot measure carries
the same instinct: it declares what it knows, declares what it has merely stipulated, and
refuses to let the second quietly become the first.

What makes him slightly unusual in his corner of the industry is a tendency to write his
own tools — not small utilities, but programming languages. The Bund language (its
compiler, its VM, its document store, its parser) lives in a long series of Rust crates on
crates.io. `rust_dynamic`, `rust_multistackvm`, `bundcore` — each is a building block that
exists because the off-the-shelf options didn't fit the shape of the work. `ucal` grew the
same way, from an irritation about units that no existing library was going to fix, because
no existing library thought it was a problem.

#section("A work of love")

`ucal` is open source, under a permissive licence — you can read it, fork it, study it,
modify it, and pass it on. It carries no analytics and no telemetry. It was not built to be
sold and it was not built to be adopted; it was built because the question would not go
away, and because the only way to find out whether a distinction can be enforced by a
compiler is to try to enforce one.

#section("A note on cooperation")

Vladimir believes firmly in the human capacity for mutual help — that we make better work,
and live better lives, when we share what we know and what we build. Open source is one of
the most concrete expressions of cooperation our era has produced: code read, improved, and
passed forward without payment, without permission, by people who will never meet.

This book is offered in the same spirit. If it helps you see one distinction more clearly —
between what an instrument measures and what it merely points at — that is enough.

#section("Where to find more")

/ *GitHub*: `@vulogov` — the source for `ucal`, Bund, and the dozen-plus Rust crates that carry the infrastructure. Issues and pull requests welcome.
/ *LinkedIn*: `/in/vladimirulogov` — posts on observability, the occasional long-form essay.
/ *YouTube*: `@vulogov` — talks and walkthroughs from the conference trail.

#v(8mm)

#text(font: body_family, style: "italic", size: 11pt, fill: ink_gray,
  "If you find a claim in this book that the source tree does not support, open an issue. That is the only kind of correction the book is built to receive, and it is the kind it most wants."
)

#v(2cm)
#align(center, text(font: body_family, size: 8pt, fill: ink_faint, tracking: 4pt,
  upper("end of the book")))
