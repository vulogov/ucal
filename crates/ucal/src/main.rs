//! The `ucal` binary (§19).
//!
//! Parse, dispatch, print, exit. All the work is in the library, so that §20's
//! golden tests can call the commands as functions.

use std::io::IsTerminal as _;

use clap::{Parser, Subcommand};
use ucal::style::{parse_group_sep, resolve_for_output, ColorChoice, Render, Role, Style};
use ucal::{
    cmd_datum, cmd_doctor, cmd_explain, cmd_ladder, exit_code, parse_rounding, parse_tier_in,
};
use ucal_core::LocaleId;
use ucal_core::codec::Form;

#[cfg(feature = "civil")]
use ucal::{cmd_from_civil, cmd_to_civil};
#[cfg(feature = "civil")]
use ucal_civil::{calendar::CivilCalendar, si::Scale};

#[derive(Parser)]
#[command(
    name = "ucal",
    version,
    about = "Universe Calendar — absolute time in Planck ticks",
    long_about = "Absolute time is an unsigned integer count of Planck-time units since a \
                  stipulated datum, with a positional base-5 calendar over it.\n\n\
                  Tick 0 is a stipulated reference point, conventionally identified with the \
                  FLRW t→0 limit. It is not a measurement and not an observed event; run \
                  `ucal datum` for the full statement, its provenance, and what is and is not \
                  being claimed."
)]
struct Cli {
    /// Profile to use. Only UC-1 exists at present.
    #[arg(long, global = true, default_value = "UC-1")]
    profile: String,

    /// Group separator for text forms. Must not be a digit (§6.3).
    #[arg(long, global = true)]
    sep: Option<char>,

    /// Locale for tier names (Rule N: names are display-only). Shipped: en, ru.
    #[arg(long, global = true, default_value = "en", value_parser = locale_values())]
    locale: String,

    /// Emit stable, versioned JSON instead of text (§19.1).
    #[arg(long, global = true)]
    json: bool,

