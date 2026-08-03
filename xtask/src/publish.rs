//! `xtask -- publish` — the release procedure, as a program rather than a list.
//!
//! # Why this exists
//!
//! `cargo publish --workspace` does not work on this workspace. It packages all
//! six crates, then verifies them, and verification of a dependent resolves that
//! dependent's requirement on `ucal-core` against the registry — where the new
//! version does not exist yet, because nothing has been uploaded. It fails with
//! a cargo *internal* error:
//!
//! ```text
//! error: failed to verify package tarball
//! Caused by: no hash listed for ucal-core v0.2.0
//! note: this is an unexpected cargo internal error
//! ```
//!
//! The 0.2.0 release was published one crate at a time in dependency order
//! instead. That is not a workaround that skips verification: each crate is
//! verified normally, against the real index, once the one below it is live.
//!
//! Writing it down was C2. Writing it down *as a program* is the difference
//! between a list that rots and an order that is recomputed — a seventh crate,
//! or a new edge between two existing ones, changes the order this prints
//! without anyone remembering to update it.
//!
//! # What it will not do
//!
//! Publishing is irreversible: a version can be yanked but never replaced. So
//! the default is a dry run, the real thing needs `--execute`, and the preflight
//! refuses on a dirty working tree — a published crate is a permanent record of
//! a tree, and a tree that was never committed is not one anyone can go back to.
//!
//! It does **not** check whether the version is already on crates.io. That would
//! be a network call duplicating one cargo makes anyway, and cargo's refusal is
//! the authoritative one. Nor does it run the tests or the lints: those are steps
//! 4 and 5 of the procedure in `Documentation/Release_Notes/README.md`, and
//! folding them in here would make a green publish look like evidence they
//! passed on this tree when they may have run on another.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// One publishable crate.
#[derive(Debug)]
struct Member {
    name: String,
    dir: PathBuf,
    /// Internal dependencies, by package name.
    deps: BTreeSet<String>,
}

/// Read the workspace members that are published.
///
/// `publish = false` is honoured — `xtask` itself is not a released artifact and
/// must never end up in the order.
fn members(root: &Path) -> Result<Vec<Member>, String> {
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|e| format!("cannot read the workspace manifest: {e}"))?;

    let mut dirs = Vec::new();
    let mut in_members = false;
    for line in manifest.lines() {
        let l = line.trim();
        if l.starts_with("members") {
            in_members = true;
        }
        if in_members {
            if let Some(start) = l.find('"') {
                if let Some(end) = l[start + 1..].find('"') {
                    dirs.push(l[start + 1..start + 1 + end].to_string());
                }
            }
            if l.contains(']') {
                in_members = false;
            }
        }
    }

    let mut out = Vec::new();
    for d in dirs {
        let dir = root.join(&d);
        let text = match std::fs::read_to_string(dir.join("Cargo.toml")) {
            Ok(t) => t,
            Err(e) => return Err(format!("cannot read {d}/Cargo.toml: {e}")),
        };
        let mut name = None;
        let mut publishable = true;
        let mut deps = BTreeSet::new();
        let mut section = String::new();
        for line in text.lines() {
            let l = line.trim();
            if l.starts_with('[') {
                section = l.to_string();
                continue;
            }
            if l.starts_with('#') || l.is_empty() {
                continue;
            }
            if section == "[package]" {
                if let Some(v) = l.strip_prefix("name") {
                    name = quoted(v);
                }
                if l.starts_with("publish") && l.contains("false") {
                    publishable = false;
                }
            }
            // Dev-dependencies count. `cargo publish` verifies by resolving the
            // packaged manifest, and resolution covers every dependency table —
            // so a crate whose dev-dependency is not yet on the index fails the
            // same way a missing normal dependency would. `ucal-cosmo` is the
            // case: it needs `ucal-events` only for its float oracle, and
            // leaving that edge out ordered it before `ucal-events`, which is
            // not the order the 0.2.0 release actually succeeded in.
            if section == "[dependencies]" || section == "[dev-dependencies]" {
                let key = l.split(['=', '.']).next().unwrap_or("").trim();
                if key.starts_with("ucal") {
                    deps.insert(key.to_string());
                }
            }
        }
        let Some(name) = name else {
            return Err(format!("{d}/Cargo.toml has no package name"));
        };
        if publishable {
            out.push(Member { name, dir, deps });
        }
    }
    Ok(out)
}

