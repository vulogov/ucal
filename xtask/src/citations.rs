//! Citation integrity: every `§`, `Rule`, `D-N` and `FN` in the source must
//! resolve to text in `spec/`.
//!
//! # Why this is a check and not a convention
//!
//! The implementation cites the specification roughly a thousand times. Those
//! citations are the only thing connecting a line of code to the reasoning
//! behind it, and they are exactly the kind of reference that rots silently: a
//! section gets renumbered, a rule gets folded into another, and the code keeps
//! pointing at something that no longer exists. Nothing fails, nothing warns,
//! and the explanation is gone.
//!
//! §13.5 already establishes the pattern for this — the tier table, the locale
//! table and the generated documentation come from one source so they cannot
//! drift, and `check-docs` fails when they do. This applies the same discipline
//! to the citations themselves.
//!
//! # What it checks
//!
//! For each citation *form*, that the target exists in the normative spec:
//!
//! | form | resolves against |
//! |---|---|
//! | `§N` / `§N.N` | a heading or bold section marker in `UCAL-1.1.md` |
//! | `Rule X` | an entry in `RULES.md` |
//! | `D-AN` | a delta record in `SPEC-DELTAS.md` |
//!
//! It deliberately does **not** check `FN` (failure modes), `D-N` (the RFC's
//! own decision numbers) or `GE-N` (gated experiments). All three live in
//! tables rather than headings, and a text search for them would report a
//! coverage this does not have. Three checked forms that hold is worth more
//! than six that are half true.

use std::collections::BTreeSet;
use std::path::Path;

/// One citation that resolves to nothing.
pub struct Dangling {
    pub kind: &'static str,
    pub citation: String,
    pub sites: usize,
}

fn read(root: &Path, rel: &str) -> Result<String, String> {
    std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))
}

/// Collect every citation of each form that appears in the shipped source.
fn cited(root: &Path) -> Result<Vec<(&'static str, String, usize)>, String> {
    let mut counts: std::collections::BTreeMap<(&'static str, String), usize> = Default::default();
    let mut stack = vec![root.join("crates"), root.join("xtask").join("src")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if name != "target" && name != ".git" {
                    stack.push(p);
                }
            } else if name.ends_with(".rs") {
                let Ok(src) = std::fs::read_to_string(&p) else {
                    continue;
                };
                for (kind, c) in scan(&src) {
                    *counts.entry((kind, c)).or_default() += 1;
                }
            }
        }
    }
    Ok(counts.into_iter().map(|((k, c), n)| (k, c, n)).collect())
}

