use std::str::FromStr;

use nom::combinator::map_res;
use nom::error::{context, ErrorKind as NomErrorKind, FromExternalError};

use crate::parser::{valid_word, Error, ErrorKind, IResult, Input};

struct UndefinedMnemonic(String);

impl<'a> FromExternalError<Input<'a>, UndefinedMnemonic> for Error<Input<'a>> {
    fn from_external_error(input: Input<'a>, kind: NomErrorKind, e: UndefinedMnemonic) -> Self {
        Error {
            errors: vec![
                (input, ErrorKind::Nom(kind)),
                (input, ErrorKind::UndefinedMnemonic(e.0)),
            ],
        }
    }
}

// TODO: BBR/BBS has special addressing mode! "BBR0 zp,rel" or "BBR 0,zp,rel"?
// TODO: RMB/SMB could maybe use special addressing mode?
#[derive(Debug, Eq, PartialEq, strum_macros::EnumString, strum_macros::Display)]
pub enum Mnemonic {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRA,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PHX,
    PHY,
    PLA,
    PLP,
    PLX,
    PLY,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STP,
    STX,
    STY,
    STZ,
    TAX,
    TAY,
    TRB,
    TSB,
    TSX,
    TXA,
    TYA,
    WAI,
}

impl Mnemonic {
    pub fn parse(i: Input) -> IResult<Self> {
        context(
            "Mnemonic",
            map_res(valid_word, |m| {
                Self::from_str(m).map_err(|_| UndefinedMnemonic(m.to_owned()))
            }),
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mnemonic_success() {
        let input = "RTS";
        let result = Mnemonic::parse(input);
        assert_eq!(Ok(("", Mnemonic::RTS)), result);
    }

    #[test]
    fn mnemonic_fail_1() {
        let input = " not This_is_a_val1d_mnemOnIc end";
        let result = Mnemonic::parse(input);
        assert!(result.is_err())
    }

    #[test]
    fn mnemonic_fail_2() {
        let input = "SAX #$0300";
        let result = Mnemonic::parse(input);
        assert!(result.is_err())
    }
}
