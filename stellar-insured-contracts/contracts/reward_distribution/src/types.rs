use soroban_sdk::{contracttype, Address, String, Vec};

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum VestingCurve {
    Linear,
    Stepped,
    Exponential,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum RewardStatus {
    Active,
    Paused,
    Completed,
}

#[contracttype]
#[derive(Clone)]
pub struct RewardToken {
    pub token_address: Address,
    pub emission_rate: i128,      // Tokens per second
    pub total_allocated: i128,
    pub total_distributed: i128,
    pub active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct VestingSchedule {
    pub cliff_duration: u64,      // Seconds before vesting starts
    pub vesting_duration: u64,    // Total vesting period
    pub curve: VestingCurve,
    pub start_time: u64,
    pub total_amount: i128,
    pub claimed_amount: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct StakePosition {
    pub staker: Address,
    pub pool_id: u32,
    pub amount: i128,
    pub stake_time: u64,
    pub last_claim_time: u64,
    pub performance_multiplier: u32,  // Basis points (10000 = 1x)
}

#[contracttype]
#[derive(Clone)]
pub struct RewardPool {
    pub pool_id: u32,
    pub name: String,
    pub total_staked: i128,
    pub reward_tokens: Vec<Address>,
    pub base_apy: u32,                // Basis points
    pub risk_adjustment_factor: u32,  // Basis points (lower = higher risk)
    pub status: RewardStatus,
    pub min_stake: i128,
    pub lock_period: u64,             // Minimum lock duration
}

#[contracttype]
#[derive(Clone)]
pub struct ClaimRecord {
    pub claimer: Address,
    pub pool_id: u32,
    pub token: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct EmissionConfig {
    pub max_emission_rate: i128,
    pub inflation_cap: u32,           // Basis points per year
    pub adjustment_interval: u64,     // Seconds between rate adjustments
    pub last_adjustment: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct PerformanceMetrics {
    pub pool_id: u32,
    pub utilization_rate: u32,        // Basis points
    pub claim_ratio: u32,             // Basis points
    pub volatility_score: u32,        // 0-10000
    pub counterparty_risk: u32,       // 0-10000
}