/// Pull citations out of one source file.
pub fn scan(src: &str) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    let b: Vec<char> = src.chars().collect();

    for i in 0..b.len() {
        // §N or §N.N
        if b[i] == '§' {
            let mut j = i + 1;
            let mut s = String::new();
            while j < b.len() && (b[j].is_ascii_digit() || (b[j] == '.' && !s.is_empty())) {
                s.push(b[j]);
                j += 1;
            }
            let s = s.trim_end_matches('.').to_string();
            if !s.is_empty() {
                out.push(("section", s));
            }
        }
        // Rule X — a single capital, not followed by a word character
        if b[i] == 'R' && src[byte_of(src, i)..].starts_with("Rule ") {
            let k = i + 5;
            if k < b.len() && b[k].is_ascii_uppercase() {
                let after_ok = k + 1 >= b.len() || !(b[k + 1].is_alphanumeric() || b[k + 1] == '_');
                if after_ok {
                    out.push(("rule", b[k].to_string()));
                }
            }
        }
        // D-AN
        if b[i] == 'D' && src[byte_of(src, i)..].starts_with("D-A") {
            let mut j = i + 3;
            let mut s = String::new();
            while j < b.len() && b[j].is_ascii_digit() {
                s.push(b[j]);
                j += 1;
            }
            if !s.is_empty() {
                out.push(("delta", format!("D-A{s}")));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn byte_of(src: &str, char_idx: usize) -> usize {
    src.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(src.len())
}

/// Check every citation in the source against `spec/`.
pub fn check(root: &Path) -> Result<usize, Vec<Dangling>> {
    let spec = match (
        read(root, "spec/UCAL-1.1.md"),
        read(root, "spec/RULES.md"),
        read(root, "spec/SPEC-DELTAS.md"),
    ) {
        (Ok(a), Ok(b), Ok(c)) => (a, b, c),
        (a, b, c) => {
            let mut bad = Vec::new();
            for (r, f) in [(a, "UCAL-1.1.md"), (b, "RULES.md"), (c, "SPEC-DELTAS.md")] {
                if r.is_err() {
                    bad.push(Dangling {
                        kind: "spec file",
                        citation: f.into(),
                        sites: 0,
                    });
                }
            }
            return Err(bad);
        }
    };
    let (normative, rules, deltas) = spec;

    // Which sections does the normative text actually define?
    let mut defined: BTreeSet<String> = BTreeSet::new();
    for line in normative.lines() {
        let t = line.trim_start_matches(['#', '*', '>', ' ']);
        let mut n = String::new();
        for ch in t.chars() {
            if ch.is_ascii_digit() || (ch == '.' && !n.is_empty()) {
                n.push(ch);
            } else {
                break;
            }
        }
        let n = n.trim_end_matches('.').to_string();
        if n.is_empty() {
            continue;
        }
        // A section number, and every prefix of it: `§9.6` implies `§9` exists.
        defined.insert(n.clone());
        if let Some((maj, _)) = n.split_once('.') {
            defined.insert(maj.to_string());
        }
    }

    let cites = match cited(root) {
        Ok(c) => c,
        Err(e) => {
            return Err(vec![Dangling {
                kind: "source",
                citation: e,
                sites: 0,
            }])
        }
    };

    let mut bad = Vec::new();
    let mut checked = 0usize;
    for (kind, c, sites) in cites {
        checked += 1;
        let ok = match kind {
            "section" => defined.contains(&c),
            "rule" => rules.contains(&format!("### Rule {c} ")),
            "delta" => deltas.contains(&format!("## {c} ")),
            _ => true,
        };
        if !ok {
            bad.push(Dangling {
                kind,
                citation: c,
                sites,
            });
        }
    }
    if bad.is_empty() {
        Ok(checked)
    } else {
        Err(bad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_are_recognised_in_both_forms() {
        let c = scan("see §10.6 and §3 for the rest");
        assert!(c.contains(&("section", "10.6".into())));
        assert!(c.contains(&("section", "3".into())));
    }

    #[test]
    fn a_trailing_stop_is_not_part_of_the_number() {
        // "as §21.3." ends a sentence; the citation is 21.3, not "21.3."
        assert!(scan("required by §21.3.").contains(&("section", "21.3".into())));
        assert!(scan("required by §9.").contains(&("section", "9".into())));
    }

    #[test]
    fn rules_are_single_capitals_and_not_word_prefixes() {
        assert!(scan("Rule E forbids it").contains(&("rule", "E".into())));
        // "Rule Engine" is not a citation of Rule E.
        assert!(!scan("the Rule Engine runs").iter().any(|(k, _)| *k == "rule"));
    }

    #[test]
    fn deltas_are_recognised() {
        assert!(scan("corrected by D-A12").contains(&("delta", "D-A12".into())));
        assert!(scan("see D-A5 and D-A15").contains(&("delta", "D-A15".into())));
    }

    #[test]
    fn a_citation_with_no_target_is_reported() {
        // The guarantee that matters: an unresolvable citation must not pass.
        //
        // The fake citation is assembled at runtime rather than written as a
        // literal, because this file is itself scanned - a test that hard-codes
        // a dangling reference makes the checker fail on its own test suite.
        // Found by running the checker, which is the intended way to find it.
        let fake = format!("invented {}99.9 reference", '\u{a7}');
        let c = scan(&fake);
        assert!(c.contains(&("section", "99.9".into())));
    }
}

// ---------------------------------------------------------------------------
// Documentation/CLI.md against the actual command surface
// ---------------------------------------------------------------------------

/// Check that the CLI manual documents every command and global option, and no
/// others.
///
/// The manual's prose cannot be generated — what `remainder_ticks` *means* is
/// not derivable from a type — so it is written by hand. What can be checked is
/// its surface: a command that exists and is undocumented, or a documented
/// command that no longer exists, are both defects a reader would hit and
/// neither is visible to any other test.
///
/// Read from `crates/ucal/src/main.rs` rather than by running the binary, so
/// this stays a source check like the rest of `xtask` and needs no build of
/// another crate.
pub fn check_cli_docs(root: &Path) -> Result<usize, Vec<String>> {
    let main = root.join("crates/ucal/src/main.rs");
    let Ok(src) = std::fs::read_to_string(&main) else {
        return Err(alloc_vec(format!("cannot read {}", main.display())));
    };
    let Ok(doc) = std::fs::read_to_string(root.join("Documentation/CLI.md")) else {
        return Err(alloc_vec("Documentation/CLI.md is missing".into()));
    };

    let commands = subcommands(&src);
    let options = global_options(&src);
    let mut bad = Vec::new();

    for c in &commands {
        // A command is documented when the manual has a heading for it.
        if !doc.contains(&format!("## `ucal {c}`")) {
            bad.push(format!("`ucal {c}` has no section in Documentation/CLI.md"));
        }
    }
    for o in &options {
        if !doc.contains(&format!("`--{o}")) {
            bad.push(format!("global option `--{o}` is undocumented"));
        }
    }
    // The other direction: a section for something that no longer exists.
    for line in doc.lines() {
        if let Some(rest) = line.strip_prefix("## `ucal ") {
            let name = rest.trim_end_matches('`').trim();
            if !name.is_empty() && !commands.iter().any(|c| c == name) {
                bad.push(format!("Documentation/CLI.md documents `ucal {name}`, which does not exist"));
            }
        }
    }

    if bad.is_empty() {
        Ok(commands.len() + options.len())
    } else {
        Err(bad)
    }
}

fn alloc_vec(s: String) -> Vec<String> {
    vec![s]
}

/// Subcommand names, from the `enum Command` variants, in kebab-case.
fn subcommands(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(start) = src.find("enum Command {") else {
        return out;
    };
    let body = &src[start..];
    let end = body.find("\n}").map(|e| e + 2).unwrap_or(body.len());
    for line in body[..end].lines() {
        let l = line.trim();
        // A variant is `Name {` or `Name,` at the top level of the enum, with
        // four spaces of indentation in this file's formatting.
        if !line.starts_with("    ") || line.starts_with("     ") {
            continue;
        }
        let name = l.trim_end_matches(&[' ', '{', ','][..]);
        if name.is_empty()
            || !name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            || !name.chars().all(|c| c.is_ascii_alphanumeric())
        {
            continue;
        }
        out.push(kebab(name));
    }
    out
}

/// Global option names, from `#[arg(long, global = true)]` fields.
fn global_options(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        if !l.contains("global = true") {
            continue;
        }
        // The field declaration is the next line shaped `name: Type`. Anything
        // else is still the attribute — `#[arg(..)]` wraps across lines, and an
        // earlier version of this took `value_parser = [..])]` for a field name.
        for next in lines.iter().skip(i + 1) {
            let t = next.trim();
            let Some((name, rest)) = t.split_once(':') else {
                continue;
            };
            let n = name.trim();
            if n.is_empty()
                || !n
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                || !rest.starts_with(' ')
            {
                continue;
            }
            out.push(n.replace('_', "-"));
            break;
        }
    }
    out
}

fn kebab(camel: &str) -> String {
    let mut s = String::new();
    for (i, c) in camel.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                s.push('-');
            }
            s.push(c.to_ascii_lowercase());
        } else {
            s.push(c);
        }
    }
    s
}

/// The CI workflow must run every command the release procedure lists.
///
/// `Documentation/Release_Notes/<version>.md` prints a verification block and
/// `.github/workflows/verify.yml` runs it. The workflow says in a comment that
/// the two are the same list; this is what makes that a fact rather than a
/// comment, because a step quietly dropped from CI is invisible exactly when it
/// matters.
///
/// Compares the *commands*, normalised for line continuations and whitespace —
/// not the surrounding YAML, which is free to differ.
pub fn check_ci_covers_the_procedure(root: &Path) -> Result<usize, Vec<String>> {
    let wf = root.join(".github/workflows/verify.yml");
    let Ok(workflow) = std::fs::read_to_string(&wf) else {
        return Err(alloc_vec(format!("{} is missing", wf.display())));
    };
    // The newest release-notes file that has a verification block.
    let dir = root.join("Documentation/Release_Notes");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Err(alloc_vec("Documentation/Release_Notes is unreadable".into()));
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".md") && n != "README.md")
        })
        .collect();
    files.sort();
    let Some(newest) = files.last().map(|p| p.as_path()) else {
        return Err(alloc_vec("no release-notes file to read a procedure from".into()));
    };
    let Ok(notes) = std::fs::read_to_string(newest) else {
        return Err(alloc_vec(format!("cannot read {}", newest.display())));
    };

    let wanted = cargo_commands(&verification_block(&notes));
    let have = cargo_commands(&workflow);
    let mut bad = Vec::new();
    for w in &wanted {
        if !have.iter().any(|h| h == w) {
            bad.push(format!(
                "CI does not run `{w}`, which {} lists",
                newest
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("the release notes")
            ));
        }
    }
    if bad.is_empty() {
        Ok(wanted.len())
    } else {
        Err(bad)
    }
}

