use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::escaped;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_till1;
use nom::character::complete::multispace0;
use nom::character::complete::multispace1;
use nom::character::complete::one_of;
use nom::combinator::map;
use nom::combinator::not;
use nom::combinator::opt;
use nom::combinator::peek;
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::tuple;

use crate::error::Res;

#[derive(Debug)]
pub enum Token<'s> {
    Atom(&'s str),
    List(Vec<Token<'s>>),
    TextWithProperties(TextWithProperties<'s>),
    Vector(Vec<Token<'s>>),
}

#[derive(Debug)]
pub struct TextWithProperties<'s> {
    #[allow(dead_code)]
    pub text: &'s str,
    #[allow(dead_code)]
    pub properties: Vec<Token<'s>>,
}

impl<'s> TextWithProperties<'s> {
    pub fn unquote(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut out = String::with_capacity(self.text.len());
        if !self.text.starts_with(r#"""#) {
            return Err("Quoted text does not start with quote.".into());
        }
        if !self.text.ends_with(r#"""#) {
            return Err("Quoted text does not end with quote.".into());
        }
        let interior_text = &self.text[1..(self.text.len() - 1)];
        let mut state = ParseState::Normal;
        for current_char in interior_text.chars().into_iter() {
            state = match (state, current_char) {
                (ParseState::Normal, '\\') => ParseState::Escape,
                (ParseState::Normal, _) => {
                    out.push(current_char);
                    ParseState::Normal
                }
                (ParseState::Escape, 'n') => {
                    out.push('\n');
                    ParseState::Normal
                }
                (ParseState::Escape, '\\') => {
                    out.push('\\');
                    ParseState::Normal
                }
                (ParseState::Escape, '"') => {
                    out.push('"');
                    ParseState::Normal
                }
                _ => todo!(),
            };
        }

        Ok(out)
    }
}

enum ParseState {
    Normal,
    Escape,
}

impl<'s> Token<'s> {
    pub fn as_vector<'p>(&'p self) -> Result<&'p Vec<Token<'s>>, Box<dyn std::error::Error>> {
        Ok(match self {
            Token::Vector(children) => Ok(children),
            _ => Err(format!("wrong token type {:?}", self)),
        }?)
    }

    pub fn as_list<'p>(&'p self) -> Result<&'p Vec<Token<'s>>, Box<dyn std::error::Error>> {
        Ok(match self {
            Token::List(children) => Ok(children),
            _ => Err(format!("wrong token type {:?}", self)),
        }?)
    }

    pub fn as_atom<'p>(&'p self) -> Result<&'s str, Box<dyn std::error::Error>> {
        Ok(match self {
            Token::Atom(body) => Ok(*body),
            _ => Err(format!("wrong token type {:?}", self)),
        }?)
    }

    pub fn as_text<'p>(&'p self) -> Result<&'p TextWithProperties<'s>, Box<dyn std::error::Error>> {
        Ok(match self {
            Token::TextWithProperties(body) => Ok(body),
            _ => Err(format!("wrong token type {:?}", self)),
        }?)
    }

    pub fn as_map<'p>(
        &'p self,
    ) -> Result<HashMap<&'s str, &'p Token<'s>>, Box<dyn std::error::Error>> {
        let mut hashmap = HashMap::new();

        let children = self.as_list()?;
        if children.len() % 2 != 0 {
            return Err("Expecting an even number of children".into());
        }
        let mut key: Option<&str> = None;
        for child in children.iter() {
            match key {
                None => {
                    key = Some(child.as_atom()?);
                }
                Some(key_val) => {
                    key = None;
                    hashmap.insert(key_val, child);
                }
            };
        }

        Ok(hashmap)
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(ret, level = "debug"))]
pub fn sexp_with_padding<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    let (remaining, _) = multispace0(input)?;
    let (remaining, tkn) = token(remaining)?;
    let (remaining, _) = multispace0(remaining)?;
    Ok((remaining, tkn))
}

#[cfg_attr(feature = "tracing", tracing::instrument(ret, level = "debug"))]
pub fn sexp<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    let (remaining, tkn) = token(input)?;
    Ok((remaining, tkn))
}

#[cfg_attr(feature = "tracing", tracing::instrument(ret, level = "debug"))]
fn token<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    alt((list, vector, atom))(input)
}

#[cfg_attr(feature = "tracing", tracing::instrument(ret, level = "debug"))]
fn list<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    let (remaining, _) = tag("(")(input)?;
    let (remaining, children) = delimited(
        multispace0,
        separated_list1(multispace1, token),
        multispace0,
    )(remaining)?;
    let (remaining, _) = tag(")")(remaining)?;
    Ok((remaining, Token::List(children)))
}

#[cfg_attr(feature = "tracing", tracing::instrument(ret, level = "debug"))]
fn vector<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    let (remaining, _) = tag("[")(input)?;
    let (remaining, children) = delimited(
        multispace0,
        separated_list1(multispace1, token),
        multispace0,
    )(remaining)?;
    let (remaining, _) = tag("]")(remaining)?;
    Ok((remaining, Token::Vector(children)))
}

#[cfg_attr(feature = "tracing", tracing::instrument(ret, level = "debug"))]
fn atom<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    not(peek(one_of(")]")))(input)?;
    alt((
        text_with_properties,
        hash_notation,
        quoted_atom,
        unquoted_atom,
    ))(input)
}

