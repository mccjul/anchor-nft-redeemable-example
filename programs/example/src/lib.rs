use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;

use anchor_spl::token::{self, Mint, Token, TokenAccount};
use core::mem::size_of;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod example {
    use super::*;
    use spl_token::instruction::AuthorityType;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        ctx.accounts.player_account.authority = ctx.accounts.authority.key();
        ctx.accounts.player_account.bump = *ctx.bumps.get("player_account").unwrap();
        ctx.accounts.player_account.items = vec![Item {
            item_type: ItemType::Fire,
            strength: 3,
        }];
        Ok(())
    }
    pub fn mint_item(ctx: Context<MintItem>) -> ProgramResult {
        let item = ctx.accounts.player.items.remove(0);
        // create metadata
        ctx.accounts.nft_metadata.item = item;
        ctx.accounts.nft_metadata.self_bump = *ctx.bumps.get("nft_metadata").unwrap();
        ctx.accounts.nft_metadata.mint = ctx.accounts.nft_mint.key();

        // We got to mint to your newly created wallet
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                token::MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    to: ctx.accounts.nft_token.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            1,
        )?;

        token::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                token::SetAuthority {
                    account_or_mint: ctx.accounts.nft_mint.to_account_info().clone(),
                    current_authority: ctx.accounts.authority.to_account_info().clone(),
                },
            ),
            AuthorityType::MintTokens,
            None,
        )?;

        Ok(())
    }
    pub fn redeem(ctx: Context<Redeem>) -> ProgramResult {
        require!(ctx.accounts.nft_token.amount == 1, InvalidTokenAmount);
        let item = ctx.accounts.nft_metadata.item;
        ctx.accounts.player.items.push(item);
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                token::Burn {
                    mint: ctx.accounts.nft_mint.to_account_info().clone(),
                    to: ctx.accounts.nft_token.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            1,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init,
    seeds = [authority.key().as_ref()],
    bump,
    payer = authority,
    space = 8 + 10000
    )]
    pub player_account: Account<'info, Player>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintItem<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)]
    pub player: Account<'info, Player>,
    #[account(init,
    mint::decimals = 0,
    mint::authority = authority,
    payer = authority)]
    pub nft_mint: Account<'info, Mint>,
    #[account(init,
    associated_token::mint = nft_mint,
    associated_token::authority = authority,
    payer = authority)]
    pub nft_token: Account<'info, TokenAccount>,
    #[account(init,
    seeds = [b"metadata".as_ref(), nft_mint.key().as_ref()],
    bump,
    payer = authority,
    space = 8 + size_of::<Metadata>()
    )]
    pub nft_metadata: Account<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    #[account(mut)]
    pub nft_token: Account<'info, TokenAccount>,
    #[account(mut,
    seeds = [b"metadata".as_ref(), nft_mint.key().as_ref()],
    bump=nft_metadata.self_bump,
    close = authority)]
    pub nft_metadata: Account<'info, Metadata>,
    #[account(mut, has_one = authority)]
    pub player: Account<'info, Player>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub enum ItemType {
    Fire,
    Water,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub struct Item {
    pub item_type: ItemType,
    pub strength: u8,
}

#[account]
pub struct Player {
    pub bump: u8,
    pub authority: Pubkey,
    pub items: Vec<Item>,
}

#[account]
pub struct Metadata {
    pub self_bump: u8,
    pub mint_bump: u8,
    pub mint: Pubkey,
    pub item: Item,
}

#[error]
pub enum ErrorCode {
    #[msg("You don't own this token")]
    InvalidTokenAmount,
}
