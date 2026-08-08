//! Lexer for a useful fragment of Lean 4 type / declaration surface syntax.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Ident(String),
    Nat(String),
    /// String or character literal, including quotes.
    Literal(String),
    /// `_`
    Underscore,
    /// `?name`
    NamedHole(String),
    /// Keywords / symbols
    Theorem,
    Lemma,
    Axiom,
    Forall,
    Exists,
    Fun,
    /// `:`
    Colon,
    /// `:=`
    Assign,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `→` or `->`
    Arrow,
    /// `=>` or `↦`
    MapsTo,
    /// `|` (pattern / match; also conclusion search prefix before `-`)
    Pipe,
    /// `|-` conclusion-search marker (produced by lexer when seeing `|` `-`)
    Turnstile,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    /// `⦃`
    LStrict,
    /// `⦄`
    RStrict,
    /// Binary / unary operators and other symbol tokens
    Op(String),
    /// End of input
    Eof,
}

impl Token {
    pub fn is_ident_like(&self) -> bool {
        matches!(
            self,
            Token::Ident(_) | Token::Underscore | Token::NamedHole(_) | Token::Nat(_)
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{s}"),
            Token::Nat(s) => write!(f, "{s}"),
            Token::Literal(s) => write!(f, "{s}"),
            Token::Underscore => write!(f, "_"),
            Token::NamedHole(s) => write!(f, "?{s}"),
            Token::Theorem => write!(f, "theorem"),
            Token::Lemma => write!(f, "lemma"),
            Token::Axiom => write!(f, "axiom"),
            Token::Forall => write!(f, "∀"),
            Token::Exists => write!(f, "∃"),
            Token::Fun => write!(f, "fun"),
            Token::Colon => write!(f, ":"),
            Token::Assign => write!(f, ":="),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Arrow => write!(f, "→"),
            Token::MapsTo => write!(f, "=>"),
            Token::Pipe => write!(f, "|"),
            Token::Turnstile => write!(f, "|-"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LStrict => write!(f, "⦃"),
            Token::RStrict => write!(f, "⦄"),
            Token::Op(s) => write!(f, "{s}"),
            Token::Eof => write!(f, "<eof>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        }
    }

    pub fn remaining(&self) -> &'a str {
        &self.src[self.pos..]
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    fn peek_char(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn starts_with(&self, s: &str) -> bool {
        self.src[self.pos..].starts_with(s)
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while let Some(c) = self.peek_char() {
                if c.is_whitespace() {
                    self.bump();
                } else {
                    break;
                }
            }
            if self.starts_with("--") {
                while let Some(c) = self.bump() {
                    if c == '\n' {
                        break;
                    }
                }
                continue;
            }
            if self.starts_with("/-") {
                self.pos += 2;
                let mut depth = 1;
                while depth > 0 && self.pos < self.bytes.len() {
                    if self.starts_with("/-") {
                        self.pos += 2;
                        depth += 1;
                    } else if self.starts_with("-/") {
                        self.pos += 2;
                        depth -= 1;
                    } else {
                        let ch = self.peek_char().unwrap();
                        self.pos += ch.len_utf8();
                    }
                }
                continue;
            }
            break;
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws_and_comments();
        if self.pos >= self.bytes.len() {
            return Token::Eof;
        }

        // Multi-char unicode / ascii operators first
        let multi = [
            ("|-", Token::Turnstile),
            (":=", Token::Assign),
            ("->", Token::Arrow),
            ("→", Token::Arrow),
            ("=>", Token::MapsTo),
            ("↦", Token::MapsTo),
            ("∀", Token::Forall),
            ("∃", Token::Exists),
            ("λ", Token::Fun),
            ("⦃", Token::LStrict),
            ("⦄", Token::RStrict),
            ("≠", Token::Op("≠".into())),
            ("≤", Token::Op("≤".into())),
            ("≥", Token::Op("≥".into())),
            ("<", Token::Op("<".into())),
            (">", Token::Op(">".into())),
            ("↔", Token::Op("↔".into())),
            ("∧", Token::Op("∧".into())),
            ("∨", Token::Op("∨".into())),
            ("¬", Token::Op("¬".into())),
            ("∘", Token::Op("∘".into())),
            ("⁻¹", Token::Op("⁻¹".into())),
            ("∈", Token::Op("∈".into())),
            ("∉", Token::Op("∉".into())),
            ("⊆", Token::Op("⊆".into())),
            ("⊂", Token::Op("⊂".into())),
            ("∪", Token::Op("∪".into())),
            ("∩", Token::Op("∩".into())),
            ("∑", Token::Op("∑".into())),
            ("∏", Token::Op("∏".into())),
            ("∫", Token::Op("∫".into())),
            ("∥", Token::Op("∥".into())),
            ("≈", Token::Op("≈".into())),
            ("≃", Token::Op("≃".into())),
            ("≅", Token::Op("≅".into())),
            ("≡", Token::Op("≡".into())),
            ("⋅", Token::Op("⋅".into())),
            ("•", Token::Op("•".into())),
            ("⋆", Token::Op("⋆".into())),
            ("·", Token::Ident("·".into())),
            ("∣", Token::Op("∣".into())),
            ("ℕ", Token::Ident("ℕ".into())),
            ("ℤ", Token::Ident("ℤ".into())),
            ("ℚ", Token::Ident("ℚ".into())),
            ("ℝ", Token::Ident("ℝ".into())),
            ("ℂ", Token::Ident("ℂ".into())),
            ("▸", Token::Op("▸".into())),
            ("|>.", Token::Op("|>.".into())),
            ("|>", Token::Op("|>".into())),
            ("<|", Token::Op("<|".into())),
            (">>", Token::Op(">>".into())),
            ("<<", Token::Op("<<".into())),
            ("++", Token::Op("++".into())),
            ("::", Token::Op("::".into())),
            ("..", Token::Op("..".into())),
            ("$", Token::Op("$".into())),
            ("@", Token::Op("@".into())),
            ("^", Token::Op("^".into())),
            ("+", Token::Op("+".into())),
            ("-", Token::Op("-".into())),
            ("*", Token::Op("*".into())),
            ("/", Token::Op("/".into())),
            ("%", Token::Op("%".into())),
            ("=", Token::Op("=".into())),
            ("|", Token::Pipe),
        ];
        for (s, tok) in multi {
            if self.starts_with(s) {
                // Don't treat single `-` as op when part of `|-` already handled.
                self.pos += s.len();
                return tok.clone();
            }
        }

        let ch = self.peek_char().unwrap();

        match ch {
            '(' => {
                self.bump();
                Token::LParen
            }
            ')' => {
                self.bump();
                Token::RParen
            }
            '{' => {
                self.bump();
                Token::LBrace
            }
            '}' => {
                self.bump();
                Token::RBrace
            }
            '[' => {
                self.bump();
                Token::LBracket
            }
            ']' => {
                self.bump();
                Token::RBracket
            }
            ':' => {
                self.bump();
                Token::Colon
            }
            ',' => {
                self.bump();
                Token::Comma
            }
            '.' => {
                self.bump();
                // field / number projection handled by parser; keep Dot
                Token::Dot
            }
            '_' => {
                self.bump();
                Token::Underscore
            }
            '?' => {
                self.bump();
                let name = self.lex_ident_tail();
                if name.is_empty() {
                    Token::Op("?".into())
                } else {
                    Token::NamedHole(name)
                }
            }
            '"' => self.lex_string(),
            '\'' => self.lex_char_or_prime(),
            '«' => self.lex_escaped_ident(),
            c if c.is_ascii_digit() => self.lex_number(),
            c if is_ident_start(c) => {
                let id = self.lex_ident();
                match id.as_str() {
                    "theorem" => Token::Theorem,
                    "lemma" => Token::Lemma,
                    "axiom" => Token::Axiom,
                    "forall" => Token::Forall,
                    "exists" => Token::Exists,
                    "fun" | "λ" => Token::Fun,
                    "Prop" | "Type" | "Sort" => Token::Ident(id),
                    _ => Token::Ident(id),
                }
            }
            _ => {
                // Unknown symbol as operator token
                let start = self.pos;
                self.bump();
                // glue common multi-byte leftover
                Token::Op(self.src[start..self.pos].to_string())
            }
        }
    }

    fn lex_ident_tail(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if is_ident_continue(c) {
                self.bump();
            } else {
                break;
            }
        }
        self.src[start..self.pos].to_string()
    }

    fn lex_ident(&mut self) -> String {
        let start = self.pos;
        if let Some(c) = self.peek_char() {
            if is_ident_start(c) {
                self.bump();
            }
        }
        while let Some(c) = self.peek_char() {
            if is_ident_continue(c) {
                self.bump();
            } else {
                break;
            }
        }
        // trailing `?` sometimes used; keep primes as part of name
        self.src[start..self.pos].to_string()
    }

    fn lex_escaped_ident(&mut self) -> Token {
        // «name with spaces»
        self.bump(); // «
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c == '»' {
                let name = self.src[start..self.pos].to_string();
                self.bump();
                return Token::Ident(name);
            }
            self.bump();
        }
        Token::Ident(self.src[start..self.pos].to_string())
    }

    fn lex_number(&mut self) -> Token {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.bump();
            } else {
                break;
            }
        }
        // scientific / decimal rare in types; keep integer token
        Token::Nat(self.src[start..self.pos].to_string())
    }

