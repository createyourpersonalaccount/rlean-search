//! Extract theorem / lemma / axiom declarations from Lean 4 source files.

use crate::ast::{Binder, BinderKind, DeclKind, Declaration, TypeExpr};
use crate::lexer::{Lexer, Token};
use crate::parser::type_expr::{parse_type, ParseError};
use std::path::Path;

/// Parse all theorem/lemma/axiom declarations from a `.lean` source string.
pub fn parse_declarations(source: &str, file: &str) -> Vec<Declaration> {
    parse_declarations_with_path(source, file, None)
}

pub fn parse_declarations_with_path(
    source: &str,
    file: &str,
    module_hint: Option<&str>,
) -> Vec<Declaration> {
    let mut decls = Vec::new();
    let mut namespace_stack: Vec<String> = Vec::new();
    let mut section_depth: usize = 0;

    // Strip / track namespaces with a simple line scan; extract decls with a more careful scan.
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0usize;

    while i < lines.len() {
        let raw = lines[i];
        let line = strip_line_comment(raw).trim();

        if line.is_empty() {
            i += 1;
            continue;
        }

        // Block comment only lines — skip start; naive skip until -/
        if line.starts_with("/-") && !line.contains("-/") {
            i += 1;
            while i < lines.len() && !lines[i].contains("-/") {
                i += 1;
            }
            i += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("namespace ") {
            let name = rest.split_whitespace().next().unwrap_or("").trim();
            if !name.is_empty() && name != "_root_" {
                for part in name.split('.') {
                    if !part.is_empty() {
                        namespace_stack.push(part.to_string());
                    }
                }
            }
            i += 1;
            continue;
        }
        if line == "end" || line.starts_with("end ") {
            // end Namespace / end Section — pop if matches last ns, else section
            let name = line.strip_prefix("end").unwrap_or("").trim();
            if name.is_empty() {
                if section_depth > 0 {
                    section_depth -= 1;
                } else if !namespace_stack.is_empty() {
                    namespace_stack.pop();
                }
            } else if namespace_stack.last().map(|s| s.as_str()) == Some(name)
                || namespace_stack
                    .iter()
                    .rev()
                    .take(name.matches('.').count() + 1)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join(".")
                    == name
            {
                let parts = name.split('.').filter(|p| !p.is_empty()).count();
                for _ in 0..parts.max(1) {
                    namespace_stack.pop();
                }
            } else if section_depth > 0 {
                section_depth -= 1;
            } else {
                // still try pop
                namespace_stack.pop();
            }
            i += 1;
            continue;
        }
        if line.starts_with("section") {
            section_depth += 1;
            i += 1;
            continue;
        }

        // Attribute lines may precede declaration
        let (attrs, decl_line, start_line_idx) = collect_decl_start(&lines, i);
        if let Some((kind, after_kw)) = match_decl_keyword(decl_line) {
            if let Some((decl, end_i)) = extract_one_decl_with_end(
                &lines,
                start_line_idx,
                kind,
                after_kw,
                attrs,
                file,
                &namespace_stack,
                module_hint,
            ) {
                decls.push(decl);
                i = end_i;
                continue;
            }
        }

        i += 1;
    }

    decls
}

fn strip_line_comment(line: &str) -> &str {
    // careful with `"--"` in strings — good enough for declarations
    if let Some(idx) = find_line_comment(line) {
        &line[..idx]
    } else {
        line
    }
}

fn find_line_comment(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut in_str = false;
    while i + 1 < bytes.len() {
        let c = bytes[i];
        if in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            in_str = true;
            i += 1;
            continue;
        }
        if c == b'-' && bytes[i + 1] == b'-' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn collect_decl_start<'a>(lines: &[&'a str], i: usize) -> (Vec<String>, &'a str, usize) {
    let mut attrs = Vec::new();
    let mut j = i;
    // gather pure attribute lines
    while j < lines.len() {
        let t = strip_line_comment(lines[j]).trim();
        if t.starts_with("@[") && !contains_decl_keyword(t) {
            attrs.push(t.to_string());
            j += 1;
            continue;
        }
        break;
    }
    if j >= lines.len() {
        return (attrs, "", i);
    }
    let t = strip_line_comment(lines[j]).trim();
    // attributes on same line as theorem
    let (more_attrs, rest) = split_leading_attrs(t);
    attrs.extend(more_attrs);
    (attrs, rest, j)
}

