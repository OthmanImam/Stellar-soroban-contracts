#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

// Import types from shared module
use shared::{RewardConfig, StakeInfo, StakingPosition, StakingStats, VoteDelegation};

#[contract]
pub struct GovernanceStakingContract;

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const GOV_TOKEN: Symbol = symbol_short!("GOV_TKN");
const REWARD_CONFIG: Symbol = symbol_short!("RWD_CONF");
const STAKING_STATS: Symbol = symbol_short!("STK_STAT");
const ACC_REWARD_PER_SHARE: Symbol = symbol_short!("ACC_RPS");
const LAST_REWARD_TIME: Symbol = symbol_short!("LAST_RWD");

// User-specific storage prefix
const STAKE_INFO: Symbol = symbol_short!("STAKE");
const DELEGATION: Symbol = symbol_short!("DELG");

// Basis points constant
const BPS_DENOMINATOR: u128 = 10000;
const REWARD_PRECISION: u128 = 1_000_000_000_000; // 1e12 for precision

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    Unauthorized = 1,
    Paused = 2,
    InvalidInput = 3,
    InsufficientBalance = 4,
    NotFound = 5,
    AlreadyExists = 6,
    InvalidState = 7,
    NotInitialized = 8,
    AlreadyInitialized = 9,
    StakeLocked = 10,
    NoRewardsToClaim = 11,
    CooldownNotComplete = 12,
    AlreadyDelegated = 13,
    SelfDelegation = 14,
    RewardsDisabled = 15,
    InsufficientStake = 16,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakingConfig {
    pub governance_token: Address,
    pub reward_token: Address,
    pub min_stake_amount: i128,
    pub min_stake_period: u64,
    pub unstake_cooldown: u64,
    pub rewards_enabled: bool,
}

fn get_stake_info(env: &Env, staker: &Address) -> Option<StakeInfo> {
    env.storage().persistent().get(&(STAKE_INFO, staker.clone()))
}

fn set_stake_info(env: &Env, staker: &Address, info: &StakeInfo) {
    env.storage().persistent().set(&(STAKE_INFO, staker.clone()), info);
}

fn get_staking_position(env: &Env, user: &Address) -> Option<StakingPosition> {
    env.storage().persistent().get(&(Symbol::new(env, "POSITION"), user.clone()))
}

fn set_staking_position(env: &Env, user: &Address, position: &StakingPosition) {
    env.storage()
        .persistent()
        .set(&(Symbol::new(env, "POSITION"), user.clone()), position);
}

fn get_reward_config(env: &Env) -> Option<RewardConfig> {
    env.storage().persistent().get(&REWARD_CONFIG)
}

fn set_reward_config(env: &Env, config: &RewardConfig) {
    env.storage().persistent().set(&REWARD_CONFIG, config);
}

fn get_staking_stats(env: &Env) -> StakingStats {
    env.storage().persistent().get(&STAKING_STATS).unwrap_or(StakingStats {
        total_stakers: 0,
        total_staked: 0,
        total_rewards_distributed: 0,
        avg_stake_duration: 0,
        last_update: env.ledger().timestamp(),
    })
}

fn set_staking_stats(env: &Env, stats: &StakingStats) {
    env.storage().persistent().set(&STAKING_STATS, stats);
}

fn get_acc_reward_per_share(env: &Env) -> u128 {
    env.storage()
        .persistent()
        .get(&ACC_REWARD_PER_SHARE)
        .unwrap_or(0u128)
}

fn set_acc_reward_per_share(env: &Env, value: u128) {
    env.storage().persistent().set(&ACC_REWARD_PER_SHARE, &value);
}

fn get_last_reward_time(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&LAST_REWARD_TIME)
        .unwrap_or(env.ledger().timestamp())
}

fn set_last_reward_time(env: &Env, time: u64) {
    env.storage().persistent().set(&LAST_REWARD_TIME, &time);
}

fn is_paused(env: &Env) -> bool {
    env.storage().persistent().get(&PAUSED).unwrap_or(false)
}

fn set_paused(env: &Env, paused: bool) {
    env.storage().persistent().set(&PAUSED, &paused);
}

fn calculate_voting_power_multiplier(stake_duration: u64) -> u32 {
    // Base multiplier is 100 (1x)
    // Add 10% bonus per year staked, max 200% (3x total)
    let years_staked = stake_duration / (365 * 24 * 60 * 60);
    let bonus = (years_staked * 10).min(200);
    100 + bonus as u32
}

