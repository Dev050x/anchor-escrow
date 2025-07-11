use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow{
    pub maker: Pubkey,
    pub mint_a:Pubkey,
    pub mint_b:Pubkey,
    pub seed:u64,
    pub bump:u8,
    pub receive:u64
}