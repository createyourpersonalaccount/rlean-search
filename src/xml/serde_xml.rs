//! Hand-written XML codec for `http://github.com/createyourpersonalaccount/rlean-search`.

use crate::ast::{
    Binder, BinderKind, DeclKind, Declaration, IndexDocument, PackageInfo, TypeExpr, RLEAN_NS,
};
use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

/// Write an index document. Paths ending in `.gz` are gzip-compressed using the
/// fastest compression level (intended for the default `index.xml.gz` cache).
pub fn write_index_file(path: impl AsRef<Path>, doc: &IndexDocument) -> Result<()> {
    let path = path.as_ref();
    let xml = index_to_xml(doc);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(path)
        .with_context(|| format!("creating index {}", path.display()))?;
    if is_gzip_path(path) {
        // Fastest gzip mode for quick cache write/read.
        let mut enc = GzEncoder::new(&mut f, Compression::fast());
        enc.write_all(xml.as_bytes())?;
        enc.finish()?;
    } else {
        f.write_all(xml.as_bytes())?;
    }
    Ok(())
}

/// Read an index document. Paths ending in `.gz` are decoded with gzip.
pub fn read_index_file(path: impl AsRef<Path>) -> Result<IndexDocument> {
    let path = path.as_ref();
    let s = if is_gzip_path(path) {
        let f = fs::File::open(path)
            .with_context(|| format!("reading index {}", path.display()))?;
        let mut dec = GzDecoder::new(f);
        let mut buf = String::new();
        dec.read_to_string(&mut buf)
            .with_context(|| format!("gunzipping index {}", path.display()))?;
        buf
    } else {
        fs::read_to_string(path)
            .with_context(|| format!("reading index {}", path.display()))?
    };
    xml_to_index(&s)
}

fn is_gzip_path(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
}

pub fn index_to_xml(doc: &IndexDocument) -> String {
    let mut out = String::with_capacity(doc.declarations.len().saturating_mul(256) + 1024);
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    out.push_str(&format!(
        r#"<rlean:index xmlns:rlean="{ns}" schema="{schema}" created_at="{created}" source_hash="{hash}">"#,
        ns = RLEAN_NS,
        schema = escape_attr(&doc.schema),
        created = escape_attr(&doc.created_at),
        hash = escape_attr(&doc.source_hash),
    ));
    out.push('\n');

    for pkg in &doc.packages {
        out.push_str("  <rlean:package");
        out.push_str(&format!(
            r#" name="{}" root="{}""#,
            escape_attr(&pkg.name),
            escape_attr(&pkg.root)
        ));
        out.push_str(">\n");
        for d in &pkg.src_dirs {
            out.push_str(&format!(
                "    <rlean:srcDir>{}</rlean:srcDir>\n",
                escape_text(d)
            ));
        }
        for lib in &pkg.lean_libs {
            out.push_str(&format!(
                "    <rlean:leanLib>{}</rlean:leanLib>\n",
                escape_text(lib)
            ));
        }
        out.push_str("  </rlean:package>\n");
    }

    for d in &doc.declarations {
        out.push_str(&declaration_to_xml(d, 1));
    }

    out.push_str("</rlean:index>\n");
    out
}

