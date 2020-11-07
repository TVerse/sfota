use std::fmt;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{alpha1, alphanumeric1, hex_digit1, newline, space1};
use nom::combinator::{all_consuming, map, map_parser, map_res, opt, recognize};
use nom::error::{
    context, ContextError, ErrorKind as NomErrorKind, FromExternalError,
    ParseError as NomParseError,
};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, tuple};
use nom::Finish;

pub type Input<'a> = &'a str;
pub type Result<'a, T> = std::result::Result<T, Error<Input<'a>>>;

type IResult<'a, T> = nom::IResult<Input<'a>, T, Error<Input<'a>>>;

#[derive(Eq, PartialEq)]
pub struct Error<I> {
    pub errors: Vec<(I, ErrorKind)>,
}

impl<'a> Error<Input<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Parse error:")?;
        for (input, kind) in self.errors.iter().rev() {
            let prefix = match kind {
                ErrorKind::Nom(err) => format!("nom error {:?}", err),
                ErrorKind::Context(ctx) => format!("in {}", ctx),
                ErrorKind::UndefinedMnemonic(m) => format!("undefined mnemonic \"{}\"", m),
            };

            let input: String = take_until_newline(input);
            writeln!(f, "{:<40} \"{}\"", prefix, input)?;
        }

        Ok(())
    }
}

impl<'a> std::error::Error for Error<Input<'a>> {}

impl<'a> fmt::Display for Error<Input<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Error::fmt(self, f)
    }
}

impl<'a> fmt::Debug for Error<Input<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Error::fmt(self, f)
    }
}

impl<I> NomParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        let errors = vec![(input, ErrorKind::Nom(kind))];
        Self { errors }
    }

    fn append(input: I, kind: NomErrorKind, mut other: Self) -> Self {
        other.errors.push((input, ErrorKind::Nom(kind)));
        other
    }

    fn or(mut self, other: Self) -> Self {
        self.errors.extend(other.errors);
        self
    }
}

impl<I> ContextError<I> for Error<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, ErrorKind::Context(ctx)));
        other
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Nom(NomErrorKind),
    Context(&'static str),
    UndefinedMnemonic(String), // TODO should not be necessary here? Depends on if we require macros to be defined before use
}

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

fn take_until_newline(input: &str) -> String {
    input.chars().take_while(|&c| c != '\n').collect()
}

#[derive(Debug, Eq, PartialEq)]
pub struct Parsed(pub Vec<Line>);

