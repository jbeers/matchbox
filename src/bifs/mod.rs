use crate::env::Environment;

pub fn register_bifs(_env: &mut Environment) {
    // We can simulate BIFs as special AST nodes, or we can handle them in the evaluator.
    // For this POC, let's treat BIFs as native Rust functions.
}
