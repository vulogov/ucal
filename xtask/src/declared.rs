//! Values transcribed verbatim from RFC UCAL-1.
//!
//! Everything in this file is a *claim by the RFC*, not a computed value. The
//! P0 harness's job is to reproduce every one of these by two independent
//! exact-integer routes and report any that do not match (§20, UC-P0; §21.3
//! "any hand-transcribed constant the P0 harness does not reproduce").
//!
//! Entries the harness has shown to be wrong are marked and cross-referenced to
//! `spec/SPEC-DELTAS.md`. They are retained here deliberately: deleting a wrong
//! claim would make the regression untestable.

/// Appendix A — declared datum and bridge constants.
pub mod appendix_a {
    pub const BEAT: &str = "867361737988403547205962240695953369140625";
    pub const BEAT_EXPONENT: u32 = 60;

    pub const ORIGIN_OFFSET: &str =
        "8070204002895596515944343085635637180530466139316558837890625";
    pub const ORIGIN_OFFSET_BEATS: &str = "9304311741502590385";

    pub const SECOND: &str = "18548584399861000000000000000000000000000000";
    /// `SECOND = SECOND_MANTISSA x 10^SECOND_DECIMAL_SCALE` (D-3).
    pub const SECOND_MANTISSA: &str = "18548584399861";
    pub const SECOND_DECIMAL_SCALE: u32 = 30;

    pub const BIG_BANG_CLAIM_HALFWIDTH: &str =
        "11706976141141069872000000000000000000000000000000000000000";
    /// `BIG_BANG_CLAIM = 631152 x SECOND_MANTISSA x 10^39`
    pub const BBC_JULIAN_SECONDS_TIMES_1000: &str = "631152";

    /// `DOMAIN = [0, 2^512)`, so `DOMAIN_MAX = 2^512 - 1`.
    pub const DOMAIN_BITS: u32 = 512;

    // ---- structural annotations on ORIGIN_OFFSET ----
    pub const OO_BITS: u32 = 203;
    pub const OO_BASE5_DIGITS: usize = 88;
    /// RFC claims 62. Exact valuation is 61 — see delta D-A2.
    pub const OO_TRAILING_BASE5_ZEROS_CLAIMED: usize = 62;
    pub const OO_TRAILING_BASE5_ZEROS_ACTUAL: usize = 61;
}

/// §2.2 — the `datum_provenance` chain, which must re-execute exactly.
pub mod provenance {
    pub const INPUT_GYR_TIMES_1000: &str = "13787";
    /// `Gyr = 10^9 x 31 557 600 s` (Julian years, exact by definition).
    pub const JULIAN_YEAR_SECONDS: &str = "31557600";
    pub const UNCERTAINTY_GYR_TIMES_1000: &str = "20";

    pub const AGE_S: &str = "435084631200000000";
    pub const AGE_TICKS: &str =
        "8070204002895596516263200000000000000000000000000000000000000";
    pub const BEATS: &str = "9304311741502590385";
    /// Signed: the rounded datum lies *before* the unrounded age.
    pub const RESIDUAL_TICKS: &str = "-318856914364362819469533860683441162109375";
    pub const RESIDUAL_SECONDS_RENDERED: &str = "-0.017190364";
}

/// §2.4 — alignment invariants, stated as minimum base-5 valuations.
pub mod alignment {
    /// A whole SI second has zero in tiers T-12..T-7 = the low 30 base-5 digits.
    pub const WHOLE_SECOND_MIN_V5: u32 = 30;
    /// A whole nanosecond has zero in the low 21 base-5 digits.
    pub const WHOLE_NANOSECOND_MIN_V5: u32 = 21;
    /// SI_EPOCH has zero in all tiers below T0 = the low 60 base-5 digits.
    pub const SI_EPOCH_MIN_V5: u32 = 60;
}

/// Appendix B — named tiers. Canonical identity is the exponent (Rule N).
pub mod tiers {
    pub const NAMED: &[(i8, u32, &str)] = &[
        (5, 85, "deep"),
        (4, 80, "drift"),
        (3, 75, "span"),
        (2, 70, "sweep"),
        (1, 65, "arc"),
        (0, 60, "beat"),
        (-1, 55, "flicker"),
        (-2, 50, "glint"),
        (-3, 45, "spark"),
        (-12, 0, "tick"),
    ];
    /// The grid is `5^(5k)`; `TIER[k] = 5^(60 + 5k)` for k in -12..=32.
    pub const K_MIN: i8 = -12;
    pub const K_MAX: i8 = 32;
    pub const COUNT: usize = 45;
}

