use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{alpha1, alphanumeric1, hex_digit1, newline, space1};
use nom::combinator::{all_consuming, map, map_parser, opt, recognize};
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

#[derive(Debug, Eq, PartialEq)]
pub struct Parsed<'a>(Vec<Line<'a>>);

#[derive(Debug, Eq, PartialEq)]
struct Line<'a>(Vec<Element<'a>>);

#[derive(Debug, Eq, PartialEq)]
enum Element<'a> {
    Instruction(Mnemonic<'a>, Operand<'a>),
}

#[derive(Debug, Eq, PartialEq)]
struct Mnemonic<'a>(&'a str);

#[derive(Debug, Eq, PartialEq)]
enum Operand<'a> {
    AbsoluteFour(&'a str),
    Implied,
}

pub fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Parsed, E> {
    map(all_consuming(many0(line)), Parsed)(i)
}

fn line<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Line<'a>, E> {
    map(delimited(space1, instruction, newline), |e| Line(vec![e]))(i)
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

fn mnemonic<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Mnemonic<'a>, E> {
    map(valid_word, Mnemonic)(i)
}

fn operand<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Operand<'a>, E> {
    map(
        preceded(tag("#$"), map_parser(hex_digit1, take(4usize))),
        Operand::AbsoluteFour,
    )(i)
}

fn instruction<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Element<'a>, E> {
    map(
        tuple((mnemonic, opt(preceded(space1, operand)))),
        |(mnemonic, o)| match o {
            Some(operand) => Element::Instruction(mnemonic, operand),
            None => Element::Instruction(mnemonic, Operand::Implied),
        },
    )(i)
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
        let result = mnemonic::<E>(input);
        assert_eq!(Ok(("\n", Mnemonic("RTS"))), result);
        let input = "This_is_a_val1d_mnemOnIc end";
        let result = mnemonic::<E>(input);
        assert_eq!(Ok((" end", Mnemonic("This_is_a_val1d_mnemOnIc"))), result)
    }

    #[test]
    fn mnemonic_fail() {
        let input = " not This_is_a_val1d_mnemOnIc end";
        let result = mnemonic::<E>(input);
        assert_eq!(err(input, ErrorKind::Alpha), result)
    }

    #[test]
    fn operand_success() {
        let input = "#$90aB; ";
        let result = operand::<E>(input);
        assert_eq!(Ok(("; ", Operand::AbsoluteFour("90aB"))), result)
    }

    #[test]
    fn operand_failure() {
        let input = "90aB; ";
        let result = operand::<E>(input);
        assert_eq!(err(input, ErrorKind::Tag), result);

        let input = "#$90a; ";
        let result = operand::<E>(input);
        assert_eq!(err("90a", ErrorKind::Eof), result);
    }

    #[test]
    fn instruction_success_1() {
        let input = "STA #$0300; ";
        let result = instruction::<E>(input);
        assert_eq!(
            Ok((
                "; ",
                Element::Instruction(Mnemonic("STA"), Operand::AbsoluteFour("0300"))
            )),
            result
        )
    }

    #[test]
    fn instruction_success_2() {
        let input = "RTS\n";
        let result = instruction::<E>(input);
        assert_eq!(
            Ok((
                "\n",
                Element::Instruction(Mnemonic("RTS"), Operand::Implied)
            )),
            result
        )
    }

    #[test]
    fn instruction_fail() {
        let input = "090";
        let result = instruction::<E>(input);
        assert_eq!(err("090", ErrorKind::Alpha), result)
    }

    #[test]
    fn line_success() {
        let input = "  STA #$0300\n ";
        let result = line::<E>(input);
        assert_eq!(
            Ok((
                " ",
                Line(vec![Element::Instruction(
                    Mnemonic("STA"),
                    Operand::AbsoluteFour("0300"),
                )])
            )),
            result
        )
    }

    #[test]
    fn line_fail() {
        let input = "STA #$0300\n ";
        let result = line::<E>(input);
        assert_eq!(err(input, ErrorKind::Space), result)
    }

    #[test]
    fn parse_success() {
        let input = "  STA #$0300\n  RTS\n";
        let result = parse::<E>(input);
        assert_eq!(
            Ok((
                "",
                Parsed(vec![
                    Line(vec![Element::Instruction(
                        Mnemonic("STA"),
                        Operand::AbsoluteFour("0300"),
                    )]),
                    Line(vec![Element::Instruction(
                        Mnemonic("RTS"),
                        Operand::Implied
                    )]),
                ])
            )),
            result
        )
    }
}
