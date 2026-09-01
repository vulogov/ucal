//! `verify-release` — C3. Check what was published against what was built.
//!
//! # The asymmetry this closes
//!
//! This repository verifies its source exhaustively: twenty-two recorded
//! defects each known to be rejected, a hundred and thirty-six citations that
//! must resolve, two integer backends that must agree bit for bit, a harness
//! that refuses to meet its own exit criterion below sixty checks.
//!
//! **All of it stopped at the repository boundary.** What a user actually
//! receives is a `.crate` from crates.io or a tarball from a GitHub release, and
//! until 1.9.0 nothing checked either. A reader who wanted to check a claim
//! about the *source* had more mechanism available than they could use; a reader
//! who downloaded a *binary* had a checksum file anyone could have written.
//!
//! # Three comparisons
//!
//! 1. **The published `.crate` against this tree.** Downloaded from
//!    `static.crates.io`, extracted, and compared file by file with what
//!    `cargo package` produces here.
//! 2. **The release binaries against `SHA256SUMS.txt`.** Downloaded from the
//!    GitHub release and hashed.
//! 3. **`SHA256SUMS.txt` against its detached signature.** The half a checksum
//!    file cannot do for itself.
//!
//! # The stop condition did not fire, and that is the finding
//!
//! C3 expected `cargo package` not to be reproducible and said what to do:
//!
//! > **Stop if** `cargo package` is not reproducible enough to compare —
//! > timestamps and file ordering may make a byte-for-byte match impossible.
//! > Then the check becomes *the file list and the content hashes of the
//! > sources*, which is weaker and still answers the question a reader is
//! > asking.
//!
//! **It is reproducible.** All six crates published for 1.8.0 are
//! byte-for-byte identical to what `cargo package` produces from a checkout of
//! `v1.8.0` — cargo normalises the mtimes a `.crate` would otherwise carry, so
//! the strong answer is available and the fallback never fired. That was
//! measured, not predicted; this paragraph said the opposite until the check
//! was run.
//!
//! Both comparisons are performed anyway and the weaker one is reported when
//! the stronger fails, because a future cargo may stop normalising and the
//! honest answer then is *every file agrees, and the archives differ* rather
//! than a red check with no detail in it.
//!
//! `Cargo.toml` is excluded from the comparison and the exclusion is named in
//! the output. Cargo rewrites it during packaging — workspace inheritance is
//! resolved, `[dev-dependencies]` paths are stripped — so the published file is
//! *supposed* to differ from the one in the tree, and comparing them would
//! report a difference that is correct behaviour. `Cargo.toml.orig` carries the
//! original and **is** compared, which is where a real substitution would show.
//!
//! # What this cannot tell you
//!
//! That the tree it is run against is the tree the tag names. It checks that
//! out itself where git allows, and says which commit it compared; a caller who
//! points it at a modified checkout gets an honest report about a modified
//! checkout. And a signature verifies who signed, never whether what they signed
//! deserved it — **the signing key has no authority behind it that is not the
//! author**, which is a fact about the world and not about this file.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// How a comparison came out.
enum Verdict {
    /// Agreed, with a note on how strongly.
    Ok(String),
    /// Disagreed. This is the finding the whole check exists for.
    Fail(Vec<String>),
    /// Could not be performed, with the reason. **Not** a pass.
    Skipped(String),
}

/// Run all three comparisons for a version. Returns a process exit code.
pub fn run(root: &Path, version: &str) -> i32 {
    println!("verify-release {version} — what was published, against what is here\n");

    for tool in ["curl", "tar", "gh"] {
        if Command::new(tool).arg("--version").output().is_err() {
            eprintln!("  FAIL  `{tool}` is not on PATH, and this check shells out to it");
            return 7;
        }
    }

    let tag = format!("v{version}");
    let tmp = root.join("target/verify-release");
    let _ = std::fs::remove_dir_all(&tmp);
    if let Err(e) = std::fs::create_dir_all(&tmp) {
        eprintln!("  FAIL  cannot create {}: {e}", tmp.display());
        return 6;
    }

    // **The tag is checked out, not assumed.** The first run of this check was
    // made from a 1.9.0 tree against the published 1.8.0 and reported five
    // failures, every one of them `cargo package` refusing because the internal
    // requirements read `^1.9.0` and 1.9.0 is not on the index yet. That is
    // correct behaviour by cargo and a useless answer from this check: it was
    // comparing the wrong tree and saying the published crate was wrong.
    //
    // So a detached worktree at the tag is what gets packaged, and the caller is
    // told which commit that is. A repository without the tag falls back to the
    // working tree with the fallback named — an answer about a different tree,
    // labelled as one.
    let worktree = tmp.join("tree");
    // A seam, and the only reason it exists: **the failure path has to be
    // exercisable.** This check asks the network, so it cannot join the defect
    // corpus — which runs offline — and X1 already lists three checks
    // hand-verified for want of a way to inject into them. `UCAL_RELEASE_TREE`
    // points the packaging half at a tree of the caller's choosing, so a
    // deliberately tampered checkout of the tag can be compared with what was
    // actually published, and the FAIL branch can be seen to fire rather than
    // assumed to.
    if let Ok(dir) = std::env::var("UCAL_RELEASE_TREE") {
        println!("  --    UCAL_RELEASE_TREE is set: packaging {dir}");
        println!("        rather than a checkout of {tag}. This exists to exercise the");
        println!("        failure path; a real check does not set it.");
        println!();
        return compare_all(root, &tmp, Path::new(&dir), version, &tag);
    }
    let pkg_root = match git(root, &["rev-parse", &format!("{tag}^{{commit}}")]) {
        Some(tagged) => {
            let added = Command::new("git")
                .current_dir(root)
                .args(["worktree", "add", "--detach", "--quiet"])
                .arg(&worktree)
                .arg(&tag)
                .output();
            match added {
                Ok(o) if o.status.success() => {
                    println!("  ok    packaging {tag} ({}), checked out into", short(&tagged));
                    println!("        {}", worktree.display());
                    worktree.clone()
                }
                Ok(o) => {
                    println!("  --    cannot check out {tag}: {}", String::from_utf8_lossy(&o.stderr).trim());
                    println!("        Comparing against this working tree instead.");
                    root.to_path_buf()
                }
                Err(e) => {
                    println!("  --    git could not run: {e}");
                    root.to_path_buf()
                }
            }
        }
        None => {
            println!("  --    no tag {tag} in this repository");
            println!("        Comparing against the working tree as it stands, which is a");
            println!("        different question from the one this check is meant to answer.");
            root.to_path_buf()
        }
    };
    println!();

    let code = compare_all(root, &tmp, &pkg_root, version, &tag);
    cleanup(root, &worktree);
    code
}

