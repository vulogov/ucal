#!/usr/bin/env python3
"""Generate the rule reference — the front-matter card and Appendix D.

The book cites rule letters 123 times across 14 rules, starting in chapter 4,
and until now defined none of them. A reader meeting "Rule N" on page 22 had
nowhere to look.

Both outputs are derived from spec/RULES.md so the book cannot drift from the
normative text. What is hand-maintained here is the one-line gloss for the card
(RULES.md's own subject lines are terse to the point of opacity for someone who
has not read the spec) and the chapter loci, which are an editorial claim.

Writes:
  chapters/00b-the-rules.typ   front matter, one page, the rules the book uses
  chapters/d-rules.typ         Appendix D, all 24 plus the other coded refs

Run after any change to spec/RULES.md.
"""

import pathlib
import re
import subprocess
import sys

BOOK = pathlib.Path(__file__).resolve().parent.parent
ROOT = BOOK.parents[1]
RULES_MD = ROOT / "spec" / "RULES.md"
CH = BOOK / "chapters"

# A reader's gloss, and where the book first leans on it. The gloss is not
# RULES.md's subject line: those are written for someone who has read the spec.
GLOSS = {
    #    letter: (name, card line (<= 58 ch), appendix gloss, chapter)
    "Q": ("the datum is stipulated", "declared, not measured — and not computable with",
          "Tick 0 is declared, not measured or observed. The physical claim about it is recorded separately and cannot be computed with.", "3"),
    "Z": ("time is unsigned", "nothing precedes the datum; earlier is an error",
          "Nothing precedes the datum. A result that would be earlier is an error, not a negative number.", "3"),
    "A": ("the tick is primitive", "everything is counted in ticks; no other unit is basic",
          "Everything is counted in ticks. No other unit is fundamental — not the second, not the beat.", "2"),
    "Y": ("metrology", "foreign units cross one boundary you can point at",
          "Earth units cross one declared boundary, and never appear in the arithmetic.", "7"),
    "F": ("the frame is declared", "what it does not model, it says it does not model",
          "The reference frame is stated rather than assumed. What the system does not model, it says it does not model.", "16"),
    "M": ("order is total and monotone", "of any two instants, exactly one order holds",
          "Of any two instants, exactly one of earlier, same, later holds.", "5"),
    "P": ("profiles are tagged and type-bound", "two profiles' values cannot be compared",
          "Two timestamps from different declared constants cannot be compared, and the text says which is which.", "6"),
    "W": ("one domain across backends", "both integer backends accept the same values",
          "The fixed-width and arbitrary-precision integers enforce one identical domain.", "5"),
    "O": ("overflow is a typed error", "arithmetic never wraps and never saturates",
          "Arithmetic never wraps and never saturates, in release builds as well as debug.", "5"),
    "E": ("integrality", "no floating point anywhere; a lint enforces it",
          "Not in a signature, a field, an intermediate, or the rendering path. A lint enforces it.", "5"),
    "R": ("rounding only on rendering", "values round when displayed, never when built",
          "Values round when displayed, never when constructed, and always under a mode the caller names.", "5"),
    "G": ("the tier grid", "units are powers of five; a timestamp is base 5",
          "Units are 5^(5k) ticks. A timestamp is the tick count written in base 5 and grouped in fives.", "4"),
    "N": ("names are display only", "a tier's identity is its exponent, not its name",
          "A tier's identity is its exponent. Nothing decides behaviour from a name.", "4"),
    "T": ("truncation is uncertainty", "a coarser value is an interval, not a padded point",
          "A value printed to a coarser tier *is* an interval, not a point padded with zeros.", "6"),
    "U": ("interval arithmetic", "operations on windows return windows",
          "Operations on intervals return intervals. A midpoint is a rendering choice, not a measurement.", "5"),
    "D": ("two text forms, one value", "both forms encode one integer, each with an anchor",
          "The human form and the digit form encode the same integer, each declaring its own anchor.", "6"),
    "S": ("sort order", "byte order is chronological order — for binary and UCID",
          "Lexicographic order equals chronological order for the fixed-width forms — and not for text.", "6"),
    "B": ("fixed 64-byte binary", "big-endian, never minimal, so the format never changes",
          "Big-endian, never minimal, so byte order is numeric order and the format never has to change.", "6"),
    "I": ("UCID range and non-uniqueness", "52 characters, sortable, no randomness — not a UUID",
          "52 characters below 2^256, sortable, containing no randomness. Not a UUID.", "6"),
    "L": ("leap seconds at the boundary", "TT is the only pivot; arithmetic never meets one",
          "TT is the only pivot. No arithmetic on absolute time ever meets a leap second.", "7"),
    "K": ("one mechanism; Earth is an instance", "every calendar is built by the same path",
          "Every calendar is built by the same path from a body's own periods. There is no Earth branch.", "8"),
    "J": ("the anchor is declared and required", "phase is supplied per body; absence is an error",
          "Phase cannot be derived from period, so it is supplied per body — and its absence is an error, never a default.", "8"),
    "C": ("body parameters carry provenance", "epoch, rate, window — outside it, a warning",
          "Epoch, secular rate, validity window, and the as-measured value. Outside the window, a warning rather than extrapolation.", "8"),
    "X": ("certified enclosures", "an interval, with its two error sources kept apart",
          "An interval that provably contains the answer, with quadrature error and parameter uncertainty reported separately.", "5"),
}

