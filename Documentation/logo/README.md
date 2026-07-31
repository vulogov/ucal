# Universe Calendar — identity

Black and white, vector only, two values: ink and surface.

## The mark

An engraved instrument dial that is deliberately *not* a clock:

- **Centre dot with a knocked-out core** — the datum, tick 0. The core is negative space because the datum is stipulated, not observed (Rule Q).
- **Solid sector** — absolute time elapsed, running clockwise from the zero notch to the hand.
- **Zero notch** — the small triangle outside the rim at the top. It marks the datum direction, so the sector's two edges are never ambiguous.
- **Tick band, 5 / 25 / 125** — three weights of tick on the base-5 grid. Five heavy divisions are what separate this from a twelve-hour dial at a glance; the 125 fine ticks are one full tier of resolution and read as engraved tone at large sizes.
- **One hand, on a fine tick** — it lands on fine tick 47 of 125, not on a coarse division, because the calendar's claim is that it can pinpoint an arbitrary exact instant.

## Files

| file | use | notes |
|---|---|---|
| `ucal-mark.svg` | primary, ≥96 px | full 5/25/125 tick band |
| `ucal-mark-mono.svg` | 32–96 px | five divisions only, heavier rim |
| `ucal-mark-micro.svg` | ≤24 px, favicon | sector and notch only |
| `ucal-lockup.svg` | headers, README, docs | mark + stacked wordmark + tagline |
| `ucal-wordmark.svg` | text-only contexts | `ucal` above a timeline rule beginning at the datum dot |

Tagline: **counting from the first tick** — "the first tick", not "the Big Bang", because the mark should not make a claim the specification refuses to make.

## Colour tokens

Every file uses two CSS variables with hard-coded fallbacks, so the identity inverts in dark terminals without a second asset:

```
--text-primary   ink            fallback #14110f
--surface-2      negative space fallback #f7f4ee
```

Only the datum core and nothing else uses `--surface-2`. Swapping the two values gives the dark-terminal form directly; no separate inverted file is needed.

## Clearance and sizing

- Clear space on all sides: the radius of the centre dot at the mark's current scale.
- Below 32 px use `ucal-mark-mono.svg`; below 24 px use `ucal-mark-micro.svg`. Do not scale the primary mark down — the 125 fine ticks moiré.
- The mark is never rotated. The zero notch is always at top; only the hand's angle carries meaning.
- Do not recolour. If a single-colour context requires it, set both variables to the same value and the datum core disappears — that is acceptable; a gradient or a second hue is not.

## Before release

The lockup and wordmark use live text with the stack `Iosevka, IBM Plex Mono, ui-monospace, SFMono-Regular, Menlo, monospace`. Convert both to outlines for any published artifact so rendering does not depend on the viewer's installed fonts. The mark itself contains no text and needs no conversion.