/// The three comparisons, given the tree to package from.
fn compare_all(root: &Path, tmp: &Path, pkg_root: &Path, version: &str, tag: &str) -> i32 {
    let mut failed = 0usize;
    let mut skipped = 0usize;

    let members = published_members(pkg_root);
    if members.is_empty() {
        eprintln!("  FAIL  no publishable members found in the workspace manifest");
        return 6;
    }

    println!("1. the published .crate against the tag");
    let mut explained_lock = false;
    let mut explained_files: Vec<String> = Vec::new();
    for name in &members {
        match compare_crate(pkg_root, tmp, name, version) {
            Verdict::Ok(note) => {
                if note.contains("Cargo.lock") {
                    explained_lock = true;
                }
                for (_, f, _) in EXPLAINED {
                    if note.contains(f) && !explained_files.iter().any(|e| e == f) {
                        explained_files.push((*f).to_string());
                    }
                }
                println!("  ok    {name} {version}  {note}");
            }
            Verdict::Skipped(why) => {
                skipped += 1;
                println!("  --    {name} {version}  {why}");
            }
            Verdict::Fail(lines) => {
                failed += 1;
                println!("  FAIL  {name} {version}");
                for l in lines {
                    println!("          {l}");
                }
            }
        }
    }

    // R2 — the reason, once. Six crates each carrying the same paragraph is a
    // legend repeated until it is scenery, which is the state that let two
    // standing failures go unread in the first place.
    let mut legend: Vec<&str> = Vec::new();
    if explained_lock {
        legend.push(
            "Cargo.lock — differs only in the recorded versions of this workspace's own \
             crates. `cargo package` strips the path from an intra-workspace dependency \
             and resolves the registry requirement instead, so it names whichever \
             compatible release existed at the moment of packaging. That makes \
             `cargo package` unreproducible for this workspace from the moment a later \
             compatible version is published — 1.9.0 measured byte-reproducibility \
             correctly, and nobody stated its shelf life.",
        );
    }
    for (v, f, why) in EXPLAINED {
        if *v == version && explained_files.iter().any(|e| e == f) {
            legend.push(why);
        }
    }
    if !legend.is_empty() {
        println!("\n  understood differences:");
        for l in &legend {
            println!("    {l}");
        }
    }

    println!("\n2. the release binaries against SHA256SUMS.txt");
    match compare_binaries(tmp, tag) {
        Verdict::Ok(note) => println!("  ok    {note}"),
        Verdict::Skipped(why) => {
            skipped += 1;
            println!("  --    {why}");
        }
        Verdict::Fail(lines) => {
            failed += 1;
            println!("  FAIL  the release binaries do not match the attached checksums");
            for l in lines {
                println!("          {l}");
            }
        }
    }

    println!("\n3. SHA256SUMS.txt against its signature");
    match check_signature(root, tmp) {
        Verdict::Ok(note) => println!("  ok    {note}"),
        Verdict::Skipped(why) => {
            skipped += 1;
            println!("  --    {why}");
        }
        Verdict::Fail(lines) => {
            failed += 1;
            println!("  FAIL  the signature does not verify");
            for l in lines {
                println!("          {l}");
            }
        }
    }

    println!();
    if failed > 0 {
        println!("  {failed} comparison(s) failed. A published artefact does not match what");
        println!("  this tree produces, which is the finding this check exists for.");
        return 6;
    }
    if skipped > 0 {
        println!("  {skipped} comparison(s) could not be performed, and are reported as");
        println!("  `--` rather than as passes. A check that could not run has not passed.");
        // §19.5 exit 5: the answer is incomplete, which is neither a pass nor a
        // failure of the thing being checked.
        return 5;
    }
    println!("  Every published artefact for {version} matches what this tree produces,");
    println!("  and the checksums are signed by the key in fixtures/ucal.pub.");
    println!("  A signature says who signed. It does not say the signer was right.");
    0
}

