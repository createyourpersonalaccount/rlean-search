//! On-disk gzipped XML cache for one-shot invocations (`index.xml.gz`).

use crate::index::SearchIndex;
use crate::xml::{read_index_file, write_index_file};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub const DEFAULT_CACHE_DIR: &str = ".rlean-search";
/// Default cache filename: gzipped XML at the fastest compression level.
pub const DEFAULT_CACHE_FILE: &str = "index.xml.gz";

pub fn default_cache_path(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref()
        .join(DEFAULT_CACHE_DIR)
        .join(DEFAULT_CACHE_FILE)
}

/// Load cache if present and `source_hash` matches expected (when provided).
pub fn load_cache(path: impl AsRef<Path>, expected_hash: Option<&str>) -> Result<Option<SearchIndex>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    let doc = read_index_file(path)?;
    if let Some(exp) = expected_hash {
        if doc.source_hash != exp {
            tracing::info!("cache hash mismatch; rebuilding");
            return Ok(None);
        }
    }
    Ok(Some(SearchIndex::from_document(doc)))
}

pub fn save_cache(path: impl AsRef<Path>, index: &SearchIndex) -> Result<()> {
    write_index_file(path.as_ref(), &index.doc)
}

/// Try load cache; if missing, build from paths and save.
pub fn load_or_build(
    paths: &[impl AsRef<Path>],
    cache_path: impl AsRef<Path>,
    force_rebuild: bool,
) -> Result<SearchIndex> {
    if !force_rebuild {
        if let Some(idx) = load_cache(cache_path.as_ref(), None)? {
            // Validate lightly: non-empty or empty paths
            if !idx.is_empty() || paths.is_empty() {
                return Ok(idx);
            }
        }
    }
    let idx = crate::index::build_index(paths)?;
    save_cache(cache_path.as_ref(), &idx)?;
    Ok(idx)
}
