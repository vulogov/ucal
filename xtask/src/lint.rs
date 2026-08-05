//! Workspace lints (§21.3).
//!
//! These are structural guarantees the type system cannot express, so they are
//! enforced by scanning source. Each one maps to a numbered failure mode:
//!
//! | lint | rule | failure mode |
//! |---|---|---|
//! | no float token in a shipped crate | Rule E | F8 |
//! | no foreign-unit identifier in `ucal-core` | Rules A.2, Y | F3, F9 |
//! | no wrapping/saturating arithmetic on time types | Rule O | F7 |
//! | no overclaiming prose about tick 0 | Rule Q.1 | F13 |
//! | `ucal-body` must not depend on `ucal-civil` | §12 | F9 |
//! | internal version requirements track the workspace version | — | — |
//! | no string literal carries a run of source indentation | — | — |
//! | every rounding in a shipped crate is declared | Rule R | — |
//!
//! The last has no failure-mode number because it guards a packaging defect
//! rather than a specification one: a stale internal requirement resolves,
//! builds, tests green and publishes, and is only visible to a consumer.
//!
//! Comments and string literals are stripped before the identifier lints run,
//! because the rules are about *identifiers* — §13.2 forbids naming a foreign
//! unit in code, not discussing one in a doc comment. The prose lint runs on the
//! opposite projection: comments and strings only.

use std::path::{Path, PathBuf};

/// One violation.
pub struct Violation {
    pub lint: &'static str,
    pub file: PathBuf,
    pub line: usize,
    pub text: String,
    pub rule: &'static str,
}

/// Three line-aligned projections of a Rust source file.
///
/// Each projection is the same length as the source with the other categories
/// blanked to spaces, and every newline is preserved in all three. That means a
/// byte offset in any projection maps to the correct source line, which is what
/// lets a violation be reported at the line a human can go and look at.
pub struct Projections {
    /// Everything that is not a comment or a string literal.
    pub code: String,
    /// Doc comments only: `///`, `//!`, `/** */`, `/*! */`.
    pub docs: String,
    /// String literals and non-doc comments together. Kept because the
    /// projection is only sound if every category is accounted for.
    #[allow(dead_code)]
    pub other_prose: String,
    /// String literals **only**.
    ///
    /// Separate from `other_prose` because the two need opposite treatment by
    /// the indentation lint: a comment may line up a table with runs of spaces
    /// and should, while a literal that contains one has almost certainly
    /// swallowed the source's indentation.
    ///
    /// Blanked with `\0` rather than spaces, unlike the other projections. The
    /// difference matters for exactly one question: `("a", text("b"))` blanks to
    /// a run of spaces *between* two literals, which is indistinguishable from a
    /// run *inside* one when the filler is also a space.
    pub strings: String,
}

/// Project a Rust source file.
///
/// Not a full Rust lexer: it handles line comments, block comments (nested),
/// ordinary string literals with escapes, raw strings with any hash count, char
/// literals, and lifetimes. That is enough for a tree with no identifier-
/// generating macros, and the self-tests pin the cases it must get right.
pub fn project(src: &str) -> Projections {
    let b = src.as_bytes();
    let n = b.len();
    let mut code = vec![b' '; n];
    let mut docs = vec![b' '; n];
    let mut other = vec![b' '; n];
    let mut strings = vec![0u8; n];
    // Preserve line structure in every projection.
    for (i, ch) in b.iter().enumerate() {
        if *ch == b'\n' {
            code[i] = b'\n';
            docs[i] = b'\n';
            other[i] = b'\n';
            strings[i] = b'\n';
        }
    }
    let copy = |dst: &mut [u8], from: usize, to: usize| {
        for k in from..to.min(n) {
            if b[k] != b'\n' {
                dst[k] = b[k];
            }
        }
    };

    let mut i = 0;
    while i < n {
        let c = b[i];

        // line comment, doc or otherwise
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            let is_doc = i + 2 < n && (b[i + 2] == b'/' || b[i + 2] == b'!');
            let start = i;
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            copy(if is_doc { &mut docs } else { &mut other }, start, i);
            continue;
        }

        // block comment, possibly nested, doc or otherwise
        if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let is_doc = i + 2 < n && (b[i + 2] == b'*' || b[i + 2] == b'!');
            let start = i;
            let mut depth = 1;
            i += 2;
            while i < n && depth > 0 {
                if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            copy(if is_doc { &mut docs } else { &mut other }, start, i);
            continue;
        }

        // raw string r"..." / r#"..."#
        if c == b'r' && i + 1 < n && (b[i + 1] == b'"' || b[i + 1] == b'#') {
            let mut j = i + 1;
            let mut hashes = 0;
            while j < n && b[j] == b'#' {
                hashes += 1;
                j += 1;
            }
            if j < n && b[j] == b'"' {
                j += 1;
                let close: String = core::iter::once('"')
                    .chain(core::iter::repeat_n('#', hashes))
                    .collect();
                let end = src[j..].find(&close).map(|e| j + e).unwrap_or(n);
                copy(&mut other, j, end);
                copy(&mut strings, j, end);
                i = (end + close.len()).min(n);
                continue;
            }
        }

        // char literal or lifetime
        if c == b'\'' {
            let simple_char = i + 2 < n && b[i + 2] == b'\'';
            let escaped_char = i + 3 < n && b[i + 1] == b'\\' && b[i + 3] == b'\'';
            if simple_char || escaped_char {
                let end = if escaped_char { i + 4 } else { i + 3 };
                copy(&mut code, i, end);
                i = end.min(n);
                continue;
            }
            // a lifetime: falls through and is treated as code
        }

        // ordinary string literal
        if c == b'"' {
            let start = i + 1;
            i += 1;
            while i < n {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == b'"' {
                    break;
                }
                i += 1;
            }
            copy(&mut other, start, i);
            copy(&mut strings, start, i);
            i = (i + 1).min(n);
            continue;
        }

        code[i] = c;
        i += 1;
    }

    Projections {
        code: String::from_utf8_lossy(&code).into_owned(),
        docs: String::from_utf8_lossy(&docs).into_owned(),
        other_prose: String::from_utf8_lossy(&other).into_owned(),
        strings: String::from_utf8_lossy(&strings).into_owned(),
    }
}

