# Z2 — more faces, and what a clock for this calendar should do next

**Status: proposal. Nothing built. Each item has a cost, a reason, and a
condition for not doing it.**

`ucal wallclock` shipped in 1.5.0 with two themes and one layout. This is what
should come next and, more usefully, what should not.

---

## The one that earns the others: `--at` and `--once`

```
ucal wallclock --once --at <INSTANT> --theme startrek
```

Render a single frame for a given instant and exit.

**This is first, and it is not a display feature.** Everything else on this page
is a claim about what the clock looks like, and today the only mechanism holding
those claims is a test that renders into a `TestBackend` and greps the result.
That is real, and it is invisible: nobody reading `Documentation/CLI.md` can see
a face, so the documentation describes one in prose and the prose can rot.

With `--once --at`, a frame becomes a **generated artefact**, like
`Documentation/CLI-EXAMPLES.md` already is. `xtask gen-examples` writes one, and
`check-docs` fails when the committed frame stops matching what the binary
produces. The same mechanism that catches a stale worked example catches a stale
screenshot.

It is also the cheapest item here — the render path already takes a `Face` built
from an arbitrary instant, because that is how the tests use it. What is missing
is a flag and a `TestBackend`-to-text dump that is not test-only.

**Cost:** small. **Do it first**, because it converts every later item from
"looks nice" to "checkable".

**Stop if:** the frame is not stable enough to commit — but it is, by
construction, once the instant is given.

---

## Themes that are only a palette

`Theme` is a struct of colours and one `lcars: bool`. Three of these are a
`const` each and nothing else:

- **`amber`** — DEC VT220 phosphor. One warm orange on near-black, no second
  hue. The block font was drawn for exactly this and has never been shown in it.
- **`green`** — IBM 3270 / VT100 green. The other half of the same idea, and
  between them they cover what most people mean by "terminal".
- **`paper`** — high-contrast dark-on-light, no colour at all. The one that is
  actually *useful* rather than evocative: `plain` uses `Color::Reset` and
  inherits whatever the terminal has, which is fine until someone screenshots it
  on a light background and the labels vanish. `paper` commits.

**Cost:** three `const`s, three lines in `ALL`, and the existing tests cover them
the moment they are listed — `every_theme_draws_at_every_plausible_size` and
`the_face_names_no_earth_unit_as_a_unit` both iterate `theme::ALL`.

**Stop if:** nothing. This is the cheapest useful thing on the page and the tests
are already written.

---

## Themes that are a layout

### `conn` — the second LCARS

The reference images were three: an LCARS watch face, a conn station, and a
bridge operations panel. 1.5.0 built one layout from the first. The conn station
is a genuinely different arrangement — denser, a status column of small blocks
down the right, readouts in a grid rather than one hero number — and it would
put the four rail hands on equal footing instead of promoting the beat.

Worth it only if the *second* layout proves the abstraction. Today `lcars: bool`
is a switch, not a layout system; a third layout is where that stops being
honest and `Theme` needs a `layout: Layout` enum. That refactor is the actual
deliverable and the theme is the reason for it.

**Stop if:** the two LCARS layouts differ only in colour and block placement, in
which case they are one layout with parameters and should say so.

### `blueprint`

Cyan on dark navy, thin rules, dimension-line flourishes. Attractive, and it
carries no idea the others do not. **Recommend not doing it** — a theme should
be a different way of *reading* the clock or a different medium, not a different
mood.

---

## The big one: an analogue face

Every theme so far renders digits. A wall clock has **hands**, and this calendar
has a natural dial: every tier has exactly 3125 stops, so each is a circle with
3125 positions, and the tiers nest exactly the way an hour hand nests in a
minute hand.

A braille-cell canvas gives 2×4 sub-pixels per character, so a 40×20 pane is a
160×80 dot field — enough for four concentric dials or a row of four faces.

**The interesting constraint is Rule E.** Drawing a hand at position `p` of 3125
needs a sine and a cosine, and this program contains no floating point anywhere
and is not about to. The answer is a **precomputed integer table**: 3125 is
`5^5`, the table is one quadrant of 782 entries scaled to a fixed denominator,
and everything else is symmetry and integer multiplication. That is a pleasing
amount of work for a clock built on powers of five, and it is exactly the kind of
thing this project should be willing to do rather than reach for `f64`.

**Cost:** the largest item here — a canvas, a table, and a generator for the
table that `check-docs` can re-derive. **Do it after `--once`**, so the result is
committable as a generated frame and reviewable without running anything.

**Stop if:** the dials are unreadable at 80×24, which is the size that matters.
A dial with 3125 stops in a 20-character circle resolves about one stop in 30;
if that reads as noise rather than a hand, the honest answer is that this
calendar's tiers are *counters* and not *dials*, and that is worth recording as
a finding rather than shipping as a face.

---

## Features, in the order they earn themselves

### Several dials

`--clock-local` taken more than once: `--clock-local earth-d --clock-local
mars-d`. An airport wall. The `Local` struct is already a list of one, and the
rendering is a loop.

**Cost:** small. **Stop if:** nothing — but note that only two of the twelve
derived calendars can be a dial at all, so the wall is a wall of two until an
anchor is established.

### `--tier <T>` — choose the hero

The big readout is `T0` because that is the tier that moves at a watchable rate.
At other scales a reader might want `T1` promoted and the beat demoted to the
bar. One argument, and it makes the clock usable as a *calendar* display rather
than a *clock* display.

**Cost:** small. **Stop if:** every choice but `T0` produces a screen where
nothing moves, which is a stopped clock with extra steps.

### `--since <INSTANT>` — an odometer

Elapsed time from a given instant, rendered in tiers. This is the classic second
function of a wall clock and it ticks, which is what makes it a clock feature
rather than a report.

The obvious extension — `--since <event-id>`, against the catalogue — is where
it gets interesting and mostly does not work. `holocene`'s window is about two
centuries wide; a reading that ticks at 21 beats per second against a citation
uncertain by two centuries is theatre. `ucal events show` already prints that
number, statically, which is the honest way to present it.

**The exception is `bridge-epoch`**, which is exact by definition, and for which
"time since the epoch" is a real, ticking, meaningful odometer.

**Stop if:** the window of the chosen origin exceeds the resolution of the
display, which is a check the program can make. Then it should refuse and say
why, rather than render a number whose last twelve digits are decoration — the
same judgement `UCAL-E0023` already makes for comparisons.

---

## What not to build

- **A settings screen, or runtime theme switching.** A clock has one job. The
  flags are the interface.
- **Notifications, alarms, or anything that writes.** `ucal` reads a clock and
  prints; nothing in it has ever had a side effect, and a clock is a bad place to
  start.
- **A web or GUI version.** `GE-U4` already declined to put a TUI in the default
  install; a second front end is a second surface to keep honest, and there is
  one maintainer.
- **Animation, easing, or transitions.** The flicker bar moves 66 000 times a
  second. Anything smoothed is a lie about a real quantity.

---

## Recommended order

1. **`--at` / `--once`** — small, and makes everything after it checkable.
2. **`amber`, `green`, `paper`** — three `const`s, covered by existing tests.
3. **Several dials**, and **`--tier`** — both small, both obvious in use.
4. **The analogue face**, with `orbit` as its theme — the substantial one, and
   the only item that would teach the reader something new about the calendar.
5. **`conn`**, if and only if it forces `Theme` to grow a real layout enum.
6. **`--since`**, with the window check as its kill criterion.

Items 1–3 are one small cycle. Item 4 is a cycle of its own.
