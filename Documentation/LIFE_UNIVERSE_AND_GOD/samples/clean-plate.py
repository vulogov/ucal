#!/usr/bin/env python3
"""Prepare an illustration plate for the book.

The plates arrive with the transparency checkerboard baked into RGB and alpha
flattened to opaque. This recovers alpha from luminance — the artwork is dark
ink on a light ground, so anything lighter than the cutoff is background — then
trims the margin and tints the ink to the book's #1a1a1a.

Usage:  python3 samples/clean-plate.py <in.png> <out.png> [cutoff]
"""

import pathlib
import sys

import numpy as np
from PIL import Image

INK = (0x1A, 0x1A, 0x1A)


def clean(src: pathlib.Path, dst: pathlib.Path, cutoff: float = 190.0) -> None:
    im = np.array(Image.open(src).convert("RGBA")).astype(np.float32)
    lum = im[..., :3].mean(axis=2)

    alpha = np.clip((cutoff - lum) / cutoff, 0, 1) * 255.0
    out = np.zeros_like(im)
    out[..., 0], out[..., 1], out[..., 2] = INK
    out[..., 3] = alpha

    img = Image.fromarray(out.astype(np.uint8), "RGBA")
    box = img.getbbox()
    img = img.crop(box)
    img.save(dst, optimize=True)
    print(f"  {src.name} -> {dst.name}  {img.size[0]}x{img.size[1]}"
          f"  ({dst.stat().st_size / 1048576:.1f} MB)")


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(__doc__)
        raise SystemExit(1)
    cut = float(sys.argv[3]) if len(sys.argv) > 3 else 190.0
    clean(pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), cut)
