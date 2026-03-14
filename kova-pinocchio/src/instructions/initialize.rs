use pinocchio::{ProgramResult, account::AccountView, address::Address, error::ProgramError};
use pinocchio_system::instructions::CreateAccount;
use crate::state::LaunchAccount;

pub fn process_initialize(
    program_id: &Address,
    accounts: &[AccountView],
    args: &[u8],
) -> ProgramResult {
    let mut accounts_iter = accounts.iter();

    // 1. Extract accounts
    let creator = accounts_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let launch_state_account = accounts_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let vault_account = accounts_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let mint = accounts_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;

    // 2. Security Checks
    if !creator.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Parse the Args
    if args.len() < 3 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let vesting_choice = args[0]; // first byte is vesting choice
    let state_bump_val = args[1];
    let vault_bump_val = args[2];

    // 3. Create the Launch State PDA
    let mint_address = mint.address().as_ref();
    let bump_state = [state_bump_val];
    let signer_seeds_state = [
        pinocchio::cpi::Seed::from(b"state"),
        pinocchio::cpi::Seed::from(mint_address),
        pinocchio::cpi::Seed::from(&bump_state),
    ];
    let state_pda_signer = pinocchio::cpi::Signer::from(&signer_seeds_state);

    CreateAccount::with_minimum_balance(
        creator,
        launch_state_account,
        LaunchAccount::LEN as u64,
        program_id,
        None,
    )?.invoke_signed(&[state_pda_signer])?;

    // 4. Create the Vault PDA
    let bump_vault = [vault_bump_val];
    let signer_seeds_vault = [
        pinocchio::cpi::Seed::from(b"vault"),
        pinocchio::cpi::Seed::from(mint_address),
        pinocchio::cpi::Seed::from(&bump_vault),
    ];
    let vault_pda_signer = pinocchio::cpi::Signer::from(&signer_seeds_vault);

    CreateAccount::with_minimum_balance(
        creator,
        vault_account,
        0, // Vault needs 0 space because it only holds pure SOL lamports
        program_id,
        None,
    )?.invoke_signed(&[vault_pda_signer])?;

    // 5. Safely Cast and Initialize the State variables
    let data = &mut *(launch_state_account.try_borrow_mut()?);
    let launch_state = LaunchAccount::from_bytes_mut(data)?;

    launch_state.creator = creator.address().clone();
    launch_state.dev_vesting_choice = vesting_choice;
    launch_state.state_bump = state_bump_val;
    launch_state.vault_bump = vault_bump_val;
    launch_state.is_graduated = 0; // 0 = false
    launch_state.reserve_sol = 0;
    launch_state.supply_tokens = 0;
    launch_state.base_k = 100_000; // Base Curve Multiplier

    Ok(())
}
