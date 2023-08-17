use nom::error::ErrorKind;
use nom::error::ParseError;
use nom::IResult;

pub type Res<T, U> = IResult<T, U, CustomError<T>>;

#[derive(Debug, PartialEq)]
pub enum CustomError<I> {
    MyError(MyError<I>),
    Nom(I, ErrorKind),
}

#[derive(Debug, PartialEq)]
pub struct MyError<I>(pub I);

impl<I> ParseError<I> for CustomError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        CustomError::Nom(input, kind)
    }

    fn append(_input: I, _kind: ErrorKind, /*mut*/ other: Self) -> Self {
        // Doesn't do append like VerboseError
        other
    }
}
