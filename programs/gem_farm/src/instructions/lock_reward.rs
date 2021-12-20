use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use gem_common::now_ts;

use crate::state::Farm;

#[derive(Accounts)]
pub struct LockReward<'info> {
    // farm
    #[account(mut, has_one = farm_manager)]
    pub farm: Box<Account<'info, Farm>>,
    #[account(mut)]
    pub farm_manager: Signer<'info>,

    // reward
    pub reward_mint: Box<Account<'info, Mint>>,
}

pub fn handler(ctx: Context<LockReward>) -> ProgramResult {
    let farm = &mut ctx.accounts.farm;
    let now_ts = now_ts()?;

    farm.lock_reward_by_mint(now_ts, ctx.accounts.reward_mint.key())?;

    Ok(())
}