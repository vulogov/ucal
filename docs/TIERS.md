# The tier grid

**Generated from `ucal_core::tier` and `ucal_core::locale` by `cargo run -p xtask -- gen-docs`. Do not edit by hand.**

§13.5 requires the tier table, the locale table and the documentation table to come from one source of truth so they cannot drift. This file is that requirement discharged; `cargo run -p xtask -- check-docs` fails if it is stale.

Rule G: tiers are the powers `5^(5k)`, indexed relative to the beat, so `T[k] = 5^(60 + 5k)`. Each tier is exactly five base-5 digits — 3125 units of the tier below. Rule N: a tier's canonical identity is its **exponent**; the names are display aliases and nothing decides behaviour from one.

The **beat** column is the ladder's own unit. §0.5 names the beat the *universe second*: 5^60 ticks, a pure power of the tick with no Earth content. Every tier is a whole power of five of it, so those values are exact by construction.

The **bridge** column is a foreign unit, shown alongside as §4.3 requires and never instead. Note that the two seconds are incommensurable above T-6: one bridge second is 21.385061835 beats, because `BEAT` carries `5^60` while `SECOND` carries only `5^30`. They share a common measure only at the tick — which is why Rule A.1 makes the tick primitive rather than either second.

The bridge column is informative (Rule A.5). It is rendered from the exact rational `5^e / SECOND` under half-even rounding, in one step — not chained from the neighbouring row, which is how Appendix B's published column came to disagree in the fifth significant figure (delta D-A3).

## How to write a tier in a formula

**`T[k]` and `5^e`.** Both are accepted wherever a name is, because Rule N requires it, and either is the right thing to write in prose, in a formula, or in an argument: `t = 3.5 x 5^80` is exact and needs no glossary, and `T4` sorts.

There is no abbreviation scheme, and the reason is not typographic. **Names are locale-dependent** — the `en` and `ru` columns above are different words for the same exponent — so a short form derived from a name cannot be universal. `bt` for *beat* means nothing under `--locale ru`. Anything short enough to want would either vary by locale, which makes it a second parse surface with a different meaning in each, or be invented independently of the names, which is what the exponent already is.

Greek was considered and rejected. In this project's own neighbourhood `Λ` is the cosmological constant of the flat ΛCDM model `ucal-cosmo` implements, `β` and `γ` are Lorentz quantities, `τ` is proper time and `t_P` is already the Planck time. Beyond the collisions, a Greek letter in a formula *reads* as a physical quantity, which invites exactly the inference Rule N forbids — that a tier is something other than an exponent with a display name attached.

Short forms scoped to a *locale* are a different question and are not ruled out by any of this; see `Documentation/Proposals/U5-cyrillic-short-forms.md`.

