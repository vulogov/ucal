//! X1 — the defect corpus: one recorded mutation per check.
//!
//! # Why this exists
//!
//! 1.6.0's audit established that every check in this repository **reads its
//! subject** — none of them can report success having examined nothing. It
//! closed by naming the property it had not established:
//!
//! > Whether a check is *correct* — whether it would catch the defect it exists
//! > for — is not answerable by pointing it at an empty directory.
//!
//! Six checks had been verified strict by hand: inject the defect, watch it
//! fire, revert. That works, and it is a **procedure**. It happens when someone
//! remembers, on whichever check they are editing, and the evidence is gone when
//! the terminal closes. This project's whole argument is that a procedure is not
//! a mechanism, and its own verification habits are not exempt.
//!
//! # How a mutation is run
//!
//! Every check takes a `root`, which is what makes this possible at all: the
//! corpus builds a **sandbox** — a copy of the files the checks read, and
//! nothing else — mutates one file in it, and calls the check against that root.
//! The real tree is never edited, so an interrupted run cannot leave the
//! repository in a mutated state.
//!
//! A mutation whose check does **not** reject it is a **survivor**, and a
//! survivor is the whole point of the exercise: it means the check passes on a
//! tree that is wrong in exactly the way the check exists to prevent.
//!
//! # What stops this file from being a defect
//!
//! `citations::check` scans `xtask/src`, so a section marker written literally
//! *here* is a citation the project must keep resolving. The first version of
//! the citation mutation carried a literal `\u{a7}99.9` and made `check-docs`
//! fail on the real repository — caught by that check, doing exactly its job on
//! the file that was about to test it.
//!
//! Markers in `replace:` strings are therefore written as escapes. Markers in
//! prose, like the `\u{a7}19` in a `what:` field below, are written literally
//! and resolve, which is the correct outcome and not an exception: a guard
//! against *dangling* markers here would duplicate the check this file exists to
//! exercise.
//!
//! # What is not here
//!
//! The UC-P0 constants harness. Its checks are hardcoded calls rather than a
//! reading of the tree, so a mutation to it is a source edit to `xtask` itself
//! and not a reversible edit to a data file. X1's stop condition anticipated
//! this: such checks are listed as hand-verified rather than pretended into the
//! corpus. See `Documentation/Proposals/X1-defect-corpus.md`.

use std::path::Path;

/// A defect that a named check must reject.
pub struct Mutation {
    /// Which check must produce a failure. Dispatched by [`run_check`].
    pub check: &'static str,
    /// One line: the defect this introduces, for the report.
    pub what: &'static str,
    /// Repo-relative path of the file to edit.
    ///
    /// `{VERSION}` is replaced with the workspace version, so an entry naming
    /// this cycle's release notes does not go stale at the next bump.
    pub file: &'static str,
    /// Exact text to find, with `{VERSION}` substituted as in [`Mutation::file`].
    ///
    /// Must occur at least once, or the mutation is
    /// **unapplied** and reported as such — a corpus entry that silently edits
    /// nothing would be a check on nothing, which is the defect this file exists
    /// to find in other people's work.
    pub find: &'static str,
    /// What to replace it with.
    pub replace: &'static str,
}

/// What happened to one mutation.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The check rejected the mutated tree. What should happen.
    Caught,
    /// The check accepted it. The interesting output.
    Survived,
    /// `find` did not occur, so nothing was mutated and nothing was tested.
    Unapplied,
    /// The check reported failure on the *unmutated* sandbox, so its verdict on
    /// the mutation means nothing.
    SandboxAlreadyFailing,
}

