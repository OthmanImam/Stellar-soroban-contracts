use soroban_sdk::{Address, Env, Vec};
use crate::types::*;

// Storage keys
pub const ADMIN: &str = "ADMIN";
pub const POOL_COUNT: &str = "POOL_COUNT";
pub const EMISSION_CONFIG: &str = "EMISSION_CFG";
pub const PAUSED: &str = "PAUSED";

// Pool storage
pub fn get_pool(env: &Env, pool_id: u32) -> Option<RewardPool> {
    let key = (pool_id,);
    env.storage().persistent().get(&key)
}

pub fn set_pool(env: &Env, pool: &RewardPool) {
    let key = (pool.pool_id,);
    env.storage().persistent().set(&key, pool);
}

// Stake position storage
pub fn get_stake(env: &Env, staker: &Address, pool_id: u32) -> Option<StakePosition> {
    let key = (staker, pool_id);
    env.storage().persistent().get(&key)
}

pub fn set_stake(env: &Env, stake: &StakePosition) {
    let key = (&stake.staker, stake.pool_id);
    env.storage().persistent().set(&key, stake);
}

pub fn remove_stake(env: &Env, staker: &Address, pool_id: u32) {
    let key = (staker, pool_id);
    env.storage().persistent().remove(&key);
}

// Reward token storage
pub fn get_reward_token(env: &Env, pool_id: u32, token: &Address) -> Option<RewardToken> {
    let key = (pool_id, token);
    env.storage().persistent().get(&key)
}

pub fn set_reward_token(env: &Env, pool_id: u32, token: &RewardToken) {
    let key = (pool_id, &token.token_address);
    env.storage().persistent().set(&key, token);
}

// Vesting schedule storage
pub fn get_vesting(env: &Env, beneficiary: &Address, pool_id: u32) -> Option<VestingSchedule> {
    let key = (beneficiary, pool_id);
    env.storage().persistent().get(&key)
}

pub fn set_vesting(env: &Env, beneficiary: &Address, pool_id: u32, schedule: &VestingSchedule) {
    let key = (beneficiary, pool_id);
    env.storage().persistent().set(&key, schedule);
}

// Performance metrics storage
pub fn get_metrics(env: &Env, pool_id: u32) -> Option<PerformanceMetrics> {
    let key = (pool_id, "METRICS");
    env.storage().persistent().get(&key)
}

pub fn set_metrics(env: &Env, metrics: &PerformanceMetrics) {
    let key = (metrics.pool_id, "METRICS");
    env.storage().persistent().set(&key, metrics);
}

// Claim history storage
pub fn add_claim_record(env: &Env, record: &ClaimRecord) {
    let mut history: Vec<ClaimRecord> = env.storage()
        .persistent()
        .get(&(&record.claimer, record.pool_id))
        .unwrap_or(Vec::new(env));
    
    history.push_back(record.clone());
    env.storage().persistent().set(&(&record.claimer, record.pool_id), &history);
}

pub fn get_claim_history(env: &Env, claimer: &Address, pool_id: u32) -> Vec<ClaimRecord> {
    env.storage()
        .persistent()
        .get(&(claimer, pool_id))
        .unwrap_or(Vec::new(env))
}