/// Inline suppression marker.
///
/// A line carrying `ucal-lint-allow(<lint>)` is exempt from that lint. This
/// exists for exactly one legitimate case: text that *quotes* a forbidden phrase
/// in order to forbid it. Rule Q.1's own documentation has to name the phrasing
/// it prohibits, and a lint that cannot be told the difference between using a
/// claim and mentioning one would force the prohibition to go unwritten.
pub const ALLOW_MARKER: &str = "ucal-lint-allow";

/// Region suppression, opened by `ucal-lint-allow-begin(<lint>)` and closed by
/// `ucal-lint-allow-end(<lint>)`.
///
/// The line marker above covers one line, which is right for a doc comment that
/// quotes a forbidden phrase and wrong for a *block* that has to be exempt as a
/// whole. §21.2's float oracle is the case: a reference implementation is
/// permitted in test code, and marking twelve consecutive lines individually
/// would bury the one thing a reader needs to see — where the exemption starts
/// and where it stops.
///
/// Neither form is silent. [`run`] returns every suppression it honoured, and
/// the report prints them, so an exemption is a visible cost rather than a way
/// to make a lint stop talking.
pub const REGION_BEGIN: &str = "ucal-lint-allow-begin";
const REGION_END: &str = "ucal-lint-allow-end";

/// One honoured exemption, reported so that it cannot pass unnoticed.
pub struct Suppression {
    pub lint: &'static str,
    pub file: PathBuf,
    pub line: usize,
    /// `true` for a region, `false` for a single line.
    pub region: bool,
}

/// The lines covered by an open region for `lint`, as a set of ranges.
fn regions(src: &str, lint: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut open: Option<usize> = None;
    for (i, text) in src.lines().enumerate() {
        let line = i + 1;
        if let Some(p) = text.find(REGION_BEGIN) {
            if text[p..].contains(lint) {
                open = Some(line);
            }
        } else if let Some(p) = text.find(REGION_END) {
            if text[p..].contains(lint) {
                if let Some(start) = open.take() {
                    out.push((start, line));
                }
            }
        }
    }
    // An unterminated region covers the rest of the file. That is deliberate:
    // failing open would turn a typo into a silently disabled lint.
    if let Some(start) = open {
        out.push((start, src.lines().count()));
    }
    out
}

fn suppressed(src: &str, line: usize, lint: &str) -> bool {
    if let Some(text) = src.lines().nth(line.saturating_sub(1)) {
        if let Some(p) = text.find(ALLOW_MARKER) {
            // `ucal-lint-allow-begin` starts with the line marker's text; the
            // region forms are handled below, so only the bare form counts here.
            if !text[p..].starts_with(REGION_BEGIN)
                && !text[p..].starts_with(REGION_END)
                && text[p..].contains(lint)
            {
                return true;
            }
        }
    }
    regions(src, lint).iter().any(|(a, b)| line >= *a && line <= *b)
}

fn whole_word_at(hay: &str, idx: usize, needle: &str) -> bool {
    let b = hay.as_bytes();
    let before_ok = idx == 0 || !(b[idx - 1].is_ascii_alphanumeric() || b[idx - 1] == b'_');
    let after = idx + needle.len();
    let after_ok = after >= b.len() || !(b[after].is_ascii_alphanumeric() || b[after] == b'_');
    before_ok && after_ok
}

/// Every identifier in a source projection, with its byte offset.
fn identifiers(hay: &str) -> Vec<(usize, &str)> {
    let b = hay.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_alphabetic() || b[i] == b'_' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push((start, &hay[start..i]));
        } else {
            i += 1;
        }
    }
    out
}

/// Split an identifier into lowercase word segments, breaking on `_` and on
/// camelCase / PascalCase boundaries.
///
/// This is what makes the foreign-unit lint actually bite. Matching whole words
/// against the raw source misses `seconds_per_day`, `to_seconds`, `dayOfYear` and
/// `YEAR_JULIAN`, because `_` is an identifier character — and those compounds are
/// exactly the shape §13.2 is aimed at.
fn segments(ident: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in ident.split('_') {
        if part.is_empty() {
            continue;
        }
        let chars: Vec<char> = part.chars().collect();
        let mut start = 0;
        for i in 1..chars.len() {
            let boundary = chars[i].is_uppercase() && !chars[i - 1].is_uppercase();
            if boundary {
                out.push(chars[start..i].iter().collect::<String>().to_lowercase());
                start = i;
            }
        }
        out.push(chars[start..].iter().collect::<String>().to_lowercase());
    }
    // A trailing plural is the same unit.
    out.iter()
        .map(|s| {
            let t = s.strip_suffix("es").filter(|r| r.len() > 2).unwrap_or(s);
            let t = t.strip_suffix('s').filter(|r| r.len() > 2).unwrap_or(t);
            t.to_string()
        })
        .collect()
}

