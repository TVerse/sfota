mod code_generator;
mod parser;

#[derive(Debug)]
pub enum Error<'a> {
    ParsingError(parser::Error<&'a str>),
    CodeGenError(code_generator::Error),
}

pub fn assemble(i: &str) -> Result<Vec<u8>, Error> {
    let parsed = parser::parse(i).map_err(Error::ParsingError)?;
    code_generator::generate_code(parsed).map_err(Error::CodeGenError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_assemble() {
        let input = "  STZ $0300\n  RTS\n";
        let result = assemble(input);
        assert_eq!(vec![0x9C, 0x00, 0x03, 0x60], result.unwrap())
    }

    #[test]
    fn loop_assemble() {
        let input = "  STZ $1234\nloop:\n  JMP loop\n";
        let result = assemble(input);
        assert_eq!(vec![0x9C, 0x34, 0x12, 0x4C, 0x03, 0x00], result.unwrap())
    }

    #[test]
    fn forward_assemble() {
        let input = "  JMP loop\nloop:\n  STZ $1234\n";
        let result = assemble(input);
        assert_eq!(vec![0x4C, 0x03, 0x00, 0x9C, 0x34, 0x12,], result.unwrap())
    }
}
