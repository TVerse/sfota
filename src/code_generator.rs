use super::*;

pub(crate) fn generate_code(parsed: Parsed) -> Vec<u8> {
    parsed
        .0
        .into_iter()
        .flat_map(|line| {
            line.0.into_iter().flat_map(|element| match element {
                Element::Instruction(mnemonic, operand) => {
                    if mnemonic.0 == "STZ" {
                        match operand {
                            Operand::AbsoluteFour(op) => {
                                let [l, h] = u16::from_str_radix(op, 16).unwrap().to_le_bytes();
                                vec![0x9C, l, h]
                            }
                            Operand::Implied => panic!("Unknown operand for STA"),
                        }
                    } else if mnemonic.0 == "RTS" {
                        match operand {
                            Operand::AbsoluteFour(_) => panic!("Unknown operand for RTS"),
                            Operand::Implied => vec![0x60],
                        }
                    } else {
                        panic!("Unknown mnemonic")
                    }
                }
            })
        })
        .collect()
}