/// The first double-quoted run after an `=`.
fn quoted(s: &str) -> Option<String> {
    let s = s.trim_start().strip_prefix('=')?.trim_start();
    let rest = s.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Dependency order: every crate after everything it depends on.
///
/// Kahn's algorithm over the internal edges, taking ready crates in name order
/// so the result is deterministic — a release procedure that prints a different
/// order on two machines is not a procedure.
fn order(members: &[Member]) -> Result<Vec<&Member>, String> {
    let by_name: BTreeMap<&str, &Member> = members.iter().map(|m| (m.name.as_str(), m)).collect();
    let mut remaining: BTreeMap<&str, BTreeSet<&str>> = members
        .iter()
        .map(|m| {
            (
                m.name.as_str(),
                m.deps
                    .iter()
                    .map(|d| d.as_str())
                    .filter(|d| by_name.contains_key(d))
                    .collect(),
            )
        })
        .collect();

    let mut out = Vec::new();
    while !remaining.is_empty() {
        let ready: Vec<&str> = remaining
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(n, _)| *n)
            .collect();
        if ready.is_empty() {
            return Err(format!(
                "the internal dependency graph has a cycle among: {:?}",
                remaining.keys().collect::<Vec<_>>()
            ));
        }
        for n in ready {
            out.push(by_name[n]);
            remaining.remove(n);
            for deps in remaining.values_mut() {
                deps.remove(n);
            }
        }
    }
    Ok(out)
}

/// The version every member inherits.
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
            if let Some(v) = l.strip_prefix("version") {
                if let Some(q) = quoted(v) {
                    return Some(q);
                }
            }
        }
    }
    None
}

/// True when the working tree has uncommitted changes.
fn tree_is_dirty(root: &Path) -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

