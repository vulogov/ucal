# D5 — Titan's anchor: what was looked for, and what was not found

**Status: searched, 0.7.0. Still not found — but the reason has changed, and the
new reason is specific enough to act on.**

`titan-d` is complete in units, intercalation and cycles, and incomplete in
phase. Asking for its local fields is `UCAL-E0062`. That has been true since
0.2.0 and this cycle went looking for the citation that would change it.

---

## What is missing, precisely

An [`Anchor`](../../crates/ucal-body/src/anchor.rs) needs seven things, and six
of them Titan already has or could have trivially:

| field | Titan |
|---|---|
| `calendar_id` | `titan-d` |
| `phase` | **the gap** |
| `tick` | follows from the phase |
| `window` | follows from the determination |
| `method` | follows from the phase |
| `citation` | **the gap** |
| `revision` | `1` |

The two gaps are one gap. A calendar's *phase* is the answer to "when does day
zero start" — and for Earth and Mars that answer is not derived, it is
**cited**. Earth's anchor is J2000.0 minus twelve hours, plus a Delta-T of
63.8285 s, which is a published quantity from a published convention. Mars's is
the Mars24 airless-mean-solar-time convention, likewise published.

Titan has no counterpart. There is no established convention naming a zero for a
Titan day, and Rule J requires an anchor to be *determined* and cited rather than
chosen.

## Why it will not be invented

GE-3's kill criterion is explicit: *"document the width rather than narrowing it
by assumption."* An invented anchor has a window of zero — not because the phase
is known to that precision, but because nothing was measured. That is the
narrowing the criterion forbids, dressed as a result.

There is a second reason, specific to this project. Rule J.2 requires an
anchor's window to contain its own tick, and `Anchor::new` returns `UCAL-E0062`
when it does not. An invented anchor would satisfy that check trivially and
would still be a fiction. The type system cannot tell a cited zero from a chosen
one, which is exactly why the citation is a field and not a comment.

## What a usable citation would have to be

Recorded so a future search knows what it is looking for, rather than
rediscovering the shape of the question:

1. **A named epoch for a Titan solar day**, published, with a stated
   determination — the way Mars24 names one for Mars.
2. **An uncertainty**, or enough detail to derive one. GE-3's question is
   whether a window narrower than one local solar day is reachable; a
   convention with no stated uncertainty cannot answer it either way, and a
   Titan solar day is about 15.9 Earth days, so the bar is not high.
3. **Independence from Earth's calendar.** An anchor expressed as "Titan noon at
   Cassini's arrival" is fine — that is an event with a tick. An anchor
   expressed as "1 January 2000 on Titan" is not: it imports an Earth epoch into
   a body-relative calendar, which is failure mode F9 and what §12's dependency
   direction exists to prevent.

## The search, run in 0.7.0

The 0.4.0 entry said the planetary-science literature had not been searched and
that D5 was therefore not a negative result, only an unperformed one. It has now
been searched against the three requirements above.

### Requirement 3 — Earth-independence: met

Not the obstacle it looked like. Every candidate below expresses itself as an
offset from J2000.0 TDB, and J2000.0 is an *instant* — a thing with a tick — not
an Earth calendar imported into a body-relative one. Earth's own anchor in this
repository is placed the same way, and for the same reason. F9 is not in play.

### Requirement 1 — a published determination: **found, but of the wrong quantity**

The IAU Working Group on Cartographic Coordinates and Rotational Elements
publishes rotational elements for Titan, current as of the 2015 report, and they
are distributed operationally in NAIF's `pck00011.tpc` as

```
BODY606_POLE_RA  =  39.4827
BODY606_POLE_DEC =  83.4279
BODY606_PM       = 186.5855   22.5769768
```

so that `W = 186.5855° + 22.5769768°/day × d`, with `d` measured in days from
J2000.0 TDB and no nutation or precession terms.

