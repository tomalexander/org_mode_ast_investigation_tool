use serde::Serialize;

use crate::sexp::{sexp_with_padding, Token};

pub fn build_owner_tree<'a>(
    body: &'a str,
    ast_raw: &'a str,
) -> Result<OwnerTree, Box<dyn std::error::Error + 'a>> {
    let (_remaining, parsed_sexp) = sexp_with_padding(ast_raw)?;
    let lists = find_lists_in_document(&parsed_sexp)?;

    Ok(OwnerTree {
        ast: ast_raw.to_owned(),
        children: lists,
    })
}

#[derive(Serialize)]
pub struct OwnerTree {
    ast: String,
    children: Vec<PlainList>,
}

#[derive(Serialize)]
pub struct PlainList {
    position: SourceRange,
    children: Vec<PlainListItem>,
}

#[derive(Serialize)]
pub struct PlainListItem {
    position: SourceRange,
    children: Vec<PlainListItem>,
}

#[derive(Serialize)]
pub struct SourceRange {
    start_line: u32,
    end_line: u32, // Exclusive
    start_character: u32,
    end_character: u32, // Exclusive
}

fn find_lists_in_document<'a>(
    current_token: &Token<'a>,
) -> Result<Vec<PlainList>, Box<dyn std::error::Error>> {
    // DFS looking for top-level lists

    let mut found_lists = Vec::new();
    let children = current_token.as_list()?;
    let token_name = "org-data";
    assert_name(current_token, token_name)?;

    // skip 2 to skip token name and standard properties
    for child_token in children.iter().skip(2) {
        found_lists.extend(recurse_token(child_token)?);
    }

    Ok(found_lists)
}

fn assert_name<'s>(emacs: &'s Token<'s>, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let children = emacs.as_list()?;
    let first_child = children
        .first()
        .ok_or("Should have at least one child.")?
        .as_atom()?;
    if first_child != name {
        Err(format!(
            "Expected a {expected} cell, but found a {found} cell.",
            expected = name,
            found = first_child
        ))?;
    }
    Ok(())
}

fn recurse_token<'a>(
    current_token: &Token<'a>,
) -> Result<Vec<PlainList>, Box<dyn std::error::Error>> {
    match current_token {
        Token::Atom(_) | Token::TextWithProperties(_) => Ok(Vec::new()),
        Token::List(_) => {
            let new_lists = find_lists_in_list(current_token)?;
            Ok(new_lists)
        }
        Token::Vector(_) => {
            let new_lists = find_lists_in_vector(current_token)?;
            Ok(new_lists)
        }
    }
}

fn find_lists_in_list<'a>(
    current_token: &Token<'a>,
) -> Result<Vec<PlainList>, Box<dyn std::error::Error>> {
    let mut found_lists = Vec::new();
    let children = current_token.as_list()?;
    if assert_name(current_token, "plain-list").is_ok() {
        // Found a list!
    }

    // skip 2 to skip token name and standard properties
    for child_token in children.iter().skip(2) {
        found_lists.extend(recurse_token(child_token)?);
    }

    Ok(found_lists)
}

fn find_lists_in_vector<'a>(
    current_token: &Token<'a>,
) -> Result<Vec<PlainList>, Box<dyn std::error::Error>> {
    let mut found_lists = Vec::new();
    let children = current_token.as_vector()?;

    for child_token in children.iter() {
        found_lists.extend(recurse_token(child_token)?);
    }

    Ok(found_lists)
}
