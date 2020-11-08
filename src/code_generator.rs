use Error::InvalidAddressingMode;

use super::parser::{Element, Line, Operand, Parsed};
use crate::parser::Instruction;

#[derive(Debug)]
pub enum Error {
    InvalidAddressingMode(&'static str),
}

pub fn generate_code(parsed: Parsed) -> Result<Vec<u8>, Error> {
    let result: Result<Vec<Vec<u8>>, _> = parsed.0.into_iter().map(generate_line).collect();
    result.map(|x| x.into_iter().flatten().collect::<Vec<u8>>())
}

fn generate_line(line: Line) -> Result<Vec<u8>, Error> {
    line.0
        .into_iter()
        .map(|element| match element {
            Element::Instruction(instruction) => match instruction {
                Instruction::StzAbsolute(addr) => {
                    let [l, h] = addr.to_le_bytes();
                    Ok(vec![0x9C, l, h])
                },
                Instruction::RtsStack => Ok(vec![0x60])
            },
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|x| x.into_iter().flatten().collect::<Vec<u8>>())
}
