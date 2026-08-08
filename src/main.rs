//! CLI for rlean-search.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rlean_search::cache::{self, default_cache_path};
use rlean_search::daemon::{self, DaemonConfig};
use rlean_search::index::build_index;
use rlean_search::protocol::{format_response, ProtocolKind, Response};
use rlean_search::xml::write_index_file;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "rlean-search",
    about = "Type-aware search over Lean 4 theorems, lemmas, and axioms",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Increase logging verbosity
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Index Lean sources (Lake-aware) and write an XML cache
    Index {
        /// Paths to Lake packages or individual .lean files
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Output index XML path (default: <first-path>/.rlean-search/index.xml)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Search types (uses cache when available)
    Search {
        /// Type pattern, e.g. `_ + _ = 0`, `?a - ?a = 0`, `|- tsum _ = _ * tsum _`
        pattern: String,

        /// Paths to search / index (default: current directory)
        #[arg(short, long)]
        path: Vec<PathBuf>,

        /// Cache file to use / write
        #[arg(long)]
        cache: Option<PathBuf>,

        /// Force rebuild of the index even if cache exists
        #[arg(long)]
        rebuild: bool,

        /// Maximum hits
        #[arg(short = 'n', long, default_value_t = 50)]
        limit: usize,

        /// Output format: text, jsonl, xml
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Run multi-client daemon (Tokio); keeps the index in memory
    Daemon {
        /// Bind address, e.g. 127.0.0.1:7878
        #[arg(long, default_value = "127.0.0.1:7878")]
        bind: String,

        /// Paths to index
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Cache file to load/save
        #[arg(long)]
        cache: Option<PathBuf>,
    },

    /// Send one request to a running daemon (JSONL or XML line)
    Query {
        /// Daemon address
        #[arg(long, default_value = "127.0.0.1:7878")]
        addr: String,

        /// Request payload (JSON object or XML). If omitted, uses --pattern.
        request: Option<String>,

        /// Convenience: build a JSONL search request from a pattern
        #[arg(short, long)]
        pattern: Option<String>,

        /// Use XML protocol instead of JSONL when combined with --pattern
        #[arg(long)]
        xml: bool,

        #[arg(short = 'n', long, default_value_t = 50)]
        limit: usize,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Index { paths, output } => {
            let out = output.unwrap_or_else(|| {
                default_cache_path(paths.first().map(|p| p.as_path()).unwrap_or(std::path::Path::new(".")))
            });
            tracing::info!("indexing {} path(s)...", paths.len());
            let idx = build_index(&paths)?;
            write_index_file(&out, &idx.doc)?;
            println!(
                "wrote {} declarations to {} (hash {})",
                idx.len(),
                out.display(),
                idx.doc.source_hash
            );
            Ok(())
        }
        Commands::Search {
            pattern,
            path,
            cache,
            rebuild,
            limit,
            format,
        } => {
            let paths = if path.is_empty() {
                vec![PathBuf::from(".")]
            } else {
                path
            };
            let cache_path = cache.unwrap_or_else(|| default_cache_path(&paths[0]));
            let idx = cache::load_or_build(&paths, &cache_path, rebuild)
                .with_context(|| format!("index/cache {}", cache_path.display()))?;
            let hits = idx.search(&pattern, limit)?;
            match format.as_str() {
                "jsonl" | "json" => {
                    let resp = Response::Search {
                        pattern: pattern.clone(),
                        count: hits.len(),
                        hits,
                    };
                    print!("{}", format_response(ProtocolKind::Jsonl, &resp));
                }
                "xml" => {
                    let resp = Response::Search {
                        pattern: pattern.clone(),
                        count: hits.len(),
                        hits,
                    };
                    print!("{}", format_response(ProtocolKind::Xml, &resp));
                }
                _ => {
                    if hits.is_empty() {
                        println!("No matches for: {pattern}");
                    } else {
                        println!("{} hit(s) for: {pattern}", hits.len());
                        for h in hits {
                            println!(
                                "  {} {}  {}:{}",
                                h.kind, h.full_name, h.file, h.line
                            );
                            println!("    {}", h.type_surface);
                        }
                    }
                }
            }
            Ok(())
        }
        Commands::Daemon {
            bind,
            paths,
            cache,
        } => {
            let cache_path = cache.or_else(|| {
                paths
                    .first()
                    .map(|p| default_cache_path(p))
            });
            daemon::run_daemon(DaemonConfig {
                bind,
                paths,
                cache_path,
            })
            .await
        }
        Commands::Query {
            addr,
            request,
            pattern,
            xml,
            limit,
        } => {
            let line = if let Some(r) = request {
                r
            } else if let Some(p) = pattern {
                if xml {
                    format!(
                        r#"<rlean:search xmlns:rlean="{}" pattern="{}" limit="{limit}"/>"#,
                        rlean_search::RLEAN_NS,
                        escape_xml(&p),
                        limit = limit
                    )
                } else {
                    serde_json::json!({
                        "cmd": "search",
                        "pattern": p,
                        "limit": limit
                    })
                    .to_string()
                }
            } else {
                anyhow::bail!("provide a request string or --pattern");
            };
            let resp = daemon::client_query(&addr, &line).await?;
            print!("{resp}");
            if !resp.ends_with('\n') {
                println!();
            }
            Ok(())
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