| k | exponent | beats (universe seconds) | bridge units | human | en | ru | ticks |
|---:|---:|---:|---:|---:|---|---|---:|
| 32 | 220 | 6.8423e+111 | 3.1996e+110 | — | — | — | `5^220` |
| 31 | 215 | 2.1895e+108 | 1.0239e+107 | — | — | — | `5^215` |
| 30 | 210 | 7.0065e+104 | 3.2763e+103 | — | — | — | `5^210` |
| 29 | 205 | 2.2421e+101 | 1.0484e+100 | — | — | — | `5^205` |
| 28 | 200 | 7.1746e+97 | 3.3550e+96 | — | — | — | `5^200` |
| 27 | 195 | 2.2959e+94 | 1.0736e+93 | — | — | — | `5^195` |
| 26 | 190 | 7.3468e+90 | 3.4355e+89 | — | — | — | `5^190` |
| 25 | 185 | 2.3510e+87 | 1.0994e+86 | — | — | — | `5^185` |
| 24 | 180 | 7.5232e+83 | 3.5180e+82 | — | — | — | `5^180` |
| 23 | 175 | 2.4074e+80 | 1.1257e+79 | — | — | — | `5^175` |
| 22 | 170 | 7.7037e+76 | 3.6024e+75 | — | — | — | `5^170` |
| 21 | 165 | 2.4652e+73 | 1.1528e+72 | — | — | — | `5^165` |
| 20 | 160 | 7.8886e+69 | 3.6888e+68 | — | — | — | `5^160` |
| 19 | 155 | 2.5244e+66 | 1.1804e+65 | — | — | — | `5^155` |
| 18 | 150 | 8.0779e+62 | 3.7774e+61 | — | — | — | `5^150` |
| 17 | 145 | 2.5849e+59 | 1.2088e+58 | — | — | — | `5^145` |
| 16 | 140 | 8.2718e+55 | 3.8680e+54 | — | — | — | `5^140` |
| 15 | 135 | 2.6470e+52 | 1.2378e+51 | — | — | — | `5^135` |
| 14 | 130 | 8.4703e+48 | 3.9609e+47 | — | — | — | `5^130` |
| 13 | 125 | 2.7105e+45 | 1.2675e+44 | — | — | — | `5^125` |
| 12 | 120 | 8.6736e+41 | 4.0559e+40 | — | — | — | `5^120` |
| 11 | 115 | 2.7756e+38 | 1.2979e+37 | — | — | — | `5^115` |
| 10 | 110 | 8.8818e+34 | 4.1533e+33 | — | — | — | `5^110` |
| 9 | 105 | 2.8422e+31 | 1.3290e+30 | — | — | — | `5^105` |
| 8 | 100 | 9.0949e+27 | 4.2529e+26 | — | — | — | `5^100` |
| 7 | 95 | 2.9104e+24 | 1.3609e+23 | 4312565.203 Gyr | — | — | `5^95` |
| 6 | 90 | 9.3132e+20 | 4.3550e+19 | 1380.021 Gyr | — | — | `5^90` |
| 5 | 85 | 2.9802e+17 | 1.3936e+16 | 441.607 Myr | deep | глубь | `5^85` |
| 4 | 80 | 9.5367e+13 | 4.4595e+12 | 141.314 kyr | drift | дрейф | `5^80` |
| 3 | 75 | 3.0518e+10 | 1.4271e+9 | 45.221 yr | span | срок | `5^75` |
| 2 | 70 | 9.7656e+6 | 4.5666e+5 | 5.285 d | sweep | обход | `5^70` |
| 1 | 65 | 3.1250e+3 | 1.4613e+2 | 146.130 s | arc | дуга | `5^65` |
| 0 | 60 | 1.0000e+0 | 4.6762e-2 | 46.762 ms | beat | бой | `5^60` |
| -1 | 55 | 3.2000e-4 | 1.4964e-5 | 14.964 us | flicker | мерцание | `5^55` |
| -2 | 50 | 1.0240e-7 | 4.7884e-9 | 4.788 ns | glint | блик | `5^50` |
| -3 | 45 | 3.2768e-11 | 1.5323e-12 | 1.532 ps | spark | искра | `5^45` |
| -4 | 40 | 1.0486e-14 | 4.9033e-16 | 490.331 as | — | — | `5^40` |
| -5 | 35 | 3.3554e-18 | 1.5691e-19 | 156.906 zs | — | — | `5^35` |
| -6 | 30 | 1.0737e-21 | 5.0210e-23 | 50.210 ys | — | — | `931322574615478515625` |
| -7 | 25 | 3.4360e-25 | 1.6067e-26 | — | — | — | `298023223876953125` |
| -8 | 20 | 1.0995e-28 | 5.1415e-30 | — | — | — | `95367431640625` |
| -9 | 15 | 3.5184e-32 | 1.6453e-33 | — | — | — | `30517578125` |
| -10 | 10 | 1.1259e-35 | 5.2649e-37 | — | — | — | `9765625` |
| -11 | 5 | 3.6029e-39 | 1.6848e-40 | — | — | — | `3125` |
| -12 | 0 | 1.1529e-42 | 5.3912e-44 | — | tick | тик | `1` |

## Notes

- **T32 is the ceiling.** `5^220` is 511 bits, the largest power of five the 512-bit domain holds, so the grid cannot extend further without widening the domain — and Rule B makes the width a wire-format commitment (D-4).
- **T−12 is the floor.** One tick is the finest addressable interval (G2). There is no sub-tick representation and intervals shorter than one tick must not be approximated (N10).
- **Unnamed tiers are not second-class.** D-20 leaves everything above T5 and below T−3 unnamed and addressable by index; Rule N requires `T[k]` and `5^e` to be accepted wherever a name is.
- **Nothing on the ladder is near a second or an hour.** That is the accepted cost of leaving the Earth paradigm (D-2), which is why the bridge column is always printed alongside.
