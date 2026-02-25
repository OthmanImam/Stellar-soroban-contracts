#![no_std]

mod types;
mod storage;
mod errors;
mod calculations;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec, token, symbol_short};
use types::*;
use errors::Error;

#[contract]
pub struct RewardDistribution;

#[contractimpl]
impl RewardDistribution {
    /// Initialize the reward distribution contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&symbol_short!("ADMIN")) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&symbol_short!("ADMIN"), &admin);
        env.storage().instance().set(&symbol_short!("POOL_CNT"), &0u32);
        env.storage().instance().set(&symbol_short!("PAUSED"), &false);

        // Initialize emission config with defaults
        let emission_config = EmissionConfig {
            max_emission_rate: 1_000_000_000,  // 1000 tokens per second max
            inflation_cap: 1000,                // 10% per year
            adjustment_interval: 86400,         // Daily adjustments
            last_adjustment: env.ledger().timestamp(),
        };
        env.storage().instance().set(&symbol_short!("EMISSION"), &emission_config);

        Ok(())
    }

    /// Create a new reward pool
    pub fn create_pool(
        env: Env,
        admin: Address,
        name: String,
        base_apy: u32,
        risk_adjustment_factor: u32,
        min_stake: i128,
        lock_period: u64,
    ) -> Result<u32, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::require_not_paused(&env)?;

        if base_apy > 100_000 {  // Max 1000% APY
            return Err(Error::InvalidAPY);
        }

        if risk_adjustment_factor > 10_000 {
            return Err(Error::InvalidRiskAdjustment);
        }

        let pool_count: u32 = env.storage().instance().get(&symbol_short!("POOL_CNT")).unwrap_or(0);
        let pool_id = pool_count + 1;

        let pool = RewardPool {
            pool_id,
            name,
            total_staked: 0,
            reward_tokens: Vec::new(&env),
            base_apy,
            risk_adjustment_factor,
            status: RewardStatus::Active,
            min_stake,
            lock_period,
        };

        storage::set_pool(&env, &pool);
        env.storage().instance().set(&symbol_short!("POOL_CNT"), &pool_id);

        env.events().publish((symbol_short!("POOL_NEW"), pool_id), name);

        Ok(pool_id)
    }

    /// Add a reward token to a pool
    pub fn add_reward_token(
        env: Env,
        admin: Address,
        pool_id: u32,
        token_address: Address,
        emission_rate: i128,
        total_allocated: i128,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut pool = storage::get_pool(&env, pool_id).ok_or(Error::PoolNotFound)?;
        
        let emission_config: EmissionConfig = env.storage()
            .instance()
            .get(&symbol_short!("EMISSION"))
            .unwrap();

        if emission_rate > emission_config.max_emission_rate {
            return Err(Error::InvalidEmissionRate);
        }

        let reward_token = RewardToken {
            token_address: token_address.clone(),
            emission_rate,
            total_allocated,
            total_distributed: 0,
            active: true,
        };

        storage::set_reward_token(&env, pool_id, &reward_token);
        pool.reward_tokens.push_back(token_address.clone());
        storage::set_pool(&env, &pool);

        env.events().publish((symbol_short!("TOKEN_ADD"), pool_id), token_address);

        Ok(())
    }

    /// Stake tokens into a reward pool
    pub fn stake(
        env: Env,
        staker: Address,
        pool_id: u32,
        amount: i128,
    ) -> Result<(), Error> {
        staker.require_auth();
        Self::require_not_paused(&env)?;

        let mut pool = storage::get_pool(&env, pool_id).ok_or(Error::PoolNotFound)?;
        
        if pool.status != RewardStatus::Active {
            return Err(Error::PoolPaused);
        }
        
        if amount < pool.min_stake {
            return Err(Error::BelowMinimumStake);
        }

        let current_time = env.ledger().timestamp();
        
        // Get or create stake position
        let mut stake = storage::get_stake(&env, &staker, pool_id).unwrap_or(StakePosition {
            staker: staker.clone(),
            pool_id,
            amount: 0,
            stake_time: current_time,
            last_claim_time: current_time,
            performance_multiplier: 10_000, // Default 1x
        });

        stake.amount += amount;
        pool.total_staked += amount;

        storage::set_stake(&env, &stake);
        storage::set_pool(&env, &pool);

        env.events().publish((symbol_short!("STAKE"), pool_id), (staker, amount));

        Ok(())
    }

    /// Unstake tokens from a reward pool
    pub fn unstake(
        env: Env,
        staker: Address,
        pool_id: u32,
        amount: i128,
    ) -> Result<(), Error> {
        staker.require_auth();

        let mut stake = storage::get_stake(&env, &staker, pool_id)
            .ok_or(Error::StakeNotFound)?;
        let mut pool = storage::get_pool(&env, pool_id).ok_or(Error::PoolNotFound)?;

        if stake.amount < amount {
            return Err(Error::InsufficientStake);
        }

        let current_time = env.ledger().timestamp();
        let time_staked = current_time.saturating_sub(stake.stake_time);

        // Check lock period
        if time_staked < pool.lock_period {
            return Err(Error::LockPeriodNotMet);
        }

        stake.amount -= amount;
        pool.total_staked -= amount;

        if stake.amount == 0 {
            storage::remove_stake(&env, &staker, pool_id);
        } else {
            storage::set_stake(&env, &stake);
        }

        storage::set_pool(&env, &pool);

        env.events().publish((symbol_short!("UNSTAKE"), pool_id), (staker, amount));

        Ok(())
    }

    /// Emergency unstake with penalty
    pub fn emergency_unstake(
        env: Env,
        staker: Address,
        pool_id: u32,
    ) -> Result<i128, Error> {
        staker.require_auth();

        let stake = storage::get_stake(&env, &staker, pool_id)
            .ok_or(Error::StakeNotFound)?;
        let mut pool = storage::get_pool(&env, pool_id).ok_or(Error::PoolNotFound)?;

        let current_time = env.ledger().timestamp();
        let time_staked = current_time.saturating_sub(stake.stake_time);

        let penalty = calculations::calculate_early_withdrawal_penalty(
            stake.amount,
            pool.lock_period,
            time_staked,
        );

        let amount_returned = stake.amount - penalty;

        pool.total_staked -= stake.amount;
        storage::remove_stake(&env, &staker, pool_id);
        storage::set_pool(&env, &pool);

        env.events().publish(
            (symbol_short!("EMERG_OUT"), pool_id),
            (staker, amount_returned, penalty),
        );

        Ok(amount_returned)
    }

    /// Claim rewards for a stake position
    pub fn claim_rewards(
        env: Env,
        staker: Address,
        pool_id: u32,
        token: Address,
    ) -> Result<i128, Error> {
        staker.require_auth();

        let mut stake = storage::get_stake(&env, &staker, pool_id)
            .ok_or(Error::StakeNotFound)?;
        let pool = storage::get_pool(&env, pool_id).ok_or(Error::PoolNotFound)?;
        let mut reward_token = storage::get_reward_token(&env, pool_id, &token)
            .ok_or(Error::TokenNotRegistered)?;

        if !reward_token.active {
            return Err(Error::NoRewardsAvailable);
        }

        let current_time = env.ledger().timestamp();
        let time_since_last_claim = current_time.saturating_sub(stake.last_claim_time);

        // Calculate base rewards
        let base_rewards = calculations::calculate_base_rewards(
            &env,
            stake.amount,
            time_since_last_claim,
            pool.base_apy,
        );

        // Apply risk adjustment
        let risk_adjusted = calculations::apply_risk_adjustment(
            base_rewards,
            pool.risk_adjustment_factor,
        );

        // Apply performance multiplier
        let final_rewards = calculations::apply_performance_multiplier(
            risk_adjusted,
            stake.performance_multiplier,
        );

        if final_rewards == 0 {
            return Err(Error::NoRewardsAvailable);
        }

        // Check if enough rewards are available
        let available = reward_token.total_allocated - reward_token.total_distributed;
        if final_rewards > available {
            return Err(Error::InsufficientRewardBalance);
        }

        // Update state
        stake.last_claim_time = current_time;
        reward_token.total_distributed += final_rewards;

        storage::set_stake(&env, &stake);
        storage::set_reward_token(&env, pool_id, &reward_token);

        // Record claim
        let claim_record = ClaimRecord {
            claimer: staker.clone(),
            pool_id,
            token: token.clone(),
            amount: final_rewards,
            timestamp: current_time,
        };
        storage::add_claim_record(&env, &claim_record);

        // Transfer rewards
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &staker, &final_rewards);

        env.events().publish(
            (symbol_short!("CLAIM"), pool_id),
            (staker, token, final_rewards),
        );

        Ok(final_rewards)
    }

    /// Create a vesting schedule for rewards
    pub fn create_vesting_schedule(
        env: Env,
        admin: Address,
        beneficiary: Address,
        pool_id: u32,
        total_amount: i128,
        cliff_duration: u64,
        vesting_duration: u64,
        curve: VestingCurve,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if vesting_duration == 0 || cliff_duration > vesting_duration {
            return Err(Error::InvalidVestingSchedule);
        }

        let schedule = VestingSchedule {
            cliff_duration,
            vesting_duration,
            curve,
            start_time: env.ledger().timestamp(),
            total_amount,
            claimed_amount: 0,
        };

        storage::set_vesting(&env, &beneficiary, pool_id, &schedule);

        env.events().publish(
            (symbol_short!("VEST_NEW"), pool_id),
            (beneficiary, total_amount),
        );

        Ok(())
    }

    /// Claim vested rewards
    pub fn claim_vested(
        env: Env,
        beneficiary: Address,
        pool_id: u32,
        token: Address,
    ) -> Result<i128, Error> {
        beneficiary.require_auth();

        let mut schedule = storage::get_vesting(&env, &beneficiary, pool_id)
            .ok_or(Error::InvalidVestingSchedule)?;

        let claimable = calculations::calculate_vested_amount(&env, &schedule)?;

        if claimable == 0 {
            return Err(Error::VestingNotStarted);
        }

        schedule.claimed_amount += claimable;
        storage::set_vesting(&env, &beneficiary, pool_id, &schedule);

        // Transfer vested tokens
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &beneficiary, &claimable);

        env.events().publish(
            (symbol_short!("VEST_CLM"), pool_id),
            (beneficiary, claimable),
        );

        Ok(claimable)
    }

    /// Update performance metrics for a pool
    pub fn update_performance_metrics(
        env: Env,
        admin: Address,
        pool_id: u32,
        utilization_rate: u32,
        claim_ratio: u32,
        volatility_score: u32,
        counterparty_risk: u32,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let metrics = PerformanceMetrics {
            pool_id,
            utilization_rate,
            claim_ratio,
            volatility_score,
            counterparty_risk,
        };

        storage::set_metrics(&env, &metrics);

        // Calculate and update performance bonus for all stakers
        let bonus_multiplier = calculations::calculate_performance_bonus(&metrics);

        env.events().publish(
            (symbol_short!("PERF_UPD"), pool_id),
            bonus_multiplier,
        );

        Ok(())
    }

    /// Apply performance bonus to a staker
    pub fn apply_performance_bonus(
        env: Env,
        admin: Address,
        staker: Address,
        pool_id: u32,
    ) -> Result<u32, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let metrics = storage::get_metrics(&env, pool_id)
            .ok_or(Error::PoolNotFound)?;
        let mut stake = storage::get_stake(&env, &staker, pool_id)
            .ok_or(Error::StakeNotFound)?;

        let bonus_multiplier = calculations::calculate_performance_bonus(&metrics);
        stake.performance_multiplier = bonus_multiplier;

        storage::set_stake(&env, &stake);

        env.events().publish(
            (symbol_short!("BONUS_APP"), pool_id),
            (staker, bonus_multiplier),
        );

        Ok(bonus_multiplier)
    }

    /// Adjust emission rate based on inflation cap
    pub fn adjust_emission_rate(
        env: Env,
        admin: Address,
        pool_id: u32,
        token: Address,
        total_supply: i128,
    ) -> Result<i128, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let emission_config: EmissionConfig = env.storage()
            .instance()
            .get(&symbol_short!("EMISSION"))
            .unwrap();

        let current_time = env.ledger().timestamp();
        let time_elapsed = current_time.saturating_sub(emission_config.last_adjustment);

        if time_elapsed < emission_config.adjustment_interval {
            return Err(Error::InvalidEmissionRate);
        }

        let mut reward_token = storage::get_reward_token(&env, pool_id, &token)
            .ok_or(Error::TokenNotRegistered)?;

        let adjusted_rate = calculations::calculate_emission_adjustment(
            reward_token.emission_rate,
            total_supply,
            emission_config.inflation_cap,
            time_elapsed,
        );

        reward_token.emission_rate = adjusted_rate;
        storage::set_reward_token(&env, pool_id, &reward_token);

        // Update last adjustment time
        let mut new_config = emission_config;
        new_config.last_adjustment = current_time;
        env.storage().instance().set(&symbol_short!("EMISSION"), &new_config);

        env.events().publish(
            (symbol_short!("EMIT_ADJ"), pool_id),
            (token, adjusted_rate),
        );

        Ok(adjusted_rate)
    }

    /// Batch distribute rewards to multiple stakers
    pub fn batch_distribute(
        env: Env,
        admin: Address,
        pool_id: u32,
        token: Address,
        recipients: Vec<Address>,
        amounts: Vec<i128>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if recipients.len() != amounts.len() || recipients.len() > 100 {
            return Err(Error::BatchSizeTooLarge);
        }

        let token_client = token::Client::new(&env, &token);
        let contract_address = env.current_contract_address();

        for i in 0..recipients.len() {
            let recipient = recipients.get(i).unwrap();
            let amount = amounts.get(i).unwrap();

            token_client.transfer(&contract_address, &recipient, &amount);

            let claim_record = ClaimRecord {
                claimer: recipient.clone(),
                pool_id,
                token: token.clone(),
                amount,
                timestamp: env.ledger().timestamp(),
            };
            storage::add_claim_record(&env, &claim_record);
        }

        env.events().publish(
            (symbol_short!("BATCH_DST"), pool_id),
            recipients.len(),
        );

        Ok(())
    }

    /// Update pool status
    pub fn update_pool_status(
        env: Env,
        admin: Address,
        pool_id: u32,
        status: RewardStatus,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut pool = storage::get_pool(&env, pool_id).ok_or(Error::PoolNotFound)?;
        pool.status = status;
        storage::set_pool(&env, &pool);

        env.events().publish((symbol_short!("POOL_STS"), pool_id), status);

        Ok(())
    }

    /// Pause/unpause the contract
    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        env.storage().instance().set(&symbol_short!("PAUSED"), &paused);

        env.events().publish(symbol_short!("PAUSED"), paused);

        Ok(())
    }

    // View functions

    /// Get pool information
    pub fn get_pool(env: Env, pool_id: u32) -> Result<RewardPool, Error> {
        storage::get_pool(&env, pool_id).ok_or(Error::PoolNotFound)
    }

    /// Get stake position
    pub fn get_stake(env: Env, staker: Address, pool_id: u32) -> Result<StakePosition, Error> {
        storage::get_stake(&env, &staker, pool_id).ok_or(Error::StakeNotFound)
    }

    /// Get vesting schedule
    pub fn get_vesting(
        env: Env,
        beneficiary: Address,
        pool_id: u32,
    ) -> Result<VestingSchedule, Error> {
        storage::get_vesting(&env, &beneficiary, pool_id)
            .ok_or(Error::InvalidVestingSchedule)
    }

    /// Get claimable vested amount
    pub fn get_claimable_vested(
        env: Env,
        beneficiary: Address,
        pool_id: u32,
    ) -> Result<i128, Error> {
        let schedule = storage::get_vesting(&env, &beneficiary, pool_id)
            .ok_or(Error::InvalidVestingSchedule)?;

        calculations::calculate_vested_amount(&env, &schedule)
    }

    /// Get pending rewards
    pub fn get_pending_rewards(
        env: Env,
        staker: Address,
        pool_id: u32,
    ) -> Result<i128, Error> {
        let stake = storage::get_stake(&env, &staker, pool_id)
            .ok_or(Error::StakeNotFound)?;
        let pool = storage::get_pool(&env, pool_id).ok_or(Error::PoolNotFound)?;

        let current_time = env.ledger().timestamp();
        let time_since_last_claim = current_time.saturating_sub(stake.last_claim_time);

        let base_rewards = calculations::calculate_base_rewards(
            &env,
            stake.amount,
            time_since_last_claim,
            pool.base_apy,
        );

        let risk_adjusted = calculations::apply_risk_adjustment(
            base_rewards,
            pool.risk_adjustment_factor,
        );

        let final_rewards = calculations::apply_performance_multiplier(
            risk_adjusted,
            stake.performance_multiplier,
        );

        Ok(final_rewards)
    }

    /// Get performance metrics
    pub fn get_metrics(env: Env, pool_id: u32) -> Result<PerformanceMetrics, Error> {
        storage::get_metrics(&env, pool_id).ok_or(Error::PoolNotFound)
    }

    /// Get claim history
    pub fn get_claim_history(
        env: Env,
        claimer: Address,
        pool_id: u32,
    ) -> Vec<ClaimRecord> {
        storage::get_claim_history(&env, &claimer, pool_id)
    }

    /// Get risk-adjusted APY
    pub fn get_risk_adjusted_apy(env: Env, pool_id: u32) -> Result<u32, Error> {
        let pool = storage::get_pool(&env, pool_id).ok_or(Error::PoolNotFound)?;
        let metrics = storage::get_metrics(&env, pool_id).unwrap_or(PerformanceMetrics {
            pool_id,
            utilization_rate: 5_000,
            claim_ratio: 1_000,
            volatility_score: 3_000,
            counterparty_risk: 2_000,
        });

        let performance_multiplier = calculations::calculate_performance_bonus(&metrics);
        let adjusted_apy = calculations::calculate_risk_adjusted_yield(
            pool.base_apy,
            pool.risk_adjustment_factor,
            performance_multiplier,
        );

        Ok(adjusted_apy)
    }

    // Helper functions

    fn require_admin(env: &Env, address: &Address) -> Result<(), Error> {
        let admin: Address = env.storage()
            .instance()
            .get(&symbol_short!("ADMIN"))
            .ok_or(Error::NotInitialized)?;

        if admin != *address {
            return Err(Error::Unauthorized);
        }

        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        let paused: bool = env.storage()
            .instance()
            .get(&symbol_short!("PAUSED"))
            .unwrap_or(false);

        if paused {
            return Err(Error::ContractPaused);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let admin = Address::generate(&env);

        let result = RewardDistribution::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());

        // Test double initialization
        let result2 = RewardDistribution::initialize(env, admin);
        assert_eq!(result2, Err(Error::AlreadyInitialized));
    }

    #[test]
    fn test_create_pool() {
        let env = Env::default();
        let admin = Address::generate(&env);

        RewardDistribution::initialize(env.clone(), admin.clone()).unwrap();

        let pool_id = RewardDistribution::create_pool(
            env.clone(),
            admin,
            String::from_str(&env, "Test Pool"),
            1_000, // 10% APY
            8_000, // Risk factor
            100_0000000, // Min stake
            86400, // 1 day lock
        ).unwrap();

        assert_eq!(pool_id, 1);
    }

    #[test]
    fn test_stake_and_unstake() {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let staker = Address::generate(&env);

        RewardDistribution::initialize(env.clone(), admin.clone()).unwrap();

        let pool_id = RewardDistribution::create_pool(
            env.clone(),
            admin,
            String::from_str(&env, "Test Pool"),
            1_000,
            8_000,
            100_0000000,
            0, // No lock period for test
        ).unwrap();

        // Stake
        let stake_amount = 1000_0000000;
        RewardDistribution::stake(
            env.clone(),
            staker.clone(),
            pool_id,
            stake_amount,
        ).unwrap();

        // Verify stake
        let stake = RewardDistribution::get_stake(env.clone(), staker.clone(), pool_id).unwrap();
        assert_eq!(stake.amount, stake_amount);

        // Unstake
        RewardDistribution::unstake(
            env.clone(),
            staker.clone(),
            pool_id,
            stake_amount,
        ).unwrap();

        // Verify unstake
        let result = RewardDistribution::get_stake(env, staker, pool_id);
        assert_eq!(result, Err(Error::StakeNotFound));
    }

    #[test]
    fn test_vesting_schedule() {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);

        RewardDistribution::initialize(env.clone(), admin.clone()).unwrap();

        let pool_id = RewardDistribution::create_pool(
            env.clone(),
            admin.clone(),
            String::from_str(&env, "Test Pool"),
            1_000,
            8_000,
            100_0000000,
            0,
        ).unwrap();

        // Create vesting schedule
        RewardDistribution::create_vesting_schedule(
            env.clone(),
            admin,
            beneficiary.clone(),
            pool_id,
            1000_0000000,
            86400,  // 1 day cliff
            2592000, // 30 day vesting
            VestingCurve::Linear,
        ).unwrap();

        // Verify schedule
        let schedule = RewardDistribution::get_vesting(
            env.clone(),
            beneficiary.clone(),
            pool_id,
        ).unwrap();

        assert_eq!(schedule.total_amount, 1000_0000000);
        assert_eq!(schedule.cliff_duration, 86400);
    }
}