fn split_leading_attrs(s: &str) -> (Vec<String>, &str) {
    let mut attrs = Vec::new();
    let mut rest = s;
    while rest.starts_with('@') {
        if let Some(end) = find_matching_bracket(rest, 1) {
            // @[...]
            attrs.push(rest[..=end].to_string());
            rest = rest[end + 1..].trim_start();
        } else {
            break;
        }
    }
    (attrs, rest)
}

fn find_matching_bracket(s: &str, open_idx: usize) -> Option<usize> {
    // open_idx points at '['
    let bytes = s.as_bytes();
    if open_idx >= bytes.len() || bytes[open_idx] != b'[' {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open_idx;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn contains_decl_keyword(s: &str) -> bool {
    s.contains("theorem ") || s.contains("lemma ") || s.contains("axiom ")
}

fn match_decl_keyword(s: &str) -> Option<(DeclKind, &str)> {
    let s = s.trim();
    // modifiers
    let mut rest = s;
    loop {
        let trimmed = rest.trim_start();
        if let Some(r) = strip_modifier(trimmed) {
            rest = r;
            continue;
        }
        break;
    }
    let rest = rest.trim_start();
    for (kw, kind) in [
        ("theorem ", DeclKind::Theorem),
        ("lemma ", DeclKind::Lemma),
        ("axiom ", DeclKind::Axiom),
    ] {
        if let Some(after) = rest.strip_prefix(kw) {
            return Some((kind, after.trim_start()));
        }
    }
    None
}

fn strip_modifier(s: &str) -> Option<&str> {
    for m in [
        "protected ",
        "private ",
        "noncomputable ",
        "public ",
        "unsafe ",
        "partial ",
    ] {
        if let Some(r) = s.strip_prefix(m) {
            return Some(r);
        }
    }
    None
}

fn extract_one_decl_with_end(
    lines: &[&str],
    start: usize,
    kind: DeclKind,
    after_kw: &str,
    attrs: Vec<String>,
    file: &str,
    namespace_stack: &[String],
    module_hint: Option<&str>,
) -> Option<(Declaration, usize)> {
    // Build a buffer from after_kw through the type, stopping at `:=` or bare axiom end.
    let mut buf = after_kw.to_string();
    let mut end = start;
    let mut depth_paren = count_balance(&buf, '(', ')');
    let mut depth_brace = count_balance(&buf, '{', '}');
    let mut depth_brack = count_balance(&buf, '[', ']');
    let seen_assign = buf.contains(":=");

    if !seen_assign {
        let mut j = start + 1;
        while j < lines.len() {
            let t = strip_line_comment(lines[j]);
            let trimmed = t.trim();
            // stop at next top-level declaration-like if we've finished type
            if depth_paren <= 0
                && depth_brace <= 0
                && depth_brack <= 0
                && looks_like_new_decl(trimmed)
                && buf.contains(':')
            {
                break;
            }
            buf.push('\n');
            buf.push_str(t);
            depth_paren += count_balance(t, '(', ')');
            depth_brace += count_balance(t, '{', '}');
            depth_brack += count_balance(t, '[', ']');
            end = j;
            if t.contains(":=") {
                break;
            }
            // `where` clause after type
            if depth_paren <= 0
                && depth_brace <= 0
                && depth_brack <= 0
                && (trimmed.starts_with("where") || trimmed.contains(" where "))
                && buf.contains(':')
            {
                break;
            }
            j += 1;
            // safety limit
            if j > start + 80 {
                break;
            }
        }
    }

    // Split name / binders / type
    let (name, binders_src, type_src) = split_name_binders_type(&buf)?;
    if name.is_empty() {
        return None;
    }

    let binders = parse_binders_src(&binders_src);
    let type_surface = collapse_ws(&type_src);
    if type_surface.is_empty() {
        return None;
    }

    let ty = match parse_type(&type_surface) {
        Ok(t) => t,
        Err(_) => {
            // Fallback: store as Raw so we still index the declaration
            TypeExpr::Raw(type_surface.clone())
        }
    };

    let full_name = if namespace_stack.is_empty() {
        name.clone()
    } else {
        format!("{}.{}", namespace_stack.join("."), name)
    };

    let module = module_hint.map(|s| s.to_string()).or_else(|| {
        Path::new(file)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
    });

    let decl = Declaration {
        kind,
        name,
        full_name,
        binders,
        ty,
        type_surface,
        file: file.to_string(),
        line: start + 1,
        module,
        namespace_path: namespace_stack.to_vec(),
        attributes: attrs,
    };

    Some((decl, end + 1))
}

fn looks_like_new_decl(trimmed: &str) -> bool {
    let t = trimmed.trim_start_matches('@');
    // attribute-only
    if trimmed.starts_with("@[") && !contains_decl_keyword(trimmed) {
        return true;
    }
    for p in [
        "theorem ",
        "lemma ",
        "axiom ",
        "def ",
        "instance ",
        "class ",
        "structure ",
        "inductive ",
        "namespace ",
        "end ",
        "section ",
        "variable ",
        "example ",
        "abbrev ",
        "opaque ",
        "mutual ",
    ] {
        if t.starts_with(p) || t == "end" {
            return true;
        }
    }
    // protected theorem etc.
    for m in ["protected ", "private ", "noncomputable ", "public "] {
        if let Some(r) = t.strip_prefix(m) {
            return looks_like_new_decl(r);
        }
    }
    false
}

fn count_balance(s: &str, open: char, close: char) -> i32 {
    let mut d = 0i32;
    for c in s.chars() {
        if c == open {
            d += 1;
        } else if c == close {
            d -= 1;
        }
    }
    d
}

/// Split `name binders* : type` (type may include nested `:`).
fn split_name_binders_type(buf: &str) -> Option<(String, String, String)> {
    // Remove `:= ...` and `where ...`
    let mut s = buf;
    if let Some(idx) = find_top_level(s, ":=") {
        s = &s[..idx];
    }
    if let Some(idx) = find_top_level_word(s, "where") {
        s = &s[..idx];
    }
    // Equation-compiler clauses: `theorem foo : T | pat => proof`
    if let Some(idx) = find_top_level_eqns(s) {
        s = &s[..idx];
    }
    let s = s.trim();

    // Name: first identifier (possibly dotted, or «escaped»)
    let mut lx = Lexer::new(s);
    let name_tok = lx.next_token();
    let name = match name_tok {
        Token::Ident(n) => n,
        _ => return None,
    };
    // optional `.` continuation already in Ident for simple names; dotted `Foo.bar` may be two tokens
    let name_end = lx.position();
    // Rest is binders + : type. Find the type colon at binder depth 0.
    let rest = s[name_end..].trim_start();
    let (binders_src, type_src) = split_binders_and_type(rest)?;
    Some((name, binders_src, type_src))
}

fn split_binders_and_type(rest: &str) -> Option<(String, String)> {
    // Scan for `:` at depth 0 that starts the type.
    // Binders are (...) {...} [...] ⦃...⦄ and bare ids rarely before colon in Lean 4
    // for theorems usually all binders are grouped.
    let chars: Vec<char> = rest.chars().collect();
    let mut i = 0usize;

    // Skip leading binder groups and whitespace
    while i < chars.len() {
        match chars[i] {
            c if c.is_whitespace() => i += 1,
            '(' => {
                let mut depth_p = 1i32;
                i += 1;
                while i < chars.len() && depth_p > 0 {
                    match chars[i] {
                        '(' => depth_p += 1,
                        ')' => depth_p -= 1,
                        _ => {}
                    }
                    i += 1;
                }
            }
            '{' => {
                let mut depth_b = 1i32;
                i += 1;
                while i < chars.len() && depth_b > 0 {
                    match chars[i] {
                        '{' => depth_b += 1,
                        '}' => depth_b -= 1,
                        _ => {}
                    }
                    i += 1;
                }
            }
            '[' => {
                let mut depth_k = 1i32;
                i += 1;
                while i < chars.len() && depth_k > 0 {
                    match chars[i] {
                        '[' => depth_k += 1,
                        ']' => depth_k -= 1,
                        _ => {}
                    }
                    i += 1;
                }
            }
            '⦃' => {
                let mut depth_s = 1i32;
                i += 1;
                while i < chars.len() && depth_s > 0 {
                    if chars[i] == '⦃' {
                        depth_s += 1;
                    } else if chars[i] == '⦄' {
                        depth_s -= 1;
                    }
                    i += 1;
                }
            }
            ':' => {
                // type starts after this
                let binders = rest[..char_byte_index(rest, i)].trim().to_string();
                let ty = rest[char_byte_index(rest, i) + 1..].trim().to_string();
                return Some((binders, ty));
            }
            // bare binder name without parens: `theorem foo n : ...` rare but handle
            c if is_name_start(c) => {
                // consume ident
                i += 1;
                while i < chars.len() && is_name_continue(chars[i]) {
                    i += 1;
                }
            }
            _ => {
                // unexpected — try finding first top-level colon
                break;
            }
        }
    }

    // Fallback: first top-level colon
    if let Some(idx) = find_top_level(rest, ":") {
        // ensure not :=
        if rest[idx..].starts_with(":=") {
            return None;
        }
        let binders = rest[..idx].trim().to_string();
        let ty = rest[idx + 1..].trim().to_string();
        Some((binders, ty))
    } else {
        None
    }
}

fn char_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn is_name_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '«'
}

fn is_name_continue(c: char) -> bool {
    is_name_start(c) || c.is_ascii_digit() || c == '\'' || c == '?' || c == '»'
}

fn find_top_level(s: &str, pat: &str) -> Option<usize> {
    let mut depth_p = 0i32;
    let mut depth_b = 0i32;
    let mut depth_k = 0i32;
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = s[i..].chars().next()?;
        match c {
            '(' => depth_p += 1,
            ')' => depth_p -= 1,
            '{' => depth_b += 1,
            '}' => depth_b -= 1,
            '[' => depth_k += 1,
            ']' => depth_k -= 1,
            _ => {}
        }
        if depth_p == 0
            && depth_b == 0
            && depth_k == 0
            && s[i..].starts_with(pat)
        {
            // special: for ":" don't match ":="
            if pat == ":" && s[i..].starts_with(":=") {
                i += c.len_utf8();
                continue;
            }
            return Some(i);
        }
        i += c.len_utf8();
    }
    None
}

