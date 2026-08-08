//! Parse types drawn from Lean 4 / Mathlib-style surfaces.
//!
//! The corpus in `fixtures/types_corpus.txt` is curated from forms found in
//! lean4-v4.32.2 and mathlib4-v4.32.2 sources.

use pretty_assertions::assert_eq;
use rlean_search::parser::parse_type;
use rlean_search::TypeExpr;
use std::fs;
use std::path::PathBuf;

fn corpus_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/types_corpus.txt")
}

fn load_corpus() -> Vec<(String, String, String)> {
    let text = fs::read_to_string(corpus_path()).expect("types_corpus.txt");
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let mut parts = l.splitn(3, '\t');
            let kind = parts.next().unwrap().to_string();
            let name = parts.next().unwrap().to_string();
            let ty = parts.next().unwrap().to_string();
            (kind, name, ty)
        })
        .collect()
}

#[test]
fn corpus_types_parse_without_panic() {
    let corpus = load_corpus();
    assert!(
        corpus.len() >= 50,
        "expected a useful corpus, got {}",
        corpus.len()
    );
    let mut ok = 0usize;
    let mut raw_fallback = 0usize;
    let mut failures = Vec::new();
    for (kind, name, ty) in &corpus {
        match parse_type(ty) {
            Ok(expr) => {
                ok += 1;
                if matches!(expr, TypeExpr::Raw(_)) {
                    raw_fallback += 1;
                }
                // Must produce a non-empty surface
                assert!(
                    !expr.surface().is_empty(),
                    "{kind} {name}: empty surface for {ty}"
                );
            }
            Err(e) => failures.push(format!("{kind} {name}: {ty} => {e}")),
        }
    }
    // Require high parse success rate on curated corpus
    let rate = ok as f64 / corpus.len() as f64;
    assert!(
        rate >= 0.90,
        "parse success rate {rate:.2} too low; failures:\n{}",
        failures.join("\n")
    );
    assert!(
        failures.len() <= corpus.len() / 10,
        "too many failures: {}",
        failures.join("\n")
    );
    // Most should not need Raw fallback
    assert!(
        raw_fallback < corpus.len() / 3,
        "too many Raw fallbacks: {raw_fallback}"
    );
    eprintln!(
        "corpus: {} types, {} parsed, {} raw, {} failed",
        corpus.len(),
        ok,
        raw_fallback,
        failures.len()
    );
}

#[test]
fn specific_lean_init_style_types() {
    let samples = [
        "n + m = m + n",
        "∀ (n m : Nat), n + m = m + n",
        "∀ {n m k : Nat}, n + m = n + k → m = k",
        "a ∧ b ↔ b ∧ a",
        "¬ n < n",
        "n ≤ m → m ≤ k → n ≤ k",
        "(n : Nat) → n + 0 = n",
        "α → β → Prop",
        "List α → Nat",
        "f ∘ g = h",
        "a ^ (m + n) = a ^ m * a ^ n",
        "∑' i, a * f i = a * ∑' i, f i",
        "[] ++ as = as",
        "as ++ bs = cs",
        "n % n = 0",
        "a ∣ a * b",
        "a - a = 0",
        "- -a = a",
        "a + -a = 0",
        "max a b = max b a",
    ];
    for s in samples {
        let t = parse_type(s).unwrap_or_else(|e| panic!("failed on `{s}`: {e}"));
        assert!(
            !matches!(t, TypeExpr::Raw(_)),
            "unexpected Raw for `{s}`: {}",
            t.surface()
        );
    }
}

#[test]
fn conclusion_extraction() {
    let t = parse_type("∀ (n : Nat), n + 0 = n").unwrap();
    let c = t.conclusion();
    assert!(matches!(c, TypeExpr::BinOp { op, .. } if op == "="));
    assert_eq!(c.head_key(), "op:=");
}

#[test]
fn pi_and_arrow() {
    let t = parse_type("(n : Nat) → n = n").unwrap();
    assert!(matches!(t, TypeExpr::Pi { .. }) || matches!(t, TypeExpr::Arrow(_, _)));
    let t2 = parse_type("Nat → Prop").unwrap();
    assert!(matches!(t2, TypeExpr::Arrow(_, _)));
}

#[test]
fn exists_and_forall_unicode() {
    let t = parse_type("∃ n : Nat, n > 0").unwrap();
    assert!(matches!(t, TypeExpr::Exists { .. }));
    let t2 = parse_type("forall (n : Nat), n = n").unwrap();
    assert!(matches!(t2, TypeExpr::Forall { .. }));
}
