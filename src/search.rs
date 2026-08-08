//! Fast type-aware pattern matching with holes and named holes.

use crate::ast::{Declaration, TypeExpr};
use crate::parser::{parse_search_pattern, SearchPattern};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One search hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub name: String,
    pub full_name: String,
    pub kind: String,
    pub type_surface: String,
    pub file: String,
    pub line: usize,
    pub score: i32,
}

/// Match pattern against a type expression with unification for named holes.
pub fn matches_type(pattern: &TypeExpr, target: &TypeExpr) -> bool {
    let mut env: HashMap<String, TypeExpr> = HashMap::new();
    unify(pattern, target, &mut env)
}

fn unify(pat: &TypeExpr, target: &TypeExpr, env: &mut HashMap<String, TypeExpr>) -> bool {
    match pat {
        TypeExpr::Hole => true,
        TypeExpr::NamedHole(name) => {
            if let Some(bound) = env.get(name) {
                // Must be structurally equal to previous binding
                alpha_eq(bound, target)
            } else {
                env.insert(name.clone(), target.clone());
                true
            }
        }
        TypeExpr::Ident(a) => match target {
            TypeExpr::Ident(b) => idents_compatible(a, b),
            // Allow matching against projections / raw loosely? no
            _ => false,
        },
        TypeExpr::NatLit(a) => matches!(target, TypeExpr::NatLit(b) if a == b),
        TypeExpr::Literal(a) => matches!(target, TypeExpr::Literal(b) if a == b),
        TypeExpr::App(pf, pa) => match target {
            TypeExpr::App(tf, ta) => unify(pf, tf, env) && unify(pa, ta, env),
            _ => false,
        },
        TypeExpr::BinOp {
            op: po,
            left: pl,
            right: pr,
        } => match target {
            TypeExpr::BinOp {
                op: to,
                left: tl,
                right: tr,
            } if ops_compatible(po, to) => unify(pl, tl, env) && unify(pr, tr, env),
            _ => false,
        },
        TypeExpr::UnaryOp { op: po, arg: pa } => match target {
            TypeExpr::UnaryOp { op: to, arg: ta } if ops_compatible(po, to) => {
                unify(pa, ta, env)
            }
            _ => false,
        },
        TypeExpr::Postfix { arg: pa, op: po } => match target {
            TypeExpr::Postfix { arg: ta, op: to } if ops_compatible(po, to) => {
                unify(pa, ta, env)
            }
            _ => false,
        },
        TypeExpr::Arrow(pa, pb) => match target {
            TypeExpr::Arrow(ta, tb) => unify(pa, ta, env) && unify(pb, tb, env),
            TypeExpr::Pi { binder, body } => {
                // Match domain against binder type if present
                let domain_ok = match &binder.ty {
                    Some(ty) => unify(pa, ty, env),
                    None => true,
                };
                domain_ok && unify(pb, body, env)
            }
            _ => false,
        },
        TypeExpr::Forall { body: pb, .. } => {
            // Pattern quantifiers: match body against target conclusion-ish or full forall
            match target {
                TypeExpr::Forall { body: tb, .. } => unify(pb, tb, env) || unify(pb, target, env),
                _ => unify(pb, target, env),
            }
        }
        TypeExpr::Exists { body: pb, .. } => match target {
            TypeExpr::Exists { body: tb, .. } => unify(pb, tb, env),
            _ => unify(pb, target, env),
        },
        TypeExpr::Lambda { body: pb, .. } => match target {
            TypeExpr::Lambda { body: tb, .. } => unify(pb, tb, env),
            _ => false,
        },
        TypeExpr::Pi {
            binder: _,
            body: pb,
        } => match target {
            TypeExpr::Pi { body: tb, .. } => unify(pb, tb, env),
            TypeExpr::Arrow(_, tb) => unify(pb, tb, env),
            _ => unify(pb, target, env),
        },
        TypeExpr::Proj {
            base: pb,
            field: pf,
        } => match target {
            TypeExpr::Proj {
                base: tb,
                field: tf,
            } if pf == tf => unify(pb, tb, env),
            _ => false,
        },
        TypeExpr::Sort { name: pn, level: pl } => match target {
            TypeExpr::Sort { name: tn, level: tl } if pn == tn => match (pl, tl) {
                (None, _) => true,
                (Some(a), Some(b)) => unify(a, b, env),
                (Some(_), None) => false,
            },
            _ => false,
        },
        TypeExpr::Raw(a) => match target {
            TypeExpr::Raw(b) => a == b,
            other => other.surface().contains(a.as_str()) || a == &other.surface(),
        },
    }
}