fn update_pool_rewards(env: &Env) {
    let config = match get_reward_config(env) {
        Some(c) => c,
        None => return,
    };

    if !config.rewards_enabled || config.remaining_rewards <= 0 {
        return;
    }

    let current_time = env.ledger().timestamp();
    let last_time = get_last_reward_time(env);
    
    if current_time <= last_time {
        return;
    }

    let stats = get_staking_stats(env);
    if stats.total_staked <= 0 {
        set_last_reward_time(env, current_time);
        return;
    }

    let time_elapsed = (current_time - last_time) as u128;
    let reward_rate = config.base_reward_rate_bps as u128;
    
    // Calculate rewards: total_staked * rate * time / (BPS * seconds_per_year)
    let seconds_per_year: u128 = 365 * 24 * 60 * 60;
    let total_reward = (stats.total_staked as u128)
        .checked_mul(reward_rate)
        .and_then(|v| v.checked_mul(time_elapsed))
        .and_then(|v| v.checked_div(BPS_DENOMINATOR))
        .and_then(|v| v.checked_div(seconds_per_year))
        .unwrap_or(0);

    let actual_reward = total_reward.min(config.remaining_rewards as u128);

    if actual_reward > 0 {
        let acc_reward = get_acc_reward_per_share(env);
        let new_acc_reward = acc_reward
            + (actual_reward * REWARD_PRECISION) / (stats.total_staked as u128);
        set_acc_reward_per_share(env, new_acc_reward);

        // Update remaining rewards
        let mut new_config = config;
        new_config.remaining_rewards -= actual_reward as i128;
        new_config.last_update = current_time;
        set_reward_config(env, &new_config);
    }

    set_last_reward_time(env, current_time);
}

fn pending_rewards(env: &Env, user: &Address) -> i128 {
    let position = match get_staking_position(env, user) {
        Some(p) => p,
        None => return 0,
    };

    let acc_reward = get_acc_reward_per_share(env);
    let pending = ((position.staked_amount as u128)
        .checked_mul(acc_reward)
        .unwrap_or(0)
        / REWARD_PRECISION) as i128;

    pending.saturating_sub(position.reward_debt)
}

#[contractimpl]
impl GovernanceStakingContract {
    /// Initialize the staking contract
    pub fn initialize(
        env: Env,
        admin: Address,
        governance_token: Address,
        reward_token: Address,
        base_reward_rate_bps: u32,
        min_stake_amount: i128,
        min_stake_period: u64,
        unstake_cooldown: u64,
        initial_reward_pool: i128,
    ) -> Result<(), ContractError> {
        // Check if already initialized
        if env.storage().persistent().has(&ADMIN) {
            return Err(ContractError::AlreadyInitialized);
        }

        admin.require_auth();

        // Validate inputs
        if base_reward_rate_bps > 10000 {
            return Err(ContractError::InvalidInput);
        }
        if min_stake_amount <= 0 {
            return Err(ContractError::InvalidInput);
        }
        if initial_reward_pool < 0 {
            return Err(ContractError::InvalidInput);
        }

        // Store admin
        env.storage().persistent().set(&ADMIN, &admin);

        // Store governance token
        env.storage().persistent().set(&GOV_TOKEN, &governance_token);

        // Initialize reward config
        let reward_config = RewardConfig {
            reward_token: reward_token.clone(),
            base_reward_rate_bps,
            loyalty_bonus_bps: 100, // 1% bonus per year
            min_stake_period,
            unstake_cooldown,
            rewards_enabled: initial_reward_pool > 0,
            total_reward_pool: initial_reward_pool,
            remaining_rewards: initial_reward_pool,
            last_update: env.ledger().timestamp(),
        };
        set_reward_config(&env, &reward_config);

        // Initialize staking stats
        let stats = StakingStats {
            total_stakers: 0,
            total_staked: 0,
            total_rewards_distributed: 0,
            avg_stake_duration: 0,
            last_update: env.ledger().timestamp(),
        };
        set_staking_stats(&env, &stats);

        // Initialize reward tracking
        set_acc_reward_per_share(&env, 0);
        set_last_reward_time(&env, env.ledger().timestamp());

        // Emit initialization event
        env.events().publish(
            (symbol_short!("init"), ())
            , (admin, governance_token, reward_token),
        );

        Ok(())
    }

