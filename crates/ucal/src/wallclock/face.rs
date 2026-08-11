//! What the clock shows, and how it is drawn.
//!
//! [`Face`] is the reading — a set of hand positions and nothing about colour or
//! layout — so it can be built and asserted on without a terminal. Rendering
//! takes a [`Theme`] and puts it on a frame.

use super::dial;
use super::digits;
use super::theme::{Layout, Theme};
use ratatui::layout::{Alignment, Constraint, Direction, Layout as Layout2, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use ucal_core::backend::TickInt;
use ucal_core::{Instant, LocaleId, Ticks, Tier, TimeError, UC1};

/// One tier's hand: which of the 3125 subdivisions of the tier above it we are
/// currently in.
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Hand {
    /// The tier this hand belongs to.
    pub tier: Tier,
    /// Its name in the active locale, where it has one.
    ///
    /// Owned rather than `&'static str` because Rule N makes a tier's name
    /// locale-scoped, and a localised name is looked up rather than compiled in.
    pub name: String,
    /// `0..3125`.
    pub position: u32,
}

impl Hand {
    /// The rail label: tier index and, where it has one, its localised name.
    pub fn label(&self) -> String {
        if self.name.is_empty() {
            self.tier.to_string()
        } else {
            format!("{} {}", self.tier, self.name)
        }
    }

    /// How far round its dial, in thousandths.
    ///
    /// Integer, because a clock face is not a place to introduce a float into a
    /// program that has spent nine releases keeping them out (Rule E).
    pub fn per_mille(&self) -> u32 {
        self.position * 1000 / 3125
    }
}

/// The tiers a wall clock puts on its face.
///
/// `T3` down to `T-1`. Above `T3` a hand does not move within a human lifetime —
/// one `T4` is 141 000 years — and below `T-1` it moves 66 000 times a second,
/// which no refresh rate reaches. Both ends are excluded for the same reason,
/// which is the reason the module header sets out.
const FACE_TIERS: [i8; 5] = [3, 2, 1, 0, -1];

impl Local {
    /// Read a body calendar's local fields at an instant.
    ///
    /// `UCAL-E0062` for a calendar that exists and has no anchor, which is ten
    /// of the twelve that ship. That is the ordinary case and not a gap: a
    /// second dial needs a phase, phase is empirical (Rule J), and D5 recorded
    /// what establishing one honestly costs. The error says so rather than
    /// showing a dial with no hand on it.
    pub fn read(id: &str, t: &Instant<UC1>) -> Result<Local, TimeError> {
        let cal = ucal_body::calendar::by_id(id)?;
        let f = cal.fields(t)?;
        // Percent through the local day, computed as a ratio and floored — no
        // float reaches this (Rule E), and the value is a hand position rather
        // than a measurement, so flooring is the honest rounding.
        let hundred = ucal_core::Ratio::from_u64(100);
        let through = f
            .day_fraction
            .mul(&hundred)
            .map(|r| r.floor().to_dec_string())
            .unwrap_or_default()
            .parse::<u32>()
            .unwrap_or(0);
        Ok(Local {
            calendar: id.to_string(),
            year: f.year.to_string(),
            day: f.day.to_string(),
            through_day: through.min(100),
            revision: f.anchor_revision,
        })
    }
}

/// A reading of the clock.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Face {
    /// The instant this face was read at.
    pub at: Instant<UC1>,
    /// The hands, coarsest first.
    pub hands: Vec<Hand>,
    /// The full `UC1` human form, for the line under the readout.
    pub human: String,
    /// A second dial: a body's own calendar, where one was asked for.
    ///
    /// A wall clock with a second face, and the analogue is exact — the second
    /// face on a real one shows another *place*, and so does this. What it shows
    /// is local fields, which need a phase, so only an anchored calendar can
    /// appear here (Rule J.3).
    pub local: Option<Local>,
}

/// The second dial.
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Local {
    /// The calendar id, e.g. `mars-d`.
    pub calendar: String,
    /// Local year, **1-based from the anchor**.
    ///
    /// Not a Gregorian year and not an offset from one. Year 1 is the year that
    /// began at the anchor, so `earth-d` year 27 is the twenty-seventh year
    /// since 2000-01-01 — which lands in Gregorian 2026, one less than a reader
    /// subtracting naively would guess, because the count starts at one.
    ///
    /// The face says so in words. A bare `year 27` invites exactly two wrong
    /// readings, "2027" and "2000 + 27", and the first person to see it asked.
    /// §15.5 defines the fields and names no era to count from instead, which
    /// [`X1-authoring-local-calendars.md`] records as a gap; until there is one,
    /// the display's job is to say what the number counts.
    ///
    /// The Gregorian equivalent is deliberately *not* shown: an Earth label
    /// beside a local one is the substitution Rule A.5 refuses, and
    /// `ucal to-civil` is the conversion for anyone who wants it.
    ///
    /// [`X1-authoring-local-calendars.md`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/X1-authoring-local-calendars.md
    pub year: String,
    /// Day of the local year.
    pub day: String,
    /// How far through the local day, as a percentage — the hand of that dial.
    pub through_day: u32,
    /// Which anchor revision produced it. Anchors are observations (Rule J).
    pub revision: u32,
}