impl Parsed {
    fn parse(i: Input) -> IResult<Self> {
        context("File", all_consuming(map(many0(Line::parse), Self)))(i)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Line(pub Vec<Element>);

impl Line {
    fn parse(i: Input) -> IResult<Self> {
        context(
            "line",
            map(delimited(space1, Element::instruction, newline), |e| {
                Self(vec![e])
            }),
        )(i)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Element {
    Instruction(Mnemonic, Operand),
}

impl Element {
    fn instruction(i: Input) -> IResult<Self> {
        context(
            "Instruction",
            map(
                tuple((Mnemonic::parse, opt(preceded(space1, Operand::parse)))),
                |(mnemonic, o)| match o {
                    Some(operand) => Self::Instruction(mnemonic, operand),
                    None => Self::Instruction(mnemonic, Operand::Implied),
                },
            ),
        )(i)
    }
}

#[derive(Debug, Eq, PartialEq, strum_macros::EnumString)]
pub enum Mnemonic {
    STZ,
    RTS,
}

impl Mnemonic {
    fn parse(i: Input) -> IResult<Self> {
        context(
            "Mnemonic",
            map_res(valid_word, |m| {
                Self::from_str(m).map_err(|_| UndefinedMnemonic(m.to_owned()))
            }),
        )(i)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Operand {
    AbsoluteFour(OperandDefinition),
    Implied,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OperandDefinition {
    Direct(u16),
}

impl Operand {
    fn parse(i: Input) -> IResult<Self> {
        context(
            "Operand",
            // TODO from_str_radix should be safe since we parse for hex digits. Maybe implement custom error?
            map(
                map(
                    preceded(tag("#$"), map_parser(hex_digit1, take(4usize))),
                    |s| u16::from_str_radix(s, 16).expect("Parser returned non-hex bytes?"),
                ),
                |addr| Self::AbsoluteFour(OperandDefinition::Direct(addr)),
            ),
        )(i)
    }
}

pub fn parse(i: Input) -> Result<Parsed> {
    Finish::finish(Parsed::parse(i)).map(|(_i, p)| p)
}

fn valid_start(i: Input) -> IResult<&str> {
    dbg!(i);
    context("valid_start", alpha1)(i)
}

fn valid_end(i: Input) -> IResult<&str> {
    dbg!(i);
    context(
        "valid_end",
        recognize(many0(alt((alphanumeric1, tag("_"))))),
    )(i)
}

fn valid_word(i: Input) -> IResult<&str> {
    dbg!(i);
    context("valid_word", recognize(tuple((valid_start, valid_end))))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_start_success() {
        let input = "abcD_0e x";
        let result = valid_start(input);
        assert_eq!(Ok(("_0e x", "abcD")), result)
    }

    #[test]
    fn valid_start_fail() {
        let input = "0abcde";
        let result = valid_start(input);
        assert!(result.is_err())
    }

    #[test]
    fn valid_end_success() {
        let input = "_abcD_0e x";
        let result = valid_end(input);
        assert_eq!(Ok((" x", "_abcD_0e")), result)
    }

    #[test]
    fn valid_end_fail() {
        let input = " _abcD_0e";
        let result = valid_end(input);
        assert_eq!(Ok((input, "")), result)
    }

    #[test]
    fn valid_word_success() {
        let input = "STA ";
        let result = valid_word(input);
        assert_eq!(Ok((" ", "STA")), result)
    }

    #[test]
    fn mnemonic_success() {
        let input = "RTS\n";
        let result = Mnemonic::parse(input);
        assert_eq!(Ok(("\n", Mnemonic::RTS)), result);
    }

    #[test]
    fn mnemonic_fail_1() {
        let input = " not This_is_a_val1d_mnemOnIc end";
        let result = Mnemonic::parse(input);
        assert!(result.is_err())
    }

    #[test]
    fn mnemonic_fail_2() {
        let input = "STA #$0300\n";
        let result = Mnemonic::parse(input);
        assert!(result.is_err())
    }

    #[test]
    fn operand_success() {
        let input = "#$1234; ";
        let result = Operand::parse(input);
        assert_eq!(
            Ok((
                "; ",
                Operand::AbsoluteFour(OperandDefinition::Direct(0x1234))
            )),
            result
        )
    }

    #[test]
    fn operand_failure() {
        let input = "90aB; ";
        let result = Operand::parse(input);
        assert!(result.is_err());

        let input = "#$90a; ";
        let result = Operand::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn instruction_success_1() {
        let input = "STZ #$0300; ";
        let result = Element::instruction(input);
        assert_eq!(
            Ok((
                "; ",
                Element::Instruction(
                    Mnemonic::STZ,
                    Operand::AbsoluteFour(OperandDefinition::Direct(0x0300))
                )
            )),
            result
        )
    }

    #[test]
    fn instruction_success_2() {
        let input = "RTS\n";
        let result = Element::instruction(input);
        assert_eq!(
            Ok(("\n", Element::Instruction(Mnemonic::RTS, Operand::Implied))),
            result
        )
    }

    #[test]
    fn instruction_fail() {
        let input = "090";
        let result = Element::instruction(input);
        assert!(result.is_err())
    }

    #[test]
    fn line_success() {
        let input = "  STZ #$0300\n ";
        let result = Line::parse(input);
        assert_eq!(
            Ok((
                " ",
                Line(vec![Element::Instruction(
                    Mnemonic::STZ,
                    Operand::AbsoluteFour(OperandDefinition::Direct(0x0300)),
                )])
            )),
            result
        )
    }

    #[test]
    fn line_fail() {
        let input = "STZ #$0300\n ";
        let result = Line::parse(input);
        assert!(result.is_err())
    }

    #[test]
    fn parse_success() {
        let input = "  STZ #$0300\n  RTS\n";
        let result = parse(input);
        assert_eq!(
            Ok(Parsed(vec![
                Line(vec![Element::Instruction(
                    Mnemonic::STZ,
                    Operand::AbsoluteFour(OperandDefinition::Direct(0x0300)),
                )]),
                Line(vec![Element::Instruction(Mnemonic::RTS, Operand::Implied)]),
            ])),
            result
        )
    }

    #[test]
    fn parse_fail_1() {
        let input = "  STZ #$0300\n  RTS\n ";
        let result = parse(input);
        assert!(result.is_err())
    }

    #[test]
    fn parse_fail_2() {
        let input = "  STZ #$0300\n  RTS";
        let result = parse(input);
        assert!(result.is_err())
    }
}