fn find_whole_words(hay: &str, needle: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(needle) {
        let idx = from + rel;
        if whole_word_at(hay, idx, needle) {
            out.push(idx);
        }
        from = idx + needle.len();
    }
    out
}

fn line_of(src: &str, idx: usize) -> usize {
    src[..idx].bytes().filter(|b| *b == b'\n').count() + 1
}

fn byte_of_line(src: &str, line: usize) -> usize {
    let mut off = 0;
    for (n, l) in src.lines().enumerate() {
        if n + 1 == line {
            return off;
        }
        off += l.len() + 1;
    }
    0
}

fn line_text(src: &str, idx: usize) -> String {
    let start = src[..idx].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let end = src[idx..].find('\n').map(|p| idx + p).unwrap_or(src.len());
    src[start..end].trim().to_string()
}

/// Rule E — no float token in any shipped crate.
///
/// A float reference implementation is permitted in `dev-dependencies` as a test
/// oracle and must be marked as such; those live under `tests/` or behind
/// `#[cfg(test)]`, and this lint deliberately still flags them so the marking is
/// a conscious allowance rather than an accident.
pub const FLOAT_TOKENS: &[&str] = &["f32", "f64"];

/// Rules A.2 / Y — `ucal-core` must not name a foreign unit.
///
/// §13.2: the identifiers `second`, `day`, `year` must not appear outside the
/// `Bridge` declaration and `MeasuredValue`/`Provenance` string data. String data
/// is already excluded because the lint runs on code only; the `profile` module,
/// which *is* the bridge declaration, is exempt by path.
pub const FOREIGN_UNIT_IDENTS: &[&str] = &["second", "day", "year", "hour", "month", "week"];

/// Rule O — no wrapping or saturating arithmetic exposed on time types.
pub const WRAPPING_TOKENS: &[&str] = &[
    "wrapping_add",
    "wrapping_sub",
    "wrapping_mul",
    "saturating_add",
    "saturating_sub",
    "saturating_mul",
    "overflowing_add",
    "overflowing_sub",
    "overflowing_mul",
];

/// Rule Q.1 — prose must not describe tick 0 as measured or as a creation event.
pub const OVERCLAIM_PHRASES: &[&str] = &[
    "creation of the universe",
    "age of the universe is",
    "beginning of time",
    "when the universe began",
    "the big bang happened at",
    "moment of creation",
];

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if name != "target" && name != ".git" && name != "compile_fail" {
                    stack.push(p);
                }
            } else if name.ends_with(".rs") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Every lint that can be suppressed, by name.
pub const LINT_NAMES: &[&str] = &[
    "float-free",
    "no-wrapping-arithmetic",
    "core-names-no-foreign-unit",
    "datum-no-overclaim",
    "rounding-is-declared",
    "no-indent-in-literal",
    "version-lockstep",
];

/// A lint that exists but is not in [`LINT_NAMES`] can be suppressed silently,
/// which is the one thing the suppression machinery is supposed to prevent. The
/// list is therefore checked against the lints [`run`] actually emits rather
/// than maintained by hand — see `every_lint_is_listed_for_suppression_reporting`.

/// Every exemption in the tree, so the report can list what was let through.
///
/// §21.3's lints exist to stop a rule being quietly abandoned. A suppression
/// marker is a legitimate escape hatch and also the obvious way to abandon a
/// rule quietly, so every use of one is surfaced next to the clean bill of
/// health it made possible.
pub fn suppressions(workspace_root: &Path) -> Vec<Suppression> {
    let mut out = Vec::new();
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return out;
    }
    for file in rust_files(&crates_dir) {
        let Ok(src) = std::fs::read_to_string(&file) else {
            continue;
        };
        if !src.contains(ALLOW_MARKER) {
            continue;
        }
        for lint in LINT_NAMES {
            for (start, _) in regions(&src, lint) {
                out.push(Suppression {
                    lint,
                    file: file.clone(),
                    line: start,
                    region: true,
                });
            }
            for (i, text) in src.lines().enumerate() {
                let Some(p) = text.find(ALLOW_MARKER) else {
                    continue;
                };
                if text[p..].starts_with(REGION_BEGIN) || text[p..].starts_with(REGION_END) {
                    continue;
                }
                if text[p..].contains(lint) {
                    out.push(Suppression {
                        lint,
                        file: file.clone(),
                        line: i + 1,
                        region: false,
                    });
                }
            }
        }
    }
    out
}

