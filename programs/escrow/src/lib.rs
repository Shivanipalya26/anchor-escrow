pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("47J6XBmMfq6JNJ1uDa3cQPVVAuCWa3dBax5v7FDE7Cta");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit_amt: u64, receive_amt: u64 ) -> Result<()>{
        ctx.accounts.init_escrow(seed, &ctx.bumps, receive_amt)?;
        ctx.accounts.deposit(deposit_amt)      
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.transfer_and_close_vault()
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }
}
