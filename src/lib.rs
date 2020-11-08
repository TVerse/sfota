mod code_generator;
mod parser;

#[derive(Debug)]
pub enum Error<'a> {
    ParsingError(parser::Error<&'a str>),
    CodeGenError(code_generator::Error)
}

pub fn assemble(i: &str) -> Result<Vec<u8>, Error> {
    let parsed = parser::parse(i).map_err(|e| Error::ParsingError(e))?; // TODO error + unwrap
    code_generator::generate_code(parsed).map_err(|e| Error::CodeGenError(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_assemble() {
        let input = "  STZ #$0300\n  RTS\n ";
        let result = assemble(input);
        assert_eq!(vec![0x9C, 0x00, 0x03, 0x60], result.unwrap())
    }
}
