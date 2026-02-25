//! Common error types for insurance contracts
//!
//! This module defines a unified set of error codes that are used across all
//! insurance contracts to ensure consistent error handling and reporting.
//!
//! # Error Code Ranges
//! | Range    | Category                          |
//! |----------|-----------------------------------|
//! | 1–19     | General / Authorization           |
//! | 20–39    | Policy-specific                   |
//! | 40–59    | Claim-specific                    |
//! | 60–79    | Oracle-specific                   |
//! | 80–99    | Governance                        |
//! | 100–119  | Treasury                          |
//! | 120–139  | Slashing                          |
//! | 140–159  | Risk Pool                         |
//! | 160–179  | Cross-Chain                       |
//! | 200–249  | Input Validation (new)            |

use soroban_sdk::contracterror;

/// Comprehensive error type for insurance contracts
///
/// All errors are assigned unique codes for easy identification and debugging.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    // ===== General / Authorization Errors (1–19) =====

    /// Caller is not authorized to perform this action
    Unauthorized = 1,

    /// Contract is paused and operations are not allowed
    Paused = 2,

    /// Specific function is paused and cannot be called
    FunctionPaused = 17,

    /// Generic invalid input (prefer specific validation errors below)
    InvalidInput = 3,

    /// Insufficient funds for operation
    InsufficientFunds = 4,

    /// Requested resource not found
    NotFound = 5,

    /// Resource already exists
    AlreadyExists = 6,

    /// Invalid state for operation
    InvalidState = 7,

    /// Arithmetic overflow occurred
    Overflow = 8,

    /// Contract not initialized
    NotInitialized = 9,

    /// Contract already initialized
    AlreadyInitialized = 10,

    /// Invalid role or permission
    InvalidRole = 11,

    /// Role not found
    RoleNotFound = 12,

    /// Contract not trusted for cross-contract calls
    NotTrustedContract = 13,

    /// Invalid address format or value
    InvalidAddress = 14,

    /// Operation would cause underflow
    Underflow = 15,

    /// Division by zero
    DivisionByZero = 16,

    // ===== Policy-Specific Errors (20–39) =====

    /// Policy not found
    PolicyNotFound = 20,

    /// Invalid policy state for operation
    InvalidPolicyState = 21,

    /// Coverage amount out of allowed bounds
    InvalidCoverageAmount = 22,

    /// Premium amount out of allowed bounds
    InvalidPremiumAmount = 23,

    /// Policy duration out of allowed bounds
    InvalidDuration = 24,

    /// Cannot renew an expired or cancelled policy
    CannotRenewPolicy = 25,

    /// State transition is not allowed
    InvalidStateTransition = 26,

    /// Premium amount exceeds coverage (sanity check)
    PremiumExceedsCoverage = 27,

    // ===== Claim-Specific Errors (40–59) =====

    /// Claim not found
    ClaimNotFound = 40,

    /// Invalid claim state for operation
    InvalidClaimState = 41,

    /// Claim amount exceeds coverage
    ClaimAmountExceedsCoverage = 42,

    /// Claim period has expired
    ClaimPeriodExpired = 43,

    /// Cannot submit claim for this policy
    CannotSubmitClaim = 44,

    /// Policy coverage has expired
    PolicyCoverageExpired = 45,

    /// Evidence-related error
    EvidenceError = 46,

    /// Evidence already exists
    EvidenceAlreadyExists = 47,

    /// Evidence not found
    EvidenceNotFound = 48,

    /// Invalid evidence hash (e.g. all-zero placeholder)
    InvalidEvidenceHash = 49,

    /// Claim amount exceeds coverage (alias used in validation module)
    ClaimExceedsCoverage = 50,

    // ===== Oracle-Specific Errors (60–79) =====

    /// Oracle validation failed
    OracleValidationFailed = 60,

    /// Insufficient oracle submissions
    InsufficientOracleSubmissions = 61,

    /// Oracle data is stale
    OracleDataStale = 62,

    /// Oracle data is an outlier
    OracleOutlierDetected = 63,

    /// Oracle contract not configured
    OracleNotConfigured = 64,

    /// Oracle contract is invalid
    InvalidOracleContract = 65,

    // ===== Governance Errors (80–99) =====

    /// Voting period has ended
    VotingPeriodEnded = 80,

    /// Address has already voted
    AlreadyVoted = 81,

    /// Proposal not active
    ProposalNotActive = 82,

    /// Quorum not met
    QuorumNotMet = 83,

    /// Threshold not met
    ThresholdNotMet = 84,

    /// Proposal not found
    ProposalNotFound = 85,

    /// Invalid proposal type
    InvalidProposalType = 86,

    /// Slashing contract not set
    SlashingContractNotSet = 87,

    /// Slashing execution failed
    SlashingExecutionFailed = 88,

    /// Voting duration is out of allowed bounds
    InvalidVotingDuration = 89,

    // ===== Treasury Errors (100–119) =====

    /// Treasury fund not found
    TreasuryFundNotFound = 100,

    /// Insufficient treasury balance
    InsufficientTreasuryBalance = 101,

    /// Invalid allocation
    InvalidAllocation = 102,

    /// Invalid distribution
    InvalidDistribution = 103,

    /// Treasury locked
    TreasuryLocked = 104,

    // ===== Slashing Errors (120–139) =====

    /// Validator not found
    ValidatorNotFound = 120,

    /// Invalid slashing amount
    InvalidSlashingAmount = 121,

    /// Slashing already executed
    SlashingAlreadyExecuted = 122,

    /// Slashing period not active
    SlashingPeriodNotActive = 123,

    /// Slashing amount exceeds validator stake
    SlashingExceedsStake = 124,

    /// Slashing percentage exceeds maximum per-event limit
    SlashingPercentTooHigh = 125,

    // ===== Risk Pool Errors (140–159) =====

    /// Risk pool not found
    RiskPoolNotFound = 140,

    /// Invalid risk pool state
    InvalidRiskPoolState = 141,

    /// Insufficient risk pool balance
    InsufficientRiskPoolBalance = 142,

    /// Risk pool locked
    RiskPoolLocked = 143,

    /// Invalid reserve ratio
    InvalidReserveRatio = 144,

    /// Deposit amount is below the minimum stake requirement
    DepositBelowMinStake = 145,

    /// Withdrawal amount exceeds provider's available balance
    WithdrawalExceedsBalance = 146,

    // ===== Cross-Chain Errors (160–179) =====

    /// Bridge not registered
    BridgeNotRegistered = 160,

    /// Chain not supported
    ChainNotSupported = 161,

    /// Message already processed
    MessageAlreadyProcessed = 162,

    /// Insufficient confirmations for cross-chain message
    InsufficientConfirmations = 163,

    /// Asset not mapped for cross-chain transfer
    AssetNotMapped = 164,

    /// Cross-chain message has expired
    MessageExpired = 165,

    /// Invalid message format
    InvalidMessageFormat = 166,

    /// Bridge is paused
    BridgePaused = 167,

    /// Validator has already confirmed this message
    ValidatorAlreadyConfirmed = 168,

    /// Cross-chain proposal not found
    CrossChainProposalNotFound = 169,

    /// Invalid chain identifier
    InvalidChainId = 170,

    /// Nonce mismatch for replay protection
    NonceMismatch = 171,

    // ===== Input Validation Errors (200–249) =====

    /// Amount must be strictly positive (> 0)
    AmountMustBePositive = 200,

    /// Amount is outside the allowed [min, max] bounds
    AmountOutOfBounds = 201,

    /// Percentage value exceeds 100
    InvalidPercentage = 202,

    /// Basis-points value exceeds 10 000
    InvalidBasisPoints = 203,

    /// Timestamp must be in the future
    TimestampNotFuture = 204,

    /// Timestamp must be in the past
    TimestampNotPast = 205,

    /// Time range is invalid (start ≥ end)
    InvalidTimeRange = 206,

    /// Input string or bytes is empty
    EmptyInput = 207,

    /// Input string or bytes exceeds maximum allowed length
    InputTooLong = 208,

    /// Input string or bytes is shorter than minimum required length
    InputTooShort = 209,

    /// Pagination limit is zero or exceeds maximum page size
    InvalidPaginationParams = 210,

    /// Both addresses in a pair must be different
    DuplicateAddress = 211,

    /// Quorum percentage is below the minimum required (10 %)
    QuorumTooLow = 212,

    /// Approval threshold must be > 50 %
    ThresholdTooLow = 213,
}

