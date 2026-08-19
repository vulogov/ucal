# V1 — an audit of every check this project claims to run

**Status: complete. Every row below was established by running something, and
the probes that established it are committed as tests.**

---

## The question

1.5.0 found that `check-docs`' worked-examples check had been **correct and unrun
in CI for four releases**, printing `--    worked examples not checked` and
moving on. That is a different failure from the one this project usually finds. A
*claim with no mechanism* is findable by reading the claim. A **mechanism with no
wire** is not: it exists, it is written correctly, it passes review, it prints
something on every run, and it examines nothing.

So: how many others are there, and how would anyone know?

Two ways a check can be hollow, and they need separate tests:

1. **It is not wired.** CI never invokes it, or invokes it on a configuration
   where it has nothing to say.
2. **It is vacuous.** It runs, examines a population of zero, finds no faults in
   it — correctly — and reports success.

## Method

For the second, the checks were pointed at a **skeleton workspace**: every file
they read, present and empty. A missing tree is not the interesting test — they
all fail on that, which is right. A file that exists and yields nothing is what
an empty population looks like in practice, and it is what a moved directory, a
renamed section or a rewritten template produces.

The probes are `xtask/src/citations.rs::vacuity_probe` and
`xtask/src/lint.rs::vacuity_probe`. They **pin the current finding exactly**, so
that closing one of these holes breaks the probe and forces this page to be
updated. A finding that can be fixed without anyone noticing is a finding that
comes back.

---

## Finding 1 — an unknown subcommand is silently the harness

The worst one, and it was not on the list of things to look for.

```
$ cargo run -p xtask -- chekc-docs
UC-P0 constants harness — RFC UCAL-1, profile UC-1
  96 passed, 0 failed
  UC-P0 exit criterion met.
$ echo $?
0
```

`xtask`'s dispatch is a sequence of `if mode == "..."` and then falls through to
the harness. **Any unrecognised argument runs the constants harness, prints a
success banner and exits 0.**

A typo in a workflow step — `check-doc`, `verify-vector`, `lints` — produces a
green CI run in which the intended check never executed and a different one
reported success in its place. Nothing anywhere would show it. This is the exact
shape of the defect the cycle is named for, sitting in the dispatcher of the tool
that runs all the other checks.

**Fix (V2):** an unknown mode is an error and a non-zero exit, and the harness
gets a name of its own.

---

## Finding 2 — three `check-docs` checks pass on an empty population

Every check in `citations.rs` ends the same way:

```rust
if bad.is_empty() { Ok(count) } else { Err(bad) }
```

The count is printed. **It is never examined.** `ok    citations resolve against
spec/ (0 distinct)` reads exactly like a pass, and on a skeleton workspace three
of the six produce one:

| check | on a skeleton | why |
|---|---|---|
| `citations resolve against spec/` | **passes**, 0 examined | no citations found, so none dangle |
| `Documentation/CLI.md covers the CLI surface` | **passes** | no commands parsed out, so none are undocumented |
| `every standing spec delta is applied` | **passes** | no deltas found, so all of them are applied |
| `contact materials quote vectors.json` | fails | needs constants it cannot find |
| `the signing key is published identically` | fails | needs a key to compare |
| `CI runs the documented verification block` | fails | needs a command list |

The three that fail do so because they need a *specific value* to exist, not
because anyone decided they should. The distinction is an accident of what each
happens to look for.

**Fix (V2):** a floor. Each check declares the smallest population it can be
believed at — there are at least a hundred citations, at least twenty commands,
at least twenty deltas — and reports `FAIL` below it. The floor is not a
guess about the future; it is a statement that a number this small means the
check has stopped reading rather than that the problem was solved.

---

## Finding 3 — ten lints pass on a workspace that does not exist

`lint::run` returns early when there is no `crates/` directory:

```rust
let crates_dir = workspace_root().join("crates");
if !crates_dir.exists() {
    return v;          // empty
}
```

An empty violation list is indistinguishable from a clean workspace, so **all
ten lints report `0 violations` and exit 0** having read nothing. The same
holds for a `crates/` that exists and is empty, which is the likelier accident: a
path that resolves, to the wrong place.

All ten go quiet together — `float-free`, `no-wrapping-arithmetic`,
`core-names-no-foreign-unit`, `datum-no-overclaim`, `rounding-is-declared`,
`public-type-is-classified`, `no-indent-in-literal`, `version-lockstep`,
`no-panic-in-cli`, `codes-have-raisers` — and the output is the same output as a
clean run.

