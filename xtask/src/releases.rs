//! R1 — a version marked *released* must exist where a reader can get it.
//!
//! # The defect this exists for
//!
//! `Documentation/Release_Notes/README.md` opens with a Contents table, one row
//! per version, each carrying a state. **1.10.0's row said `released`, and
//! 1.10.0 exists nowhere**: no tag in any remote, no GitHub release, no crate on
//! the registry, which goes 1.9.0 → 1.11.0. The release procedure has thirteen
//! steps and the first seven ran; the row was written on the strength of the
//! cut.
//!
//! It was found by reading which baseline `cargo semver-checks` had chosen — it
//! fetches the newest *published* version and fetched 1.9.0, while the cycle's
//! notes said twice that the comparison was against 1.10.0. Nothing in this tree
//! asks the world whether a release happened, because everything in this tree
//! checks files that are in it.
//!
//! # Why it is not on `push`
//!
//! Three network services decide the answer, and a check that reddens the tree
//! because `crates.io` is having a bad morning trains its reader to ignore it —
//! which is the *same* defect this check exists for, one level up. So it runs on
//! a schedule beside [`links`](crate::links), and a failure opens an issue.
//!
//! # What it deliberately does not check
//!
//! **That the artefact is any good.** A tag can point anywhere, a release can
//! carry the wrong binaries, and a crate can be a different tree entirely.
//! `verify-release` is the check for that and it needs a version at a time. This
//! one asks the much smaller question the table was getting wrong: *is it
//! there at all.*

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// Where GitHub releases begin.
///
/// Measured rather than assumed: of the 0.x line only `v0.6.0` has one, and
/// every version from 1.0.0 does. Binaries-as-a-release started partway through
/// 0.x, so requiring one of `v0.1.0` would report eight failures that are
/// history rather than defects — and a check whose output is mostly known-bad
/// entries is one nobody reads.
///
/// Stored as a floor rather than a list of exceptions so that it cannot quietly
/// grow: a new version below the floor is impossible, and a missing release
/// above it fails.
const GH_RELEASES_FROM: (u64, u64, u64) = (1, 0, 0);

/// A row of the Contents table.
struct Row {
    version: String,
    /// The state cell, verbatim.
    state: String,
}

/// The states a row may declare.
///
/// A closed vocabulary, because the check reads the state to decide what to
/// demand — and a state it did not recognise would otherwise be a row it
/// silently skipped. `cut, never published` is 1.10.0's, and it is *enumerated*
/// rather than pattern-matched so that it cannot become the escape hatch every
/// awkward release takes.
const STATES: &[(&str, Demand)] = &[
    ("released", Demand::Everywhere),
    ("unreleased", Demand::Nothing),
    ("cut, never published", Demand::Nothing),
];

#[derive(Clone, Copy, PartialEq)]
enum Demand {
    /// Tag, crates, and — above the floor — a GitHub release.
    Everywhere,
    Nothing,
}

/// Parse the Contents table.
fn rows(root: &Path) -> Result<Vec<Row>, String> {
    let path = root.join("Documentation/Release_Notes/README.md");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if !l.starts_with("| [") {
            continue;
        }
        let cells: Vec<&str> = l.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() < 3 {
            continue;
        }
        let version = cells[0]
            .trim_start_matches("[")
            .split(']')
            .next()
            .unwrap_or_default()
            .to_string();
        if version.is_empty() {
            continue;
        }
        out.push(Row {
            version,
            state: cells[2].to_string(),
        });
    }
    if out.is_empty() {
        return Err("the Contents table has no version rows, so this check would \
                    pass having examined nothing"
            .into());
    }
    Ok(out)
}

