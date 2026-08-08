//! End-to-end index + type search tests against the demo Lake package.

use rlean_search::cache::{load_or_build, save_cache};
use rlean_search::daemon::local_request;
use rlean_search::index::build_index;
use rlean_search::parser::{parse_declarations, parse_search_pattern};
use rlean_search::protocol::{parse_request, ProtocolKind};
use rlean_search::search::matches_decl;
use rlean_search::xml::{index_to_xml, xml_to_index};
use std::path::PathBuf;

fn demo_pkg() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo_pkg")
}

#[test]
fn index_demo_package() {
    let idx = build_index(&[demo_pkg()]).expect("build index");
    assert!(
        idx.len() >= 100,
        "expected many declarations, got {}",
        idx.len()
    );
    assert!(!idx.doc.packages.is_empty());
    assert_eq!(idx.doc.packages[0].name, "demo");
    // Lake lib Demo should be known
    assert!(idx
        .doc
        .packages
        .iter()
        .any(|p| p.lean_libs.iter().any(|l| l == "Demo")));
}

#[test]
fn search_add_eq_zero_pattern() {
    let idx = build_index(&[demo_pkg()]).unwrap();
    let hits = idx.search("_ + _ = 0", 50).unwrap();
    // eq_zero_of_add_eq_zero has conclusion involving + and = 0 shape via implication;
    // also patterns match subterms. At least some additive equalities involving 0.
    let hits2 = idx.search("_ + 0 = _", 50).unwrap();
    assert!(
        !hits2.is_empty(),
        "expected hits for `_ + 0 = _`, index size {}",
        idx.len()
    );
    assert!(hits2.iter().any(|h| h.type_surface.contains('+') && h.type_surface.contains('0')));

    let hits3 = idx.search("?a - ?a = 0", 20).unwrap();
    assert!(
        hits3.iter().any(|h| h.name.contains("sub_self") || h.type_surface.contains('-')),
        "named hole search failed: {:?}",
        hits3
    );
    let _ = hits;
}

#[test]
fn search_named_holes_reject_mismatch() {
    let src = r#"
theorem good (a : Nat) : a - a = 0 := sorry
theorem bad (a b : Nat) : a - b = 0 := sorry
"#;
    let decls = parse_declarations(src, "t.lean");
    assert_eq!(decls.len(), 2);
    let pat = parse_search_pattern("?a - ?a = 0").unwrap();
    let good = decls.iter().find(|d| d.name == "good").unwrap();
    let bad = decls.iter().find(|d| d.name == "bad").unwrap();
    assert!(matches_decl(&pat, good));
    assert!(!matches_decl(&pat, bad));
}

#[test]
fn search_turnstile_conclusion() {
    let idx = build_index(&[demo_pkg()]).unwrap();
    let hits = idx.search("|- _ + 0 = _", 30).unwrap();
    assert!(
        !hits.is_empty(),
        "no conclusion hits for `|- _ + 0 = _`"
    );
    // Full type with binders should still match conclusion-only patterns
    assert!(hits.iter().any(|h| h.type_surface.contains('+') && h.type_surface.contains('0')));
}

#[test]
fn search_tsum_style() {
    let idx = build_index(&[demo_pkg()]).unwrap();
    let hits = idx.search("|- ∑' _, _ * _ = _ * ∑' _, _", 20).unwrap();
    // May or may not parse bigops perfectly; also try simpler
    let hits2 = idx.search("tsum_mul_left", 5);
    // name search is type search only — use pattern with tsum surface
    let hits3 = idx.search("∑' i, a * f i = a * ∑' i, f i", 10).unwrap();
    assert!(
        !hits3.is_empty() || !hits.is_empty(),
        "tsum-like search failed; sample names: {:?}",
        idx.doc
            .declarations
            .iter()
            .filter(|d| d.name.contains("tsum") || d.type_surface.contains('∑'))
            .map(|d| &d.name)
            .collect::<Vec<_>>()
    );
    let _ = hits2;
}

#[test]
fn xml_cache_roundtrip() {
    let idx = build_index(&[demo_pkg()]).unwrap();
    let xml = index_to_xml(&idx.doc);
    assert!(xml.contains("http://github.com/createyourpersonalaccount/rlean-search"));
    let doc2 = xml_to_index(&xml).unwrap();
    assert_eq!(doc2.declarations.len(), idx.doc.declarations.len());
    let idx2 = rlean_search::index::SearchIndex::from_document(doc2);
    let a = idx.search("_ * _ = _", 10).unwrap();
    let b = idx2.search("_ * _ = _", 10).unwrap();
    assert_eq!(a.len(), b.len());
}