/// Appendix C — fixtures. Civil times are TT labels on proleptic Gregorian with
/// astronomical year numbering (§2.5).
pub mod fixtures {
    pub struct Declared {
        pub name: &'static str,
        /// (year, month, day, hour, minute, second); `None` = not civil-derived.
        pub civil: Option<(i64, u8, u8, u8, u8, u8)>,
        pub ticks: &'static str,
        /// Human form as the RFC prints it (T5..T0), without the sub-beat part.
        pub human_beat: &'static str,
        /// Sub-beat groups as the RFC prints them. The RFC quotes five groups
        /// (T-1..T-5); a tick-exact whole-second instant needs six — delta D-A4.
        pub human_sub_rfc: Option<&'static str>,
        pub ucid: &'static str,
        /// Set where the harness has shown the RFC's value to be wrong. Empty at
        /// present: every Appendix C tick value reproduces (see D-A1's withdrawal).
        /// Kept because the next erratum should have somewhere to go.
        #[allow(dead_code)]
        pub delta: Option<&'static str>,
    }

    pub const ALL: &[Declared] = &[
        Declared {
            name: "absolute zero (the datum)",
            civil: None,
            ticks: "0",
            human_beat: "0000·0000·0000·0000·0000·0000",
            human_sub_rfc: None,
            ucid: "0000000000000000000000000000000000000000000000000000",
            delta: None,
        },
        Declared {
            name: "SI_EPOCH 0000-01-01T00:00:00 TT",
            civil: Some((0, 1, 1, 0, 0, 0)),
            ticks: "8070204002895596515944343085635637180530466139316558837890625",
            human_beat: "0031·0687·2437·0454·2703·2885",
            human_sub_rfc: None,
            ucid: "0000000000050PM5TBHF4BFKRZC1KVN566SZGWG5DZ0SSBM29FJ1",
            delta: None,
        },
        Declared {
            name: "-0043-03-15T00:00:00 TT (44 BC)",
            civil: Some((-43, 3, 15, 0, 0, 0)),
            ticks: "8070203977843789392286957152835637180530466139316558837890625",
            human_beat: "0031·0687·2436·0622·0843·1347",
            human_sub_rfc: Some("2726·0773·2384·0202·2453"),
            ucid: "0000000000050PM5STSSZT2034C3TGX8CMS2Z79C0SGBSBM29FJ1",
            // Correct. An earlier oracle disagreed by one day because it applied
            // Hinnant's `y2 - 399` era adjustment on top of flooring division;
            // that adjustment exists to make *truncating* division floor, and
            // double-applying it puts `yoe` outside [0, 399] for negative years.
            // Retained as a regression: negative astronomical years are the only
            // place this class of bug shows up.
            delta: None,
        },
        Declared {
            name: "1969-07-20T20:17:40 TT (Apollo 11)",
            civil: Some((1969, 7, 20, 20, 17, 40)),
            ticks: "8070205155746435292175415045495637180530466139316558837890625",
            human_beat: "0031·0687·2480·2184·1466·1514",
            human_sub_rfc: Some("0493·1291·0839·2005·2854"),
            ucid: "0000000000050PM6JDVVP2F5SVFPGQVWXDMG2X4TRBKE3ZM29FJ1",
            delta: None,
        },
        Declared {
            name: "1970-01-01T00:00:00 TT (Unix epoch label)",
            civil: Some((1970, 1, 1, 0, 0, 0)),
            ticks: "8070205156009508751803579616835637180530466139316558837890625",
            human_beat: "0031·0687·2480·2215·1648·1438",
            human_sub_rfc: Some("0170·1214·0735·1806·1815"),
            ucid: "0000000000050PM6JE1FP4P4GK3BBVJDEP65A8Y6G77WSBM29FJ1",
            delta: None,
        },
        Declared {
            name: "1980-01-06T00:00:00 TT (GPS epoch label)",
            civil: Some((1980, 1, 6, 0, 0, 0)),
            ticks: "8070205161870208511988780509635637180530466139316558837890625",
            human_beat: "0031·0687·2480·2907·1365·0098",
            human_sub_rfc: Some("1925·2279·0814·1116·0715"),
            ucid: "0000000000050PM6JHYSQVFAA0446J292X7AGVGHSP9PNBM29FJ1",
            delta: None,
        },
        Declared {
            name: "2000-01-01T12:00:00 TT (J2000.0)",
            civil: Some((2000, 1, 1, 12, 0, 0)),
            ticks: "8070205173569972963515184424835637180530466139316558837890625",
            human_beat: "0031·0687·2481·1163·2191·0758",
            human_sub_rfc: Some("1924·0749·2247·0012·1174"),
            ucid: "0000000000050PM6JSRZ1JEN8CJ8JG0H3SXHYWVS2CY7KBM29FJ1",
            delta: None,
        },
        Declared {
            name: "2026-07-29T00:00:00 TT",
            civil: Some((2026, 7, 29, 0, 0, 0)),
            ticks: "8070205189123984864657505252035637180530466139316558837890625",
            human_beat: "0031·0687·2481·2999·3108·2437",
            human_sub_rfc: Some("1104·2790·0251·2597·0804"),
            ucid: "0000000000050PM6K45HH4YGQJ6SEDGDDZ1NKFHD32F2XBM29FJ1",
            delta: None,
        },
        Declared {
            name: "Earth formation (SI_EPOCH - 4.54 Gyr)",
            civil: None,
            ticks: "5412720418856573655000343085635637180530466139316558837890625",
            human_beat: "0020·2935·2420·2803·2533·2001",
            human_sub_rfc: Some("2269·2517·0923·1945·1875"),
            ucid: "000000000003BS5WVY8XGGMN3M0D068RR37G6W0DEWKHSBM29FJ1",
            delta: None,
        },
        Declared {
            name: "recombination (datum + 380 kyr)",
            civil: None,
            ticks: "222432546681680327568000000000000000000000000000000000000",
            human_beat: "0000·0002·2153·0825·0246·0025",
            human_sub_rfc: Some("1908·2584·2019·0482·2740"),
            ucid: "000000000000004H4KEWEGEB5M995XKBZHX3425VFFD900000000",
            delta: None,
        },
    ];