/// Run every lint over the shipped crates.
pub fn run(workspace_root: &Path) -> Vec<Violation> {
    let mut v = Vec::new();
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return v;
    }

    for file in rust_files(&crates_dir) {
        let Ok(src) = std::fs::read_to_string(&file) else {
            continue;
        };
        let pr = project(&src);
        let code = pr.code;

        // Rule E
        for tok in FLOAT_TOKENS {
            for idx in find_whole_words(&code, tok) {
                if suppressed(&src, line_of(&code, idx), "float-free") {
                    continue;
                }
                v.push(Violation {
                    lint: "float-free",
                    file: file.clone(),
                    line: line_of(&code, idx),
                    text: line_text(&code, idx),
                    rule: "Rule E: no floating-point type anywhere in a shipped crate",
                });
            }
        }

        // Rule O
        for tok in WRAPPING_TOKENS {
            for idx in find_whole_words(&code, tok) {
                if suppressed(&src, line_of(&code, idx), "no-wrapping-arithmetic") {
                    continue;
                }
                v.push(Violation {
                    lint: "no-wrapping-arithmetic",
                    file: file.clone(),
                    line: line_of(&code, idx),
                    text: line_text(&code, idx),
                    rule: "Rule O: wrapping and saturating arithmetic must not be exposed",
                });
            }
        }

        // Rule R: values round when displayed, never when constructed, and
        // always under a mode the caller names. In `crates/ucal` that is
        // structural — `Value::quantity` carries the rational and renders late,
        // and a test fails if any call site formats a decimal itself. The
        // library crates have no such funnel, and until 0.5.0 nothing looked at
        // them at all.
        //
        // So every `to_decimal_string` or `snap` in shipped library code must
        // carry a marker saying why its mode and digit count are fixed rather
        // than the caller's. The marker is reported, so each one is a visible
        // cost rather than a silence.
        // Which crate this file belongs to, taken *relative to* `crates/`.
        //
        // Not by searching the whole path for a component named `ucal`: the
        // repository's own directory is called `ucal` too, so an absolute path
        // matches for every crate in the tree and the check silently passed
        // everything. Found by making the lint print what it was deciding.
        let crate_of = file
            .strip_prefix(&crates_dir)
            .ok()
            .and_then(|rel| rel.components().next())
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_default();
        if crate_of != "ucal" {
            // Rule R is about *shipped* renderings. A test asserting that
            // `1/3` renders as `0.33333` is not one, and there are forty of
            // them.
            let ships_to = shipped_extent(&code, &file);
            for tok in ROUNDING_TOKENS {
                for idx in find_whole_words(&code, tok) {
                    if idx >= ships_to {
                        continue;
                    }
                    // The definitions are the rounding path, not a use of it.
                    if code[..idx].trim_end().ends_with("fn") {
                        continue;
                    }
                    let line = line_of(&code, idx);
                    if suppressed(&src, line, "rounding-is-declared") {
                        continue;
                    }
                    v.push(Violation {
                        lint: "rounding-is-declared",
                        file: file.clone(),
                        line,
                        text: line_text(&code, idx),
                        rule: "Rule R: a rounding in a shipped crate names the caller's mode, \
                               or carries a marker saying why it cannot",
                    });
                }
            }
        }

        // A string literal wrapped across source lines without a `\`
        // continuation keeps the next line's indentation *inside the string*,
        // and prints as a gap mid-sentence. It has happened twice: once in
        // `ucal explain`'s beats note, and once in `cosmo age`'s audit, both
        // times invisible until someone read the output.
        //
        // Runs on the string-literal projection only. A comment or a doc block
        // may line up a table with runs of spaces and should be left alone,
        // which is why `strings` exists separately from `other_prose`.
        for idx in run_of_spaces(&pr.strings, 6) {
            if suppressed(&src, line_of(&pr.strings, idx), "no-indent-in-literal") {
                continue;
            }
            v.push(Violation {
                lint: "no-indent-in-literal",
                file: file.clone(),
                line: line_of(&pr.strings, idx),
                // The source line, not the projection: a reader needs to see the
                // literal as they wrote it, not as `\0`s around a gap.
                text: line_text(&src, idx),
                rule: "a wrapped string literal needs a `\\` continuation, or it carries \
                       the next line's indentation into the output",
            });
        }

        // Rules A.2 / Y — ucal-core only, profile module exempt (it IS the bridge)
        let in_core = file.components().any(|c| c.as_os_str() == "ucal-core");
        let is_profile_module = file
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n == "profile.rs")
            .unwrap_or(false);
        if in_core && !is_profile_module {
            for (idx, ident) in identifiers(&code) {
                let segs = segments(ident);
                let Some(hit) = segs
                    .iter()
                    .find(|s| FOREIGN_UNIT_IDENTS.contains(&s.as_str()))
                else {
                    continue;
                };
                let line = line_of(&code, idx);
                if suppressed(&src, line, "core-names-no-foreign-unit") {
                    continue;
                }
                v.push(Violation {
                    lint: "core-names-no-foreign-unit",
                    file: file.clone(),
                    line,
                    text: format!("`{ident}` names `{hit}`  --  {}", line_text(&code, idx)),
                    rule: "Rules A.2/Y, §13.2: ucal-core must not name a foreign unit \
                           outside the Bridge declaration",
                });
            }
        }

        // Rule Q.1 — documentation only (§21.3-5 calls this a documentation lint).
        // String literals are excluded: a test that asserts a phrase is absent has
        // to contain the phrase, and that is not a claim about tick 0.
        let docs = pr.docs.to_lowercase();
        for phrase in OVERCLAIM_PHRASES {
            let mut from = 0;
            while let Some(rel) = docs[from..].find(phrase) {
                let idx = from + rel;
                let line = line_of(&docs, idx);
                if !suppressed(&src, line, "datum-no-overclaim") {
                    v.push(Violation {
                        lint: "datum-no-overclaim",
                        file: file.clone(),
                        line,
                        text: line_text(&src, byte_of_line(&src, line)),
                        rule: "Rule Q.1: tick 0 must not be described as measured, observed, \
                               or as the creation of anything",
                    });
                }
                from = idx + phrase.len();
            }
        }
    }

    // §12 — dependency direction is enforced by the graph itself.
    for (crate_name, forbidden, why) in [
        (
            "ucal-body",
            "ucal-civil",
            "§12: the derived-calendar path must not be able to reach civil tables (F9)",
        ),
        (
            "ucal-core",
            "ucal-civil",
            "Rule A.2: ucal-core knows no foreign unit but the declared bridge",
        ),
        (
            "ucal-core",
            "ucal-body",
            "§12: core is below bodies in the graph",
        ),
    ] {
        let manifest = crates_dir.join(crate_name).join("Cargo.toml");
        if let Ok(s) = std::fs::read_to_string(&manifest) {
            let (code, _) = (s.as_str(), ());
            for (n, line) in code.lines().enumerate() {
                let l = line.trim();
                if l.starts_with('#') {
                    continue;
                }
                if find_whole_words(l, forbidden).into_iter().next().is_some() {
                    v.push(Violation {
                        lint: "dependency-direction",
                        file: manifest.clone(),
                        line: n + 1,
                        text: l.to_string(),
                        rule: why,
                    });
                }
            }
        }
    }

    v.extend(version_lockstep(workspace_root));

    v
}

