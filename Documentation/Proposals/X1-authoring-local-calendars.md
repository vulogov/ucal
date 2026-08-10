# X1 — deriving a local calendar for a body that does not ship with the program

**Status: research. No work started. Two findings recorded below are defects
independent of whether any of this is built.**

---

## The question

`ucal-body` derives a calendar from a body: intercalation from the ratio of the
solar day to the year, cycles from a satellite's synodic period, phase from a
cited anchor. Seven calendars ship. Rule K.5 says Earth is an ordinary instance
and §15.4 says its entry has no special code path, and both are true.

So: **can somebody who is not the author derive a calendar for a body of their
choosing?** Not read one of the seven — produce one.

The answer today is no, and the reasons are worth separating, because one of
them is a specification requirement that was never implemented.

## What already works

The derivation chain is real and end-to-end auditable, which §15.2 requires:

```
  body parameters (cited, with epoch + window)
        │
        ├─ solar_day / orbital_period ──► derive_leap_rule ──► LeapRule
        │                                  (continued fraction, every convergent kept)
        │
        ├─ satellite synodic period ────► derive_cycles ─────► Cycle
        │                                  (or nothing — §15.3 forbids a fallback)
        │
        └─ + Anchor (cited, versioned, interval-valued)
                    │
                    ▼
              BodyCalendar::fields(t) ──► year, day, day_fraction, cycle, window
```

Nothing in that is Earth-shaped, and 0.8.0 added four bodies without the
mechanism moving, which is the only evidence that claim can have.

---

## Finding 1 — §15.1 is unimplemented, and was never recorded as such

> **15.1** Loader `deser-hjson`, strict (unknown keys → `UCAL-E0012`). Body files
> and anchor files are separate and version independently: parameters change
> with better measurement, anchors with re-determination.

There is no loader. `data::all()` is a hardcoded `Vec` of Rust constructor calls
and `anchors::CALENDARS_WITH_ANCHORS` is a hardcoded `&[&str]`. Adding a body
means editing `ucal-body`, rebuilding, and republishing to crates.io.

**This is the blocker.** Everything else on this page is a design question;
this one is the difference between "a calendar can be derived" and "a calendar
can be derived *by someone else*".

It is also a conformance gap in a specification this project has spent eighteen
deltas keeping honest, and it is not among them. `spec/SPEC-DELTAS.md` records
where the RFC was wrong; nothing records where the implementation is simply
absent — which is a category the delta machinery does not currently have.

### The corroborating detail

`UCAL-E0012` — *unknown key in an HJSON data file* — **has no raiser anywhere in
the workspace.** It is a diagnostic defined for a loader that does not exist.

That was invisible until 1.2.0, when D-A19 moved `ucal events show`'s misuse of
it onto `UCAL-E0016`. Removing the one wrong caller left the code with no
callers at all, and the absence became legible. A borrowed code had been
concealing a missing feature.

---

## Finding 2 — the leap rule is derived, the leap *placement* is not, and nothing declares it

Rule K derives *how many* intercalary days a cycle holds: Earth's `31/128`,
Mars's `45/76`, Mercury's `1/2`. That is a fact about two periods.

*Which* day is intercalated is not. `BodyCalendar::days_before_year` computes

```
    days_before(y) = y·whole_days + ⌊y·p / q⌋
```

which spreads the intercalations as evenly as integers allow. That is a
defensible choice and it **is a choice**: the Gregorian calendar does not do it,
clumping its leap day at the end of February, and a calendar that placed them
all at the end of the cycle would satisfy the same `LeapRule` and disagree about
which absolute instant is day 366.

So a second conforming implementation could reproduce every conformance vector,
every convergent table and every leap rule, and still disagree with this one
about a date. **The placement is load-bearing and undeclared** — the exact shape
of defect the last six cycles have been finding, in a place nobody has looked.

The implementation has a one-line comment saying the distribution is even.
Nothing states it as a rule, no test pins it against an alternative, and
`spec/RULES.md` does not mention it.

---

## What else is missing, in descending order of consequence