**Fix (V2):** report the population — *`0 violations across 148 files`* — and
fail below a floor.

---

## Finding 4 — the constants harness has no floor either

```
  96 passed, 0 failed
  UC-P0 exit criterion met.
```

The exit is `if failed > 0 { exit(6) }`. A run with **zero checks** prints
`0 passed, 0 failed` and meets its exit criterion. This is less reachable than
the others — the checks are hardcoded calls rather than a directory walk — but it
is the same missing statement, and the harness is the thing the whole
specification's numbers rest on.

**Fix (V2):** the same floor.

---

## Finding 5 — the wall clock's tests had never run in CI

Found at 1.6.0's opening, before this audit began, and fixed there.

`cargo test --workspace` does not reach them: `tui` is not a default feature, so
`#![cfg(feature = "tui")]` compiles the file away and 22 tests report as zero.
They had been **built** in CI from the day they were written — the `features`
workflow builds `-p ucal --features tui` — and never run. A test that compiles is
not a test that runs, and a feature-gated test file is invisible to the command
that is supposed to run everything.

`check-docs`' CI-covers-procedure check caught it the moment 1.6.0's verification
block listed the command, which is that check doing exactly its job.

**Already fixed.** CI now runs `cargo test -p ucal --release --features tui` and
reports 41 tests in the suite it had been skipping.

---

## What is wired, and what is not

| invoked by CI | not invoked by CI |
|---|---|
| the constants harness (default mode) | `check-links` |
| `lint` | `gen-docs`, `gen-schema`, `gen-examples` |
| `check-docs` | `publish` |
| `verify-vectors` | |

The right-hand column is fine and should be recorded as fine rather than left to
be rediscovered:

- **`gen-*` are generators, not checks.** `check-docs` verifies their output, so
  a stale generated file fails CI even though the generator never runs there.
- **`publish` is a release action.**
- **`check-links` is deliberately opt-in.** It makes network requests, and a CI
  job that fails when somebody else's web server is down is a job that trains
  people to ignore red. The cost is real and should be stated plainly: **nothing
  automatic notices when a cited URL rots.** Two had already rotted when the
  command was written. The mitigation is that it is run at each release, which is
  a procedure and not a mechanism.

---

## The count

**Twenty-two named checks**, counted from the code rather than from memory: one
constants harness, **ten** lints (`xtask lint`'s `LINT_NAMES`), nine
`check-docs` checks, one vector verifier, one link checker.

**Eight are sound under every condition tested.** Fourteen are not:

| | count | finding |
|---|---|---|
| the ten lints | 10 | pass on a workspace that does not exist |
| `citations`, `cli-docs`, `deltas-applied` | 3 | pass on an empty population |
| the constants harness | 1 | no floor; `0 passed, 0 failed` meets its exit criterion |
| **total affected** | **14** | |

Plus two that are not checks and affect all of them:

- **an unknown subcommand runs the harness and exits 0**, so a typo in a workflow
  step is a green run of the wrong check;
- **the wall clock's tests were never run in CI** — found at 1.6.0's opening and
  fixed there.

Every one of the fourteen was previously unknown.

Two arithmetic corrections were needed while writing this section, both because
the first numbers were recalled rather than counted: the lints are ten and not
twelve, and `xtask lint` prints five summary lines for them because several
lints share one. An audit that miscounts its own subject is not worth much, so
the counts above come from `LINT_NAMES` and from running the commands.

## What the audit says about the method

The premise was that hollow checks would be rare and hard to find. They were not
rare. What made them invisible is that **every one of them prints something that
looks like work**: a count, a banner, a list of rule names. The output of a
vacuous check and a thorough one are the same sentence with a different number
in it, and nobody reads the number.

That is the argument for V2 being a floor rather than a review. A person reading
`ok    citations resolve against spec/ (0 distinct)` sees a pass. A machine
comparing `0` against `100` does not.

**One thing this audit could not test.** Whether a check is *correct* — whether
it would catch the defect it exists for — is not answerable by pointing it at an
empty directory. That needs a defect injected per check, which this project has
done ad hoc and never systematically. Six were verified strict this way during
1.5.0. Fifteen have not been.

That is not V2's job and it should not be smuggled into it. It is the next
question, and it is a larger one.