/// Find start of equation-compiler arms after a type: top-level ` | ` with `=>` later.
fn find_top_level_eqns(s: &str) -> Option<usize> {
    let mut depth_p = 0i32;
    let mut depth_b = 0i32;
    let mut depth_k = 0i32;
    let mut i = 0usize;
    // Only consider after the type colon has appeared at depth 0.
    let mut seen_type_colon = false;
    while i < s.len() {
        let c = s[i..].chars().next()?;
        match c {
            '(' => depth_p += 1,
            ')' => depth_p -= 1,
            '{' => depth_b += 1,
            '}' => depth_b -= 1,
            '[' => depth_k += 1,
            ']' => depth_k -= 1,
            ':' if depth_p == 0 && depth_b == 0 && depth_k == 0 && !s[i..].starts_with(":=") => {
                seen_type_colon = true;
            }
            '|' if seen_type_colon && depth_p == 0 && depth_b == 0 && depth_k == 0 => {
                // Ensure it's not `||` or infix `∣` already handled; look for `=>` or `↦` after
                let rest = &s[i..];
                if rest.starts_with("||") || rest.starts_with("|>") {
                    i += c.len_utf8();
                    continue;
                }
                // Match arms typically: `| pat =>` or multi `| a, b =>`
                if rest.contains("=>") || rest.contains('↦') {
                    // require the pipe to be preceded by whitespace or start
                    let before_ok = i == 0
                        || s[..i]
                            .chars()
                            .next_back()
                            .map(|ch| ch.is_whitespace())
                            .unwrap_or(true);
                    if before_ok {
                        return Some(i);
                    }
                }
            }
            _ => {}
        }
        i += c.len_utf8();
    }
    None
}

