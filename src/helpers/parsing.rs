use std::str;
use std::{fmt::Debug, marker::PhantomData};

use nom::combinator::{iterator, ParserIterator};
use nom::sequence::terminated;
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::{complete::line_ending, is_digit},
    combinator::{all_consuming, eof, map_res},
    error::{Error, ParseError},
    multi::many0,
    Err, Parser,
};

pub type ParsingError<'a> = nom::Err<nom::error::Error<&'a [u8]>>;

pub trait AocParsable: Sized + Debug {
    fn parse_from_string(input: &[u8]) -> Result<(&[u8], Self), ParsingError>;
}

pub trait AocLineParsable: Sized + Debug {
    fn parse_from_line(input: &[u8]) -> Result<(&[u8], Self), ParsingError>;
}

impl <T: AocLineParsable> AocParsable for T {
    fn parse_from_string(input: &[u8]) -> Result<(&[u8], Self), ParsingError> {
        terminated(Self::parse_from_line, line_ending_or_eof())(input)
    }
}

pub struct AocParser<T: AocParsable> {
    output: PhantomData<T>,
}

impl<'a, T: AocParsable> Parser<&'a [u8], T, Error<&'a [u8]>> for AocParser<T> {
    fn parse(&mut self, input: &'a [u8]) -> nom::IResult<&'a [u8], T, Error<&'a [u8]>> {
        T::parse_from_string(input)
    }
}

pub fn parse_all<T: AocParsable>(input: &[u8]) -> Result<Vec<T>, Err<Error<&[u8]>>> {
    let parser = AocParser {
        output: PhantomData,
    };
    all_consuming(many0(parser))(input).map(|(_, result)| result)
}

pub fn iterate_all<T: AocParsable>(
    input: &[u8],
) -> ParserIterator<
    &[u8],
    Error<&[u8]>,
    fn(&[u8]) -> Result<(&[u8], T), ParsingError>,
> {
    iterator(input, T::parse_from_string)
}

macro_rules! text_parser_for_unsigned_int {
    ($name:ident for $utype:ty) => {
        pub fn $name<'a>() -> impl Parser<&'a [u8], $utype, Error<&'a [u8]>> {
            map_res(take_while1(is_digit), |digit_str: &'a [u8]| {
                str::from_utf8(digit_str).unwrap().parse::<$utype>()
            })
        }
    };
}

text_parser_for_unsigned_int!(text_u8 for u8);
text_parser_for_unsigned_int!(text_u16 for u16);
text_parser_for_unsigned_int!(text_u32 for u32);
text_parser_for_unsigned_int!(text_u64 for u64);
text_parser_for_unsigned_int!(text_usize for usize);

pub fn line_ending_or_eof<'a, E: ParseError<&'a [u8]>>() -> impl Parser<&'a [u8], &'a [u8], E> {
    alt((line_ending, eof))
}

pub fn generic_error_for_input<T>(
    input: &[u8],
) -> Result<T, ParsingError> {
    Err(nom::Err::Error(nom::error::Error {
        input,
        code: nom::error::ErrorKind::Fail,
    }))
}
