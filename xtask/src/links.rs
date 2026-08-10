//! `check-links` — do the cited URLs still reach the cited documents?
//!
//! # Why this is opt-in and not in CI
//!
//! Every other check in this tree is deterministic and offline: the same tree
//! gives the same answer on any machine, forever. This one asks the internet,
//! and the internet answers differently depending on who is asking, from where,
//! and whether somebody else's server is having a bad morning.
//!
//! Putting it in CI would mean a third party's outage turns this repository red
//! — and "CI green on every push, with no known-failing job" is one of the 1.0
//! exit criteria. A check that cries wolf trains its reader to ignore it, which
//! is worse than not having it: `Documentation/Proposals/ROAD-TO-1.0.md`
//! already records one criterion that was asserted for a whole release while
//! being false, and the way that happened was nobody reading a red job.
//!
//! So it is run by a person, at release time, and the release procedure says so.
//!
//! # Why a 200 is not enough
//!
//! This check exists because two citations rotted silently, and **one of them
//! answered `200 OK`**. `nssdc.gsfc.nasa.gov/planetary/factsheet/` 307s to a
//! general NASA page, which serves a perfectly good document that is not the
//! one being cited. A naive status check would have passed it and the citation
//! would still be broken.
//!
//! So a redirect that leaves the host is reported. It is not always wrong —
//! `stratigraphy.org/chart` to `stratigraphy.org/chart/` stays put and is fine,
//! and an archive link resolves within `web.archive.org` — but a citation whose
//! locator now lands on a different site is a citation that needs a human to
//! look at it.
//!
//! # What it cannot check
//!
//! Whether the page still says what was cited. A URL that resolves, on the
//! right host, serving a rewritten document, passes this and is exactly the
//! failure a citation is meant to prevent. Nothing mechanical reaches that, and
//! saying so here is cheaper than letting a green run imply otherwise.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// This repository's own `blob/main` prefix.
///
/// A link to a file this cycle added will 404 until the branch merges, and the
/// release procedure runs this check *before* the merge. That is a false alarm
/// with a completely reliable signature: the URL points at this repository's
/// `main`, and the file it names is sitting in the working tree.
const OWN_MAIN: &str = "https://github.com/vulogov/ucal/blob/main/";

/// What happened to one URL.
enum Outcome {
    /// Reached, on the host it named.
    Ok(String),
    /// Reached, but the final URL is on a different host.
    Moved { status: String, to: String },
    /// Not reached.
    Failed(String),
    /// `curl` itself could not run.
    NoTool(String),
    /// A link to this repository's `main` naming a file that exists locally:
    /// unmerged, not dead.
    Unmerged(String),
}

/// Check every cited URL in the tree. Returns a process exit code.
pub fn run(root: &Path) -> i32 {
    if Command::new("curl").arg("--version").output().is_err() {
        eprintln!("  FAIL  `curl` is not on PATH, and check-links shells out to it");
        eprintln!("        rather than adding an HTTP client to this workspace.");
        return 7;
    }

    let urls = collect(root);
    if urls.is_empty() {
        println!("  ok    no cited URLs found");
        return 0;
    }
    println!("checking {} cited URL(s) — this asks the network\n", urls.len());

    let mut failed = 0usize;
    let mut moved = 0usize;
    for (url, sites) in &urls {
        match check_one(root, url) {
            Outcome::Ok(status) => println!("  ok    {status}  {url}"),
            Outcome::Moved { status, to } => {
                moved += 1;
                println!("  MOVED {status}  {url}");
                println!("          now lands on a different host: {to}");
                for s in sites {
                    println!("          cited at {s}");
                }
            }
            Outcome::Failed(status) => {
                failed += 1;
                println!("  FAIL  {status}  {url}");
                for s in sites {
                    println!("          cited at {s}");
                }
            }
            Outcome::NoTool(e) => {
                failed += 1;
                println!("  FAIL  curl: {e}  {url}");
            }
            Outcome::Unmerged(rel) => {
                println!("  --    {url}");
                println!("          not on `main` yet; {rel} is in the working tree, so this");
                println!("          resolves when the branch merges. Not counted as a failure.");
            }
        }
    }

    println!();
    if failed == 0 && moved == 0 {
        println!("  {} cited URL(s) resolve on the host they name.", urls.len());
        println!("  This does not check that the page still says what was cited.");
        return 0;
    }
    if failed > 0 {
        println!("  {failed} unreachable.");
    }
    if moved > 0 {
        println!("  {moved} reachable but redirected off-host — a citation that now");
        println!("  lands somewhere else needs a person to look at it, even at 200.");
    }
    println!("\n  A locator that has rotted should point at an archived copy of the");
    println!("  document that was actually read, with the source string saying so.");
    2
}