fn find_top_level_word(s: &str, word: &str) -> Option<usize> {
    let mut depth_p = 0i32;
    let mut depth_b = 0i32;
    let mut depth_k = 0i32;
    let mut i = 0usize;
    while i < s.len() {
        let c = s[i..].chars().next()?;
        match c {
            '(' => depth_p += 1,
            ')' => depth_p -= 1,
            '{' => depth_b += 1,
            '}' => depth_b -= 1,
            '[' => depth_k += 1,
            ']' => depth_k -= 1,
            _ => {}
        }
        if depth_p == 0 && depth_b == 0 && depth_k == 0 && s[i..].starts_with(word) {
            let before_ok = i == 0
                || s[..i]
                    .chars()
                    .next_back()
                    .map(|ch| !ch.is_alphanumeric() && ch != '_')
                    .unwrap_or(true);
            let after = i + word.len();
            let after_ok = after >= s.len()
                || s[after..]
                    .chars()
                    .next()
                    .map(|ch| !ch.is_alphanumeric() && ch != '_')
                    .unwrap_or(true);
            if before_ok && after_ok {
                return Some(i);
            }
        }
        i += c.len_utf8();
    }
    None
}

fn parse_binders_src(src: &str) -> Vec<Binder> {
    let src = src.trim();
    if src.is_empty() {
        return Vec::new();
    }
    // Reuse type parser binder groups by wrapping: parse successive groups via lexer
    let mut binders = Vec::new();
    let mut rest = src;
    while !rest.is_empty() {
        let rest_trim = rest.trim_start();
        if rest_trim.is_empty() {
            break;
        }
        let (b, consumed) = match rest_trim.chars().next() {
            Some('(') => parse_one_group(rest_trim, BinderKind::Default, '(', ')'),
            Some('{') => parse_one_group(rest_trim, BinderKind::Implicit, '{', '}'),
            Some('[') => parse_one_group(rest_trim, BinderKind::Instance, '[', ']'),
            Some('⦃') => parse_one_group_strict(rest_trim),
            _ => {
                // bare name
                let mut lx = Lexer::new(rest_trim);
                match lx.next_token() {
                    Token::Ident(n) => {
                        let c = lx.position();
                        binders.push(Binder {
                            kind: BinderKind::Default,
                            names: vec![n],
                            ty: None,
                        });
                        rest = &rest_trim[c..];
                        continue;
                    }
                    _ => break,
                }
            }
        };
        if let Some(b) = b {
            binders.push(b);
        }
        if consumed == 0 {
            break;
        }
        rest = &rest_trim[consumed..];
    }
    binders
}

