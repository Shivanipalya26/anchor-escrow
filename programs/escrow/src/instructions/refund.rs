use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{ Mint, TokenAccount, TransferChecked, TokenInterface, CloseAccount, transfer_checked, close_account }
};

use crate::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut, 
        close = maker,  // maker gets the rent after closing
        has_one = mint_a,   // validates the mint_a of this struct with the mint_a of the Esrow struct
        has_one = maker,    // similar to mint_a
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,    // only mutable because it is already initialised and just need to be closed
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    // this vault is needed to actually hold the tokens that are being escrowed (token A)
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund_and_close_vault(&mut self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]; 1] = &[&[
            b"escrow",
            self.maker.key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump]
        ]];

        // get no. of decimals for this mint
        let decimals = self.mint_a.decimals;

        // program being invoked in the CPI
        let cpi_program = self.system_program.to_account_info();

        // create the TransferChecked struct with required accounts
        let cpi_accounts = TransferChecked {
            mint: self.mint_a.to_account_info(),
            from: self.vault.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // combine the accounts and program into a "CpiContext"
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_ctx, self.vault.amount, decimals)?;

        // accounts for closing vault
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let close_cpi_ctx = CpiContext::new_with_signer(self.system_program.to_account_info(), close_accounts, signer_seeds);

        // close the vault ATA and refund rent
        close_account(close_cpi_ctx)?;

        Ok(())
    }
}