/// Check one URL, allowing for links to unmerged files in this repository.
fn check_one(root: &Path, url: &str) -> Outcome {
    let outcome = head(url);
    // Only a 404 on our own `main` can be an unmerged link, and only if the file
    // is actually here. Anything else is what it says it is.
    if let Outcome::Failed(status) = &outcome {
        if status == "404" {
            if let Some(rel) = url.strip_prefix(OWN_MAIN) {
                if root.join(rel).exists() {
                    return Outcome::Unmerged(rel.to_string());
                }
            }
        }
    }
    outcome
}

/// `curl -sIL`, reduced to a final status and a final URL.
fn head(url: &str) -> Outcome {
    let out = Command::new("curl")
        .args([
            "-sIL",
            "--max-time",
            "25",
            "-A",
            // A default curl UA is refused by some publishers, which would make
            // this report a live document dead. Both rotted locators were
            // re-checked this way before being called dead.
            "Mozilla/5.0 (compatible; ucal-check-links/1; +https://github.com/vulogov/ucal)",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code} %{url_effective}",
            url,
        ])
        .output();
    let out = match out {
        Ok(o) => o,
        Err(e) => return Outcome::NoTool(e.to_string()),
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut parts = text.trim().splitn(2, ' ');
    let status = parts.next().unwrap_or("000").to_string();
    let effective = parts.next().unwrap_or("").to_string();

    let reached = status.starts_with('2');
    if !reached {
        return Outcome::Failed(status);
    }
    match (host_of(url), host_of(&effective)) {
        (Some(a), Some(b)) if a != b => Outcome::Moved {
            status,
            to: effective,
        },
        _ => Outcome::Ok(status),
    }
}

/// The host part of a URL, lowercased, without a leading `www.`.
///
/// `www.` is dropped because a site adding or removing it is a redirect nobody
/// needs to be told about, and this check is only worth running if its output is
/// worth reading.
fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://")?.1;
    let host = rest.split(['/', '?', '#']).next()?;
    let host = host.trim_start_matches("www.").to_ascii_lowercase();
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

/// Every cited URL in the tree, with where each is cited.
///
/// Source files and the specification, which is where citations live. Release
/// notes and proposals link to plenty of things, but a broken link in a release
/// note is a nuisance and a broken *citation* is a claim that cannot be checked
/// — the second is what this is for.
fn collect(root: &Path) -> Vec<(String, Vec<String>)> {
    let mut found: std::collections::BTreeMap<String, BTreeSet<String>> = Default::default();
    let mut walk = |dir: &Path| {
        let mut stack = alloc_stack(dir);
        while let Some(p) = stack.pop() {
            if p.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&p) {
                    for e in entries.flatten() {
                        let name = e.file_name();
                        let name = name.to_string_lossy();
                        if name.starts_with('.') || name == "target" {
                            continue;
                        }
                        stack.push(e.path());
                    }
                }
                continue;
            }
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
            if ext != "rs" && ext != "md" {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&p) else {
                continue;
            };
            let rel = p
                .strip_prefix(root)
                .unwrap_or(&p)
                .to_string_lossy()
                .into_owned();
            for (n, line) in text.lines().enumerate() {
                for url in urls_in(line) {
                    found
                        .entry(url)
                        .or_default()
                        .insert(format!("{rel}:{}", n + 1));
                }
            }
        }
    };
    walk(&root.join("crates"));
    walk(&root.join("spec"));
    found
        .into_iter()
        .map(|(u, s)| (u, s.into_iter().collect()))
        .collect()
}

fn alloc_stack(dir: &Path) -> Vec<std::path::PathBuf> {
    vec![dir.to_path_buf()]
}

/// URLs on one line, trimmed of the punctuation that follows them in prose.
fn urls_in(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find("http") {
        let tail = &rest[at..];
        if !(tail.starts_with("http://") || tail.starts_with("https://")) {
            rest = &tail[4..];
            continue;
        }
        let end = tail
            .find(|c: char| c.is_whitespace() || c == '"' || c == ')' || c == '>' || c == '`')
            .unwrap_or(tail.len());
        let url = tail[..end].trim_end_matches(['.', ',', ';', ':', ']']);
        if url.len() > 10 {
            out.push(url.to_string());
        }
        rest = &tail[end..];
    }
    out
}
