#import "../design.typ": *

#appendix(letter: "B", title: "Glossary")

Terms as this book uses them. Where a word is a technical term of the software,
the chapter that introduces it is given; where it is a technical term of a
tradition, the chapter that borrows it.

Two words carry a warning. *Legacy* and *declared* are classifications and not
judgements, and chapters 8 and 22 both say so, because they read like judgements
and will be taken for them.

#section("The software")

/ anchor: The phase of a calendar — where its count begins, as opposed to how
  long its units are. Not derivable from any period, so declared, cited and
  interval-valued, with its absence an error rather than a default. Chs. 8, 16.

/ beat: $5^60$ ticks, about 46.762 ms. The reference rung of the tier ladder and
  what §0.5 calls the *universe second*: a unit of human-noticeable size with no
  Earth content. Ch. 4.

/ `BIG_BANG_CLAIM`: The published identification of tick 0 with the FLRW
  $t arrow.r 0$ limit, with its ±0.020 Gyr uncertainty, recorded as metadata. A
  `SignedWindow`, and therefore not computable with. Ch. 12.

/ bridge: The single declared constant through which a foreign unit enters and
  leaves. `SECOND`, an exact integer of ticks. Conversion in is multiplication
  and never rounds; conversion out is division and is the only place a rounding
  mode is chosen. Ch. 7.

/ convergent: A truncation of a continued fraction — the best rational
  approximation at its denominator. Intercalation rules are derived by walking
  them. Chs. 8, 10, 15.

/ datum: Tick 0. A *stipulated* reference point, conventionally identified with
  the FLRW $t arrow.r 0$ limit; not a measurement and not an observed event.
  Chs. 3, 11.

/ declared: Of a rule or constant: supplied rather than computed. The opposite of
  *derived*, and not its inferior — chapter 22's Julian retention is the case
  where declaring is the better choice. Chs. 8, 22.

/ derived: Of a rule: produced by the mechanism from cited parameters. The Julian
  leap rule is derived; the Gregorian is not. Chs. 8, 10.

/ díastēma: See the traditions below. The term chapter 22 supplies for what every
  quantity in this system measures.

/ digit form: The base-5 notation, five digits to a group, anchored at T32.
  Canonical for parsing and sorting. Ch. 6.

/ human form: The decimal tier-group notation, anchored at T0. Canonical for
  statements about time. Ch. 6.

/ instant: A point in absolute time — an unsigned integer count of ticks since
  the datum. Ch. 5.

/ interval extension: The bounding of a function over a panel by evaluating its
  monotone parts at the panel's ends. Used on every panel of the cosmological
  quadrature, because the integrand is not monotone. Ch. 5.

/ legacy: A calendar whose authority comes from declared tables rather than from
  derivation. A classification of *where the authority is*, not a verdict on the
  calendar. Chs. 8, 22.

/ profile: A named, versioned, type-bound set of declared constants. Values from
  two profiles cannot be compared, and the text forms carry the tag so the
  refusal is visible. Chs. 6, 19, 21.

/ resonance: A convergence that is genuine, striking, and proves nothing.
  Labelled as such, and never a premise. Ch. 18's Archimedes figure is the only
  one in the book.

/ `SignedWindow`: The type of `BIG_BANG_CLAIM`. Two fields, no arithmetic
  operators, no conversion to any computable type, and three compile-fail tests
  that enforce the absence. Ch. 12.

/ tick: The Planck time, about $5.39 times 10^(-44)$ s. The atomic unit — the
  resolution floor of the instrument, and *not* a claim that time is discrete.
  Ch. 2.

/ tier: A rung of the ladder, $5^(60 + 5k)$ ticks. Each is 3125 of the one below,
  which is exactly five base-5 digits. A tier's canonical identity is its
  exponent; its name is display only. Ch. 4.

/ UCID: A 52-character Crockford base-32 identifier, defined below $2^256$. Sorts
  chronologically, contains no randomness, and is not a UUID. Ch. 6.

/ validity window: The range over which a body parameter is asserted. Outside it,
  `UCAL-W0003` rather than confident extrapolation. Chs. 8, 16, 20.

#section("The traditions")

/ ἀνθυφαίρεσις (anthyphairesis): Reciprocal subtraction, Euclid *Elements* X.2.
  The same procedure as continued-fraction expansion, and therefore the same
  procedure as the intercalation mechanism. Ch. 18.

/ ἀριθμός (arithmos): In Greek usage, a definite number of definite things —
  not a number in the modern abstract sense. Klein's distinction, without which
  chapter 18's conflict is invisible. Chs. 18, 26.

/ αἰών (aion): In Maximus, the mode of created being that has a beginning
  without that beginning being a temporal position — a "before" that is not a
  "when". What UC-Θ would require. Chs. 12, 22.

/ διάστημα (diastema): In Gregory of Nyssa, interval or extension: the mark of
  createdness. God is ἀδιάστατος, without interval. Everything this system
  measures is διάστημα, which is why the instrument reaches exactly as far as it
  does. Chs. 22, 27, 30.

/ ἐκπύρωσις (ekpyrosis): The Stoic cyclical conflagration — a cosmology in which
  no datum is possible, because there is no first cycle. Ch. 18.

/ essence and energies: Palamas' distinction between what is unknowable in God
  and what is genuinely known and participated. The strongest precedent the
  book's thesis receives. Ch. 22.

/ ḥelek: 1/1080 of an hour, in the Hebrew calendar. Chosen for divisibility, as
  `SECOND` was chosen to be a multiple of $10^30$. Ch. 19.

/ имяславие (imyaslavie): The dispute over whether the divine name participates
  in what it names. Condemned by the Synod in 1913; defended by Florensky and
  Losev. Rule N sides with the Synod. Chs. 22, 25.

/ molad tohu: "The new moon of chaos" — the Hebrew calendar's computational
  epoch, placed about a year before the creation it anchors and named for its
  own emptiness. Rule Q's content, nineteen centuries early. Ch. 19.

/ regulative and constitutive: Kant's distinction between a principle that
  directs inquiry and one that describes an object. Transcendental illusion is
  the slide from the first to the second. Chs. 12, 24.

/ transcendental illusion: In Kant, an appearance that is natural, unavoidable,
  and persists after diagnosis. The astronomer knows the moon is not larger at
  the horizon and sees it larger anyway. Ch. 32.

/ ʿāda: In al-Ghazālī, divine custom — the regularity that grounds practical
  certainty without carrying necessity. What a validity window encodes. Ch. 20.

#section("This book's own apparatus")

/ conflict: The section every chapter of Part VI must contain, naming where the
  direction and the artifact disagree. Fixed in advance so none could be
  omitted. Ch. 18's opening callout.

/ deletion test: A-P8. Strip every *interpretation* and *resonance* block,
  compile, and confirm no surviving prose refers into a deleted one. Run by
  `deletion-test.py`. Ch. 0, and the book's README.

/ locus: The chapter that uses a source. Given in Appendix A because the prose
  carries no inline citation marks. App. A.

/ null result: What a sample failed to establish, reported as plainly as what it
  did. Ch. 28.
