#!/usr/bin/env python3
"""A-P8, the deletion test — RFC UCAL-A1 failure mode AF4.

Rule M requires that no factual claim about the software depend on an
interpretive one. The book asserts this in its preface, so it has to be
checkable rather than believed.

The test: remove every `#claim("interpretation")` and `#claim("resonance")`
block, compile what remains, and confirm the technical book still stands. A
surviving reference *into* a deleted block — "as the interpretation above
shows" — is a failure, because it means a technical claim was leaning on one.

Run:  python3 deletion-test.py
Exit: 0 if the technical book stands without the marked blocks.
"""

import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).parent
MARKED = ("interpretation", "resonance")

# Phrases that would mean prose outside a marked block is depending on one.
DANGLING = [
    re.compile(r"as the (interpretation|resonance) (above|below)", re.I),
    re.compile(r"the (interpretation|resonance) (above|below) (shows|argues|establishes)", re.I),
    re.compile(r"(shown|established|argued) in the (interpretation|resonance)", re.I),
]


def strip_marked(src: str) -> tuple[str, int]:
    """Remove `#claim("kind")[ ... ]` blocks, matching brackets properly."""
    out, removed, i = [], 0, 0
    while i < len(src):
        m = re.compile(r'#claim\("(\w+)"\)\s*\[').search(src, i)
        if not m:
            out.append(src[i:])
            break
        if m.group(1) not in MARKED:
            out.append(src[i:m.end()])
            i = m.end()
            continue
        out.append(src[i:m.start()])
        depth, j = 1, m.end()
        while j < len(src) and depth:
            if src[j] == "\\":
                j += 2
                continue
            if src[j] == "[":
                depth += 1
            elif src[j] == "]":
                depth -= 1
            j += 1
        removed += 1
        i = j
    return "".join(out), removed


def main() -> int:
    chapters = sorted((ROOT / "chapters").glob("*.typ"))
    drafted = [c for c in chapters if "unwritten" not in c.name]

    with tempfile.TemporaryDirectory() as td:
        work = pathlib.Path(td) / "book"
        shutil.copytree(ROOT, work, ignore=shutil.ignore_patterns("*.pdf", "*.png"))

        total, per_file = 0, []
        for ch in drafted:
            f = work / "chapters" / ch.name
            stripped, n = strip_marked(f.read_text())
            f.write_text(stripped)
            total += n
            if n:
                per_file.append((ch.name, n))

        print(f"  removed {total} marked block(s) from {len(per_file)} chapter(s)")
        for name, n in per_file:
            print(f"      {name:<34} {n}")

        # 1. It must still compile.
        r = subprocess.run(
            ["typst", "compile", str(work / "BOOK.typ"), str(work / "out.pdf")],
            capture_output=True, text=True,
        )
        if r.returncode != 0:
            print("\n  FAIL  the book does not compile without its marked blocks")
            print(r.stderr[:1500])
            return 1
        print("  ok    compiles without the marked blocks")

        # 2. Nothing may refer into a deleted block.
        bad = []
        for ch in drafted:
            text = (work / "chapters" / ch.name).read_text()
            for pat in DANGLING:
                for m in pat.finditer(text):
                    bad.append((ch.name, m.group(0)))
        if bad:
            print("\n  FAIL  prose outside a marked block depends on one:")
            for name, frag in bad:
                print(f"      {name}: {frag!r}")
            return 1
        print("  ok    no surviving prose refers into a deleted block")

    print("\n  A-P8 GREEN — AF4 holds for the drafted chapters.")
    print("  Note: this checks structural dependence. It cannot check whether an")
    print("  argument became less persuasive, only whether a claim became unsupported.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