OTHER = [
    ("N1", "non-goal",
     "The tick is not claimed to be a quantum of time. It is the resolution floor of an instrument.", "2"),
    ("N12", "non-goal",
     "No time before the datum. The value is not representable and the request is refused.", "3"),
    ("F1", "failure mode",
     "Timestamps shifting when the age constant is revised — what Rule P prevents.", "11"),
    ("F9", "failure mode",
     "Earth becoming the template rather than an instance — what Rule K prevents.", "14"),
]

DELTAS = [
    ("D-A4",  "Appendix C's human forms are truncated at T−5, not tick-exact", "6"),
    ("D-A5",  "grouping cycles are declared per body, not admitted by a global bound", "9, 16"),
    ("D-A7",  "full-width encode is 45 divmod steps, not 44", "9"),
    ("D-A8",  "what a printed form means, and how each form is anchored", "6, 9"),
    ("D-A11", "obliquity is an angle and cannot be a rated parameter", "9"),
    ("D-A12", "§9.6's synodic formula computes the wrong quantity", "9, 16"),
    ("D-A13", "a drift bound is a rate in local units, not a duration", "9, 15"),
    ("D-A14", "§10.3's integral is improper and cannot be quadratured as written", "9"),
]

ORDER_CARD = ["Q", "Z", "A", "G", "N", "T", "P", "K", "J", "C", "E", "X",
              "Y", "B", "S", "M", "F"]


def cited_rules() -> set:
    text = "\n".join(f.read_text() for f in sorted(CH.glob("[0-9]*.typ")))
    return set(re.findall(r"Rule ([A-Z])\b", text))


def card(used: set) -> str:
    L = ['#import "../design.typ": *', "",
         "#pagebreak(weak: true)", "", "#v(1cm)",
         "#align(center)[",
         '  #text(font: body_family, size: 20pt, weight: "bold", fill: ink_black,',
         '    "The rules, in one page")', "]", "#v(6mm)", "",
         "The software this book is about is specified by twenty-four rules, each named",
         "by a letter. The book cites them constantly and from chapter 2 onward, so they",
         "are here at the front rather than only in the back.",
         "",
         "You do not need to learn these. Read past them on first encounter and come back",
         "when a chapter leans on one — that is what this page is for.",
         "",
         "#v(3mm)",
         "#block(width: 100%)[", "  #set text(size: 8.5pt)",
         "  #set par(justify: false)", "  #table(",
         "    columns: (auto, 34%, 1fr, auto),",
         "    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },",
         "    inset: (x: 4pt, y: 3.4pt),",
         "    align: (center, left, left, center),",
         "    [*rule*], [*name*], [*what it requires*], [*ch.*],"]
    for r in ORDER_CARD:
        if r not in used:
            continue
        name, short, _long, ch = GLOSS[r]
        L.append(f"    [*{r}*], [{name}], [{short}], [{ch}],")
    L += ["  )", "]", "", "#v(3mm)", "",
          "Ten further rules govern parts of the software the book does not discuss.",
          "Appendix D lists all twenty-four, together with the non-goals, failure modes",
          "and specification corrections cited in the text. The normative statements are",
          "in `spec/RULES.md` in the source tree.", "",
          "#callout(label: \"Why letters\")[",
          "  Because the specification names them that way, and renaming them for the book",
          "  would make every citation in the source tree — there are 538 — resolve to",
          "  nothing.",
          "",
          "  The letters are not mnemonic and the specification does not pretend they are.",
          "  `Q` is the datum rule because it was the seventeenth thing written down.",
          "]", ""]
    return "\n".join(L) + "\n"