fn alpha_eq(a: &TypeExpr, b: &TypeExpr) -> bool {
    match (a, b) {
        (TypeExpr::Hole, _) | (_, TypeExpr::Hole) => true,
        (TypeExpr::NamedHole(x), TypeExpr::NamedHole(y)) => x == y,
        (TypeExpr::Ident(x), TypeExpr::Ident(y)) => idents_compatible(x, y),
        (TypeExpr::NatLit(x), TypeExpr::NatLit(y)) => x == y,
        (TypeExpr::Literal(x), TypeExpr::Literal(y)) => x == y,
        (TypeExpr::App(f1, a1), TypeExpr::App(f2, a2)) => alpha_eq(f1, f2) && alpha_eq(a1, a2),
        (
            TypeExpr::BinOp {
                op: o1,
                left: l1,
                right: r1,
            },
            TypeExpr::BinOp {
                op: o2,
                left: l2,
                right: r2,
            },
        ) => ops_compatible(o1, o2) && alpha_eq(l1, l2) && alpha_eq(r1, r2),
        (TypeExpr::UnaryOp { op: o1, arg: a1 }, TypeExpr::UnaryOp { op: o2, arg: a2 }) => {
            ops_compatible(o1, o2) && alpha_eq(a1, a2)
        }
        (TypeExpr::Postfix { arg: a1, op: o1 }, TypeExpr::Postfix { arg: a2, op: o2 }) => {
            ops_compatible(o1, o2) && alpha_eq(a1, a2)
        }
        (TypeExpr::Arrow(a1, b1), TypeExpr::Arrow(a2, b2)) => alpha_eq(a1, a2) && alpha_eq(b1, b2),
        (TypeExpr::Forall { body: b1, .. }, TypeExpr::Forall { body: b2, .. }) => alpha_eq(b1, b2),
        (TypeExpr::Exists { body: b1, .. }, TypeExpr::Exists { body: b2, .. }) => alpha_eq(b1, b2),
        (TypeExpr::Pi { body: b1, .. }, TypeExpr::Pi { body: b2, .. }) => alpha_eq(b1, b2),
        (TypeExpr::Proj { base: b1, field: f1 }, TypeExpr::Proj { base: b2, field: f2 }) => {
            f1 == f2 && alpha_eq(b1, b2)
        }
        (TypeExpr::Sort { name: n1, level: l1 }, TypeExpr::Sort { name: n2, level: l2 }) => {
            n1 == n2
                && match (l1, l2) {
                    (None, None) => true,
                    (Some(a), Some(b)) => alpha_eq(a, b),
                    _ => false,
                }
        }
        (TypeExpr::Raw(x), TypeExpr::Raw(y)) => x == y,
        _ => false,
    }
}

fn idents_compatible(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    // Allow suffix match: `add` vs `Nat.add`, `HAdd.hAdd` vs etc.
    let a_last = a.rsplit('.').next().unwrap_or(a);
    let b_last = b.rsplit('.').next().unwrap_or(b);
    a_last == b_last
}

fn ops_compatible(a: &str, b: &str) -> bool {
    a == b
}

/// Does `pattern` match declaration type (or conclusion if requested)?
pub fn matches_decl(pat: &SearchPattern, decl: &Declaration) -> bool {
    let effective = decl.effective_type();
    if pat.conclusion_only {
        // Match pattern against conclusion; also try matching if pattern itself is full type
        // whose conclusion matches.
        let conc = effective.conclusion();
        if matches_type(&pat.expr, conc) {
            return true;
        }
        // Pattern may include binders; use its conclusion
        if matches_type(pat.expr.conclusion(), conc) {
            return true;
        }
        // Also allow matching conclusion pattern against any sub-conclusion along arrows
        return match_any_conclusion(&pat.expr, &effective);
    }

    // Full type match, or match against conclusion (common UX: `_ + _ = _` finds theorems)
    if matches_type(&pat.expr, &effective) {
        return true;
    }
    if matches_type(&pat.expr, effective.conclusion()) {
        return true;
    }
    // Match ignoring outer binders on both sides
    if matches_type(pat.expr.conclusion(), effective.conclusion()) {
        return true;
    }
    // Subterm match for patterns that are pure relational goals
    match_subterm(&pat.expr, &effective)
}

