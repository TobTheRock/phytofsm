use nom::{
    IResult, Parser,
    character::complete::{multispace0, space0},
    error::ParseError,
    sequence::delimited,
};
use nom_language::error::VerboseError;

pub type NomResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

pub fn ws<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(space0, inner, space0)
}

pub fn multi_ws<'a, O, E: ParseError<&'a str>, F>(
    inner: F,
) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}
