#!/usr/bin/env python3
"""S1–S6 — the six samples of RFC UCAL-A1 §18.

Each writes a real artifact to ../assets/output/. Nothing here is illustrative:
every number in Part VII of the book comes from a file this script produced,
against the source tree pinned in PINNED.md.

Run:  python3 samples/run-samples.py
"""

import pathlib
import subprocess
import sys
from fractions import Fraction as F

ROOT = pathlib.Path(__file__).resolve().parents[3]
OUT = pathlib.Path(__file__).resolve().parent.parent / "assets" / "output"
UCAL = ROOT / "target" / "release" / "ucal"

# A pinned instant. S4 and S5 must produce the same artifact on every run, so
# they cannot read the clock — a sample whose output changes between runs is a
# demonstration, not evidence.
PINNED_INSTANT = "8070205189128471254993117657693008777530466139316558837890625"


def ucal(*args) -> str:
    r = subprocess.run([str(UCAL), *args], capture_output=True, text=True)
    if r.returncode != 0:
        return f"<<error: {r.stderr.strip()}>>"
    return r.stdout


def field(text: str, key: str) -> str:
    for line in text.splitlines():
        p = line.split()
        if p and p[0] == key:
            return p[1]
    return "?"


def write(name: str, body: str) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / name).write_text(body)
    print(f"  wrote assets/output/{name}  ({len(body)} bytes)")


# ---------------------------------------------------------------------------
# S1 — comparative chronology on one axis
# ---------------------------------------------------------------------------

EPOCHS = [
    ("Seder Olam (Masoretic AM 1)", "3761 BC-10-07", "rabbinic; Seder Olam Rabbah"),
    ("Byzantine AM 1", "5509 BC-09-01", "Septuagint genealogies"),
    ("Ussher", "4004 BC-10-23", "Annales Veteris Testamenti, 1650"),
    ("Julian Day epoch", "4713 BC-01-01", "Scaliger, 1583 — computational"),
]


def s1() -> str:
    L = ["S1 — comparative chronology on one axis",
         "=" * 74, "",
         "Four declared epochs, each converted to absolute ticks. Rule P forbids",
         "comparing them as though they were the same profile; putting them on one",
         "axis is exactly what a profile tag exists to make safe.", ""]
    rows = []
    for name, date, note in EPOCHS:
        out = ucal("from-civil", date, "--calendar", "julian", "--scale", "tt")
        t = field(out, "ticks")
        rows.append((name, date, t, note))
    datum_to_now = int(field(ucal("now"), "ticks"))
    L.append(f"{'epoch':<30}{'civil (Julian)':<18}{'ticks since datum'}")
    L.append("-" * 74)
    for name, date, t, _ in rows:
        L.append(f"{name:<30}{date:<18}{t}")
    L.append("")
    L.append("Spread between the widest pair, in ticks and in years:")
    ts = sorted(int(r[2]) for r in rows if r[2].isdigit())
    year = 31_557_600 * 18_548_584_399_861 * 10**30
    L.append(f"  {ts[-1] - ts[0]}")
    L.append(f"  {(ts[-1] - ts[0]) / year:.1f} years")
    L.append("")
    L.append("Provenance:")
    for name, _, _, note in rows:
        L.append(f"  {name:<30}{note}")
    L.append("")
    L.append("NOTE. These are four different profiles. The table is legitimate only")
    L.append("because each row carries its own; arithmetic across rows is not.")
    return "\n".join(L) + "\n"


# ---------------------------------------------------------------------------
# S2 — calendar audit by convergent   (gated experiment GE-A2)
# ---------------------------------------------------------------------------

# Earth's tropical year as the crate declares it: 365.242190 mean solar days.
FRAC = F(365242190, 1000000) - 365

# The historical form is kept alongside the value, because Fraction reduces
# 218/900 to 109/450 and the rule is not known by that name.
RULES = [
    ("Julian",             F(1, 4),     "1/4",     "Rome, 45 BC"),
    ("Gregorian",          F(97, 400),  "97/400",  "reform of 1582"),
    ("Revised Julian",     F(218, 900), "218/900", "Milankovic, 1923"),
    ("Persian (Jalali)",   F(8, 33),    "8/33",    "Khayyam et al., 1079"),
    ("Medler",             F(31, 128),  "31/128",  "proposed 1864"),
    ("Coptic / Ethiopian", F(1, 4),     "1/4",     "same rule as Julian"),
]