fn match_any_conclusion(pat: &TypeExpr, ty: &TypeExpr) -> bool {
    if matches_type(pat, ty.conclusion()) {
        return true;
    }
    match ty {
        TypeExpr::Arrow(_, r) | TypeExpr::Pi { body: r, .. } | TypeExpr::Forall { body: r, .. } => {
            match_any_conclusion(pat, r)
        }
        _ => false,
    }
}

fn match_subterm(pat: &TypeExpr, ty: &TypeExpr) -> bool {
    if matches_type(pat, ty) {
        return true;
    }
    match ty {
        TypeExpr::App(f, a) => match_subterm(pat, f) || match_subterm(pat, a),
        TypeExpr::BinOp { left, right, .. } => {
            match_subterm(pat, left) || match_subterm(pat, right)
        }
        TypeExpr::UnaryOp { arg, .. } | TypeExpr::Postfix { arg, .. } => match_subterm(pat, arg),
        TypeExpr::Arrow(a, b) => match_subterm(pat, a) || match_subterm(pat, b),
        TypeExpr::Forall { body, .. }
        | TypeExpr::Exists { body, .. }
        | TypeExpr::Lambda { body, .. }
        | TypeExpr::Pi { body, .. } => match_subterm(pat, body),
        TypeExpr::Proj { base, .. } => match_subterm(pat, base),
        _ => false,
    }
}

/// Score a hit for ranking (higher is better).
pub fn score_hit(pat: &SearchPattern, decl: &Declaration) -> i32 {
    let mut score = 0;
    let effective = decl.effective_type();
    if matches_type(&pat.expr, effective.conclusion()) {
        score += 100;
    }
    if pat.conclusion_only {
        score += 10;
    }
    // Prefer shorter names slightly
    score += 50usize.saturating_sub(decl.full_name.len()) as i32;
    // Prefer theorems
    score += match decl.kind {
        crate::ast::DeclKind::Theorem => 3,
        crate::ast::DeclKind::Lemma => 2,
        crate::ast::DeclKind::Axiom => 1,
    };
    score
}

pub fn parse_pattern(input: &str) -> anyhow::Result<SearchPattern> {
    parse_search_pattern(input).map_err(|e| anyhow::anyhow!(e.to_string()))
}

/// Candidate filtering keys from a pattern for inverted index lookup.
pub fn pattern_index_keys(pat: &SearchPattern) -> Vec<String> {
    let expr = if pat.conclusion_only {
        pat.expr.conclusion()
    } else {
        &pat.expr
    };
    let mut keys = Vec::new();
    let head = expr.head_key();
    if !head.starts_with("hole") {
        keys.push(head);
    }
    for op in expr.operators() {
        keys.push(format!("op:{op}"));
    }
    // also conclusion head of full pattern
    let ch = pat.expr.conclusion().head_key();
    if !ch.starts_with("hole") {
        keys.push(ch);
    }
    keys.sort();
    keys.dedup();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::DeclKind;
    use crate::parser::parse_type;

    fn decl_with_type(surface: &str) -> Declaration {
        let ty = parse_type(surface).unwrap_or(TypeExpr::Raw(surface.into()));
        Declaration {
            kind: DeclKind::Theorem,
            name: "t".into(),
            full_name: "T.t".into(),
            binders: vec![],
            ty,
            type_surface: surface.into(),
            file: "t.lean".into(),
            line: 1,
            module: None,
            namespace_path: vec![],
            attributes: vec![],
        }
    }

    #[test]
    fn hole_match_add_eq_zero() {
        let d = decl_with_type("n + m = 0");
        let p = parse_pattern("_ + _ = 0").unwrap();
        assert!(matches_decl(&p, &d));
    }

    #[test]
    fn named_hole_same() {
        let d = decl_with_type("x - x = 0");
        let p = parse_pattern("?a - ?a = 0").unwrap();
        assert!(matches_decl(&p, &d));
        let d2 = decl_with_type("x - y = 0");
        assert!(!matches_decl(&p, &d2));
    }

    #[test]
    fn turnstile_conclusion() {
        let d = decl_with_type("∀ (n : Nat), n + 0 = n");
        let p = parse_pattern("|- _ + 0 = _").unwrap();
        assert!(matches_decl(&p, &d));
    }

    #[test]
    fn no_match_different_op() {
        let d = decl_with_type("n * m = 0");
        let p = parse_pattern("_ + _ = 0").unwrap();
        assert!(!matches_decl(&p, &d));
    }
}