#[cfg_attr(feature = "tracing", tracing::instrument(ret, level = "debug"))]
fn unquoted_atom<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    let (remaining, body) = take_till1(|c| match c {
        ' ' | '\t' | '\r' | '\n' | ')' | ']' => true,
        _ => false,
    })(input)?;
    Ok((remaining, Token::Atom(body)))
}

#[cfg_attr(feature = "tracing", tracing::instrument(ret, level = "debug"))]
fn quoted_atom<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    let (remaining, _) = tag(r#"""#)(input)?;
    let (remaining, _) = escaped(
        take_till1(|c| match c {
            '\\' | '"' => true,
            _ => false,
        }),
        '\\',
        one_of(r#""n\\"#),
    )(remaining)?;
    let (remaining, _) = tag(r#"""#)(remaining)?;
    let source = get_consumed(input, remaining);
    Ok((remaining, Token::Atom(source)))
}

#[cfg_attr(feature = "tracing", tracing::instrument(ret, level = "debug"))]
fn hash_notation<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    let (remaining, _) = tag("#<")(input)?;
    let (remaining, _body) = take_till1(|c| match c {
        '>' => true,
        _ => false,
    })(remaining)?;
    let (remaining, _) = tag(">")(remaining)?;
    let source = get_consumed(input, remaining);
    Ok((remaining, Token::Atom(source)))
}

fn text_with_properties<'s>(input: &'s str) -> Res<&'s str, Token<'s>> {
    let (remaining, _) = tag("#(")(input)?;
    let (remaining, (text, props)) = delimited(
        multispace0,
        tuple((
            map(quoted_atom, |atom| match atom {
                Token::Atom(body) => body,
                _ => unreachable!(),
            }),
            preceded(multispace1, opt(separated_list1(multispace1, token))),
        )),
        multispace0,
    )(remaining)?;
    let (remaining, _) = tag(")")(remaining)?;
    Ok((
        remaining,
        Token::TextWithProperties(TextWithProperties {
            text,
            properties: props.unwrap_or(Vec::new()),
        }),
    ))
}

/// Get a slice of the string that was consumed in a parser using the original input to the parser and the remaining input after the parser.
fn get_consumed<'s>(input: &'s str, remaining: &'s str) -> &'s str {
    assert!(is_slice_of(input, remaining));
    let source = {
        let offset = remaining.as_ptr() as usize - input.as_ptr() as usize;
        &input[..offset]
    };
    source
}

/// Check if the child string slice is a slice of the parent string slice.
fn is_slice_of(parent: &str, child: &str) -> bool {
    let parent_start = parent.as_ptr() as usize;
    let parent_end = parent_start + parent.len();
    let child_start = child.as_ptr() as usize;
    let child_end = child_start + child.len();
    child_start >= parent_start && child_end <= parent_end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let input = "  (foo bar baz )  ";
        let (remaining, parsed) = sexp_with_padding(input).expect("Parse the input");
        assert_eq!(remaining, "");
        assert!(match parsed {
            Token::Atom(_) => false,
            Token::List(_) => true,
            Token::TextWithProperties(_) => false,
            Token::Vector(_) => false,
        });
    }

    #[test]
    fn quoted() {
        let input = r#"  ("foo" bar baz )  "#;
        let (remaining, parsed) = sexp_with_padding(input).expect("Parse the input");
        assert_eq!(remaining, "");
        assert!(match parsed {
            Token::Atom(_) => false,
            Token::List(_) => true,
            Token::TextWithProperties(_) => false,
            Token::Vector(_) => false,
        });
        let children = match parsed {
            Token::List(children) => children,
            _ => panic!("Should be a list."),
        };
        assert_eq!(
            match children.first() {
                Some(Token::Atom(body)) => *body,
                _ => panic!("First child should be an atom."),
            },
            r#""foo""#
        )
    }

    #[test]
    fn quoted_containing_paren() {
        let input = r#"  (foo "b(a)r" baz )  "#;
        let (remaining, parsed) = sexp_with_padding(input).expect("Parse the input");
        assert_eq!(remaining, "");
        assert!(match parsed {
            Token::List(_) => true,
            _ => false,
        });
        let children = match parsed {
            Token::List(children) => children,
            _ => panic!("Should be a list."),
        };
        assert_eq!(
            match children.first() {
                Some(Token::Atom(body)) => *body,
                _ => panic!("First child should be an atom."),
            },
            r#"foo"#
        );
        assert_eq!(
            match children.iter().nth(1) {
                Some(Token::Atom(body)) => *body,
                _ => panic!("Second child should be an atom."),
            },
            r#""b(a)r""#
        );
        assert_eq!(
            match children.iter().nth(2) {
                Some(Token::Atom(body)) => *body,
                _ => panic!("Third child should be an atom."),
            },
            r#"baz"#
        );
    }

    #[test]
    fn string_containing_escaped_characters() {
        let input = r#"  (foo "\\( x=2 \\)" bar)  "#;
        let (remaining, parsed) = sexp_with_padding(input).expect("Parse the input");
        assert_eq!(remaining, "");
        assert!(match parsed {
            Token::Atom(_) => false,
            Token::List(_) => true,
            Token::TextWithProperties(_) => false,
            Token::Vector(_) => false,
        });
        let children = match parsed {
            Token::List(children) => children,
            _ => panic!("Should be a list."),
        };
        assert_eq!(
            match children.get(1) {
                Some(Token::Atom(body)) => *body,
                _ => panic!("First child should be an atom."),
            },
            r#""\\( x=2 \\)""#
        )
    }
}