impl Face {
    /// Read the system clock.
    pub fn read_now(locale: LocaleId, clock_local: Option<&str>) -> Result<Face, TimeError> {
        Face::at(super::now_instant()?, locale, clock_local)
    }

    /// Read a given instant, which is what the tests use.
    /// `locale` names the language the tiers are drawn in; `clock_local` names
    /// a place, and is a calendar id.
    pub fn at(
        t: Instant<UC1>,
        locale: LocaleId,
        clock_local: Option<&str>,
    ) -> Result<Face, TimeError> {
        let ticks = t.ticks();
        let mut hands = Vec::new();
        for k in FACE_TIERS {
            let tier = Tier::new(k)?;
            // position = (ticks / tier) mod 3125. Each rung is 5^5 of the one
            // below, so a tier's hand has 3125 stops — the same relationship the
            // printed form's groups have, computed rather than parsed out of it.
            let (q, _) = ticks.quot_rem(&tier.ticks());
            let (_, r) = q.quot_rem(&<Ticks as TickInt>::from_u64(3125));
            let position = r.to_dec_string().parse::<u32>().unwrap_or(0);
            hands.push(Hand {
                tier,
                // Rule N: a tier's name is locale-scoped and display-only. The
                // face is display, so it uses the locale's name; the *index* is
                // beside it and is not, which is what a reader compares across
                // two machines set to different languages.
                name: match ucal_core::tier::name_of(tier) {
                    Some(_) => ucal_core::locale::display(locale, tier),
                    None => String::new(),
                },
                position,
            });
        }
        let local = match clock_local {
            None => None,
            Some(id) => Some(Local::read(id, &t)?),
        };
        Ok(Face {
            human: crate::render_at(&t, Tier::new(0)?),
            at: t,
            hands,
            local,
        })
    }

    /// The hand a reader watches: `T0`, the beat, at about 21 per second.
    pub fn beat(&self) -> Option<&Hand> {
        self.hands.iter().find(|h| h.tier.index() == 0)
    }

    /// The hand below it, which is a blur and is drawn as one.
    pub fn blur(&self) -> Option<&Hand> {
        self.hands.iter().find(|h| h.tier.index() == -1)
    }

    /// Draw the face.
    pub fn render(&self, f: &mut Frame, theme: &Theme) {
        let area = f.area();
        f.render_widget(
            Block::default().style(Style::default().bg(theme.background)),
            area,
        );
        match theme.layout {
            Layout::Plain => self.render_plain(f, area, theme),
            Layout::Lcars => self.render_lcars(f, area, theme),
            Layout::Targeting => self.render_targeting(f, area, theme),
            Layout::Panel => self.render_panel(f, area, theme),
            Layout::Dsky => self.render_dsky(f, area, theme),
            Layout::Orbit => self.render_orbit(f, area, theme),
        }
    }

    // ---- plain -----------------------------------------------------------

    fn render_plain(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let rows = Layout2::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(digits::HEIGHT as u16 + 1),
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(area);

        f.render_widget(
            Paragraph::new("UCAL — universe calendar")
                .style(Style::default().fg(theme.label)),
            rows[0],
        );
        self.render_readout(f, rows[1], theme, Alignment::Left);
        let mut body = self.hand_lines(theme);
        body.extend(self.local_lines(theme));
        f.render_widget(Paragraph::new(body), rows[2]);
        f.render_widget(
            Paragraph::new(vec![
                Line::styled(self.human.clone(), Style::default().fg(theme.text)),
                Line::styled("q to quit", Style::default().fg(theme.label)),
            ]),
            rows[3],
        );
    }

    // ---- LCARS -----------------------------------------------------------

