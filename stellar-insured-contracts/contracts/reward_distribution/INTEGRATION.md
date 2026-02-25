# Integration Guide

## Overview

This guide demonstrates how to integrate the Reward Distribution & Incentive Engine into your Stellar Soroban application.

## Quick Start

### 1. Deploy the Contract

```bash
# Build the contract
cd stellar-insured-contracts/contracts/reward_distribution
cargo build --target wasm32-unknown-unknown --release

# Optimize
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/reward_distribution.wasm

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/reward_distribution.wasm \
  --source ADMIN_SECRET_KEY \
  --network testnet
```

### 2. Initialize the Contract

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source ADMIN_SECRET_KEY \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS>
```

## Integration Scenarios

### Scenario 1: Simple Staking Pool

Create a basic staking pool with single reward token:

```rust
// 1. Create pool
let pool_id = client.create_pool(
    &admin,
    &String::from_str(&env, "Basic Staking"),
    &1_500,      // 15% APY
    &9_000,      // Low risk
    &100_0000000, // Min 100 tokens
    &0,          // No lock period
);

// 2. Add reward token
client.add_reward_token(
    &admin,
    &pool_id,
    &reward_token,
    &10_0000000,  // 10 tokens/second
    &1_000_000_0000000, // 1M total
);

// 3. Users stake
client.stake(&user, &pool_id, &1000_0000000);

// 4. Users claim rewards
let rewards = client.claim_rewards(&user, &pool_id, &reward_token);
```

### Scenario 2: High-Risk, High-Reward Pool

Create a pool with risk-adjusted yields:

```rust
// Higher risk = higher rewards
let pool_id = client.create_pool(
    &admin,
    &String::from_str(&env, "High Risk Pool"),
    &5_000,      // 50% base APY
    &5_000,      // High risk factor (lower = higher risk)
    &1000_0000000, // Min 1000 tokens
    &2592000,    // 30 day lock
);

// Risk adjustment will increase actual APY
// Effective APY = 50% × (20000 - 5000) / 10000 = 75%
```

### Scenario 3: Performance-Based Rewards

Implement dynamic rewards based on pool performance:

```rust
// 1. Create pool
let pool_id = client.create_pool(
    &admin,
    &String::from_str(&env, "Performance Pool"),
    &2_000,      // 20% base APY
    &8_000,      // Medium risk
    &500_0000000,
    &604800,     // 7 day lock
);

// 2. Update metrics regularly (e.g., daily)
client.update_performance_metrics(
    &admin,
    &pool_id,
    &8_500,  // 85% utilization
    &500,    // 5% claim ratio
    &1_500,  // Low volatility
    &1_000,  // Low counterparty risk
);

// 3. Apply bonuses to stakers
client.apply_performance_bonus(&admin, &user, &pool_id);

// User now gets bonus multiplier (potentially 1.45x)
```

### Scenario 4: Team Token Vesting

Implement token vesting for team members:

```rust
// 1. Create vesting schedule
client.create_vesting_schedule(
    &admin,
    &team_member,
    &pool_id,
    &100_000_0000000,  // 100k tokens
    &7_776_000,        // 90 day cliff
    &126_144_000,      // 4 year vesting
    &VestingCurve::Linear,
);

// 2. Team member claims vested tokens periodically
let vested = client.claim_vested(&team_member, &pool_id, &token);
```

### Scenario 5: Multi-Token Rewards

Support multiple reward tokens in one pool:

```rust
let pool_id = client.create_pool(
    &admin,
    &String::from_str(&env, "Multi-Reward Pool"),
    &2_500,
    &8_000,
    &100_0000000,
    &0,
);

// Add multiple reward tokens
client.add_reward_token(&admin, &pool_id, &token_a, &5_0000000, &500_000_0000000);
client.add_reward_token(&admin, &pool_id, &token_b, &2_0000000, &200_000_0000000);
client.add_reward_token(&admin, &pool_id, &token_c, &1_0000000, &100_000_0000000);

// Users claim each token separately
let rewards_a = client.claim_rewards(&user, &pool_id, &token_a);
let rewards_b = client.claim_rewards(&user, &pool_id, &token_b);
let rewards_c = client.claim_rewards(&user, &pool_id, &token_c);
```

### Scenario 6: Fair Launch with Lock Period

Prevent early dumping with lock periods and penalties:

```rust
let pool_id = client.create_pool(
    &admin,
    &String::from_str(&env, "Fair Launch Pool"),
    &3_000,      // 30% APY
    &8_000,
    &50_0000000,
    &2_592_000,  // 30 day lock
);

// User stakes
client.stake(&user, &pool_id, &1000_0000000);

// Try to unstake early (will fail)
// client.unstake(&user, &pool_id, &1000_0000000); // Error: LockPeriodNotMet