    /// The RFC's digit5 line for 2026-07-29 (§Appendix C). 22 groups.
    pub const DIGIT5_2026: &str = "00000.00000.00000.00000.00111.10222.34411.43444.\
44413.34222.13404.42130.02001.40342.11204.13400.00000.00000.00000.00000.00000.00000";

    /// Offsets that are not civil dates: (name, whole Julian years before/after datum).
    pub const EARTH_FORMATION_GYR_TIMES_100_BEFORE_EPOCH: &str = "454";
    pub const RECOMBINATION_KYR_AFTER_DATUM: &str = "380";
}

/// Appendix I — derived-calendar test vectors.
pub mod appendix_i {
    pub struct CfVector {
        pub label: &'static str,
        /// The ratio as the RFC prints it, exact decimal.
        pub ratio: &'static str,
        /// Continued fraction of the *fractional* part, as printed.
        pub cf_frac: &'static [u64],
        /// Convergents of the fractional part, as printed, in order.
        pub convergents: &'static [(u64, u64)],
    }

    pub const I1_EARTH_INTERCALATION: CfVector = CfVector {
        label: "I.1 Earth intercalation (tropical year / mean solar day)",
        ratio: "365.242190",
        cf_frac: &[0, 4, 7, 1, 3, 24, 6, 2, 2],
        convergents: &[(1, 4), (7, 29), (8, 33), (31, 128), (752, 3105), (4543, 18758)],
    };

    /// I.2 prints the *whole* ratio's convergents, starting 12/1.
    pub const I2_EARTH_GROUPING: CfVector = CfVector {
        label: "I.2 Earth grouping cycle (tropical year / synodic month)",
        ratio: "12.368266761",
        cf_frac: &[12, 2, 1, 2, 1, 1, 17, 2, 1],
        convergents: &[
            (12, 1),
            (25, 2),
            (37, 3),
            (99, 8),
            (136, 11),
            (235, 19),
            (4131, 334),
        ],
    };

    pub const I3_MARS_INTERCALATION: CfVector = CfVector {
        label: "I.3 Mars intercalation (tropical year / solar day)",
        ratio: "668.592165627",
        cf_frac: &[0, 1, 1, 2, 4, 1, 2, 2, 1],
        convergents: &[
            (1, 1),
            (1, 2),
            (3, 5),
            (13, 22),
            (16, 27),
            (45, 76),
            (106, 179),
        ],
    };

    pub const I5_TITAN_INTERCALATION: CfVector = CfVector {
        label: "I.5 Titan intercalation (Saturn tropical year / Titan solar day)",
        ratio: "673.983719443",
        cf_frac: &[0, 1, 60, 2, 2, 1, 2, 1, 11],
        convergents: &[(1, 1), (60, 61), (121, 123), (302, 307), (423, 430)],
    };

    /// I.4 — Mars satellites. Values in seconds, from Appendix G.
    pub const MARS_SOLAR_DAY_S: (&str, u32) = ("88775244", 3); // 88775.244 s, scale 10^-3
    pub const PHOBOS_ORBITAL_S: (&str, u32) = ("27553", 0);
    pub const DEIMOS_ORBITAL_S: (&str, u32) = ("109123", 0);
    /// Synodic periods the RFC prints, in sols.
    pub const PHOBOS_SYNODIC_SOLS: &str = "0.4500";
    pub const DEIMOS_SYNODIC_SOLS: &str = "5.3629";
    /// D-11 default bounds, in solar days. Superseded as an admission gate by
    /// delta D-A5 (declared grouping satellite); retained as a sanity filter.
    pub const CYCLE_BOUNDS_SOLS: (u32, u32) = (5, 100);

    /// The Gregorian rule, which MUST NOT appear as a convergent (§9.5, §21.3-6).
    pub const GREGORIAN_RULE: (u64, u64) = (97, 400);
    /// The Metonic cycle, which MUST appear (§21.3-7).
    pub const METONIC: (u64, u64) = (235, 19);
}
