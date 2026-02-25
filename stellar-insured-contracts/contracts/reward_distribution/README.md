# Reward Distribution & Incentive Engine

A comprehensive reward distribution and incentive engine for Stellar Soroban smart contracts, supporting multi-token rewards, time-locked vesting, performance bonuses, and risk-adjusted yield calculations.

## Features

### 1. Multi-Token Reward Distribution

- Support for multiple reward tokens per pool
- Configurable emission rates per token
- Automatic reward allocation tracking
- Batch distribution capabilities

### 2. Time-Locked Vesting

- Flexible vesting schedules with cliff periods
- Multiple vesting curves:
  - **Linear**: Constant vesting rate over time
  - **Stepped**: Quarterly releases (25% per quarter)
  - **Exponential**: Accelerating vesting rate
- Partial claim support
- Vesting progress tracking

### 3. Performance-Based Bonuses

- Dynamic multipliers based on pool metrics:
  - **Utilization Rate**: High utilization = higher rewards (up to +20%)
  - **Claim Ratio**: Low claims = higher rewards (up to +15%)
  - **Volatility Score**: Low volatility = higher rewards (up to +10%)
  - **Counterparty Risk**: Low risk = higher rewards (up to +10%)
- Maximum bonus multiplier: 1.55x
- Automatic bonus calculation and application

### 4. Risk-Adjusted Yield

- Base APY adjusted by risk factors
- Risk adjustment factor (lower = higher risk premium)
- Performance multiplier integration
- Capped at 10,000% APY for safety

### 5. Emission Rate Controls

- Maximum emission rate limits
- Inflation cap enforcement (basis points per year)
- Automatic rate adjustments
- Time-based adjustment intervals

### 6. Reward Escrow Mechanisms

- Secure reward token storage
- Allocation tracking
- Distribution verification
- Balance checks before payouts

### 7. Fair Launch Protections

- Minimum stake requirements
- Lock periods for early withdrawal prevention
- Early withdrawal penalties (up to 20%, decreasing linearly)
- Emergency unstake with penalty option

## Architecture

### Core Components

#### RewardPool

```rust
pub struct RewardPool {
    pub pool_id: u32,
    pub name: String,
    pub total_staked: i128,
    pub reward_tokens: Vec<Address>,
    pub base_apy: u32,                // Basis points
    pub risk_adjustment_factor: u32,  // Basis points
    pub status: RewardStatus,
    pub min_stake: i128,
    pub lock_period: u64,
}
```

#### StakePosition

```rust
pub struct StakePosition {
    pub staker: Address,
    pub pool_id: u32,
    pub amount: i128,
    pub stake_time: u64,
    pub last_claim_time: u64,
    pub performance_multiplier: u32,  // Basis points (10000 = 1x)
}
```

#### VestingSchedule

```rust
pub struct VestingSchedule {
    pub cliff_duration: u64,
    pub vesting_duration: u64,
    pub curve: VestingCurve,
    pub start_time: u64,
    pub total_amount: i128,
    pub claimed_amount: i128,
}
```

## Usage Examples

### Initialize Contract

```rust
RewardDistribution::initialize(env, admin_address);
```

### Create Reward Pool

```rust
let pool_id = RewardDistribution::create_pool(
    env,
    admin,
    String::from_str(&env, "High Yield Pool"),
    2_000,      // 20% base APY
    7_000,      // Risk adjustment factor
    1000_0000000, // Min stake: 1000 tokens
    604800,     // 7 day lock period
)?;
```

### Add Reward Token

```rust
RewardDistribution::add_reward_token(
    env,
    admin,
    pool_id,
    token_address,
    100_0000000,  // 100 tokens per second emission
    10_000_0000000, // 10,000 tokens total allocated
)?;
```

### Stake Tokens

```rust
RewardDistribution::stake(
    env,
    staker_address,
    pool_id,
    5000_0000000, // Stake 5000 tokens
)?;
```

### Claim Rewards

```rust
let rewards = RewardDistribution::claim_rewards(
    env,
    staker_address,
    pool_id,
    reward_token_address,
)?;
```

### Create Vesting Schedule

```rust
RewardDistribution::create_vesting_schedule(
    env,
    admin,
    beneficiary,
    pool_id,
    10000_0000000,  // 10,000 tokens
    2592000,        // 30 day cliff
    31536000,       // 1 year vesting
    VestingCurve::Linear,
)?;
```

### Claim Vested Rewards

```rust
let vested_amount = RewardDistribution::claim_vested(
    env,
    beneficiary,
    pool_id,
    token_address,
)?;
```

### Update Performance Metrics

```rust
RewardDistribution::update_performance_metrics(
    env,
    admin,
    pool_id,
    8500,  // 85% utilization
    800,   // 8% claim ratio
    2000,  // Low volatility
    1500,  // Low counterparty risk
)?;
```

### Apply Performance Bonus

```rust
let multiplier = RewardDistribution::apply_performance_bonus(
    env,
    admin,
    staker_address,
    pool_id,
)?;
```

## Reward Calculation Formula

### Base Rewards

```
base_rewards = (stake_amount × base_apy × duration) / (365 days × 10000)
```

### Risk-Adjusted Rewards

```
risk_adjusted = base_rewards × (20000 - risk_factor) / 10000
```

### Final Rewards

```
final_rewards = risk_adjusted × performance_multiplier / 10000
```

### Performance Multiplier

```
multiplier = 10000 (base)
  + utilization_bonus (0-2000)
  + low_claim_bonus (0-1500)
  + low_volatility_bonus (0-1000)
  + low_risk_bonus (0-1000)

Max: 15500 (1.55x)
```

## Security Features

1. **Authorization Checks**: All admin functions require authentication
2. **Pause Mechanism**: Emergency pause for all staking operations
3. **Lock Periods**: Prevent early withdrawal gaming
4. **Emission Caps**: Prevent excessive inflation
5. **Balance Verification**: Check reward availability before distribution
6. **Penalty System**: Discourage early withdrawals

## View Functions

- `get_pool(pool_id)` - Get pool information
- `get_stake(staker, pool_id)` - Get stake position
- `get_vesting(beneficiary, pool_id)` - Get vesting schedule
- `get_claimable_vested(beneficiary, pool_id)` - Get claimable vested amount
- `get_pending_rewards(staker, pool_id)` - Calculate pending rewards
- `get_metrics(pool_id)` - Get performance metrics
- `get_claim_history(claimer, pool_id)` - Get claim history
- `get_risk_adjusted_apy(pool_id)` - Get current risk-adjusted APY

## Admin Functions

- `initialize(admin)` - Initialize contract
- `create_pool(...)` - Create new reward pool
- `add_reward_token(...)` - Add reward token to pool
- `create_vesting_schedule(...)` - Create vesting schedule
- `update_performance_metrics(...)` - Update pool metrics
- `apply_performance_bonus(...)` - Apply bonus to staker
- `adjust_emission_rate(...)` - Adjust emission based on inflation cap
- `batch_distribute(...)` - Distribute to multiple recipients
- `update_pool_status(...)` - Change pool status
- `set_paused(paused)` - Pause/unpause contract

## Testing

Run tests with:

```bash
cargo test
```

## Deployment

Build the contract:

```bash
cargo build --target wasm32-unknown-unknown --release
```

Optimize the WASM:

```bash
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/reward_distribution.wasm
```

Deploy to Stellar:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/reward_distribution.wasm \
  --source <SOURCE_ACCOUNT> \
  --network <NETWORK>
```

## License

MIT
