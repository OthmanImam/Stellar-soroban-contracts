use soroban_sdk::Env;
use crate::types::*;
use crate::errors::Error;

/// Calculate rewards based on stake amount, time, and pool parameters
pub fn calculate_base_rewards(
    env: &Env,
    stake_amount: i128,
    stake_duration: u64,
    base_apy: u32,
) -> i128 {
    // APY in basis points (10000 = 100%)
    // Formula: (amount * apy * duration) / (365 days * 10000)
    let seconds_per_year: i128 = 31_536_000;
    let basis_points: i128 = 10_000;
    
    let rewards = (stake_amount * base_apy as i128 * stake_duration as i128) 
        / (seconds_per_year * basis_points);
    
    rewards
}

/// Apply risk adjustment to rewards
pub fn apply_risk_adjustment(
    base_rewards: i128,
    risk_adjustment_factor: u32,
) -> i128 {
    // Risk adjustment factor in basis points (10000 = 1x, lower = higher risk premium)
    // Higher risk = higher rewards
    let inverse_factor = 20_000 - risk_adjustment_factor as i128;
    (base_rewards * inverse_factor) / 10_000
}

/// Apply performance multiplier to rewards
pub fn apply_performance_multiplier(
    rewards: i128,
    multiplier: u32,
) -> i128 {
    // Multiplier in basis points (10000 = 1x)
    (rewards * multiplier as i128) / 10_000
}

/// Calculate vested amount based on vesting schedule
pub fn calculate_vested_amount(
    env: &Env,
    schedule: &VestingSchedule,
) -> Result<i128, Error> {
    let current_time = env.ledger().timestamp();
    
    // Check if cliff period has passed
    if current_time < schedule.start_time + schedule.cliff_duration {
        return Ok(0);
    }
    
    let elapsed = current_time.saturating_sub(schedule.start_time + schedule.cliff_duration);
    let vesting_duration = schedule.vesting_duration;
    
    if elapsed >= vesting_duration {
        // Fully vested
        return Ok(schedule.total_amount - schedule.claimed_amount);
    }
    
    let vested_amount = match schedule.curve {
        VestingCurve::Linear => {
            // Linear vesting
            (schedule.total_amount * elapsed as i128) / vesting_duration as i128
        },
        VestingCurve::Stepped => {
            // Stepped vesting (25% every quarter)
            let quarters_passed = elapsed / (vesting_duration / 4);
            (schedule.total_amount * quarters_passed as i128) / 4
        },
        VestingCurve::Exponential => {
            // Exponential vesting (accelerating)
            let progress = (elapsed as i128 * 10_000) / vesting_duration as i128;
            let exponential_progress = (progress * progress) / 10_000;
            (schedule.total_amount * exponential_progress) / 10_000
        },
    };
    
    Ok(vested_amount.saturating_sub(schedule.claimed_amount))
}

/// Calculate performance-based bonus multiplier
pub fn calculate_performance_bonus(
    metrics: &PerformanceMetrics,
) -> u32 {
    // Base multiplier is 10000 (1x)
    let mut multiplier: u32 = 10_000;
    
    // High utilization bonus (up to +20%)
    if metrics.utilization_rate > 8_000 {
        multiplier += 2_000;
    } else if metrics.utilization_rate > 6_000 {
        multiplier += 1_000;
    }
    
    // Low claim ratio bonus (up to +15%)
    if metrics.claim_ratio < 1_000 {
        multiplier += 1_500;
    } else if metrics.claim_ratio < 2_000 {
        multiplier += 750;
    }
    
    // Low volatility bonus (up to +10%)
    if metrics.volatility_score < 2_000 {
        multiplier += 1_000;
    } else if metrics.volatility_score < 4_000 {
        multiplier += 500;
    }
    
    // Low counterparty risk bonus (up to +10%)
    if metrics.counterparty_risk < 2_000 {
        multiplier += 1_000;
    } else if metrics.counterparty_risk < 4_000 {
        multiplier += 500;
    }
    
    // Cap at 1.55x (15500)
    if multiplier > 15_500 {
        multiplier = 15_500;
    }
    
    multiplier
}

/// Calculate risk-adjusted yield
pub fn calculate_risk_adjusted_yield(
    base_apy: u32,
    risk_adjustment_factor: u32,
    performance_multiplier: u32,
) -> u32 {
    let adjusted_apy = (base_apy as i128 * (20_000 - risk_adjustment_factor as i128)) / 10_000;
    let final_apy = (adjusted_apy * performance_multiplier as i128) / 10_000;
    
    // Cap at 10000% APY (1,000,000 basis points)
    if final_apy > 1_000_000 {
        1_000_000
    } else {
        final_apy as u32
    }
}

/// Calculate emission rate adjustment based on inflation cap
pub fn calculate_emission_adjustment(
    current_rate: i128,
    total_supply: i128,
    inflation_cap: u32,
    time_elapsed: u64,
) -> i128 {
    // Calculate max allowed emission based on inflation cap
    let seconds_per_year: i128 = 31_536_000;
    let max_annual_inflation = (total_supply * inflation_cap as i128) / 10_000;
    let max_rate = max_annual_inflation / seconds_per_year;
    
    if current_rate > max_rate {
        max_rate
    } else {
        current_rate
    }
}

/// Calculate early withdrawal penalty
pub fn calculate_early_withdrawal_penalty(
    amount: i128,
    lock_period: u64,
    time_staked: u64,
) -> i128 {
    if time_staked >= lock_period {
        return 0;
    }
    
    // Penalty decreases linearly from 20% to 0%
    let max_penalty = 2_000; // 20% in basis points
    let time_remaining = lock_period.saturating_sub(time_staked);
    let penalty_rate = (max_penalty as u64 * time_remaining) / lock_period;
    
    (amount * penalty_rate as i128) / 10_000
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_base_rewards_calculation() {
        // Mock env would be needed for full test
        let stake_amount = 1_000_0000000; // 1000 tokens (7 decimals)
        let stake_duration = 31_536_000; // 1 year
        let base_apy = 1_000; // 10%
        
        // Expected: 1000 * 0.10 = 100 tokens
        // Actual calculation will be close to this
        let rewards = calculate_base_rewards(
            &soroban_sdk::Env::default(),
            stake_amount,
            stake_duration,
            base_apy,
        );
        
        assert!(rewards > 0);
    }
    
    #[test]
    fn test_risk_adjustment() {
        let base_rewards = 100_0000000;
        let risk_factor = 8_000; // Lower risk
        
        let adjusted = apply_risk_adjustment(base_rewards, risk_factor);
        
        // Should increase rewards for higher risk
        assert!(adjusted > base_rewards);
    }
    
    #[test]
    fn test_performance_multiplier() {
        let rewards = 100_0000000;
        let multiplier = 12_000; // 1.2x
        
        let result = apply_performance_multiplier(rewards, multiplier);
        
        assert_eq!(result, 120_0000000);
    }
}
