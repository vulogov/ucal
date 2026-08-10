//! The `ucal` binary (§19).
//!
//! Parse, dispatch, print, exit. All the work is in the library, so that §20's
//! golden tests can call the commands as functions.

use std::io::IsTerminal as _;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use ucal::style::{parse_group_sep, resolve_for_output, ColorChoice, Render, Role, Style};
use ucal_core::Rounding;
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

    /// Fractional digits for rendered rationals, overriding each field's own
    /// default. Without it every field keeps the precision it was written with.
    #[arg(long, global = true, value_name = "N")]
    decimals: Option<u32>,

    /// Rounding mode for rendered values: trunc, ceil, half-even or half-up.
    ///
    /// Rule R makes rendering the only place a value may be rounded, so the
    /// mode is a declared choice rather than a constant. Without it each field
    /// keeps its own.
    #[arg(long, global = true, value_name = "MODE",
          value_parser = ["trunc", "ceil", "half-even", "half-up"])]
    round: Option<String>,

    /// Also show foreign-unit conversions: SI seconds, Julian years, Gyr.
    ///
    /// Off by default. A Julian year is 365.25 of Earth's rotations and an SI
    /// second is an Earth unit; using either to describe something that is not
    /// of Earth is the substitution this program exists to object to. The
    /// conversion is available on request and is not performed unasked.
    #[arg(long, global = true)]
    bridge: bool,

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
        /// Annotate each field with the rule or section that requires it.
        #[arg(long)]
        why: bool,
    },
    /// How far apart two instants are, on the tier ladder.
    Between {
        /// A `UC1` text form, a UCID, or a decimal tick count.
        from: String,
        /// A `UC1` text form, a UCID, or a decimal tick count.
        to: String,
        /// Also report the whole count and remainder at one named tier.
        #[arg(long)]
        at: Option<String>,
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
    /// The catalogue against the tier ladder: the whole of time, in one document.
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
    /// Re-derive the declared constants and check this build reproduces them.
    Verify,
    /// The first five minutes: what to type, what it shows, and why.
    Tour,
    /// Shell completions, generated from this program's own argument parser.
    Completions {
        /// bash, zsh, fish, powershell or elvish.
        shell: Shell,
    },
    /// The manual page, in roff, generated from this program's own argument parser.
    Man {
        /// A subcommand, for its own page. Without one, the top-level page.
        command: Option<String>,
    },
    /// Profile, backend, domain ceiling, leap table, features, provenance.
    Doctor,
}

#[cfg(feature = "cosmo")]
#[derive(Subcommand)]
enum CosmoCommand {
    /// The age of the universe at a redshift, as a certified enclosure.
    #[command(allow_negative_numbers = true)]
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
        /// Also print how the enclosure was reached, and which direction each
        /// rounding in the chain moved.
        #[arg(long)]
        audit: bool,
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
    /// Read a body file (§15.1) and show the calendar it derives.
    Derive {
        /// Path to a body file.
        file: String,
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

/// Exit code for a panic that reached the top. §19.5 assigns 0–9 to defined
/// conditions; 70 is `EX_SOFTWARE` from `sysexits.h`, which is what this is.
const EXIT_INTERNAL: i32 = 70;

/// Turn any panic into a diagnostic and a defined exit code.
///
/// # Why this exists
///
/// Every failure this program *knows about* already leaves through
/// [`ucal::exit_code`] with an Appendix E code and a sentence. A panic leaves
/// through neither: the default hook prints `thread 'main' panicked at ...`,
/// suggests `RUST_BACKTRACE`, and exits 101 — a Rust implementation detail
/// shown to someone who typed a date wrong, and an exit code that means nothing
/// in §19.5.
///
/// A panic here would be a defect rather than a user error, and the difference
/// is exactly what the message should say. So the hook keeps the location —
/// which is the one genuinely useful part, and what a bug report needs — drops
/// the backtrace machinery, and asks for the report.
///
/// This is a backstop, not a policy. The policy is that the CLI crate contains
/// no panicking construct at all, which `xtask -- lint` enforces
/// (`no-panic-in-cli`). The backstop covers the libraries beneath it, where an
/// `expect` on a genuine invariant is the right code and an unreachable branch
/// is still reachable by a bug.
fn install_panic_handler() {
    std::panic::set_hook(Box::new(|info| {
        let where_ = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "an unknown location".into());
        let what = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "no message".into());
        eprintln!("ucal: internal error — this is a bug in ucal, not in your input.");
        eprintln!("      {what}");
        eprintln!("      at {where_}");
        eprintln!();
        eprintln!("      Nothing was computed incorrectly; the program stopped instead.");
        eprintln!("      Please report it with the command you ran:");
        eprintln!("      https://github.com/vulogov/ucal/issues");
        std::process::exit(EXIT_INTERNAL);
    }));
}

