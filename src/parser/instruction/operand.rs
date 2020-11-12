use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{hex_digit1, space1};
use nom::combinator::{map, map_parser, map_res, opt, success};
use nom::error::{context, ErrorKind as NomErrorKind, FromExternalError};
use nom::sequence::{delimited, preceded};

use crate::parser::{valid_word, Error, ErrorKind, IResult, Input};

struct OperandTooLong(u16);

impl<'a> FromExternalError<Input<'a>, OperandTooLong> for Error<Input<'a>> {
    fn from_external_error(input: &'a str, kind: NomErrorKind, e: OperandTooLong) -> Self {
        Error {
            errors: vec![
                (input, ErrorKind::Nom(kind)),
                (input, ErrorKind::OperandTooLong(e.0)),
            ],
        }
    }
}

#[derive(Debug, Eq, PartialEq, strum_macros::Display, Clone)]
pub enum Operand {
    Absolute(OperandExpression<u16>),
    AbsoluteIndexedIndirect(OperandExpression<u16>),
    AbsoluteIndexedX(OperandExpression<u16>),
    AbsoluteIndexedY(OperandExpression<u16>),
    AbsoluteIndirect(OperandExpression<u16>),
    Immediate(OperandExpression<u8>),
    ZeroPage(OperandExpression<u8>),
    ZeroPageIndexedIndirect(OperandExpression<u8>),
    ZeroPageIndexedX(OperandExpression<u8>),
    ZeroPageIndexedY(OperandExpression<u8>),
    ZeroPageIndirect(OperandExpression<u8>),
    ZeroPageIndirectIndexedY(OperandExpression<u8>),
    NoOperand,
}

impl Operand {
    pub fn parse(i: Input) -> IResult<Self> {
        dbg!(i);
        context(
            "Operand",
            alt((
                preceded(
                    space1,
                    alt((
                        Self::immediate,
                        Self::indirect,
                        map(Number::parse, |r| match r {
                            Number::N8b(n) => Operand::ZeroPage(OperandExpression::Known(n)),
                            Number::N16b(n) => Operand::Absolute(OperandExpression::Known(n)),
                        }),
                        map(valid_word, |w| {
                            Operand::Absolute(OperandExpression::Label(w.to_owned()))
                        }),
                    )),
                ),
                success(Operand::NoOperand),
            )),
        )(i)
    }

    fn immediate(i: Input) -> IResult<Self> {
        map_res(preceded(tag("#"), Number::parse), |r| match r {
            Number::N8b(n) => Ok(Operand::Immediate(OperandExpression::Known(n))),
            Number::N16b(n) => Err(OperandTooLong(n)),
        })(i)
    }

    fn indirect(i: Input) -> IResult<Self> {
        map(delimited(tag("("), Number::parse, tag(")")), |r| match r {
            Number::N8b(n) => Operand::ZeroPageIndirect(OperandExpression::Known(n)),
            Number::N16b(n) => Operand::AbsoluteIndirect(OperandExpression::Known(n)),
        })(i)
    }
}

enum OperandType {
    Abs,
    IndexedIndirect,
    IndexedX,
    IndexedY,
    Indirect,
    IndirectIndexedY,
    Immediate,
}

enum Number {
    N8b(u8),
    N16b(u16),
}

impl Number {
    // TODO custom exception
    fn parse(i: Input) -> IResult<Self> {
        context("Number", Self::hex_number)(i)
    }

    fn hex_number(i: Input) -> IResult<Self> {
        preceded(tag("$"), alt((Number::parse_16bit, Number::parse_8bit)))(i)
    }

    fn parse_8bit(i: Input) -> IResult<Self> {
        map(
            map_parser(hex_digit1, take(2usize)),
            // TODO from_str_radix should be safe since we parse for hex digits. Maybe implement custom error?
            |s| Number::N8b(u8::from_str_radix(s, 16).expect("Parser returned non-hex bytes?")),
        )(i)
    }

    fn parse_16bit(i: Input) -> IResult<Self> {
        map(
            map_parser(hex_digit1, take(4usize)),
            // TODO from_str_radix should be safe since we parse for hex digits. Maybe implement custom error?
            |s| Number::N16b(u16::from_str_radix(s, 16).expect("Parser returned non-hex bytes?")),
        )(i)
    }
}

#[derive(Debug, Eq, PartialEq, strum_macros::Display, Clone)]
pub enum OperandExpression<T> {
    Known(T),
    Label(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_success_1() {
        let input = " $1234; ";
        let result = Operand::parse(input);
        assert_eq!(
            Ok(("; ", Operand::Absolute(OperandExpression::Known(0x1234)))),
            result
        )
    }

    #[test]
    fn absolute_success_2() {
        let input = " $12; ";
        let result = Operand::parse(input);
        assert_eq!(
            Ok(("; ", Operand::ZeroPage(OperandExpression::Known(0x12)))),
            result
        )
    }

    #[test]
    fn absolute_miss() {
        let input = "90aB; ";
        let result = Operand::parse(input);
        dbg!(&result);
        assert_eq!(Ok((input, Operand::NoOperand)), result); // TODO is this good?

        let input = "$90a; ";
        let result = Operand::parse(input);
        dbg!(&result);
        assert_eq!(Ok((input, Operand::NoOperand)), result); // TODO is this good?
    }

    #[test]
    fn label_success() {
        let input = " loop; ";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "; ",
                Operand::Absolute(OperandExpression::Label("loop".to_owned()))
            )),
            result
        )
    }

    #[test]
    fn absolute_indirect_indexed_success() {
        let input = " ($1234, X); ";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "; ",
                Operand::AbsoluteIndexedIndirect(OperandExpression::Known(0x1234))
            )),
            result
        )
    }

    #[test]
    fn absolute_indexed_x_success() {
        let input = " $1234, X";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "",
                Operand::AbsoluteIndexedX(OperandExpression::Known(0x1234))
            )),
            result
        )
    }

    #[test]
    fn absolute_indexed_y_success() {
        let input = " $1234, Y";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "",
                Operand::AbsoluteIndexedY(OperandExpression::Known(0x1234))
            )),
            result
        )
    }

    #[test]
    fn absolute_indirect_success() {
        let input = " (indirect)\n";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "\n",
                Operand::AbsoluteIndirect(OperandExpression::Label("indirect".to_owned()))
            )),
            result
        )
    }

    #[test]
    fn immediate_success() {
        let input = " #$12\n";
        let result = Operand::parse(input);
        assert_eq!(
            Ok(("\n", Operand::Immediate(OperandExpression::Known(0x12)))),
            result
        )
    }

    #[test]
    fn zero_page_success() {
        let input = " $12; ";
        let result = Operand::parse(input);
        assert_eq!(
            Ok(("; ", Operand::ZeroPage(OperandExpression::Known(0x12)))),
            result
        )
    }

    #[test]
    fn zero_page_indirect_indexed_success() {
        let input = " ($12, X); ";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "; ",
                Operand::ZeroPageIndexedIndirect(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn zero_page_indexed_x_success() {
        let input = " $12, X";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "",
                Operand::ZeroPageIndexedX(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn zero_page_indexed_y_success() {
        let input = " $12, Y";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "",
                Operand::ZeroPageIndexedY(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn zero_page_indirect_success() {
        let input = " ($12)\n";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "\n",
                Operand::ZeroPageIndirect(OperandExpression::Known(0x12))
            )),
            result
        )
    }
}