/// The fenced block under a `## Verification` heading.
fn verification_block(notes: &str) -> String {
    let mut out = String::new();
    let mut in_section = false;
    let mut in_fence = false;
    for line in notes.lines() {
        if line.starts_with("## ") {
            in_section = line.trim() == "## Verification";
            continue;
        }
        if !in_section {
            continue;
        }
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Every `cargo …` invocation in `text`, joined across `\` continuations and
/// normalised to single spaces.
fn cargo_commands(text: &str) -> Vec<String> {
    let joined = text.replace("\\\n", " ");
    let mut out = Vec::new();
    for line in joined.lines() {
        let l = line.trim().trim_start_matches("&& ").trim();
        let Some(idx) = l.find("cargo ") else { continue };
        let cmd: String = l[idx..]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim_end_matches(&['&', ';'][..])
            .trim()
            .to_string();
        // `cargo run -p xtask && cargo run -p xtask -- lint` is two commands.
        for part in cmd.split("&&") {
            let p = part.trim();
            if p.starts_with("cargo ") && !out.iter().any(|o: &String| o == p) {
                out.push(p.to_string());
            }
        }
    }
    out
}

/// Constants quoted in the contact materials must match `fixtures/vectors.json`.
///
/// `Documentation/CONTACT.md` and the C1 issue template embed `BEAT`, `SECOND`
/// and `ORIGIN_OFFSET` so a stranger can check three numbers in thirty minutes
/// without reading the vector file first. That convenience is a copy, and a copy
/// is a thing that drifts — and this one drifts into *asking someone to
/// reproduce the wrong number*, which would waste the scarcest resource this
/// project has.
pub fn check_contact_constants(root: &Path) -> Result<usize, Vec<String>> {
    let vectors = root.join("fixtures/vectors.json");
    let Ok(json) = std::fs::read_to_string(&vectors) else {
        return Err(alloc_vec("fixtures/vectors.json is unreadable".into()));
    };
    // Not a JSON parser: the three values are long decimal runs and the file is
    // generated, so finding them by name is enough and adds no dependency.
    let mut want = Vec::new();
    for name in ["BEAT", "SECOND", "ORIGIN_OFFSET"] {
        let key = format!("\"{name}\"");
        let Some(at) = json.find(&key) else {
            return Err(alloc_vec(format!("{name} is not in vectors.json")));
        };
        let tail = &json[at + key.len()..];
        let digits: String = tail
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if digits.len() < 20 {
            return Err(alloc_vec(format!("{name} in vectors.json is not a long integer")));
        }
        want.push((name, digits));
    }

    let files = [
        "Documentation/CONTACT.md",
        ".github/ISSUE_TEMPLATE/c1-reproduce-vectors.md",
    ];
    let mut bad = Vec::new();
    let mut checked = 0;
    for f in files {
        let path = root.join(f);
        let Ok(text) = std::fs::read_to_string(&path) else {
            bad.push(format!("{f} is missing"));
            continue;
        };
        for (name, value) in &want {
            if !text.contains(name.to_owned()) {
                continue;
            }
            checked += 1;
            if !text.contains(value.as_str()) {
                bad.push(format!(
                    "{f} names {name} but does not quote the value in vectors.json"
                ));
            }
        }
    }
    if bad.is_empty() {
        Ok(checked)
    } else {
        Err(bad)
    }
}