That is a published, cited, current determination of **where Titan's prime
meridian points**. It is not a determination of **when the prime meridian faces
away from the Sun**, and the anchor's `phase` field is the second thing. Earth's
anchor is mean solar midnight at the prime meridian; Mars's is the Mars24 recipe
for the same; **Titan has no counterpart, and the search did not find one.** No
mission convention, no published recipe, nothing analogous to Mars24. The
Huygens landing — the one event that could have seeded such a convention — is
reported in UTC.

One incidental confirmation, worth recording because it cost nothing: the IAU
rotation rate implies `360 / 22.5769768 = 15.945421` days, which is exactly the
rotation period `data::titan()` already carries from the NASA fact sheet. Two
independent sources, same number, and this repository was not wrong about it.

### Requirement 2 — an uncertainty: **the bar is met, and that is the surprise**

The IAU report publishes no uncertainty for Titan's `W₀`. What the literature
publishes instead is better for GE-3's purpose: Titan's rotation is measurably
**non-synchronous**, so the linear model above has a known systematic drift, and
the size of the disagreement between published models *is* a defensible width.

Stiles et al. (2008) reported a spin-rate deviation of about 0.36°/yr from
Cassini SAR landmark tracking; the same authors corrected it to about 0.12°/yr
in 2010, and Meriggiola et al. (2016) revised the rotational model again from
the full Cassini SAR set. Taking the corrected 0.12°/yr over the ~26.6 years
from J2000.0 to now gives about 3.2° of accumulated longitude, and at
22.5769768°/day that is **0.14 days ≈ 3.4 hours**.

Against GE-3's bar — one Titan solar day, which `data::titan()` derives as the
synodic period, 2045 s longer than the 15.945421-day rotation — that width is
**under one per cent of a local solar day**. Even the discredited 0.36°/yr
figure gives under three per cent.

**So the kill criterion would not fire on width.** GE-3 asked whether an anchor
could be established to a window narrower than one local solar day, and for
Titan the honest answer is now *yes, by two orders of magnitude* — if it could
be established at all.

## What actually blocks it, stated precisely

Not "no convention exists", which the 0.4.0 entry was careful not to claim, and
not "the window would be too coarse", which turns out to be false.

**The last step is an ephemeris evaluation this project does not perform.**
Turning `W(t)` into a phase means knowing where the Sun is as seen from Titan at
the epoch, and that needs a planetary and satellite ephemeris — DE440 with a
Saturnian satellite kernel, or equivalent. `ucal-body` carries measured
parameters with citations and derives from them; it does not integrate orbits
and does not link SPICE. Adding that is a different project, not a missing line
of data.

There is also the question of whether this repository *should* do it even if it
could. Deriving the phase here would make `ucal` the publisher of Titan's
solar-time convention rather than a citer of one. Rule J.5 makes anchors data
with citations, and the citation would have to be to this repository — which is
the shape of claim §19.4's `Kind` distinction exists to keep out of the
`derived` column.

## Status after the search

`titan-d` stays as it is: complete in units, intercalation and cycles,
incomplete in phase, `UCAL-E0062` when asked for local fields. The GE-3 table in
`crates/ucal-body/src/anchors.rs` keeps `Titan | no`, and
`ge3_titan_has_no_anchor_and_that_is_the_answer` keeps asserting the absence.

What changes is the reason, and it is now falsifiable by one publication: **a
recipe for Titan mean solar time, of the kind Mars24 is for Mars.** If one
appears, the width is already known to be adequate and nothing else is in the
way.

## If a citation is found

Nothing else has to change. `Anchor::new` will accept it, `require_anchor`
stops returning `UCAL-E0062`, `ucal cal show titan-d <instant>` starts
rendering local fields, and `ge3_titan_has_no_anchor_and_that_is_the_answer` in
`crates/ucal-body/src/anchors.rs` becomes the test that has to change — which is
the right place for the decision to be recorded, since that test currently
asserts the absence deliberately rather than by omission.
