use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{space0, space1};
use nom::combinator::{map, success};
use nom::error::context;
use nom::sequence::{delimited, preceded, terminated, tuple};

use crate::parser::operand_expression::OperandExpression;
use crate::parser::{IResult, Input};

#[derive(Debug, Eq, PartialEq, strum_macros::Display, Clone)]
pub enum AddressingMode {
    Immediate(OperandExpression),
    IndexedX(OperandExpression),
    IndexedY(OperandExpression),
    IndirectIndexedY(OperandExpression),
    IndexedIndirectX(OperandExpression),
    Indirect(OperandExpression),
    AbsoluteOrRelative(OperandExpression),
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
                        Self::absolute_or_relative,
                    )),
                ),
                success(AddressingMode::NoOperand),
            )),
        )(i)
    }

    fn immediate(i: Input) -> IResult<Self> {
        map(
            preceded(tag("#"), OperandExpression::parse),
            AddressingMode::Immediate,
        )(i)
    }

    fn indirect(i: Input) -> IResult<Self> {
        map(
            delimited(
                tag("("),
                OperandExpression::parse,
                tag(")"),
            ),
            AddressingMode::Indirect,
        )(i)
    }

    fn indexed_indirect(i: Input) -> IResult<Self> {
        map(
            delimited(
                tag("("),
                OperandExpression::parse,
                tuple((tag(","), space0, tag("X"), tag(")"))),
            ),
            AddressingMode::IndexedIndirectX,
        )(i)
    }

    fn absolute_or_relative(i: Input) -> IResult<Self> {
        map(
            OperandExpression::parse,
            AddressingMode::AbsoluteOrRelative,
        )(i)
    }

    fn indexed_x(i: Input) -> IResult<Self> {
        map(
            terminated(
                OperandExpression::parse,
                tuple((tag(","), space0, tag("Y"))),
            ),
            AddressingMode::IndexedY,
        )(i)
    }

    fn indexed_y(i: Input) -> IResult<Self> {
        map(
            terminated(
                OperandExpression::parse,
                tuple((tag(","), space0, tag("X"))),
            ),
            AddressingMode::IndexedX,
        )(i)
    }

    fn indirect_indexed_y(i: Input) -> IResult<Self> {
        map(
            delimited(
                tag("("),
                OperandExpression::parse,
                tuple((tag(")"), tag(","), space0, tag("Y"))),
            ),
            AddressingMode::IndirectIndexedY,
        )(i)
    }
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
                AddressingMode::AbsoluteOrRelative(OperandExpression::Known(0x1234))
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
                AddressingMode::AbsoluteOrRelative(OperandExpression::Known(0x12))
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
                AddressingMode::AbsoluteOrRelative(OperandExpression::Label("loop".to_owned()))
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
                AddressingMode::IndexedIndirectX(OperandExpression::Known(0x1234))
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
                AddressingMode::IndexedX(OperandExpression::Known(0x1234))
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
                AddressingMode::IndexedY(OperandExpression::Known(0x1234))
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
                AddressingMode::Indirect(OperandExpression::Label("indirect".to_owned()))
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
    fn indirect_indexed_y_success() {
        let input = " ($12), Y\n";
        let result = AddressingMode::parse(input);
        assert_eq!(
            Ok((
                "\n",
                AddressingMode::IndirectIndexedY(OperandExpression::Known(0x12))
            )),
            result
        )
    }
}