/// The corpus.
///
/// One entry per check, deliberately: a second mutation of the same check is a
/// better test of that check and a worse map of which checks are tested at all,
/// and the map is what 1.6.0 showed was missing.
///
/// The exception is a check with two distinct **paths**, which is a different
/// thing from two mutations of one path. `version-lockstep` reads the root
/// manifest's dependency table and each member's `[package]` section, and a
/// corpus covering one of those would report a check as tested while half of it
/// was not — the shape of claim this whole exercise exists to refuse.
pub const MUTATIONS: &[Mutation] = &[
    // ---- check-docs ------------------------------------------------------
    Mutation {
        check: "citations",
        what: "a citation in source pointing at a section that does not exist",
        file: "crates/ucal-core/src/tier.rs",
        find: "/// Digits per tier.",
        // The section marker is written as an escape on purpose. Spelled
        // literally, this line *is* a dangling citation in a file the citation
        // check reads, and the first run of this corpus made `check-docs` fail
        // on the real repository. A corpus that plants its own defects in the
        // tree under test is worse than no corpus.
        replace: "/// See \u{a7}99.9.\n/// Digits per tier.",
    },
    Mutation {
        // X1 recorded this as the corpus's one survivor: `cited()` walked
        // `crates/**/*.rs` and `xtask/src/**/*.rs` and read no Markdown at all,
        // so a dangling reference in the documentation was invisible to a check
        // announced as "citations resolve against spec/". X2 widened the scan
        // and this entry is what holds it widened.
        check: "citations",
        what: "a dangling citation in the documentation, which is scanned since 1.7.0",
        file: "Documentation/CLI.md",
        find: "## `ucal doctor`",
        replace: "## `ucal doctor`\n\nSee \u{a7}99.9 for the details.",
    },
    Mutation {
        check: "cli-docs",
        what: "a command the program has and the manual does not document",
        file: "Documentation/CLI.md",
        find: "## `ucal doctor`",
        replace: "## `ucal doctor-renamed`",
    },
    Mutation {
        check: "deltas-applied",
        what: "a standing delta dropped from the normative text",
        file: "spec/UCAL-1.1.md",
        find: "> **[D-A24 · CORRECTION]**",
        replace: "> **[D-A24-not-applied · CORRECTION]**",
    },
    Mutation {
        check: "signing-key",
        what: "one published copy of the signing key altered",
        file: "Documentation/CONTACT.md",
        find: "RWTMVJ5DqeXk0HgeN+BIdnQaamRTdzkjITkdprOPLVsGWP8R/2HYIj0r",
        replace: "RWTMVJ5DqeXk0HgeN+BIdnQaamRTdzkjITkdprOPLVsGWP8R/2HYIj0X",
    },
    Mutation {
        check: "ci-covers-procedure",
        what: "a verification command the release notes claim and CI does not run",
        file: "Documentation/Release_Notes/{VERSION}.md",
        find: "cargo test --workspace --release",
        replace: "cargo test --workspace --release\ncargo run -p xtask -- invented-check",
    },
    Mutation {
        check: "generated-docs",
        what: "a stale generated tier table",
        file: "docs/TIERS.md",
        find: "# The tier grid",
        replace: "# The tier grid, edited by hand",
    },
    Mutation {
        check: "schema",
        what: "a stale ucal-json/1 schema",
        file: "fixtures/ucal-json-1.schema.json",
        find: "\"ticks\"",
        replace: "\"ticks_renamed\"",
    },
    Mutation {
        // X1 listed this check as hand-verified because it read `workspace_root()`
        // and could not be pointed at a sandbox. B6 gave it a root parameter, and
        // this is the mutation it existed to be given: the committed digest no
        // longer matches what the two integer routes re-derive.
        check: "verify-vectors",
        what: "a conformance digest that the two routes do not re-derive",
        file: "fixtures/SHA256SUMS",
        find: "1f99cf62",
        replace: "1f99cf63",
    },
    // ---- lints -----------------------------------------------------------
    Mutation {
        check: "lint:float-free",
        what: "a float literal in a shipped crate (Rule E)",
        file: "crates/ucal-core/src/tier.rs",
        find: "pub fn all_descending()",
        replace: "pub fn injected_float() -> f64 {\n        1.5\n    }\n\n    pub fn all_descending()",
    },
    Mutation {
        check: "lint:no-wrapping-arithmetic",
        what: "saturating arithmetic on a shipped path (Rule O)",
        file: "crates/ucal-core/src/tier.rs",
        find: "pub fn all_descending()",
        replace: "pub fn injected_saturating(a: u32) -> u32 {\n        a.saturating_sub(1)\n    }\n\n    pub fn all_descending()",
    },
    Mutation {
        check: "lint:no-panic-in-cli",
        what: "an unwrap in the binary, which promises none (§19)",
        file: "crates/ucal/src/style.rs",
        find: "pub fn is_plain(&self) -> bool {",
        replace: "pub fn injected_panic(v: Option<u8>) -> u8 {\n        v.unwrap()\n    }\n\n    pub fn is_plain(&self) -> bool {",
    },
    Mutation {
        check: "lint:codes-have-raisers",
        what: "a diagnostic code declared and raised nowhere",
        file: "crates/ucal-core/src/error.rs",
        find: "    E0018,\n}",
        replace: "    E0018,\n\n    /// Injected by the defect corpus.\n    E0019,\n}",
    },
    Mutation {
        check: "lint:public-type-is-classified",
        what: "a public type that is neither non_exhaustive nor a closed vocabulary",
        // At column zero: the lint reads `pub struct ` as a line prefix, and
        // the first attempt injected it indented inside an `impl`, where it
        // matched nothing. The mutation was wrong, not the lint.
        file: "crates/ucal-core/src/tier.rs",
        find: "/// Digits per tier.",
        replace: "pub struct InjectedOpenType {\n    pub field: u8,\n}\n\n/// Digits per tier.",
    },
    Mutation {
        check: "lint:core-names-no-foreign-unit",
        what: "an identifier in ucal-core naming an Earth unit (Rules A.2/Y)",
        file: "crates/ucal-core/src/tier.rs",
        find: "/// Digits per tier.",
        replace: "pub fn injected_seconds_per_beat() -> u64 {\n    0\n}\n\n/// Digits per tier.",
    },
    Mutation {
        check: "lint:datum-no-overclaim",
        what: "prose claiming tick 0 is the creation of the universe (Rule Q.1)",
        file: "crates/ucal-core/src/tier.rs",
        find: "/// Digits per tier.",
        replace: "/// Tick 0 is the creation of the universe.\n/// Digits per tier.",
    },
    Mutation {
        check: "lint:rounding-is-declared",
        what: "a rounding in a shipped crate that does not name the caller's mode",
        file: "crates/ucal-core/src/tier.rs",
        find: "/// Digits per tier.",
        replace: "pub fn injected_rounding(r: &crate::Ratio) -> String {\n    r.to_decimal_string(6, crate::Rounding::HalfEven).unwrap_or_default()\n}\n\n/// Digits per tier.",
    },
    Mutation {
        check: "lint:no-indent-in-literal",
        what: "an indented continuation inside a string literal, which prints as ragged output",
        file: "crates/ucal-core/src/tier.rs",
        find: "/// Digits per tier.",
        replace: "pub const INJECTED_LITERAL: &str = \"a line\n        and an indented continuation\";\n\n/// Digits per tier.",
    },
    Mutation {
        check: "contact-constants",
        what: "a constant in the contact materials that no longer matches the vectors",
        file: "Documentation/CONTACT.md",
        find: "8070204002895596515944343085635637180530466139316558837890625",
        replace: "8070204002895596515944343085635637180530466139316558837890626",
    },
    Mutation {
        check: "worked-examples",
        what: "a committed example that is not what the program prints",
        file: "Documentation/CLI-EXAMPLES.md",
        find: "# Worked examples",
        replace: "# Worked examples (edited by hand)",
    },
    Mutation {
        check: "lint:version-lockstep",
        what: "an internal dependency pinned to the previous version",
        // The root manifest, because that is the only file this lint reads. The
        // first attempt edited `crates/ucal-core/Cargo.toml` and survived, which
        // is not a defect in the mutation: `version-lockstep` does not look at
        // member manifests at all. See the survivor note in
        // `Documentation/Proposals/X1-defect-corpus.md`.
        file: "Cargo.toml",
        find: "ucal-core = { version = \"{VERSION}\"",
        replace: "ucal-core = { version = \"0.0.1\"",
    },
    Mutation {
        // The second path, added when X2 widened this lint to member manifests.
        //
        // Note what it pins: the **current** version, not a stale one. A stale
        // pin is caught by cargo before any tool runs — `ucal-civil` requires
        // 1.7.0 and resolution fails — so it is not the interesting case. A
        // member pinned to the version the workspace is already on builds
        // perfectly and drifts silently at the next bump, and this lint is the
        // only thing that sees it.
        check: "lint:version-lockstep",
        what: "a member manifest that stops inheriting the workspace version",
        file: "crates/ucal-core/Cargo.toml",
        find: "version.workspace = true",
        replace: "version = \"{VERSION}\"",
    },
];

