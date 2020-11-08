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

#[derive(Debug, Eq, PartialEq, strum_macros::EnumString, strum_macros::Display)]
pub enum Mnemonic {
    STZ,
    RTS,
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
        let input = "STA #$0300";
        let result = Mnemonic::parse(input);
        assert!(result.is_err())
    }
}