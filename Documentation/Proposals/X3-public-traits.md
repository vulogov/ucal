# X3 — every public trait, not one

**Status: complete. Four public traits, four downstream implementors in the test
suite, each verified to fail on the change `cargo semver-checks` passes.**

---

## The question

V1 Finding 6 established that `cargo semver-checks` **does not catch a required
method added to a public trait**, confirmed against 0.50.0 with a real external
crate: it compiles against 1.5.0, fails with `not all trait items implemented`
after the change, and the tool reports `no semver update required` for the
difference.

That is the mechanism behind `STABILITY.md`'s central promise — *within `1.x`,
nothing that compiled stops compiling*. 1.6.0 answered it for `Profile` with one
compile-pass fixture and said plainly what that did not cover:

> **This does not generalise.** One fixture covers one trait. Every other public
> trait in the workspace has the same exposure and no such fixture, and the
> honest statement is that `semver-checks` is trusted for the rest.

This closes that.

## The enumeration

Four public traits in the workspace, and the interesting question about each is
not whether it *is* implemented outside but whether it *can be*.

| trait | crate | implementable outside? | how |
|---|---|---|---|
| `Profile` | `ucal-core` | **yes** | must delegate `bridge()`; `Bridge` has no constructor |
| `TickInt` | `ucal-core` | **yes** | supplies its own integer and its own `Wide` |
| `CalendarIdentity` | `ucal-core` | **yes** | two required methods, one defaulted |
| `LegacyCalendar` | `ucal-civil` | **yes** | must delegate `tables()`; `DeclaredTables` has no constructor |

**All four.** The scope's stop condition — *if most public traits are
unimplementable outside the workspace, that is a finding about the API's shape
and belongs in `ROAD-TO-2.0.md`* — does not trigger. Each was established by
building an actual external crate against the workspace and compiling an
implementation, not by reading the signatures.

### The pattern in two of them

`Profile` and `LegacyCalendar` both require returning a type an outsider cannot
construct: `Bridge` and `DeclaredTables` are `#[non_exhaustive]` with public
fields and no constructor. An implementor must **borrow** one from a shipped
instance — delegate `bridge()` to `UC1::bridge()`, `tables()` to
`Gregorian.tables()`.

That narrows what a downstream implementation can *be*: a second profile cannot
invent a bridge, and a third legacy calendar cannot declare its own month
lengths. It does **not** seal the trait, and the difference was nearly missed
once already — V1 Finding 6 records the near-miss, where *`Bridge` has no
constructor* looked like *the trait cannot be implemented* right up until someone
checked how the in-crate implementor gets one.

Whether that narrowing is intended is a separate question. It is recorded here
rather than answered: `DeclaredTables` with no constructor means §8.6's "legacy
calendars are preserved for interoperation" admits no calendar this crate does
not already ship, which is a real limit on a documented extension point.

### The surprising one

**`TickInt` is implementable from outside.** Rule B makes the value width a
wire-format commitment and the project ships exactly two backends, refusing to
compile both at once — but nothing seals the trait, and an outside crate can
supply its own integer type and its own `Wide`.

Whether a third backend *should* be possible is a design question this does not
answer. What the fixture does is make the current answer visible, so that
changing it becomes a decision rather than a side effect.

## The mechanism

A trybuild fixture is compiled **as its own crate** depending on the crate under
test, which is a downstream implementor's position exactly. So each trait now has
one kept in the suite:

- `ucal-core/tests/compile_pass/profile_is_implementable.rs` (1.6.0)
- `ucal-core/tests/compile_pass/calendar_identity_is_implementable.rs`
- `ucal-core/tests/compile_pass/tick_int_is_implementable.rs`
- `ucal-civil/tests/compile_pass/legacy_calendar_is_implementable.rs`

**Each verified strict**, by adding a required method to its trait and confirming
the fixture fails with the outsider's own error:

```
ucal-core   CalendarIdentity  -> 1 × "not all trait items implemented"
ucal-core   TickInt           -> 2 ×
ucal-civil  LegacyCalendar    -> 2 ×
```

The `TickInt` fixture's arithmetic is deliberately wrong — `wide_quot_rem`
returns zero — because the property under test is that the trait can be
*implemented*, not that this is a usable backend. A correct one would be a second
`ucal-core`.

**One thing the fixtures found on the way in.** `xtask lint` walks `crates/` and
skipped `compile_fail` but knew nothing of `compile_pass`, which did not exist
until now — so Rule O objected to a `wrapping_mul` in the new fixture. The lint
was right and the exclusion was missing: a trybuild fixture is a separate crate
compiled to check that something builds, not shipped code, and a deliberately
useless backend is allowed a wrapping multiply. `compile_pass` is excluded now,
and the fixture uses `checked_mul` regardless, so it does not model a Rule O
violation as exemplary.

## What this still does not cover

**Public traits are not the only thing `semver-checks` can miss.** X3 fixes the
one failure V1 measured, on the four traits it applies to. Nothing here says the
tool is complete about enums, struct fields, generic bounds or lifetimes, and
nothing tests that it is.

The honest position: `semver-checks` runs on every push and is trusted, with one
known exception now covered by four fixtures. Whether there is a second exception
is unmeasured, and finding out would mean building the same kind of external
probe for every category of breaking change — which is a larger exercise than
this cycle, and a fair candidate for a later one.
