//! rlean-search: type-aware search over Lean 4 theorems, lemmas, and axioms.
//!
//! Schema namespace: `http://github.com/createyourpersonalaccount/rlean-search`

pub mod ast;
pub mod cache;
pub mod daemon;
pub mod index;
pub mod lake;
pub mod lexer;
pub mod parser;
pub mod protocol;
pub mod search;
pub mod xml;

pub use ast::{DeclKind, Declaration, IndexDocument, TypeExpr, RLEAN_NS};
pub use index::{build_index, SearchIndex};
pub use search::{matches_decl, matches_type, SearchHit};
