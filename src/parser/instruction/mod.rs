use nom::character::complete::space1;
use nom::combinator::map;
use nom::error::context;
use nom::sequence::{preceded, tuple};

use mnemonic::Mnemonic;
use operand::AddressingMode;

use super::{IResult, Input};

pub mod mnemonic;
pub mod operand;

#[derive(Debug, Eq, PartialEq)]
pub struct Instruction {
    pub mnemonic: Mnemonic,
    pub addressing_mode: AddressingMode,
}

impl Instruction {
    pub fn parse(i: Input) -> IResult<Self> {
        context(
            "Instruction",
            map(
                preceded(space1, tuple((Mnemonic::parse, AddressingMode::parse))),
                |(mnemonic, addressing_mode)| Instruction {
                    mnemonic,
                    addressing_mode,
                },
            ),
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::operand_expression::OperandExpression;

    #[test]
    fn instruction_success_1() {
        let input = "  STZ $0300; ";
        let result = Instruction::parse(input);
        assert_eq!(
            Ok((
                "; ",
                Instruction {
                    mnemonic: Mnemonic::STZ,
                    addressing_mode: AddressingMode::AbsoluteOrRelative(OperandExpression::Known(
                        0x300
                    ))
                }
            )),
            result
        )
    }

    #[test]
    fn instruction_success_2() {
        let input = "  RTS ";
        let result = Instruction::parse(input);
        assert_eq!(
            Ok((
                " ",
                Instruction {
                    mnemonic: Mnemonic::RTS,
                    addressing_mode: AddressingMode::NoOperand
                }
            )),
            result
        )
    }

    #[test]
    fn instruction_success_3() {
        let input = "  JMP loop ";
        let result = Instruction::parse(input);
        assert_eq!(
            Ok((
                " ",
                Instruction {
                    mnemonic: Mnemonic::JMP,
                    addressing_mode: AddressingMode::AbsoluteOrRelative(OperandExpression::Label(
                        "loop".to_owned()
                    ))
                }
            )),
            result
        )
    }

    #[test]
    fn instruction_fail() {
        let input = "090";
        let result = Instruction::parse(input);
        assert!(result.is_err())
    }
}
