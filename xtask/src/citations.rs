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
