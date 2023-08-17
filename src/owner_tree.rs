use serde::Serialize;

use crate::sexp::sexp_with_padding;

pub fn build_owner_tree(file_contents: &str) -> Result<OwnerTree, Box<dyn std::error::Error + '_>> {
    let (_remaining, parsed_sexp) = sexp_with_padding(file_contents)?;

    Ok(OwnerTree {})
}

#[derive(Serialize)]
pub struct OwnerTree {}
