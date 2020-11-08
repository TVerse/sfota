use crate::code_generator::Error::InvalidAddressingMode;

use super::parser::{Element, Mnemonic, Operand, OperandDefinition, Parsed, Line};

#[derive(Debug)]
pub enum Error {
    InvalidAddressingMode(&'static str)
}

pub fn generate_code(parsed: Parsed) -> Result<Vec<u8>, Error> {
    let result: Result<Vec<Vec<u8>>, _> = parsed
        .0
        .into_iter()
        .map(|line| generate_line(line))
        .collect();
    result
        .map(|x| x.into_iter().flatten().collect::<Vec<u8>>())
}

fn generate_line(line: Line) -> Result<Vec<u8>, Error> {
    line.0.into_iter().map(|element| match element {
        Element::Instruction(mnemonic, operand) => match mnemonic {
            Mnemonic::STZ => stz(&operand),
            Mnemonic::RTS => rts(operand),
        },
    })
        .collect::<Result<Vec<_>, _>>().map(|x| x.into_iter().flatten().collect::<Vec<u8>>())
}

fn stz(operand: &Operand) -> Result<Vec<u8>, Error> {
    match operand {
        Operand::Absolute(od) => match od {
            OperandDefinition::Direct(direct) => {
                let [l, h] = direct.to_le_bytes();
                Ok(vec![0x9C, l, h])
            }
        },
        Operand::Implied => Err(InvalidAddressingMode("Implied")),
    }
}

fn rts(operand: Operand) -> Result<Vec<u8>, Error> {
    match operand {
        Operand::Absolute(_) => Err(InvalidAddressingMode("Absolute")),
        Operand::Implied => Ok(vec![0x60]),
    }
}
