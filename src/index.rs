//! In-memory inverted index for fast type search.

use crate::ast::{IndexDocument, PackageInfo};
use crate::lake::{self, module_name_for};
use crate::parser::parse_declarations_with_path;
use crate::search::{
    matches_decl, parse_pattern, pattern_index_keys, score_hit, SearchHit,
};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Fast searchable index kept in memory (daemon) or built per query (oneshot + cache).
#[derive(Debug, Clone, Default)]
pub struct SearchIndex {
    pub doc: IndexDocument,
    /// head_key / op:key → declaration indices
    pub inverted: HashMap<String, Vec<usize>>,
    /// full_name → index
    pub by_name: HashMap<String, usize>,
}

impl SearchIndex {
    pub fn from_document(doc: IndexDocument) -> Self {
        let mut inverted: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_name = HashMap::new();
        for (i, d) in doc.declarations.iter().enumerate() {
            by_name.insert(d.full_name.clone(), i);
            by_name.entry(d.name.clone()).or_insert(i);
            let eff = d.effective_type();
            let mut keys = HashSet::new();
            keys.insert(eff.head_key());
            keys.insert(eff.conclusion().head_key());
            for op in eff.operators() {
                keys.insert(format!("op:{op}"));
            }
            for k in keys {
                inverted.entry(k).or_default().push(i);
            }
        }
        Self {
            doc,
            inverted,
            by_name,
        }
    }

    pub fn len(&self) -> usize {
        self.doc.declarations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.doc.declarations.is_empty()
    }

    pub fn search(&self, pattern: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let pat = parse_pattern(pattern)?;
        let keys = pattern_index_keys(&pat);

        let candidates: Vec<usize> = if keys.is_empty() {
            (0..self.doc.declarations.len()).collect()
        } else {
            // Intersection-ish: start from smallest posting list among non-empty keys
            let mut lists: Vec<&Vec<usize>> = keys
                .iter()
                .filter_map(|k| self.inverted.get(k))
                .collect();
            if lists.is_empty() {
                // Fallback: scan all (pattern too open or unknown op)
                (0..self.doc.declarations.len()).collect()
            } else {
                lists.sort_by_key(|l| l.len());
                let mut set: HashSet<usize> = lists[0].iter().copied().collect();
                for l in lists.iter().skip(1) {
                    let other: HashSet<usize> = l.iter().copied().collect();
                    // union for recall (type patterns often need union of op keys)
                    set.extend(other);
                }
                let mut v: Vec<usize> = set.into_iter().collect();
                v.sort_unstable();
                v
            }
        };

        let mut hits = Vec::new();
        for i in candidates {
            let d = &self.doc.declarations[i];
            if matches_decl(&pat, d) {
                hits.push(SearchHit {
                    name: d.name.clone(),
                    full_name: d.full_name.clone(),
                    kind: d.kind.as_str().to_string(),
                    type_surface: d.type_surface.clone(),
                    file: d.file.clone(),
                    line: d.line,
                    score: score_hit(&pat, d),
                });
            }
        }
        hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.full_name.cmp(&b.full_name)));
        hits.truncate(limit);
        Ok(hits)
    }
}

/// Build an index by scanning package roots / files.
pub fn build_index(paths: &[impl AsRef<Path>]) -> Result<SearchIndex> {
    let mut doc = IndexDocument::new();
    let mut hasher = Sha256::new();
    let mut packages: Vec<PackageInfo> = Vec::new();
    let mut all_files: Vec<(PackageInfo, std::path::PathBuf)> = Vec::new();

    for p in paths {
        let p = p.as_ref();
        let (pkg, files) = lake::discover_from_path(p)?;
        hasher.update(pkg.root.as_bytes());
        hasher.update(pkg.name.as_bytes());
        packages.push(pkg.clone());
        for f in files {
            all_files.push((pkg.clone(), f));
        }
    }

    // de-dupe packages by root
    let mut seen_pkg = HashSet::new();
    for pkg in packages {
        if seen_pkg.insert(pkg.root.clone()) {
            doc.packages.push(pkg);
        }
    }

    for (pkg, file) in all_files {
        let content = match fs::read_to_string(&file) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("skip {}: {e}", file.display());
                continue;
            }
        };
        hasher.update(file.display().to_string().as_bytes());
        hasher.update(content.as_bytes());

        let file_str = file.display().to_string();
        let module = module_name_for(&pkg, &file);
        let decls =
            parse_declarations_with_path(&content, &file_str, module.as_deref());
        doc.declarations.extend(decls);
    }

    doc.source_hash = hex::encode(hasher.finalize());
    doc.created_at = chrono::Utc::now().to_rfc3339();
    Ok(SearchIndex::from_document(doc))
}

/// Shared handle for the daemon.
pub type SharedIndex = Arc<parking_lot::RwLock<SearchIndex>>;

pub fn shared_index(idx: SearchIndex) -> SharedIndex {
    Arc::new(parking_lot::RwLock::new(idx))
}
