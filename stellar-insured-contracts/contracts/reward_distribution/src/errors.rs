use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PoolNotFound = 4,
    StakeNotFound = 5,
    InsufficientStake = 6,
    BelowMinimumStake = 7,
    PoolPaused = 8,
    ContractPaused = 9,
    InvalidEmissionRate = 10,
    ExceedsInflationCap = 11,
    LockPeriodNotMet = 12,
    VestingNotStarted = 13,
    NoRewardsAvailable = 14,
    InvalidVestingSchedule = 15,
    InvalidPerformanceMultiplier = 16,
    TokenNotRegistered = 17,
    InsufficientRewardBalance = 18,
    InvalidRiskAdjustment = 19,
    InvalidAPY = 20,
    BatchSizeTooLarge = 21,
    InvalidPoolStatus = 22,
}