**No era or epoch structure.** `DerivedFields::year` is "local years since the
anchor, 1-based". A real calendar names an era and often counts from something
other than its own anchor; there is nowhere to declare that, and no way to
express "year 1 of this reckoning is the anchor + N".

**No names for anything.** A cycle yields a *position*, not a month name.
Weekday is absent by §15.3 and correctly so, but month names are the most
recognisable part of any calendar and there is no slot for them. Under Rule N
names are display-only and locale-scoped, so this is a locale-table question —
which loops back to Finding 1, because a locale table is also compiled in.

**No authoring or validation path.** Nothing answers *are these parameters
sane* before they are committed: no `ucal cal validate`, no way to see the
derived rule for a body that does not ship, no dry run.

**Anchors remain expensive and scarce.** Two of seven calendars have one, and
[`D5-titan-anchor.md`](D5-titan-anchor.md) established what it costs to get a
third honestly: the rotational elements are published and the mean-solar-time
convention is not.

**Sub-day structure: answered, negatively.** [`W4`](W4-two-ladders.md) step 1
placed every unit of every shipped body on the universal ladder and found them
all on `T1` or `T2`. There is no local second and none is derivable, because
nothing below the day is a period of the body at all.

---

## The plan

Four stages. Each is independently useful, each has a condition for stopping,
and the first two are corrections rather than features.

### X1.1 — record the two findings (a cycle's worth of writing, no code)

A spec delta for §15.1 saying the loader is **not implemented**, and a rule or
delta declaring the leap placement. Neither needs the feature built; both stop
the project asserting things that are not so.

`SPEC-DELTAS.md` currently has no class for *"the RFC is right and the
implementation does not do it"*. Adding one — `UNIMPLEMENTED`, beside
`CORRECTION`, `AMENDMENT`, `EDITORIAL` and `WITHDRAWN` — makes the gap
countable and gives `check-docs` something to check.

**Stop if:** the survey turns up more unimplemented normative requirements than
can be honestly recorded in one cycle. Then the deliverable is the survey, and
the delta class, and a list.

### X1.2 — pin the placement (small, and overdue)

Declare the even distribution as the rule, and add a test that a *different*
placement with the same `LeapRule` produces different fields — so the
convention is pinned by something other than the code that implements it.

**Stop if:** the alternative placement turns out to be indistinguishable, which
would mean the convention is not load-bearing after all. Recorded either way.

### X1.3 — the loader (§15.1), the substantial one

Body files and anchor files, strict, versioned independently, `UCAL-E0012` for
an unknown key — which finally gives that code a caller.

The hard parts are not parsing:

- **Every parameter carries four Rule C obligations** — value verbatim, unit,
  epoch, validity window, citation. A file format that lets any of them be
  omitted has re-introduced the uncited constant the whole project exists to
  refuse.
- **Anchors must stay separable.** §15.1's reason for two files is that
  parameters and anchors are revised for different reasons on different
  schedules; one file would lose that.
- **Loading must not become a way to invent an anchor.** GE-3's kill criterion
  forbids narrowing a window by assumption, and a file is a much easier place to
  do it than a Rust constant.

**Stop if:** the format cannot express the Rule C obligations without being
harder to write than the Rust it replaces. Then the honest answer is that bodies
are code, §15.1 was wrong, and the delta from X1.1 changes class from
`UNIMPLEMENTED` to `CORRECTION`.

### X1.4 — authoring: `ucal cal derive <file>`

Read a body file, print the derived rule, cycles and convergents, and say what
is missing — almost always the anchor. This is what makes the mechanism usable
by someone who wants to *try* a body rather than commit one.

**Stop if:** X1.3 lands and nobody uses it, which is the same condition every
other feature here has been unable to test.

---

## What this does not fix

Seven cycles of asking have produced no users
([`CONTACT.md`](../CONTACT.md)). A loader makes it *possible* for someone else
to derive a calendar; it does not make anyone want to. X1.1 and X1.2 are worth
doing regardless, because they correct claims the project is currently making
about itself. X1.3 and X1.4 are worth doing only if the answer to "who would
author a body file" is not "nobody" — and today it is.

That is not an argument for skipping them. It is an argument for doing them in
that order, so the parts that are true independently of an audience land first.
