//! C4 — what actually stops `z_of_t` from reaching a fine tolerance.
//!
//! ```text
//! cargo run --release -p ucal-cosmo --example c4_bisection_ceiling
//! ```
//!
//! `z_of_t` bisects `[0, 10000]` for at most 64 halvings and returns
//! `UCAL-E0071` if the time bracket has not fallen below the requested
//! tolerance by then. The release notes recorded that budget as "a ceiling
//! nobody has tried to raise". This raises it and measures what happens.
//!
//! The loop below is the library's, with the budget as a parameter, driving the
//! public `t_of_z`. Reimplementing ten lines is what lets the experiment vary
//! the one number under test without adding an API surface that the result might
//! say should not exist.
//!
//! Three questions, in order:
//!
//! 1. How many halvings does a given tolerance actually need?
//! 2. Does the loop keep converging as the budget rises, or stall?
//! 3. What does it cost — and is the cost the budget, or something else?

use std::time::Instant as Clock;

use ucal_core::num::Ratio;
use ucal_core::profile::UC1;
use ucal_core::value::{Delta, Instant};
use ucal_core::{TickInt, Ticks};
use ucal_cosmo::{LambdaCdm, DEFAULT_DEPTH, DEFAULT_SCALE};

/// Durations in ticks. `SECOND = 18 548 584 399 861 x 10^30` (D-3), and the
/// year is the Julian year the rest of the tree uses, 31 557 600 s exactly.
/// Written out rather than imported: `ucal-cosmo` does not depend on
/// `ucal-civil`, and an experiment is not a reason to change the graph.
const ONE_SECOND: &str = "18548584399861000000000000000000000000000000";
const ONE_HOUR: &str = "66774903839499600000000000000000000000000000000";
const ONE_DAY: &str = "1602597692147990400000000000000000000000000000000";
const ONE_MILLISECOND: &str = "18548584399861000000000000000000000000000";
const ONE_YEAR: &str = "585348807057053493600000000000000000000000000000000";

fn delta(dec: &str) -> Delta {
    Delta::from_ticks(<Ticks as TickInt>::from_dec_str(dec).expect("decimal literal"))
}

/// One side of the library's bisection, with the step budget exposed.
///
/// Returns the bracket end, the steps used, and the final time bracket width —
/// the last of which is the number the convergence test is actually looking at.
fn bisect(
    m: &LambdaCdm,
    target: &Instant<UC1>,
    tolerance: &Delta,
    depth: u32,
    scale: u32,
    use_hi: bool,
    budget: u32,
) -> Outcome {
    let two = Ratio::from_u64(2);
    let mut lo = Ratio::zero();
    let mut hi = Ratio::from_u64(10_000);

    let age = |z: &Ratio| -> Result<Instant<UC1>, ucal_core::TimeError> {
        let w = m.t_of_z(z, depth, scale)?.value;
        Ok(if use_hi { w.hi().clone() } else { w.lo().clone() })
    };
    let mut t_lo = age(&lo).expect("age at 0");
    let mut t_hi = age(&hi).expect("age at ceiling");
    let mut last = t_lo.since(&t_hi).expect("since");

    for step in 0..budget {
        last = t_lo.since(&t_hi).expect("since");
        if last <= *tolerance {
            return Outcome::Converged {
                z: if use_hi { hi } else { lo },
                steps: step,
                width: last,
            };
        }
        let mid = lo.add(&hi).expect("add").div(&two).expect("div");
        let t_mid = match age(&mid) {
            Ok(t) => t,
            Err(e) => {
                return Outcome::Failed {
                    steps: step,
                    width: last,
                    denom_digits: den_digits(&mid),
                    code: format!("{:?}", e.code),
                }
            }
        };
        if &t_mid >= target {
            lo = mid;
            t_lo = t_mid;
        } else {
            hi = mid;
            t_hi = t_mid;
        }
    }
    Outcome::Exhausted {
        steps: budget,
        width: last,
    }
}

