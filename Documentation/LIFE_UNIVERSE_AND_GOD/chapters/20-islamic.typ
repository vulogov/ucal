#import "../design.typ": *

#chapter(number: 20, title: "Islamic")

Chapter 8 said that every body parameter carries a validity window, and that
evaluating outside it warns rather than extrapolating. That looked like ordinary
engineering caution. This chapter is about the position it turns out to encode, and
about the fact that the artifact took a side in a live metaphysical dispute without
noticing.

#section("What the direction holds")

#claim("tradition")[
  Kalām atomism holds that the world is composed of indivisible atoms and their
  accidents, and — the part that matters here — that *accidents do not endure for two
  moments*. What appears as a persisting quality is God re-creating it instant by
  instant. Time itself is composed of indivisible moments.

  Al-Ghazālī, *Tahāfut al-Falāsifa* XVII, argues that "the connection between what is
  habitually believed to be a cause and what is habitually believed to be an effect is
  not necessary." Fire does not burn cotton; God creates burning in the cotton at the
  moment of contact. The two are conjoined constantly, not necessarily.

  His term for the regularity is *ʿāda* — custom, habit. Divine custom is utterly
  reliable in practice, and it grounds ordinary knowledge, but it carries no necessity
  and God is not bound by it.

  Ibn Rushd replies in *Tahāfut al-Tahāfut* that to deny that things have natures from
  which their effects follow is to deny the possibility of knowledge. If fire has no
  nature that burns, the word "fire" names nothing, and demonstration collapses into
  the observation of sequences.
]

#section("Which rule it illuminates")

N1, the refusal to claim the tick is a quantum of time — which chapter 2 declined an
Epicurean position with, and declines a Kalām one with equally. And Rule C, on body
parameters carrying `rate` and `valid`.

#section("The convergences")

#subsection("A validity window is ʿāda, compiled")

This is the convergence, and it is exact enough to be startling.

Consider what a validity window actually asserts. Earth's rotation is measured near
J2000 and found to be lengthening at about 1.8 ms per century. The system records the
value, the rate, and a window — and *inside* the window it computes confidently, while
*outside* it warns rather than extrapolating.

What is the epistemic content of that?

It is: *the regularity has been observed here, and I will assert it here, and I decline
to assert that it holds beyond where it was observed.* Not because a different law is
expected outside — no alternative is proposed — but because the warrant runs out where
the observation does.

#claim("interpretation")[
  That is Ghazālī's position with the theology removed. Practical certainty inside the
  range of custom; no claim of necessity; no assertion that the regularity is
  guaranteed to continue.

  `UCAL-W0003` — the warning a parameter emits outside its window — is *ʿāda*
  compiled. Whatever the author believed about causation while writing it, the
  epistemic posture the code takes is the one Ghazālī argued for.

  I want to be careful about how much this is worth. It does not show Ghazālī was
  right about causation. What it shows is that a working engineer, forced by the
  problem to be precise about the scope of an empirical claim, arrives at a
  distinction — *reliable here, not thereby necessary* — that was worked out in
  Baghdad in the eleventh century for entirely different reasons.
]

#subsection("Atomism, declined twice")

Chapter 2 declined to call the tick a quantum of time, and noted that asserting
otherwise would have joined Epicurus against Aristotle.

The Kalām position is the more thoroughgoing one: not merely that time has least
parts, but that persistence itself is not a fact about things — that endurance is
re-creation.

N1 declines this too, and it is worth noting what the declining costs. A system whose
finest unit is a tick and whose values are integers is *formally* very close to a
world of temporal atoms. If the author had wanted a metaphysical warrant for the
design, this tradition offers one, fully worked out, with a serious apparatus behind
it.

It is refused for the reason chapter 2 gave: the discreteness is a fact about the
instrument's resolution, and taking a metaphysical position as a side effect of
choosing an integer type is not a respectable way to hold one.

#conflict[
  *A guaranteed drift bound is not something this tradition can grant.*

  Chapter 15 reported that Earth's `31/128` intercalation drifts one day in 400,000
  years, and chapter 16 qualified that: the guarantee holds under parameters that will
  not hold for 400,000 years.

  Under Ghazālī's seventeenth discussion the qualification is far more severe. If an
  orbital period is a *habit* rather than a nature, then there is no fact about what
  Earth's orbit will do that could underwrite a guarantee over four hundred millennia.
  The regularity holds because God customarily maintains it, and the only honest bound
  would be *until God wills otherwise*.

  Which is not a bound. It has no number in it, and a drift-bound parameter that takes
  it would not select a convergent.

  So the mechanism's central promise — *choose the rule that guarantees this accuracy
  for this long* — presupposes that periods have natures from which their behaviour
  follows. *The crate's ontology is Rushdian.* It took a side in a dispute that is
  still live, and it did so silently, as an implementation consequence.
]

#section("What it changes")

Here is the finding, and it is the most peculiar thing Part VI turned up.

#claim("interpretation")[
  The artifact's *ontology* is Rushdian and its *epistemology* is Ghazālian.

  What it computes assumes natures. Continued-fraction expansion over an orbital period
  yields a guarantee about four hundred thousand years, and that guarantee means
  nothing unless the period is the kind of thing that has a nature from which its
  future follows.

  What it *claims* assumes custom. The validity window says: I assert this where it was
  observed and not beyond. `UCAL-W0003` is exactly the refusal to project a regularity
  past its warrant.

  So the system computes as though Ibn Rushd were right and reports as though Ghazālī
  were.
]

Is that coherent, or merely convenient?

#claim("interpretation")[
  The case for coherence: the two do different jobs. The ontological assumption is
  *internal to a conditional* — given that the period is thus, the rule drifts thus —
  and conditionals do not commit you to their antecedents. The epistemic caution is
  about *asserting* the antecedent, which the system declines to do outside the window.
  Read that way there is no contradiction, only a careful separation between what
  follows from what.

  The case for convenience: that reading is available after the fact and was not the
  reason for either choice. The natures came from wanting a guarantee; the caution came
  from knowing that parameters drift. Nobody weighed them against each other. A
  consistency discovered afterwards, which nobody designed and which happens to make
  the design defensible, should be suspected exactly as chapter 10's flattering
  sentence should have been.

  I think the first reading is right and I am aware that I would. What can be said
  without prejudice is that the artifact contains a metaphysical commitment its
  specification never declares, and that Rule F — which requires the *frame* to be
  declared — has no analogue requiring this to be.

  That is a gap in the rules, found by a tradition, and this chapter is the whole
  record of it.
]

#recap((
  [Kalām atomism: indivisible moments, accidents that do not endure, persistence as re-creation. Ghazālī XVII: causal connection is custom, not necessity. Ibn Rushd: deny natures and you deny knowledge.],
  [A validity window is *ʿāda* compiled — reliable where observed, no claim of necessity beyond. `UCAL-W0003` is that posture in code.],
  [N1 declines Kalām atomism as it declined the Epicurean version, and refuses the metaphysical warrant this tradition would have supplied.],
  [*Conflict:* a guaranteed drift bound over 400,000 years presupposes that periods have natures. Under Ghazālī the only honest bound is "until God wills otherwise", which is not a bound. The crate's ontology is Rushdian and it never argued for it.],
  [*What changes:* the artifact computes as though Ibn Rushd were right and reports as though Ghazālī were — and the rules require the *frame* to be declared but contain nothing requiring this commitment to be.],
))