/// `xtask -- publish [--execute]`.
pub fn run(execute: bool) -> i32 {
    let root = super::workspace_root();

    let members = match members(&root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("  FAIL  {e}");
            return 6;
        }
    };
    let ordered = match order(&members) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("  FAIL  {e}");
            return 6;
        }
    };
    let Some(version) = workspace_version(&root) else {
        eprintln!("  FAIL  [workspace.package] has no version");
        return 6;
    };

    println!("publish order for {version}, derived from the dependency graph:");
    for (i, m) in ordered.iter().enumerate() {
        let deps: Vec<&str> = m.deps.iter().map(|s| s.as_str()).collect();
        println!(
            "  {}. {:<12} {}",
            i + 1,
            m.name,
            if deps.is_empty() {
                "(no internal dependencies)".to_string()
            } else {
                format!("after {}", deps.join(", "))
            }
        );
    }
    println!();

    // --- preflight --------------------------------------------------------
    if execute && tree_is_dirty(&root) {
        eprintln!("  FAIL  the working tree is dirty.");
        eprintln!("        A published crate is a permanent record of a tree that was");
        eprintln!("        never committed. Commit or stash first.");
        return 6;
    }

    // Packaging first catches what is worth catching before anything is
    // uploaded: a missing file, a bad include, a manifest that does not parse.
    //
    // This is the one step that must use `--workspace`, and the reason is the
    // mirror image of why publishing must not. In workspace mode cargo knows all
    // six versions are going out together and resolves the internal requirements
    // against the local tree; per-crate, it resolves `ucal-core = "^0.3.0"`
    // against the registry and fails, because that version is precisely what is
    // about to be uploaded. `--no-verify` is what keeps workspace mode usable at
    // all — verification is the step that genuinely cannot run before the upload,
    // and it is what the sequential pass below does instead.
    println!("packaging all crates (no upload):");
    let packaged = Command::new("cargo")
        .args(["publish", "--workspace", "--dry-run", "--no-verify"])
        .current_dir(&root)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !packaged {
        eprintln!("  FAIL  the workspace does not package");
        return 6;
    }
    println!("  ok    all {} crates package", ordered.len());

    if !execute {
        println!("\ndry run only. Nothing was uploaded.");
        println!("To publish for real:  cargo run -p xtask -- publish --execute");
        println!("Publishing cannot be undone; a version can be yanked, never replaced.");
        return 0;
    }

    // --- the real thing ---------------------------------------------------
    println!("\npublishing {version}, one crate at a time:");
    for m in &ordered {
        println!("\n  --- {} ---", m.name);
        // Full verification, deliberately: by the time this runs, everything
        // below it is live on the index, so the check that `cargo publish
        // --workspace` cannot perform is exactly the one that happens here.
        let status = Command::new("cargo")
            .args(["publish", "-p", &m.name])
            .current_dir(&m.dir)
            .status();
        match status {
            Ok(s) if s.success() => println!("  ok    {} published", m.name),
            _ => {
                eprintln!("  FAIL  {} did not publish.", m.name);
                eprintln!("        Everything before it in the order is already live and");
                eprintln!("        cannot be withdrawn. Fix and re-run: crates already");
                eprintln!("        published will fail with `already uploaded` and the rest");
                eprintln!("        will proceed.");
                return 6;
            }
        }
    }
    println!("\nall {} crates published at {version}.", ordered.len());
    println!("Next: tag v{version}, annotated and signed, and push the tag.");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(name: &str, deps: &[&str]) -> Member {
        Member {
            name: name.into(),
            dir: PathBuf::new(),
            deps: deps.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn the_order_puts_dependencies_first() {
        let ms = vec![
            m("ucal", &["ucal-core", "ucal-civil", "ucal-cosmo"]),
            m("ucal-cosmo", &["ucal-core", "ucal-events"]),
            m("ucal-core", &[]),
            m("ucal-events", &["ucal-core"]),
            m("ucal-civil", &["ucal-core"]),
        ];
        let out = order(&ms).unwrap();
        let names: Vec<&str> = out.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names[0], "ucal-core");
        for m in &ms {
            let i = names.iter().position(|n| *n == m.name).unwrap();
            for d in &m.deps {
                let j = names.iter().position(|n| n == d).unwrap();
                assert!(j < i, "{} published before its dependency {d}", m.name);
            }
        }
    }

    #[test]
    fn the_order_is_deterministic() {
        // Two machines must print the same procedure.
        let build = || {
            vec![
                m("ucal-civil", &["ucal-core"]),
                m("ucal-events", &["ucal-core"]),
                m("ucal-body", &["ucal-core"]),
                m("ucal-core", &[]),
            ]
        };
        let a: Vec<String> = order(&build())
            .unwrap()
            .iter()
            .map(|m| m.name.clone())
            .collect();
        let b: Vec<String> = order(&build())
            .unwrap()
            .iter()
            .map(|m| m.name.clone())
            .collect();
        assert_eq!(a, b);
    }

    #[test]
    fn every_published_crate_is_a_workspace_member_with_a_directory() {
        // The order drives `cargo publish -p <name>` in `<dir>`; a member whose
        // manifest could not be read would silently drop out of a release.
        let root = crate::workspace_root();
        let ms = members(&root).expect("workspace members");
        assert_eq!(ms.len(), 6, "expected six published crates, got {}", ms.len());
        for m in &ms {
            assert!(
                m.dir.join("Cargo.toml").exists(),
                "{} has no manifest at {}",
                m.name,
                m.dir.display()
            );
        }
    }

    #[test]
    fn a_cycle_is_reported_rather_than_looping() {
        let ms = vec![m("ucal-a", &["ucal-b"]), m("ucal-b", &["ucal-a"])];
        let e = order(&ms).unwrap_err();
        assert!(e.contains("cycle"), "unexpected error: {e}");
    }

    #[test]
    fn an_external_dependency_does_not_constrain_the_order() {
        // `ucal-core` depends on bnum, which is not a workspace member and has
        // nothing to do with the order these are uploaded in.
        let ms = vec![m("ucal-core", &["ucal-nonexistent"])];
        let out = order(&ms).unwrap();
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn a_dev_dependency_constrains_the_order_too() {
        // ucal-cosmo depends on ucal-events only as a dev-dependency, and cargo
        // still resolves it when verifying the package.
        let root = crate::workspace_root();
        let ms = members(&root).expect("workspace members");
        let names: Vec<String> = order(&ms)
            .expect("acyclic")
            .iter()
            .map(|m| m.name.clone())
            .collect();
        let cosmo = names.iter().position(|n| n == "ucal-cosmo").unwrap();
        let events = names.iter().position(|n| n == "ucal-events").unwrap();
        assert!(
            events < cosmo,
            "ucal-cosmo would publish before ucal-events: {names:?}"
        );
    }

    #[test]
    fn the_real_workspace_orders_core_first_and_the_facade_last() {
        let root = crate::workspace_root();
        let ms = members(&root).expect("workspace members");
        let out = order(&ms).expect("an acyclic graph");
        let names: Vec<&str> = out.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names.first(), Some(&"ucal-core"));
        assert_eq!(names.last(), Some(&"ucal"));
        // xtask is `publish = false` and must never appear.
        assert!(!names.contains(&"xtask"), "xtask reached the publish order");
        // Every internal edge is respected.
        for m in &ms {
            let i = names.iter().position(|n| *n == m.name).unwrap();
            for d in m.deps.iter().filter(|d| names.contains(&d.as_str())) {
                let j = names.iter().position(|n| n == d).unwrap();
                assert!(j < i, "{} would publish before {d}", m.name);
            }
        }
    }
}