    /// When to colour the output. `auto` colours only into a terminal.
    #[arg(long, global = true, default_value = "auto",
          value_parser = ["auto", "always", "never"])]
    color: String,

    /// Never colour. An alias for `--color never`, and it wins over `--color`.
    #[arg(long, global = true)]
    no_color: bool,

    /// Columns to render tables at. Never below 80; defaults to the terminal
    /// width when there is one, and to 80 when output is redirected.
    #[arg(long, global = true, value_name = "N")]
    width: Option<usize>,

    /// Separator between three-digit groups in decimal counts, e.g. `--tick-sep _`.
    ///
    /// Off by default: a tick count is often copied out of this output into
    /// something that wants an integer, and a separator breaks that. With colour
    /// the groups are already distinguishable without adding a character.
    #[arg(long, global = true, value_name = "CHAR")]
    tick_sep: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// The current instant, from the system clock (§8.4: offline).
    Now {
        /// Tier to render to: a name, `T<k>`, or `5^e`.
        #[arg(long, default_value = "T-12")]
        precision: String,
        /// Text form to render.
        #[arg(long, default_value = "human")]
        form: String,
    },
    /// The datum: what tick 0 is, what is claimed about it, and how it was fixed.
    Datum,
    /// Convert a civil date to absolute time. Exact or an error, never rounded.
    #[cfg(feature = "civil")]
    FromCivil {
        /// `2026-07-29`, `2026-07-29T12:34:56.5`, `-0043-03-15`, `44 BC-03-15`.
        date: String,
        /// Time scale of the label.
        #[arg(long, default_value = "tt")]
        scale: String,
        /// Input calendar (§8.5). Both are legacy (§8.6).
        #[arg(long, default_value = "gregorian")]
        calendar: String,
    },
    /// Render absolute time as a civil label. Rounds only here (Rule R).
    #[cfg(feature = "civil")]
    ToCivil {
        /// A `UC1` text form, a UCID, or a decimal tick count.
        instant: String,
        #[arg(long, default_value = "tt")]
        scale: String,
        /// Fractional-second digits, up to 30.
        #[arg(long, default_value_t = 0)]
        digits: u8,
        #[arg(long, default_value = "half-even")]
        round: String,
        #[arg(long, default_value = "gregorian")]
        calendar: String,
    },
    /// Everything about an instant: forms, tiers, the SI bridge, any warning.
    Explain {
        /// A `UC1` text form, a UCID, or a decimal tick count.
        instant: String,
        /// Also print BIG_BANG_CLAIM (metadata; never an operand).
        #[arg(long)]
        claim: bool,
    },
    /// The universal tier grid (§4.2), in the chosen locale.
    Ladder {
        /// Show only the named tiers (D-20 leaves the rest addressable by index).
        #[arg(long)]
        named_only: bool,
    },
    /// Calendars, derived and legacy, each with its kind (§19.4).
    #[cfg(all(feature = "body", feature = "civil"))]
    Cal {
        #[command(subcommand)]
        what: CalCommand,
    },
    /// One instant in several local calendars (§19.4).
    #[cfg(all(feature = "body", feature = "civil"))]
    Show {
        /// A `UC1` text form, a UCID, or a decimal tick count.
        instant: String,
        /// Comma-separated calendar ids, e.g. `earth-d,mars-d,earth-civil`.
        #[arg(long, default_value = "earth-d,mars-d,earth-civil")]
        calendars: String,
    },
    /// Cited, interval-valued milestones (§17).
    #[cfg(feature = "events")]
    Events {
        #[command(subcommand)]
        what: EventCommand,
    },
    /// The catalogue against the tier ladder — the whole of time, on one screen.
    #[cfg(feature = "events")]
    Timeline {
        /// Tier to place events at: a name, `T<k>`, or `5^e`.
        #[arg(long, default_value = "drift")]
        tier: String,
    },
    /// Evenly spaced marks on the tier grid.
    Ruler {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        /// Step tier: a name, `T<k>`, or `5^e`.
        #[arg(long, default_value = "sweep")]
        step: String,
    },
    /// Flat ΛCDM, by certified integer quadrature (§10).
    #[cfg(feature = "cosmo")]
    Cosmo {
        #[command(subcommand)]
        what: CosmoCommand,
    },
    /// Profile, backend, domain ceiling, leap table, features, provenance.
    Doctor,
}

#[cfg(feature = "cosmo")]
#[derive(Subcommand)]
enum CosmoCommand {
    /// The age of the universe at a redshift, as a certified enclosure.
    Age {
        /// Redshift, as an exact decimal, e.g. `1100` or `0.5`.
        #[arg(long)]
        z: String,
        /// Subdivision depth: 2^depth panels. Higher is narrower and slower —
        /// see `ucal cosmo model` for the measured cost (GE-1).
        #[arg(long, default_value_t = ucal_cosmo::DEFAULT_DEPTH)]
        depth: u32,
        /// Decimal digits for the directed square roots (D-6).
        #[arg(long, default_value_t = ucal_cosmo::DEFAULT_SCALE)]
        scale: u32,
    },
    /// The redshift at an absolute time, by bisection.
    Z {
        /// A `UC1` text form, a UCID, or a decimal tick count.
        #[arg(long)]
        at: String,
        /// How finely to resolve the answer, in years. A tick is unreachable
        /// and is refused with UCAL-E0071 rather than faked.
        #[arg(long, default_value_t = 1)]
        tolerance_years: u64,
        #[arg(long, default_value_t = 8)]
        depth: u32,
        #[arg(long, default_value_t = ucal_cosmo::DEFAULT_SCALE)]
        scale: u32,
    },
    /// The parameter set, its provenance, and the measured GE-1/GE-2 outcomes.
    Model,
}

#[cfg(feature = "events")]
#[derive(Subcommand)]
enum EventCommand {
    /// The whole catalogue, chronologically.
    List,
    /// One milestone, in full.
    Show {
        /// Event id, e.g. `recombination`.
        id: String,
    },
}