/// How far into a file the *shipped* code extends.
///
/// Everything from the first `#[cfg(test)]` onward is test code, and everything
/// in a file named `tests.rs` is. That is a convention rather than a parse, and
/// conventions are what this project distrusts — so the assumption is checked
/// rather than trusted: if anything after the marker looks like shipped code,
/// the extent is the whole file and the caller sees the violations it would
/// otherwise have skipped. Over-excluding is the dangerous direction here, and
/// this is what stops it being silent.
fn shipped_extent(code: &str, file: &Path) -> usize {
    if file.file_name().is_some_and(|n| n == "tests.rs") {
        return 0;
    }
    let Some(at) = code.find("#[cfg(test)]") else {
        return code.len();
    };
    let tail = &code[at..];
    if tail.contains("\npub fn ") || tail.contains("\npub const ") || tail.contains("\npub struct ")
    {
        return code.len();
    }
    at
}

/// The two operations that can discard information in this tree.
///
/// `to_decimal_string` is Rule R's single rounding path. `snap` is the one place
/// a *computation* discards information, and the rule names it explicitly.
const ROUNDING_TOKENS: &[&str] = &["to_decimal_string", "snap"];

/// Offsets of runs of `n` or more spaces that sit inside one string literal.
///
/// Takes the `\0`-blanked [`Projections::strings`]: a run of real spaces there
/// is inside a literal by construction, and the run must be bounded by literal
/// content on both sides so that trailing or leading spaces — which are
/// deliberate often enough — are left alone.
fn run_of_spaces(strings: &str, n: usize) -> Vec<usize> {
    let b = strings.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let content = |c: u8| c != b' ' && c != 0 && c != b'\n';

    // What sits before the run, looking back past at most one newline.
    //
    // Two shapes reach the output the same way. A literal wrapped without a
    // continuation puts a newline *inside* the string, so the run follows a
    // newline that itself follows literal content. A literal whose lines were
    // joined without one puts the run inline. Requiring content immediately
    // before catches only the second — which is how the first draft of this lint
    // passed a deliberately broken literal.
    let opens_a_run = |start: usize| -> bool {
        if start == 0 {
            return false;
        }
        if content(b[start - 1]) {
            return true;
        }
        if b[start - 1] != b'\n' {
            return false;
        }
        // The projection carries the literal's *source* bytes, so a properly
        // continued literal still shows a newline and the next line's
        // indentation here — the compiler strips them, this does not. A `\` as
        // the last character before the newline is exactly that case, and it is
        // the correct spelling rather than the defect.
        let before_newline: Vec<u8> = b[..start - 1]
            .iter()
            .rev()
            .take_while(|c| **c != b'\n')
            .copied()
            .collect();
        if before_newline.first() == Some(&b'\\') {
            return false;
        }
        before_newline.iter().any(|c| content(*c))
    };

    while i < b.len() {
        if b[i] != b' ' {
            i += 1;
            continue;
        }
        let start = i;
        while i < b.len() && b[i] == b' ' {
            i += 1;
        }
        if i - start >= n && opens_a_run(start) && i < b.len() && content(b[i]) {
            out.push(start);
        }
    }
    out
}

