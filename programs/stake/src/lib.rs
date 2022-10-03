
use std::time::{SystemTime, UNIX_EPOCH};

use anchor_lang::{__private::bytemuck::__core::time, prelude::*};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod stake {
    use super::*;
    pub fn stake(ctx: Context<Stake>, amount: u128) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        ctx.accounts.stakedata.bump = *ctx.bumps.get("stakedata").unwrap();
        pool.StakeToken(amount, &mut ctx.accounts.stakedata)
    }
    pub fn createPool(
        ctx: Context<CreatePool>,
        pool_id: String,
        addr: [Pubkey; 2],
        data: [u128; 1],
        configs: [u128; 5],
    ) -> Result<()> {
        ctx.accounts.pool.pool_id = pool_id;
        ctx.accounts.pool.staking_token = addr[0];
        ctx.accounts.pool.reward_token = addr[1];
        ctx.accounts.pool.staked_balance = 0;
        ctx.accounts.pool.total_reward_claimed = 0;
        ctx.accounts.pool.reward_fund = data[0];
        ctx.accounts.pool.initial_fund = data[0];
        ctx.accounts.pool.last_update_time = 0;
        ctx.accounts.pool.reward_per_token_store = 0;
        ctx.accounts.pool.total_user_stake = 0;
        ctx.accounts.pool.active = true;
        ctx.accounts.pool.configs = configs;
        ctx.accounts.pool.bump = *ctx.bumps.get("pool").unwrap();
        Ok(())
    }
}

impl Pool {
    pub const ONE_DAY_IN_SECOND: u128 = 86400;
    pub fn StakeToken(&mut self, amount: u128, stakeData: &mut StakingData) -> Result<()> {
        self.staked_balance += amount;
        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        require_gte!(time_now, self.configs[0], StakeError::TimeInvalid);
        require_gte!(self.configs[1], time_now, StakeError::TimeInvalid);
        require!(amount > 0, StakeError::AmountInvalid);

        // update reward
        self.reward_per_token_store = self.rewardPerToken().unwrap();
        self.last_update_time = time_now;
        stakeData.reward = self.earned(stakeData).unwrap();
        stakeData.reward_per_token_paid = self.reward_per_token_store;

        if stakeData.balance == 0 {
            self.total_user_stake += 1;
        }

        // Update staked balance
        stakeData.balance += amount;

        // Update staking time
        stakeData.staked_time = time_now;

        self.staked_balance += amount;

        Ok(())
    }

    pub fn earned(&mut self, stakeData: &StakingData) -> Result<u128> {
        if (stakeData.balance == 0) {
            return Ok(0);
        } else {
            let amount = stakeData.balance
                * (self.rewardPerToken().unwrap() - stakeData.reward_per_token_paid)
                / 10u128.pow(8)
                + stakeData.reward;
            Ok(amount)
        }
    }
    fn rewardPerToken(&mut self) -> Result<u128> {
        let pool_duration = self.configs[1] - self.configs[0];
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        if self.staked_balance == 0 || pool_duration == 0 {
            Ok(0)
        } else if (current_timestamp < self.last_update_time) {
            Ok(self.reward_per_token_store)
        } else {
            let reward_pool = self.reward_fund * (current_timestamp - self.last_update_time);
            let reward_per_token = reward_pool / (pool_duration * self.staked_balance)
                * 10u128.pow(8)
                + self.reward_per_token_store;
            Ok(reward_per_token)
        }
    }
    pub fn canGetReward(&mut self, stakeData: &StakingData) -> Result<bool> {
        if (self.configs[2] == 0) {
            Ok(true)
        } else {
            let time_now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            return Ok(
                stakeData.staked_time + self.configs[2] * Pool::ONE_DAY_IN_SECOND <= time_now,
            );
        }
    }
}

#[account]
pub struct Pool {
    pool_id: String,
    staking_token: Pubkey,
    reward_token: Pubkey,
    staked_balance: u128,
    total_reward_claimed: u128,
    reward_fund: u128,
    initial_fund: u128,
    last_update_time: u128,
    reward_per_token_store: u128,
    total_user_stake: u128,
    active: bool,
    configs: [u128; 5],
    bump: u8,
}

#[account]
pub struct StakingData {
    balance: u128,
    staked_time: u128,
    unstaked_time: u128,
    reward: u128,
    reward_per_token_paid: u128,
    account: Pubkey,
    bump: u8,
}

#[derive(Accounts)]
#[instruction(pool_id: String, internalId: String)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8+32*2+12*16+1+1+4+200, seeds = [b"pool",system_program.key().as_ref() ,pool_id.as_ref(), internalId.as_ref()], bump
    )]
    pub pool: Account<'info, Pool>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(pool_id: String)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user, 
        space = 8 + 16*5 + 32 + 1, seeds= [b"stakedata", user.key().as_ref(),pool_id.as_ref()], bump
    )]
    pub stakedata : Account<'info, StakingData>,

    #[account(mut, seeds = [b"pool", system_program.key().as_ref() ,pool_id.as_ref()], bump = pool.bump)]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
}
#[error_code]
pub enum StakeError {
    TimeInvalid,
    AmountInvalid,
    RewardIs0,
    NotEnounghTime,
}