pub fn declaration_to_xml(d: &Declaration, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let mut out = String::new();
    out.push_str(&format!(
        r#"{pad}<rlean:declaration kind="{kind}" name="{name}" full_name="{full}" file="{file}" line="{line}""#,
        kind = d.kind.as_str(),
        name = escape_attr(&d.name),
        full = escape_attr(&d.full_name),
        file = escape_attr(&d.file),
        line = d.line,
    ));
    if let Some(m) = &d.module {
        out.push_str(&format!(r#" module="{}""#, escape_attr(m)));
    }
    if !d.namespace_path.is_empty() {
        out.push_str(&format!(
            r#" namespace="{}""#,
            escape_attr(&d.namespace_path.join("."))
        ));
    }
    out.push_str(">\n");

    for a in &d.attributes {
        out.push_str(&format!(
            "{pad}  <rlean:attribute>{}</rlean:attribute>\n",
            escape_text(a)
        ));
    }
    for b in &d.binders {
        out.push_str(&binder_to_xml(b, indent + 1));
    }
    out.push_str(&format!(
        "{pad}  <rlean:typeSurface>{}</rlean:typeSurface>\n",
        escape_text(&d.type_surface)
    ));
    out.push_str(&format!("{pad}  <rlean:type>\n"));
    out.push_str(&type_to_xml(&d.ty, indent + 2));
    out.push_str(&format!("{pad}  </rlean:type>\n"));
    out.push_str(&format!(
        "{pad}  <rlean:conclusion head=\"{}\">\n",
        escape_attr(&d.effective_type().conclusion().head_key())
    ));
    out.push_str(&type_to_xml(d.effective_type().conclusion(), indent + 2));
    out.push_str(&format!("{pad}  </rlean:conclusion>\n"));
    out.push_str(&format!("{pad}</rlean:declaration>\n"));
    out
}

fn binder_to_xml(b: &Binder, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let kind = match b.kind {
        BinderKind::Default => "default",
        BinderKind::Implicit => "implicit",
        BinderKind::Instance => "instance",
        BinderKind::StrictImplicit => "strictImplicit",
    };
    let mut out = format!(r#"{pad}<rlean:binder kind="{kind}""#);
    if !b.names.is_empty() {
        out.push_str(&format!(r#" names="{}""#, escape_attr(&b.names.join(" "))));
    }
    out.push_str(">\n");
    if let Some(ty) = &b.ty {
        out.push_str(&type_to_xml(ty, indent + 1));
    }
    out.push_str(&format!("{pad}</rlean:binder>\n"));
    out
}

fn type_to_xml(t: &TypeExpr, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    match t {
        TypeExpr::Hole => format!("{pad}<rlean:hole/>\n"),
        TypeExpr::NamedHole(n) => format!(
            "{pad}<rlean:namedHole name=\"{}\"/>\n",
            escape_attr(n)
        ),
        TypeExpr::Ident(s) => format!(
            "{pad}<rlean:ident name=\"{}\"/>\n",
            escape_attr(s)
        ),
        TypeExpr::NatLit(n) => format!(
            "{pad}<rlean:natLit value=\"{}\"/>\n",
            escape_attr(n)
        ),
        TypeExpr::Literal(s) => format!(
            "{pad}<rlean:literal>{}</rlean:literal>\n",
            escape_text(s)
        ),
        TypeExpr::App(f, a) => {
            let mut out = format!("{pad}<rlean:app>\n");
            out.push_str(&format!("{pad}  <rlean:fn>\n"));
            out.push_str(&type_to_xml(f, indent + 2));
            out.push_str(&format!("{pad}  </rlean:fn>\n"));
            out.push_str(&format!("{pad}  <rlean:arg>\n"));
            out.push_str(&type_to_xml(a, indent + 2));
            out.push_str(&format!("{pad}  </rlean:arg>\n"));
            out.push_str(&format!("{pad}</rlean:app>\n"));
            out
        }
        TypeExpr::BinOp { op, left, right } => {
            let mut out = format!(
                "{pad}<rlean:binOp op=\"{}\">\n",
                escape_attr(op)
            );
            out.push_str(&format!("{pad}  <rlean:left>\n"));
            out.push_str(&type_to_xml(left, indent + 2));
            out.push_str(&format!("{pad}  </rlean:left>\n"));
            out.push_str(&format!("{pad}  <rlean:right>\n"));
            out.push_str(&type_to_xml(right, indent + 2));
            out.push_str(&format!("{pad}  </rlean:right>\n"));
            out.push_str(&format!("{pad}</rlean:binOp>\n"));
            out
        }
        TypeExpr::UnaryOp { op, arg } => {
            let mut out = format!(
                "{pad}<rlean:unaryOp op=\"{}\">\n",
                escape_attr(op)
            );
            out.push_str(&type_to_xml(arg, indent + 1));
            out.push_str(&format!("{pad}</rlean:unaryOp>\n"));
            out
        }
        TypeExpr::Postfix { arg, op } => {
            let mut out = format!(
                "{pad}<rlean:postfix op=\"{}\">\n",
                escape_attr(op)
            );
            out.push_str(&type_to_xml(arg, indent + 1));
            out.push_str(&format!("{pad}</rlean:postfix>\n"));
            out
        }
        TypeExpr::Arrow(a, b) => {
            let mut out = format!("{pad}<rlean:arrow>\n");
            out.push_str(&format!("{pad}  <rlean:domain>\n"));
            out.push_str(&type_to_xml(a, indent + 2));
            out.push_str(&format!("{pad}  </rlean:domain>\n"));
            out.push_str(&format!("{pad}  <rlean:codomain>\n"));
            out.push_str(&type_to_xml(b, indent + 2));
            out.push_str(&format!("{pad}  </rlean:codomain>\n"));
            out.push_str(&format!("{pad}</rlean:arrow>\n"));
            out
        }
        TypeExpr::Forall { binders, body } => {
            let mut out = format!("{pad}<rlean:forall>\n");
            for b in binders {
                out.push_str(&binder_to_xml(b, indent + 1));
            }
            out.push_str(&format!("{pad}  <rlean:body>\n"));
            out.push_str(&type_to_xml(body, indent + 2));
            out.push_str(&format!("{pad}  </rlean:body>\n"));
            out.push_str(&format!("{pad}</rlean:forall>\n"));
            out
        }
        TypeExpr::Exists { binders, body } => {
            let mut out = format!("{pad}<rlean:exists>\n");
            for b in binders {
                out.push_str(&binder_to_xml(b, indent + 1));
            }
            out.push_str(&format!("{pad}  <rlean:body>\n"));
            out.push_str(&type_to_xml(body, indent + 2));
            out.push_str(&format!("{pad}  </rlean:body>\n"));
            out.push_str(&format!("{pad}</rlean:exists>\n"));
            out
        }
        TypeExpr::Lambda { binders, body } => {
            let mut out = format!("{pad}<rlean:lambda>\n");
            for b in binders {
                out.push_str(&binder_to_xml(b, indent + 1));
            }
            out.push_str(&format!("{pad}  <rlean:body>\n"));
            out.push_str(&type_to_xml(body, indent + 2));
            out.push_str(&format!("{pad}  </rlean:body>\n"));
            out.push_str(&format!("{pad}</rlean:lambda>\n"));
            out
        }
        TypeExpr::Pi { binder, body } => {
            let mut out = format!("{pad}<rlean:pi>\n");
            out.push_str(&binder_to_xml(binder, indent + 1));
            out.push_str(&format!("{pad}  <rlean:body>\n"));
            out.push_str(&type_to_xml(body, indent + 2));
            out.push_str(&format!("{pad}  </rlean:body>\n"));
            out.push_str(&format!("{pad}</rlean:pi>\n"));
            out
        }
        TypeExpr::Proj { base, field } => {
            let mut out = format!(
                "{pad}<rlean:proj field=\"{}\">\n",
                escape_attr(field)
            );
            out.push_str(&type_to_xml(base, indent + 1));
            out.push_str(&format!("{pad}</rlean:proj>\n"));
            out
        }
        TypeExpr::Sort { name, level } => {
            let mut out = format!(
                r#"{pad}<rlean:sort name="{}""#,
                escape_attr(name)
            );
            if level.is_none() {
                out.push_str("/>\n");
            } else {
                out.push_str(">\n");
                out.push_str(&type_to_xml(level.as_ref().unwrap(), indent + 1));
                out.push_str(&format!("{pad}</rlean:sort>\n"));
            }
            out
        }
        TypeExpr::Raw(s) => format!("{pad}<rlean:raw>{}</rlean:raw>\n", escape_text(s)),
    }
}

pub fn xml_to_index(xml: &str) -> Result<IndexDocument> {
    // Lightweight tag-driven parser for our own format.
    let mut doc = IndexDocument::new();
    if let Some(cap) = attr_of_open(xml, "rlean:index", "source_hash") {
        doc.source_hash = cap;
    }
    if let Some(cap) = attr_of_open(xml, "rlean:index", "created_at") {
        doc.created_at = cap;
    }
    if let Some(cap) = attr_of_open(xml, "rlean:index", "schema") {
        doc.schema = cap;
    }

    // Packages
    for pkg_xml in iter_elements(xml, "rlean:package") {
        let name = attr_of_open(&pkg_xml, "rlean:package", "name").unwrap_or_default();
        let root = attr_of_open(&pkg_xml, "rlean:package", "root").unwrap_or_default();
        let mut src_dirs = Vec::new();
        let mut lean_libs = Vec::new();
        for s in iter_text_elements(&pkg_xml, "rlean:srcDir") {
            src_dirs.push(unescape(&s));
        }
        for s in iter_text_elements(&pkg_xml, "rlean:leanLib") {
            lean_libs.push(unescape(&s));
        }
        doc.packages.push(PackageInfo {
            name,
            root,
            src_dirs,
            lean_libs,
        });
    }

    for decl_xml in iter_elements(xml, "rlean:declaration") {
        if let Some(d) = parse_declaration_xml(&decl_xml) {
            doc.declarations.push(d);
        }
    }

    if doc.declarations.is_empty() && !xml.contains("rlean:declaration") {
        // still valid empty index
    }
    Ok(doc)
}

fn parse_declaration_xml(xml: &str) -> Option<Declaration> {
    let kind = DeclKind::parse(&attr_of_open(xml, "rlean:declaration", "kind")?)?;
    let name = attr_of_open(xml, "rlean:declaration", "name")?;
    let full_name = attr_of_open(xml, "rlean:declaration", "full_name").unwrap_or_else(|| name.clone());
    let file = attr_of_open(xml, "rlean:declaration", "file").unwrap_or_default();
    let line = attr_of_open(xml, "rlean:declaration", "line")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let module = attr_of_open(xml, "rlean:declaration", "module");
    let namespace_path = attr_of_open(xml, "rlean:declaration", "namespace")
        .map(|s| {
            if s.is_empty() {
                vec![]
            } else {
                s.split('.').map(|x| x.to_string()).collect()
            }
        })
        .unwrap_or_default();
    let mut attributes = Vec::new();
    for a in iter_text_elements(xml, "rlean:attribute") {
        attributes.push(unescape(&a));
    }
    let type_surface = iter_text_elements(xml, "rlean:typeSurface")
        .into_iter()
        .next()
        .map(|s| unescape(&s))
        .unwrap_or_default();

    // Prefer reconstructing type from typeSurface via parser for fidelity of matching
    let ty = crate::parser::parse_type(&type_surface).unwrap_or(TypeExpr::Raw(type_surface.clone()));

    // binders: optional
    let mut binders = Vec::new();
    for bxml in iter_elements(xml, "rlean:binder") {
        let bkind = match attr_of_open(&bxml, "rlean:binder", "kind")
            .unwrap_or_else(|| "default".into())
            .as_str()
        {
            "implicit" => BinderKind::Implicit,
            "instance" => BinderKind::Instance,
            "strictImplicit" => BinderKind::StrictImplicit,
            _ => BinderKind::Default,
        };
        let names = attr_of_open(&bxml, "rlean:binder", "names")
            .map(|s| s.split_whitespace().map(|x| x.to_string()).collect())
            .unwrap_or_default();
        binders.push(Binder {
            kind: bkind,
            names,
            ty: None,
        });
    }

    Some(Declaration {
        kind,
        name,
        full_name,
        binders,
        ty,
        type_surface,
        file,
        line,
        module,
        namespace_path,
        attributes,
    })
}

fn attr_of_open(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{tag}");
    let idx = xml.find(&open)?;
    let after = &xml[idx..];
    let end = after.find('>')?;
    let head = &after[..end];
    let key = format!(r#"{attr}=""#);
    let a = head.find(&key)?;
    let rest = &head[a + key.len()..];
    let endq = rest.find('"')?;
    Some(unescape(&rest[..endq]))
}

fn iter_elements(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = xml[search_from..].find(&open) {
        let start = search_from + rel;
        // ensure tag boundary
        let after_tag = start + open.len();
        if after_tag < xml.len() {
            let c = xml[after_tag..].chars().next().unwrap_or(' ');
            if c.is_alphanumeric() || c == ':' || c == '_' || c == '-' {
                search_from = after_tag;
                continue;
            }
        }
        let rest = &xml[start..];
        // self-closing?
        if let Some(gt) = rest.find('>') {
            if rest[..gt].ends_with('/') {
                out.push(rest[..=gt].to_string());
                search_from = start + gt + 1;
                continue;
            }
        }
        if let Some(rel_c) = rest.find(&close) {
            let end = rel_c + close.len();
            out.push(rest[..end].to_string());
            search_from = start + end;
        } else {
            break;
        }
    }
    out
}

fn iter_text_elements(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = xml[search_from..].find(&open) {
        let start = search_from + rel + open.len();
        if let Some(rel_c) = xml[start..].find(&close) {
            out.push(xml[start..start + rel_c].to_string());
            search_from = start + rel_c + close.len();
        } else {
            break;
        }
    }
    out
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape_text(s).replace('"', "&quot;").replace('\'', "&apos;")
}

fn unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[allow(dead_code)]
fn _unused_anyhow() -> Result<()> {
    Err(anyhow!("x"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::DeclKind;

    #[test]
    fn roundtrip_minimal() {
        let mut doc = IndexDocument::new();
        doc.source_hash = "abc".into();
        doc.declarations.push(Declaration {
            kind: DeclKind::Theorem,
            name: "add_comm".into(),
            full_name: "Nat.add_comm".into(),
            binders: vec![],
            ty: TypeExpr::BinOp {
                op: "=".into(),
                left: Box::new(TypeExpr::BinOp {
                    op: "+".into(),
                    left: Box::new(TypeExpr::Ident("n".into())),
                    right: Box::new(TypeExpr::Ident("m".into())),
                }),
                right: Box::new(TypeExpr::BinOp {
                    op: "+".into(),
                    left: Box::new(TypeExpr::Ident("m".into())),
                    right: Box::new(TypeExpr::Ident("n".into())),
                }),
            },
            type_surface: "n + m = m + n".into(),
            file: "Nat.lean".into(),
            line: 10,
            module: Some("Nat".into()),
            namespace_path: vec!["Nat".into()],
            attributes: vec![],
        });
        let xml = index_to_xml(&doc);
        assert!(xml.contains(RLEAN_NS));
        assert!(xml.contains("add_comm"));
        let back = xml_to_index(&xml).unwrap();
        assert_eq!(back.declarations.len(), 1);
        assert_eq!(back.declarations[0].name, "add_comm");
        assert_eq!(back.source_hash, "abc");
    }

    #[test]
    fn gzip_roundtrip_index_xml_gz() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.xml.gz");
        let mut doc = IndexDocument::new();
        doc.source_hash = "gzip-test".into();
        doc.declarations.push(Declaration {
            kind: DeclKind::Lemma,
            name: "sub_self".into(),
            full_name: "Nat.sub_self".into(),
            binders: vec![],
            ty: TypeExpr::BinOp {
                op: "=".into(),
                left: Box::new(TypeExpr::BinOp {
                    op: "-".into(),
                    left: Box::new(TypeExpr::Ident("n".into())),
                    right: Box::new(TypeExpr::Ident("n".into())),
                }),
                right: Box::new(TypeExpr::NatLit("0".into())),
            },
            type_surface: "n - n = 0".into(),
            file: "Nat.lean".into(),
            line: 1,
            module: None,
            namespace_path: vec![],
            attributes: vec![],
        });
        write_index_file(&path, &doc).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[..2], &[0x1f, 0x8b]);
        let back = read_index_file(&path).unwrap();
        assert_eq!(back.source_hash, "gzip-test");
        assert_eq!(back.declarations[0].name, "sub_self");
    }
}
