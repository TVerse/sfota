use std::collections::HashMap;

use crate::parser::{AddressingMode, Instruction};

use super::parser::{Element, Parsed};
use crate::code_generator::lookup_tables::lookup;

use crate::parser::operand_expression::OperandExpression;

mod lookup_tables;

type Labels = HashMap<String, u16>;

struct OperandTooLong(OperandExpression);

// TODO multiple errors?
#[derive(Debug)]
pub enum Error {
    InvalidOperand,
}

enum EmitResult {
    FullyDetermined(Vec<u8>),
    PartiallyUnknown(Instruction),
    NoBytesRequired,
}

pub enum Target {
    ZeroBytes,
    OneByte(u8),
    TwoByte(u16),
}

impl Target {
    fn resolve_from(addressing_mode: &AddressingMode, _labels: &Labels) -> Option<Target> {
        match addressing_mode {
            AddressingMode::Immediate(_oe) => {}
            AddressingMode::IndexedX(_oe) => {}
            AddressingMode::IndexedY(_oe) => {}
            AddressingMode::IndirectIndexedY(_oe) => {}
            AddressingMode::IndexedIndirectX(_oe) => {}
            AddressingMode::Indirect(_oe) => {}
            AddressingMode::AbsoluteOrRelative(_oe) => {}
            AddressingMode::NoOperand => {}
        }

        todo!()
    }
}

#[derive(Debug)]
struct GenerationState {
    program_counter: u16,
    label_locations: Labels,
}

impl Default for GenerationState {
    fn default() -> Self {
        GenerationState {
            program_counter: 0,
            label_locations: HashMap::new(),
        }
    }
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
    dbg!(&generation_state);
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
    let GenerationState {
        program_counter,
        label_locations,
    } = generation_state;
    let (result, size) = handle_instruction(&instruction, label_locations)?;
    increment_pc(program_counter, size);
    Ok(result)
}

fn handle_instruction(
    instruction: &Instruction,
    label_locations: &Labels,
) -> Result<(EmitResult, u16), Error> {
    let Instruction {
        mnemonic,
        addressing_mode,
    } = instruction;
    let target = Target::resolve_from(addressing_mode, label_locations);
    let _result = lookup(mnemonic, &target.unwrap())?;

    todo!()
}

fn increment_pc(program_counter: &mut u16, by: u16) {
    *program_counter = program_counter
        .checked_add(by)
        .expect("Program counter overflow")
}
