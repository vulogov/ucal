#!/usr/bin/env python3
"""Generate the diagnostic-code appendix from ucal-core's error module.

The table is derived rather than transcribed, for the same reason §13.5 makes
the tier table generated: a hand-copied list of error codes drifts silently, and
a reference that disagrees with the software is worse than no reference.

Writes chapters/c-diagnostics.typ. Run after any change to error.rs.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
ERR = ROOT / "crates" / "ucal-core" / "src" / "error.rs"
OUT = pathlib.Path(__file__).resolve().parent.parent / "chapters" / "c-diagnostics.typ"

# Where each family is discussed. Hand-maintained, because a chapter reference
# is an editorial claim and not something the source knows about.
CHAPTER = {
    "E00": "6", "E002": "6",
    "E0020": "3", "E0021": "5", "E0022": "12", "E0023": "5", "E0024": "5",
    "E0025": "12", "E0030": "6", "E0031": "6", "E0032": "6",
    "E0040": "7", "E0041": "7", "E0042": "7", "E0043": "7",
    "E0050": "5", "E0060": "5", "E0061": "5",
    "E0062": "8", "E0063": "8", "E0064": "8", "E0065": "8",
    "E0070": "5", "E0071": "5", "E0080": "5",
    "E0010": "12", "E0011": "12", "E0012": "12", "E0013": "12",
    "W0001": "5", "W0002": "7", "W0003": "8", "W0004": "5",
    "W0005": "8", "W0006": "12",
}

BANDS = [
    ("E0001–E0007", "notation and parsing", "2"),
    ("E0010–E0013", "profile and provenance", "6"),
    ("E0020–E0025", "domain, ordering, and the claim", "3, 9"),
    ("E0030–E0032", "identifiers and encoding", "2, 3"),
    ("E0040–E0043", "the SI bridge and civil time", "2, 4"),
    ("E0050–E0065", "bodies, anchors, and calendars", "5, 7"),
    ("E0070–E0080", "numerics and cosmology", "3, 8"),
]


def parse():
    s = ERR.read_text()
    desc = {}
    for c, d in re.findall(r'Code::([EW]\d+) => "([^"]+)"', s):
        if not d.startswith("UCAL-"):
            desc[c] = d
    for c, d in re.findall(r'Warning::(W\d+) => "([^"]+)"', s):
        if not d.startswith("UCAL-"):
            desc[c] = d
    exits = {}
    body = s[s.index("fn exit_code"):]
    body = body[:body.index("\n    }")]
    for arm in re.finditer(r'((?:Code::[EW]\d+\s*\|?\s*)+)=>\s*(\d+)', body):
        for c in re.findall(r'[EW]\d+', arm.group(1)):
            exits[c] = arm.group(2)
    return desc, exits


def main() -> int:
    if not ERR.exists():
        print(f"  {ERR} not found")
        return 1
    desc, exits = parse()
    errs = sorted(c for c in desc if c.startswith("E"))
    warns = sorted(c for c in desc if c.startswith("W"))

    L = ['#import "../design.typ": *', "",
         '#appendix(letter: "C", title: "Diagnostic codes")', "",
         "Generated from `ucal-core`'s error module by",
         "`samples/gen-diagnostics.py`. The table is derived rather than",
         "transcribed, for the same reason §13.5 makes the tier table generated: a",
         "hand-copied list drifts silently, and a reference that disagrees with the",
         "software is worse than none.", "",
         "#section(\"What the codes are for\")", "",
         "Chapter 16 counted four epistemic limits and found the same response in all",
         "four — the system errors or warns and never defaults. This appendix is that",
         "policy enumerated. Every entry below is a place where the artifact declines",
         "to produce a plausible number.", "",
         "#section(\"Exit codes\")", "",
         "The command line maps each family to a process exit status, so a failure is",
         "distinguishable by class without parsing the message.", "",
         "#block(width: 100%)[", "  #set text(size: 9.5pt)", "  #table(",
         "    columns: (auto, 1fr, auto),",
         "    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },",
         "    inset: (x: 5pt, y: 4.5pt),",
         "    [*band*], [*subject*], [*exit*],"]
    for band, subject, ex in BANDS:
        L.append(f"    [`{band}`], [{subject}], [{ex}],")
    L += ["  )", "]", "",
          "#section(\"Errors\")", "",
          "#block(width: 100%)[", "  #set text(size: 9pt)", "  #table(",
          "    columns: (auto, 1fr, auto, auto),",
          "    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },",
          "    inset: (x: 5pt, y: 4pt),",
          "    [*code*], [*meaning*], [*exit*], [*ch.*],"]
    for c in errs:
        ch = CHAPTER.get(c, "—")
        L.append(f"    [`UCAL-{c}`], [{desc[c]}], [{exits.get(c, '—')}], [{ch}],")
    L += ["  )", "]", "",
          "#section(\"Warnings\")", "",
          "A warning is returned alongside a value. It never replaces one, and it is",
          "never suppressed by default — chapter 8's `UCAL-W0003` and chapter 5's",
          "`UCAL-W0004` are both cases where the answer is real and incomplete.", "",
          "#block(width: 100%)[", "  #set text(size: 9pt)", "  #table(",
          "    columns: (auto, 1fr, auto),",
          "    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },",
          "    inset: (x: 5pt, y: 4pt),",
          "    [*code*], [*meaning*], [*ch.*],"]
    for c in warns:
        L.append(f"    [`UCAL-{c}`], [{desc[c]}], [{CHAPTER.get(c, '—')}],")
    L += ["  )", "]", "",
          "#callout(label: \"The four this book turns on\")[",
          "  / `UCAL-E0020`: a result preceding the datum. Chapter 3 — a malformed",
          "    question refused, not a value on the far side of an origin.",
          "  / `UCAL-E0025`: `BIG_BANG_CLAIM` used as an operand. Chapter 12, and the",
          "    only code in the set that no program can reach, because the type has no",
          "    operators for it to reach through.",
          "  / `UCAL-E0062`: a calendar with no anchor. Chapter 16 — an absence",
          "    reported rather than defaulted.",
          "  / `UCAL-W0003`: a parameter outside its validity window. Chapter 20 found",
          "    this to be *ʿāda* compiled: reliable where observed, no claim beyond.",
          "]", ""]
    OUT.write_text("\n".join(L) + "\n")
    print(f"  wrote chapters/c-diagnostics.typ — {len(errs)} errors, {len(warns)} warnings")
    return 0


if __name__ == "__main__":
    sys.exit(main())
