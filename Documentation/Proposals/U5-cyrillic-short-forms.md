# U5 — Cyrillic short forms for the `ru` locale

**Status: decided and implemented.** T3 is `пролёт`, short form `пр`.

Cyrillic short forms for the named tiers, scoped to the `ru` locale. Not a
universal symbol layer — that question is settled in `docs/TIERS.md`, and the
answer is `T[k]` and `5^e`.

---

## Why the locale is the right scope

The `ru` locale already ships the full Cyrillic ladder — глубь, дрейф, срок,
обход, дуга, бой, мерцание, блик, искра, тик — and Rule N already sanctions it
as display-only. Short forms scoped to that locale inherit the sanction, and
inherit something better: **locale-invariance holds by construction rather than
by care.** The locale is stated, so a form that means one thing under `--locale
ru` cannot silently mean another elsewhere.

That is the whole difficulty with a universal scheme, and it is why this one is
scoped rather than promoted. Two further objections apply only to the universal
case, and are recorded because they would otherwise be rediscovered:

**Appendix D forbids national content in names.** That criterion is what
produced *drift* and *glint* instead of *aeon* and *epoch*. A layer adopted in
honour of a person or a tradition is commemorative content by definition, so it
would be overruled by the rule that shaped the English ladder. Inside `ru` the
question does not arise: a Russian locale carrying Russian short forms is a
locale doing its job.

**Homoglyphs are already a settled defect class here.** The UCID alphabet is
Crockford base-32 with `I`, `L` and `O` removed because they are confusable with
`1` and `0`. Visual ambiguity in a parse surface is something this project
designs out rather than documents around. Cyrillic against Latin is that problem
at scale.

## The homoglyph analysis

Twelve lowercase Cyrillic letters are pixel-identical to Latin ones in most
terminal fonts: **а с е о р х у к м н в т**. A short form built only from those
is indistinguishable from Latin text, and the `--locale ru` tag is the only
thing telling a reader which alphabet they are looking at.

Reproduce it:

```
python3 - <<'PY'
homo = set("асеорхукмнвт")
for w in ["глубь","дрейф","срок","обход","дуга","бой","мерцание","блик","искра","тик"]:
    safe = "".join(c for c in w if c not in homo)
    print(f"{w:<12} {safe or '(none)  <-- fully Latin-homoglyphic'}")
PY
```

```text
глубь        глбь
дрейф        дйф
срок         (none)  <-- fully Latin-homoglyphic
обход        бд
дуга         дг
бой          бй
мерцание     ци
блик         бли
искра        и
тик          и
```

The requirement that follows: **every short form must contain at least one
non-confusable letter**, so that the string is detectably Cyrillic rather than
ambiguous. That makes it a detection problem instead of a guessing one.

## The scheme

| tier | exponent | ru name | short | safe letter |
|---|---|---|---|---|
| T5 | 5^85 | глубь | `гл` | г, л |
| T4 | 5^80 | дрейф | `др` | д |
| T3 | 5^75 | срок | **— see below** | none available |
| T2 | 5^70 | обход | `обх` | б |
| T1 | 5^65 | дуга | `ду` | д |
| T0 | 5^60 | бой | `бо` | б |
| T-1 | 5^55 | мерцание | `мц` | ц |
| T-2 | 5^50 | блик | `бл` | б, л |
| T-3 | 5^45 | искра | `ис` | и |
| T-12 | 5^0 | тик | `ти` | и |

Two details that are not arbitrary:

`обход` takes three letters rather than two. `об` and `бо` are reversals of each
other, and `бо` is the beat — the most-used tier on the ladder. A pair that
differs only in letter order, on the rung a reader looks at most often, is a
mistake waiting to be made.

`мерцание` takes `мц` rather than `ме`. `ме` is entirely homoglyphic — м and е
both have Latin twins — and would render identically to the Latin word "me".

## The one that was blocked, and how it was resolved

**Decided: T3 is `пролёт`, short form `пр`.**

**`срок` had no safe short form.** Its four letters are с→c, р→p, о→o, к→k:
every one has a Latin twin, so `ср`, `сро` and `срк` all render exactly as
Latin text. No abbreviation of this word can satisfy the requirement above.

Two ways out, and the choice is the author's because it is a choice about
Russian, not about software:

**Ship `ср` anyway** and let the stated locale carry the disambiguation. Honest,
and it makes one rung of the ladder weaker than the other nine for no reason a
reader can see.

**Give T3 a different Russian word** containing at least one of
б г д ж з и й л п ф ц ч ш щ ъ ы ь э ю я. The English name is *span*, and
candidates that keep the sense while satisfying Appendix D's "short, concrete
motion words":

| candidate | sense | short | note |
|---|---|---|---|
| пролёт | a span, as of a bridge — *пролёт моста* | `пр` | closest to *span*; concrete and structural |
| прогон | a run, a pass | `пр` | motion sense, collides with пролёт's short form |
| пласт | a layer, a stratum | `пл` | geological rather than temporal |
| заход | an approach, a setting | `за` | shades toward обход's sense |

**`пролёт`** was chosen. It is the one that actually means *span* in the
structural sense the English name carries — *пролёт моста*, the span of a
bridge — it is a concrete noun with no mythological or national content, and
`пр` is detectable because of the `п`.

## Built

- `Names` carries a `short` field. `ru` ships ten; `en` ships none, because
  `T[k]` is already short, locale-invariant, and accepted everywhere.
- Short forms resolve wherever a name does. Rule N requires index notation to be
  accepted wherever a name is; a short form *is* a name, and an abbreviation a
  reader can see but not type would be a worse alias than none.
- Four tests, verified strict by injection: reverting T3 to `срок`/`ср` fails
  `every_short_form_is_detectably_not_latin`, and giving `обход` the form `об`
  fails `no_two_short_forms_collide_or_reverse_each_other` against `бо`.
- `docs/TIERS.md` regenerated, since §13.5 makes the locale table and the
  documentation table one source.

## What this exposed

`--locale` was reaching only the *display* of tier names. `ucal ruler --step
пролёт --locale ru` failed while `--step span` worked in every locale, because
the CLI resolved tiers against the default locale regardless of the flag. Rule N
makes names display aliases — a statement about what decides *behaviour*, not
licence for one locale's aliases to be the only ones a parser accepts.

`parse_tier_in` threads the locale through `now`, `ruler` and `timeline`. The
stable keys, `T[k]` and `5^e` still resolve in every locale, so nothing that
worked before stopped working, and `пр` does not resolve under `en` — which is
the scoping doing its job.
