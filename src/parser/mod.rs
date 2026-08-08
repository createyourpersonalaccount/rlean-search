//! Parsers for Lean type expressions and theorem/lemma/axiom declarations.

mod decl;
mod type_expr;

pub use decl::{parse_declarations, parse_declarations_with_path};
pub use type_expr::{parse_type, parse_search_pattern, SearchPattern};
