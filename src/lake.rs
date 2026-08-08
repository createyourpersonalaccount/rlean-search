//! Lake build system / package manager hierarchy awareness.

use crate::ast::PackageInfo;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Discover a Lake package rooted at `root` (directory containing lakefile).
pub fn discover_package(root: impl AsRef<Path>) -> Result<PackageInfo> {
    let root = root.as_ref().canonicalize().with_context(|| {
        format!(
            "canonicalize package root {}",
            root.as_ref().display()
        )
    })?;
    let root_str = root.display().to_string();

    let lakefile_toml = root.join("lakefile.toml");
    let lakefile_lean = root.join("lakefile.lean");
    let lakefile_toml_in = root.join("lakefile.toml.in");

    if lakefile_toml.exists() {
        return parse_lakefile_toml(&root, &fs::read_to_string(&lakefile_toml)?);
    }
    if lakefile_toml_in.exists() {
        return parse_lakefile_toml(&root, &fs::read_to_string(&lakefile_toml_in)?);
    }
    if lakefile_lean.exists() {
        return parse_lakefile_lean(&root, &fs::read_to_string(&lakefile_lean)?);
    }

    // Fallback: treat directory as a loose source tree of .lean files.
    Ok(PackageInfo {
        name: root
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "root".into()),
        root: root_str,
        src_dirs: vec![".".into()],
        lean_libs: vec![],
    })
}

fn parse_lakefile_toml(root: &Path, text: &str) -> Result<PackageInfo> {
    let mut name = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "package".into());
    let mut src_dir = ".".to_string();
    let mut lean_libs = Vec::new();
    let mut package_name_set = false;
    let mut in_lean_lib = false;
    let mut in_package_table = false;

    // Minimal TOML scrape: package `name`, `srcDir`, and `[[lean_lib]] name`
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("[[lean_lib]]") || t.starts_with("[[lean-lib]]") {
            in_lean_lib = true;
            in_package_table = false;
            continue;
        }
        if t.starts_with('[') {
            in_lean_lib = false;
            in_package_table = t == "[package]" || t.starts_with("[package.");
            continue;
        }
        if in_lean_lib {
            if let Some(v) = toml_str_value(t, "name") {
                lean_libs.push(v);
            }
            continue;
        }
        if let Some(v) = toml_str_value(t, "name") {
            // Prefer first top-level / package name; ignore later keys.
            if !package_name_set || in_package_table {
                name = v;
                package_name_set = true;
            }
        }
        if let Some(v) = toml_str_value(t, "srcDir") {
            src_dir = v;
        }
    }

    // Also match `name = "Init"` style under lean_lib via simple state — already done.

    // If no libs found, look for directories matching common roots
    if lean_libs.is_empty() {
        for cand in ["Mathlib", "Init", "Std", "Lean", "Lake"] {
            if root.join(cand).is_dir() || root.join("src").join(cand).is_dir() {
                lean_libs.push(cand.into());
            }
        }
    }

    let mut src_dirs = vec![src_dir.clone()];
    // Lake often has srcDir = "src" for lean4; also include root for Mathlib style.
    if src_dir != "." && root.join(&src_dir).is_dir() {
        // ok
    } else if root.join("src").is_dir() {
        src_dirs = vec!["src".into()];
    }

    Ok(PackageInfo {
        name,
        root: root.display().to_string(),
        src_dirs,
        lean_libs,
    })
}

fn parse_lakefile_lean(root: &Path, text: &str) -> Result<PackageInfo> {
    let mut name = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "package".into());
    let mut lean_libs = Vec::new();

    // package mathlib where
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("package ") {
            let n = rest.split_whitespace().next().unwrap_or("").trim();
            if !n.is_empty() && n != "where" {
                name = n.to_string();
            }
        }
        // lean_lib Mathlib where
        if let Some(rest) = t.strip_prefix("lean_lib ") {
            let n = rest.split_whitespace().next().unwrap_or("").trim();
            if !n.is_empty() && n != "where" {
                lean_libs.push(n.to_string());
            }
        }
    }

    let src_dirs = if root.join("src").is_dir() {
        vec!["src".into()]
    } else {
        vec![".".into()]
    };

    if lean_libs.is_empty() {
        for cand in ["Mathlib", "Init", "Std", "Lean"] {
            if root.join(cand).is_dir() {
                lean_libs.push(cand.into());
            }
        }
    }

    Ok(PackageInfo {
        name,
        root: root.display().to_string(),
        src_dirs,
        lean_libs,
    })
}

