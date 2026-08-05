#!/bin/sh
# GE-U4's cheapest test: the walk, recorded, without building the navigator.
#
# The proposal names three things that would make a tier-scale TUI not worth
# building, and says of the third:
#
#   "If it needs to be interactive to work at all. That is a hypothesis, not a
#    finding. An animation -- the same walk, recorded -- would test the same
#    idea at a fraction of the cost, and if the walk works recorded, the
#    interactivity was never the load-bearing part."
#
#   "The third is the one I would test first if this were resumed."
#
# This is that. Forty-five frames, one per tier, each showing one step of the
# ladder instead of the whole grid at once -- so a reader travels T-12 to T32
# rather than reading a table of it.
#
# What it deliberately is not
# ---------------------------
# Not a program that computes anything. Every value comes from
# `ucal ladder --json`, which is the same Doc `ucal ladder` renders, so this
# cannot drift from the table it is being compared against. The proposal's own
# constraint -- "it renders Docs, not its own data" -- applies to the cheap
# version too, and is why this is shell and not a crate.
#
# Not the experiment. The kill criterion needs two readers who have not read
# this repository, one given `ucal ladder` and one given this. The author cannot
# run it; what the author can do is make it runnable, which is all this is.
#
# Usage
# -----
#   ./GE-U4-walk.sh            step on Enter, which is the honest comparison
#   ./GE-U4-walk.sh --auto     play at 0.6s per frame
#   ./GE-U4-walk.sh --auto 0.2 play at a chosen delay
#
# Run it from anywhere; it finds the workspace itself.

set -eu

delay=""
case "${1:-}" in
  --auto) delay="${2:-0.6}" ;;
  "") ;;
  *) echo "usage: $0 [--auto [seconds]]" >&2; exit 2 ;;
esac

root=$(unset CDPATH; cd -- "$(dirname -- "$0")/../.." && pwd)

# One invocation. Forty-five frames out of one Doc, so every frame is the same
# ladder at the same revision -- a per-frame invocation would let the tiers come
# from forty-five separate runs, which is the kind of seam this project spends
# its time closing.
json=$(cd "$root" && cargo run -q -p ucal -- ladder --json)

# The ladder descends T32..T-12; a walk climbs. `tier`, `exponent`, `name` and
# `beats` each occupy their own line in the generated JSON, in that order.
# BSD sed has no `\?`, so the tier is matched as a character class rather than
# an optional sign -- this script runs on the author's laptop and on CI.
tiers=$(printf '%s\n' "$json" | grep -E '^    "T-?[0-9]+": \{' | sed 's/.*"\(T[-0-9]*\)".*/\1/')
exps=$(printf '%s\n'  "$json" | grep -E '^      "exponent"' | sed 's/.*: "\(.*\)".*/\1/')
names=$(printf '%s\n' "$json" | grep -E '^      "name"'     | sed 's/.*: "\(.*\)".*/\1/')
# Ticks, not beats. The first attempt walked the `beats` column and the first
# twelve frames all read `0` -- every tier below the beat is under one beat, and
# six fractional digits cannot show it. In ticks each tier is exactly 5^e, an
# integer, so the number grows from one digit to a hundred and fifty-four and
# the growth is the thing being shown.
ticks=$(printf '%s\n' "$json" | grep -E '^      "ticks"'    | sed 's/.*: "\(.*\)".*/\1/')

n=$(printf '%s\n' "$tiers" | wc -l | tr -d ' ')
[ "$n" -gt 0 ] || { echo "no tiers parsed from ucal ladder --json" >&2; exit 1; }

field() { printf '%s\n' "$2" | sed -n "$1p"; }

# The ladder is emitted high-to-low, so frame i counts back from the end.
i=1
prev_name=""
while [ "$i" -le "$n" ]; do
  k=$((n - i + 1))
  tier=$(field "$k" "$tiers")
  exponent=$(field "$k" "$exps")
  name=$(field "$k" "$names")
  shown=$(field "$k" "$ticks")
  digits=${#shown}
  if [ "$digits" -eq 1 ]; then unit=digit; else unit=digits; fi

  printf '\033[2J\033[H'
  printf '  step %s of %s\n\n' "$i" "$n"
  printf '  %-6s %s\n' "$tier" "$name"
  printf '  one %s is 5^%s ticks -- %s %s:\n\n' "$tier" "$exponent" "$digits" "$unit"
  # Wrapped at 72 so a 61-digit number is a *shape* on the screen rather than a
  # line that scrolls off it. Watching the shape grow is the whole exercise.
  printf '%s\n' "$shown" | fold -w 72 | sed 's/^/    /'
  printf '\n'
  if [ -n "$prev_name" ]; then
    printf '  x 3125 from %s\n' "$prev_name"
  else
    printf '  the floor: one tick\n'
  fi

  if [ -n "$delay" ]; then
    sleep "$delay"
  else
    printf '\n  [Enter] '
    read -r _ || break
  fi
  prev_name="$tier"
  i=$((i + 1))
done

printf '\n  Forty-five steps, each x 3125. T-12 to T32.\n'
printf '  The same grid that "ucal ladder" prints as a table.\n\n'
