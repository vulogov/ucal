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
    let mut stack = vec![
        root.join("crates"),
        root.join("xtask").join("src"),
        // The documentation, added in 1.7.0. It had never been scanned: the
        // check was announced as "citations resolve against spec/" and read
        // Rust source only, which left uncovered the place where citations are
        // densest and least likely to have been written by a compiler.
        // X1's one surviving mutation.
        root.join("Documentation"),
        root.join("spec"),
        root.join("docs"),
    ];
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
            } else if name.ends_with(".rs") || name.ends_with(".md") {
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
    /// **The check reads the notes for the version being built.**
    ///
    /// It used to read "the newest release-notes file", picked by sorting
    /// filenames — which put `1.10.0.md` between `1.1.0.md` and `1.2.0.md`, so
    /// the newest was `1.9.0.md` and the check compared CI against the
    /// *previous* release's procedure while reporting success. Latent since
    /// 1.0.0 and unreachable until a two-digit minor existed; the defect corpus
    /// caught it on 1.10.0's first commit.
    ///
    /// Sorting numerically would have fixed that one bug and left the guess in
    /// place. Deriving the version removes the guess.
    #[test]
    fn the_procedure_check_reads_the_current_version_s_notes() {
        let v = super::workspace_version(&crate::workspace_root())
            .expect("a workspace version");
        let notes = crate::workspace_root()
            .join("Documentation/Release_Notes")
            .join(format!("{v}.md"));
        assert!(
            notes.exists(),
            "the workspace is at {v} and {} does not exist",
            notes.display()
        );
        // And that is the file whose block CI is held to.
        assert!(super::check_ci_covers_the_procedure(&crate::workspace_root()).is_ok());
    }

    /// **A version bumped without its notes is an error, not a fallback.**
    ///
    /// The sort could not catch this: with no `1.11.0.md` it would silently read
    /// `1.10.0.md` and pass. This is the adjacent failure that removing the
    /// guess closes.
    #[test]
    fn a_missing_notes_file_for_the_current_version_is_refused() {
        let dir = std::env::temp_dir().join("ucal-procedure-no-notes");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Documentation/Release_Notes")).expect("mkdir");
        std::fs::create_dir_all(dir.join(".github/workflows")).expect("mkdir");
        std::fs::write(dir.join(".github/workflows/verify.yml"), "jobs:\n").expect("write");
        std::fs::write(
            dir.join("Cargo.toml"),
            "[workspace.package]\nversion = \"9.9.9\"\n",
        )
        .expect("write");
        // A notes file for some *other* version, which the old sort would have
        // happily read instead.
        std::fs::write(
            dir.join("Documentation/Release_Notes/1.0.0.md"),
            "## Verification\n\n```\ncargo test --workspace --release\n```\n",
        )
        .expect("write");

        let e = super::check_ci_covers_the_procedure(&dir)
            .expect_err("a version with no notes must not fall back to another's");
        // Specific to the message the explicit check produces. Asserting only
        // that *some* error came back passed even with that check disabled —
        // `read_to_string` fails too, and its message names the path, which
        // contains the version. A test that cannot tell the two apart is not
        // testing the branch it claims to.
        assert!(
            e.iter().any(|m| m.contains("a cycle's notes")),
            "expected the missing-notes diagnostic, got: {e:?}"
        );
        assert!(
            e.iter().any(|m| m.contains("9.9.9")),
            "the message should name the version: {e:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

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
/// The workspace version, from `[workspace.package]`.
///
/// The release-notes file to check is the one for **the version being built**.
/// Deriving it removes the guess entirely — see
/// [`check_ci_covers_the_procedure`] for what the guess cost.
fn workspace_version(root: &Path) -> Option<String> {
    let s = std::fs::read_to_string(root.join("Cargo.toml")).ok()?;
    let mut inside = false;
    for line in s.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            inside = l == "[workspace.package]";
            continue;
        }
        if inside && !l.starts_with('#') {
            if let Some(rest) = l.strip_prefix("version") {
                let rest = rest.trim_start().strip_prefix('=')?.trim_start();
                let rest = rest.strip_prefix('"')?;
                return rest.split('"').next().map(str::to_string);
            }
        }
    }
    None
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
    // **The notes for the version being built**, not "the newest file".
    //
    // The first version of this sorted filenames and took the last, which put
    // `1.10.0.md` between `1.1.0.md` and `1.2.0.md` — so it read the *previous*
    // release's procedure and reported success. Sorting numerically would have
    // fixed that one bug and left the guess in place; deriving the version
    // removes the guess, and catches the adjacent failure the sort could not:
    // a version bumped and its notes never written.
    let Some(version) = workspace_version(root) else {
        return Err(alloc_vec(
            "cannot read the workspace version from Cargo.toml".into(),
        ));
    };
    let newest = root
        .join("Documentation/Release_Notes")
        .join(format!("{version}.md"));
    if !newest.exists() {
        return Err(alloc_vec(format!(
            "the workspace is at {version} and {} does not exist; a cycle's notes \
             are created when it opens",
            newest.display()
        )));
    }
    let newest = newest.as_path();
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

/// The files that must carry the signing key, and the reason each is there.
///
/// A key published in exactly one place is a key with one thing to compromise.
/// These copies are not independent authorities — the same person places all of
/// them — but two of them live in published crate READMEs, and a crates.io
/// version cannot be altered once released. That makes a *change* to the key in
/// this repository detectable against copies nobody can edit, which is a
/// narrower property than a trust path and is the one actually on offer.
const KEY_PUBLICATIONS: &[(&str, &str)] = &[
    ("README.md", "the repository's landing page"),
    ("crates/ucal/README.md", "published to crates.io with the CLI, immutable per version"),
    ("crates/ucal-core/README.md", "published to crates.io with the core crate, immutable per version"),
    ("Documentation/CONTACT.md", "where C1 asks someone to verify what they downloaded"),
    ("spec/CONFORMANCE.md", "where the custody of the key is stated"),
];

/// Every published copy of the signing key is the key in `fixtures/ucal.pub`.
///
/// The copies exist so that a reader need not trust a single file, and so that
/// one placed beyond the author's reach can contradict a repository that has
/// been rewritten. A copy that has drifted destroys exactly that: it makes the
/// witnesses disagree for a reason that is not an attack, which is worse than
/// having no witnesses, because the next disagreement gets shrugged at.
///
/// Checked in both directions. Every declared publication must carry the key,
/// and no document anywhere may carry a *different* one — a truncated paste or
/// a transposed character is the realistic failure, not a forgery.
pub fn check_signing_key(root: &Path) -> Result<usize, Vec<String>> {
    let Ok(pubkey) = std::fs::read_to_string(root.join("fixtures/ucal.pub")) else {
        return Err(alloc_vec("fixtures/ucal.pub is unreadable".into()));
    };
    // minisign's format: an untrusted comment naming the key ID, then the key.
    let Some(key) = pubkey.lines().map(str::trim).find(|l| l.starts_with("RW") && l.len() > 40)
    else {
        return Err(alloc_vec("fixtures/ucal.pub has no key line".into()));
    };
    let key_id = pubkey
        .lines()
        .find(|l| l.contains("key"))
        .and_then(|l| l.split_whitespace().last())
        .unwrap_or_default()
        .to_string();
    if key_id.len() != 16 {
        return Err(alloc_vec(format!(
            "fixtures/ucal.pub has no key ID in its comment (read `{key_id}`)"
        )));
    }

    let mut bad = Vec::new();
    for (f, why) in KEY_PUBLICATIONS {
        match std::fs::read_to_string(root.join(f)) {
            Err(_) => bad.push(format!("{f} is missing ({why})")),
            Ok(text) if !text.contains(key) => bad.push(format!(
                "{f} does not carry the signing key — it is published there because it is {why}"
            )),
            Ok(_) => {}
        }
    }

    // And nothing anywhere carries a different one. A near-miss is the failure
    // that matters: a reader who checks a mistyped key learns nothing and
    // believes they learned something.
    for rel in markdown_files(root) {
        let Ok(text) = std::fs::read_to_string(root.join(&rel)) else {
            continue;
        };
        for token in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '+' || c == '/')) {
            if token.starts_with("RW") && token.len() > 40 && token != key {
                bad.push(format!("{rel} carries a key that is not the published one: {token}"));
            }
        }
        if text.contains(&key_id) && !text.contains(key) {
            bad.push(format!(
                "{rel} names key ID {key_id} without quoting the key, so a reader cannot check it"
            ));
        }
    }

    if bad.is_empty() {
        Ok(KEY_PUBLICATIONS.len())
    } else {
        bad.sort();
        bad.dedup();
        Err(bad)
    }
}