#[cfg(all(feature = "body", feature = "civil"))]
#[derive(Subcommand)]
enum CalCommand {
    /// Every calendar, with its kind.
    List,
    /// One calendar's derivation in full: anchor, intercalation, cycles.
    Show {
        /// Calendar id, e.g. `earth-d`.
        id: String,
        /// A `UC1` text form, a UCID, or a decimal tick count.
        instant: String,
    },
    /// A calendar's anchor (Rule J).
    Anchor {
        /// Calendar id.
        id: String,
    },
}

#[cfg(feature = "civil")]
fn parse_scale(s: &str) -> Result<Scale, ucal_core::TimeError> {
    match s {
        "tt" => Ok(Scale::Tt),
        "tai" => Ok(Scale::Tai),
        "utc" => Ok(Scale::Utc),
        _ => Err(ucal_core::TimeError::with_context(
            ucal_core::Code::E0001,
            "scale must be tt, tai or utc",
        )),
    }
}

#[cfg(feature = "civil")]
fn parse_calendar(s: &str) -> Result<CivilCalendar, ucal_core::TimeError> {
    match s {
        "gregorian" => Ok(CivilCalendar::Gregorian),
        "julian" => Ok(CivilCalendar::Julian),
        _ => Err(ucal_core::TimeError::with_context(
            ucal_core::Code::E0001,
            "calendar must be gregorian or julian",
        )),
    }
}

fn parse_form(s: &str) -> Result<Form, ucal_core::TimeError> {
    match s {
        "human" => Ok(Form::HumanGroups),
        "digit5" => Ok(Form::Digit5),
        "named" => Ok(Form::Named),
        _ => Err(ucal_core::TimeError::with_context(
            ucal_core::Code::E0001,
            "form must be human, digit5 or named",
        )),
    }
}

/// The accepted `--locale` values, taken from the locale table itself so that
/// `--help` cannot list a locale the library does not ship (§13.5).
fn locale_values() -> clap::builder::PossibleValuesParser {
    clap::builder::PossibleValuesParser::new(
        LocaleId::ALL.iter().map(|l| l.tag()).collect::<Vec<_>>(),
    )
}

