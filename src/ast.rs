//! Abstract syntax for Lean 4 types and declarations.
//!
//! The goal is a useful fragment of Lean surface types: binders, arrows,
//! applications, common infix operators, quantifiers, and search holes.

use serde::{Deserialize, Serialize};
use std::fmt;

/// XML / schema namespace for rlean-search documents.
pub const RLEAN_NS: &str = "http://github.com/createyourpersonalaccount/rlean-search";

/// Kind of searchable declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeclKind {
    Theorem,
    Lemma,
    Axiom,
}

impl DeclKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DeclKind::Theorem => "theorem",
            DeclKind::Lemma => "lemma",
            DeclKind::Axiom => "axiom",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "theorem" => Some(DeclKind::Theorem),
            "lemma" => Some(DeclKind::Lemma),
            "axiom" => Some(DeclKind::Axiom),
            _ => None,
        }
    }
}

impl fmt::Display for DeclKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Explicit binder kind in Lean surface syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BinderKind {
    /// `(x : T)` default
    Default,
    /// `{x : T}` implicit
    Implicit,
    /// `[x : T]` instance-implicit
    Instance,
    /// `⦃x : T⦄` strict-implicit
    StrictImplicit,
}

/// A binder group: `(a b : Nat)` or `{α : Type u}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binder {
    pub kind: BinderKind,
    pub names: Vec<String>,
    pub ty: Option<Box<TypeExpr>>,
}

/// Surface type expression used for indexing and pattern matching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeExpr {
    /// Anonymous hole `_` (search only, or explicit underscore in source).
    Hole,
    /// Named hole `?a` (search patterns; also metavariable-like tokens).
    NamedHole(String),
    /// Identifier / constant, e.g. `Nat`, `List`, `add_comm`.
    Ident(String),
    /// Numeric literal.
    NatLit(String),
    /// String / char literal (kept raw).
    Literal(String),
    /// Function application `f a` (left-associative chain folded as nested apps).
    App(Box<TypeExpr>, Box<TypeExpr>),
    /// Infix binary operator: `a + b`, `x = y`, `P ∧ Q`, etc.
    BinOp {
        op: String,
        left: Box<TypeExpr>,
        right: Box<TypeExpr>,
    },
    /// Unary prefix operator: `¬P`, `-n`, `⁻¹` is usually postfix — see `Postfix`.
    UnaryOp {
        op: String,
        arg: Box<TypeExpr>,
    },
    /// Postfix operator: `a⁻¹`, `f'`.
    Postfix {
        arg: Box<TypeExpr>,
        op: String,
    },
    /// `A → B` / `A -> B`
    Arrow(Box<TypeExpr>, Box<TypeExpr>),
    /// `∀ binders, body` / `forall`
    Forall {
        binders: Vec<Binder>,
        body: Box<TypeExpr>,
    },
    /// `∃ binders, body` / `exists`
    Exists {
        binders: Vec<Binder>,
        body: Box<TypeExpr>,
    },
    /// `fun binders => body` / `λ`
    Lambda {
        binders: Vec<Binder>,
        body: Box<TypeExpr>,
    },
    /// Explicit binder-typed term used in Pi: `(x : A) → B`
    Pi {
        binder: Binder,
        body: Box<TypeExpr>,
    },
    /// Projection `e.field` / `e.1`
    Proj {
        base: Box<TypeExpr>,
        field: String,
    },
    /// Universe / sort: `Prop`, `Type`, `Type u`, `Sort u`, `Type*`, `Sort _`
    Sort {
        name: String,
        level: Option<Box<TypeExpr>>,
    },
    /// Explicit list / structure sugar kept as raw for robustness.
    Raw(String),
}

impl TypeExpr {
    /// Fold a function and argument list into nested `App` nodes.
    pub fn apps(f: TypeExpr, args: impl IntoIterator<Item = TypeExpr>) -> TypeExpr {
        args.into_iter()
            .fold(f, |acc, a| TypeExpr::App(Box::new(acc), Box::new(a)))
    }

    /// Collect free identifiers (rough; for indexing).
    pub fn idents(&self) -> Vec<&str> {
        let mut out = Vec::new();
        self.collect_idents(&mut out);
        out
    }

