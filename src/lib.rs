mod code_generator;
mod parser;

#[derive(Debug, Eq, PartialEq)]
struct Parsed<'a>(Vec<Line<'a>>);

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

pub fn assemble(i: &str) -> Vec<u8> {
    let parsed = parser::parse::<()>(i).unwrap(); // TODO error + unwrap
    code_generator::generate_code(parsed.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_assemble() {
        let input = "  STA #$0300\n  RTS\n";
        let result = assemble(input);
        assert_eq!(vec![0x8D, 0x00, 0x03, 0x60], result)
    }
}