/// Human-readable descriptions for every error variant.
impl ContractError {
    /// Get a human-readable description of the error.
    pub fn message(&self) -> &str {
        match self {
            // General / Authorization
            ContractError::Unauthorized => "Caller is not authorized",
            ContractError::Paused => "Contract is paused",
            ContractError::FunctionPaused => "Specific function is paused",
            ContractError::InvalidInput => "Invalid input provided",
            ContractError::InsufficientFunds => "Insufficient funds",
            ContractError::NotFound => "Resource not found",
            ContractError::AlreadyExists => "Resource already exists",
            ContractError::InvalidState => "Invalid state for operation",
            ContractError::Overflow => "Arithmetic overflow",
            ContractError::NotInitialized => "Contract not initialized",
            ContractError::AlreadyInitialized => "Contract already initialized",
            ContractError::InvalidRole => "Invalid role",
            ContractError::RoleNotFound => "Role not found",
            ContractError::NotTrustedContract => "Contract not trusted",
            ContractError::InvalidAddress => "Invalid address",
            ContractError::Underflow => "Arithmetic underflow",
            ContractError::DivisionByZero => "Division by zero",

            // Policy-Specific
            ContractError::PolicyNotFound => "Policy not found",
            ContractError::InvalidPolicyState => "Invalid policy state",
            ContractError::InvalidCoverageAmount => {
                "Coverage amount is outside the allowed range [1 XLM, 1 000 000 XLM]"
            }
            ContractError::InvalidPremiumAmount => {
                "Premium amount is outside the allowed range [0.1 XLM, 100 000 XLM]"
            }
            ContractError::InvalidDuration => {
                "Policy duration must be between 1 and 1 825 days"
            }
            ContractError::CannotRenewPolicy => "Cannot renew this policy",
            ContractError::InvalidStateTransition => "Invalid state transition",
            ContractError::PremiumExceedsCoverage => {
                "Premium amount must be less than coverage amount"
            }

            // Claim-Specific
            ContractError::ClaimNotFound => "Claim not found",
            ContractError::InvalidClaimState => "Invalid claim state",
            ContractError::ClaimAmountExceedsCoverage => "Claim exceeds coverage",
            ContractError::ClaimPeriodExpired => "Claim period expired",
            ContractError::CannotSubmitClaim => "Cannot submit claim for this policy",
            ContractError::PolicyCoverageExpired => "Policy coverage has expired",
            ContractError::EvidenceError => "Evidence error",
            ContractError::EvidenceAlreadyExists => "Evidence already exists",
            ContractError::EvidenceNotFound => "Evidence not found",
            ContractError::InvalidEvidenceHash => {
                "Evidence hash is invalid or is an all-zero placeholder"
            }
            ContractError::ClaimExceedsCoverage => "Claim amount exceeds policy coverage",

            // Oracle-Specific
            ContractError::OracleValidationFailed => "Oracle validation failed",
            ContractError::InsufficientOracleSubmissions => "Insufficient oracle submissions",
            ContractError::OracleDataStale => "Oracle data is stale",
            ContractError::OracleOutlierDetected => "Oracle data is an outlier",
            ContractError::OracleNotConfigured => "Oracle not configured",
            ContractError::InvalidOracleContract => "Invalid oracle contract",

            // Governance
            ContractError::VotingPeriodEnded => "Voting period has ended",
            ContractError::AlreadyVoted => "Already voted on this proposal",
            ContractError::ProposalNotActive => "Proposal is not active",
            ContractError::QuorumNotMet => "Quorum not met",
            ContractError::ThresholdNotMet => "Threshold not met",
            ContractError::ProposalNotFound => "Proposal not found",
            ContractError::InvalidProposalType => "Invalid proposal type",
            ContractError::SlashingContractNotSet => "Slashing contract not set",
            ContractError::SlashingExecutionFailed => "Slashing execution failed",
            ContractError::InvalidVotingDuration => {
                "Voting duration must be between 1 hour and 30 days"
            }

            // Treasury
            ContractError::TreasuryFundNotFound => "Treasury fund not found",
            ContractError::InsufficientTreasuryBalance => "Insufficient treasury balance",
            ContractError::InvalidAllocation => "Invalid allocation",
            ContractError::InvalidDistribution => "Invalid distribution",
            ContractError::TreasuryLocked => "Treasury is locked",

            // Slashing
            ContractError::ValidatorNotFound => "Validator not found",
            ContractError::InvalidSlashingAmount => "Invalid slashing amount",
            ContractError::SlashingAlreadyExecuted => "Slashing already executed",
            ContractError::SlashingPeriodNotActive => "Slashing period not active",
            ContractError::SlashingExceedsStake => "Slashing amount exceeds validator stake",
            ContractError::SlashingPercentTooHigh => {
                "Slashing percentage exceeds maximum allowed per event (10%)"
            }

            // Risk Pool
            ContractError::RiskPoolNotFound => "Risk pool not found",
            ContractError::InvalidRiskPoolState => "Invalid risk pool state",
            ContractError::InsufficientRiskPoolBalance => "Insufficient risk pool balance",
            ContractError::RiskPoolLocked => "Risk pool is locked",
            ContractError::InvalidReserveRatio => {
                "Reserve ratio must be between 20% and 100%"
            }
            ContractError::DepositBelowMinStake => {
                "Deposit amount is below the minimum stake requirement"
            }
            ContractError::WithdrawalExceedsBalance => {
                "Withdrawal amount exceeds provider's available balance"
            }

            // Cross-Chain
            ContractError::BridgeNotRegistered => "Bridge not registered",
            ContractError::ChainNotSupported => "Chain not supported",
            ContractError::MessageAlreadyProcessed => "Message already processed",
            ContractError::InsufficientConfirmations => "Insufficient confirmations",
            ContractError::AssetNotMapped => "Asset not mapped",
            ContractError::MessageExpired => "Cross-chain message expired",
            ContractError::InvalidMessageFormat => "Invalid message format",
            ContractError::BridgePaused => "Bridge is paused",
            ContractError::ValidatorAlreadyConfirmed => "Validator already confirmed",
            ContractError::CrossChainProposalNotFound => "Cross-chain proposal not found",
            ContractError::InvalidChainId => "Invalid chain ID",
            ContractError::NonceMismatch => "Nonce mismatch",

            // Input Validation
            ContractError::AmountMustBePositive => "Amount must be strictly positive (> 0)",
            ContractError::AmountOutOfBounds => "Amount is outside the allowed bounds",
            ContractError::InvalidPercentage => "Percentage must be between 0 and 100",
            ContractError::InvalidBasisPoints => "Basis points must be between 0 and 10 000",
            ContractError::TimestampNotFuture => "Timestamp must be in the future",
            ContractError::TimestampNotPast => "Timestamp must be in the past or present",
            ContractError::InvalidTimeRange => "Start time must be strictly before end time",
            ContractError::EmptyInput => "Input string or bytes cannot be empty",
            ContractError::InputTooLong => "Input exceeds maximum allowed length",
            ContractError::InputTooShort => "Input is shorter than the minimum required length",
            ContractError::InvalidPaginationParams => {
                "Pagination limit must be between 1 and 1 000"
            }
            ContractError::DuplicateAddress => "Both addresses in the pair must be different",
            ContractError::QuorumTooLow => "Quorum percentage must be at least 10%",
            ContractError::ThresholdTooLow => "Approval threshold must be greater than 50%",
        }
    }
}