fn parse_one_group(
    s: &str,
    kind: BinderKind,
    open: char,
    close: char,
) -> (Option<Binder>, usize) {
    if !s.starts_with(open) {
        return (None, 0);
    }
    let mut depth = 0i32;
    let mut end = 0usize;
    for (idx, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                end = idx + c.len_utf8();
                break;
            }
        }
    }
    if end == 0 {
        return (None, 0);
    }
    let inner = &s[open.len_utf8()..end - close.len_utf8()];
    let binder = parse_binder_inner(kind, inner);
    (Some(binder), end)
}

fn parse_one_group_strict(s: &str) -> (Option<Binder>, usize) {
    if !s.starts_with('⦃') {
        return (None, 0);
    }
    let mut depth = 0i32;
    let mut end = 0usize;
    for (idx, c) in s.char_indices() {
        if c == '⦃' {
            depth += 1;
        } else if c == '⦄' {
            depth -= 1;
            if depth == 0 {
                end = idx + c.len_utf8();
                break;
            }
        }
    }
    if end == 0 {
        return (None, 0);
    }
    let inner = &s['⦃'.len_utf8()..end - '⦄'.len_utf8()];
    (Some(parse_binder_inner(BinderKind::StrictImplicit, inner)), end)
}

fn parse_binder_inner(kind: BinderKind, inner: &str) -> Binder {
    let inner = inner.trim();
    if let Some(colon) = find_top_level(inner, ":") {
        let names_part = inner[..colon].trim();
        let ty_part = inner[colon + 1..].trim();
        let names = if names_part.is_empty() {
            vec![]
        } else {
            names_part
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        };
        let ty = parse_type(ty_part).ok().map(Box::new);
        Binder { kind, names, ty }
    } else {
        // type only (instance) or names only
        if inner.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '\'' || c == ' ' || c == '.') {
            let names: Vec<_> = inner.split_whitespace().map(|s| s.to_string()).collect();
            // if looks like Type App, store as type
            if names.len() > 1 && names[0].chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            {
                Binder {
                    kind,
                    names: vec![],
                    ty: parse_type(inner).ok().map(Box::new),
                }
            } else {
                Binder {
                    kind,
                    names,
                    ty: None,
                }
            }
        } else {
            Binder {
                kind,
                names: vec![],
                ty: parse_type(inner).ok().map(Box::new),
            }
        }
    }
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[allow(dead_code)]
fn _parse_error_unused(_: ParseError) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_theorems() {
        let src = r#"
namespace Nat

theorem add_comm (n m : Nat) : n + m = m + n := sorry

@[simp] theorem add_zero (n : Nat) : n + 0 = n := rfl

lemma foo : True := trivial

axiom choice {α : Sort u} : Nonempty α → α

end Nat
"#;
        let decls = parse_declarations(src, "Test.lean");
        assert!(decls.len() >= 4, "got {:?}", decls.iter().map(|d| &d.name).collect::<Vec<_>>());
        let add_comm = decls.iter().find(|d| d.name == "add_comm").unwrap();
        assert_eq!(add_comm.kind, DeclKind::Theorem);
        assert!(add_comm.full_name.contains("add_comm"));
        assert!(matches!(&add_comm.ty, TypeExpr::BinOp { op, .. } if op == "="));
        let ax = decls.iter().find(|d| d.name == "choice").unwrap();
        assert_eq!(ax.kind, DeclKind::Axiom);
    }

    #[test]
    fn parse_multiline_type() {
        let src = r#"
theorem multi (n : Nat)
    (m : Nat) :
    n + m = m + n := by
  sorry
"#;
        let decls = parse_declarations(src, "M.lean");
        assert_eq!(decls.len(), 1);
        assert!(decls[0].type_surface.contains('='));
    }
}
