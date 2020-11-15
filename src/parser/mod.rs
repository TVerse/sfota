use std::fmt;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, newline};
use nom::combinator::{all_consuming, map, recognize};
use nom::error::{context, ContextError, ErrorKind as NomErrorKind, ParseError as NomParseError};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, terminated, tuple};
use nom::Finish;

pub use instruction::mnemonic::Mnemonic;
pub use instruction::operand::{AddressingMode, OperandExpression};
pub use instruction::Instruction;

mod instruction;

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

            // TODO better way to do line numbering
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
    // TODO should not be necessary here? Depends on if we require macros to be defined before use
    UndefinedMnemonic(String),
}

fn take_until_newline(input: &str) -> String {
    input.chars().take_while(|&c| c != '\n').collect()
}

#[derive(Debug, Eq, PartialEq)]
pub struct Parsed(pub Vec<Element>);

impl Parsed {
    fn parse(i: Input) -> IResult<Self> {
        context(
            "File",
            all_consuming(map(
                many0(delimited(many0(newline), Element::parse, many1(newline))),
                Self,
            )),
        )(i)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Element {
    Instruction(instruction::Instruction),
    Label(String),
}

impl Element {
    fn parse(i: Input) -> IResult<Self> {
        context(
            "Element",
            alt((
                map(Label::parse, Element::Label),
                map(instruction::Instruction::parse, Element::Instruction),
            )),
        )(i)
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Label(String);

impl Label {
    fn parse(i: Input) -> IResult<String> {
        dbg!(i);
        context(
            "Label",
            map(terminated(valid_word, tag(":")), |s| s.to_owned()),
        )(i)
    }
}

pub fn parse(i: Input) -> Result<Parsed> {
    Finish::finish(Parsed::parse(i)).map(|(_i, p)| p)
}

fn valid_word(i: Input) -> IResult<&str> {
    dbg!(i);
    context("valid_word", recognize(tuple((valid_start, valid_end))))(i)
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
    fn element_success_1() {
        let input = "  STZ $0300; ";
        let result = Element::parse(input);
        assert_eq!(
            Ok((
                "; ",
                Element::Instruction(Instruction{
                    mnemonic: Mnemonic::STZ,
                    addressing_mode: AddressingMode::AbsoluteOrRelative(OperandExpression::Known(0x300))
                })
            )),
            result
        )
    }

    #[test]
    fn element_success_2() {
        let input = "  RTS ";
        let result = Element::parse(input);
        assert_eq!(
            Ok((" ", Element::Instruction(Instruction{
                mnemonic: Mnemonic::RTS,
                addressing_mode: AddressingMode::NoOperand
            }))),
            result
        )
    }

    #[test]
    fn element_fail() {
        let input = "090";
        let result = Element::parse(input);
        assert!(result.is_err())
    }

    #[test]
    fn parse_success_1() {
        let input = "  STZ $0300\n  RTS\n";
        let result = parse(input);
        assert_eq!(
            Ok(Parsed(vec![
                Element::Instruction(Instruction{
                    mnemonic: Mnemonic::STZ,
                    addressing_mode: AddressingMode::AbsoluteOrRelative(OperandExpression::Known(0x300))
                }),
                Element::Instruction(Instruction{
                    mnemonic: Mnemonic::RTS,
                    addressing_mode: AddressingMode::NoOperand
                }),
            ])),
            result
        )
    }

    #[test]
    fn parse_success_2() {
        let input = "\n\n  STZ $0300\n\n\n  RTS\n\n";
        let result = parse(input);
        assert_eq!(
            Ok(Parsed(vec![
                Element::Instruction(Instruction{
                    mnemonic: Mnemonic::STZ,
                    addressing_mode: AddressingMode::AbsoluteOrRelative(OperandExpression::Known(0x300))
                }),
                Element::Instruction(Instruction{
                    mnemonic: Mnemonic::RTS,
                    addressing_mode: AddressingMode::NoOperand
                }),
            ])),
            result
        )
    }

    #[test]
    fn parse_fail_1() {
        let input = "  STZ $0300\n  RTS";
        let result = parse(input);
        assert!(result.is_err())
    }
}