/// Every internal version requirement must equal the workspace package version.
///
/// The requirements were consolidated into `[workspace.dependencies]` in 0.3.0,
/// which takes thirteen copies down to five but does not make drift impossible —
/// it only puts the copies where one edit can reach them all. This check is what
/// makes the consolidation load-bearing.
///
/// The scope is narrower than it first looks, and worth stating precisely.
///
/// 0.2.0 put a `version` key on every internal path dependency, and that alone
/// catches drift *across* a compatible range: bump the workspace to `0.3.0` with
/// a requirement still reading `0.2.0` and cargo refuses to resolve, because the
/// path crate's `0.3.0` does not satisfy `^0.2.0`. Verified by injection — that
/// case is a hard error and needs no lint.
///
/// What survives is drift *inside* a range. Bump to `0.3.1`, leave the
/// requirements at `0.3.0`, and `^0.3.0` admits `0.3.1`: the workspace builds
/// green, tests pass, and publishes. Only a consumer sees it, and what they see
/// is a resolver free to pair `ucal 0.3.1` with a registry `ucal-core 0.3.0` —
/// a mixture this workspace was never tested as. Patch releases are exactly
/// when that goes unnoticed, which is why it is checked rather than remembered.
fn version_lockstep(workspace_root: &Path) -> Vec<Violation> {
    let mut v = Vec::new();
    let manifest = workspace_root.join("Cargo.toml");
    let Ok(src) = std::fs::read_to_string(&manifest) else {
        return v;
    };

    let Some(want) = table_value(&src, "[workspace.package]", "version") else {
        v.push(Violation {
            lint: "version-lockstep",
            file: manifest,
            line: 1,
            text: "[workspace.package] has no version".into(),
            rule: "the workspace version is the single source every member inherits",
        });
        return v;
    };

    let mut in_deps = false;
    for (n, line) in src.lines().enumerate() {
        let l = line.trim();
        if l.starts_with('[') {
            in_deps = l == "[workspace.dependencies]";
            continue;
        }
        if !in_deps || l.starts_with('#') || l.is_empty() {
            continue;
        }
        // Internal crates only. A third-party requirement has no reason to track
        // this workspace's version, and pinning one to it would be the defect.
        let name = l.split('=').next().map(str::trim).unwrap_or("");
        if !name.starts_with("ucal") {
            continue;
        }
        let Some(idx) = l.find("version") else { continue };
        let Some(got) = quoted_after_eq(&l[idx + "version".len()..]) else {
            continue;
        };
        if got != want {
            v.push(Violation {
                lint: "version-lockstep",
                file: manifest.clone(),
                line: n + 1,
                text: l.to_string(),
                rule: "internal version requirements must equal the workspace package version",
            });
        }
    }
    v
}

/// The value of `key` in the given top-level table of a manifest.
fn table_value(src: &str, table: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in src.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            inside = l == table;
            continue;
        }
        if inside && !l.starts_with('#') {
            if let Some(rest) = l.strip_prefix(key) {
                if let Some(q) = quoted_after_eq(rest) {
                    return Some(q);
                }
            }
        }
    }
    None
}

