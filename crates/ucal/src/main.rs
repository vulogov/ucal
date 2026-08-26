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

    /// Locale for tier names, and for the wall clock's chrome (Rule N: names
    /// are display-only). Shipped: en, ru.
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
        /// A body file (§15.1), shown alongside the calendars named by
        /// `--calendars`. Needs `--anchor`: local fields need a phase (Rule J.3).
        #[arg(long, value_name = "FILE", requires = "anchor")]
        body: Option<String>,
        /// The anchor file matching `--body`.
        #[arg(long, value_name = "FILE")]
        anchor: Option<String>,
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
    /// Instants at a tier interval, one per line. `seq`, for time.
    #[cfg(feature = "body")]
    Seq {
        /// The first instant, and the first line of output.
        from: String,
        /// The last instant. The walk stops at or before it.
        to: String,
        /// The interval, as a tier: `T0`, `T1`, `T-3`, or a name like `arc`.
        #[arg(long, default_value = "T1")]
        step: String,
        /// Refuse rather than print more lines than this.
        #[arg(long, default_value = "1000000")]
        max: u64,
    },
    /// A full-screen clock showing universe time. `q` quits.
    #[cfg(feature = "tui")]
    Wallclock {
        /// Theme key, or `list` to name them all.
        #[arg(long, default_value = "plain")]
        theme: String,
        /// Shorthand for `--theme startrek`, which is LCARS.
        #[arg(long, conflicts_with_all = ["gagarin", "armstrong"])]
        startrek: bool,
        /// Shorthand for `--theme gagarin`, a Vostok instrument panel.
        #[arg(long, conflicts_with_all = ["startrek", "armstrong"])]
        gagarin: bool,
        /// Shorthand for `--theme armstrong`, an Apollo DSKY.
        #[arg(long, conflicts_with_all = ["startrek", "gagarin"])]
        armstrong: bool,
        /// A body's own calendar, shown as a further dial: `earth-d`, `mars-d`.
        ///
        /// **Repeatable.** `--clock-local earth-d --clock-local mars-d` is an
        /// airport wall. Only an anchored calendar can be a dial at all (Rule
        /// J.3), which today is two of fifteen — so the wall is a wall of two
        /// until a third anchor is established, and that is a fact about anchors
        /// rather than about this flag.
        ///
        /// `--clock-local` and not `--locale`, which is already this program's
        /// *language* flag (Rule N). The two are different vocabularies and one
        /// name for both is the confusion this project spends its time removing:
        /// `--locale ru` translates the tier names on the face, and
        /// `--clock-local mars-d` puts Mars beside them.
        #[arg(long = "clock-local", value_name = "ID")]
        clock_local: Vec<String>,
        /// The tier in the big readout: `T0`, `T2`, or a name like `sweep`.
        ///
        /// `T0` by default, because it is the tier that moves at a rate a person
        /// watches. Promoting a slower one turns the face from a clock display
        /// into a calendar display, and the face says so — a hand that changes
        /// every 45 years is pixel-identical to a clock that has stopped.
        #[arg(long, value_name = "T")]
        tier: Option<String>,
        /// An origin to count from, shown as an odometer: an instant, or an
        /// event id from `ucal events list`.
        ///
        /// An event is refused when its window is wider than the finest tier on
        /// the face. A reading that ticks 66 000 times a second against a
        /// citation uncertain by two centuries is theatre, and `ucal events show`
        /// already prints that number honestly, which is statically.
        #[arg(long, value_name = "ORIGIN")]
        since: Option<String>,
        /// Draw one frame to stdout and exit, instead of taking over the screen.
        #[arg(long)]
        once: bool,
        /// The instant to draw. Requires `--once`; without it the clock is live.
        #[arg(long, value_name = "INSTANT")]
        at: Option<String>,
        /// Rows for `--once`. Columns come from the global `--width`.
        #[arg(long, default_value = "32", value_name = "N")]
        height: u16,
    },
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
    ///
    /// For a calendar defined by §15.1 files rather than compiled in, use
    /// `ucal cal derive <body> --anchor <anchor> --at <instant>`, which prints
    /// the same derivation from the same code. A second spelling here would be
    /// two ways to ask one question.
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
        /// Path to an anchor file (§15.1), which turns the derivation into a date.
        #[arg(long, value_name = "FILE")]
        anchor: Option<String>,
        /// The instant to render in the derived calendar. Requires --anchor.
        #[arg(long, value_name = "INSTANT")]
        at: Option<String>,
    },
    /// Check a §15.1 file: does it load, and does a calendar follow from it?
    ///
    /// Two questions, because they have different answers. A file can be
    /// well-formed and still derive nothing — a body whose year is a whole
    /// number of its solar days has no fractional day to intercalate — and an
    /// author reading a red exit code from `cal derive` cannot tell which of the
    /// two they have.
    Validate {
        /// A body file, an anchor file, or a shipped calendar id like `mars-d`.
        ///
        /// A path wins over an id: a caller who names a file that exists means
        /// that file.
        #[arg(required_unless_present = "all")]
        file: Option<String>,
        /// Every shipped calendar at once: which published figure decides which
        /// leap rule, and which figures more than one calendar rests on.
        #[arg(long, conflicts_with = "anchor")]
        all: bool,
        /// An anchor file to check the body file against.
        #[arg(long, value_name = "FILE")]
        anchor: Option<String>,
    },
}

