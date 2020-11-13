use nom::character::complete::space1;
use nom::combinator::map_res;
use nom::error::{context, ErrorKind as NomErrorKind, FromExternalError};
use nom::sequence::{preceded, tuple};

use mnemonic::Mnemonic;
use operand::AddressingMode;
use operand::OperandExpression;

use super::{Error, ErrorKind, IResult, Input};

pub mod mnemonic;
pub mod operand;

struct InvalidAddressingMode(Mnemonic, AddressingMode);

impl<'a> FromExternalError<Input<'a>, InvalidAddressingMode> for Error<Input<'a>> {
    fn from_external_error(input: Input<'a>, kind: NomErrorKind, e: InvalidAddressingMode) -> Self {
        Error {
            errors: vec![
                (input, ErrorKind::Nom(kind)),
                (input, ErrorKind::InvalidAddressingMode(e.0, e.1)),
            ],
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
    StzAbsolute(OperandExpression<u16>),
    RtsStack,
    JmpAbsolute(OperandExpression<u16>),
}

impl Instruction {
    pub fn parse(i: Input) -> IResult<Self> {
        use Instruction::*;
        context(
            "Instruction",
            map_res(
                preceded(space1, tuple((Mnemonic::parse, AddressingMode::parse))),
                |(mnemonic, operand)| match (mnemonic, operand) {
                    (Mnemonic::STZ, AddressingMode::Absolute(a)) => Ok(StzAbsolute(a)),
                    (Mnemonic::RTS, AddressingMode::NoOperand) => Ok(RtsStack),
                    (Mnemonic::JMP, AddressingMode::Absolute(a)) => Ok(JmpAbsolute(a)),
                    (mnemonic, operand) => Err(InvalidAddressingMode(mnemonic, operand)),
                },
            ),
        )(i)
    }

    pub fn instruction_byte(&self) -> u8 {
        match self {
            Instruction::StzAbsolute(_) => 0x9C,
            Instruction::RtsStack => 0x60,
            Instruction::JmpAbsolute(_) => 0x4C,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_success_1() {
        let input = "  STZ $0300; ";
        let result = Instruction::parse(input);
        assert_eq!(
            Ok((
                "; ",
                Instruction::StzAbsolute(OperandExpression::Known(0x0300))
            )),
            result
        )
    }

    #[test]
    fn instruction_success_2() {
        let input = "  RTS ";
        let result = Instruction::parse(input);
        assert_eq!(Ok((" ", Instruction::RtsStack)), result)
    }

    #[test]
    fn instruction_success_3() {
        let input = "  JMP loop ";
        let result = Instruction::parse(input);
        assert_eq!(
            Ok((
                " ",
                Instruction::JmpAbsolute(OperandExpression::Label("loop".to_owned()))
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
