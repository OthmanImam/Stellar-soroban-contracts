//! Simplified error types for insurance contracts

use soroban_sdk::contracterror;

/// Comprehensive error type for insurance contracts
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    // ===== General / Authorization Errors (1–19) =====
    Unauthorized = 1,
    Paused = 2,
    InvalidInput = 3,
    InsufficientFunds = 4,
    NotFound = 5,
    AlreadyExists = 6,
    InvalidState = 7,
    Overflow = 8,
    NotInitialized = 9,
    AlreadyInitialized = 10,
    InvalidRole = 11,
    RoleNotFound = 12,
    NotTrustedContract = 13,
    InvalidAddress = 14,
    Underflow = 15,
    DivisionByZero = 16,
    FunctionPaused = 17,

    // ===== Policy-Specific Errors (20–39) =====
    PolicyNotFound = 20,
    InvalidPolicyState = 21,
    InvalidCoverageAmount = 22,
    InvalidPremiumAmount = 23,
    InvalidDuration = 24,
    CannotRenewPolicy = 25,
    InvalidStateTransition = 26,
    PremiumExceedsCoverage = 27,

    // ===== Claim-Specific Errors (40–59) =====
    ClaimNotFound = 40,
    InvalidClaimState = 41,
    ClaimAmountExceedsCoverage = 42,
    ClaimPeriodExpired = 43,
    CannotSubmitClaim = 44,
    PolicyCoverageExpired = 45,
    EvidenceError = 46,
    EvidenceAlreadyExists = 47,
    EvidenceNotFound = 48,
    InvalidEvidenceHash = 49,
    ClaimExceedsCoverage = 50,

    // ===== Oracle-Specific Errors (60–79) =====
    OracleValidationFailed = 60,
    InsufficientOracleSubmissions = 61,
    OracleDataStale = 62,
    OracleOutlierDetected = 63,
    OracleNotConfigured = 64,
    InvalidOracleContract = 65,

    // ===== Governance Errors (80–99) =====
    VotingPeriodEnded = 80,
    AlreadyVoted = 81,
    ProposalNotActive = 82,
    QuorumNotMet = 83,
    ThresholdNotMet = 84,
    ProposalNotFound = 85,
    InvalidProposalType = 86,
    SlashingContractNotSet = 87,
    SlashingExecutionFailed = 88,
    InvalidVotingDuration = 89,

    // ===== Treasury Errors (100–119) =====
    TreasuryFundNotFound = 100,
    InsufficientTreasuryBalance = 101,
    InvalidAllocation = 102,
    InvalidDistribution = 103,
    TreasuryLocked = 104,

    // ===== Slashing Errors (120–139) =====
    ValidatorNotFound = 120,
    InvalidSlashingAmount = 121,
    SlashingAlreadyExecuted = 122,
    SlashingPeriodNotActive = 123,
    SlashingExceedsStake = 124,
    SlashingPercentTooHigh = 125,

    // ===== Risk Pool Errors (140–159) =====
    RiskPoolNotFound = 140,
    InvalidRiskPoolState = 141,
    InsufficientRiskPoolBalance = 142,
    RiskPoolLocked = 143,
    InvalidReserveRatio = 144,
    DepositBelowMinStake = 145,
    WithdrawalExceedsBalance = 146,

    // ===== Cross-Chain Errors (160–179) =====
    BridgeNotRegistered = 160,
    ChainNotSupported = 161,
    MessageAlreadyProcessed = 162,
    InsufficientConfirmations = 163,
    AssetNotMapped = 164,
    MessageExpired = 165,
    InvalidMessageFormat = 166,
    BridgePaused = 167,
    ValidatorAlreadyConfirmed = 168,
    CrossChainProposalNotFound = 169,
    InvalidChainId = 170,
    NonceMismatch = 171,

    // ===== Input Validation Errors (200–249) =====
    AmountMustBePositive = 200,
    AmountOutOfBounds = 201,
    InvalidPercentage = 202,
    InvalidBasisPoints = 203,
    TimestampNotFuture = 204,
    TimestampNotPast = 205,
    InvalidTimeRange = 206,
    EmptyInput = 207,
    InputTooLong = 208,
    InputTooShort = 209,
    InvalidPaginationParams = 210,
    DuplicateAddress = 211,
    QuorumTooLow = 212,
    ThresholdTooLow = 213,
}