/// How a bisection ended. The distinction is the experiment's whole point: a
/// budget that runs out is a different fact from arithmetic that cannot
/// represent the next midpoint.
#[allow(dead_code)] //  is read through the match in main, not a method
#[allow(dead_code)] // read through the match in main rather than a method
enum Outcome {
    Converged {
        z: Ratio,
        steps: u32,
        width: Delta,
    },
    /// The step budget ran out with the bracket still too wide. `UCAL-E0071`.
    Exhausted { steps: u32, width: Delta },
    /// The next midpoint could not be evaluated at all.
    Failed {
        steps: u32,
        width: Delta,
        denom_digits: usize,
        code: String,
    },
}

impl Outcome {
    fn label(&self) -> &'static str {
        match self {
            Outcome::Converged { .. } => "converged",
            Outcome::Exhausted { .. } => "budget out",
            Outcome::Failed { .. } => "DOMAIN",
        }
    }
    fn steps(&self) -> u32 {
        match self {
            Outcome::Converged { steps, .. }
            | Outcome::Exhausted { steps, .. }
            | Outcome::Failed { steps, .. } => *steps,
        }
    }
}

/// Decimal digits in a rational's denominator — the thing that grows.
fn den_digits(r: &Ratio) -> usize {
    r.denom().to_dec_string().len()
}

fn main() {
    let m = LambdaCdm::planck2018();
    let depth = DEFAULT_DEPTH;
    let scale = DEFAULT_SCALE;

    // A target the inversion can actually find: the age at z = 1 as this model
    // computes it.
    let z1 = Ratio::from_u64(1);
    let target = m
        .t_of_z(&z1, depth, scale)
        .expect("t_of_z")
        .value
        .lo()
        .clone();

    println!("C4 — raising the z_of_t bisection ceiling");
    println!("depth {depth}, scale {scale}, bracket [0, 10000], target = age at z = 1");
    println!("the shipped budget is 64 halvings\n");

    println!("tolerance    budget   steps  outcome      wall     z denominator digits");
    println!("──────────   ──────   ─────  ──────────   ──────   ────────────────────");
    for (label, dec) in [
        ("1 year", ONE_YEAR),
        ("1 day", ONE_DAY),
        ("1 hour", ONE_HOUR),
        ("1 second", ONE_SECOND),
        ("1 ms", ONE_MILLISECOND),
        ("1 tick", "1"),
    ] {
        let tol = delta(dec);
        let t0 = Clock::now();
        let out = bisect(&m, &target, &tol, depth, scale, true, 512);
        let el = t0.elapsed();
        let digits = match &out {
            Outcome::Converged { z, .. } => den_digits(z),
            Outcome::Failed { denom_digits, .. } => *denom_digits,
            Outcome::Exhausted { .. } => 0,
        };
        println!(
            "{label:<10}   {:>6}   {:>5}  {:<10}   {:>4} ms   {digits:>20}",
            512,
            out.steps(),
            out.label(),
            el.as_millis()
        );
        if let Outcome::Failed { code, width, .. } = &out {
            println!(
                "             stopped by {code}; bracket still {} ticks",
                width.ticks().to_dec_string()
            );
            break;
        }
    }

    // --- does depth move the wall? -----------------------------------------
    //
    // The question C3 turns on. If a finer quadrature reached further, the
    // second-order form would be buying something. If the wall is in the same
    // place at every depth, it is a property of the 512-bit domain and no
    // quadrature changes it.
    println!("\nthe one-tick request, at each depth:");
    println!("depth   steps  outcome   denominator digits at the wall");
    println!("─────   ─────  ───────   ──────────────────────────────");
    for d in [4u32, 8, 12] {
        let target_d = m
            .t_of_z(&z1, d, scale)
            .expect("t_of_z")
            .value
            .lo()
            .clone();
        let out = bisect(&m, &target_d, &delta("1"), d, scale, true, 512);
        let digits = match &out {
            Outcome::Failed { denom_digits, .. } => *denom_digits,
            Outcome::Converged { z, .. } => den_digits(z),
            Outcome::Exhausted { .. } => 0,
        };
        println!(
            "{d:>5}   {:>5}  {:<7}   {digits:>30}",
            out.steps(),
            out.label()
        );
    }
}
