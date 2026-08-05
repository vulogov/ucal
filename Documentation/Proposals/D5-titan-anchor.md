# D5 — Titan's anchor: what was looked for, and what was not found

**Status: not found. Recorded as the result, not as unfinished work.**

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

## What was actually searched, and what was not

Stated plainly because the difference matters.

**Searched:** this repository and its specification — `spec/UCAL-1.1.md` §9 and
Appendix I, `crates/ucal-body`'s data and anchor modules, and the citations
already carried for Earth and Mars, to establish what a comparable Titan
citation would have to look like. That produced the requirement above.

**Not searched:** the planetary-science literature. This cycle had no access to
it, and D5 is therefore *not* the negative result GE-3 would want — it is a
statement that the search has not been done, with the search's specification
written out so that doing it is a bounded task rather than an open one.

Calling this "no convention exists" would be an overclaim. What can be said is
narrower: **no convention is cited in this repository, none was found in what
was available to look at, and the entry stays as it is.**

## If a citation is found

Nothing else has to change. `Anchor::new` will accept it, `require_anchor`
stops returning `UCAL-E0062`, `ucal cal show titan-d <instant>` starts
rendering local fields, and `ge3_titan_has_no_anchor_and_that_is_the_answer` in
`crates/ucal-body/src/anchors.rs` becomes the test that has to change — which is
the right place for the decision to be recorded, since that test currently
asserts the absence deliberately rather than by omission.