    /// Stake governance tokens
    pub fn stake(env: Env, user: Address, amount: i128) -> Result<(), ContractError> {
        user.require_auth();

        if is_paused(&env) {
            return Err(ContractError::Paused);
        }

        if amount <= 0 {
            return Err(ContractError::InvalidInput);
        }

        let config = get_reward_config(&env).ok_or(ContractError::NotInitialized)?;
        
        if amount < config.min_stake_amount {
            return Err(ContractError::InvalidInput);
        }

        // Update pool rewards before user action
        update_pool_rewards(&env);

        let current_time = env.ledger().timestamp();
        let mut stats = get_staking_stats(&env);

        // Get or create staking position
        let mut position = get_staking_position(&env, &user).unwrap_or(StakingPosition {
            user: user.clone(),
            staked_amount: 0,
            reward_debt: 0,
            stake_start_time: current_time,
            lock_end_time: 0,
            pending_rewards: 0,
        });

        // Calculate pending rewards before updating stake
        let pending = pending_rewards(&env, &user);
        position.pending_rewards += pending;

        // Update position
        position.staked_amount += amount;
        position.reward_debt = ((position.staked_amount as u128)
            .checked_mul(get_acc_reward_per_share(&env))
            .unwrap_or(0)
            / REWARD_PRECISION) as i128;

        set_staking_position(&env, &user, &position);

        // Update stake info for backward compatibility
        let stake_duration = if position.staked_amount > amount {
            current_time - position.stake_start_time
        } else {
            0
        };

        let stake_info = StakeInfo {
            staker: user.clone(),
            amount: position.staked_amount,
            staked_at: position.stake_start_time,
            last_claim_at: current_time,
            pending_rewards: position.pending_rewards,
            is_locked: false,
            unlock_at: None,
            voting_power_multiplier: calculate_voting_power_multiplier(stake_duration),
        };
        set_stake_info(&env, &user, &stake_info);

        // Update stats
        if position.staked_amount == amount {
            // New staker
            stats.total_stakers += 1;
        }
        stats.total_staked += amount;
        stats.last_update = current_time;
        set_staking_stats(&env, &stats);

        // Transfer tokens from user to contract
        let gov_token: Address = env.storage().persistent().get(&GOV_TOKEN).unwrap();
        let token_client = soroban_sdk::token::Client::new(&env, &gov_token);
        token_client.transfer(&user, &env.current_contract_address(), &amount);

        // Emit stake event
        env.events().publish(
            (symbol_short!("stake"), user.clone()),
            (amount, position.staked_amount),
        );

        Ok(())
    }

