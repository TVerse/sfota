use crate::parser::{Instruction, OperandType};

use super::parser::{Element, Line,  Parsed};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {}

enum EmitResult {
    FullyDetermined(Vec<u8>),
    PartiallyUnknown(Instruction)
}

#[derive(Debug)]
struct GenerationState {
    program_counter: u16,
    label_locations: HashMap<String, u16>,
}

impl Default for GenerationState {
    fn default() -> Self {
        GenerationState {
            program_counter: 0,
            label_locations: HashMap::new(),
        }
    }
}

pub fn generate_code(parsed: Parsed) -> Result<Vec<u8>, Error> {
    let mut state = GenerationState::default();
    let res = parsed
        .0
        .into_iter()
        .map(|l| generate_line(l, &mut state))
        .collect::<Result<Vec<_>, _>>()
        .map(|v| v.into_iter().flatten().collect::<Vec<_>>())
        .map(|ers| ers.into_iter().map(|er| {
            match er {
                EmitResult::FullyDetermined(bytes) => bytes,
                EmitResult::PartiallyUnknown(_) => panic!("PartiallyUnknown")
            }
        }).flatten().collect::<Vec<u8>>());
    dbg!(state);
    res
}

fn generate_line(line: Line, generation_state: &mut GenerationState) -> Result<Vec<EmitResult>, Error> {
    line.0
        .into_iter()
        .map(|element| match element {
            Element::Instruction(instruction) => emit_instruction(instruction, generation_state),
            Element::Label(l) => {
                generation_state.label_locations.insert(l, generation_state.program_counter);
                Ok(EmitResult::FullyDetermined(vec![])) // TODO pretty wasteful?
            }
        })
        .collect::<Result<Vec<EmitResult>, _>>()
}

fn emit_instruction(instruction: Instruction, generation_state: &mut GenerationState) -> Result<EmitResult, Error> {
    match &instruction {
        Instruction::StzAbsolute(ot) => {
            match ot {
                OperandType::Known(addr) => {
                    let [l, h] = addr.to_le_bytes();
                    let bytes = vec![0x9C, l, h];
                    generation_state.program_counter = generation_state.program_counter.checked_add(bytes.len() as u16).expect("Program counter overflow"); // TODO
                    Ok(EmitResult::FullyDetermined(bytes))
                }
                OperandType::Label(l) => {
                    match generation_state.label_locations.get(l) {
                        Some(loc) => {
                            let [l, h] = loc.to_le_bytes();
                            let bytes = vec![0x9C, l, h];
                            generation_state.program_counter = generation_state.program_counter.checked_add(bytes.len() as u16).expect("Program counter overflow"); // TODO
                            Ok(EmitResult::FullyDetermined(bytes))
                        }
                        None => Ok(EmitResult::PartiallyUnknown(instruction))
                    }
                }
            }
        }
        Instruction::RtsStack => Ok(EmitResult::FullyDetermined(vec![0x60])),
    }
}