/// Remove the temporary worktree, so a second run is not refused by git.
fn cleanup(root: &Path, worktree: &Path) {
    if worktree.exists() {
        let _ = Command::new("git")
            .current_dir(root)
            .args(["worktree", "remove", "--force"])
            .arg(worktree)
            .output();
    }
}

fn short(sha: &str) -> String {
    sha.chars().take(12).collect()
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// The workspace members that are published, in manifest order.
///
/// Read from the manifest rather than listed here, for the same reason
/// [`crate::publish`] recomputes its order: a seventh crate must not need
/// anybody to remember this file.
fn published_members(root: &Path) -> Vec<String> {
    let Ok(manifest) = std::fs::read_to_string(root.join("Cargo.toml")) else {
        return Vec::new();
    };
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
        let m = root.join(&d).join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&m) else {
            continue;
        };
        if text.contains("publish = false") {
            continue;
        }
        for line in text.lines() {
            let l = line.trim();
            if let Some(rest) = l.strip_prefix("name") {
                if let Some(q) = rest.find('"') {
                    if let Some(e) = rest[q + 1..].find('"') {
                        out.push(rest[q + 1..q + 1 + e].to_string());
                    }
                }
                break;
            }
        }
    }
    out
}

/// Files cargo is *supposed* to rewrite while packaging.
///
/// `Cargo.toml` only. Workspace inheritance is resolved into it and
/// `[dev-dependencies]` path entries are stripped, so the published file differs
/// from the one in the tree by design and comparing them would report correct
/// behaviour as a defect. `Cargo.toml.orig` holds the original and is compared,
/// which is where a substitution would actually show.
const REWRITTEN: &[&str] = &["Cargo.toml"];

/// Files cargo *adds* while packaging, which have no counterpart in the tree.
const GENERATED: &[&str] = &[".cargo_vcs_info.json", "Cargo.lock", "Cargo.toml.orig"];

/// R2 — differences that are understood, enumerated so they cannot grow quietly.
///
/// A standing failure is worse than no check: `verify-release` reported five
/// crates differing for v1.9.0 and six for v1.11.0 from the day each was
/// published, and nothing read either. *CI green with no known-failing job* is a
/// 1.0 exit criterion, and a check whose output is known-bad entries is one
/// nobody reads.
///
/// So each is either fixed or **named here with its reason**, keyed to the exact
/// version it applies to — the shape the retired signing key got in
/// `check_signing_key`. A blanket exemption for `.cargo_vcs_info.json` would
/// make the next release free to repeat the mistake unnoticed; this one does not.
const EXPLAINED: &[(&str, &str, &str)] = &[(
    "1.11.0",
    ".cargo_vcs_info.json",
    "1.11.0 published from the cut commit while the tag sits on the merge, so \
     cargo recorded a commit that a checkout of the tag cannot reproduce. Every \
     other byte matches. The procedure now says to merge before publishing; a \
     published version cannot be replaced, so this one stays",
)];

/// Whether a `Cargo.lock` difference is only in the versions of *this
/// workspace's own crates*.
///
/// **Measured, and it is the whole of the v1.9.0 finding.** `cargo package`
/// strips the `path` from an intra-workspace dependency and resolves the
/// registry requirement instead, so packaging `ucal-civil` records whichever
/// `ucal-core` satisfies `^1.9.0` *at that moment*. In August that was 1.9.0,
/// because nothing newer existed; today it is 1.11.0. Four lines differ, all of
/// them one `[[package]]` block.
///
/// That makes `cargo package` **not reproducible** for a workspace whose crates
/// depend on each other by caret requirement, from the moment a later compatible
/// version is published. 1.9.0's notes recorded byte-reproducibility measured
/// across 1.8.0; the measurement was correct and the claim had a shelf life that
/// nobody stated. `ucal-core` still matches to the byte, because it is the one
/// crate with no sibling to resolve.
///
/// Anything else differing is still a failure. This normalises exactly the
/// entries it can explain and compares the rest verbatim.
fn lock_differs_only_in_siblings(published: &str, local: &str, siblings: &[String]) -> bool {
    fn strip(text: &str, siblings: &[String]) -> Vec<String> {
        let mut out = Vec::new();
        let mut in_sibling = false;
        for line in text.lines() {
            if line.trim() == "[[package]]" {
                in_sibling = false;
            }
            if let Some(rest) = line.trim().strip_prefix("name = ") {
                let name = rest.trim_matches('"');
                in_sibling = siblings.iter().any(|s| s == name);
            }
            // Inside a sibling's block, the two fields cargo resolves at
            // packaging time are dropped. The block's presence, its name and
            // its dependency list are all still compared.
            if in_sibling {
                let t = line.trim();
                if t.starts_with("version = ") || t.starts_with("checksum = ") {
                    continue;
                }
            }
            out.push(line.to_string());
        }
        out
    }
    !siblings.is_empty() && strip(published, siblings) == strip(local, siblings)
}

/// The crates this workspace publishes, read from the tree rather than listed.
///
/// A hard-coded list here would be a second enumeration of the six crates, and
/// this cycle's R3 is about exactly that: two hand-maintained lists of the same
/// commands, neither checked for completeness.
fn sibling_crates(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root.join("crates")) else {
        return out;
    };
    for e in entries.flatten() {
        if !e.path().join("Cargo.toml").exists() {
            continue;
        }
        if let Some(name) = e.file_name().to_str() {
            out.push(name.to_string());
        }
    }
    out.sort();
    out
}