    /// Initiate unstaking (starts cooldown period)
    pub fn initiate_unstake(env: Env, user: Address, amount: i128) -> Result<u64, ContractError> {
        user.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidInput);
        }

        let mut position = get_staking_position(&env, &user).ok_or(ContractError::NotFound)?;

        if position.staked_amount < amount {
            return Err(ContractError::InsufficientBalance);
        }

        let config = get_reward_config(&env).ok_or(ContractError::NotInitialized)?;
        
        // Check minimum stake period
        let current_time = env.ledger().timestamp();
        let stake_duration = current_time - position.stake_start_time;
        if stake_duration < config.min_stake_period {
            return Err(ContractError::StakeLocked);
        }

        // Update pool rewards
        update_pool_rewards(&env);

        // Calculate any pending rewards
        let pending = pending_rewards(&env, &user);
        position.pending_rewards += pending;

        // Set unlock time
        let unlock_at = current_time + config.unstake_cooldown;
        position.lock_end_time = unlock_at;

        set_staking_position(&env, &user, &position);

        // Update stake info
        if let Some(mut stake_info) = get_stake_info(&env, &user) {
            stake_info.is_locked = true;
            stake_info.unlock_at = Some(unlock_at);
            set_stake_info(&env, &user, &stake_info);
        }

        // Emit unstake initiated event
        env.events().publish(
            (symbol_short!("unstake_start"), user.clone()),
            (amount, unlock_at),
        );

        Ok(unlock_at)
    }

    /// Complete unstaking after cooldown
    pub fn complete_unstake(env: Env, user: Address) -> Result<i128, ContractError> {
        user.require_auth();

        let mut position = get_staking_position(&env, &user).ok_or(ContractError::NotFound)?;

        let current_time = env.ledger().timestamp();
        
        if position.lock_end_time == 0 || current_time < position.lock_end_time {
            return Err(ContractError::CooldownNotComplete);
        }

        // Update pool rewards
        update_pool_rewards(&env);

        // Calculate final pending rewards
        let pending = pending_rewards(&env, &user);
        position.pending_rewards += pending;

        let unstake_amount = position.staked_amount;
        let rewards_to_claim = position.pending_rewards;

        // Update stats
        let mut stats = get_staking_stats(&env);
        stats.total_stakers -= 1;
        stats.total_staked -= unstake_amount;
        stats.last_update = current_time;
        set_staking_stats(&env, &stats);

        // Remove staking position
        env.storage().persistent().remove(&(Symbol::new(&env, "POSITION"), user.clone()));
        env.storage().persistent().remove(&(STAKE_INFO, user.clone()));

        // Transfer staked tokens back to user
        let gov_token: Address = env.storage().persistent().get(&GOV_TOKEN).unwrap();
        let token_client = soroban_sdk::token::Client::new(&env, &gov_token);
        token_client.transfer(&env.current_contract_address(), &user, &unstake_amount);

        // Transfer any pending rewards
        if rewards_to_claim > 0 {
            let reward_config = get_reward_config(&env).unwrap();
            let reward_client = soroban_sdk::token::Client::new(&env, &reward_config.reward_token);
            reward_client.transfer(&env.current_contract_address(), &user, &rewards_to_claim);
            
            stats.total_rewards_distributed += rewards_to_claim;
            set_staking_stats(&env, &stats);
        }

        // Emit unstake completed event
        env.events().publish(
            (symbol_short!("unstake_end"), user.clone()),
            (unstake_amount, rewards_to_claim),
        );

        Ok(unstake_amount)
    }

    /// Claim rewards without unstaking
    pub fn claim_rewards(env: Env, user: Address) -> Result<i128, ContractError> {
        user.require_auth();

        let config = get_reward_config(&env).ok_or(ContractError::NotInitialized)?;
        
        if !config.rewards_enabled {
            return Err(ContractError::RewardsDisabled);
        }

        // Update pool rewards
        update_pool_rewards(&env);

        let mut position = get_staking_position(&env, &user).ok_or(ContractError::NotFound)?;

        // Calculate pending rewards
        let pending = pending_rewards(&env, &user);
        let total_rewards = position.pending_rewards + pending;

        if total_rewards <= 0 {
            return Err(ContractError::NoRewardsToClaim);
        }

        // Reset pending rewards and update debt
        position.pending_rewards = 0;
        position.reward_debt = ((position.staked_amount as u128)
            .checked_mul(get_acc_reward_per_share(&env))
            .unwrap_or(0)
            / REWARD_PRECISION) as i128;

        set_staking_position(&env, &user, &position);

        // Update stake info
        if let Some(mut stake_info) = get_stake_info(&env, &user) {
            stake_info.pending_rewards = 0;
            stake_info.last_claim_at = env.ledger().timestamp();
            set_stake_info(&env, &user, &stake_info);
        }

        // Transfer rewards
        let reward_client = soroban_sdk::token::Client::new(&env, &config.reward_token);
        reward_client.transfer(&env.current_contract_address(), &user, &total_rewards);

        // Update stats
        let mut stats = get_staking_stats(&env);
        stats.total_rewards_distributed += total_rewards;
        set_staking_stats(&env, &stats);

        // Emit claim event
        env.events().publish(
            (symbol_short!("claim"), user.clone()),
            total_rewards,
        );

        Ok(total_rewards)
    }

    /// Delegate voting power to another address
    pub fn delegate(
        env: Env,
        delegator: Address,
        delegatee: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        delegator.require_auth();

        if delegator == delegatee {
            return Err(ContractError::SelfDelegation);
        }

        if amount <= 0 {
            return Err(ContractError::InvalidInput);
        }

        let position = get_staking_position(&env, &delegator).ok_or(ContractError::NotFound)?;

        if position.staked_amount < amount {
            return Err(ContractError::InsufficientStake);
        }

        // Check if already delegated
        if env
            .storage()
            .persistent()
            .has(&(DELEGATION, delegator.clone()))
        {
            return Err(ContractError::AlreadyDelegated);
        }

        let delegation = VoteDelegation {
            delegator: delegator.clone(),
            delegatee: delegatee.clone(),
            amount,
            delegated_at: env.ledger().timestamp(),
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&(DELEGATION, delegator.clone()), &delegation);

        // Emit delegation event
        env.events().publish(
            (symbol_short!("delegate"), delegator.clone()),
            (delegatee, amount),
        );

        Ok(())
    }

    /// Remove delegation
    pub fn undelegate(env: Env, delegator: Address) -> Result<(), ContractError> {
        delegator.require_auth();

        if !env
            .storage()
            .persistent()
            .has(&(DELEGATION, delegator.clone()))
        {
            return Err(ContractError::NotFound);
        }

        env.storage()
            .persistent()
            .remove(&(DELEGATION, delegator.clone()));

        // Emit undelegation event
        env.events().publish(
            (symbol_short!("undelegate"), delegator.clone()),
            (),
        );

        Ok(())
    }

    /// Add rewards to the pool (admin only)
    pub fn add_rewards(
        env: Env,
        admin: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        admin.require_auth();

        let stored_admin: Address = env.storage().persistent().get(&ADMIN).ok_or(ContractError::NotInitialized)?;
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }

        if amount <= 0 {
            return Err(ContractError::InvalidInput);
        }

        let mut config = get_reward_config(&env).ok_or(ContractError::NotInitialized)?;
        
        // Transfer reward tokens to contract
        let reward_client = soroban_sdk::token::Client::new(&env, &config.reward_token);
        reward_client.transfer(&admin, &env.current_contract_address(), &amount);

        // Update reward pool
        config.total_reward_pool += amount;
        config.remaining_rewards += amount;
        config.rewards_enabled = true;
        set_reward_config(&env, &config);

        // Emit event
        env.events().publish(
            (symbol_short!("add_rwd"), admin),
            amount,
        );

        Ok(())
    }

    /// Update reward rate (admin only)
    pub fn set_reward_rate(
        env: Env,
        admin: Address,
        new_rate_bps: u32,
    ) -> Result<(), ContractError> {
        admin.require_auth();

        let stored_admin: Address = env.storage().persistent().get(&ADMIN).ok_or(ContractError::NotInitialized)?;
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }

        if new_rate_bps > 10000 {
            return Err(ContractError::InvalidInput);
        }

        // Update pool rewards before changing rate
        update_pool_rewards(&env);

        let mut config = get_reward_config(&env).ok_or(ContractError::NotInitialized)?;
        config.base_reward_rate_bps = new_rate_bps;
        set_reward_config(&env, &config);

        // Emit event
        env.events().publish(
            (symbol_short!("set_rate"), admin),
            new_rate_bps,
        );

        Ok(())
    }

    /// Pause/unpause staking (admin only)
    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), ContractError> {
        admin.require_auth();

        let stored_admin: Address = env.storage().persistent().get(&ADMIN).ok_or(ContractError::NotInitialized)?;
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }

        set_paused(&env, paused);

        env.events().publish(
            (symbol_short!("paused"), admin),
            paused,
        );

        Ok(())
    }

    // ===== View Functions =====

    /// Get staking position for a user
    pub fn get_position(env: Env, user: Address) -> Option<StakingPosition> {
        get_staking_position(&env, &user)
    }

    /// Get stake info for a user
    pub fn get_stake_info(env: Env, user: Address) -> Option<StakeInfo> {
        get_stake_info(&env, &user)
    }

    /// Get pending rewards for a user
    pub fn get_pending_rewards(env: Env, user: Address) -> i128 {
        update_pool_rewards(&env);
        pending_rewards(&env, &user)
    }

    /// Get total voting power for a user (including multiplier)
    pub fn get_voting_power(env: Env, user: Address) -> i128 {
        let position = match get_staking_position(&env, &user) {
            Some(p) => p,
            None => return 0,
        };

        let current_time = env.ledger().timestamp();
        let stake_duration = current_time - position.stake_start_time;
        let multiplier = calculate_voting_power_multiplier(stake_duration);

        // Apply multiplier to staked amount
        ((position.staked_amount as u128)
            .checked_mul(multiplier as u128)
            .unwrap_or(0)
            / 100) as i128
    }

    /// Get staking statistics
    pub fn get_stats(env: Env) -> StakingStats {
        get_staking_stats(&env)
    }

    /// Get reward configuration
    pub fn get_reward_config_view(env: Env) -> Option<RewardConfig> {
        get_reward_config(&env)
    }

    /// Get delegation info
    pub fn get_delegation(env: Env, delegator: Address) -> Option<VoteDelegation> {
        env.storage().persistent().get(&(DELEGATION, delegator))
    }

    /// Check if user can unstake
    pub fn can_unstake(env: Env, user: Address) -> bool {
        let position = match get_staking_position(&env, &user) {
            Some(p) => p,
            None => return false,
        };

        if position.lock_end_time > 0 {
            return env.ledger().timestamp() >= position.lock_end_time;
        }

        let config = match get_reward_config(&env) {
            Some(c) => c,
            None => return false,
        };

        let stake_duration = env.ledger().timestamp() - position.stake_start_time;
        stake_duration >= config.min_stake_period
    }

    /// Get list of all stakers (for governance integration)
    pub fn get_all_stakers(env: Env) -> Vec<Address> {
        // This is a placeholder - in production, you'd maintain an index
        // For now, return empty vector
        Vec::new(&env)
    }
}