    /// The elbow, the rail, and the readout.
    ///
    /// LCARS is a structure before it is a palette: a header bar that turns a
    /// corner into a vertical rail of blocks, everything rounded on the outside
    /// of the turn and square on the inside, and numbers set hard against the
    /// rail. Drawing it in a terminal means the corner is a block and the
    /// rounding is a half-block, which is as close as a character cell gets.
    fn render_lcars(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let cols = Layout2::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(18), Constraint::Min(0)])
            .split(area);
        let rail = cols[0];
        let main = cols[1];

        // The elbow: a solid block at the top of the rail, joined to the header.
        let rail_rows = Layout2::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(rail);
        f.render_widget(
            Paragraph::new(vec![
                Line::from("▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄"),
                Line::from("████████████████"),
                Line::from("████████████████"),
                Line::from("███████▀▀▀▀▀▀▀▀▀"),
            ])
            .style(Style::default().fg(theme.primary)),
            rail_rows[0],
        );

        // The rail blocks: one per hand, each a coloured bar with its name and
        // position. This is the LCARS idiom — a stack of labelled buttons that
        // are not buttons.
        let hands = &self.hands;
        let block_rows = Layout2::default()
            .direction(Direction::Vertical)
            .constraints(
                hands
                    .iter()
                    .map(|_| Constraint::Length(3))
                    .chain(core::iter::once(Constraint::Min(0)))
                    .collect::<Vec<_>>(),
            )
            .split(rail_rows[1]);

        // The rail runs to the bottom of the screen. An LCARS rail that stopped
        // where its content did would read as an unfinished panel — the frame is
        // structural in that design language, not decoration around the data.
        if let Some(tail) = block_rows.last() {
            let colour = theme.blocks[hands.len() % theme.blocks.len()];
            let filler: Vec<Line> = (0..tail.height)
                .map(|i| {
                    Line::from(Span::styled(
                        if i == 0 { "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄" } else { "                " },
                        Style::default().fg(colour).bg(if i == 0 {
                            theme.background
                        } else {
                            colour
                        }),
                    ))
                })
                .collect();
            f.render_widget(Paragraph::new(filler), *tail);
        }
        for (i, h) in hands.iter().enumerate() {
            let colour = theme.blocks[i % theme.blocks.len()];
            let label = h.label();
            if let Some(r) = block_rows.get(i) {
                f.render_widget(
                    Paragraph::new(vec![
                        Line::from(Span::styled(
                            "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄",
                            Style::default().fg(colour),
                        )),
                        Line::from(Span::styled(
                            format!("{:>15} ", label.to_uppercase()),
                            Style::default().bg(colour).fg(theme.background),
                        )),
                        Line::from(Span::styled(
                            format!("{:>15} ", h.position),
                            Style::default().fg(colour).add_modifier(Modifier::BOLD),
                        )),
                    ]),
                    *r,
                );
            }
        }

        // The main panel.
        let main_rows = Layout2::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(digits::HEIGHT as u16 + 1),
                Constraint::Length(2),
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(main);

        f.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    " UNIVERSE CALENDAR · UC1",
                    Style::default()
                        .bg(theme.primary)
                        .fg(theme.background)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    " ABSOLUTE TIME · PLANCK TICKS · BASE FIVE",
                    Style::default().fg(theme.label),
                )),
                Line::from(""),
            ]),
            main_rows[0],
        );

        self.render_readout(f, main_rows[1], theme, Alignment::Left);

        // The arc: the one hand that moves at a pace a person reads rather than
        // watches, at one stop every 2 min 26 s. It gets a line of its own
        // because it is the hand a reader would actually use to tell the time.
        let arc = self
            .hands
            .iter()
            .find(|h| h.tier.index() == 1)
            .map_or(0, |h| h.position);
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" T1 ARC ", Style::default().bg(theme.blocks[2]).fg(theme.background)),
                Span::styled(
                    format!("  {arc:04}  "),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "ONE STOP EVERY 2 MIN 26 S",
                    Style::default().fg(theme.label),
                ),
            ])),
            main_rows[2],
        );

        self.render_blur(f, main_rows[3], theme);
        let mut tail = vec![
            Line::from(""),
            Line::styled(self.human.clone(), Style::default().fg(theme.text)),
        ];
        tail.extend(self.local_lines(theme));
        f.render_widget(Paragraph::new(tail), main_rows[4]);
        // The bottom bar, which LCARS always has.
        f.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄",
                    Style::default().fg(theme.primary),
                )),
                Line::from(Span::styled(
                    " Q TO DISENGAGE",
                    Style::default().bg(theme.primary).fg(theme.background),
                )),
            ]),
            main_rows[5],
        );
    }

    // ---- targeting -------------------------------------------------------

    /// A gunsight.
    ///
    /// Structurally the opposite of LCARS, which is why it is a layout and not a
    /// palette. LCARS is a console — coloured blocks, generous space, an
    /// interface for reading. A gunsight is an instrument: a frame at the edge
    /// of vision, a reticle in the middle holding the one number that matters,
    /// and everything else compressed into a strip along the bottom.
    //
    // # Rule O, and why a region rather than seven line markers
    //
    // Every saturating subtraction below is a `u16` column or row count. A pane
    // two cells narrower than its border is a pane of zero cells, which is the
    // right answer and the only one; Rule O exists because a silently clamped
    // *duration* is a wrong answer where an error was available.
    //
    // The region is bounded by types rather than by discipline, which is what
    // makes it safe to draw this wide: nothing in scope here is a quantity in
    // this calendar. `Rect` is `u16`s, `Hand::position` is a `u32` under 3125,
    // and the only `Ratio` in the whole module is inside `Local::read`, which is
    // a different function in a different impl.
    // ucal-lint-allow-begin(no-wrapping-arithmetic): u16 terminal geometry only
    fn render_targeting(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let w = area.width;
        let h = area.height;
        let amber = Style::default().fg(theme.text);
        let dim = Style::default().fg(theme.label);
        let hot = Style::default().fg(theme.blocks[1 % theme.blocks.len()]);
        let lock = Style::default().fg(theme.blocks[2 % theme.blocks.len()]);

        // The canopy frame: corner brackets only. A closed box would be a
        // window; brackets are what a HUD draws, because the pilot is looking
        // through the middle of it.
        let corner = 6.min(w.saturating_sub(2) / 2) as usize;
        let bar: String = core::iter::repeat_n('─', corner).collect();
        let pad: String = core::iter::repeat_n(' ', (w as usize).saturating_sub(corner * 2 + 2)).collect();
        f.render_widget(
            Paragraph::new(Line::styled(format!("┌{bar}{pad}{bar}┐"), amber)),
            Rect::new(area.x, area.y, w, 1),
        );
        if h >= 2 {
            f.render_widget(
                Paragraph::new(Line::styled(format!("└{bar}{pad}{bar}┘"), amber)),
                Rect::new(area.x, area.y + h - 1, w, 1),
            );
        }

        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            w.saturating_sub(2),
            h.saturating_sub(2),
        );
        if inner.height < 6 || inner.width < 18 {
            return;
        }

        let rows = Layout2::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(digits::HEIGHT as u16 + 2),
                Constraint::Length(2),
                Constraint::Min(0),
            ])
            .split(inner);

        // Header: what this is, and the slow hand as a "lock" readout.
        let arc = self
            .hands
            .iter()
            .find(|hh| hh.tier.index() == 1)
            .map_or(0, |hh| hh.position);
        f.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(" TARGETING · UC1", amber),
                    Span::styled(
                        format!("{:>width$}", format!("LOCK T1 {arc:04} "), width = (rows[0].width as usize).saturating_sub(16)),
                        lock,
                    ),
                ]),
                Line::styled(
                    " ABSOLUTE TIME · PLANCK TICKS · BASE FIVE",
                    dim,
                ),
            ]),
            rows[0],
        );

        // The reticle: corner ticks around the readout, and the readout inside.
        let rw = rows[1].width as usize;
        let tick = 3.min(rw / 4);
        let inner_pad: String = core::iter::repeat_n(' ', rw.saturating_sub(tick * 2 + 2)).collect();
        let tick_bar: String = core::iter::repeat_n('─', tick).collect();
        let mut reticle: Vec<Line> = Vec::new();
        reticle.push(Line::styled(
            format!("┌{tick_bar}{inner_pad}{tick_bar}┐"),
            hot,
        ));
        let beat = self.beat().map_or(0, |hh| hh.position);
        for row in digits::render(&format!("{beat:04}")) {
            let body = format!("{row:^width$}", width = rw.saturating_sub(2));
            reticle.push(Line::from(vec![
                Span::styled("│", hot),
                Span::styled(body, Style::default().fg(theme.primary)),
                Span::styled("│", hot),
            ]));
        }
        reticle.push(Line::styled(
            format!("└{tick_bar}{inner_pad}{tick_bar}┘"),
            hot,
        ));
        f.render_widget(Paragraph::new(reticle), rows[1]);

        // The crosshair, with the flicker riding it. The bar and the sight line
        // are the same row on purpose: the fastest hand is the one a gunsight
        // would put on the axis.
        let cross_w = rows[2].width as usize;
        let filled = self.blur().map_or(0, |b| b.per_mille() as usize) * cross_w / 1000;
        let cross: String = (0..cross_w)
            .map(|i| {
                if i == cross_w / 2 {
                    '┼'
                } else if i < filled {
                    '━'
                } else {
                    '─'
                }
            })
            .collect();
        f.render_widget(
            Paragraph::new(vec![
                Line::styled(cross, hot),
                Line::styled(" T-1 FLICKER ON THE AXIS · 66 000 PER SECOND", dim),
            ]),
            rows[2],
        );

        // The HUD strip: every hand on one line, because an instrument does not
        // give a number its own panel.
        let strip: String = self
            .hands
            .iter()
            .map(|hh| format!("{} {:04}", hh.label().to_uppercase(), hh.position))
            .collect::<Vec<_>>()
            .join("   ");
        let mut tail = vec![
            Line::styled(format!(" {strip}"), amber),
            Line::from(""),
            Line::styled(format!(" {}", self.human), dim),
        ];
        tail.extend(self.local_lines(theme));
        tail.push(Line::from(""));
        tail.push(Line::styled(" [Q] DISENGAGE", hot));
        f.render_widget(Paragraph::new(tail), rows[3]);
    }
    // ucal-lint-allow-end(no-wrapping-arithmetic)

    // ---- panel (Vostok) --------------------------------------------------

    /// An enamelled plate with gauges set into it.
    ///
    /// The oldest tradition of the five and the only light one, because the
    /// object was: a pale panel built in 1960 to be read and operated with a
    /// glove on. Where LCARS is a screen and a gunsight is a projection, this is
    /// a surface — bezelled instruments in a row, each with an engraved plate
    /// under it saying what it measures.
    ///
    /// The chrome is Cyrillic and the tier names are not: `--locale` decides a
    /// name's language (Rule N) and a theme does not get to override it. The
    /// intended pairing is `--gagarin --locale ru`.
    // ucal-lint-allow-begin(no-wrapping-arithmetic): u16 terminal geometry only
    fn render_panel(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let ink = Style::default().fg(theme.text);
        let engraved = Style::default().fg(theme.label);
        let lamp = Style::default().fg(theme.blocks[1 % theme.blocks.len()]);
        let ready = Style::default().fg(theme.blocks[2 % theme.blocks.len()]);

        let rows = Layout2::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(digits::HEIGHT as u16 + 4),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        // The title plate.
        f.render_widget(
            Paragraph::new(vec![
                Line::styled(" ВРЕМЯ ВСЕЛЕННОЙ · UC1", ink),
                Line::styled(" АБСОЛЮТНОЕ ВРЕМЯ · ПЛАНКОВСКИЕ ТИКИ · ОСНОВАНИЕ ПЯТЬ", engraved),
                Line::styled(" ───────────────────────────────────────────────────", engraved),
            ]),
            rows[0],
        );

        // A row of gauges for the slow hands, each in a bezel over its plate.
        let slow: Vec<&Hand> = self
            .hands
            .iter()
            .filter(|h| h.tier.index() >= 1)
            .collect();
        if !slow.is_empty() {
            let cells = Layout2::default()
                .direction(Direction::Horizontal)
                .constraints(
                    slow.iter()
                        .map(|_| Constraint::Length(16))
                        .chain(core::iter::once(Constraint::Min(0)))
                        .collect::<Vec<_>>(),
                )
                .split(rows[1]);
            for (i, h) in slow.iter().enumerate() {
                if let Some(r) = cells.get(i) {
                    f.render_widget(
                        Paragraph::new(vec![
                            Line::styled(" ┌────────────┐", ink),
                            Line::from(vec![
                                Span::styled(" │", ink),
                                Span::styled(format!("{:^12}", h.position), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                                Span::styled("│", ink),
                            ]),
                            Line::styled(" └────────────┘", ink),
                            Line::styled(format!("{:^15}", h.label().to_uppercase()), engraved),
                        ]),
                        *r,
                    );
                }
            }
        }

        // The main instrument.
        let beat = self.beat().map_or(0, |h| h.position);
        let mut main = vec![Line::styled(" ┌──────────────────────────────┐", ink)];
        for row in digits::render(&format!("{beat:04}")) {
            main.push(Line::from(vec![
                Span::styled(" │ ", ink),
                Span::styled(format!("{row:<28}"), Style::default().fg(theme.primary)),
                Span::styled("│", ink),
            ]));
        }
        main.push(Line::styled(" └──────────────────────────────┘", ink));
        // The plate carries the tier's name in the active locale, like every
        // other plate on this panel. It read `T0 · ОСНОВНОЙ ОТСЧЁТ` first,
        // which made the main instrument the one gauge whose label ignored
        // `--locale` — chrome and name are separate here, and that had quietly
        // mixed them.
        let beat_label = self
            .beat()
            .map_or_else(|| "T0".to_string(), |h| h.label().to_uppercase());
        main.push(Line::styled(
            format!("        {beat_label} · ОСНОВНОЙ ОТСЧЁТ"),
            engraved,
        ));
        f.render_widget(Paragraph::new(main), rows[2]);

        // The lamp: the sub-visible tier, as a red bar rather than a number.
        let width = rows[3].width.saturating_sub(4) as usize;
        let filled = self.blur().map_or(0, |b| b.per_mille() as usize) * width / 1000;
        let bar: String = core::iter::repeat_n('▆', filled)
            .chain(core::iter::repeat_n('▁', width.saturating_sub(filled)))
            .collect();
        f.render_widget(
            Paragraph::new(vec![
                Line::from(vec![Span::styled(format!(" ● {bar}"), lamp)]),
                Line::styled("   T-1 · 66 000 В СЕКУНДУ · ПОЛОЖЕНИЕ, НЕ ЧИСЛО", engraved),
            ]),
            rows[3],
        );

        let mut tail = vec![Line::styled(format!(" {}", self.human), ink)];
        tail.extend(self.local_lines(theme));
        tail.push(Line::from(""));
        tail.push(Line::from(vec![
            Span::styled(" ● ГОТОВ", ready),
            Span::styled("     [Q] ВЫХОД", engraved),
        ]));
        f.render_widget(Paragraph::new(tail), rows[4]);
    }

    // ---- DSKY (Apollo) ---------------------------------------------------

    /// A verb, a noun, and three registers.
    ///
    /// The other answer to the same decade. Vostok's panel was a surface you
    /// read; this was a terminal you addressed — two digits for a verb, two for
    /// a noun, and three numeric registers showing whatever you had just asked
    /// for. `V16 N65` is a real pair: monitor, decimal, and the time register.
    ///
    /// The lamps are drawn unlit except `COMP ACTY`. The rest report conditions
    /// this program does not have, and a lit lamp that means nothing is a
    /// decoration pretending to be an instrument.
    fn render_dsky(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let green = Style::default().fg(theme.text);
        let bright = Style::default().fg(theme.primary).add_modifier(Modifier::BOLD);
        let dim = Style::default().fg(theme.label);
        let caution = Style::default().fg(theme.blocks[1 % theme.blocks.len()]);

        let cols = Layout2::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(14), Constraint::Min(0)])
            .split(area);

        // The annunciator column.
        let lamps = [
            ("COMP ACTY", true),
            ("UPLINK ACTY", false),
            ("NO ATT", false),
            ("STBY", false),
            ("KEY REL", false),
            ("OPR ERR", false),
            ("TRACKER", false),
        ];
        let mut column = vec![Line::styled("┌────────────┐", dim)];
        for (name, lit) in lamps {
            column.push(Line::from(vec![
                Span::styled("│", dim),
                Span::styled(
                    format!("{name:^12}"),
                    if lit { bright } else { dim },
                ),
                Span::styled("│", dim),
            ]));
        }
        column.push(Line::styled("└────────────┘", dim));
        f.render_widget(Paragraph::new(column), cols[0]);

        let rows = Layout2::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Length(digits::HEIGHT as u16 + 2),
                Constraint::Min(0),
            ])
            .split(cols[1]);

        // PROG, VERB, NOUN.
        f.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("  PROG  ", dim),
                    Span::styled("01", bright),
                    Span::styled("    UNIVERSE CALENDAR · UC1", dim),
                ]),
                Line::from(vec![
                    Span::styled("  VERB  ", dim),
                    Span::styled("16", bright),
                    Span::styled("    NOUN  ", dim),
                    Span::styled("65", bright),
                    Span::styled("   MONITOR DECIMAL · TIME", dim),
                ]),
                Line::styled("  ──────────────────────────────────────────", dim),
                Line::from(""),
            ]),
            rows[0],
        );

        // R1 and R2: the slow registers, in the instrument's own numerals.
        let reg = |k: i8| self.hands.iter().find(|h| h.tier.index() == k);
        let mut regs: Vec<Line> = Vec::new();
        for (name, k) in [("R1", 2i8), ("R2", 1i8)] {
            let h = reg(k);
            regs.push(Line::from(vec![
                Span::styled(format!("  {name}   "), dim),
                Span::styled(
                    format!("+{:05}", h.map_or(0, |h| h.position)),
                    green,
                ),
                Span::styled(
                    format!("   {}", h.map_or(String::new(), |h| h.label().to_uppercase())),
                    dim,
                ),
            ]));
        }
        regs.push(Line::from(""));
        f.render_widget(Paragraph::new(regs), rows[1]);

        // R3: the register that moves, in the block font. A real DSKY gives all
        // three the same size; see the theme's own note on the departure.
        let beat = self.beat().map_or(0, |h| h.position);
        let mut r3 = vec![Line::from(vec![
            Span::styled("  R3   ", dim),
            Span::styled("T0 BEAT · 21 PER SECOND", dim),
        ])];
        for row in digits::render(&format!("{beat:04}")) {
            r3.push(Line::styled(format!("  {row}"), bright));
        }
        f.render_widget(Paragraph::new(r3), rows[2]);

        let width = rows[3].width.saturating_sub(4) as usize;
        let filled = self.blur().map_or(0, |b| b.per_mille() as usize) * width / 1000;
        let bar: String = core::iter::repeat_n('▮', filled)
            .chain(core::iter::repeat_n('▯', width.saturating_sub(filled)))
            .collect();
        let mut tail = vec![
            Line::styled(format!("  {bar}"), caution),
            Line::styled("  T-1 FLICKER · TOO FAST FOR A REGISTER", dim),
            Line::from(""),
            Line::styled(format!("  {}", self.human), green),
        ];
        tail.extend(self.local_lines(theme));
        tail.push(Line::from(""));
        tail.push(Line::styled("  [Q] KEY REL", dim));
        f.render_widget(Paragraph::new(tail), rows[3]);
    }
    // ucal-lint-allow-end(no-wrapping-arithmetic)

    // ---- orbit -----------------------------------------------------------

    /// A row of dials with hands.
    ///
    /// One per tier, coarsest on the left, each drawn on its own braille canvas
    /// with the numeral beneath it. The numerals are not a hedge: a dial this
    /// size resolves about one stop in thirty, so it says which part of the tier
    /// you are in and the number says which stop — the same division of labour a
    /// clock with numerals on its face has always had.
    // ucal-lint-allow-begin(no-wrapping-arithmetic): u16 terminal geometry only
    fn render_orbit(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let ink = Style::default().fg(theme.text);
        let dim = Style::default().fg(theme.label);

        let rows = Layout2::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        f.render_widget(
            Paragraph::new(vec![
                Line::styled(" UCAL — universe calendar, on dials", ink),
                Line::styled(
                    " every tier has 3125 stops, because every rung is 5^5 of the one below",
                    dim,
                ),
            ]),
            rows[0],
        );

        // One dial per hand, side by side.
        let n = self.hands.len().max(1);
        let each = (rows[1].width as usize / n).clamp(1, 24);
        let cells = Layout2::default()
            .direction(Direction::Horizontal)
            .constraints(
                self.hands
                    .iter()
                    .map(|_| Constraint::Length(each as u16))
                    .chain(core::iter::once(Constraint::Min(0)))
                    .collect::<Vec<_>>(),
            )
            .split(rows[1]);

        for (i, h) in self.hands.iter().enumerate() {
            let Some(r) = cells.get(i) else { continue };
            if r.width < 4 || r.height < 4 {
                continue;
            }
            let colour = theme.blocks[i % theme.blocks.len()];
            let cols = (r.width as usize).saturating_sub(1);
            let dial_rows = (r.height as usize).saturating_sub(2);
            let mut canvas = dial::Canvas::new(cols, dial_rows);
            canvas.dial(h.position);
            let mut lines: Vec<Line> = canvas
                .lines()
                .into_iter()
                .map(|l| Line::styled(l, Style::default().fg(colour)))
                .collect();
            lines.push(Line::styled(
                format!("{:^width$}", h.position, width = cols),
                Style::default().fg(theme.primary),
            ));
            lines.push(Line::styled(
                format!("{:^width$}", h.label(), width = cols),
                dim,
            ));
            f.render_widget(Paragraph::new(lines), *r);
        }

        let width = rows[2].width.saturating_sub(2) as usize;
        let filled = self.blur().map_or(0, |b| b.per_mille() as usize) * width / 1000;
        let bar: String = core::iter::repeat_n('▁', filled)
            .chain(core::iter::repeat_n(' ', width.saturating_sub(filled)))
            .collect();
        f.render_widget(
            Paragraph::new(vec![
                Line::styled(format!(" {bar}"), Style::default().fg(theme.blur)),
                Line::styled(
                    " the finest hand has no dial: 66 000 stops a second is not a hand",
                    dim,
                ),
            ]),
            rows[2],
        );

        let mut tail = vec![Line::styled(format!(" {}", self.human), ink)];
        tail.extend(self.local_lines(theme));
        tail.push(Line::from(""));
        tail.push(Line::styled(" q to quit", dim));
        f.render_widget(Paragraph::new(tail), rows[3]);
    }
    // ucal-lint-allow-end(no-wrapping-arithmetic)

    // ---- shared ----------------------------------------------------------

    /// The big hand, in block digits.
    fn render_readout(&self, f: &mut Frame, area: Rect, theme: &Theme, align: Alignment) {
        let beat = self.beat().map_or(0, |h| h.position);
        let rows = digits::render(&format!("{beat:04}"));
        let lines: Vec<Line> = rows
            .iter()
            .map(|r| Line::styled(r.clone(), Style::default().fg(theme.primary)))
            .collect();
        f.render_widget(Paragraph::new(lines).alignment(align), area);
    }

    /// The tier below the readout, as a bar rather than as digits.
    ///
    /// It advances 66 000 times a second. Printing it as a number would be
    /// printing a number that is wrong by the time it is drawn, and printing it
    /// as *nothing* would hide a real quantity. A bar is what it is: a position
    /// on a dial, moving too fast to read and not too fast to see.
    fn render_blur(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let Some(blur) = self.blur() else { return };
        // Terminal geometry, not a time quantity. Rule O forbids saturating
        // arithmetic because a silently clamped *duration* is a wrong answer
        // where an error was available; a bar two cells narrower than a two-cell
        // pane is a bar of zero cells, which is the right answer and the only
        // one. Both saturations are on `u16` column counts.
        // ucal-lint-allow-begin(no-wrapping-arithmetic): column counts, not durations
        let width = area.width.saturating_sub(2) as usize;
        let filled = (blur.per_mille() as usize * width) / 1000;
        let bar: String = core::iter::repeat_n('▓', filled)
            .chain(core::iter::repeat_n('░', width.saturating_sub(filled)))
            .collect();
        // ucal-lint-allow-end(no-wrapping-arithmetic)
        f.render_widget(
            Paragraph::new(vec![
                Line::styled(bar, Style::default().fg(theme.blur)),
                Line::styled(
                    "T-1 FLICKER · 66 000 PER SECOND · A POSITION, NOT A NUMBER",
                    Style::default().fg(theme.label),
                ),
            ]),
            area,
        );
    }

    /// The second dial, where one was asked for.
    ///
    /// A block of four lines rather than a panel of its own: it is a *second*
    /// face, and giving it equal weight would make the clock a comparison table.
    /// The bar is how far through the local day the instant falls, which is the
    /// only quantity here that moves at a rate worth drawing — one local day is
    /// a day of that body and not of this one.
    fn local_lines(&self, theme: &Theme) -> Vec<Line<'static>> {
        let Some(l) = &self.local else {
            return Vec::new();
        };
        let width = 40usize;
        let filled = l.through_day as usize * width / 100;
        let bar: String = core::iter::repeat_n('▓', filled)
            .chain(core::iter::repeat_n('░', width - filled))
            .collect();
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!("{:<14}", l.calendar.to_uppercase()),
                    Style::default().fg(theme.label),
                ),
                Span::styled(
                    format!("year {}  day {}", l.year, l.day),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
                // No ordinal: "1th local year" was the first draft, and a
                // suffix table for one label is a rule to get wrong later. The
                // convention stated plainly needs no agreement about English.
                Span::styled(
                    "   counted from the anchor — year 1 began there",
                    Style::default().fg(theme.label),
                ),
            ]),
            Line::from(vec![
                Span::styled(format!("{bar} "), Style::default().fg(theme.blocks[1 % theme.blocks.len()])),
                Span::styled(
                    format!("{}% through the local day", l.through_day),
                    Style::default().fg(theme.label),
                ),
            ]),
            Line::styled(
                format!(
                    "anchor revision {} — an anchor is an observation and is versioned (Rule J)",
                    l.revision
                ),
                Style::default().fg(theme.label),
            ),
        ]
    }

    fn hand_lines(&self, theme: &Theme) -> Vec<Line<'static>> {
        self.hands
            .iter()
            .map(|h| {
                Line::from(vec![
                    Span::styled(
                        format!("{:<14}", h.label()),
                        Style::default().fg(theme.label),
                    ),
                    Span::styled(format!("{:>4}", h.position), Style::default().fg(theme.text)),
                ])
            })
            .collect()
    }
}