fn compare_crate(root: &Path, tmp: &Path, name: &str, version: &str) -> Verdict {
    let siblings = sibling_crates(root);
    let stem = format!("{name}-{version}");
    let url = format!("https://static.crates.io/crates/{name}/{stem}.crate");
    let downloaded = tmp.join(format!("{stem}.published.crate"));

    let out = Command::new("curl")
        .args(["-fsSL", "--max-time", "120", "-o"])
        .arg(&downloaded)
        .arg(&url)
        .output();
    match out {
        Ok(o) if o.status.success() => {}
        Ok(_) => {
            return Verdict::Skipped(format!(
                "not on crates.io, or not reachable: {url}"
            ))
        }
        Err(e) => return Verdict::Skipped(format!("curl could not run: {e}")),
    }

    // `--no-verify`: the compile is not what is being compared, and building six
    // crates would make this check cost a release rather than a coffee.
    let packed = Command::new("cargo")
        .current_dir(root)
        .args(["package", "-p", name, "--no-verify", "--allow-dirty", "--quiet"])
        .output();
    match packed {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            return Verdict::Fail(vec![
                "cargo package failed here, so there is nothing to compare against".into(),
                String::from_utf8_lossy(&o.stderr).trim().to_string(),
            ])
        }
        Err(e) => return Verdict::Skipped(format!("cargo could not run: {e}")),
    }
    let local = root.join(format!("target/package/{stem}.crate"));
    if !local.exists() {
        return Verdict::Skipped(format!("cargo package produced no {}", local.display()));
    }

    let a = match entries(&downloaded, tmp, &format!("{stem}.published")) {
        Ok(v) => v,
        Err(e) => return Verdict::Fail(vec![format!("cannot read the published crate: {e}")]),
    };
    let b = match entries(&local, tmp, &format!("{stem}.local")) {
        Ok(v) => v,
        Err(e) => return Verdict::Fail(vec![format!("cannot read the local crate: {e}")]),
    };

    let mut problems = Vec::new();
    let mut explained: Vec<String> = Vec::new();
    let read = |label: &str, path: &str| -> Option<String> {
        std::fs::read_to_string(tmp.join(label).join(format!("{stem}/{path}"))).ok()
    };
    for (path, hash) in &a {
        if REWRITTEN.contains(&path.as_str()) {
            continue;
        }
        match b.get(path) {
            None => problems.push(format!("published, and not produced here: {path}")),
            Some(h) if h != hash => {
                // R2 — a difference that is understood says so, once, rather
                // than standing as a failure nobody reads.
                if path == "Cargo.lock" {
                    if let (Some(pv), Some(lv)) = (
                        read(&format!("{stem}.published"), path),
                        read(&format!("{stem}.local"), path),
                    ) {
                        if lock_differs_only_in_siblings(&pv, &lv, &siblings) {
                            explained.push("Cargo.lock".to_string());
                            continue;
                        }
                    }
                }
                if EXPLAINED
                    .iter()
                    .any(|(v, f, _)| *v == version && f == path)
                {
                    explained.push(path.clone());
                    continue;
                }
                problems.push(format!("differs: {path}"));
                problems.push(format!("  published {hash}"));
                problems.push(format!("  here      {h}"));
            }
            Some(_) => {}
        }
    }
    for path in b.keys() {
        if REWRITTEN.contains(&path.as_str()) || GENERATED.contains(&path.as_str()) {
            continue;
        }
        if !a.contains_key(path) {
            problems.push(format!("produced here, and not published: {path}"));
        }
    }

    if !problems.is_empty() {
        return Verdict::Fail(problems);
    }

    let identical = std::fs::read(&downloaded).ok() == std::fs::read(&local).ok();
    let n = a.len();
    if !explained.is_empty() {
        return Verdict::Ok(format!(
            "{n} files agree; {} understood: {}",
            explained.len(),
            explained.join(", ")
        ));
    }
    Verdict::Ok(if identical {
        // What actually happens, against the expectation C3 recorded: cargo
        // normalises the mtimes, so the archives match to the byte.
        format!("{n} files, and the archives are byte-identical")
    } else {
        // C3's fallback, kept for a future cargo that stops normalising. Not as
        // weak as it sounds: every file in the published crate is present here
        // with the same contents.
        format!(
            "{n} files agree; the archives differ in bytes, and Cargo.toml is \
             excluded because cargo rewrites it while packaging"
        )
    })
}

/// Every regular file in a `.crate`, by path within the crate, with its sha256.
fn entries(
    archive: &Path,
    tmp: &Path,
    label: &str,
) -> Result<BTreeMap<String, String>, String> {
    let dir = tmp.join(label);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let out = Command::new("tar")
        .arg("xzf")
        .arg(archive)
        .arg("-C")
        .arg(&dir)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }

    // One top-level directory, `<name>-<version>`. Paths are reported relative
    // to it so the two sides are comparable.
    let mut roots = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect::<Vec<_>>();
    roots.sort();
    let base = roots.pop().ok_or("the archive has no top-level directory")?;

    let mut out = BTreeMap::new();
    let mut stack = vec![base.clone()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(bytes) = std::fs::read(&p) {
                let rel = p
                    .strip_prefix(&base)
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_default();
                let mut h = Sha256::new();
                h.update(&bytes);
                out.insert(rel, h.finalize().iter().map(|b| format!("{b:02x}")).collect());
            }
        }
    }
    Ok(out)
}