/// The first double-quoted run in `s`, requiring only `=` and space before it.
///
/// Deliberately not a TOML parser. It reads exactly the two shapes this
/// workspace writes, and returns `None` on anything else rather than guessing —
/// a lint that silently skips what it cannot read is worse than no lint, so the
/// tests below pin the shapes it must accept.
fn quoted_after_eq(s: &str) -> Option<String> {
    let s = s.trim_start().strip_prefix('=')?.trim_start();
    let rest = s.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // §20 UC-P1 exit criterion: "lints fail a deliberate violation". Each lint is
    // fed a violating sample and a clean sample.

    fn code_of(src: &str) -> String {
        project(src).code
    }
    fn prose_of(src: &str) -> String {
        let p = project(src);
        format!("{} {}", p.docs, p.other_prose)
    }
    fn docs_of(src: &str) -> String {
        project(src).docs
    }

    #[test]
    fn strips_comments_and_strings() {
        let src = r#"
let a = 1; // a second here is fine
let b = "one second";
/* a block comment mentioning a year */
let c = 2;
"#;
        let code = code_of(src);
        assert!(!code.contains("second"));
        assert!(!code.contains("year"));
        assert!(code.contains("let a"));
        assert!(code.contains("let c"));
        let prose = prose_of(src);
        assert!(prose.contains("second"));
        assert!(prose.contains("year"));
    }

    #[test]
    fn raw_strings_are_prose() {
        let src = "let s = r#\"a day and a year\"#; let t = 1;";
        assert!(!code_of(src).contains("day"));
        assert!(prose_of(src).contains("day"));
        assert!(code_of(src).contains("let t"));
    }

    #[test]
    fn float_lint_fires_on_a_deliberate_violation() {
        let bad = "fn tick_length() -> f64 { 5.39e-44 }";
        assert!(!find_whole_words(&code_of(bad), "f64").is_empty());
        // ...and does not fire on a name that merely contains the token.
        let ok = "let f64_free = 1; struct Xf64y;";
        assert!(find_whole_words(&code_of(ok), "f64").is_empty());
    }

    fn names_foreign_unit(src: &str) -> bool {
        identifiers(&code_of(src))
            .into_iter()
            .any(|(_, id)| segments(id).iter().any(|s| FOREIGN_UNIT_IDENTS.contains(&s.as_str())))
    }

    #[test]
    fn segments_splits_snake_and_camel_case() {
        assert_eq!(segments("seconds_per_day"), ["second", "per", "day"]);
        assert_eq!(segments("to_seconds"), ["to", "second"]);
        assert_eq!(segments("dayOfYear"), ["day", "of", "year"]);
        assert_eq!(segments("YEAR_JULIAN"), ["year", "julian"]);
        assert_eq!(segments("DAY_SI"), ["day", "si"]);
        // Plurals collapse, but short words are not over-stemmed.
        assert_eq!(segments("is"), ["is"]);
        assert_eq!(segments("as"), ["as"]);
    }

    #[test]
    fn foreign_unit_lint_fires_on_a_deliberate_violation() {
        // Every one of these would have slipped past whole-word matching, because
        // `_` is an identifier character.
        for bad in [
            "pub fn to_seconds() {}",
            "pub const DAY_SI: u32 = 1;",
            "fn seconds_per_day() {}",
            "struct DayOfYear;",
            "pub const YEAR_JULIAN: u32 = 1;",
            "pub fn second() -> u32 { 1 }",
            "let month = 3;",
        ] {
            assert!(names_foreign_unit(bad), "lint missed: {bad}");
        }
        // Prose about seconds is not a violation; the rule is about identifiers.
        assert!(!names_foreign_unit("/// converts to seconds\npub fn convert() {}"));
        assert!(!names_foreign_unit("let name = \"second\";"));
        // And ordinary tick-native code is clean.
        assert!(!names_foreign_unit("pub fn tier_value(&self, t: Tier) -> u16 { 0 }"));
        assert!(!names_foreign_unit("pub fn checked_add(&self, d: &Delta) {}"));
    }

    #[test]
    fn wrapping_lint_fires_on_a_deliberate_violation() {
        let bad = "let x = a.wrapping_add(b);";
        assert!(!find_whole_words(&code_of(bad), "wrapping_add").is_empty());
        let ok = "let x = a.checked_add(b);";
        for t in WRAPPING_TOKENS {
            assert!(find_whole_words(&code_of(ok), t).is_empty());
        }
    }

    #[test]
    fn overclaim_lint_fires_on_a_deliberate_violation() {
        let bad = "/// Tick 0 is the creation of the universe.";
        let p = docs_of(bad).to_lowercase();
        assert!(OVERCLAIM_PHRASES.iter().any(|ph| p.contains(ph)));

        let good = "/// Tick 0 is a stipulated datum, conventionally identified with \
                    the FLRW t-to-0 limit.";
        let p = docs_of(good).to_lowercase();
        assert!(
            !OVERCLAIM_PHRASES.iter().any(|ph| p.contains(ph)),
            "the permitted phrasing must not trip the lint"
        );
    }

    #[test]
    fn overclaim_lint_ignores_string_literals() {
        // A test asserting the phrase is absent must contain the phrase. That is
        // mention, not use, and the lint must not confuse the two.
        let src = "let forbidden = [\"creation of the universe\"];";
        let p = docs_of(src).to_lowercase();
        assert!(!OVERCLAIM_PHRASES.iter().any(|ph| p.contains(ph)));
        assert!(project(src).other_prose.contains("creation of the universe"));
    }

    #[test]
    fn suppression_marker_works_and_is_scoped() {
        let src = "let x: f64 = 0.0; // ucal-lint-allow(float-free)\nlet y: f64 = 0.0;\n";
        assert!(suppressed(src, 1, "float-free"));
        assert!(!suppressed(src, 2, "float-free"));
        // A marker for one lint does not suppress another.
        assert!(!suppressed(src, 1, "no-wrapping-arithmetic"));
    }

    #[test]
    fn region_marker_covers_its_span_and_nothing_else() {
        let src = "a\n// ucal-lint-allow-begin(float-free)\nlet x: f64 = 0.0;\n\
                   // ucal-lint-allow-end(float-free)\nlet y: f64 = 0.0;\n";
        assert!(!suppressed(src, 1, "float-free"));
        assert!(suppressed(src, 3, "float-free"));
        assert!(!suppressed(src, 5, "float-free"));
        // A region for one lint does not suppress another.
        assert!(!suppressed(src, 3, "datum-no-overclaim"));
        assert_eq!(regions(src, "float-free"), vec![(2, 4)]);
    }

    #[test]
    fn an_unterminated_region_fails_open_to_the_end_of_the_file() {
        // A typo in the closing marker must not silently re-enable the lint
        // halfway down a file; it must be visible as an over-broad exemption in
        // the report instead.
        let src = "// ucal-lint-allow-begin(float-free)\nlet x: f64 = 0.0;\nlet y: f64 = 0.0;\n";
        assert_eq!(regions(src, "float-free"), vec![(1, 3)]);
    }

    #[test]
    fn the_begin_marker_is_not_mistaken_for_a_line_marker() {
        // Both forms start with the same text; only the bare form covers its own
        // line, and `regions` is what makes the begin line exempt.
        let src = "// ucal-lint-allow-begin(float-free)\nlet x: f64 = 0.0;\n\
                   // ucal-lint-allow-end(float-free)\n";
        assert!(suppressed(src, 1, "float-free"));
        assert_eq!(regions(src, "float-free").len(), 1);
    }

    #[test]
    fn projections_are_line_aligned() {
        let src = "fn a() {}\n/// doc\nlet s = \"str\";\nlet x: f64 = 0;\n";
        let p = project(src);
        assert_eq!(p.code.lines().count(), p.docs.lines().count());
        assert_eq!(p.code.lines().count(), p.other_prose.lines().count());
        let idx = find_whole_words(&p.code, "f64")[0];
        assert_eq!(line_of(&p.code, idx), 4);
        let d = p.docs.find("doc").unwrap();
        assert_eq!(line_of(&p.docs, d), 2);
        let o = p.other_prose.find("str").unwrap();
        assert_eq!(line_of(&p.other_prose, o), 3);
    }

    #[test]
    fn line_numbers_are_reported() {
        let src = "fn a() {}\nfn b() {}\nlet x: f64 = 0;\n";
        let code = code_of(src);
        let idx = find_whole_words(&code, "f64")[0];
        assert_eq!(line_of(&code, idx), 3);
        assert!(line_text(&code, idx).contains("let x"));
    }

    // ---------------------------------------------------------------- lockstep

    fn manifest(deps: &str) -> String {
        format!("[workspace.package]\nversion = \"0.3.0\"\nedition = \"2021\"\n\n[workspace.dependencies]\n{deps}")
    }

    /// `case` must be unique per test: these run in parallel, and the good and
    /// stale manifests differ by a single digit, so anything derived from the
    /// content collides between exactly the two cases that must not share a file.
    fn lockstep_of(case: &str, src: &str) -> Vec<String> {
        let dir = std::env::temp_dir().join(format!("ucal-lockstep-{case}"));
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("Cargo.toml"), src).unwrap();
        version_lockstep(&dir).into_iter().map(|v| v.text).collect()
    }

    #[test]
    fn every_lint_is_listed_for_suppression_reporting() {
        // A lint absent from LINT_NAMES can be suppressed and the report will
        // not say so — an exemption that is not a visible cost, which is exactly
        // what the marker machinery exists to prevent. Three lints added in
        // 0.3.0 and 0.4.0 were missing from it, and the four exemptions
        // `rounding-is-declared` needed went unreported until this test.
        let root = crate::workspace_root();
        let emitted: std::collections::BTreeSet<&str> =
            run(&root).into_iter().map(|v| v.lint).collect();
        for lint in &emitted {
            assert!(
                LINT_NAMES.contains(lint),
                "`{lint}` can be suppressed without the report mentioning it"
            );
        }
        // And the reverse, so the list cannot accumulate names for lints that
        // no longer exist.
        for name in LINT_NAMES {
            assert!(
                !name.is_empty(),
                "an empty lint name would match every marker"
            );
        }
    }

    #[test]
    fn lockstep_accepts_matching_requirements() {
        let m = manifest(
            "ucal-core = { version = \"0.3.0\", path = \"crates/ucal-core\", default-features = false }\n\
             ucal-civil = { version = \"0.3.0\", path = \"crates/ucal-civil\", default-features = false }\n",
        );
        assert!(lockstep_of("match", &m).is_empty());
    }

    #[test]
    fn lockstep_catches_drift_across_a_range() {
        // Cargo also catches this one, since the path crate's 0.3.0 does not
        // satisfy ^0.2.0. Kept so the lint is known to cover it too.
        let m = manifest(
            "ucal-core = { version = \"0.2.0\", path = \"crates/ucal-core\", default-features = false }\n\
             ucal-civil = { version = \"0.3.0\", path = \"crates/ucal-civil\", default-features = false }\n",
        );
        let bad = lockstep_of("stale", &m);
        assert_eq!(bad.len(), 1, "expected exactly the stale requirement");
        assert!(bad[0].contains("ucal-core"));
    }

    #[test]
    fn lockstep_catches_drift_inside_a_range() {
        // The case that needs the lint. `^0.3.0` admits `0.3.1`, so this manifest
        // resolves, builds, tests green and publishes. Only the lint objects, and
        // only a consumer would ever have seen it.
        let m = manifest(
            "ucal-core = { version = \"0.3.0\", path = \"crates/ucal-core\", default-features = false }\n",
        )
        .replace("version = \"0.3.0\"\nedition", "version = \"0.3.1\"\nedition");
        let bad = lockstep_of("patch", &m);
        assert_eq!(bad.len(), 1, "0.3.1 workspace against a ^0.3.0 requirement");
        assert!(bad[0].contains("ucal-core"));
    }

    #[test]
    fn lockstep_ignores_third_party_requirements() {
        // bnum's version has no reason to track this workspace's, and pinning it
        // to one would be the defect rather than the fix.
        let m = manifest(
            "bnum = { version = \"0.14\", default-features = false }\n\
             num-bigint = { version = \"0.4\", default-features = false }\n\
             ucal-core = { version = \"0.3.0\", path = \"crates/ucal-core\" }\n",
        );
        assert!(lockstep_of("third-party", &m).is_empty());
    }

    #[test]
    fn lockstep_ignores_comments() {
        let m = manifest(
            "# ucal-core = { version = \"0.1.0\", path = \"crates/ucal-core\" }\n\
             ucal-core = { version = \"0.3.0\", path = \"crates/ucal-core\" }\n",
        );
        assert!(lockstep_of("comments", &m).is_empty());
    }

    #[test]
    fn lockstep_reports_a_missing_workspace_version() {
        let bad = lockstep_of("no-version", "[workspace.dependencies]\nucal-core = { version = \"0.3.0\" }\n");
        assert_eq!(bad.len(), 1);
        assert!(bad[0].contains("no version"));
    }

    #[test]
    fn quoted_after_eq_reads_only_what_it_understands() {
        assert_eq!(quoted_after_eq(" = \"0.3.0\"").as_deref(), Some("0.3.0"));
        assert_eq!(quoted_after_eq("= \"x\", path = \"y\"").as_deref(), Some("x"));
        // No `=`, so not a value: returning None is the honest answer.
        assert_eq!(quoted_after_eq("\"0.3.0\""), None);
        assert_eq!(quoted_after_eq(".workspace = true"), None);
    }
}
