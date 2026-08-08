//! Tokio multi-client daemon serving JSONL / XML type search.

use crate::index::{build_index, shared_index, SearchIndex, SharedIndex};
use crate::protocol::{format_response, parse_request, ProtocolKind, Request, Response};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
pub struct DaemonConfig {
    pub bind: String,
    pub paths: Vec<PathBuf>,
    pub cache_path: Option<PathBuf>,
}

pub async fn run_daemon(cfg: DaemonConfig) -> Result<()> {
    let idx = if let Some(cache) = &cfg.cache_path {
        crate::cache::load_or_build(&cfg.paths, cache, false)?
    } else {
        build_index(&cfg.paths)?
    };
    tracing::info!(
        "indexed {} declarations from {} package(s)",
        idx.len(),
        idx.doc.packages.len()
    );
    let shared = shared_index(idx);
    let paths = Arc::new(cfg.paths.clone());
    let cache_path = cfg.cache_path.clone();
    let listener = TcpListener::bind(&cfg.bind)
        .await
        .with_context(|| format!("bind {}", cfg.bind))?;
    tracing::info!("rlean-search daemon listening on {}", cfg.bind);

    loop {
        let (socket, peer) = listener.accept().await?;
        tracing::debug!("connection from {peer}");
        let shared = Arc::clone(&shared);
        let paths = Arc::clone(&paths);
        let cache_path = cache_path.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, shared, paths, cache_path).await {
                tracing::debug!("client error: {e}");
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    shared: SharedIndex,
    paths: Arc<Vec<PathBuf>>,
    cache_path: Option<PathBuf>,
) -> Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let (kind, req) = match parse_request(&line) {
            Ok(v) => v,
            Err(e) => {
                let resp = Response::Error { message: e };
                writer
                    .write_all(format_response(ProtocolKind::detect(&line), &resp).as_bytes())
                    .await?;
                continue;
            }
        };
        let resp = process_request(&req, &shared, &paths, cache_path.as_deref()).await;
        writer
            .write_all(format_response(kind, &resp).as_bytes())
            .await?;
    }
    Ok(())
}

async fn process_request(
    req: &Request,
    shared: &SharedIndex,
    paths: &[PathBuf],
    cache_path: Option<&Path>,
) -> Response {
    match req {
        Request::Ping {} => Response::Pong {},
        Request::Stats {} => {
            let idx = shared.read();
            Response::Stats {
                declarations: idx.len(),
                packages: idx.doc.packages.len(),
                source_hash: idx.doc.source_hash.clone(),
            }
        }
        Request::Search { pattern, limit } => {
            let idx = shared.read();
            match idx.search(pattern, *limit) {
                Ok(hits) => Response::Search {
                    pattern: pattern.clone(),
                    count: hits.len(),
                    hits,
                },
                Err(e) => Response::Error {
                    message: e.to_string(),
                },
            }
        }
        Request::Reload {} => {
            let paths = paths.to_vec();
            let cache_path = cache_path.map(|p| p.to_path_buf());
            let result = tokio::task::spawn_blocking(move || {
                if let Some(c) = &cache_path {
                    crate::cache::load_or_build(&paths, c, true)
                } else {
                    build_index(&paths)
                }
            })
            .await;
            match result {
                Ok(Ok(new_idx)) => {
                    let n = new_idx.len();
                    *shared.write() = new_idx;
                    Response::Ok {
                        message: format!("reloaded {n} declarations"),
                    }
                }
                Ok(Err(e)) => Response::Error {
                    message: e.to_string(),
                },
                Err(e) => Response::Error {
                    message: e.to_string(),
                },
            }
        }
    }
}

/// One-shot TCP client helper used by CLI `query` against a running daemon.
pub async fn client_query(addr: &str, line: &str) -> Result<String> {
    let mut stream = TcpStream::connect(addr)
        .await
        .with_context(|| format!("connect {addr}"))?;
    stream.write_all(line.as_bytes()).await?;
    if !line.ends_with('\n') {
        stream.write_all(b"\n").await?;
    }
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;
    // XML responses may be multi-line; read until we have a full root element or JSON line done
    if response.trim_start().starts_with('<') && !response.contains("</rlean:response>") && !response.trim_end().ends_with("/>")
    {
        loop {
            let mut more = String::new();
            let n = reader.read_line(&mut more).await?;
            if n == 0 {
                break;
            }
            response.push_str(&more);
            if response.contains("</rlean:response>") {
                break;
            }
        }
    }
    Ok(response)
}

/// Process a single request against an in-memory index (no network).
pub fn local_request(idx: &SearchIndex, line: &str) -> String {
    match parse_request(line) {
        Ok((kind, req)) => {
            let resp = match req {
                Request::Ping {} => Response::Pong {},
                Request::Stats {} => Response::Stats {
                    declarations: idx.len(),
                    packages: idx.doc.packages.len(),
                    source_hash: idx.doc.source_hash.clone(),
                },
                Request::Search { pattern, limit } => match idx.search(&pattern, limit) {
                    Ok(hits) => Response::Search {
                        pattern,
                        count: hits.len(),
                        hits,
                    },
                    Err(e) => Response::Error {
                        message: e.to_string(),
                    },
                },
                Request::Reload {} => Response::Error {
                    message: "reload not supported in one-shot mode".into(),
                },
            };
            format_response(kind, &resp)
        }
        Err(e) => format_response(
            ProtocolKind::detect(line),
            &Response::Error { message: e },
        ),
    }
}
