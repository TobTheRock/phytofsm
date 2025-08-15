use nom::{
    IResult, Parser,
    character::complete::{alphanumeric1, line_ending, multispace0, space0, space1},
    combinator::recognize,
    error::ParseError,
    multi::separated_list1,
    sequence::{delimited, terminated},
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

