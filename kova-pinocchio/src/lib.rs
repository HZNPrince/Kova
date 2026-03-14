use pinocchio::{ProgramResult, account::AccountView, address::Address, entrypoint};
mod curve;
mod instructions;
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
        0 => instructions::initialize::process_initialize(_program_id, _accounts, args),
        1 => instructions::trade::process_trade(_program_id, _accounts, args),
        _ => Err(pinocchio::error::ProgramError::InvalidInstructionData),
    }
}