/// Fill `{VERSION}` in a mutation's paths and patterns.
///
/// Three entries name the version — the current release notes, and the two
/// halves of `version-lockstep`. Hardcoding it meant the corpus went stale at
/// every bump, which it duly did on the first one after it was written: three
/// entries reported `UNAPP` on the 1.8.0 branch and tested nothing.
///
/// The `UNAPP` verdict caught it, which is the design working. Not needing to be
/// caught is better than being caught, and a mechanism requiring a hand edit
/// every release is one that eventually gets edited wrong.
pub fn subst(s: &str) -> String {
    s.replace("{VERSION}", env!("CARGO_PKG_VERSION"))
}

/// Copy the parts of the tree the checks read into a scratch directory.
///
/// Not the whole repository: `target/` alone is gigabytes, and a corpus that
/// took a minute to set up would be run once.
pub fn sandbox(root: &Path, into: &Path) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(into);
    std::fs::create_dir_all(into).map_err(|e| e.to_string())?;
    for rel in [
        "spec",
        "docs",
        "Documentation",
        "fixtures",
        ".github",
        "crates",
        "xtask",
    ] {
        copy_tree(&root.join(rel), &into.join(rel))?;
    }
    // The worked-examples check runs the built binary. Without it the check
    // skips, and a skip would read as "the mutation survived".
    let bin = root.join("target/release/ucal");
    if bin.exists() {
        let dst = into.join("target/release");
        std::fs::create_dir_all(&dst).map_err(|e| e.to_string())?;
        std::fs::copy(&bin, dst.join("ucal")).map_err(|e| format!("binary: {e}"))?;
    }
    for f in ["README.md", "Cargo.toml"] {
        let (from, to) = (root.join(f), into.join(f));
        if from.exists() {
            std::fs::copy(&from, &to).map_err(|e| format!("{f}: {e}"))?;
        }
    }
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    if !from.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(from).map_err(|e| e.to_string())?.flatten() {
        let name = entry.file_name();
        // `target/` is where the size is, and no check reads it.
        if name == "target" {
            continue;
        }
        let (src, dst) = (entry.path(), to.join(&name));
        if src.is_dir() {
            copy_tree(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst).map_err(|e| format!("{}: {e}", src.display()))?;
        }
    }
    Ok(())
}