fn compare_binaries(tmp: &Path, tag: &str) -> Verdict {
    let dir = tmp.join("release-assets");
    let _ = std::fs::remove_dir_all(&dir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return Verdict::Skipped(format!("cannot create {}: {e}", dir.display()));
    }

    let out = Command::new("gh")
        .args(["release", "download", tag, "--dir"])
        .arg(&dir)
        .args(["--pattern", "*"])
        .output();
    match out {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            return Verdict::Skipped(format!(
                "no release assets for {tag}: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            ))
        }
        Err(e) => return Verdict::Skipped(format!("gh could not run: {e}")),
    }

    hash_against_sums(&dir, tag)
}

/// Hash every file `SHA256SUMS.txt` names, in a directory of downloaded assets.
///
/// Split from the download so it can be tested offline against a directory
/// built by hand. The download half is the network; **this half is the check**,
/// and a check whose failing branch has never run is a check nobody has seen
/// work.
fn hash_against_sums(dir: &Path, tag: &str) -> Verdict {
    let sums = dir.join("SHA256SUMS.txt");
    let Ok(text) = std::fs::read_to_string(&sums) else {
        return Verdict::Skipped(format!("{tag} has no SHA256SUMS.txt attached"));
    };

    let mut problems = Vec::new();
    let mut checked = 0usize;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(want), Some(name)) = (parts.next(), parts.next()) else {
            continue;
        };
        // `sha256sum` writes `hash  name` and `shasum` writes `hash *name`.
        let name = name.trim_start_matches('*');
        let f = dir.join(name);
        let Ok(bytes) = std::fs::read(&f) else {
            problems.push(format!("named in SHA256SUMS.txt and not attached: {name}"));
            continue;
        };
        let mut h = Sha256::new();
        h.update(&bytes);
        let got: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
        checked += 1;
        if got != want {
            problems.push(format!("differs: {name}"));
            problems.push(format!("  attached checksum {want}"));
            problems.push(format!("  the file hashes to {got}"));
        }
    }

    if checked == 0 {
        // The floor. A checksum file that named nothing would otherwise report
        // success having compared nothing — V1 Finding 1's shape, and the reason
        // every check in this tree now has one.
        return Verdict::Fail(vec![
            "SHA256SUMS.txt named no files, so nothing was compared".into(),
        ]);
    }
    if !problems.is_empty() {
        return Verdict::Fail(problems);
    }
    Verdict::Ok(format!(
        "{checked} attached artefact(s) hash to what SHA256SUMS.txt says. \
         This proves the download was not corrupted, not that the build was honest"
    ))
}

fn check_signature(root: &Path, tmp: &Path) -> Verdict {
    let dir = tmp.join("release-assets");
    let sums = dir.join("SHA256SUMS.txt");
    let sig = dir.join("SHA256SUMS.txt.minisig");
    if !sums.exists() {
        return Verdict::Skipped("no SHA256SUMS.txt was downloaded, so there is nothing to verify".into());
    }
    if !sig.exists() {
        return Verdict::Skipped(
            "UNSIGNED — no SHA256SUMS.txt.minisig is attached. Every release before \
             1.9.0 is in this state, and the release notes said so in every one of them"
                .into(),
        );
    }
    if Command::new("minisign").arg("-v").output().is_err() {
        return Verdict::Skipped(
            "`minisign` is not on PATH. The signature is attached and was not checked; \
             see https://jedisct1.github.io/minisign/"
                .into(),
        );
    }

    // **Which key signed this release.** The 2026-08-31 rotation means one
    // public key no longer covers the whole history: v1.9.0 was signed by the
    // retired key, v1.11.0 onwards by the current one. Trying both, rather than
    // deciding from the version number, because a table of which key signed
    // which release is a second record of a fact the signature already carries
    // — and it is the table, not the signature, that would go stale.
    //
    // A release that verifies under the *retired* key says so rather than
    // reporting a plain pass: it is a weaker statement, and one worth reading.
    let current = root.join("fixtures/ucal.pub");
    let retired = root.join("fixtures/ucal-retired.pub");
    let verify = |key: &std::path::Path| {
        Command::new("minisign")
            .arg("-Vm")
            .arg(&sums)
            .arg("-p")
            .arg(key)
            .output()
    };
    match verify(&current) {
        Ok(o) if o.status.success() => {
            return Verdict::Ok(
                "SHA256SUMS.txt verifies against fixtures/ucal.pub, which is published in \
                 five places. The key has no authority behind it that is not the author"
                    .into(),
            )
        }
        Err(e) => return Verdict::Skipped(format!("minisign could not run: {e}")),
        Ok(_) => {}
    }
    if retired.exists() {
        if let Ok(o) = verify(&retired) {
            if o.status.success() {
                return Verdict::Ok(
                    "SHA256SUMS.txt verifies against fixtures/ucal-retired.pub — the key \
                     retired on 2026-08-31 when its passphrase was lost. The signature is \
                     as good as it ever was; the key was orphaned, not compromised, and \
                     nothing signs its replacement"
                        .into(),
                );
            }
        }
    }
    match verify(&current) {
        Ok(o) => Verdict::Fail(vec![
            String::from_utf8_lossy(&o.stderr).trim().to_string(),
            String::from_utf8_lossy(&o.stdout).trim().to_string(),
            "and it does not verify against the retired key either".into(),
        ]),
        Err(e) => Verdict::Skipped(format!("minisign could not run: {e}")),
    }
}

