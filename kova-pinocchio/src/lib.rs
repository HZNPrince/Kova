#![no_std]

use pinocchio::{ProgramResult, account::AccountView, address::Address, entrypoint};

mod state;

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Address,
    _accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(pinocchio::error::ProgramError::InvalidInstructionData);
    }

    // First Byte determines the action
    let instruction_id = instruction_data[0];

    // rest are args for the action
    let args = &instruction_data[1..];

    match instruction_id {
        0 => process_initialize(_program_id, _accounts, args),
        1 => process_trade(_program_id, _accounts, args),
        _ => Err(pinocchio::error::ProgramError::InvalidInstructionData),
    }
}

// Handler for instruction_id = 0
fn process_initialize(
    _program_id: &Address,
    _accounts: &[AccountView],
    _args: &[u8],
) -> ProgramResult {
    // TODO: Write Initialize Logic
    Ok(())
}

// Handler for instruction_id = 1
fn process_trade(_program_id: &Address, _accounts: &[AccountView], _args: &[u8]) -> ProgramResult {
    // TODO: Write trade Logic
    Ok(())
}