/// Run one named check against a root. `true` means it **passed**.
///
/// The corpus asserts the negation: a mutated tree must not pass.
pub fn run_check(name: &str, root: &Path) -> bool {
    if let Some(lint) = name.strip_prefix("lint:") {
        let (violations, scanned) = crate::lint::run(root);
        // A sandbox that read no files would make every lint mutation look like
        // a survivor, which is the failure mode V1 found in the lints
        // themselves. Reported as a sandbox problem rather than a verdict.
        if scanned == 0 {
            return true;
        }
        return !violations.iter().any(|v| v.lint == lint);
    }
    match name {
        "citations" => crate::citations::check(root).is_ok(),
        "cli-docs" => crate::citations::check_cli_docs(root).is_ok(),
        "deltas-applied" => crate::citations::check_deltas_are_applied(root).is_ok(),
        "signing-key" => crate::citations::check_signing_key(root).is_ok(),
        "ci-covers-procedure" => crate::citations::check_ci_covers_the_procedure(root).is_ok(),
        "contact-constants" => crate::citations::check_contact_constants(root).is_ok(),
        "generated-docs" => crate::gendocs::check(root).is_ok(),
        // `Ok(None)` is the skip: no binary, nothing checked. Treated as a pass
        // here so a sandbox without one shows up as a survivor rather than as a
        // silent success — the same distinction V2 drew for CI.
        "worked-examples" => !matches!(crate::examples::check(root), Err(_)),
        // Exit 0 means the digest matched. The check prints as it goes, which is
        // noisy inside a corpus run and is the check doing its job.
        "verify-vectors" => crate::run_verify_vectors_at(root) == 0,
        "schema" => crate::schema::check(root).is_ok(),
        other => panic!("the corpus names a check that does not exist: `{other}`"),
    }
}

