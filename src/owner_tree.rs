use serde::Serialize;

use crate::sexp::{sexp_with_padding, Token};

pub fn build_owner_tree<'a>(
    body: &'a str,
    ast_raw: &'a str,
) -> Result<OwnerTree, Box<dyn std::error::Error + 'a>> {
    let (_remaining, parsed_sexp) = sexp_with_padding(ast_raw)?;
    assert_name(&parsed_sexp, "org-data")?;
    let ast_node = build_ast_node(body, None, &parsed_sexp)?;

    Ok(OwnerTree {
        input: body.to_owned(),
        ast: ast_raw.to_owned(),
        tree: ast_node,
    })
}

#[derive(Serialize)]
pub struct OwnerTree {
    input: String,
    ast: String,
    tree: AstNode,
}

#[derive(Serialize)]
pub struct AstNode {
    name: String,
    position: SourceRange,
    children: Vec<AstNode>,
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

fn build_ast_node<'a>(
    original_source: &str,
    parent_range: Option<&SourceRange>,
    current_token: &Token<'a>,
) -> Result<AstNode, Box<dyn std::error::Error>> {
    let maybe_plain_text = current_token.as_text();
    let ast_node = match maybe_plain_text {
        Ok(plain_text) => {
            let parent_range =
                parent_range.ok_or("parent_range should be set for all plain text nodes.")?;
            let parameters = &plain_text.properties;
            let begin = parent_range.start_character
                + parameters
                    .get(0)
                    .ok_or("Missing first element past the text.")?
                    .as_atom()?
                    .parse::<u32>()?;
            let end = parent_range.start_character
                + parameters
                    .get(1)
                    .ok_or("Missing second element past the text.")?
                    .as_atom()?
                    .parse::<u32>()?;
            let (start_line, end_line) = get_line_numbers(original_source, begin, end)?;
            AstNode {
                name: "plain-text".to_owned(),
                position: SourceRange {
                    start_line,
                    end_line,
                    start_character: begin,
                    end_character: end,
                },
                children: Vec::new(),
            }
        }
        Err(_) => {
            // Not plain text, so it must be a list
            let parameters = current_token.as_list()?;
            let name = parameters
                .first()
                .ok_or("Should have at least one child.")?
                .as_atom()?;
            let position = get_bounds(original_source, current_token)?;
            let children: Result<Vec<_>, _> = parameters
                .iter()
                .skip(2)
                .map(|param| build_ast_node(original_source, Some(&position), param))
                .collect();

            AstNode {
                name: name.to_owned(),
                position,
                children: children?,
            }
        }
    };

    Ok(ast_node)
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
    let (start_line, end_line) = get_line_numbers(original_source, begin, end)?;
    Ok(SourceRange {
        start_line,
        end_line,
        start_character: begin,
        end_character: end,
    })
}

fn get_line_numbers<'s>(
    original_source: &'s str,
    begin: u32,
    end: u32,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
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
    Ok((u32::try_from(start_line)?, u32::try_from(end_line)?))
}