    fn lex_string(&mut self) -> Token {
        let start = self.pos;
        self.bump(); // "
        while let Some(c) = self.peek_char() {
            if c == '\\' {
                self.bump();
                self.bump();
                continue;
            }
            if c == '"' {
                self.bump();
                break;
            }
            self.bump();
        }
        Token::Literal(self.src[start..self.pos].to_string())
    }

    fn lex_char_or_prime(&mut self) -> Token {
        // Could be 'a' char literal or trailing prime on previous token — as standalone, treat as op/prime
        let start = self.pos;
        self.bump();
        if let Some(c) = self.peek_char() {
            if c != '\'' && c != '\\' {
                // likely prime operator suffix used alone
                return Token::Op("'".into());
            }
            if c == '\\' {
                self.bump();
                self.bump();
            } else {
                self.bump();
            }
            if self.peek_char() == Some('\'') {
                self.bump();
            }
            return Token::Literal(self.src[start..self.pos].to_string());
        }
        Token::Op("'".into())
    }

    /// Tokenize entire input into a vector (excluding trailing Eof unless empty).
    pub fn tokenize(src: &str) -> Vec<Token> {
        let mut lx = Lexer::new(src);
        let mut toks = Vec::new();
        loop {
            let t = lx.next_token();
            if t == Token::Eof {
                toks.push(t);
                break;
            }
            toks.push(t);
        }
        toks
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == 'ℕ' || c == 'ℤ' || c == 'ℚ' || c == 'ℝ' || c == 'ℂ'
        || c == 'α' || c == 'β' || c == 'γ' || c == 'δ' || c == 'ε' || c == 'ζ' || c == 'η'
        || c == 'θ' || c == 'ι' || c == 'κ' || c == 'λ' || c == 'μ' || c == 'ν' || c == 'ξ'
        || c == 'π' || c == 'ρ' || c == 'σ' || c == 'τ' || c == 'υ' || c == 'φ' || c == 'χ'
        || c == 'ψ' || c == 'ω' || c == 'Γ' || c == 'Δ' || c == 'Θ' || c == 'Λ' || c == 'Ξ'
        || c == 'Π' || c == 'Σ' || c == 'Φ' || c == 'Ψ' || c == 'Ω' || c == '𝒜' || c == 'ℳ'
        || ('\u{0370}'..='\u{03FF}').contains(&c) // Greek
        || ('\u{1D400}'..='\u{1D7FF}').contains(&c) // math alphanumerics
}

fn is_ident_continue(c: char) -> bool {
    is_ident_start(c)
        || c.is_ascii_digit()
        || c == '\''
        || c == '?'
        || c == '!'
        || c == '₀'
        || c == '₁'
        || c == '₂'
        || c == '₃'
        || c == '₄'
        || c == '₅'
        || c == '₆'
        || c == '₇'
        || c == '₈'
        || c == '₉'
        || c == 'ₙ'
        || c == 'ₘ'
        || c == 'ᵢ'
        || c == 'ⱼ'
        || c == 'ₖ'
        || ('\u{2080}'..='\u{209F}').contains(&c) // subscripts
        || ('\u{2070}'..='\u{209F}').contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_simple_eq() {
        let toks = Lexer::tokenize("n + m = m + n");
        assert!(toks.iter().any(|t| matches!(t, Token::Op(s) if s == "+")));
        assert!(toks.iter().any(|t| matches!(t, Token::Op(s) if s == "=")));
    }

    #[test]
    fn lex_holes() {
        let toks = Lexer::tokenize("_ + ?a = 0");
        assert_eq!(toks[0], Token::Underscore);
        assert!(matches!(&toks[2], Token::NamedHole(s) if s == "a"));
    }

    #[test]
    fn lex_turnstile() {
        let toks = Lexer::tokenize("|- tsum _ = _");
        assert_eq!(toks[0], Token::Turnstile);
    }
}