#[cfg(feature = "civil")]
fn parse_scale(s: &str) -> Result<Scale, ucal_core::TimeError> {
    match s {
        "tt" => Ok(Scale::Tt),
        "tai" => Ok(Scale::Tai),
        "utc" => Ok(Scale::Utc),
        _ => Err(ucal_core::TimeError::with_context(
            ucal_core::Code::E0018,
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
            ucal_core::Code::E0018,
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
            ucal_core::Code::E0018,
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
    // Plain lines, not a `Doc`, for the same reason `completions` is: a
    // generator's output is an input to something else — here `ucal to-civil -`.
    #[cfg(feature = "body")]
    if let Command::Seq {
        from,
        to,
        step,
        max,
    } = &cli.command
    {
        // G2 — `--tick-sep` reaches here too. It is a global flag and every
        // other command honours it; `seq` ran before the flag was even parsed,
        // so passing it did nothing and said nothing, which is worse than
        // refusing it. A caller who asks for separators in a stream they meant
        // to pipe into `ucal to-civil -` gets a loud failure from the parser at
        // the other end rather than a silent one here.
        let out = cli
            .tick_sep
            .as_deref()
            .map(parse_group_sep)
            .transpose()
            .and_then(|sep| {
                LocaleId::parse(&cli.locale)
                    .and_then(|l| parse_tier_in(l, step))
                    .and_then(|t| ucal::cmd_seq(from, to, t, *max))
                    .map(|lines| (sep, lines))
            });
        match out {
            Ok((sep, lines)) => {
                // `PLAIN`, because this runs before the style is resolved and
                // a generator cannot borrow colours from machinery that has not
                // started. Grouping is not colour: it is what the caller asked
                // for in as many words.
                let render = Render::PLAIN.group(sep);
                for l in lines {
                    println!("{}", ucal::style::group_decimal(&render, &l));
                }
                return;
            }
            Err(e) => {
                // Before the style is resolved, so this one prints plain. A
                // generator that runs before the rendering machinery cannot
                // borrow its colours from it.
                eprintln!("{e}");
                std::process::exit(exit_code(&e));
            }
        }
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

    // F2. `-` as an instant reads lines from stdin, and the whole dispatch runs
    // once per line. `over` substitutes the line for whichever argument carried
    // the `-`; every other argument is the one the caller typed, so a streamed
    // run and a single run differ in exactly one value.
    let streaming = streamed(&cli.command);
    let mut lines: Vec<String> = Vec::new();
    if streaming {
        use std::io::BufRead;
        for line in std::io::stdin().lock().lines() {
            match line {
                Ok(l) if !l.trim().is_empty() => lines.push(l.trim().to_string()),
                Ok(_) => {}
                Err(_) => {
                    eprintln!("could not read stdin");
                    std::process::exit(2);
                }
            }
        }
    }

    fn pick<'x>(replacement: Option<&'x str>, typed: &'x str) -> &'x str {
        replacement.unwrap_or(typed)
    }
    let dispatch = |replacement: Option<&str>| -> ucal::CmdResult {
        match &cli.command {
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
            CalCommand::Show { id, instant } => ucal::cmd_cal_show(id, pick(replacement, instant)),
            CalCommand::Anchor { id } => ucal::cmd_cal_anchor(id),
            CalCommand::Validate { file, all, anchor } => {
                if *all {
                    ucal::cmd_cal_validate_all()
                } else {
                    match file {
                        Some(f) => ucal::cmd_cal_validate(f, anchor.as_deref()),
                        // Unreachable: clap's `required_unless_present` has
                        // already refused. Said rather than unwrapped.
                        None => Err(ucal_core::TimeError::with_context(
                            ucal_core::Code::E0018,
                            "cal validate needs a file, a calendar id, or --all",
                        )),
                    }
                }
            }
            CalCommand::Derive { file, anchor, at } => {
                ucal::cmd_cal_derive_with(file, anchor.as_deref(), at.as_deref())
            }
        },
        #[cfg(all(feature = "body", feature = "civil"))]
        Command::Show {
            instant,
            calendars,
            body,
            anchor,
        } => {
            let ids: Vec<String> = calendars.split(',').map(|s| s.trim().to_string()).collect();
            match (body, anchor) {
                (Some(b), Some(a)) => {
                    ucal::cmd_show_with_file(pick(replacement, instant), &ids, b, a)
                }
                _ => ucal::cmd_show(pick(replacement, instant), &ids),
            }
        }
        Command::Ladder { named_only } => {
            LocaleId::parse(&cli.locale).and_then(|l| cmd_ladder(l, *named_only))
        }
        Command::Verify => ucal::cmd_verify(),
        // Handled by the early return above, like `completions` and `man`: its
        // output is lines, not a document. A diagnostic rather than
        // `unreachable!()`, because this crate carries no panicking construct.
        #[cfg(feature = "body")]
        Command::Seq { .. } => Err(ucal_core::TimeError::with_context(
            ucal_core::Code::E0001,
            "internal: `seq` is handled before dispatch and should not have reached it",
        )),
        Command::Tour => ucal::cmd_tour(),
        // Only `--theme list` reaches here; every other value ran the clock and
        // returned above.
        #[cfg(feature = "tui")]
        Command::Wallclock { .. } => Ok(ucal::cmd_wallclock_themes()),
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
                ucal::cmd_explain_why(pick(replacement, instant), *claim)
            } else {
                cmd_explain(pick(replacement, instant), *claim)
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
                cmd_to_civil(pick(replacement, instant), s, *digits, round.unwrap_or(Rounding::HalfEven), c)
            }),
        }
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
    let terminal = terminal_width();
    // Also not a `Doc`: it owns the terminal until the user quits, and its
    // output is the screen rather than a document. `--theme list` *is* a Doc,
    // and falls through to the dispatch below.
    #[cfg(feature = "tui")]
    if let Command::Wallclock {
        theme,
        startrek,
        gagarin,
        armstrong,
        clock_local,
        tier,
        since,
        once,
        at,
        height,
    } = &cli.command
    {
        // The shorthands are mutually exclusive by `conflicts_with_all`, so at
        // most one is set and the order here cannot hide a second choice.
        let key = match (*startrek, *gagarin, *armstrong) {
            (true, _, _) => "startrek",
            (_, true, _) => "gagarin",
            (_, _, true) => "armstrong",
            _ => theme.as_str(),
        };
        if key != "list" {
            // `--width` is the global flag, and off a terminal it resolves to
            // the 80-column baseline — which is the size a committed frame
            // should be, and the reason this reuses it rather than adding a
            // second width nobody would keep in step with the first.
            let cols = Render::resolve_width(cli.width, terminal) as u16;
            let result = LocaleId::parse(&cli.locale).and_then(|l| {
                let mut dials =
                    ucal::wallclock::Dials::new(l)?.with_clock_local(clock_local);
                if let Some(t) = tier {
                    dials = dials.with_hero(ucal::parse_tier(t)?);
                }
                if let Some(origin) = since {
                    let (t, label) = ucal::wallclock_origin(origin)?;
                    dials = dials.with_since(t, label);
                }
                if *once {
                    ucal::wallclock::run_once_with(
                        key,
                        &dials,
                        at.as_deref(),
                        cols,
                        *height,
                        // The *resolved* style, not the raw `--color` choice.
                        // `NO_COLOR` and "is this a terminal" are decided in
                        // `resolve_for_output`, so asking the choice gets the
                        // answer before those applied — which put ANSI escapes
                        // into the frame `gen-examples` commits, the one thing
                        // `frame.rs` says must not happen.
                        !style.is_plain(),
                    )
                    .map(|frame| print!("{frame}"))
                } else if at.is_some() {
                    Err(ucal_core::TimeError::with_context(
                        ucal_core::Code::E0018,
                        "--at draws a fixed instant and only makes sense with --once; \
                         a live clock's instant is now",
                    ))
                } else {
                    ucal::wallclock::run_with(key, &dials)
                }
            });
            if let Err(e) = result {
                eprintln!("{e}");
                std::process::exit(exit_code(&e));
            }
            return;
        }
    }

    let sep = match cli.tick_sep.as_deref().map(parse_group_sep).transpose() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(exit_code(&e));
        }
    };
    let render = Render::styled(style)
        .group(sep)
        .form_sep(form_sep)
        .decimals(cli.decimals)
        .round(round)
        .bridge(cli.bridge)
        .width(Render::resolve_width(cli.width, terminal));

    // F2: one document per input line, in whichever form the caller asked for.
    if streaming {
        let mut bad = 0;
        for line in &lines {
            match dispatch(Some(line)) {
                Ok(doc) => {
                    if cli.json {
                        // JSON Lines: one record, one line, so the stream is a
                        // filter rather than a concatenation of documents.
                        println!("{}", compact_json(&doc.to_json_with(&render)));
                    } else {
                        print!("{}", doc.render(&render));
                    }
                }
                Err(e) => {
                    // A bad line is reported and the stream continues: a filter
                    // that dies on line 3 of 10 000 has thrown away the other
                    // 9 999 answers. The exit code still says something went
                    // wrong, so a script cannot mistake a partial run for a
                    // clean one.
                    let err_style = error_style(choice);
                    eprintln!("{}: {}", line, err_style.paint(Role::Error, &e.to_string()));
                    bad += 1;
                }
            }
        }
        if bad > 0 {
            std::process::exit(6);
        }
        return;
    }

    let result = dispatch(None);
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

/// The terminal's columns, or `None` off a terminal.
///
/// Off a terminal the width is the baseline, always: if it followed the terminal
/// on a redirected stream, `ucal ladder > f` and `ucal ladder | cat` would
/// differ, and so would the same command on two machines.
///
/// `wallclock --once` depends on that being true. A frame committed into the
/// documentation must not change width with the window it was generated in, and
/// `gen-examples` redirects, so it gets the baseline by the same rule everything
/// else does rather than by a special case.
fn terminal_width() -> Option<usize> {
    if std::io::stdout().is_terminal() {
        terminal_size::terminal_size().map(|(terminal_size::Width(w), _)| w as usize)
    } else {
        None
    }
}

/// Does this invocation read its instant from stdin?
///
/// F2. `-` is the conventional spelling, and the commands that accept it are the
/// ones taking **exactly one** instant. `between` and `ruler` take two, and a
/// line-oriented filter has no natural answer for which of the two a line is —
/// so they do not accept it rather than accepting it and guessing.
fn streamed(cmd: &Command) -> bool {
    let one: Option<&String> = match cmd {
        // Gated for the same reason the variants are. `streamed` named
        // `Command::ToCivil` unconditionally and the `features` workflow caught
        // it on `--no-default-features --features u512,std`, where `civil` is
        // absent and the variant does not exist — the fourth time that workflow
        // has found a feature-gating miss by building a combination nobody
        // would type.
        #[cfg(feature = "civil")]
        Command::ToCivil { instant, .. } => Some(instant),
        Command::Explain { instant, .. } => Some(instant),
        #[cfg(all(feature = "body", feature = "civil"))]
        Command::Show { instant, .. } => Some(instant),
        #[cfg(feature = "body")]
        Command::Cal { what } => match what {
            CalCommand::Show { instant, .. } => Some(instant),
            _ => None,
        },
        _ => None,
    };
    one.is_some_and(|i| i == "-")
}

/// One JSON document on one line.
///
/// F2. A stream is only a filter if each record is a line, and `to_json_with`
/// pretty-prints — so this removes the whitespace *between* tokens and touches
/// nothing inside a string.
///
/// Deliberately a post-pass over the one serialiser rather than a second
/// serialiser. A compact writer beside the pretty one would be two descriptions
/// of the `ucal-json/1` surface, and `fixtures/json-surface.txt` pins only one
/// of them — which is exactly how the two would come to disagree.
fn compact_json(pretty: &str) -> String {
    let mut out = String::with_capacity(pretty.len());
    let mut in_string = false;
    let mut escaped = false;
    for ch in pretty.chars() {
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => {
                in_string = true;
                out.push(ch);
            }
            ' ' | '\n' | '\t' | '\r' => {}
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod stream_tests {
    use super::compact_json;

    /// Compaction removes layout and preserves content.
    #[test]
    fn compaction_touches_nothing_inside_a_string() {
        let pretty = "{\n  \"a\": \"two words\",\n  \"b\": [\n    1,\n    2\n  ]\n}";
        assert_eq!(compact_json(pretty), "{\"a\":\"two words\",\"b\":[1,2]}");
    }

    /// An escaped quote does not end the string early.
    ///
    /// The failure this guards against is silent: a citation containing `\"`
    /// would flip the parser's idea of where the string ends and the rest of the
    /// document would have its spaces eaten.
    #[test]
    fn an_escaped_quote_does_not_end_the_string() {
        let pretty = "{\"q\": \"a \\\" b\",  \"n\": 1}";
        assert_eq!(compact_json(pretty), "{\"q\":\"a \\\" b\",\"n\":1}");
    }

    /// And a trailing backslash inside a string is not treated as an escape of
    /// the closing quote.
    #[test]
    fn a_literal_backslash_is_handled() {
        let pretty = "{\"p\": \"a\\\\\", \"n\": 2}";
        assert_eq!(compact_json(pretty), "{\"p\":\"a\\\\\",\"n\":2}");
    }
}
