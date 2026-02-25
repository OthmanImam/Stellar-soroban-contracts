#![no_std]
//! # Shared Insurance Contracts Library
//!
//! A comprehensive library providing reusable types, errors, constants, and validation
//! helpers for all Stellar Insured Soroban contracts.
//!
//! ## Modules
//!
//! - `errors`     – Common error types used across contracts
//! - `types`      – Shared data types and enums (PolicyStatus, ClaimStatus, etc.)
//! - `constants`  – Configuration constants for validation and limits
//! - `validation` – Centralized, domain-specific validation helper functions
//!
//! ## Usage
//!
//! Import the shared library in your contract's Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! shared = { path = "shared" }
//! ```
//!
//! Then use it in your code:
//!
//! ```rust,ignore
//! use shared::errors::ContractError;
//! use shared::types::{PolicyStatus, ClaimStatus};
//! use shared::validation::{validate_policy_params, validate_claim_params};
//! use shared::constants::MIN_COVERAGE_AMOUNT;
//! ```

// pub mod errors;
pub mod types;
pub mod constants;
// pub mod validation;
pub mod versioning;
pub mod upgradeable;
// pub mod gas_optimization;
// pub mod emergency_pause;
pub mod events;
// pub mod audit_events;
// pub mod event_verification;

// Re-export commonly used types
// pub use errors::ContractError;
pub use types::{
    PolicyStatus, ClaimStatus, ProposalStatus, ProposalType, VoteType,
    RiskPoolStatus, ClaimEvidence, VoteRecord, OracleConfig, RiskMetrics,
    PolicyMetadata, ClaimMetadata, TreasuryAllocation, DataKey,
    CrossChainMessageStatus, CrossChainMessageType, BridgeStatus,
    // Governance staking types
    RewardConfig, StakeInfo, StakingPosition, StakingStats, VoteDelegation,
    // Privacy/ZKP types
    ZkProof, PrivacySettings, ConfidentialClaim, PrivatePolicyData,
    ZkVerificationResult, PrivacyProof, ComplianceRecord,
    // DID (Decentralized Identity) types
    DidDocument, VerificationMethod, PublicKeyJwk, DidService, ServiceProperty,
    IdentityVerification, KycRecord, ZkIdentityProof, DidResolutionResult,
    MetadataProperty,
};

pub use constants::{
    // Contract limits
    MAX_POLICY_DURATION_DAYS, MIN_COVERAGE_AMOUNT, MAX_COVERAGE_AMOUNT,
    MIN_PREMIUM_AMOUNT, MAX_PREMIUM_AMOUNT,
    // Time constants
    ONE_DAY_SECONDS, ONE_MONTH_SECONDS, ONE_YEAR_SECONDS,
    CLAIM_GRACE_PERIOD_SECONDS,
    // Governance constants
    DEFAULT_VOTING_PERIOD_SECONDS, PROPOSAL_EXPIRY_SECONDS,
    DEFAULT_MIN_QUORUM_PERCENT,
};

pub use versioning::{
    VersionManager, VersioningError, VersionInfo, VersionTransition,
    MigrationState, migration_state_to_u32, u32_to_migration_state,
};
pub use upgradeable::UpgradeableContract;
pub use events::{
    EventCategory, EventSeverity, StructuredEvent, EventBuilder,
    events::{
        policy_issued, claim_submitted, risk_pool_deposit,
    },
};
// Include test modules
#[cfg(test)]
mod simple_test;
