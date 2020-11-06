use super::*;

pub(crate) fn generate_code(parsed: Parsed) -> Vec<u8> {
    parsed
        .0
        .into_iter()
        .flat_map(|line| {
            line.0.into_iter().flat_map(|element| match element {
                Element::Instruction(mnemonic, operand) => match mnemonic {
                    Mnemonic::STZ => match operand {
                        Operand::AbsoluteFour(op) => {
                            let [l, h] = op.to_le_bytes();
                            vec![0x9C, l, h]
                        }
                        Operand::Implied => panic!("Unknown operand for STA"),
                    },
                    Mnemonic::RTS => match operand {
                        Operand::AbsoluteFour(_) => panic!("Unknown operand for RTS"),
                        Operand::Implied => vec![0x60],
                    },
                    Mnemonic::UserDefined(m) => panic!(
                        "Found mnemonic {}, user-defined mnemonics are not yet supported",
                        m
                    ),
                },
            })
        })
        .collect()
}
