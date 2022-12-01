use std::{marker::PhantomData, fmt::Debug};
use std::str;

use nom::{
    bytes::complete::take_while1,
    character::{is_digit, complete::line_ending},
    combinator::{all_consuming, map_res, eof},
    branch::alt,
    error::{Error, ParseError},
    multi::many0,
    Err, Parser,
};

pub trait AocParsable: Sized + Debug {
    fn parse_from_string<'a>(input: &'a [u8]) -> Result<(&'a [u8], Self), Err<Error<&'a [u8]>>>;
}

pub struct AocParser<T: AocParsable> {
    output: PhantomData<T>,
}

impl<'a, T: AocParsable> Parser<&'a [u8], T, Error<&'a [u8]>> for AocParser<T> {
    fn parse(&mut self, input: &'a [u8]) -> nom::IResult<&'a [u8], T, Error<&'a [u8]>> {
        T::parse_from_string(input)
    }
}

pub fn parse_all<'a, T: AocParsable>(input: &'a [u8]) -> Result<Vec<T>, Err<Error<&'a [u8]>>> {
    let parser = AocParser {
        output: PhantomData,
    };
    all_consuming(many0(parser))(input).map(|(_, result)| result)
}

macro_rules! text_parser_for_unsigned_int {
    ($name:ident for $utype:ty) => {
        pub fn $name<'a>() -> impl Parser<&'a [u8], $utype, Error<&'a [u8]>> {
            map_res(take_while1(is_digit), |digit_str: &'a [u8]| {
                <$utype>::from_str_radix(str::from_utf8(digit_str).unwrap(), 10)
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