//! What the clock shows, and how it is drawn.
//!
//! [`Face`] is the reading — a set of hand positions and nothing about colour or
//! layout — so it can be built and asserted on without a terminal. Rendering
//! takes a [`Theme`] and puts it on a frame.

use super::digits;
use super::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use ucal_core::backend::TickInt;
use ucal_core::{Instant, Ticks, Tier, TimeError, UC1};

/// One tier's hand: which of the 3125 subdivisions of the tier above it we are
/// currently in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Hand {
    /// The tier this hand belongs to.
    pub tier: Tier,
    /// Its name, where it has one.
    pub name: &'static str,
    /// `0..3125`.
    pub position: u32,
}

impl Hand {
    /// How far round its dial, in thousandths.
    ///
    /// Integer, because a clock face is not a place to introduce a float into a
    /// program that has spent nine releases keeping them out (Rule E).
    pub fn per_mille(self) -> u32 {
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
}

impl Face {
    /// Read the system clock.
    pub fn read_now() -> Result<Face, TimeError> {
        Face::at(super::now_instant()?)
    }

    /// Read a given instant, which is what the tests use.
    pub fn at(t: Instant<UC1>) -> Result<Face, TimeError> {
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
                name: ucal_core::tier::name_of(tier).map_or("", |n| n.key()),
                position,
            });
        }
        Ok(Face {
            human: crate::render_at(&t, Tier::new(0)?),
            at: t,
            hands,
        })
    }

    /// The hand a reader watches: `T0`, the beat, at about 21 per second.
    pub fn beat(&self) -> Option<Hand> {
        self.hands.iter().copied().find(|h| h.tier.index() == 0)
    }

    /// The hand below it, which is a blur and is drawn as one.
    pub fn blur(&self) -> Option<Hand> {
        self.hands.iter().copied().find(|h| h.tier.index() == -1)
    }

    /// Draw the face.
    pub fn render(&self, f: &mut Frame, theme: &Theme) {
        let area = f.area();
        f.render_widget(
            Block::default().style(Style::default().bg(theme.background)),
            area,
        );
        if theme.lcars {
            self.render_lcars(f, area, theme);
        } else {
            self.render_plain(f, area, theme);
        }
    }

    // ---- plain -----------------------------------------------------------

    fn render_plain(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let rows = Layout::default()
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
        f.render_widget(Paragraph::new(self.hand_lines(theme)), rows[2]);
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
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(18), Constraint::Min(0)])
            .split(area);
        let rail = cols[0];
        let main = cols[1];

        // The elbow: a solid block at the top of the rail, joined to the header.
        let rail_rows = Layout::default()
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
        let block_rows = Layout::default()
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
            let label = if h.name.is_empty() {
                h.tier.to_string()
            } else {
                format!("{} {}", h.tier, h.name)
            };
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
        let main_rows = Layout::default()
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
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::styled(self.human.clone(), Style::default().fg(theme.text)),
            ]),
            main_rows[4],
        );
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

    fn hand_lines(&self, theme: &Theme) -> Vec<Line<'static>> {
        self.hands
            .iter()
            .map(|h| {
                let label = if h.name.is_empty() {
                    h.tier.to_string()
                } else {
                    format!("{} {}", h.tier, h.name)
                };
                Line::from(vec![
                    Span::styled(format!("{label:<12}"), Style::default().fg(theme.label)),
                    Span::styled(format!("{:>4}", h.position), Style::default().fg(theme.text)),
                ])
            })
            .collect()
    }
}
