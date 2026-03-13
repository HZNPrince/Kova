use pinocchio::{ProgramResult, account::AccountView, address::Address, error::ProgramError};

use crate::state::LaunchAccount;

pub fn process_initialize(
    program_id: &Address,
    accounts: &[AccountView],
    args: &[u8],
) -> ProgramResult {
    // iter over the accounts
    let mut accounts_iter = accounts.iter();

    // extract accounts in the exact order we expect
    let creator = accounts_iter
        .next()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;
    let launch_account_info = accounts_iter
        .next()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;

    // Security Checks
    if !creator.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    unsafe {
        if launch_account_info.owner() != program_id {
            return Err(ProgramError::IllegalOwner);
        }
    }

    // Parse the Args
    if args.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let vesting_choice = args[0]; // first byte is vesting choice

    let data = unsafe { &mut *(launch_account_info.try_borrow_mut()?) };

    let launch_state = LaunchAccount::from_bytes_mut(data)?;

    launch_state.creator = creator.address().clone();
    launch_state.dev_vesting_choice = vesting_choice;
    launch_state.is_graduated = 0; // 0 = false
    launch_state.reserve_sol = 0;
    launch_state.supply_tokens = 0;
    launch_state.base_k = 100_000; // Base Curve Multiplier

    Ok(())
}
