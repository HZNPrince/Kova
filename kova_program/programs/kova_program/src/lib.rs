use anchor_lang::prelude::*;

declare_id!("9gUKSrP6jWq3wbkVxhuZxXokXsZvqaZgdnr17Ty3Hkam");

#[program]
pub mod kova_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
