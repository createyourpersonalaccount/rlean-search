//! Recursive-descent parser for a useful fragment of Lean 4 types and search patterns.

use crate::ast::{Binder, BinderKind, TypeExpr};
use crate::lexer::{Lexer, Token};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("unexpected token {found} (expected {expected})")]
    Unexpected {
        expected: String,
        found: String,
    },
    #[error("unexpected end of input while parsing {context}")]
    Eof { context: String },
    #[error("parse error: {0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;

/// A user search pattern, optionally restricted to the conclusion (`|-`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPattern {
    /// When true, match only against `TypeExpr::conclusion()`.
    pub conclusion_only: bool,
    pub expr: TypeExpr,
}

pub fn parse_type(input: &str) -> Result<TypeExpr> {
    let mut p = Parser::new(input);
    let t = p.parse_type()?;
    // Allow trailing junk lightly — but prefer clean parse
    Ok(t)
}

pub fn parse_search_pattern(input: &str) -> Result<SearchPattern> {
    let trimmed = input.trim();
    let (conclusion_only, rest) = if let Some(r) = trimmed.strip_prefix("|-") {
        (true, r.trim())
    } else if let Some(r) = trimmed.strip_prefix("⊢") {
        (true, r.trim())
    } else {
        (false, trimmed)
    };
    let expr = parse_type(rest)?;
    Ok(SearchPattern {
        conclusion_only,
        expr,
    })
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        let tokens = Lexer::tokenize(input);
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn bump(&mut self) -> Token {
        let t = self.peek().clone();
        if !matches!(t, Token::Eof) {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<()> {
        let found = self.bump();
        if &found == expected {
            Ok(())
        } else {
            Err(ParseError::Unexpected {
                expected: expected.to_string(),
                found: found.to_string(),
            })
        }
    }

    fn parse_type(&mut self) -> Result<TypeExpr> {
        self.parse_arrow()
    }

    /// Right-associative arrows: `A → B → C`
    fn parse_arrow(&mut self) -> Result<TypeExpr> {
        // Leading binder Pi: `(x : A) → B` or `{x : A} → B`
        if matches!(
            self.peek(),
            Token::LParen | Token::LBrace | Token::LBracket | Token::LStrict
        ) {
            // Could be parenthesized type OR binder then arrow. Lookahead.
            if let Some(binder) = self.try_parse_leading_pi_binder()? {
                if matches!(self.peek(), Token::Arrow) {
                    self.bump();
                    let body = self.parse_arrow()?;
                    return Ok(TypeExpr::Pi {
                        binder,
                        body: Box::new(body),
                    });
                }
                // Not an arrow — treat the binder group as a parenthesized/typed term is wrong.
                // Fall through: re-parse as atomic via backtrack-ish by reconstructing.
                // We already consumed the binder; if no arrow, interpret as the type inside
                // only when single default binder with type used as parenthesized expression —
                // actually `(A)` is not a binder. try_parse only succeeds for binders with names.
                // If we parsed `(x : A)` without arrow, it's still a term-ish raw.
                return Ok(TypeExpr::Raw(format_binder_raw(&binder_fallback_name(
                    &binder,
                ))));
            }
        }

        let left = self.parse_bin_expr(0)?;
        if matches!(self.peek(), Token::Arrow) {
            self.bump();
            let right = self.parse_arrow()?;
            Ok(TypeExpr::Arrow(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    /// Operator precedence climbing for infix operators.
    fn parse_bin_expr(&mut self, min_prec: u8) -> Result<TypeExpr> {
        let mut left = self.parse_app()?;

        loop {
            let op = match self.peek() {
                Token::Op(s) if is_infix_op(s) => s.clone(),
                // `⁻¹` as postfix handled in parse_app
                _ => break,
            };
            let prec = op_prec(&op);
            if prec < min_prec {
                break;
            }
            self.bump();
            // right-assoc for `^` and `→` (arrow handled elsewhere), `∘` sometimes
            let next_min = if is_right_assoc(&op) { prec } else { prec + 1 };
            let right = self.parse_bin_expr(next_min)?;
            left = TypeExpr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_app(&mut self) -> Result<TypeExpr> {
        // Prefix unary
        if let Token::Op(s) = self.peek() {
            if s == "¬" || s == "-" {
                let op = s.clone();
                self.bump();
                let arg = self.parse_app()?;
                return Ok(TypeExpr::UnaryOp {
                    op,
                    arg: Box::new(arg),
                });
            }
        }

        let mut expr = self.parse_atomic()?;

        // Postfix: ⁻¹ and projections .field
        loop {
            if let Token::Op(s) = self.peek() {
                if s == "⁻¹" || s == "'" {
                    let op = s.clone();
                    self.bump();
                    expr = TypeExpr::Postfix {
                        arg: Box::new(expr),
                        op,
                    };
                    continue;
                }
            }
            if matches!(self.peek(), Token::Dot) {
                self.bump();
                match self.bump() {
                    Token::Ident(f) => {
                        expr = TypeExpr::Proj {
                            base: Box::new(expr),
                            field: f,
                        };
                    }
                    Token::Nat(n) => {
                        expr = TypeExpr::Proj {
                            base: Box::new(expr),
                            field: n,
                        };
                    }
                    Token::Op(s) if s == ".." => {
                        // `x..` range sugar → raw-ish
                        expr = TypeExpr::Postfix {
                            arg: Box::new(expr),
                            op: "..".into(),
                        };
                    }
                    other => {
                        return Err(ParseError::Unexpected {
                            expected: "field name".into(),
                            found: other.to_string(),
                        });
                    }
                }
                continue;
            }
            break;
        }

        // Juxtaposition application
        while is_atomic_start(self.peek()) {
            // Don't consume infix ops as apps
            if let Token::Op(s) = self.peek() {
                if is_infix_op(s) {
                    break;
                }
            }
            if matches!(self.peek(), Token::Arrow | Token::Comma | Token::Assign) {
                break;
            }
            let arg = self.parse_atomic()?;
            // Allow postfix on arg already inside parse_atomic path — apply
            let mut arg = arg;
            loop {
                if let Token::Op(s) = self.peek() {
                    if s == "⁻¹" || s == "'" {
                        let op = s.clone();
                        self.bump();
                        arg = TypeExpr::Postfix {
                            arg: Box::new(arg),
                            op,
                        };
                        continue;
                    }
                }
                if matches!(self.peek(), Token::Dot) {
                    self.bump();
                    match self.bump() {
                        Token::Ident(f) => {
                            arg = TypeExpr::Proj {
                                base: Box::new(arg),
                                field: f,
                            };
                        }
                        Token::Nat(n) => {
                            arg = TypeExpr::Proj {
                                base: Box::new(arg),
                                field: n,
                            };
                        }
                        _ => break,
                    }
                    continue;
                }
                break;
            }
            expr = TypeExpr::App(Box::new(expr), Box::new(arg));
        }

        Ok(expr)
    }

    fn parse_atomic(&mut self) -> Result<TypeExpr> {
        match self.peek().clone() {
            Token::Underscore => {
                self.bump();
                Ok(TypeExpr::Hole)
            }
            Token::NamedHole(n) => {
                self.bump();
                Ok(TypeExpr::NamedHole(n))
            }
            Token::Nat(n) => {
                self.bump();
                Ok(TypeExpr::NatLit(n))
            }
            Token::Literal(s) => {
                self.bump();
                Ok(TypeExpr::Literal(s))
            }
            Token::Ident(s) => {
                self.bump();
                // Sort / Type / Prop with optional level
                if s == "Prop" {
                    return Ok(TypeExpr::Sort {
                        name: "Prop".into(),
                        level: None,
                    });
                }
                if s == "Type" || s == "Sort" {
                    let level = if is_atomic_start(self.peek())
                        && !matches!(self.peek(), Token::Op(_))
                    {
                        // Type u / Type _ / Type*
                        if matches!(self.peek(), Token::Op(op) if op == "*") {
                            self.bump();
                            Some(Box::new(TypeExpr::Ident("*".into())))
                        } else if matches!(
                            self.peek(),
                            Token::Ident(_)
                                | Token::Underscore
                                | Token::NamedHole(_)
                                | Token::Nat(_)
                                | Token::LParen
                        ) {
                            // Only take a simple level atom, not a full app chain of term
                            match self.peek() {
                                Token::Ident(_)
                                | Token::Underscore
                                | Token::NamedHole(_)
                                | Token::Nat(_) => Some(Box::new(self.parse_level_atom()?)),
                                Token::LParen => Some(Box::new(self.parse_atomic()?)),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    return Ok(TypeExpr::Sort { name: s, level });
                }
                Ok(TypeExpr::Ident(s))
            }
            Token::Forall => {
                self.bump();
                self.parse_quantifier(true)
            }
            Token::Exists => {
                self.bump();
                self.parse_quantifier(false)
            }
            Token::Fun => {
                self.bump();
                let binders = self.parse_binder_list_until_mapsto()?;
                if matches!(self.peek(), Token::MapsTo) {
                    self.bump();
                } else if matches!(self.peek(), Token::Arrow) {
                    // fun x → body sometimes
                    self.bump();
                } else {
                    // fun x => may use comma? rare
                }
                let body = self.parse_type()?;
                Ok(TypeExpr::Lambda {
                    binders,
                    body: Box::new(body),
                })
            }
            Token::LParen => {
                self.bump();
                // Empty `()`
                if matches!(self.peek(), Token::RParen) {
                    self.bump();
                    return Ok(TypeExpr::Raw("()".into()));
                }
                // Grouping / type ascription `(e)` or `(e : T)` or `(e : T) → ...` handled higher
                let inner = self.parse_type()?;
                // Type ascription: `(1 : M)`, `(Inv.inv : G → G)`
                let inner = if matches!(self.peek(), Token::Colon) {
                    self.bump();
                    let ty = self.parse_type()?;
                    TypeExpr::App(
                        Box::new(TypeExpr::App(
                            Box::new(TypeExpr::Ident("ascribe".into())),
                            Box::new(inner),
                        )),
                        Box::new(ty),
                    )
                } else {
                    inner
                };
                if matches!(self.peek(), Token::Comma) {
                    let mut parts = vec![inner];
                    while matches!(self.peek(), Token::Comma) {
                        self.bump();
                        parts.push(self.parse_type()?);
                    }
                    self.expect(&Token::RParen)?;
                    let s = parts
                        .iter()
                        .map(|p| p.surface())
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Ok(TypeExpr::Raw(format!("({s})")));
                }
                self.expect(&Token::RParen)?;
                Ok(inner)
            }
            Token::LBrace => {
                // Implicit binder as term is rare; parse as binder group raw or type
                let binders = self.parse_brace_binder_group(BinderKind::Implicit)?;
                Ok(TypeExpr::Raw(format_binders_surface(&binders)))
            }
            Token::LBracket => {
                // Empty list `[]` or instance-like `[Group α]`
                self.bump();
                if matches!(self.peek(), Token::RBracket) {
                    self.bump();
                    return Ok(TypeExpr::Ident("[]".into()));
                }
                let inner = self.parse_type()?;
                // optional ascription inside brackets rare
                let inner = if matches!(self.peek(), Token::Colon) {
                    self.bump();
                    let ty = self.parse_type()?;
                    TypeExpr::App(
                        Box::new(TypeExpr::App(
                            Box::new(TypeExpr::Ident("ascribe".into())),
                            Box::new(inner),
                        )),
                        Box::new(ty),
                    )
                } else {
                    inner
                };
                self.expect(&Token::RBracket)?;
                Ok(TypeExpr::App(
                    Box::new(TypeExpr::Ident("inst".into())),
                    Box::new(inner),
                ))
            }
            Token::Pipe => {
                // Absolute value `|e|`
                self.bump();
                let inner = self.parse_bin_expr(0)?;
                if matches!(self.peek(), Token::Pipe) {
                    self.bump();
                    Ok(TypeExpr::App(
                        Box::new(TypeExpr::Ident("abs".into())),
                        Box::new(inner),
                    ))
                } else {
                    Ok(TypeExpr::UnaryOp {
                        op: "|".into(),
                        arg: Box::new(inner),
                    })
                }
            }
            Token::LStrict => {
                let binders = self.parse_brace_binder_group(BinderKind::StrictImplicit)?;
                Ok(TypeExpr::Raw(format_binders_surface(&binders)))
            }
            Token::Op(s) if s == "∑" || s == "∏" || s == "∫" => {
                // Big operators: keep as unary-ish application chain
                let op = s.clone();
                self.bump();
                // optional binder `(i ∈ s)` etc.
                let arg = if is_atomic_start(self.peek()) {
                    self.parse_app()?
                } else {
                    TypeExpr::Hole
                };
                Ok(TypeExpr::UnaryOp {
                    op,
                    arg: Box::new(arg),
                })
            }
            other => Err(ParseError::Unexpected {
                expected: "type expression".into(),
                found: other.to_string(),
            }),
        }
    }

    fn parse_level_atom(&mut self) -> Result<TypeExpr> {
        match self.bump() {
            Token::Ident(s) => Ok(TypeExpr::Ident(s)),
            Token::Nat(n) => Ok(TypeExpr::NatLit(n)),
            Token::Underscore => Ok(TypeExpr::Hole),
            Token::NamedHole(n) => Ok(TypeExpr::NamedHole(n)),
            other => Err(ParseError::Unexpected {
                expected: "universe level".into(),
                found: other.to_string(),
            }),
        }
    }

    fn parse_quantifier(&mut self, is_forall: bool) -> Result<TypeExpr> {
        let binders = self.parse_quantifier_binders()?;
        // optional comma
        if matches!(self.peek(), Token::Comma) {
            self.bump();
        }
        let body = self.parse_type()?;
        if is_forall {
            Ok(TypeExpr::Forall {
                binders,
                body: Box::new(body),
            })
        } else {
            Ok(TypeExpr::Exists {
                binders,
                body: Box::new(body),
            })
        }
    }

    fn parse_quantifier_binders(&mut self) -> Result<Vec<Binder>> {
        let mut binders = Vec::new();
        // ∀ x y : Nat, ...  or ∀ (x : Nat) (y : Nat), ... or ∀ x, ...
        loop {
            match self.peek() {
                Token::LParen | Token::LBrace | Token::LBracket | Token::LStrict => {
                    binders.push(self.parse_one_binder_group()?);
                }
                Token::Ident(_) | Token::Underscore => {
                    // bare names until `:` or `,`
                    let mut names = Vec::new();
                    while matches!(self.peek(), Token::Ident(_) | Token::Underscore) {
                        match self.bump() {
                            Token::Ident(n) => names.push(n),
                            Token::Underscore => names.push("_".into()),
                            _ => unreachable!(),
                        }
                        // stop if next would start body wrongly — if next is Op or known end
                        if matches!(
                            self.peek(),
                            Token::Colon
                                | Token::Comma
                                | Token::LParen
                                | Token::LBrace
                                | Token::LBracket
                                | Token::LStrict
                        ) {
                            break;
                        }
                        // also stop before another quantifier body keyword-less: if Op infix, it's body start without comma — rare
                        if matches!(self.peek(), Token::Op(_) | Token::Arrow) {
                            break;
                        }
                    }
                    let ty = if matches!(self.peek(), Token::Colon) {
                        self.bump();
                        Some(Box::new(self.parse_bin_expr(0)?))
                    } else {
                        None
                    };
                    binders.push(Binder {
                        kind: BinderKind::Default,
                        names,
                        ty,
                    });
                    // if next is another binder group continue; if comma break to body
                    if matches!(self.peek(), Token::Comma) {
                        break;
                    }
                    if !matches!(
                        self.peek(),
                        Token::LParen
                            | Token::LBrace
                            | Token::LBracket
                            | Token::LStrict
                            | Token::Ident(_)
                            | Token::Underscore
                    ) {
                        break;
                    }
                }
                Token::Comma => break,
                _ => break,
            }
            if matches!(self.peek(), Token::Comma) {
                break;
            }
        }
        if binders.is_empty() {
            return Err(ParseError::Message(
                "expected binders after quantifier".into(),
            ));
        }
        Ok(binders)
    }

    fn parse_binder_list_until_mapsto(&mut self) -> Result<Vec<Binder>> {
        let mut binders = Vec::new();
        while !matches!(
            self.peek(),
            Token::MapsTo | Token::Arrow | Token::Eof | Token::Comma
        ) {
            if matches!(
                self.peek(),
                Token::LParen | Token::LBrace | Token::LBracket | Token::LStrict
            ) {
                binders.push(self.parse_one_binder_group()?);
            } else if matches!(self.peek(), Token::Ident(_) | Token::Underscore) {
                let mut names = Vec::new();
                while matches!(self.peek(), Token::Ident(_) | Token::Underscore) {
                    match self.bump() {
                        Token::Ident(n) => names.push(n),
                        Token::Underscore => names.push("_".into()),
                        _ => unreachable!(),
                    }
                    if matches!(self.peek(), Token::Colon) {
                        break;
                    }
                    if matches!(self.peek(), Token::MapsTo | Token::Arrow) {
                        break;
                    }
                    if matches!(
                        self.peek(),
                        Token::LParen | Token::LBrace | Token::LBracket | Token::LStrict
                    ) {
                        break;
                    }
                }
                let ty = if matches!(self.peek(), Token::Colon) {
                    self.bump();
                    Some(Box::new(self.parse_bin_expr(0)?))
                } else {
                    None
                };
                binders.push(Binder {
                    kind: BinderKind::Default,
                    names,
                    ty,
                });
            } else {
                break;
            }
        }
        Ok(binders)
    }

    fn parse_one_binder_group(&mut self) -> Result<Binder> {
        match self.peek() {
            Token::LParen => self.parse_paren_binder(BinderKind::Default),
            Token::LBrace => self.parse_brace_binder_group(BinderKind::Implicit).map(|mut v| {
                v.pop().unwrap_or(Binder {
                    kind: BinderKind::Implicit,
                    names: vec!["_".into()],
                    ty: None,
                })
            }),
            Token::LBracket => self.parse_brace_binder_group(BinderKind::Instance).map(|mut v| {
                v.pop().unwrap_or(Binder {
                    kind: BinderKind::Instance,
                    names: vec!["_".into()],
                    ty: None,
                })
            }),
            Token::LStrict => self
                .parse_brace_binder_group(BinderKind::StrictImplicit)
                .map(|mut v| {
                    v.pop().unwrap_or(Binder {
                        kind: BinderKind::StrictImplicit,
                        names: vec!["_".into()],
                        ty: None,
                    })
                }),
            _ => Err(ParseError::Message("expected binder group".into())),
        }
    }

    fn parse_paren_binder(&mut self, kind: BinderKind) -> Result<Binder> {
        self.expect(&Token::LParen)?;
        // `(x y : T)` or `(x)` or `(_ : T)`
        let mut names = Vec::new();
        while matches!(self.peek(), Token::Ident(_) | Token::Underscore) {
            match self.bump() {
                Token::Ident(n) => names.push(n),
                Token::Underscore => names.push("_".into()),
                _ => unreachable!(),
            }
            if matches!(self.peek(), Token::Colon | Token::RParen) {
                break;
            }
        }
        let ty = if matches!(self.peek(), Token::Colon) {
            self.bump();
            Some(Box::new(self.parse_type()?))
        } else {
            None
        };
        self.expect(&Token::RParen)?;
        if names.is_empty() {
            // `(T)` was not a binder — but we already consumed. Represent type-only.
            if let Some(t) = ty {
                return Ok(Binder {
                    kind,
                    names: vec!["_".into()],
                    ty: Some(t),
                });
            }
        }
        Ok(Binder { kind, names, ty })
    }

    fn parse_brace_binder_group(&mut self, kind: BinderKind) -> Result<Vec<Binder>> {
        let (open, close) = match kind {
            BinderKind::Implicit => (Token::LBrace, Token::RBrace),
            BinderKind::Instance => (Token::LBracket, Token::RBracket),
            BinderKind::StrictImplicit => (Token::LStrict, Token::RStrict),
            BinderKind::Default => (Token::LParen, Token::RParen),
        };
        self.expect(&open)?;
        let mut binders = Vec::new();
        // instance `[Group G]` may have no names with colon: Ident+ as type
        // Try names : type, or just type
        let mut names = Vec::new();
        let start_pos = self.pos;
        while matches!(self.peek(), Token::Ident(_) | Token::Underscore) {
            match self.bump() {
                Token::Ident(n) => names.push(n),
                Token::Underscore => names.push("_".into()),
                _ => unreachable!(),
            }
            if matches!(self.peek(), Token::Colon) {
                break;
            }
            // could be `Group G` type application without names
            if matches!(
                self.peek(),
                Token::RBrace | Token::RBracket | Token::RStrict | Token::RParen
            ) {
                break;
            }
            // continue collecting — ambiguous
        }
        if matches!(self.peek(), Token::Colon) {
            self.bump();
            let ty = self.parse_type()?;
            binders.push(Binder {
                kind,
                names,
                ty: Some(Box::new(ty)),
            });
        } else {
            // Rewind-ish: treat everything as type expression
            // If we collected names without colon, it's `Group α` type
            self.pos = start_pos;
            let ty = self.parse_type()?;
            binders.push(Binder {
                kind,
                names: vec![],
                ty: Some(Box::new(ty)),
            });
        }
        // optional more binder groups inside same braces rare
        self.expect(&close)?;
        Ok(binders)
    }

    /// Try to parse `(x : T)` as a Pi binder; returns None if it looks like grouping.
    fn try_parse_leading_pi_binder(&mut self) -> Result<Option<Binder>> {
        let save = self.pos;
        let kind = match self.peek() {
            Token::LParen => BinderKind::Default,
            Token::LBrace => BinderKind::Implicit,
            Token::LBracket => BinderKind::Instance,
            Token::LStrict => BinderKind::StrictImplicit,
            _ => return Ok(None),
        };
        // Need: open, name+, colon, type, close, arrow
        let open_tok = self.bump();
        let mut names = Vec::new();
        while matches!(self.peek(), Token::Ident(_) | Token::Underscore) {
            match self.bump() {
                Token::Ident(n) => names.push(n),
                Token::Underscore => names.push("_".into()),
                _ => unreachable!(),
            }
            if matches!(self.peek(), Token::Colon) {
                break;
            }
            // if we see something else before colon and only one "name", might be `(Nat)` grouping
            if !matches!(self.peek(), Token::Ident(_) | Token::Underscore | Token::Colon)
            {
                break;
            }
        }
        if names.is_empty() || !matches!(self.peek(), Token::Colon) {
            self.pos = save;
            return Ok(None);
        }
        self.bump(); // colon
        let ty = match self.parse_type() {
            Ok(t) => t,
            Err(_) => {
                self.pos = save;
                return Ok(None);
            }
        };
        let close_ok = match kind {
            BinderKind::Default => matches!(self.peek(), Token::RParen),
            BinderKind::Implicit => matches!(self.peek(), Token::RBrace),
            BinderKind::Instance => matches!(self.peek(), Token::RBracket),
            BinderKind::StrictImplicit => matches!(self.peek(), Token::RStrict),
        };
        if !close_ok {
            self.pos = save;
            return Ok(None);
        }
        self.bump();
        // Only treat as binder if arrow follows OR we already know it's binder form
        if matches!(self.peek(), Token::Arrow) {
            Ok(Some(Binder {
                kind,
                names,
                ty: Some(Box::new(ty)),
            }))
        } else {
            // `(x : T)` alone isn't a type; restore
            let _ = open_tok;
            self.pos = save;
            Ok(None)
        }
    }
}

fn is_atomic_start(tok: &Token) -> bool {
    match tok {
        Token::Ident(_)
        | Token::Nat(_)
        | Token::Literal(_)
        | Token::Underscore
        | Token::NamedHole(_)
        | Token::LParen
        | Token::LBrace
        | Token::LBracket
        | Token::LStrict
        | Token::Forall
        | Token::Exists
        | Token::Fun
        | Token::Pipe => true,
        Token::Op(s) => s == "∑" || s == "∏" || s == "∫" || s == "¬" || s == "-",
        _ => false,
    }
}

fn is_infix_op(s: &str) -> bool {
    matches!(
        s,
        "=" | "≠"
            | "<"
            | ">"
            | "≤"
            | "≥"
            | "+"
            | "-"
            | "*"
            | "/"
            | "%"
            | "^"
            | "∧"
            | "∨"
            | "↔"
            | "∘"
            | "∈"
            | "∉"
            | "⊆"
            | "⊂"
            | "∪"
            | "∩"
            | "++"
            | "::"
            | "|>"
            | "<|"
            | "|>."
            | "$"
            | "≈"
            | "≃"
            | "≅"
            | "≡"
            | "⋅"
            | "•"
            | "⋆"
            | "▸"
            | "∥"
            | "∣"
            | "→"
    ) || s == "→"
}

fn op_prec(op: &str) -> u8 {
    match op {
        "$" | "<|" | "|>" | "|>." => 1,
        "↔" => 2,
        "∨" => 3,
        "∧" => 4,
        "=" | "≠" | "<" | ">" | "≤" | "≥" | "∈" | "∉" | "⊆" | "⊂" | "≈" | "≃" | "≅" | "≡"
        | "∥" | "∣" => 5,
        "∪" | "++" => 6,
        "∩" => 7,
        "::" => 8,
        "+" | "-" => 9,
        "*" | "/" | "%" | "⋅" | "•" | "⋆" => 10,
        "∘" => 11,
        "^" => 12,
        "▸" => 13,
        _ => 5,
    }
}

fn is_right_assoc(op: &str) -> bool {
    matches!(op, "^" | "↔" | "::" | "$" | "∘")
}

fn format_binders_surface(binders: &[Binder]) -> String {
    binders
        .iter()
        .map(|b| {
            let names = b.names.join(" ");
            let core = match &b.ty {
                Some(t) if names.is_empty() => t.surface(),
                Some(t) => format!("{names} : {}", t.surface()),
                None => names,
            };
            match b.kind {
                BinderKind::Default => format!("({core})"),
                BinderKind::Implicit => format!("{{{core}}}"),
                BinderKind::Instance => format!("[{core}]"),
                BinderKind::StrictImplicit => format!("⦃{core}⦄"),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_binder_raw(s: &str) -> String {
    s.to_string()
}

fn binder_fallback_name(b: &Binder) -> String {
    format_binders_surface(std::slice::from_ref(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_eq_add() {
        let t = parse_type("n + m = m + n").unwrap();
        assert!(matches!(&t, TypeExpr::BinOp { op, .. } if op == "="));
        assert_eq!(t.conclusion().head_key(), "op:=");
    }

    #[test]
    fn parse_forall() {
        let t = parse_type("∀ (n m : Nat), n + m = m + n").unwrap();
        match t {
            TypeExpr::Forall { binders, body } => {
                assert_eq!(binders[0].names, vec!["n", "m"]);
                assert!(matches!(body.as_ref(), TypeExpr::BinOp { op, .. } if op == "="));
            }
            _ => panic!("expected forall"),
        }
    }

    #[test]
    fn parse_arrow_chain() {
        let t = parse_type("Nat → Nat → Prop").unwrap();
        match t {
            TypeExpr::Arrow(a, b) => {
                assert!(matches!(a.as_ref(), TypeExpr::Ident(s) if s == "Nat"));
                assert!(matches!(b.as_ref(), TypeExpr::Arrow(_, _)));
            }
            _ => panic!("expected arrow"),
        }
    }

    #[test]
    fn parse_pi_binder() {
        let t = parse_type("(n : Nat) → n + 0 = n").unwrap();
        assert!(matches!(t, TypeExpr::Pi { .. }));
    }

    #[test]
    fn parse_holes_and_named() {
        let t = parse_type("?a - ?a = 0").unwrap();
        match t {
            TypeExpr::BinOp { op, left, right } if op == "=" => {
                assert!(matches!(right.as_ref(), TypeExpr::NatLit(n) if n == "0"));
                match left.as_ref() {
                    TypeExpr::BinOp { op, left, right } if op == "-" => {
                        assert!(matches!(left.as_ref(), TypeExpr::NamedHole(a) if a == "a"));
                        assert!(matches!(right.as_ref(), TypeExpr::NamedHole(a) if a == "a"));
                    }
                    _ => panic!("expected subtraction"),
                }
            }
            _ => panic!("expected eq"),
        }
    }

    #[test]
    fn parse_search_turnstile() {
        let p = parse_search_pattern("|- tsum _ = _ * tsum _").unwrap();
        assert!(p.conclusion_only);
        assert!(matches!(p.expr, TypeExpr::BinOp { op, .. } if op == "="));
    }

    #[test]
    fn parse_iff_and() {
        let t = parse_type("p ∧ q ↔ q ∧ p").unwrap();
        assert!(matches!(t, TypeExpr::BinOp { op, .. } if op == "↔"));
    }
}
