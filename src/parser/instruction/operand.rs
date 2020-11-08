use nom::bytes::complete::{tag, take};
use nom::character::complete::{hex_digit1, space1};
use nom::combinator::{map, map_parser, opt};
use nom::error::context;
use nom::sequence::preceded;

use crate::parser::{IResult, Input, valid_word};
use nom::branch::alt;

#[derive(Debug, Eq, PartialEq, strum_macros::Display)]
pub enum Operand {
    Absolute(OperandType<u16>),
    NoOperand,
}

#[derive(Debug, Eq, PartialEq, strum_macros::Display)]
pub enum OperandType<T> {
    Known(T),
    Label(String),
}

impl Operand {
    pub fn parse(i: Input) -> IResult<Self> {
        dbg!(i);
        context(
            "Operand",
            map(
                opt(
                    alt((
                        map(
                    preceded(
                        preceded(space1, tag("$")),
                        map_parser(hex_digit1, take(4usize)),
                    ),
                    // TODO from_str_radix should be safe since we parse for hex digits. Maybe implement custom error?
                    |s| {
                        Self::Absolute(OperandType::Known(
                            u16::from_str_radix(s, 16).expect("Parser returned non-hex bytes?"),
                        ))
                    },
                ),
                        map(
                            preceded(space1, valid_word),
                            |w| Self::Absolute(OperandType::Label(w.to_owned()))
                        )
                ))
        ),
                |maybe_op| match maybe_op {
                    Some(op) => op,
                    None => Operand::NoOperand,
                },
            ),
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_success() {
        let input = " $1234; ";
        let result = Operand::parse(input);
        assert_eq!(
            Ok(("; ", Operand::Absolute(OperandType::Known(0x1234)))),
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
                Operand::Absolute(OperandType::Label("loop".to_owned()))
            )),
            result
        )
    }
}