    fn collect_idents<'a>(&'a self, out: &mut Vec<&'a str>) {
        match self {
            TypeExpr::Ident(s) => out.push(s),
            TypeExpr::App(f, a) => {
                f.collect_idents(out);
                a.collect_idents(out);
            }
            TypeExpr::BinOp { left, right, .. } => {
                left.collect_idents(out);
                right.collect_idents(out);
            }
            TypeExpr::UnaryOp { arg, .. } | TypeExpr::Postfix { arg, .. } => {
                arg.collect_idents(out);
            }
            TypeExpr::Arrow(a, b) => {
                a.collect_idents(out);
                b.collect_idents(out);
            }
            TypeExpr::Forall { binders, body }
            | TypeExpr::Exists { binders, body }
            | TypeExpr::Lambda { binders, body } => {
                for b in binders {
                    if let Some(ty) = &b.ty {
                        ty.collect_idents(out);
                    }
                }
                body.collect_idents(out);
            }
            TypeExpr::Pi { binder, body } => {
                if let Some(ty) = &binder.ty {
                    ty.collect_idents(out);
                }
                body.collect_idents(out);
            }
            TypeExpr::Proj { base, .. } => base.collect_idents(out),
            TypeExpr::Sort { level: Some(l), .. } => l.collect_idents(out),
            _ => {}
        }
    }

    /// Operators appearing in the expression (for inverted index).
    pub fn operators(&self) -> Vec<&str> {
        let mut out = Vec::new();
        self.collect_ops(&mut out);
        out
    }

    fn collect_ops<'a>(&'a self, out: &mut Vec<&'a str>) {
        match self {
            TypeExpr::BinOp { op, left, right } => {
                out.push(op.as_str());
                left.collect_ops(out);
                right.collect_ops(out);
            }
            TypeExpr::UnaryOp { op, arg } => {
                out.push(op.as_str());
                arg.collect_ops(out);
            }
            TypeExpr::Postfix { op, arg } => {
                out.push(op.as_str());
                arg.collect_ops(out);
            }
            TypeExpr::App(f, a) => {
                f.collect_ops(out);
                a.collect_ops(out);
            }
            TypeExpr::Arrow(a, b) => {
                out.push("→");
                a.collect_ops(out);
                b.collect_ops(out);
            }
            TypeExpr::Forall { binders, body } => {
                out.push("∀");
                for b in binders {
                    if let Some(ty) = &b.ty {
                        ty.collect_ops(out);
                    }
                }
                body.collect_ops(out);
            }
            TypeExpr::Exists { binders, body } => {
                out.push("∃");
                for b in binders {
                    if let Some(ty) = &b.ty {
                        ty.collect_ops(out);
                    }
                }
                body.collect_ops(out);
            }
            TypeExpr::Lambda { binders, body } => {
                for b in binders {
                    if let Some(ty) = &b.ty {
                        ty.collect_ops(out);
                    }
                }
                body.collect_ops(out);
            }
            TypeExpr::Pi { binder, body } => {
                out.push("→");
                if let Some(ty) = &binder.ty {
                    ty.collect_ops(out);
                }
                body.collect_ops(out);
            }
            TypeExpr::Proj { base, .. } => base.collect_ops(out),
            TypeExpr::Sort { level: Some(l), .. } => l.collect_ops(out),
            _ => {}
        }
    }

    /// Strip outer binders / arrows to obtain the main conclusion.
    ///
    /// `∀ x, P → Q → R` concludes with `R`.
    pub fn conclusion(&self) -> &TypeExpr {
        match self {
            TypeExpr::Forall { body, .. }
            | TypeExpr::Exists { body, .. }
            | TypeExpr::Lambda { body, .. }
            | TypeExpr::Pi { body, .. } => body.conclusion(),
            TypeExpr::Arrow(_, right) => right.conclusion(),
            other => other,
        }
    }

    /// Head symbol for inverted indexing (operator or leading ident).
    pub fn head_key(&self) -> String {
        match self.conclusion() {
            TypeExpr::BinOp { op, .. } => format!("op:{op}"),
            TypeExpr::UnaryOp { op, .. } => format!("uop:{op}"),
            TypeExpr::Postfix { op, .. } => format!("pop:{op}"),
            TypeExpr::Ident(s) => format!("id:{s}"),
            TypeExpr::App(f, _) => match f.as_ref() {
                TypeExpr::Ident(s) => format!("id:{s}"),
                TypeExpr::App(ff, _) => match ff.as_ref() {
                    TypeExpr::Ident(s) => format!("id:{s}"),
                    _ => "app".into(),
                },
                _ => "app".into(),
            },
            TypeExpr::Arrow(_, _) => "op:→".into(),
            TypeExpr::Forall { .. } => "op:∀".into(),
            TypeExpr::Exists { .. } => "op:∃".into(),
            TypeExpr::Sort { name, .. } => format!("sort:{name}"),
            TypeExpr::NatLit(_) => "lit:nat".into(),
            TypeExpr::Hole | TypeExpr::NamedHole(_) => "hole".into(),
            _ => "other".into(),
        }
    }

    /// Pretty-print a compact surface form (for display / cache).
    pub fn surface(&self) -> String {
        match self {
            TypeExpr::Hole => "_".into(),
            TypeExpr::NamedHole(n) => format!("?{n}"),
            TypeExpr::Ident(s) => s.clone(),
            TypeExpr::NatLit(n) => n.clone(),
            TypeExpr::Literal(s) => s.clone(),
            TypeExpr::App(f, a) => {
                let fa = f.surface();
                let aa = match a.as_ref() {
                    TypeExpr::App(_, _)
                    | TypeExpr::BinOp { .. }
                    | TypeExpr::Arrow(_, _)
                    | TypeExpr::Forall { .. }
                    | TypeExpr::Exists { .. }
                    | TypeExpr::Lambda { .. }
                    | TypeExpr::Pi { .. } => format!("({})", a.surface()),
                    _ => a.surface(),
                };
                format!("{fa} {aa}")
            }
            TypeExpr::BinOp { op, left, right } => {
                format!("({} {} {})", left.surface(), op, right.surface())
            }
            TypeExpr::UnaryOp { op, arg } => format!("{op}{}", arg.surface()),
            TypeExpr::Postfix { arg, op } => format!("{}{op}", arg.surface()),
            TypeExpr::Arrow(a, b) => format!("({} → {})", a.surface(), b.surface()),
            TypeExpr::Forall { binders, body } => {
                format!("(∀ {}, {})", format_binders(binders), body.surface())
            }
            TypeExpr::Exists { binders, body } => {
                format!("(∃ {}, {})", format_binders(binders), body.surface())
            }
            TypeExpr::Lambda { binders, body } => {
                format!("(fun {} => {})", format_binders(binders), body.surface())
            }
            TypeExpr::Pi { binder, body } => {
                format!("({} → {})", format_binder(binder), body.surface())
            }
            TypeExpr::Proj { base, field } => format!("{}.{}", base.surface(), field),
            TypeExpr::Sort { name, level } => match level {
                Some(l) => format!("{name} {}", l.surface()),
                None => name.clone(),
            },
            TypeExpr::Raw(s) => s.clone(),
        }
    }
}