/// What a row's state demands, or an error naming the states that exist.
fn demand(state: &str) -> Result<Demand, String> {
    // The state cell carries prose after the state — `released — the artefact,
    // not the repository` — so this matches the leading word(s) rather than the
    // whole cell. Longest first, so `cut, never published` is not read as an
    // unknown state beginning with `cut`.
    let mut known: Vec<&(&str, Demand)> = STATES.iter().collect();
    known.sort_by_key(|(s, _)| std::cmp::Reverse(s.len()));
    let plain = state.replace("**", "");
    for (s, d) in known {
        if plain.starts_with(s) {
            return Ok(*d);
        }
    }
    Err(format!(
        "unknown state `{state}`. The states are: {}",
        STATES
            .iter()
            .map(|(s, _)| format!("`{s}`"))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let mut it = v.split('.');
    Some((
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    ))
}

/// Every tag on the remote, without the `refs/tags/` and without `^{}`.
fn remote_tags(root: &Path) -> Result<BTreeSet<String>, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["ls-remote", "--tags", "origin"])
        .output()
        .map_err(|e| format!("git ls-remote could not run: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git ls-remote failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let tags: BTreeSet<String> = text
        .lines()
        .filter_map(|l| l.split("refs/tags/").nth(1))
        .filter(|t| !t.ends_with("^{}"))
        .map(str::to_string)
        .collect();
    if tags.is_empty() {
        return Err("the remote reports no tags at all, which is not an answer \
                    about any particular version"
            .into());
    }
    Ok(tags)
}

/// Every GitHub release's tag.
fn gh_releases(root: &Path) -> Result<BTreeSet<String>, String> {
    let out = Command::new("gh")
        .current_dir(root)
        .args(["release", "list", "--limit", "200", "--json", "tagName", "-q", ".[].tagName"])
        .output()
        .map_err(|e| format!("gh could not run: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "gh release list failed — if this says authentication, run `gh auth \
             login`, because an unauthenticated `gh` reports no releases and \
             that is indistinguishable from a repository with none: {}",
            err.trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

/// Every version of a crate on the registry, from the sparse index.
fn registry_versions(krate: &str) -> Result<BTreeSet<String>, String> {
    // The sparse index's path layout: 1/2/3-character prefixes, then `xx/yy` for
    // longer names. Every crate here is longer than three characters.
    let (a, b) = (&krate[..2], &krate[2..4]);
    let url = format!("https://index.crates.io/{a}/{b}/{krate}");
    let out = Command::new("curl")
        .args(["-sSL", "--max-time", "60", "--fail", &url])
        .output()
        .map_err(|e| format!("curl could not run: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{url} did not answer: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let body = String::from_utf8_lossy(&out.stdout);
    let mut vers = BTreeSet::new();
    for line in body.lines().filter(|l| !l.trim().is_empty()) {
        // One JSON object per line. Reading the `vers` field without a JSON
        // parser, the same way `citations` reads what it needs: the field is
        // written by cargo and its shape is part of the index format.
        if let Some(rest) = line.split("\"vers\":\"").nth(1) {
            if let Some(v) = rest.split('"').next() {
                vers.insert(v.to_string());
            }
        }
    }
    if vers.is_empty() {
        return Err(format!("{url} answered with no versions"));
    }
    Ok(vers)
}

/// The crates this workspace publishes.
fn published_crates(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root.join("crates")) else {
        return out;
    };
    for e in entries.flatten() {
        let manifest = e.path().join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        // `publish = false` is how `xtask` stays off the registry; a crate that
        // says so is not expected to be there.
        if text.contains("publish = false") {
            continue;
        }
        if let Some(name) = e.file_name().to_str() {
            out.push(name.to_string());
        }
    }
    out.sort();
    out
}

/// Run the check. Exit code, printed as it goes.
pub fn run(root: &Path) -> i32 {
    println!("check-releases — every version marked released, in the world\n");

    let rows = match rows(root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  FAIL  {e}");
            return 6;
        }
    };
    let tags = match remote_tags(root) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("  FAIL  {e}");
            return 6;
        }
    };
    let releases = match gh_releases(root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  FAIL  {e}");
            return 6;
        }
    };

    let crates = published_crates(root);
    if crates.is_empty() {
        eprintln!("  FAIL  no publishable crate found under crates/, so this check \
                   would examine nothing");
        return 6;
    }
    let mut registry = Vec::new();
    for c in &crates {
        match registry_versions(c) {
            Ok(v) => registry.push((c.clone(), v)),
            Err(e) => {
                eprintln!("  FAIL  {e}");
                return 6;
            }
        }
    }

    let mut bad: Vec<String> = Vec::new();
    let mut checked = 0usize;
    let mut exempt = 0usize;

    for row in &rows {
        let d = match demand(&row.state) {
            Ok(d) => d,
            Err(e) => {
                bad.push(format!("{}: {e}", row.version));
                continue;
            }
        };
        if d == Demand::Nothing {
            exempt += 1;
            println!("  --    {:<8} {}", row.version, row.state.replace("**", ""));
            continue;
        }
        checked += 1;
        let tag = format!("v{}", row.version);
        let mut missing: Vec<String> = Vec::new();

        if !tags.contains(&tag) {
            missing.push(format!("no `{tag}` on the remote"));
        }
        let above_floor = parse_version(&row.version).is_some_and(|v| v >= GH_RELEASES_FROM);
        if above_floor && !releases.contains(&tag) {
            missing.push(format!("no GitHub release for `{tag}`"));
        }
        for (c, vers) in &registry {
            if !vers.contains(&row.version) {
                missing.push(format!("`{c}` {} is not on the registry", row.version));
            }
        }

        if missing.is_empty() {
            println!("  ok    {:<8} tag, release, and {} crates", row.version, crates.len());
        } else {
            println!("  FAIL  {:<8} {}", row.version, missing.join("; "));
            bad.push(format!("{} — {}", row.version, missing.join("; ")));
        }
    }

    println!();
    if bad.is_empty() {
        println!(
            "  {checked} released version(s) exist where a reader can get them; \
             {exempt} row(s) claim nothing."
        );
        println!("  This does not check that the artefacts are the right ones.");
        println!("  `verify-release <version>` is that check.");
        0
    } else {
        eprintln!("  {} version(s) marked released are not where the table says:", bad.len());
        for b in &bad {
            eprintln!("    {b}");
        }
        eprintln!(
            "\n  Either the release did not finish, or the row is wrong. Both are\n  \
             worth knowing; neither is fixed by editing this check."
        );
        7
    }
}
