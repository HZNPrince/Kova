use pinocchio::{ProgramResult, account::AccountView, address::Address, error::ProgramError};

use crate::curve;
use crate::state::LaunchAccount;

pub fn process_trade(program_id: &Address, accounts: &[AccountView], args: &[u8]) -> ProgramResult {
    // Extract accounts
    let mut accounts_iter = accounts.iter();

    let user = accounts_iter
        .next()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;
    let launch_account_info = accounts_iter
        .next()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;

    // Security Checks
    if !user.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let owner = unsafe { launch_account_info.owner() };
    if owner != program_id {
        return Err(ProgramError::Custom(1)); // Custom Error: IllegalOwner
    }

    // Parse Arguments (9 bytes: 1 byte for Buy or Sell and 8 bytes for u64 token_amount)
    if args.len() < 9 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let is_buy = args[0] == 0;

    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&args[1..9]);
    let token_amount = u64::from_le_bytes(amount_bytes);

    if token_amount == 0 {
        return Err(ProgramError::Custom(2)); // Custom Error: AmountCannotBeZero
    }

    // Read State
    let data = &mut *(launch_account_info.try_borrow_mut()?);
    let launch_state = LaunchAccount::from_bytes_mut(data)?;

    if launch_state.is_graduated == 1 {
        return Err(ProgramError::Custom(3)); // Custom Error: AlreadyGraduated
    }

    // Fetch time
    let current_timestamp = launch_state.last_trade_timestamp + 10;

    // Calculation
    if is_buy {
        launch_state.current_velocity = curve::calculate_new_velocity(
            launch_state.current_velocity,
            launch_state.last_trade_timestamp,
            current_timestamp,
            token_amount,
            launch_state.supply_tokens,
        );
        launch_state.last_trade_timestamp = current_timestamp;

        let cost_lamports = curve::calculate_cost(
            launch_state.supply_tokens,
            token_amount,
            launch_state.base_k,
            launch_state.current_velocity,
        )?;

        // CPI to transfer user sols to vault
        // CPI to mint 'token_amounts' Tokens to User

        // Update State
        launch_state.supply_tokens = launch_state
            .supply_tokens
            .checked_add(token_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        launch_state.reserve_sol = launch_state
            .reserve_sol
            .checked_add(cost_lamports)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    } else {
        // No Velocity Penalty for selling
        let reward_lamports = curve::calculate_cost(
            launch_state.supply_tokens.saturating_sub(token_amount),
            token_amount,
            launch_state.base_k,
            100,
        )?;

        // CPI to transfer vault to user
        // CPI to burn 'token_amounts' Tokens from User

        // Update state
        launch_state.supply_tokens = launch_state
            .supply_tokens
            .checked_sub(token_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        launch_state.reserve_sol = launch_state
            .reserve_sol
            .checked_sub(reward_lamports)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    let graduation_target: u64 = 500 * 1_000_000_000;
    if launch_state.reserve_sol >= graduation_target {
        launch_state.is_graduated = 1;

        todo!(
            // CPI call to raydium to construct AMM and deposit liquidity
        );
    }

    Ok(())
}