def convergents(x: F, depth: int = 20):
    """Continued-fraction convergents of x — the same walk derive_leap_rule does.

    The leading term is dropped. For x < 1 the integer part is 0, so the first
    convergent would be 0/1, which is not an intercalation rule and which would
    shift every index by one against the crate's own numbering.
    """
    a, out, h1, h2, k1, k2 = x, [], 1, 0, 0, 1
    for _ in range(depth):
        q = a.numerator // a.denominator
        h1, h2 = q * h1 + h2, h1
        k1, k2 = q * k1 + k2, k1
        out.append(F(h1, k1))
        rem = a - q
        if rem == 0:
            break
        a = 1 / rem
    return [c for c in out if c != 0]


def s2() -> str:
    conv = convergents(FRAC)
    L = ["S2 — calendar audit by convergent",
         "=" * 74, "",
         f"Earth's fractional day-per-year: {FRAC} = {float(FRAC):.6f}",
         "",
         "The convergents are the best rational approximations: no fraction with a",
         "smaller denominator comes closer. A historical rule either is one or is not.",
         "",
         f"{'rule':<20}{'value':>12}{'convergent?':>14}{'1 day slips in':>18}",
         "-" * 74]
    verdicts = []
    for name, r, shown, _src in RULES:
        err = abs(FRAC - r)
        slip = "never (exact)" if err == 0 else f"{float(1/err):,.0f} yr"
        idx = conv.index(r) + 1 if r in conv else None
        verdict = f"yes — #{idx}" if idx else "NO"
        verdicts.append((name, r, idx))
        L.append(f"{name:<20}{shown:>12}{verdict:>14}{slip:>18}")
    L.append("")
    L.append("Earth's convergent ladder, for reference:")
    for i, c in enumerate(conv, 1):
        e = abs(FRAC - c)
        s = "exact" if e == 0 else f"{float(1/e):,.0f} yr"
        L.append(f"  {i:>2}: {str(c):<16}{s}")
    L.append("")
    derived = [n for n, _, i in verdicts if i]
    declared = [n for n, _, i in verdicts if not i]
    L.append("FINDING")
    L.append(f"  derived (a convergent): {', '.join(sorted(set(derived)))}")
    L.append(f"  declared (not one):     {', '.join(sorted(set(declared)))}")
    L.append("")
    L.append("GE-A2 asked whether this audit produces a non-trivial result — the kill")
    L.append("criterion being that every historical calendar turns out to be a")
    L.append("convergent, which would make the finding empty. It does not: the split")
    L.append("is real, and it does not follow accuracy. The Persian rule of 1079 is")
    L.append("derived and the Gregorian reform of 1582 is not.")
    return "\n".join(L) + "\n"


# ---------------------------------------------------------------------------
# S3 — uncertainty audit: cited against stipulated
# ---------------------------------------------------------------------------

def s3() -> str:
    L = ["S3 — uncertainty audit",
         "=" * 74, "",
         "For any chronology, separate what is cited from what is stipulated.",
         "The worked case is UC-1's own datum, because it is the one this project",
         "is answerable for.", "",
         ucal("datum").split("datum_provenance:")[1].split("residual")[0].rstrip()
         if "datum_provenance:" in ucal("datum") else "<<no provenance>>",
         "",
         "CITED      13.787 Gyr +/- 0.020 Gyr, Planck 2018 VI",
         "           31 557 600 s per Julian year (definitional)",
         "           SECOND, as the declared bridge constant",
         "STIPULATED that tick 0 is the origin of the count",
         "           that the datum is rounded to a whole beat",
         "DISCARDED  0.017190364 s, reported rather than absorbed",
         "",
         "The same audit applied to Seder Olam would separate the scriptural",
         "genealogies (cited, in the sense that they are quoted) from the",
         "compression of the Persian period (stipulated, and not presented as",
         "such). That audit is computable. Chapter 19 explains why this book",
         "computes it and does not conclude from it.",
         ]
    return "\n".join(L) + "\n"


# ---------------------------------------------------------------------------
# S4 — cross-body simultaneity
# ---------------------------------------------------------------------------

def s4() -> str:
    now = PINNED_INSTANT
    body = ucal("show", now, "--calendars", "earth-d,mars-d,earth-civil")
    return ("S4 — cross-body simultaneity\n" + "=" * 74 + "\n\n"
            "One instant, three calendars. Each rendering carries its kind and its\n"
            "anchor revision, so values from different determinations are never\n"
            "silently compared. \"Now\" is not a shared object.\n\n" + body)