fn format_binders(binders: &[Binder]) -> String {
    binders
        .iter()
        .map(format_binder)
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_binder(b: &Binder) -> String {
    let names = b.names.join(" ");
    let inner = match &b.ty {
        Some(ty) => format!("{names} : {}", ty.surface()),
        None => names,
    };
    match b.kind {
        BinderKind::Default => format!("({inner})"),
        BinderKind::Implicit => format!("{{{inner}}}"),
        BinderKind::Instance => format!("[{inner}]"),
        BinderKind::StrictImplicit => format!("⦃{inner}⦄"),
    }
}

/// A parsed declaration ready for indexing / XML export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Declaration {
    pub kind: DeclKind,
    pub name: String,
    /// Fully qualified-ish name if namespace known: `Nat.add_comm`.
    pub full_name: String,
    pub binders: Vec<Binder>,
    pub ty: TypeExpr,
    /// Original type surface text (as in source, trimmed).
    pub type_surface: String,
    pub file: String,
    pub line: usize,
    pub module: Option<String>,
    pub namespace_path: Vec<String>,
    pub attributes: Vec<String>,
}

impl Declaration {
    /// Type including explicit binders as Pi/forall-like arrow chain for matching.
    pub fn effective_type(&self) -> TypeExpr {
        if self.binders.is_empty() {
            return self.ty.clone();
        }
        // Represent leading binders as a Forall wrapping the stated type.
        TypeExpr::Forall {
            binders: self.binders.clone(),
            body: Box::new(self.ty.clone()),
        }
    }
}

/// One indexed Lean source package / lake root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub root: String,
    pub src_dirs: Vec<String>,
    pub lean_libs: Vec<String>,
}

/// Full in-memory / on-disk index document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexDocument {
    pub schema: String,
    pub created_at: String,
    pub packages: Vec<PackageInfo>,
    pub declarations: Vec<Declaration>,
    /// Source fingerprint for cache validation.
    pub source_hash: String,
}

impl IndexDocument {
    pub fn new() -> Self {
        Self {
            schema: RLEAN_NS.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            packages: Vec::new(),
            declarations: Vec::new(),
            source_hash: String::new(),
        }
    }
}
