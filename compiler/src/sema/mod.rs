//! Semantic analysis: the class table, then the checked form of every body.

pub mod check;
pub mod decl;
pub mod program;
pub mod tir;
pub mod types;

use crate::ast;
use program::Program;

/// Checks a whole program. The diagnostics are in the result.
pub fn analyze(units: Vec<ast::Unit>) -> Program {
    let mut p = decl::build(units);
    if p.diags.is_empty() || p.wk.object != p.wk.null {
        check::check_bodies(&mut p);
    }
    p
}