# ---------------------------------------------------------------------------
# S5 — measuring diastema with no Earth content
# ---------------------------------------------------------------------------

def s5() -> str:
    now = PINNED_INSTANT
    exp = ucal("explain", now)
    tiers = exp.split("tiers:")[1].split("beats_since_datum:")[0].rstrip() \
        if "tiers:" in exp else ""
    beats = exp.split("beats_since_datum:")[1].split("si_bridge:")[0].rstrip() \
        if "beats_since_datum:" in exp else ""
    return ("S5 — measuring diastema\n" + "=" * 74 + "\n\n"
            "The datum-to-present interval, stated with no Earth content in the\n"
            "units. Every quantity below is a count of ticks or of powers of five\n"
            "of ticks. No second, no day, no year appears.\n\n"
            f"ticks since the datum:\n  {now}\n\n"
            f"on the tier ladder:{tiers}\n\n"
            f"in beats — the universe second:{beats}\n\n"
            "Chapter 22 supplies the word for what this measures: diastema, the\n"
            "interval that is the mark of createdness. That is why the instrument\n"
            "reaches exactly this far and no further.\n")


# ---------------------------------------------------------------------------
# S6 — a revealed ratio, evaluated
# ---------------------------------------------------------------------------

def s6() -> str:
    # Abraham 3:4 — one Kolob revolution = 1000 years "according to the time
    # appointed unto that whereon thou standest".
    year_s = 31_557_600
    sec_ticks = 18_548_584_399_861 * 10**30
    kolob_rev_ticks = 1000 * year_s * sec_ticks
    beat = 5**60
    L = ["S6 — a revealed ratio, evaluated",
         "=" * 74, "",
         "Abraham 3:4 states a conversion with a profile tag on each side:",
         "",
         '  "one revolution was a day unto the Lord, after his manner of',
         '   reckoning, it being one thousand years according to the time',
         '   appointed unto that whereon thou standest."',
         "",
         "Read as a bridge constant (Rule Y), the source unit is one Kolob",
         "revolution and the target is one thousand Earth years.",
         "",
         "WHAT IT IMPLIES",
         f"  1 Kolob revolution = {kolob_rev_ticks} ticks",
         f"                     = {kolob_rev_ticks // beat} beats",
         f"                     = {kolob_rev_ticks / (5**80):.6f} drifts",
         "",
         "  It is NOT a whole number of beats. The remainder is",
         f"    {kolob_rev_ticks % beat} ticks",
         "  which is 5^60-relative, not a rounding artifact: a thousand Julian",
         "  years is a whole number of seconds and seconds carry only 5^30, so",
         "  the ratio lands off the tier grid by construction. Chapter 7's",
         "  incommensurability, arriving where nobody was looking for it.",
         "",
         "WHAT IT LEAVES UNDETERMINED",
         "  the phase — when a revolution begins. Rule J requires an anchor, and",
         "  the text supplies a period without one.",
         "  the uncertainty — Rule C requires an epoch, a rate and a validity",
         "  window. The text supplies a ratio with none of the three.",
         "  the determination method — 'revealed' is not a value the schema's",
         "  Determination field admits.",
         "",
         "VERDICT",
         "  ACCEPTED as a declared bridge constant. Rule Y requires a bridge to",
         "  be declared and tagged on both sides, and this one is — more",
         "  explicitly than most technical writing manages.",
         "  REFUSED as a Rule J anchor. An anchor must carry a determination",
         "  with a stated uncertainty window, and no reading of the text supplies",
         "  one without inventing it.",
         "",
         "This is a template, not a special case. Any tradition's stated ratio",
         "gets the same treatment: the period is usable, the phase is not, and",
         "the refusal is about the schema rather than about the source.",
         ]
    return "\n".join(L) + "\n"


def main() -> int:
    if not UCAL.exists():
        print(f"  {UCAL} not found — run `cargo build --release` first")
        return 1
    for name, fn in [("S1-chronologies.txt", s1), ("S2-calendar-audit.txt", s2),
                     ("S3-uncertainty-audit.txt", s3), ("S4-simultaneity.txt", s4),
                     ("S5-diastema.txt", s5), ("S6-revealed-ratio.txt", s6)]:
        write(name, fn())
    return 0


if __name__ == "__main__":
    sys.exit(main())