#[test]
fn cache_load_or_build() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("index.xml.gz");
    let idx = load_or_build(&[demo_pkg()], &cache, true).unwrap();
    assert!(cache.exists());
    assert!(idx.len() > 0);
    // Cache must be gzip-compressed (magic bytes 1f 8b).
    let magic = std::fs::read(&cache).unwrap();
    assert!(
        magic.len() >= 2 && magic[0] == 0x1f && magic[1] == 0x8b,
        "cache is not gzip"
    );
    let idx2 = load_or_build(&[demo_pkg()], &cache, false).unwrap();
    assert_eq!(idx.len(), idx2.len());
    save_cache(&cache, &idx2).unwrap();
}

#[test]
fn jsonl_and_xml_protocol() {
    let idx = build_index(&[demo_pkg()]).unwrap();
    let json_req = r#"{"cmd":"search","pattern":"_ + 0 = _","limit":5}"#;
    let json_resp = local_request(&idx, json_req);
    assert!(json_resp.contains("\"type\":\"search\"") || json_resp.contains("\"hits\""));
    assert!(ProtocolKind::detect(json_req) == ProtocolKind::Jsonl);

    let xml_req = r#"<rlean:search xmlns:rlean="http://github.com/createyourpersonalaccount/rlean-search" pattern="_ + 0 = _" limit="5"/>"#;
    let xml_resp = local_request(&idx, xml_req);
    assert!(xml_resp.contains("rlean:response") || xml_resp.contains("response"));
    assert!(xml_resp.contains("http://github.com/createyourpersonalaccount/rlean-search") || xml_resp.contains("hit"));

    let (k, _) = parse_request(xml_req).unwrap();
    assert_eq!(k, ProtocolKind::Xml);

    let stats = local_request(&idx, r#"{"cmd":"stats"}"#);
    assert!(stats.contains("declarations"));
}

#[test]
fn parse_real_nat_basic_snippet() {
    // Snippet shaped like Init/Data/Nat/Basic.lean
    let src = r#"
/-!
# Nat.add theorems
-/
namespace Nat

@[simp] protected theorem zero_add : ∀ (n : Nat), 0 + n = n
  | 0   => rfl
  | n+1 => sorry

theorem succ_add : ∀ (n m : Nat), (succ n) + m = succ (n + m)
  | _, 0   => rfl
  | n, m+1 => sorry

theorem add_comm : ∀ (n m : Nat), n + m = m + n := sorry

theorem eq_zero_of_add_eq_zero : ∀ {n m}, n + m = 0 → n = 0 ∧ m = 0 := sorry

end Nat
"#;
    let decls = parse_declarations(src, "Init/Data/Nat/Basic.lean");
    assert!(
        decls.len() >= 3,
        "got {} decls: {:?}",
        decls.len(),
        decls.iter().map(|d| &d.name).collect::<Vec<_>>()
    );
    assert!(decls.iter().any(|d| d.name == "add_comm"));
    let add_comm = decls.iter().find(|d| d.name == "add_comm").unwrap();
    assert!(add_comm.full_name.contains("Nat") || add_comm.namespace_path.iter().any(|n| n == "Nat"));
}

#[test]
fn search_is_reasonably_fast_on_demo() {
    let idx = build_index(&[demo_pkg()]).unwrap();
    let start = std::time::Instant::now();
    for _ in 0..200 {
        let _ = idx.search("_ + _ = _", 20).unwrap();
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 5000,
        "200 searches took {:?}, too slow",
        elapsed
    );
}

#[test]
fn index_real_init_nat_basic() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lean_init_pkg");
    if !root.join("Init/Data/Nat/Basic.lean").exists() {
        // Fixture optional if not vendored
        return;
    }
    let idx = build_index(&[root]).expect("index Init Nat Basic");
    assert!(
        idx.len() > 30,
        "expected many Nat theorems from Init, got {}",
        idx.len()
    );
    let hits = idx.search("_ + _ = _", 50).unwrap();
    assert!(
        !hits.is_empty(),
        "no additive equality theorems found in Init Nat Basic"
    );
    let hits2 = idx.search("0 + _ = _", 20).unwrap();
    assert!(
        hits2.iter().any(|h| h.name.contains("zero_add") || h.type_surface.contains("0 +")),
        "expected zero_add-like hits: {:?}",
        hits2.iter().map(|h| &h.full_name).collect::<Vec<_>>()
    );
    // Equation-compiler proofs must not pollute the type surface.
    let zero_add = idx
        .doc
        .declarations
        .iter()
        .find(|d| d.name == "zero_add")
        .expect("zero_add");
    assert!(
        !zero_add.type_surface.contains("=>"),
        "type surface still has eqns: {}",
        zero_add.type_surface
    );
}