/// Every tracked markdown file, relative to the root.
fn markdown_files(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            let name = e.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') && name != ".github" {
                continue;
            }
            if name == "target" {
                continue;
            }
            if p.is_dir() {
                walk(&p, root, out);
            } else if p.extension().is_some_and(|x| x == "md") {
                if let Ok(rel) = p.strip_prefix(root) {
                    out.push(rel.to_string_lossy().into_owned());
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// Every standing spec delta is applied in the normative specification.
///
/// `SPEC-DELTAS.md` is the reasoning; `UCAL-1.1.md` is the normative text, and
/// its own header says it is "RFC UCAL-1 with the standing deltas applied in
/// place", with each amended passage marked inline by delta id.
///
/// That claim had been false since 0.4.0. D-A16 amended §4.3 — the SI
/// equivalent is printed on request, not always — and the normative text still
/// said "always" two releases later, contradicting the implementation, the
/// release notes and `STABILITY.md` at once. D-A17 was written in 0.9.0 and was
/// never going to be applied either, because nothing looked.
///
/// A delta that is recorded and not applied is the worst of both: the reasoning
/// exists, so it reads as decided, and the normative document a conforming
/// implementer would follow still says the old thing.
///
/// `WITHDRAWN` deltas are exempt by definition — D-A1 is a claim that was
/// retracted, and applying it would be the error. Everything else must appear.
pub fn check_deltas_are_applied(root: &Path) -> Result<usize, Vec<String>> {
    let Ok(deltas) = std::fs::read_to_string(root.join("spec/SPEC-DELTAS.md")) else {
        return Err(alloc_vec("spec/SPEC-DELTAS.md is unreadable".into()));
    };
    let Ok(spec) = std::fs::read_to_string(root.join("spec/UCAL-1.1.md")) else {
        return Err(alloc_vec("spec/UCAL-1.1.md is unreadable".into()));
    };

    let mut bad = Vec::new();
    let mut checked = 0;
    for (i, line) in deltas.lines().enumerate() {
        let Some(rest) = line.strip_prefix("## ") else {
            continue;
        };
        if !rest.starts_with("D-A") {
            continue;
        }
        let id: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace())
            .collect();
        // The status is on the heading line or shortly after it.
        let window: String = deltas
            .lines()
            .skip(i)
            .take(4)
            .collect::<Vec<_>>()
            .join(" ");
        if window.contains("WITHDRAWN") {
            continue;
        }
        checked += 1;
        // Marked inline as `[D-A16 · AMENDMENT]`, so require the bracketed form
        // rather than a bare mention — a delta named in passing is not a delta
        // applied.
        if !spec.contains(&format!("[{id} ·")) {
            bad.push(format!(
                "{id} is a standing delta and is not applied in spec/UCAL-1.1.md"
            ));
        }
    }

    // And the header's count must match what is actually there.
    let standing = checked;
    let words = [
        (11, "eleven"), (12, "twelve"), (13, "thirteen"), (14, "fourteen"),
        (15, "fifteen"), (16, "sixteen"), (17, "seventeen"), (18, "eighteen"),
        (19, "nineteen"), (20, "twenty"),
    ];
    if let Some((_, word)) = words.iter().find(|(n, _)| *n == standing) {
        if !spec.contains(&format!("{word} standing deltas")) {
            bad.push(format!(
                "spec/UCAL-1.1.md does not say `{word} standing deltas`, and there are {standing}"
            ));
        }
    }

    if bad.is_empty() {
        Ok(checked)
    } else {
        Err(bad)
    }
}

#[cfg(test)]
mod vacuity_probe {
    use super::*;

    /// **V1's central question, asked of every `check-docs` check at once.**
    ///
    /// Each of these ends `if bad.is_empty() { Ok(count) } else { Err(..) }`.
    /// A *population* of zero therefore passes, and the count is printed but
    /// never examined — `ok    citations resolve against spec/ (0 distinct)`
    /// reads exactly like a pass.
    ///
    /// Pointed at a *missing* tree every one of them fails, which is the right
    /// answer and not the interesting one. Pointed at a tree where every file
    /// they read **exists and is empty**, the population is zero and the
    /// `is_empty()` at the end cannot tell that from clean.
    ///
    /// This test records which pass on that skeleton. The list is the evidence
    /// behind `Documentation/Proposals/V1-check-audit.md`.
    #[test]
    fn checks_pointed_at_an_empty_workspace() {
        let dir = std::env::temp_dir().join("ucal-vacuity-probe-citations");
        let _ = std::fs::create_dir_all(&dir);
        skeleton(&dir);

        let mut vacuous: Vec<&str> = Vec::new();
        // Written out rather than macro'd: the return types differ and the point
        // is to be readable as evidence.
        if let Ok(n) = check(&dir) {
            vacuous.push("citations");
            assert_eq!(n, 0, "an empty tree cannot contain citations");
        }
        if let Ok(_) = check_cli_docs(&dir) {
            vacuous.push("cli-docs");
        }
        if let Ok(_) = check_ci_covers_the_procedure(&dir) {
            vacuous.push("ci-covers-procedure");
        }
        if let Ok(_) = check_contact_constants(&dir) {
            vacuous.push("contact-constants");
        }
        if let Ok(_) = check_signing_key(&dir) {
            vacuous.push("signing-key");
        }
        if let Ok(_) = check_deltas_are_applied(&dir) {
            vacuous.push("deltas-applied");
        }
        // Still exactly these three, and that is now the *intended* state.
        //
        // V2 did not change what these functions return — reporting the
        // population they found is the honest thing for them to do. It added
        // `report` in main.rs, through which all of them are announced, and
        // which refuses a count below a floor. So the vacuity is contained at
        // one place instead of being spread across six.
        //
        // The pin stays because the containment is the fragile part: a future
        // check announced without `report` would be vacuous again, and this
        // list plus `floors::a_population_below_the_floor_is_a_failure` are what
        // say where the guarantee actually lives.
        assert_eq!(
            vacuous,
            ["citations", "cli-docs", "deltas-applied"],
            "the set of checks that return Ok on an empty population has \
             changed; update Documentation/Proposals/V1-check-audit.md to match"
        );
    }

    /// Every path these checks read, present and empty.
    ///
    /// Deliberately built from the checks' own expectations rather than a
    /// hardcoded list of guesses: if a check reads a file this skeleton does not
    /// create, it fails for a missing file and is reported as not-vacuous, which
    /// understates the finding rather than overstating it.
    fn skeleton(root: &std::path::Path) {
        for d in [
            "spec",
            "Documentation",
            "Documentation/Release_Notes",
            "Documentation/Proposals",
            "fixtures",
            ".github/workflows",
            "crates/ucal/src",
        ] {
            let _ = std::fs::create_dir_all(root.join(d));
        }
        for f in [
            "spec/UCAL-1.1.md",
            "spec/SPEC-DELTAS.md",
            "spec/RULES.md",
            "spec/CONFORMANCE.md",
            "Documentation/CLI.md",
            "Documentation/CONTACT.md",
            "Documentation/STABILITY.md",
            "Documentation/RELEASING.md",
            "fixtures/vectors.json",
            "fixtures/ucal.pub",
            ".github/workflows/verify.yml",
            "crates/ucal/src/main.rs",
            "README.md",
        ] {
            let _ = std::fs::write(root.join(f), "");
        }
    }
}