fn main() {
    let cli = Cli::parse();

    if cli.profile != "UC-1" && cli.profile != "UC1" {
        eprintln!(
            "UCAL-E0002: unknown profile tag `{}`. Only UC-1 exists (Rule P).",
            cli.profile
        );
        std::process::exit(5);
    }

    let result = match &cli.command {
        Command::Datum => cmd_datum(),
        Command::Doctor => cmd_doctor(),
        #[cfg(feature = "cosmo")]
        Command::Cosmo { what } => match what {
            CosmoCommand::Age { z, depth, scale } => ucal::cmd_cosmo_age(z, *depth, *scale),
            CosmoCommand::Z {
                at,
                tolerance_years,
                depth,
                scale,
            } => ucal::cmd_cosmo_z(at, *tolerance_years, *depth, *scale),
            CosmoCommand::Model => ucal::cmd_cosmo_model(),
        },
        #[cfg(feature = "events")]
        Command::Events { what } => match what {
            EventCommand::List => ucal::cmd_events_list(),
            EventCommand::Show { id } => ucal::cmd_events_show(id),
        },
        #[cfg(feature = "events")]
        Command::Timeline { tier } => LocaleId::parse(&cli.locale)
            .and_then(|l| parse_tier_in(l, tier))
            .and_then(ucal::cmd_timeline),
        Command::Ruler { from, to, step } => {
            LocaleId::parse(&cli.locale)
                .and_then(|l| parse_tier_in(l, step))
                .and_then(|s| ucal::cmd_ruler(from, to, s))
        }
        #[cfg(all(feature = "body", feature = "civil"))]
        Command::Cal { what } => match what {
            CalCommand::List => ucal::cmd_cal_list(),
            CalCommand::Show { id, instant } => ucal::cmd_cal_show(id, instant),
            CalCommand::Anchor { id } => ucal::cmd_cal_anchor(id),
        },
        #[cfg(all(feature = "body", feature = "civil"))]
        Command::Show {
            instant,
            calendars,
        } => {
            let ids: Vec<String> = calendars.split(',').map(|s| s.trim().to_string()).collect();
            ucal::cmd_show(instant, &ids)
        }
        Command::Ladder { named_only } => {
            LocaleId::parse(&cli.locale).and_then(|l| cmd_ladder(l, *named_only))
        }
        Command::Explain { instant, claim } => cmd_explain(instant, *claim),
        Command::Now { precision, form } => run_now(&cli.locale, precision, form),
        #[cfg(feature = "civil")]
        Command::FromCivil {
            date,
            scale,
            calendar,
        } => parse_scale(scale)
            .and_then(|s| parse_calendar(calendar).map(|c| (s, c)))
            .and_then(|(s, c)| cmd_from_civil(date, s, c)),
        #[cfg(feature = "civil")]
        Command::ToCivil {
            instant,
            scale,
            digits,
            round,
            calendar,
        } => parse_scale(scale)
            .and_then(|s| parse_rounding(round).map(|r| (s, r)))
            .and_then(|(s, r)| parse_calendar(calendar).map(|c| (s, r, c)))
            .and_then(|(s, r, c)| cmd_to_civil(instant, s, *digits, r, c)),
    };

    // --no-color beats --color, because it is the flag a script sets when it
    // cannot know what the caller's environment has already put in `--color`.
    let choice = if cli.no_color {
        ColorChoice::Never
    } else {
        match ColorChoice::parse(&cli.color) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(exit_code(&e));
            }
        }
    };
    let style = resolve_for_output(choice, cli.json);
    let sep = match cli.tick_sep.as_deref().map(parse_group_sep).transpose() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(exit_code(&e));
        }
    };
    // Off a terminal the width is the baseline, always: if it followed the
    // terminal on a redirected stream, `ucal ladder > f` and `ucal ladder | cat`
    // would differ, and so would the same command on two machines.
    let terminal = if std::io::stdout().is_terminal() {
        terminal_size::terminal_size().map(|(terminal_size::Width(w), _)| w as usize)
    } else {
        None
    };
    let render = Render::styled(style)
        .group(sep)
        .width(Render::resolve_width(cli.width, terminal));

    match result {
        Ok(doc) => {
            print!(
                "{}",
                if cli.json {
                    doc.to_json()
                } else {
                    doc.render(&render)
                }
            );
        }
        Err(e) => {
            // stderr is a different stream with its own answer to "is this a
            // terminal", so it gets its own resolution rather than reusing the
            // one computed for stdout.
            let err_style = error_style(choice);
            eprintln!("{}", err_style.paint(Role::Error, &e.to_string()));
            std::process::exit(exit_code(&e));
        }
    }
}

/// The style for diagnostics on stderr.
///
/// `--json` does not suppress it: the JSON contract in §19.1 is about stdout,
/// and a diagnostic is not part of the document a consumer parses.
fn error_style(choice: ColorChoice) -> Style {
    use std::io::IsTerminal as _;
    match choice {
        ColorChoice::Never => Style::PLAIN,
        ColorChoice::Always => Style::colored(),
        ColorChoice::Auto => {
            if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty())
                || !std::io::stderr().is_terminal()
            {
                Style::PLAIN
            } else {
                Style::colored()
            }
        }
    }
}

#[cfg(all(feature = "civil", feature = "std"))]
fn run_now(locale: &str, precision: &str, form: &str) -> ucal::CmdResult {
    let tier = parse_tier_in(LocaleId::parse(locale)?, precision)?;
    let f = parse_form(form)?;
    ucal::cmd_now(tier, f)
}

#[cfg(not(all(feature = "civil", feature = "std")))]
fn run_now(_locale: &str, _precision: &str, _form: &str) -> ucal::CmdResult {
    Err(ucal_core::TimeError::with_context(
        ucal_core::Code::E0001,
        "`ucal now` requires the `civil` and `std` features",
    ))
}
