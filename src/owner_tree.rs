use serde::Serialize;

use crate::{
    rtrim_iterator::RTrimIterator,
    sexp::{sexp_with_padding, Token},
};

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
    start_line: usize,
    end_line: usize, // Exclusive
    start_character: usize,
    end_character: usize, // Exclusive
}

fn build_ast_node<'a>(
    original_source: &str,
    parent_contents_begin: Option<usize>,
    current_token: &Token<'a>,
) -> Result<AstNode, Box<dyn std::error::Error>> {
    let maybe_plain_text = current_token.as_text();
    let ast_node = match maybe_plain_text {
        Ok(plain_text) => {
            let parent_contents_begin = parent_contents_begin
                .ok_or("parent_contents_begin should be set for all plain text nodes.")?;
            let mut parameters = plain_text.properties.iter();
            let begin = parent_contents_begin
                + maybe_token_to_usize(parameters.next())?
                    .ok_or("Missing first element past the text.")?;
            let end = parent_contents_begin
                + maybe_token_to_usize(parameters.next())?
                    .ok_or("Missing second element past the text.")?;
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
            let mut children = Vec::new();
            let original_contents_begin = get_contents_begin(current_token);
            match original_contents_begin {
                Ok(original_contents_begin) => {
                    let mut contents_begin = original_contents_begin;
                    for child in parameters.into_iter().skip(2) {
                        let new_ast_node =
                            build_ast_node(original_source, Some(contents_begin), child)?;
                        contents_begin = new_ast_node.position.end_character;
                        children.push(new_ast_node);
                    }
                }
                Err(_) => {
                    // Some nodes don't have a contents begin, so hopefully plain text can't be inside them.
                    for child in parameters.into_iter().skip(2) {
                        let new_ast_node = build_ast_node(original_source, None, child)?;
                        children.push(new_ast_node);
                    }
                }
            };

            AstNode {
                name: name.to_owned(),
                position,
                children,
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
    let standard_properties = get_standard_properties(emacs)?;
    let (begin, end) = (
        standard_properties
            .begin
            .ok_or("Token should have a begin.")?,
        standard_properties.end.ok_or("Token should have an end.")?,
    );
    let (start_line, end_line) = get_line_numbers(original_source, begin, end)?;
    Ok(SourceRange {
        start_line,
        end_line,
        start_character: begin,
        end_character: end,
    })
}

fn get_contents_begin<'s>(emacs: &'s Token<'s>) -> Result<usize, Box<dyn std::error::Error>> {
    let standard_properties = get_standard_properties(emacs)?;
    Ok(standard_properties
        .contents_begin
        .ok_or("Token should have a contents-begin.")?)
}

fn get_line_numbers<'s>(
    original_source: &'s str,
    begin: usize,
    end: usize,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    // This is used for highlighting which lines contain text relevant to the token, so even if a token does not extend all the way to the end of the line, the end_line figure will be the following line number (since the range is exclusive, not inclusive).
    let start_line = original_source
        .chars()
        .into_iter()
        .take(usize::try_from(begin)? - 1)
        .filter(|x| *x == '\n')
        .count()
        + 1;
    let end_line = {
        let content_up_to_and_including_token = original_source
            .chars()
            .into_iter()
            .take(usize::try_from(end)? - 1);
        // Remove the trailing newline (if there is one) because we're going to add an extra line regardless of whether or not this ends with a new line.
        let without_trailing_newline = RTrimIterator::new(content_up_to_and_including_token, '\n');
        without_trailing_newline.filter(|x| *x == '\n').count() + 2
    };

    Ok((usize::try_from(start_line)?, usize::try_from(end_line)?))
}

struct StandardProperties {
    begin: Option<usize>,
    #[allow(dead_code)]
    post_affiliated: Option<usize>,
    #[allow(dead_code)]
    contents_begin: Option<usize>,
    #[allow(dead_code)]
    contents_end: Option<usize>,
    end: Option<usize>,
    #[allow(dead_code)]
    post_blank: Option<usize>,
}

fn get_standard_properties<'s>(
    emacs: &'s Token<'s>,
) -> Result<StandardProperties, Box<dyn std::error::Error>> {
    let children = emacs.as_list()?;
    let attributes_child = children
        .iter()
        .nth(1)
        .ok_or("Should have an attributes child.")?;
    let attributes_map = attributes_child.as_map()?;
    let standard_properties = attributes_map.get(":standard-properties");
    Ok(if standard_properties.is_some() {
        let mut std_props = standard_properties
            .expect("if statement proves its Some")
            .as_vector()?
            .into_iter();
        let begin = maybe_token_to_usize(std_props.next())?;
        let post_affiliated = maybe_token_to_usize(std_props.next())?;
        let contents_begin = maybe_token_to_usize(std_props.next())?;
        let contents_end = maybe_token_to_usize(std_props.next())?;
        let end = maybe_token_to_usize(std_props.next())?;
        let post_blank = maybe_token_to_usize(std_props.next())?;
        StandardProperties {
            begin,
            post_affiliated,
            contents_begin,
            contents_end,
            end,
            post_blank,
        }
    } else {
        let begin = maybe_token_to_usize(attributes_map.get(":begin").map(|token| *token))?;
        let end = maybe_token_to_usize(attributes_map.get(":end").map(|token| *token))?;
        let contents_begin =
            maybe_token_to_usize(attributes_map.get(":contents-begin").map(|token| *token))?;
        let contents_end =
            maybe_token_to_usize(attributes_map.get(":contents-end").map(|token| *token))?;
        let post_blank =
            maybe_token_to_usize(attributes_map.get(":post-blank").map(|token| *token))?;
        let post_affiliated =
            maybe_token_to_usize(attributes_map.get(":post-affiliated").map(|token| *token))?;
        StandardProperties {
            begin,
            post_affiliated,
            contents_begin,
            contents_end,
            end,
            post_blank,
        }
    })
}

fn maybe_token_to_usize(
    token: Option<&Token<'_>>,
) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    Ok(token
        .map(|token| token.as_atom())
        .map_or(Ok(None), |r| r.map(Some))?
        .map(|val| {
            if val == "nil" {
                None
            } else {
                Some(val.parse::<usize>())
            }
        })
        .flatten() // Outer option is whether or not the param exists, inner option is whether or not it is nil
        .map_or(Ok(None), |r| r.map(Some))?)
}
