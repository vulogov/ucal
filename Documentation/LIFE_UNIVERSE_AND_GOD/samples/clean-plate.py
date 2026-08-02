#!/usr/bin/env python3
"""Prepare an illustration plate for the book.

The plates arrive with the transparency checkerboard baked into RGB and alpha
flattened to opaque. This recovers alpha from luminance — the artwork is dark
ink on a light ground, so anything lighter than the cutoff is background — then
trims the margin and tints the ink to the book's #1a1a1a.

The cutoff is measured from the file rather than assumed. A fixed 190 worked for
three plates and failed on a fourth whose checkerboard grey was around 171: the
squares survived at alpha ~25 and printed as a visible grid behind the artwork.
Different exports use different greys, so the tool reads the corners.

A floor is then applied. Below it, alpha goes to zero — that is what removes the
last of a checkerboard whose tone overlaps the artwork's faintest ink, and it
costs only marks too pale to print.

Usage:  python3 samples/clean-plate.py <in.png> <out.png> [cutoff] [floor]
        cutoff/floor default to auto and 33.

Verify:  python3 samples/check-plates.py
"""

import pathlib
import sys

import numpy as np
from PIL import Image

INK = (0x1A, 0x1A, 0x1A)
FLOOR = 33


def background_level(lum: np.ndarray) -> float:
    """Darkest tone that is still background, read from the four corners.

    A checkerboard puts two tones there — white and a grey. The cutoff has to
    sit below the darker of them, or the grey squares survive as faint alpha.
    """
    h, w = lum.shape
    k = max(40, min(h, w) // 20)
    corners = np.concatenate([
        lum[:k, :k].ravel(), lum[:k, -k:].ravel(),
        lum[-k:, :k].ravel(), lum[-k:, -k:].ravel(),
    ])
    # 2nd percentile: the darker checkerboard tone, ignoring stray ink.
    darkest_bg = float(np.percentile(corners, 2))
    # Sit a little below it so the squares clear completely.
    return max(60.0, darkest_bg - 8.0)


def clean(src: pathlib.Path, dst: pathlib.Path,
          cutoff: float | None = None, floor: int = FLOOR) -> None:
    im = np.array(Image.open(src).convert("RGBA")).astype(np.float32)
    lum = im[..., :3].mean(axis=2)

    auto = cutoff is None
    if auto:
        cutoff = background_level(lum)

    alpha = np.clip((cutoff - lum) / cutoff, 0, 1) * 255.0
    alpha[alpha < floor] = 0.0

    out = np.zeros_like(im)
    out[..., 0], out[..., 1], out[..., 2] = INK
    out[..., 3] = alpha

    img = Image.fromarray(out.astype(np.uint8), "RGBA")
    box = img.getbbox()
    if box is None:
        print(f"  {src.name}: nothing survived the cutoff — is it light ink?")
        return
    img = img.crop(box)
    img.save(dst, optimize=True)
    print(f"  {src.name} -> {dst.name}  {img.size[0]}x{img.size[1]}"
          f"  cutoff {cutoff:.0f}{' (auto)' if auto else ''}"
          f"  floor {floor}  ({dst.stat().st_size / 1048576:.1f} MB)")


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(__doc__)
        raise SystemExit(1)
    cut = float(sys.argv[3]) if len(sys.argv) > 3 else None
    flr = int(sys.argv[4]) if len(sys.argv) > 4 else FLOOR
    clean(pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), cut, flr)
