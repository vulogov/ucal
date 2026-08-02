#!/usr/bin/env python3
"""Detect a residual transparency checkerboard in any plate.

The plates arrive with the checkerboard baked into RGB. clean-plate.py removes
it by reading alpha back out of luminance — but if the cutoff lands above the
checkerboard's grey, the squares survive as a faint, perfectly regular alpha
pattern that is invisible on screen at small sizes and obvious in print.

That is exactly what happened to the moon plate: alpha ~25 in a 44-pixel grid,
which reached a compiled PDF before anyone noticed.

The signature is flatness. Real ink at faint alpha is an antialiased edge, so it
varies from pixel to pixel. A checkerboard is large blocks of one constant
value. This measures the fraction of faint 8x8 blocks that are near-constant.

Run:  python3 samples/check-plates.py
Exit: 0 if no plate carries a residual checkerboard.
"""

import pathlib
import sys

import numpy as np
from PIL import Image

IMAGES = pathlib.Path(__file__).resolve().parent.parent / "assets" / "images"
FLAT_LIMIT = 0.35          # above this, the faint pixels are a grid, not ink


def score(path: pathlib.Path):
    im = Image.open(path)
    if im.mode not in ("RGBA", "LA"):
        return None
    a = np.array(im.convert("RGBA"))[..., 3].astype(int)
    h, w = a.shape
    H, W = h // 8 * 8, w // 8 * 8
    blocks = (a[:H, :W].reshape(H // 8, 8, W // 8, 8)
              .transpose(0, 2, 1, 3).reshape(-1, 64))
    means = blocks.mean(axis=1)
    faint = blocks[(means > 8) & (means < 60)]
    if len(faint) < 32:
        return 0.0, 0
    return float((faint.std(axis=1) < 3).mean()), len(faint)


def main() -> int:
    bad = []
    for p in sorted(IMAGES.glob("*.png")):
        s = score(p)
        if s is None:
            print(f"  {p.name:<24} opaque, nothing to check")
            continue
        flat, n = s
        mark = ""
        if flat > FLAT_LIMIT:
            mark = "  <-- RESIDUAL CHECKERBOARD"
            bad.append(p.name)
        print(f"  {p.name:<24} faint blocks {n:>7,}   flat {flat:4.2f}{mark}")

    if bad:
        print(f"\n  FAIL  {len(bad)} plate(s) carry a checkerboard: {', '.join(bad)}")
        print("        Re-run clean-plate.py; the auto cutoff reads the corners,")
        print("        and the floor removes what overlaps the artwork's palest ink.")
        return 1
    print("\n  ok    no plate carries a residual checkerboard")
    return 0


if __name__ == "__main__":
    sys.exit(main())
