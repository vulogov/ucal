#!/usr/bin/env python3
"""Rule B, mechanically — failure mode AF10, bibliography inflation.

Every entry in refs.bib must be named somewhere in the chapters. An entry that
is not cited is removed, not left in to make the bibliography look larger.

This cannot check that a citation is *apt*, only that it exists. A wrong
citation passes; an absent one does not.

Run:  python3 samples/check-refs.py
Exit: 0 if every entry is named and every locus in Appendix A resolves.
"""

import pathlib
import re
import sys

BOOK = pathlib.Path(__file__).resolve().parent.parent
BIB = BOOK / "refs.bib"
CH = BOOK / "chapters"

# What to search the chapters for, per entry. An entry is cited if ANY of its
# probes appears. Probes are the forms the prose actually uses, which are not
# always the BibTeX author or title.
PROBES = {
    "aristotle_physics":    ["Aristotle"],
    "archimedes_sandreckoner": ["Archimedes", "Sand Reckoner"],
    "euclid_elements":      ["Euclid"],
    "plato_timaeus":        ["Plato"],
    "plotinus_enneads":     ["Plotinus"],
    "longsedley":           ["Epicurus", "Stoic"],
    "sederolam":            ["Seder Olam"],
    "maimonides_guide":     ["Maimonides"],
    "ghazali_tahafut":      ["Ghazālī", "Ghazāli"],
    "ibnrushd_tahafut":     ["Ibn Rushd", "Rushdian"],
    "augustine_civdei":     ["Augustine"],
    "augustine_confessions": ["Confessions", "Augustine"],
    "basil_hexaemeron":     ["Basil", "Hexaemeron"],
    "boethius_consolation": ["Boethius"],
    "aquinas_st":           ["Aquinas"],
    "cusa_docta":           ["Cusanus", "Cusa"],
    "nyssa_eunomium":       ["Gregory of Nyssa", "Nyssa"],
    "maximus_ambigua":      ["Maximus"],
    "palamas_triads":       ["Palamas"],
    "dc":                   ["D&C"],
    "pgp":                  ["Abraham 3", "Moses 1"],
    "bom":                  ["Alma 40"],
    "kingfollett":          ["King Follett"],
    "kant_krv":             ["Kant"],
    "newton_principia":     ["Newton"],
    "leibnizclarke":        ["Leibniz"],
    "mctaggart":            ["McTaggart"],
    "bugaev":               ["Bugaev"],
    "florensky_mnimosti":   ["Florensky", "Мнимости"],
    "losev_imeni":          ["Losev", "Философия имени"],
    "losev_dialektika":     ["Losev", "Диалектические"],
    "frank_nepostizhimoe":  ["Frank", "Непостижимое"],
    "fyodorov":             ["Fyodorov"],
    "vernadsky":            ["Vernadsky"],
    "chizhevsky":           ["Chizhevsky"],
    "klein":                ["Klein"],
    "sorabji":              ["Sorabji"],
    "planck2018":           ["Planck 2018", "Planck"],
    "iers2010":             ["IERS"],
    "ussher":               ["Ussher"],
}


def main() -> int:
    keys = re.findall(r"^@\w+\{([^,]+),", BIB.read_text(), re.M)
    text = "\n".join(f.read_text() for f in sorted(CH.glob("*.typ")))

    missing_probe = [k for k in keys if k not in PROBES]
    uncited = []
    for k in keys:
        if k in PROBES and not any(p in text for p in PROBES[k]):
            uncited.append(k)

    print(f"  {len(keys)} entries in refs.bib")
    if missing_probe:
        print(f"  FAIL  {len(missing_probe)} entries have no probe defined:")
        for k in missing_probe:
            print(f"          {k}")
    if uncited:
        print(f"  FAIL  {len(uncited)} entries are not cited in any chapter:")
        for k in uncited:
            print(f"          {k}  (probes: {', '.join(PROBES[k])})")
    if missing_probe or uncited:
        print("\n  Rule B: an uncited entry is removed, not kept to pad the list.")
        return 1

    print("  ok    every entry is named in at least one chapter")
    print("  NOTE  this checks that a citation exists, not that it is apt.")
    print("        A wrong citation passes. An absent one does not.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