fn toml_str_value(line: &str, key: &str) -> Option<String> {
    let line = line.trim();
    let prefix = format!("{key}");
    if !line.starts_with(&prefix) {
        return None;
    }
    let after = line[prefix.len()..].trim_start();
    if !after.starts_with('=') {
        return None;
    }
    let v = after[1..].trim();
    // strip quotes / placeholders
    let v = v.trim_matches('"').trim_matches('\'').trim();
    // skip cmake placeholders
    if v.contains("${") {
        return None;
    }
    if v.is_empty() {
        return None;
    }
    Some(v.to_string())
}

/// Enumerate `.lean` source files for a package, respecting Lake layout.
pub fn enumerate_lean_files(pkg: &PackageInfo) -> Result<Vec<PathBuf>> {
    let root = Path::new(&pkg.root);
    let mut files = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut roots: Vec<PathBuf> = Vec::new();
    for sd in &pkg.src_dirs {
        let p = if sd == "." {
            root.to_path_buf()
        } else {
            root.join(sd)
        };
        if p.is_dir() {
            roots.push(p);
        }
    }
    if roots.is_empty() {
        roots.push(root.to_path_buf());
    }

    // If lean_libs specified, prefer those subtrees when they exist.
    let mut search_dirs = Vec::new();
    if !pkg.lean_libs.is_empty() {
        for lib in &pkg.lean_libs {
            for r in &roots {
                let cand = r.join(lib);
                if cand.is_dir() {
                    search_dirs.push(cand);
                } else if r.file_name().and_then(|s| s.to_str()) == Some(lib.as_str()) {
                    search_dirs.push(r.clone());
                }
            }
            // Mathlib style: package root / Mathlib
            let cand = root.join(lib);
            if cand.is_dir() {
                search_dirs.push(cand);
            }
        }
    }
    if search_dirs.is_empty() {
        search_dirs = roots;
    }

    for dir in search_dirs {
        for entry in WalkDir::new(&dir)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // skip build artifacts / git / lake cache
                !matches!(
                    name.as_ref(),
                    ".git" | ".lake" | "build" | "lake-packages" | "target" | ".rlean-search"
                )
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("lean") {
                let canon = path.to_path_buf();
                if seen.insert(canon.display().to_string()) {
                    files.push(canon);
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Guess Lake module name from file path relative to package root / srcDir.
pub fn module_name_for(pkg: &PackageInfo, file: &Path) -> Option<String> {
    let root = Path::new(&pkg.root);
    let rel = file.strip_prefix(root).ok()?;
    // strip srcDir prefix
    let mut rel = rel.to_path_buf();
    for sd in &pkg.src_dirs {
        if sd != "." {
            if let Ok(r) = rel.strip_prefix(sd) {
                rel = r.to_path_buf();
                break;
            }
        }
    }
    let mut parts: Vec<String> = rel
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    if let Some(last) = parts.last_mut() {
        if let Some(stem) = Path::new(last).file_stem() {
            *last = stem.to_string_lossy().into_owned();
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("."))
    }
}

/// Discover package from a path that may be a file or directory.
pub fn discover_from_path(path: impl AsRef<Path>) -> Result<(PackageInfo, Vec<PathBuf>)> {
    let path = path.as_ref();
    if path.is_file() {
        let parent = path.parent().unwrap_or(Path::new("."));
        // walk up looking for lakefile
        let pkg_root = find_lake_root(parent).unwrap_or_else(|| parent.to_path_buf());
        let pkg = discover_package(&pkg_root)?;
        return Ok((pkg, vec![path.canonicalize().unwrap_or_else(|_| path.to_path_buf())]));
    }
    let pkg = discover_package(path)?;
    let files = enumerate_lean_files(&pkg)?;
    Ok((pkg, files))
}

pub fn find_lake_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.canonicalize().ok()?;
    loop {
        if cur.join("lakefile.toml").exists()
            || cur.join("lakefile.lean").exists()
            || cur.join("lakefile.toml.in").exists()
        {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn discover_toml_package() {
        let dir = tempdir().unwrap();
        let mut f = fs::File::create(dir.path().join("lakefile.toml")).unwrap();
        writeln!(
            f,
            r#"
name = "demo"
srcDir = "."

[[lean_lib]]
name = "Demo"
"#
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("Demo")).unwrap();
        fs::write(dir.path().join("Demo/Basic.lean"), "theorem t : True := trivial\n").unwrap();
        let pkg = discover_package(dir.path()).unwrap();
        assert_eq!(pkg.name, "demo");
        assert!(pkg.lean_libs.iter().any(|l| l == "Demo"));
        let files = enumerate_lean_files(&pkg).unwrap();
        assert_eq!(files.len(), 1);
    }
}