/// `sign-release` — C1. Sign a release's checksum file, from the laptop.
///
/// # Why this is a command and not automation
///
/// The signature has to be produced by a person, and that is not a limitation
/// to be engineered around. **The minisign secret key is held on one laptop
/// with an offline backup and must never enter this repository or CI.** A
/// signature CI could produce would attest to a GitHub secret, which is a
/// different claim from the one this project makes about
/// `fixtures/SHA256SUMS.minisig`, and a weaker one presented in the same shape
/// would be worse than no signature at all.
///
/// C1 said as much when it opened, and said what to build instead:
///
/// > **Stop if** the signature cannot be produced without the key reaching CI.
/// > Then the honest outcome is a documented manual step and a release note
/// > saying the artefacts are signed by hand, which is worth less than
/// > automation and more than nothing.
///
/// So this is that manual step, reduced to one command. It downloads the
/// checksum file CI attached, signs it, **verifies the signature against the
/// published public key before uploading it**, and attaches it. What it removes
/// is not the person — it is every opportunity to sign the wrong file, upload
/// an unverified signature, or silently re-sign one that already exists.
pub fn sign(root: &Path, version: &str) -> i32 {
    let tag = format!("v{version}");
    for tool in ["gh", "minisign"] {
        let ok = if tool == "minisign" {
            // minisign has no `--version`; `-v` prints it and exits 0.
            Command::new(tool).arg("-v").output().is_ok()
        } else {
            Command::new(tool).arg("--version").output().is_ok()
        };
        if !ok {
            eprintln!("  FAIL  `{tool}` is not on PATH, and signing shells out to it");
            if tool == "minisign" {
                eprintln!("        https://jedisct1.github.io/minisign/");
            }
            return 7;
        }
    }

    let dir = root.join("target/verify-release/sign");
    let _ = std::fs::remove_dir_all(&dir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("  FAIL  cannot create {}: {e}", dir.display());
        return 6;
    }

    // Never re-sign silently. A second signature over a checksum file that has
    // one is either a mistake or a substitution, and neither should be a
    // side effect of running a command twice.
    let listed = Command::new("gh")
        .args(["release", "view", &tag, "--json", "assets", "--jq", ".assets[].name"])
        .output();
    match listed {
        Ok(o) if o.status.success() => {
            let names = String::from_utf8_lossy(&o.stdout);
            if names.lines().any(|n| n.trim() == "SHA256SUMS.txt.minisig") {
                println!("  --    {tag} already has SHA256SUMS.txt.minisig attached.");
                println!("        Nothing to do. Verify it with:");
                println!("          cargo run -p xtask -- verify-release {version}");
                return 0;
            }
            if !names.lines().any(|n| n.trim() == "SHA256SUMS.txt") {
                eprintln!("  FAIL  {tag} has no SHA256SUMS.txt to sign.");
                eprintln!("        The release workflow attaches it; wait for it to finish.");
                return 6;
            }
        }
        _ => {
            // **Two conditions, two messages.** This said *no release, or `gh`
            // cannot see it* and left the reader to guess which — and the first
            // person to hit it was unauthenticated, not missing a release. That
            // is the shape D-A24 catalogued: one diagnostic covering conditions
            // with different remedies.
            let authed = Command::new("gh")
                .args(["auth", "status"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if authed {
                eprintln!("  FAIL  there is no release {tag} to sign.");
                eprintln!("        Push the tag first; the release workflow builds the");
                eprintln!("        binaries and attaches SHA256SUMS.txt for this to sign.");
            } else {
                eprintln!("  FAIL  `gh` is not authenticated, so it cannot see any release.");
                eprintln!("        Run `gh auth login`, or set GH_TOKEN in this shell.");
                eprintln!("        Nothing has been read or changed.");
            }
            return 6;
        }
    }

    let out = Command::new("gh")
        .args(["release", "download", &tag, "--pattern", "SHA256SUMS.txt", "--dir"])
        .arg(&dir)
        .output();
    match out {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            eprintln!("  FAIL  could not download SHA256SUMS.txt: {}",
                String::from_utf8_lossy(&o.stderr).trim());
            return 6;
        }
        Err(e) => {
            eprintln!("  FAIL  gh could not run: {e}");
            return 6;
        }
    }
    let sums = dir.join("SHA256SUMS.txt");
    let Ok(text) = std::fs::read_to_string(&sums) else {
        eprintln!("  FAIL  SHA256SUMS.txt did not download");
        return 6;
    };
    let named = text.lines().filter(|l| !l.trim().is_empty()).count();
    if named == 0 {
        // The floor, before a signature is put over it. Signing an empty
        // checksum file would produce a valid signature vouching for nothing,
        // which is exactly the shape of evidence this project refuses.
        eprintln!("  FAIL  SHA256SUMS.txt names no files. Signing it would produce a");
        eprintln!("        valid signature over nothing.");
        return 6;
    }
    println!("  ok    downloaded SHA256SUMS.txt, naming {named} artefact(s)");
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        println!("          {line}");
    }

    // The trusted comment is signed along with the file, so it cannot later be
    // presented as vouching for a different release. Same convention as the
    // conformance vectors, and `spec/CONFORMANCE.md` states it.
    // Since the 2026-08-31 rotation the key carries no passphrase, so there is
    // no prompt to warn about — and promising one that does not appear reads as
    // the wrong key having been picked up.
    println!("\n  The key is yours and stays on this machine; nothing here reads,");
    println!("  copies or transmits it. It has no passphrase (spec/CONFORMANCE.md).\n");
    let signed = Command::new("minisign")
        .arg("-Sm")
        .arg(&sums)
        .arg("-t")
        .arg(format!("ucal {tag} release checksums"))
        .status();
    match signed {
        Ok(st) if st.success() => {}
        _ => {
            eprintln!("  FAIL  minisign did not sign the file");
            return 6;
        }
    }
    let sig = dir.join("SHA256SUMS.txt.minisig");
    if !sig.exists() {
        eprintln!("  FAIL  minisign reported success and wrote no signature");
        return 6;
    }

    // **Verify before uploading.** A signature made with the wrong key verifies
    // against nothing, and finding that out after it is attached to a release
    // is finding out too late.
    let pubkey = root.join("fixtures/ucal.pub");
    let checked = Command::new("minisign")
        .arg("-Vm")
        .arg(&sums)
        .arg("-p")
        .arg(&pubkey)
        .output();
    match checked {
        Ok(o) if o.status.success() => {
            println!("  ok    the signature verifies against fixtures/ucal.pub");
        }
        Ok(o) => {
            eprintln!("  FAIL  the signature does not verify against fixtures/ucal.pub.");
            eprintln!("        Nothing has been uploaded. A different key was used, or the");
            eprintln!("        published public key is wrong — both are worth stopping for.");
            eprintln!("        {}", String::from_utf8_lossy(&o.stderr).trim());
            return 6;
        }
        Err(e) => {
            eprintln!("  FAIL  minisign could not verify: {e}");
            return 6;
        }
    }

    let uploaded = Command::new("gh")
        .args(["release", "upload", &tag])
        .arg(&sig)
        .arg("--clobber")
        .output();
    match uploaded {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            eprintln!("  FAIL  could not attach the signature: {}",
                String::from_utf8_lossy(&o.stderr).trim());
            return 6;
        }
        Err(e) => {
            eprintln!("  FAIL  gh could not run: {e}");
            return 6;
        }
    }

    println!("  ok    SHA256SUMS.txt.minisig is attached to {tag}");
    println!("\n  A downloader verifies it with:");
    println!("    minisign -Vm SHA256SUMS.txt \\");
    println!("      -P RWTgVaXr8eTV6+dsVwvMkwZglwUJS69tF+78i2MFUi5LBaUXPf66M+FV");
    println!("\n  That key signs v1.11.0 onwards. v1.9.0 was signed by the retired");
    println!("  key in fixtures/ucal-retired.pub, whose secret half was lost on");
    println!("  2026-08-31 — see spec/CONFORMANCE.md.");
    println!("\n  And this repository checks the whole release with:");
    println!("    cargo run -p xtask -- verify-release {version}");
    println!("\n  The key has no authority behind it that is not the author. What a");
    println!("  signature adds is that the checksums came from whoever holds it, which");
    println!("  a file generated by CI beside the binaries it describes cannot say.");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The members are read from the manifest, not listed in this file.
    #[test]
    fn the_published_members_come_from_the_manifest() {
        let root = crate::workspace_root();
        let m = published_members(&root);
        assert!(m.contains(&"ucal-core".to_string()), "{m:?}");
        assert!(m.contains(&"ucal".to_string()), "{m:?}");
        assert!(
            !m.contains(&"xtask".to_string()),
            "xtask is `publish = false` and must never reach a release check: {m:?}"
        );
    }

    /// A directory of assets, with the checksum file the test wants.
    fn assets(label: &str, files: &[(&str, &str)], sums: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ucal-verify-release-{label}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir");
        for (name, body) in files {
            std::fs::write(dir.join(name), body).expect("write");
        }
        std::fs::write(dir.join("SHA256SUMS.txt"), sums).expect("write");
        dir
    }

    fn sha(body: &str) -> String {
        let mut h = Sha256::new();
        h.update(body.as_bytes());
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }

    /// The happy path: every named file hashes to what the file says.
    #[test]
    fn matching_checksums_pass() {
        let dir = assets(
            "match",
            &[("ucal.tar.gz", "a binary")],
            &format!("{}  ucal.tar.gz\n", sha("a binary")),
        );
        assert!(
            matches!(hash_against_sums(&dir, "v0.0.0"), Verdict::Ok(_)),
            "a matching checksum was not accepted"
        );
    }

    /// **The branch the check exists for.** A file whose contents do not match
    /// the checksum attached beside it.
    #[test]
    fn a_substituted_artefact_is_caught() {
        let dir = assets(
            "swapped",
            &[("ucal.tar.gz", "something else entirely")],
            &format!("{}  ucal.tar.gz\n", sha("a binary")),
        );
        let Verdict::Fail(lines) = hash_against_sums(&dir, "v0.0.0") else {
            panic!("a substituted artefact was accepted");
        };
        assert!(lines.iter().any(|l| l.contains("ucal.tar.gz")), "{lines:?}");
    }

    /// A checksum file naming a file that is not there.
    #[test]
    fn a_missing_artefact_is_caught() {
        let dir = assets("missing", &[], &format!("{}  ucal.tar.gz\n", sha("a binary")));
        assert!(matches!(hash_against_sums(&dir, "v0.0.0"), Verdict::Fail(_)));
    }

    /// **The floor.** An empty checksum file must not report success having
    /// compared nothing — V1 Finding 1's shape, which was fourteen checks in
    /// this tree at once.
    #[test]
    fn an_empty_checksum_file_is_not_a_pass() {
        let dir = assets("empty", &[("ucal.tar.gz", "a binary")], "\n");
        let Verdict::Fail(lines) = hash_against_sums(&dir, "v0.0.0") else {
            panic!("a checksum file naming nothing reported success");
        };
        assert!(lines.iter().any(|l| l.contains("nothing was compared")), "{lines:?}");
    }

    /// `shasum -a 256` writes `hash *name` and `sha256sum` writes `hash  name`.
    /// The release workflow uses whichever the runner has, so both parse.
    #[test]
    fn both_checksum_dialects_parse() {
        let dir = assets(
            "dialects",
            &[("ucal.tar.gz", "a binary")],
            &format!("{} *ucal.tar.gz\n", sha("a binary")),
        );
        assert!(matches!(hash_against_sums(&dir, "v0.0.0"), Verdict::Ok(_)));
    }

    /// `Cargo.toml` is excluded and `Cargo.toml.orig` is not.
    ///
    /// The exclusion is the one place this check deliberately does not compare
    /// something, so it is asserted rather than left to a comment. If a future
    /// edit adds a path here, this test is where the reason has to be written
    /// down again.
    #[test]
    fn only_the_manifest_cargo_rewrites_is_excluded() {
        assert_eq!(REWRITTEN, &["Cargo.toml"]);
        assert!(
            !REWRITTEN.contains(&"Cargo.toml.orig"),
            "Cargo.toml.orig carries the original and is where a substitution would show"
        );
    }

    // ---- R2 ----

    const LOCK_A: &str = "\
[[package]]
name = \"bnum\"
version = \"0.13.0\"
checksum = \"aaa\"

