use pinocchio::{
    ProgramResult,
    account::AccountView,
    address::Address,
    cpi::Signer,
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_system::instructions::Transfer;
use pinocchio_token::instructions::{Burn, MintTo};

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
    let vault_account = accounts_iter
        .next()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;
    let user_ata = accounts_iter
        .next()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;
    let mint = accounts_iter
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
    let current_timestamp = Clock::get()?.unix_timestamp;

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
        Transfer {
            from: user,
            to: vault_account,
            lamports: cost_lamports,
        }
        .invoke()?;

        // CPI to mint 'token_amounts' Tokens to User
        let mint_address = mint.address().as_ref();
        let bump_state = [launch_state.state_bump];
        let signer_seeds_state = [
            // launch state is the pda
            pinocchio::cpi::Seed::from(b"state"),
            pinocchio::cpi::Seed::from(mint_address),
            pinocchio::cpi::Seed::from(&bump_state),
        ];
        let pda_signer = Signer::from(&signer_seeds_state);

        MintTo {
            mint,
            account: user_ata,
            mint_authority: launch_account_info,
            amount: token_amount,
        }
        .invoke_signed(&[pda_signer])?;

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

        // MANUALLY transfer lamports from Vault back to User
        // Because Kova owns the Vault and it has 0 data, we can directly modify pointers.
        let vault_lamports = vault_account.lamports();
        let user_lamports = user.lamports();

        vault_account.set_lamports(
            vault_lamports
                .checked_sub(reward_lamports)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );
        user.set_lamports(
            user_lamports
                .checked_add(reward_lamports)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );

        // CPI to burn 'token_amounts' Tokens from User's ATA
        Burn {
            account: user_ata,
            mint,
            authority: user,
            amount: token_amount,
        }
        .invoke()?;

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
