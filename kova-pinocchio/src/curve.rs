use pinocchio::error::ProgramError;

// Constants for math
pub const VELOCITY_MAX_MULTIPLIER: u64 = 500;
pub const VELOCITY_COOL_DOWN_SEC: u64 = 60; // Cooldown back to 1.0x takes 60 secs

pub fn calculate_cost(
    current_supply: u64,
    tokens_to_buy: u64,
    base_k: u64,
    velocity: u64,
) -> Result<u64, ProgramError> {
    let supply_start = current_supply as u128;
    let supply_end = (current_supply
        .checked_add(tokens_to_buy)
        .ok_or(ProgramError::ArithmeticOverflow)?) as u128;
    let k = base_k as u128;

    let cost_start = (k * supply_start * supply_start) / 2;
    let cost_end = (k * supply_end * supply_end) / 2;
    let base_cost = cost_end
        .checked_sub(cost_start)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Velocity Penalty !
    let final_cost = (base_cost * (velocity as u128)) / 100;

    Ok(final_cost as u64)
}

pub fn calculate_new_velocity(
    current_velocity: u64,
    last_trade_time: i64,
    current_time: i64,
    tokens_bought: u64,
    total_supply: u64,
) -> u64 {
    let time_elapsed = (current_time.saturating_sub(last_trade_time)) as u64;

    let mut cooled_velocity = current_velocity;
    if time_elapsed > 0 {
        // Drop 10 points (0.1x) per second
        let decay = time_elapsed * 10;
        cooled_velocity = cooled_velocity.saturating_sub(decay);
    }

    // To make velocity below 100 (1.0x) impossible
    if cooled_velocity < 100 {
        cooled_velocity = 100;
    }

    // Percentage bought from total supply
    // Spike 1% per 1% supply being bought
    let percent_bought = (tokens_bought * 100) / total_supply.max(1);
    let spike = percent_bought * 20;
    let new_velocity = cooled_velocity + spike;

    if new_velocity > VELOCITY_MAX_MULTIPLIER {
        VELOCITY_MAX_MULTIPLIER
    } else {
        new_velocity
    }
}