// Emergency unstake with penalty
let returned = client.emergency_unstake(&user, &pool_id);
// Returns less than staked amount due to penalty
```

## Advanced Integration Patterns

### Pattern 1: Automated Reward Distribution

```rust
// Backend service runs periodically
pub fn distribute_rewards_batch(
    env: &Env,
    admin: &Address,
    pool_id: u32,
    token: &Address,
) {
    // Get all stakers (from off-chain index)
    let stakers = get_active_stakers(pool_id);

    let mut recipients = Vec::new(env);
    let mut amounts = Vec::new(env);

    for staker in stakers {
        let pending = client.get_pending_rewards(&staker, &pool_id);
        if pending > 0 {
            recipients.push_back(staker);
            amounts.push_back(pending);
        }
    }

    // Batch distribute
    client.batch_distribute(admin, &pool_id, token, &recipients, &amounts);
}
```

### Pattern 2: Dynamic APY Adjustment

```rust
// Adjust APY based on market conditions
pub fn adjust_pool_apy(
    env: &Env,
    admin: &Address,
    pool_id: u32,
    market_conditions: &MarketData,
) {
    let mut pool = client.get_pool(&pool_id);

    // Calculate new APY based on:
    // - TVL (Total Value Locked)
    // - Market volatility
    // - Competitor rates
    let new_apy = calculate_competitive_apy(market_conditions);

    // Update pool (would need additional admin function)
    // This is a conceptual example
}
```

### Pattern 3: Tiered Reward System

```rust
// Create multiple pools for different tiers
pub fn create_tiered_system(env: &Env, admin: &Address) {
    // Bronze tier
    let bronze = client.create_pool(
        admin,
        &String::from_str(env, "Bronze Tier"),
        &1_000,  // 10% APY
        &9_000,
        &10_0000000,   // Low min stake
        &0,
    );

    // Silver tier
    let silver = client.create_pool(
        admin,
        &String::from_str(env, "Silver Tier"),
        &2_000,  // 20% APY
        &8_000,
        &100_0000000,  // Medium min stake
        &604800,       // 7 day lock
    );

    // Gold tier
    let gold = client.create_pool(
        admin,
        &String::from_str(env, "Gold Tier"),
        &4_000,  // 40% APY
        &7_000,
        &1000_0000000, // High min stake
        &2592000,      // 30 day lock
    );
}
```

### Pattern 4: Governance Integration

```rust
// Reward governance participants
pub fn reward_governance_participation(
    env: &Env,
    admin: &Address,
    voter: &Address,
    pool_id: u32,
    votes_cast: u32,
) {
    // Calculate bonus based on participation
    let participation_bonus = calculate_governance_bonus(votes_cast);

    // Apply bonus multiplier
    let mut stake = client.get_stake(voter, &pool_id);
    stake.performance_multiplier += participation_bonus;

    // Would need additional function to update multiplier
}
```

## Monitoring & Analytics

### Track Pool Performance

```rust
pub fn get_pool_analytics(env: &Env, pool_id: u32) -> PoolAnalytics {
    let pool = client.get_pool(&pool_id);
    let metrics = client.get_metrics(&pool_id);
    let apy = client.get_risk_adjusted_apy(&pool_id);

    PoolAnalytics {
        total_staked: pool.total_staked,
        current_apy: apy,
        utilization: metrics.utilization_rate,
        claim_ratio: metrics.claim_ratio,
        volatility: metrics.volatility_score,
        risk_score: metrics.counterparty_risk,
    }
}
```

### Monitor User Positions

```rust
pub fn get_user_dashboard(
    env: &Env,
    user: &Address,
    pool_id: u32,
) -> UserDashboard {
    let stake = client.get_stake(user, &pool_id);
    let pending = client.get_pending_rewards(user, &pool_id);
    let history = client.get_claim_history(user, &pool_id);

    UserDashboard {
        staked_amount: stake.amount,
        stake_duration: env.ledger().timestamp() - stake.stake_time,
        pending_rewards: pending,
        performance_multiplier: stake.performance_multiplier,
        total_claimed: calculate_total_claimed(&history),
        claim_count: history.len(),
    }
}
```

## Best Practices

1. **Regular Metric Updates**: Update performance metrics at least daily for accurate bonus calculations

2. **Emission Rate Monitoring**: Regularly check and adjust emission rates to stay within inflation caps

3. **Lock Period Strategy**: Use lock periods to align incentives and prevent gaming

4. **Multi-Token Diversification**: Offer multiple reward tokens to attract different user segments

5. **Vesting for Team**: Always use vesting schedules for team allocations

6. **Emergency Procedures**: Keep pause functionality for emergency situations

7. **Gas Optimization**: Use batch operations when distributing to multiple users

8. **Off-Chain Indexing**: Maintain off-chain indexes for efficient querying of stakers and positions

## Security Considerations

1. **Admin Key Management**: Secure admin keys with multi-sig or hardware wallets

2. **Reward Token Funding**: Ensure sufficient reward tokens are deposited before enabling claims

3. **Rate Limiting**: Implement rate limiting for claim operations if needed

4. **Audit Trail**: Monitor all admin operations and large claims

5. **Gradual Rollout**: Start with small pools and gradually increase limits

6. **Testing**: Thoroughly test all scenarios on testnet before mainnet deployment

## Troubleshooting

### Common Issues

**Issue**: Users can't claim rewards

- Check if reward tokens are funded
- Verify emission rates are set correctly
- Ensure pool status is Active

**Issue**: APY seems incorrect

- Verify risk adjustment factor is set properly
- Check if performance metrics are updated
- Confirm performance multiplier is applied

**Issue**: Vesting not working

- Ensure cliff period has passed
- Verify vesting schedule was created correctly
- Check if beneficiary address is correct

## Support

For issues or questions:

- GitHub Issues: [repository-url]
- Discord: [discord-link]
- Documentation: [docs-url]
