use std::collections::HashMap;

use crate::parser::{Instruction, OperandType};

use super::parser::{Element, Parsed};

// TODO multiple errors?
#[derive(Debug)]
pub enum Error {}

enum EmitResult {
    FullyDetermined(Vec<u8>),
    PartiallyUnknown(Instruction),
    NoBytesRequired,
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

macro_rules! impl_16bit {
    ($ot:expr, $instruction:expr, $generation_state:expr) => {{
        let GenerationState {
            program_counter,
            label_locations,
        } = $generation_state;
        let instruction_byte = $instruction.instruction_byte();
        match $ot {
            OperandType::Known(addr) => known_16bit(instruction_byte, program_counter, *addr),
            OperandType::Label(l) => match label_locations.get(l) {
                Some(addr) => known_16bit(instruction_byte, program_counter, *addr),
                None => {
                    increment_pc(program_counter, 3);
                    Ok(EmitResult::PartiallyUnknown($instruction))
                }
            },
        }
    }};
}

// TODO the second time around the PC is still being incremented.
// That's probably fine, since we don't use it to create new labels?
pub fn generate_code(parsed: Parsed) -> Result<Vec<u8>, Error> {
    dbg!(&parsed);
    let mut generation_state = GenerationState::default();
    let res = parsed
        .0
        .into_iter()
        .map(|element| match element {
            Element::Instruction(instruction) => {
                emit_instruction(instruction, &mut generation_state)
            }
            Element::Label(l) => {
                dbg!(&generation_state);
                generation_state
                    .label_locations
                    .insert(l, generation_state.program_counter);
                Ok(EmitResult::NoBytesRequired) // TODO pretty wasteful?
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .and_then(|v| fill_in_states(v.into_iter(), &mut generation_state))
        .map(|ers| {
            ers.into_iter()
                .filter_map(|er| match er {
                    EmitResult::FullyDetermined(bytes) => Some(bytes),
                    EmitResult::PartiallyUnknown(_) => panic!("PartiallyUnknown"),
                    EmitResult::NoBytesRequired => None,
                })
                .flatten()
                .collect::<Vec<u8>>()
        });
    dbg!(generation_state);
    res
}

fn fill_in_states<I: Iterator<Item = EmitResult>>(
    ers: I,
    generation_state: &mut GenerationState,
) -> Result<Vec<EmitResult>, Error> {
    ers.map(|er| match er {
        EmitResult::PartiallyUnknown(instruction) => {
            emit_instruction(instruction, generation_state)
        }
        other => Ok(other),
    })
    .collect()
}

fn emit_instruction(
    instruction: Instruction,
    generation_state: &mut GenerationState,
) -> Result<EmitResult, Error> {
    match &instruction {
        Instruction::StzAbsolute(ot) => impl_16bit!(ot, instruction, generation_state),
        Instruction::JmpAbsolute(ot) => impl_16bit!(ot, instruction, generation_state),
        Instruction::RtsStack => Ok(EmitResult::FullyDetermined(vec![0x60])),
    }
}

fn increment_pc(program_counter: &mut u16, by: u16) {
    *program_counter = program_counter
        .checked_add(by)
        .expect("Program counter overflow")
}

fn known_16bit(
    instruction_byte: u8,
    program_counter: &mut u16,
    val: u16,
) -> Result<EmitResult, Error> {
    let [l, h] = val.to_le_bytes();
    let bytes = vec![instruction_byte, l, h];
    increment_pc(program_counter, 3);
    Ok(EmitResult::FullyDetermined(bytes))
}

// fn known_8bit(instruction_byte: u8, program_counter: &mut u16, val: u8) -> Result<EmitResult, Error> {
//     let bytes = vec![instruction_byte, val];
//     increment_pc(program_counter, 2);
//     Ok(EmitResult::FullyDetermined(bytes))
// }