def appendix(used: set) -> str:
    md = RULES_MD.read_text() if RULES_MD.exists() else ""
    subjects = dict(re.findall(r"^### Rule ([A-Z]) — (.+?)\s+§", md, re.M))
    L = ['#import "../design.typ": *', "",
         '#appendix(letter: "D", title: "The rules")', "",
         "All twenty-four, with the chapters that use them. Subject lines are taken from",
         "`spec/RULES.md`, which is the normative statement; the glosses are this book's.",
         "",
         "The fourteen the book actually cites are marked. The other ten govern parts of",
         "the software the book does not discuss, and are listed so that the set is",
         "complete rather than curated.", "",
         "#section(\"The twenty-four rules\")", "",
         "#block(width: 100%)[", "  #set text(size: 9pt)", "  #table(",
         "    columns: (auto, auto, 1fr, auto),",
         "    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },",
         "    inset: (x: 5pt, y: 4pt),",
         "    align: (center, left, left, center),",
         "    [*rule*], [*subject*], [*what it requires*], [*ch.*],"]
    for r in sorted(GLOSS):
        name, _short, req, ch = GLOSS[r]
        subj = subjects.get(r, name)
        mark = f"*{r}*" if r in used else f"{r}"
        loc = ch if r in used else "—"
        L.append(f"    [{mark}], [{subj}], [{req}], [{loc}],")
    L += ["  )", "]", "",
          "#section(\"Non-goals and failure modes\")", "",
          "The specification numbers what it refuses (`N`) and what it is built to prevent",
          "(`F`). The book cites four.", "",
          "#block(width: 100%)[", "  #set text(size: 9pt)", "  #table(",
          "    columns: (auto, auto, 1fr, auto),",
          "    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },",
          "    inset: (x: 5pt, y: 4pt),",
          "    [*id*], [*kind*], [*what it says*], [*ch.*],"]
    for code, kind, what, ch in OTHER:
        L.append(f"    [`{code}`], [{kind}], [{what}], [{ch}],")
    L += ["  )", "]", "",
          "#section(\"Specification corrections\")", "",
          "Where verification found the specification wrong, the correction carries a",
          "`D-A` number. Chapter 9 is the account; `spec/SPEC-DELTAS.md` is the record.", "",
          "#block(width: 100%)[", "  #set text(size: 9pt)", "  #table(",
          "    columns: (auto, 1fr, auto),",
          "    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },",
          "    inset: (x: 5pt, y: 4pt),",
          "    [*delta*], [*what changed*], [*ch.*],"]
    for code, what, ch in DELTAS:
        L.append(f"    [`{code}`], [{what}], [{ch}],")
    L += ["  )", "]", "",
          "#callout(label: \"Three the book found in itself\")[",
          "  Chapter 20 found the artifact assuming that periods have natures from which",
          "  their behaviour follows. Chapter 25 found it assuming a clean line between a",
          "  structure and a reading of it. Chapter 26 found the *book* assuming that the",
          "  code is the invariant and the traditions the variables.",
          "",
          "  None of the three is a rule. All three are commitments the artifact and the",
          "  book make without declaring, and there is no `N` or `F` number for them —",
          "  which is itself a gap in the scheme.",
          "]", ""]
    return "\n".join(L) + "\n"


def main() -> int:
    used = cited_rules()
    (CH / "00b-the-rules.typ").write_text(card(used))
    (CH / "d-rules.typ").write_text(appendix(used))
    print(f"  {len(used)} rules cited: {' '.join(sorted(used))}")
    missing = used - set(GLOSS)
    if missing:
        print(f"  FAIL  cited but not documented: {' '.join(sorted(missing))}")
        return 1
    print("  wrote chapters/00b-the-rules.typ  (front matter)")
    print("  wrote chapters/d-rules.typ        (Appendix D)")
    print("  ok    every rule the book cites is in the reference")
    return 0


if __name__ == "__main__":
    sys.exit(main())