/// Apply every mutation in turn and report what each check did.
pub fn run(root: &Path, into: &Path) -> Result<Vec<(&'static Mutation, Verdict)>, String> {
    sandbox(root, into)?;
    let mut out = Vec::new();
    for m in MUTATIONS {
        let path = into.join(subst(m.file));
        let Ok(original) = std::fs::read_to_string(&path) else {
            out.push((m, Verdict::Unapplied));
            continue;
        };
        if !original.contains(&subst(m.find)) {
            out.push((m, Verdict::Unapplied));
            continue;
        }
        // The check must be happy *before* the mutation, or its verdict after
        // one says nothing about the mutation.
        if !run_check(m.check, into) {
            out.push((m, Verdict::SandboxAlreadyFailing));
            continue;
        }
        std::fs::write(
            &path,
            original.replacen(&subst(m.find), &subst(m.replace), 1),
        )
        .map_err(|e| e.to_string())?;
        let passed = run_check(m.check, into);
        std::fs::write(&path, &original).map_err(|e| e.to_string())?;
        out.push((
            m,
            if passed {
                Verdict::Survived
            } else {
                Verdict::Caught
            },
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod diagnose {
    use super::*;

    /// Why is a check unhappy on an unmutated sandbox? Prints, asserts nothing.
    ///
    /// `DIRTY` says a check was already failing before the mutation, which makes
    /// its verdict meaningless — but not *why*. This is the "why", and it is
    /// `#[ignore]`d because it is a debugging aid rather than a check.
    ///
    /// Run it with:
    /// `cargo test -p xtask sandbox_baseline -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn sandbox_baseline() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_path_buf();
        let into = std::env::temp_dir().join("ucal-corpus-diagnose");
        sandbox(&root, &into).expect("sandbox");
        match crate::examples::check(&into) {
            Ok(Some(n)) => println!("worked-examples ok, {n}"),
            Ok(None) => println!("worked-examples SKIPPED (no binary in sandbox)"),
            Err(e) => println!("worked-examples FAIL: {}", &e.chars().take(400).collect::<String>()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every mutation names a check that exists.
    ///
    /// `run_check` panics on an unknown name, which would surface as a confusing
    /// failure in the middle of a run. This turns it into a clear one.
    #[test]
    fn every_mutation_names_a_real_check() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_path_buf();
        for m in MUTATIONS {
            // A check that does not exist panics inside `run_check`; a check that
            // does exist returns some verdict about the real tree, which is not
            // what is being asserted here.
            let _ = run_check(m.check, &root);
        }
    }

    /// Every mutation edits a file that exists and contains what it searches for.
    ///
    /// The `UNAPP` verdict catches this at run time. This catches it at test
    /// time, which is where a corpus entry that has drifted from the tree should
    /// be noticed — one of the twenty was written against text that had never
    /// existed, and reported `UNAPP` for two runs before it was read.
    #[test]
    fn every_mutation_finds_its_anchor() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_path_buf();
        for m in MUTATIONS {
            let path = root.join(subst(m.file));
            let src = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} names {}: {e}", m.check, subst(m.file)));
            assert!(
                src.contains(&subst(m.find)),
                "{}: `{}` does not contain the anchor `{}`",
                m.check,
                m.file,
                m.find.lines().next().unwrap_or(m.find)
            );
        }
    }

}

#[cfg(test)]
mod probe_markdown {
    /// How many citations does the documentation carry, and how many resolve?
    ///
    /// X1's survivor is that `cited()` reads no Markdown. Before widening it,
    /// this measures what widening would surface: a check that lands with two
    /// hundred failures is a check nobody will turn on.
    #[test]
    #[ignore]
    fn what_would_widening_the_scan_surface() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_path_buf();
        let mut files = 0usize;
        let mut cites = 0usize;
        let mut per: std::collections::BTreeMap<(&str, String), usize> = Default::default();
        let mut stack = vec![root.join("Documentation"), root.join("spec"), root.join("docs")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if !p.extension().is_some_and(|x| x == "md") {
                    continue;
                }
                let Ok(src) = std::fs::read_to_string(&p) else { continue };
                files += 1;
                for (kind, c) in crate::citations::scan(&src) {
                    cites += 1;
                    *per.entry((kind, c)).or_default() += 1;
                }
            }
        }
        println!("markdown files scanned: {files}");
        println!("distinct citations:     {}", per.len());
        println!("citation sites:         {cites}");
    }
}
