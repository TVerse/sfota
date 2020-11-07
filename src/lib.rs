mod code_generator;
mod parser;

pub fn assemble(i: &str) -> Vec<u8> {
    let parsed = parser::parse(i).unwrap(); // TODO error + unwrap
    code_generator::generate_code(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_assemble() {
        let input = "  STZ #$0300\n  RTS\n";
        let result = assemble(input);
        assert_eq!(vec![0x9C, 0x00, 0x03, 0x60], result)
    }
}
