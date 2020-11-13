use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{hex_digit1, space0, space1};
use nom::combinator::{map, map_parser, map_res, success};
use nom::error::{context, ErrorKind as NomErrorKind, FromExternalError};
use nom::sequence::{delimited, preceded, terminated, tuple};

use crate::parser::{valid_word, Error, ErrorKind, IResult, Input};
use either::Either;

struct OperandTooLong(OperandExpression<u16>);

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
pub enum AddressingMode {
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

impl AddressingMode {
    pub fn parse(i: Input) -> IResult<Self> {
        dbg!(i);
        context(
            "AddressingMode",
            alt((
                preceded(
                    space1,
                    alt((
                        Self::immediate,
                        Self::indexed_x,
                        Self::indexed_y,
                        Self::indirect_indexed_y,
                        Self::indexed_indirect,
                        Self::indirect,
                        Self::absolute,
                    )),
                ),
                success(AddressingMode::NoOperand),
            )),
        )(i)
    }

    fn immediate(i: Input) -> IResult<Self> {
        map_res(preceded(tag("#"), parse_operand_expression), |r| match r {
            Either::Left(oe) => Ok(AddressingMode::Immediate(oe)),
            Either::Right(oe) => Err(OperandTooLong(oe)),
        })(i)
    }

    fn indirect(i: Input) -> IResult<Self> {
        map(
            delimited(tag("("), parse_operand_expression, tag(")")),
            |r| match r {
                Either::Left(oe) => AddressingMode::ZeroPageIndirect(oe),
                Either::Right(oe) => AddressingMode::AbsoluteIndirect(oe),
            },
        )(i)
    }

    fn indexed_indirect(i: Input) -> IResult<Self> {
        map(
            delimited(
                tag("("),
                parse_operand_expression,
                tuple((tag(","), space0, tag("X"), tag(")"))),
            ),
            |r| match r {
                Either::Left(oe) => AddressingMode::ZeroPageIndexedIndirect(oe),
                Either::Right(oe) => AddressingMode::AbsoluteIndexedIndirect(oe),
            },
        )(i)
    }

    fn absolute(i: Input) -> IResult<Self> {
        map(parse_operand_expression, |r| match r {
            Either::Left(oe) => AddressingMode::ZeroPage(oe),
            Either::Right(oe) => AddressingMode::Absolute(oe),
        })(i)
    }

    fn indexed_x(i: Input) -> IResult<Self> {
        map(
            terminated(
                parse_operand_expression,
                tuple((tag(","), space0, tag("Y"))),
            ),
            |r| match r {
                Either::Left(oe) => AddressingMode::ZeroPageIndexedY(oe),
                Either::Right(oe) => AddressingMode::AbsoluteIndexedY(oe),
            },
        )(i)
    }

    fn indexed_y(i: Input) -> IResult<Self> {
        map(
            terminated(
                parse_operand_expression,
                tuple((tag(","), space0, tag("X"))),
            ),
            |r| match r {
                Either::Left(oe) => AddressingMode::ZeroPageIndexedX(oe),
                Either::Right(oe) => AddressingMode::AbsoluteIndexedX(oe),
            },
        )(i)
    }

    fn indirect_indexed_y(i: Input) -> IResult<Self> {
        map_res(
            delimited(
                tag("("),
                parse_operand_expression,
                tuple((tag(")"), tag(","), space0, tag("Y"))),
            ),
            |r| match r {
                Either::Left(oe) => Ok(AddressingMode::ZeroPageIndirectIndexedY(oe)),
                Either::Right(oe) => Err(OperandTooLong(oe)),
            },
        )(i)
    }
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

// TODO any way to get this inside the impl block?
fn parse_operand_expression(
    i: Input,
) -> IResult<Either<OperandExpression<u8>, OperandExpression<u16>>> {
    context(
        "OperandExpression",
        alt((
            map(valid_word, |l| {
                Either::Right(OperandExpression::Label(l.to_owned()))
            }),
            map(Number::parse, |r| match r {
                Number::N8b(n) => Either::Left(OperandExpression::Known(n)),
                Number::N16b(n) => Either::Right(OperandExpression::Known(n)),
            }),
        )),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_success_1() {
        let input = " $1234; ";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "; ",
                AddressingMode::Absolute(OperandExpression::Known(0x1234))
            )),
            result
        )
    }

    #[test]
    fn absolute_success_2() {
        let input = " $12; ";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "; ",
                AddressingMode::ZeroPage(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn absolute_miss() {
        let input = "90aB; ";
        let result = AddressingMode::parse(input);
        dbg!(&result);
        assert_eq!(Ok((input, AddressingMode::NoOperand)), result); // TODO is this good?

        let input = "$90a; ";
        let result = AddressingMode::parse(input);
        dbg!(&result);
        assert_eq!(Ok((input, AddressingMode::NoOperand)), result); // TODO is this good?
    }

    #[test]
    fn label_success() {
        let input = " loop; ";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "; ",
                AddressingMode::Absolute(OperandExpression::Label("loop".to_owned()))
            )),
            result
        )
    }

    #[test]
    fn absolute_indirect_indexed_success() {
        let input = " ($1234, X); ";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "; ",
                AddressingMode::AbsoluteIndexedIndirect(OperandExpression::Known(0x1234))
            )),
            result
        )
    }

    #[test]
    fn absolute_indexed_x_success() {
        let input = " $1234, X";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "",
                AddressingMode::AbsoluteIndexedX(OperandExpression::Known(0x1234))
            )),
            result
        )
    }

    #[test]
    fn absolute_indexed_y_success() {
        let input = " $1234, Y";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "",
                AddressingMode::AbsoluteIndexedY(OperandExpression::Known(0x1234))
            )),
            result
        )
    }

    #[test]
    fn absolute_indirect_success() {
        let input = " (indirect)\n";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "\n",
                AddressingMode::AbsoluteIndirect(OperandExpression::Label("indirect".to_owned()))
            )),
            result
        )
    }

    #[test]
    fn immediate_success() {
        let input = " #$12\n";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "\n",
                AddressingMode::Immediate(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn zero_page_success() {
        let input = " $12; ";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "; ",
                AddressingMode::ZeroPage(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn zero_page_indirect_indexed_success() {
        let input = " ($12, X); ";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "; ",
                AddressingMode::ZeroPageIndexedIndirect(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn zero_page_indexed_x_success() {
        let input = " $12, X";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "",
                AddressingMode::ZeroPageIndexedX(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn zero_page_indexed_y_success() {
        let input = " $12,Y";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "",
                AddressingMode::ZeroPageIndexedY(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn zero_page_indirect_success() {
        let input = " ($12)\n";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "\n",
                AddressingMode::ZeroPageIndirect(OperandExpression::Known(0x12))
            )),
            result
        )
    }

    #[test]
    fn zero_page_indirect_indexed_y_success() {
        let input = " ($12), Y\n";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "\n",
                AddressingMode::ZeroPageIndirectIndexedY(OperandExpression::Known(0x12))
            )),
            result
        )
    }
}
