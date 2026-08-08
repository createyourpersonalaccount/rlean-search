//! JSONL and XML request/response protocol for the daemon and client.

use crate::ast::RLEAN_NS;
use crate::search::SearchHit;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Request {
    #[serde(rename = "search")]
    Search {
        pattern: String,
        #[serde(default = "default_limit")]
        limit: usize,
    },
    #[serde(rename = "stats")]
    Stats {},
    #[serde(rename = "reload")]
    Reload {},
    #[serde(rename = "ping")]
    Ping {},
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Response {
    #[serde(rename = "search")]
    Search {
        pattern: String,
        count: usize,
        hits: Vec<SearchHit>,
    },
    #[serde(rename = "stats")]
    Stats {
        declarations: usize,
        packages: usize,
        source_hash: String,
    },
    #[serde(rename = "ok")]
    Ok { message: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "pong")]
    Pong {},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolKind {
    Jsonl,
    Xml,
}

impl ProtocolKind {
    pub fn detect(line: &str) -> Self {
        let t = line.trim_start();
        if t.starts_with('<') {
            ProtocolKind::Xml
        } else {
            ProtocolKind::Jsonl
        }
    }
}

pub fn parse_request(line: &str) -> Result<(ProtocolKind, Request), String> {
    let kind = ProtocolKind::detect(line);
    match kind {
        ProtocolKind::Jsonl => {
            let req: Request =
                serde_json::from_str(line.trim()).map_err(|e| format!("json parse: {e}"))?;
            Ok((kind, req))
        }
        ProtocolKind::Xml => {
            let req = parse_xml_request(line)?;
            Ok((kind, req))
        }
    }
}

pub fn format_response(kind: ProtocolKind, resp: &Response) -> String {
    match kind {
        ProtocolKind::Jsonl => {
            let mut s = serde_json::to_string(resp).unwrap_or_else(|e| {
                format!(
                    r#"{{"type":"error","message":{}}}"#,
                    serde_json::to_string(&e.to_string()).unwrap()
                )
            });
            s.push('\n');
            s
        }
        ProtocolKind::Xml => {
            let mut s = response_to_xml(resp);
            if !s.ends_with('\n') {
                s.push('\n');
            }
            s
        }
    }
}

fn parse_xml_request(xml: &str) -> Result<Request, String> {
    let xml = xml.trim();
    // <search pattern="..." limit="50"/> or <request><search .../></request>
    if let Some(pat) = xml_attr(xml, "pattern") {
        let limit = xml_attr(xml, "limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(50);
        if xml.contains("search") {
            return Ok(Request::Search {
                pattern: pat,
                limit,
            });
        }
    }
    if xml.contains("<stats") || xml.contains("<rlean:stats") {
        return Ok(Request::Stats {});
    }
    if xml.contains("<reload") || xml.contains("<rlean:reload") {
        return Ok(Request::Reload {});
    }
    if xml.contains("<ping") || xml.contains("<rlean:ping") {
        return Ok(Request::Ping {});
    }
    // <rlean:request>...<rlean:search pattern="..."/>
    if let Some(inner) = extract_tag(xml, "rlean:search").or_else(|| extract_tag(xml, "search")) {
        let pattern = xml_attr(&inner, "pattern").ok_or("missing pattern")?;
        let limit = xml_attr(&inner, "limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(50);
        return Ok(Request::Search { pattern, limit });
    }
    Err("unrecognized XML request".into())
}

fn xml_attr(xml: &str, name: &str) -> Option<String> {
    let key = format!(r#"{name}=""#);
    let idx = xml.find(&key)?;
    let rest = &xml[idx + key.len()..];
    let end = rest.find('"')?;
    Some(rest[..end]
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&"))
}

fn extract_tag<'a>(xml: &'a str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let idx = xml.find(&open)?;
    let rest = &xml[idx..];
    if let Some(end) = rest.find("/>") {
        return Some(rest[..=end + 1].to_string());
    }
    let close = format!("</{tag}>");
    let end = rest.find(&close)?;
    Some(rest[..end + close.len()].to_string())
}

fn response_to_xml(resp: &Response) -> String {
    match resp {
        Response::Search {
            pattern,
            count,
            hits,
        } => {
            let mut out = format!(
                r#"<rlean:response xmlns:rlean="{ns}" type="search" pattern="{pat}" count="{count}">"#,
                ns = RLEAN_NS,
                pat = escape(pattern),
                count = count,
            );
            out.push('\n');
            for h in hits {
                out.push_str(&format!(
                    r#"  <rlean:hit name="{name}" full_name="{full}" kind="{kind}" file="{file}" line="{line}" score="{score}">
    <rlean:typeSurface>{ty}</rlean:typeSurface>
  </rlean:hit>
"#,
                    name = escape(&h.name),
                    full = escape(&h.full_name),
                    kind = escape(&h.kind),
                    file = escape(&h.file),
                    line = h.line,
                    score = h.score,
                    ty = escape(&h.type_surface),
                ));
            }
            out.push_str("</rlean:response>");
            out
        }
        Response::Stats {
            declarations,
            packages,
            source_hash,
        } => format!(
            r#"<rlean:response xmlns:rlean="{ns}" type="stats" declarations="{d}" packages="{p}" source_hash="{h}"/>"#,
            ns = RLEAN_NS,
            d = declarations,
            p = packages,
            h = escape(source_hash),
        ),
        Response::Ok { message } => format!(
            r#"<rlean:response xmlns:rlean="{ns}" type="ok" message="{m}"/>"#,
            ns = RLEAN_NS,
            m = escape(message),
        ),
        Response::Error { message } => format!(
            r#"<rlean:response xmlns:rlean="{ns}" type="error" message="{m}"/>"#,
            ns = RLEAN_NS,
            m = escape(message),
        ),
        Response::Pong {} => format!(
            r#"<rlean:response xmlns:rlean="{ns}" type="pong"/>"#,
            ns = RLEAN_NS
        ),
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
