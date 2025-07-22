#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HoMPu3r77wHfzEYfRfKtQcV8RvM571t6KmyxMrqBKiy5");

#[program]
pub mod anchor_escrow {
    use super::*;

    pub fn initialize_escrow(ctx:Context<Make> , seed:u64 , receive:u64 , amount:u64 ) -> Result<()>{
        ctx.accounts.init_escrow(seed , receive , &ctx.bumps)?;
        ctx.accounts.deposit(amount)?;
        Ok(())
    }
    
    pub fn taking_refund(ctx:Context<Refund>) -> Result<()>{
        ctx.accounts.refund_and_close_vault()?;
        Ok(())
    }

    pub fn finalize_deal(ctx:Context<Take>) -> Result<()> {
        ctx.accounts.transfer_to_maker()?;
        ctx.accounts.transfer_to_taker_and_close_account()?;
        Ok(())
    }

    
}
