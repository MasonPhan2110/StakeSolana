use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod stake {
    use super::*;
    pub fn createPool(
        ctx: Context<CreatePool>,
        poolId: String,
        addr: [Pubkey; 2],
        data: [u128; 1],
        configs: [u128; 5],
    ) -> Result<()> {
        ctx.accounts.pool.poolId = poolId;
        ctx.accounts.pool.stakingToken = addr[0];
        ctx.accounts.pool.rewardToken = addr[1];
        ctx.accounts.pool.stakedBalance = 0;
        ctx.accounts.pool.totalRewardClaimed = 0;
        ctx.accounts.pool.rewardFund = data[0];
        ctx.accounts.pool.initialFund = data[0];
        ctx.accounts.pool.lastUpdateTime = 0;
        ctx.accounts.pool.rewardPerTokenStore = 0;
        ctx.accounts.pool.totalUserStake = 0;
        ctx.accounts.pool.active = true;
        ctx.accounts.pool.configs = configs;
        ctx.accounts.pool.bump = *ctx.bumps.get("pool").unwrap();
        Ok(())
    }
}

impl Pool {
    pub fn StakeToken() {}
}

#[account]
pub struct Pool {
    poolId: String,
    stakingToken: Pubkey,
    rewardToken: Pubkey,
    stakedBalance: u128,
    totalRewardClaimed: u128,
    rewardFund: u128,
    initialFund: u128,
    lastUpdateTime: u128,
    rewardPerTokenStore: u128,
    totalUserStake: u128,
    active: bool,
    configs: [u128; 5],
    bump: u8,
}

#[derive(Accounts)]
#[instruction(poolId: String)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8+32*2+12*16+1+1+4+200, seeds = [b"pool", user.key().as_ref(),poolId.as_ref()], bump
    )]
    pub pool: Account<'info, Pool>,
    pub system_program: Program<'info, System>,
}
