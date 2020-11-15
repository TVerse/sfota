use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::hex_digit1;
use nom::combinator::{map, map_parser};
use nom::error::context;
use nom::sequence::{preceded};

use crate::parser::{valid_word, IResult, Input};

#[derive(Debug, Eq, PartialEq, strum_macros::Display, Clone)]
// TODO u8 and i8 (for relative)
pub enum OperandExpression {
    Known(u16),
    Label(String),
}

impl OperandExpression {
    // TODO any way to get this inside the impl block?
    pub fn parse(i: Input) -> IResult<Self> {
        context(
            "OperandExpression",
            alt((
                map(valid_word, |l| OperandExpression::Label(l.to_owned())),
                map(Number::parse, |r| match r {
                    Number::Nu8(n) => Self::Known(n as u16), // TODO
                    Number::Ni8(n) => Self::Known(n as u16), // TODO
                    Number::Nu16(n) => Self::Known(n),
                }),
            )),
        )(i)
    }
}

enum Number {
    Nu8(u8),
    Ni8(i8),
    Nu16(u16),
}

impl Number {
    // TODO custom exception?
    fn parse(i: Input) -> IResult<Self> {
        context("Number", Self::hex_number)(i)
    }

    fn hex_number(i: Input) -> IResult<Self> {
        preceded(tag("$"), alt((Number::parse_u16, Number::parse_u8)))(i)
    }

    fn parse_u8(i: Input) -> IResult<Self> {
        map(
            map_parser(hex_digit1, take(2usize)),
            // TODO from_str_radix should be safe since we parse for hex digits. Maybe implement custom error?
            |s| Number::Nu8(u8::from_str_radix(s, 16).expect("Parser returned non-hex bytes?")),
        )(i)
    }

    fn parse_u16(i: Input) -> IResult<Self> {
        map(
            map_parser(hex_digit1, take(4usize)),
            // TODO from_str_radix should be safe since we parse for hex digits. Maybe implement custom error?
            |s| Number::Nu16(u16::from_str_radix(s, 16).expect("Parser returned non-hex bytes?")),
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        let input = "$FF";
        let result = OperandExpression::parse(input);
        todo!()
    }
}