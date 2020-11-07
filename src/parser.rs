use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{alpha1, alphanumeric1, hex_digit1, newline, space1};
use nom::combinator::{all_consuming, map, map_parser, opt, recognize};
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub struct Parsed(pub Vec<Line>);

#[derive(Debug, Eq, PartialEq)]
pub struct Line(pub Vec<Element>);

impl Line {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Line, E> {
        map(delimited(space1, Element::instruction, newline), |e| Line(vec![e]))(i)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Element {
    Instruction(Mnemonic, Operand),
}

impl Element {
    fn instruction<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Element, E> {
        map(
            tuple((Mnemonic::parse, opt(preceded(space1, Operand::parse)))),
            |(mnemonic, o)| match o {
                Some(operand) => Element::Instruction(mnemonic, operand),
                None => Element::Instruction(mnemonic, Operand::Implied),
            },
        )(i)
    }
}

#[derive(Debug, Eq, PartialEq, strum_macros::EnumString)]
pub enum Mnemonic {
    STZ,
    RTS,
    #[strum(default)]
    UserDefined(String),
}

impl Mnemonic {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Mnemonic, E> {
        // TODO unwrap is safe as long as the default is set. Maybe implement custom error?
        map(valid_word, |s| Mnemonic::from_str(s).unwrap())(i)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Operand {
    AbsoluteFour(u16),
    Implied,
}

impl Operand {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Operand, E> {
        // TODO from_str_radix should be safe since we parse for hex digits. Maybe implement custom error?
        map(
            map(
                preceded(tag("#$"), map_parser(hex_digit1, take(4usize))),
                |s| u16::from_str_radix(s, 16).expect("Parser returned non-hex bytes?"),
            ),
            Operand::AbsoluteFour,
        )(i)
    }
}

pub fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Parsed, E> {
    map(all_consuming(many0(Line::parse)), Parsed)(i)
}

fn valid_start<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    alpha1(i)
}

fn valid_end<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize(many0(alt((alphanumeric1, tag("_")))))(i)
}

fn valid_word<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize(tuple((valid_start, valid_end)))(i)
}

#[cfg(test)]
mod tests {
    use nom::error;
    use nom::error::ErrorKind;

    use super::*;

    type E<'a> = error::Error<&'a str>;

    fn err<O>(input: &str, code: ErrorKind) -> IResult<&str, O, E> {
        Err(nom::Err::Error(error::Error { input, code }))
    }

    #[test]
    fn valid_start_success() {
        let input = "abcD_0e x";
        let result = valid_start::<E>(input);
        assert_eq!(Ok(("_0e x", "abcD")), result)
    }

    #[test]
    fn valid_start_fail() {
        let input = "0abcde";
        let result = valid_start::<E>(input);
        assert_eq!(err(input, ErrorKind::Alpha), result)
    }

    #[test]
    fn valid_end_success() {
        let input = "_abcD_0e x";
        let result = valid_end::<E>(input);
        assert_eq!(Ok((" x", "_abcD_0e")), result)
    }

    #[test]
    fn valid_end_fail() {
        let input = " _abcD_0e";
        let result = valid_end::<E>(input);
        assert_eq!(Ok((input, "")), result)
    }

    #[test]
    fn mnemonic_success() {
        let input = "RTS\n";
        let result = Mnemonic::parse::<E>(input);
        assert_eq!(Ok(("\n", Mnemonic::RTS)), result);
        let input = "This_is_a_val1d_mnemOnIc end";
        let result = Mnemonic::parse::<E>(input);
        assert_eq!(
            Ok((
                " end",
                Mnemonic::UserDefined("This_is_a_val1d_mnemOnIc".to_owned())
            )),
            result
        );
        let input = "STA";
        let result = Mnemonic::parse::<E>(input);
        assert_eq!(Ok(("", Mnemonic::UserDefined("STA".to_owned()))), result)
    }

    #[test]
    fn mnemonic_fail() {
        let input = " not This_is_a_val1d_mnemOnIc end";
        let result = Mnemonic::parse::<E>(input);
        assert_eq!(err(input, ErrorKind::Alpha), result)
    }

    #[test]
    fn operand_success() {
        let input = "#$1234; ";
        let result = Operand::parse::<E>(input);
        assert_eq!(Ok(("; ", Operand::AbsoluteFour(0x1234))), result)
    }

    #[test]
    fn operand_failure() {
        let input = "90aB; ";
        let result = Operand::parse::<E>(input);
        assert_eq!(err(input, ErrorKind::Tag), result);

        let input = "#$90a; ";
        let result = Operand::parse::<E>(input);
        assert_eq!(err("90a", ErrorKind::Eof), result);
    }

    #[test]
    fn instruction_success_1() {
        let input = "STZ #$0300; ";
        let result = Element::instruction::<E>(input);
        assert_eq!(
            Ok((
                "; ",
                Element::Instruction(Mnemonic::STZ, Operand::AbsoluteFour(0x0300))
            )),
            result
        )
    }

    #[test]
    fn instruction_success_2() {
        let input = "RTS\n";
        let result = Element::instruction::<E>(input);
        assert_eq!(
            Ok(("\n", Element::Instruction(Mnemonic::RTS, Operand::Implied))),
            result
        )
    }

    #[test]
    fn instruction_fail() {
        let input = "090";
        let result = Element::instruction::<E>(input);
        assert_eq!(err("090", ErrorKind::Alpha), result)
    }

    #[test]
    fn line_success() {
        let input = "  STZ #$0300\n ";
        let result = Line::parse::<E>(input);
        assert_eq!(
            Ok((
                " ",
                Line(vec![Element::Instruction(
                    Mnemonic::STZ,
                    Operand::AbsoluteFour(0x0300),
                )])
            )),
            result
        )
    }

    #[test]
    fn line_fail() {
        let input = "STZ #$0300\n ";
        let result = Line::parse::<E>(input);
        assert_eq!(err(input, ErrorKind::Space), result)
    }

    #[test]
    fn parse_success() {
        let input = "  STZ #$0300\n  RTS\n";
        let result = parse::<E>(input);
        assert_eq!(
            Ok((
                "",
                Parsed(vec![
                    Line(vec![Element::Instruction(
                        Mnemonic::STZ,
                        Operand::AbsoluteFour(0x0300),
                    )]),
                    Line(vec![Element::Instruction(Mnemonic::RTS, Operand::Implied,)]),
                ])
            )),
            result
        )
    }

    #[test]
    fn parse_fail_1() {
        let input = "  STZ #$0300\n  RTS\n ";
        let result = parse::<E>(input);
        assert_eq!(err(" ", ErrorKind::Eof), result)
    }

    #[test]
    fn parse_fail_2() {
        let input = "  STZ #$0300\n  RTS";
        let result = parse::<E>(input);
        assert_eq!(err("  RTS", ErrorKind::Eof), result)
    }
}