fn main() {
    install_panic_handler();

    // Checking that the handler works needs a panic to hand. An environment
    // variable rather than a flag: it is not part of the CLI surface, does not
    // appear in `--help`, adds no JSON path, and cannot be reached by a user
    // who is not looking for it — the same shape as `UCAL_BLESS` in the test
    // suite. `panic_handler.rs` sets it and asserts on what comes out, so the
    // hook is verified rather than read.
    // ucal-lint-allow-begin(no-panic-in-cli): this *is* the panic, on purpose.
    // It is the only way to check the handler from outside the process, and it
    // is unreachable without setting an environment variable that appears in no
    // help text.
    if std::env::var_os("UCAL_PANIC_SELFTEST").is_some() {
        panic!("induced panic: UCAL_PANIC_SELFTEST is set");
    }
    // ucal-lint-allow-end(no-panic-in-cli)

    let cli = Cli::parse();

    if cli.profile != "UC-1" && cli.profile != "UC1" {
        eprintln!(
            "UCAL-E0002: unknown profile tag `{}`. Only UC-1 exists (Rule P); try `--profile UC-1`, which is the default.",
            cli.profile
        );
        std::process::exit(5);
    }

    // `--sep` was declared, documented as "must not be a digit (§6.3)", and
    // never read: `--sep 1` was accepted in violation of its own help text, and
    // `--sep _` changed nothing while appearing to work. It is validated here
    // and applied at render time through `Render::form_sep`.
    let form_sep = match cli.sep.map(|c| parse_group_sep(&c.to_string())).transpose() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(exit_code(&e));
        }
    };

    // Parsed before dispatch: `--round` reaches both the rendering and
    // `to-civil`'s sub-second field, so it has to exist before either runs.
    let round = match cli.round.as_deref().map(parse_rounding).transpose() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(exit_code(&e));
        }
    };

    // Handled before the rendering machinery, because it is the one command
    // whose output is not a `Doc`: a completion script is a shell program, and
    // there is nothing in it to style, group, round or convert.
    if let Command::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        clap_complete::generate(*shell, &mut cmd, name, &mut std::io::stdout());
        return;
    }
    if let Command::Man { command } = &cli.command {
        // roff on stdout, for the same reason as the completions above: a page
        // written by hand is a second description of the CLI, and this one comes
        // out of the parser that will actually reject the arguments.
        //
        // The top-level page's SUBCOMMANDS section cross-references `ucal-now(1)`
        // and its siblings, which is roff convention and was a dangling promise
        // until this took an argument: `ucal man now` is that page.
        let top = Cli::command();
        let page = match command {
            None => clap_mangen::Man::new(top),
            Some(name) => match top.find_subcommand(name) {
                Some(sub) => {
                    // `Command::name` wants a `&'static str`. Leaking one
                    // short string in a process that is about to write a manual
                    // page and exit is cheaper than threading a lifetime, and it
                    // is bounded by the single call site.
                    let titled: &'static str =
                        Box::leak(format!("ucal-{name}").into_boxed_str());
                    let sub = sub.clone().name(titled);
                    clap_mangen::Man::new(sub)
                }
                None => {
                    eprintln!(
                        "UCAL-E0001: malformed timestamp (no subcommand `{name}`; \
                         `ucal --help` lists them)"
                    );
                    std::process::exit(2);
                }
            },
        };
        if let Err(e) = page.render(&mut std::io::stdout()) {
            eprintln!("ucal: could not write the manual page: {e}");
            std::process::exit(1);
        }
        return;
    }

    let result = match &cli.command {
        Command::Datum => cmd_datum(),
        Command::Doctor => cmd_doctor(),
        #[cfg(feature = "cosmo")]
        Command::Cosmo { what } => match what {
            CosmoCommand::Age {
                z,
                depth,
                scale,
                audit,
            } => ucal::cmd_cosmo_age_audited(z, *depth, *scale, *audit),
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
            CalCommand::Derive { file } => ucal::cmd_cal_derive(file),
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
        Command::Verify => ucal::cmd_verify(),
        Command::Tour => ucal::cmd_tour(),
        // Handled by the early return above; this arm exists only because a
        // `match` must be exhaustive. A diagnostic rather than `unreachable!()`
        // — the CLI crate carries no panicking construct (`no-panic-in-cli`),
        // and if that early return is ever deleted this becomes a message and
        // an exit code instead of an abort.
        Command::Completions { .. } | Command::Man { .. } => {
            Err(ucal_core::TimeError::with_context(
                ucal_core::Code::E0001,
                "internal: this command is handled before dispatch and should \
                 not have reached it",
            ))
        }
        Command::Explain { instant, claim, why } => {
            if *why {
                ucal::cmd_explain_why(instant, *claim)
            } else {
                cmd_explain(instant, *claim)
            }
        }
        Command::Between { from, to, at } => match at {
            Some(a) => LocaleId::parse(&cli.locale)
                .and_then(|l| parse_tier_in(l, a))
                .and_then(|t| ucal::cmd_between(from, to, Some(t))),
            None => ucal::cmd_between(from, to, None),
        },
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
            calendar,
        } => parse_scale(scale)
            // `--round` is global now: a civil label's sub-second field and a
            // rendered rational are rounded by the same declared choice, and
            // half-even is still the default for both.
            .and_then(|s| parse_calendar(calendar).map(|c| (s, c)))
            .and_then(|(s, c)| {
                cmd_to_civil(instant, s, *digits, round.unwrap_or(Rounding::HalfEven), c)
            }),
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
        .form_sep(form_sep)
        .decimals(cli.decimals)
        .round(round)
        .bridge(cli.bridge)
        .width(Render::resolve_width(cli.width, terminal));

    match result {
        Ok(doc) => {
            print!(
                "{}",
                if cli.json {
                    doc.to_json_with(&render)
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
