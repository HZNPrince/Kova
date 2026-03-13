use pinocchio::{address::Address, error::ProgramError};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VestingChoice {
    None = 0,     // 0 Months
    Lite = 1,     // 1 Months
    Commited = 2, // 3 Months
    Zealot = 3,   // 6 Months
}

#[repr(C)]
#[derive(Clone)]
pub struct LaunchAccount {
    pub creator: Address,
    pub mint: Address,
    pub reserve_sol: u64,
    pub supply_tokens: u64,
    pub is_graduated: u8,

    // Dev Vesting States
    pub dev_vesting_choice: u8,
    pub padding: [u8; 6],
    pub dev_allocate_tokens: u64,
    pub dev_lock_end_time: i64,

    // Dynamic Curve Parameters
    pub base_k: u64,
    pub current_velocity: u64,
    pub last_trade_timestamp: i64,
}

impl LaunchAccount {
    pub const LEN: usize = core::mem::size_of::<Self>();

    pub fn from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::AccountDataTooSmall);
        }
        let ptr = data.as_mut_ptr() as *mut Self; // Safe rust would fail to cast [u8] to LaunchAccount

        let account = unsafe { &mut *ptr }; // deref the raw mut pointer to return &mut LaunchAccount

        Ok(account)
    }
}
