use serde::Serialize;

use crate::sexp::{sexp_with_padding, Token};

pub fn build_owner_tree<'a>(
    body: &'a str,
    ast_raw: &'a str,
) -> Result<OwnerTree, Box<dyn std::error::Error + 'a>> {
    let (_remaining, parsed_sexp) = sexp_with_padding(ast_raw)?;
    let lists = find_lists_in_document(body, &parsed_sexp)?;

    Ok(OwnerTree {
        input: body.to_owned(),
        ast: ast_raw.to_owned(),
        lists,
    })
}

#[derive(Serialize)]
pub struct OwnerTree {
    input: String,
    ast: String,
    lists: Vec<PlainList>,
}

#[derive(Serialize)]
pub struct PlainList {
    position: SourceRange,
    items: Vec<PlainListItem>,
}

#[derive(Serialize)]
pub struct PlainListItem {
    position: SourceRange,
    lists: Vec<PlainList>,
}

#[derive(Serialize)]
pub struct SourceRange {
    start_line: u32,
    end_line: u32, // Exclusive
    start_character: u32,
    end_character: u32, // Exclusive
}

fn find_lists_in_document<'a>(
    original_source: &str,
    current_token: &Token<'a>,
) -> Result<Vec<PlainList>, Box<dyn std::error::Error>> {
    // DFS looking for top-level lists

    let mut found_lists = Vec::new();
    let children = current_token.as_list()?;
    let token_name = "org-data";
    assert_name(current_token, token_name)?;

    // skip 2 to skip token name and standard properties
    for child_token in children.iter().skip(2) {
        found_lists.extend(recurse_token(original_source, child_token)?);
    }

    Ok(found_lists)
}

fn recurse_token<'a>(
    original_source: &str,
    current_token: &Token<'a>,
) -> Result<Vec<PlainList>, Box<dyn std::error::Error>> {
    match current_token {
        Token::Atom(_) | Token::TextWithProperties(_) => Ok(Vec::new()),
        Token::List(_) => {
            let new_lists = find_lists_in_list(original_source, current_token)?;
            Ok(new_lists)
        }
        Token::Vector(_) => {
            let new_lists = find_lists_in_vector(original_source, current_token)?;
            Ok(new_lists)
        }
    }
}

fn find_lists_in_list<'a>(
    original_source: &str,
    current_token: &Token<'a>,
) -> Result<Vec<PlainList>, Box<dyn std::error::Error>> {
    let mut found_lists = Vec::new();
    let children = current_token.as_list()?;
    if assert_name(current_token, "plain-list").is_ok() {
        // Found a list!
        let mut found_items = Vec::new();
        // skip 2 to skip token name and standard properties
        for child_token in children.iter().skip(2) {
            found_items.push(get_item_in_list(original_source, child_token)?);
        }

        found_lists.push(PlainList {
            position: get_bounds(original_source, current_token)?,
            items: found_items,
        });
    } else {
        // skip 2 to skip token name and standard properties
        for child_token in children.iter().skip(2) {
            found_lists.extend(recurse_token(original_source, child_token)?);
        }
    }

    Ok(found_lists)
}

fn find_lists_in_vector<'a>(
    original_source: &str,
    current_token: &Token<'a>,
) -> Result<Vec<PlainList>, Box<dyn std::error::Error>> {
    let mut found_lists = Vec::new();
    let children = current_token.as_vector()?;

    for child_token in children.iter() {
        found_lists.extend(recurse_token(original_source, child_token)?);
    }

    Ok(found_lists)
}

fn get_item_in_list<'a>(
    original_source: &str,
    current_token: &Token<'a>,
) -> Result<PlainListItem, Box<dyn std::error::Error>> {
    let mut found_lists = Vec::new();
    let children = current_token.as_list()?;
    let token_name = "item";
    assert_name(current_token, token_name)?;

    // skip 2 to skip token name and standard properties
    for child_token in children.iter().skip(2) {
        found_lists.extend(recurse_token(original_source, child_token)?);
    }

    Ok(PlainListItem {
        position: get_bounds(original_source, current_token)?,
        lists: found_lists,
    })
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

fn get_bounds<'s>(
    original_source: &'s str,
    emacs: &'s Token<'s>,
) -> Result<SourceRange, Box<dyn std::error::Error>> {
    let children = emacs.as_list()?;
    let attributes_child = children
        .iter()
        .nth(1)
        .ok_or("Should have an attributes child.")?;
    let attributes_map = attributes_child.as_map()?;
    let standard_properties = attributes_map.get(":standard-properties");
    let (begin, end) = if standard_properties.is_some() {
        let std_props = standard_properties
            .expect("if statement proves its Some")
            .as_vector()?;
        let begin = std_props
            .get(0)
            .ok_or("Missing first element in standard properties")?
            .as_atom()?;
        let end = std_props
            .get(1)
            .ok_or("Missing first element in standard properties")?
            .as_atom()?;
        (begin, end)
    } else {
        let begin = attributes_map
            .get(":begin")
            .ok_or("Missing :begin attribute.")?
            .as_atom()?;
        let end = attributes_map
            .get(":end")
            .ok_or("Missing :end attribute.")?
            .as_atom()?;
        (begin, end)
    };
    let begin = begin.parse::<u32>()?;
    let end = end.parse::<u32>()?;
    let start_line = original_source
        .chars()
        .into_iter()
        .take(usize::try_from(begin)? - 1)
        .filter(|x| *x == '\n')
        .count()
        + 1;
    let end_line = original_source
        .chars()
        .into_iter()
        .take(usize::try_from(end)? - 1)
        .filter(|x| *x == '\n')
        .count()
        + 1;
    Ok(SourceRange {
        start_line: u32::try_from(start_line)?,
        end_line: u32::try_from(end_line)?,
        start_character: begin,
        end_character: end,
    })
}