[[package]]
name = \"ucal-core\"
version = \"1.9.0\"
source = \"registry+https://github.com/rust-lang/crates.io-index\"
checksum = \"bbb\"
dependencies = [
 \"bnum\",
]
";

    fn siblings() -> Vec<String> {
        vec!["ucal-core".to_string(), "ucal-civil".to_string()]
    }

    /// A sibling's version moving is the difference cargo makes by design.
    #[test]
    fn a_sibling_resolved_to_a_later_release_is_understood() {
        let later = LOCK_A
            .replace("version = \"1.9.0\"", "version = \"1.11.0\"")
            .replace("checksum = \"bbb\"", "checksum = \"ccc\"");
        assert!(lock_differs_only_in_siblings(LOCK_A, &later, &siblings()));
    }

    /// A third-party dependency moving is not, and must still fail.
    ///
    /// The exemption exists for one cause and would be worthless if it covered
    /// every version line in the file — a substituted dependency is precisely
    /// what this whole check is for.
    #[test]
    fn a_third_party_version_moving_is_not_understood() {
        let tampered = LOCK_A.replace("version = \"0.13.0\"", "version = \"0.14.0\"");
        assert!(!lock_differs_only_in_siblings(LOCK_A, &tampered, &siblings()));
    }

    /// Nor is a sibling's *dependency list* changing, which is not a version.
    #[test]
    fn a_siblings_dependencies_changing_is_not_understood() {
        let tampered = LOCK_A.replace(" \"bnum\",\n", " \"bnum\",\n \"something-else\",\n");
        assert!(!lock_differs_only_in_siblings(LOCK_A, &tampered, &siblings()));
    }

    /// With no siblings known, nothing is explained away.
    ///
    /// `sibling_crates` reads the tree, and a read that returned nothing would
    /// otherwise turn this into a blanket exemption for `Cargo.lock`.
    #[test]
    fn no_siblings_means_no_exemption() {
        let later = LOCK_A.replace("version = \"1.9.0\"", "version = \"1.11.0\"");
        assert!(!lock_differs_only_in_siblings(LOCK_A, &later, &[]));
    }

    /// Every enumerated exception names a version, and none is a wildcard.
    ///
    /// A blanket entry would let the next release repeat 1.11.0's
    /// publish-before-merge unnoticed, which is the opposite of recording it.
    #[test]
    fn every_explained_difference_is_pinned_to_one_version() {
        assert!(!EXPLAINED.is_empty(), "the constant exists to be read");
        for (version, file, why) in EXPLAINED {
            assert!(
                version.split('.').count() == 3 && !version.contains('*'),
                "`{version}` is not a single version"
            );
            assert!(!file.is_empty() && why.len() > 40, "{file}: give the reason");
        }
    }
}
