use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface , TransferChecked , transfer_checked}};

use crate::Escrow;

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Make<'info>{
    #[account(mut)]
    pub maker:Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a:InterfaceAccount<'info,Mint>,
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b:InterfaceAccount<'info,Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority=maker,
        associated_token::token_program = token_program

    )]
    pub maker_ata_a:InterfaceAccount<'info , TokenAccount>,
    #[account(
        init,
        payer = maker,
        seeds = [b"escrow",maker.key().as_ref() , seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + Escrow::INIT_SPACE
    )]
    pub escrow:Account<'info , Escrow>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault : InterfaceAccount<'info,TokenAccount>,

    pub token_program: Interface<'info , TokenInterface>,
    pub system_program:Program<'info,System>,
    pub associated_token_program:Program<'info,AssociatedToken>

}

impl<'info> Make<'info> {
    pub fn init_escrow(&mut self , seed:u64 , receive:u64 , bumps:&MakeBumps)  -> Result<()>  {

        self.escrow.set_inner(
            Escrow {
                maker:self.maker.key(),
                mint_a:self.mint_a.key(),
                mint_b:self.mint_b.key(),
                seed,
                bump:bumps.escrow,
                receive,
            }
            
        );

        Ok(())
    }

    pub fn deposit(&mut self , deposit:u64) -> Result<()> {
        msg!("inside the deposit....");

        let cpi_context =CpiContext::new(self.token_program.to_account_info(), TransferChecked{
            from:self.maker_ata_a.to_account_info(),
            mint:self.mint_a.to_account_info(),
            to:self.vault.to_account_info(),
            authority:self.maker.to_account_info()
        });

        transfer_checked(cpi_context, deposit, self.mint_a.decimals)?;
        msg!("Deposit of {} tokens from {}", deposit, self.maker.key());
        Ok(())
    }
}

