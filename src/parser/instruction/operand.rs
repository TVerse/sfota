use nom::bytes::complete::{tag, take};
use nom::character::complete::{hex_digit1, space1};
use nom::combinator::{map, map_parser, opt};
use nom::error::context;
use nom::sequence::preceded;

use crate::parser::{IResult, Input};

#[derive(Debug, Eq, PartialEq, strum_macros::Display)]
pub enum Operand {
    Absolute(u16),
    NoOperand,
}

impl Operand {
    pub fn parse(i: Input) -> IResult<Self> {
        dbg!(i);
        context(
            "Operand",
            map(
                opt(
                    map(
                        map(
                            preceded(
                                preceded(space1, tag("#$")),
                                map_parser(hex_digit1, take(4usize)),
                            ),
                            // TODO from_str_radix should be safe since we parse for hex digits. Maybe implement custom error?
                            |s| u16::from_str_radix(s, 16).expect("Parser returned non-hex bytes?"),
                        ),
                        Self::Absolute,
                    ),
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
    fn operand_success() {
        let input = " #$1234; ";
        let result = Operand::parse(input);
        assert_eq!(Ok(("; ", Operand::Absolute(0x1234))), result)
    }

    #[test]
    fn operand_miss() {
        let input = "90aB; ";
        let result = Operand::parse(input);
        dbg!(&result);
        assert_eq!(Ok((input, Operand::NoOperand)), result); // TODO is this good?

        let input = "#$90a; ";
        let result = Operand::parse(input);
        dbg!(&result);
        assert_eq!(Ok((input, Operand::NoOperand)), result); // TODO is this good?
    }
}