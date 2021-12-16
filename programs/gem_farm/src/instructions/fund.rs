use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::rewards::update_accrued_rewards;
use gem_common::*;

use crate::state::*;

#[derive(Accounts)]
#[instruction(bump_proof: u8, bump_fr: u8, bump_pot: u8)]
pub struct Fund<'info> {
    // farm
    #[account(mut)]
    pub farm: Account<'info, Farm>,
    pub farm_authority: AccountInfo<'info>,

    // funder
    #[account(has_one = farm, has_one = authorized_funder ,seeds = [
            b"authorization".as_ref(),
            farm.key().as_ref(),
            authorized_funder.key().as_ref(),
        ],
        bump = bump_proof)]
    pub authorization_proof: Account<'info, AuthorizationProof>,
    #[account(mut)]
    pub authorized_funder: Signer<'info>,
    #[account(init_if_needed, seeds = [
            b"funding_receipt".as_ref(),
            authorized_funder.key().as_ref(),
            reward_mint.key().as_ref(),
        ],
        bump = bump_fr,
        payer = authorized_funder,
        space = 8 + std::mem::size_of::<FundingReceipt>())]
    pub funding_receipt: Box<Account<'info, FundingReceipt>>,

    // reward
    #[account(mut, seeds = [
            b"reward_pot".as_ref(),
            farm.key().as_ref(),
            reward_mint.key().as_ref(),
        ],
        bump = bump_pot)]
    pub reward_pot: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub reward_source: Box<Account<'info, TokenAccount>>,
    pub reward_mint: Box<Account<'info, Mint>>,

    // misc
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Fund<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reward_source.to_account_info(),
                to: self.reward_pot.to_account_info(),
                authority: self.authorized_funder.to_account_info(),
            },
        )
    }
}

pub fn handler(ctx: Context<Fund>, amount: u64, duration_sec: u64) -> ProgramResult {
    let now_ts = now_ts()?;

    // update existing rewards + post new ones
    let farm = &mut ctx.accounts.farm;

    update_accrued_rewards(farm, None)?;

    farm.fund_reward_by_mint(now_ts, amount, duration_sec, ctx.accounts.reward_mint.key())?;

    // create/update fr
    let fr = &mut ctx.accounts.funding_receipt;
    fr.funder = ctx.accounts.authorized_funder.key();
    fr.reward_mint = ctx.accounts.reward_mint.key();
    fr.total_deposited_amount.try_self_add(amount);
    fr.deposit_count.try_self_add(1);
    fr.last_deposit_ts = now_ts;

    // do the transfer
    token::transfer(
        ctx.accounts
            .transfer_ctx()
            .with_signer(&[&ctx.accounts.farm.farm_seeds()]),
        amount,
    )?;

    msg!(
        "{} reward tokens deposited into {} pot",
        amount,
        ctx.accounts.reward_pot.key()
    );
    Ok(())
}
