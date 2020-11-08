use nom::character::complete::space1;
use nom::combinator::map_res;
use nom::error::{context, ErrorKind as NomErrorKind, FromExternalError};
use nom::sequence::{preceded, tuple};

use mnemonic::Mnemonic;
use operand::Operand;
use operand::OperandType;

use super::{Error, ErrorKind, IResult, Input};

pub mod mnemonic;
pub mod operand;

struct InvalidAddressingMode(Mnemonic, Operand);

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
    StzAbsolute(OperandType<u16>),
    RtsStack,
    JmpAbsolute(OperandType<u16>),
}

impl Instruction {
    pub fn parse(i: Input) -> IResult<Self> {
        use Instruction::*;
        context(
            "Instruction",
            map_res(
                preceded(space1, tuple((Mnemonic::parse, Operand::parse))),
                |(mnemonic, operand)| match (mnemonic, operand) {
                    (Mnemonic::STZ, Operand::Absolute(a)) => Ok(StzAbsolute(a)),
                    (Mnemonic::RTS, Operand::NoOperand) => Ok(RtsStack),
                    (Mnemonic::JMP, Operand::Absolute(a)) => Ok(JmpAbsolute(a)),
                    (mnemonic, operand) => Err(InvalidAddressingMode(mnemonic, operand)),
                },
            ),
        )(i)
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
            Ok(("; ", Instruction::StzAbsolute(OperandType::Known(0x0300)))),
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
        assert_eq!(Ok((" ", Instruction::JmpAbsolute(OperandType::Label("loop".to_owned())))), result)
    }

    #[test]
    fn instruction_fail() {
        let input = "090";
        let result = Instruction::parse(input);
        assert!(result.is_err())
    }
}
