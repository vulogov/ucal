---
name: "C1 — I reproduced (or failed to reproduce) the constants"
about: "Thirty minutes, any language, no Rust. The most valuable thing this project can receive."
labels: ["contact", "conformance"]
---

<!--
Documentation/CONTACT.md has the whole task. Short version: compute BEAT,
SECOND and ORIGIN_OFFSET from fixtures/vectors.json in any language with big
integers, and say what you got.

A mismatch is worth more than a match. "I could not tell what to compute from
the documents" is worth more than either, because it says the specification is
not self-contained, and this repository cannot discover that from the inside.
-->

**Language / tools used:**

**BEAT** — expected `867361737988403547205962240695953369140625`
- got:

**SECOND** — expected `18548584399861000000000000000000000000000000`
- got:

**ORIGIN_OFFSET** — expected
`8070204002895596515944343085635637180530466139316558837890625`
- got:

**Did you go further than the three?** (tiers, fixtures, continued fractions)

**Was anything unclear, ambiguous, or only discoverable by reading the Rust?**
<!-- This is the field that matters most. The specification is supposed to be
     implementable without this codebase; if it was not, say where. -->
