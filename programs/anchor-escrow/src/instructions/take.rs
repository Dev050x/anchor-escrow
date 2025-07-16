use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{close_account, CloseAccount}, token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked}};

use crate::Escrow;



#[derive(Accounts)]
pub struct Take<'info>{
    #[account(mut)]
    pub taker:Signer<'info>,
    #[account(mut)]
    pub maker:SystemAccount<'info>,
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a:InterfaceAccount<'info,Mint>,
   #[account(
        mint::token_program = token_program
   )]
   pub mint_b:InterfaceAccount<'info , Mint>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority=taker,
        associated_token::token_program = token_program

    )]
    pub taker_ata_a:InterfaceAccount<'info , TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority=taker,
        associated_token::token_program = token_program

    )]
    pub taker_ata_b:InterfaceAccount<'info , TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority=maker,
        associated_token::token_program = token_program

    )]
    pub maker_ata_b:InterfaceAccount<'info , TokenAccount>,
    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = mint_b,
        has_one = maker,
        seeds = [b"escrow",maker.key().as_ref() , escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow:Account<'info , Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault : InterfaceAccount<'info,TokenAccount>,

    pub token_program: Interface<'info , TokenInterface>,
    pub system_program:Program<'info,System>,
    pub associated_token_program:Program<'info,AssociatedToken>

}

impl<'info> Take<'info>{
    //taker -> maker(mintb)
    pub fn transfer_to_maker(&mut self) -> Result<()>{ 
        let cpiContext = CpiContext::new(self.token_program.to_account_info(), TransferChecked{
            from:self.taker_ata_b.to_account_info(),
            mint:self.mint_b.to_account_info(),
            to:self.maker_ata_b.to_account_info(),
            authority:self.taker.to_account_info(),
        });
        transfer_checked(cpiContext, self.escrow.receive, self.mint_b.decimals)?;
        Ok(())
    }

    //transferign mint_a to taker & closign vault account & escrow will be auto close(you can see in Accounts(above) bcz owned by this program)
    pub fn transfer_to_taker_and_close_account(&mut self) -> Result<()> {


        let seeds = &[&b"escrow"[..] , self.maker.to_account_info().key.as_ref() , &self.escrow.seed.to_le_bytes() ,  &[self.escrow.bump]];
        let sigenr_seeds = &[&seeds[..]];

        let transfer_context = CpiContext::new_with_signer(self.token_program.to_account_info(), TransferChecked{
            from:self.vault.to_account_info(),
            mint:self.mint_a.to_account_info(),
            to:self.taker_ata_a.to_account_info(),
            authority:self.escrow.to_account_info()
        } , sigenr_seeds);

        transfer_checked(transfer_context, self.vault.amount, self.mint_a.decimals)?;

        let close_context = CpiContext::new_with_signer(self.token_program.to_account_info(), CloseAccount{
            account:self.vault.to_account_info(),
            destination:self.maker.to_account_info(),
            authority:self.escrow.to_account_info(),
        } , sigenr_seeds);

        close_account(close_context)?;

        Ok(())
    }
}