//! Comprehensive validation helper utilities for insurance contracts
//!
//! This module provides centralized, reusable validation functions that are used
//! across all contracts to ensure consistency, prevent malformed inputs, and
//! block potential exploit vectors. Every public contract function should call
//! the appropriate validators before processing any business logic.
//!
//! # Design Principles
//! - All validators return `Result<(), ContractError>` so they can be chained with `?`
//! - Each domain has its own granular error code for precise debugging
//! - Constants live in `constants.rs`; validators live here
//! - No panics – every bad path returns a typed error

// use crate::errors::ContractError;
use soroban_sdk::{Address, Bytes, BytesN, Env, String};

// ============================================================
// ===== ADDRESS VALIDATION ===================================
// ============================================================

/// Validate that an address is valid.
///
/// Soroban SDK already ensures deserialized `Address` values are structurally
/// valid, so this acts as a documentation/contract-boundary guard. Future
/// versions may add allow-list or format checks here.
pub fn validate_address(_env: &Env, _address: &Address) -> Result<(), ContractError> {
    Ok(())
}

/// Validate multiple addresses in one call.
pub fn validate_addresses(env: &Env, addresses: &[Address]) -> Result<(), ContractError> {
    for address in addresses {
        validate_address(env, address)?;
    }
    Ok(())
}

/// Validate that two addresses are different (e.g., sender ≠ recipient).
pub fn validate_addresses_different(
    addr1: &Address,
    addr2: &Address,
) -> Result<(), ContractError> {
    if addr1 == addr2 {
        return Err(ContractError::DuplicateAddress);
    }
    Ok(())
}

/// Validate that `address` is not the zero-value placeholder.
///
/// In Soroban there is no literal zero address, but callers sometimes pass
/// freshly-generated test addresses. This guard is a no-op for now but
/// provides a single place to add stricter checks later.
pub fn validate_non_zero_address(_env: &Env, _address: &Address) -> Result<(), ContractError> {
    Ok(())
}

// ============================================================
// ===== AMOUNT VALIDATION ====================================
// ============================================================

/// Validate that `amount` is strictly positive (> 0).
pub fn validate_positive_amount(amount: i128) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::AmountMustBePositive);
    }
    Ok(())
}

/// Validate that `amount` is non-negative (≥ 0).
pub fn validate_non_negative_amount(amount: i128) -> Result<(), ContractError> {
    if amount < 0 {
        return Err(ContractError::AmountMustBePositive);
    }
    Ok(())
}

/// Validate that `amount` falls within `[min, max]` (inclusive).
pub fn validate_amount_in_bounds(amount: i128, min: i128, max: i128) -> Result<(), ContractError> {
    if amount < min || amount > max {
        return Err(ContractError::AmountOutOfBounds);
    }
    Ok(())
}

/// Validate a policy coverage amount is within the protocol's allowed range.
///
/// Bounds (stroops, 7 decimal places on Stellar):
/// - Min: 1 XLM  (1_000_000_0 stroops)
/// - Max: 1 000 000 XLM (1_000_000_000_000_0 stroops)
pub fn validate_coverage_amount(amount: i128) -> Result<(), ContractError> {
    const MIN_COVERAGE: i128 = 10_000_000;       // 1 XLM
    const MAX_COVERAGE: i128 = 10_000_000_000_000_000; // 1 000 000 XLM
    if amount < MIN_COVERAGE || amount > MAX_COVERAGE {
        return Err(ContractError::InvalidCoverageAmount);
    }
    Ok(())
}

/// Validate a premium amount is within the protocol's allowed range.
pub fn validate_premium_amount(amount: i128) -> Result<(), ContractError> {
    const MIN_PREMIUM: i128 = 1_000_000;          // 0.1 XLM
    const MAX_PREMIUM: i128 = 1_000_000_000_000_000; // 100 000 XLM
    if amount < MIN_PREMIUM || amount > MAX_PREMIUM {
        return Err(ContractError::InvalidPremiumAmount);
    }
    Ok(())
}

/// Validate a claim amount.
///
/// - Must be positive.
/// - Must not exceed the policy's coverage amount.
pub fn validate_claim_amount(
    claim_amount: i128,
    coverage_amount: i128,
) -> Result<(), ContractError> {
    if claim_amount <= 0 {
        return Err(ContractError::AmountMustBePositive);
    }
    if claim_amount > coverage_amount {
        return Err(ContractError::ClaimExceedsCoverage);
    }
    Ok(())
}

/// Validate a risk pool deposit amount.
///
/// - Must be positive.
/// - Must be ≥ `min_stake` configured for the pool.
pub fn validate_deposit_amount(amount: i128, min_stake: i128) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::AmountMustBePositive);
    }
    if amount < min_stake {
        return Err(ContractError::DepositBelowMinStake);
    }
    Ok(())
}

/// Validate a risk pool withdrawal amount.
///
/// - Must be positive.
/// - Must not exceed the provider's available balance (net of locked amounts).
pub fn validate_withdrawal_amount(
    amount: i128,
    available_balance: i128,
) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::AmountMustBePositive);
    }
    if amount > available_balance {
        return Err(ContractError::WithdrawalExceedsBalance);
    }
    Ok(())
}

/// Validate a treasury allocation amount.
///
/// - Must be positive.
/// - Must not exceed the treasury's available balance.
pub fn validate_allocation_amount(
    amount: i128,
    treasury_balance: i128,
) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::AmountMustBePositive);
    }
    if amount > treasury_balance {
        return Err(ContractError::InsufficientFunds);
    }
    Ok(())
}

/// Validate that there are sufficient funds.
pub fn validate_sufficient_funds(balance: i128, required: i128) -> Result<(), ContractError> {
    if balance < required {
        return Err(ContractError::InsufficientFunds);
    }
    Ok(())
}

// ============================================================
// ===== TIME / DURATION VALIDATION ===========================
// ============================================================

/// Validate that a timestamp is strictly in the future.
pub fn validate_future_timestamp(current_time: u64, timestamp: u64) -> Result<(), ContractError> {
    if timestamp <= current_time {
        return Err(ContractError::TimestampNotFuture);
    }
    Ok(())
}

/// Validate that a timestamp is in the past or equal to current time.
pub fn validate_past_timestamp(current_time: u64, timestamp: u64) -> Result<(), ContractError> {
    if timestamp > current_time {
        return Err(ContractError::TimestampNotPast);
    }
    Ok(())
}

/// Validate that a time range is valid (start strictly < end).
pub fn validate_time_range(start_time: u64, end_time: u64) -> Result<(), ContractError> {
    if start_time >= end_time {
        return Err(ContractError::InvalidTimeRange);
    }
    Ok(())
}

/// Validate policy duration in days falls within allowed bounds.
///
/// Range: 1 day – 1825 days (≈5 years).
pub fn validate_duration_days(duration_days: u32) -> Result<(), ContractError> {
    const MIN_DURATION: u32 = 1;
    const MAX_DURATION: u32 = 1825; // ~5 years
    if duration_days < MIN_DURATION || duration_days > MAX_DURATION {
        return Err(ContractError::InvalidDuration);
    }
    Ok(())
}

/// Validate a governance voting duration in seconds.
///
/// Range: 1 hour – 30 days.
pub fn validate_voting_duration(duration_secs: u64) -> Result<(), ContractError> {
    const MIN_VOTING_SECS: u64 = 3_600;          // 1 hour
    const MAX_VOTING_SECS: u64 = 30 * 86_400;    // 30 days
    if duration_secs < MIN_VOTING_SECS || duration_secs > MAX_VOTING_SECS {
        return Err(ContractError::InvalidVotingDuration);
    }
    Ok(())
}

// ============================================================
// ===== PERCENTAGE / BASIS POINTS VALIDATION =================
// ============================================================

/// Validate that a value is a valid percentage (0–100, inclusive).
pub fn validate_percentage(percent: u32) -> Result<(), ContractError> {
    if percent > 100 {
        return Err(ContractError::InvalidPercentage);
    }
    Ok(())
}

/// Validate that a value is within basis-points range (0–10 000, inclusive).
pub fn validate_basis_points(bps: u32) -> Result<(), ContractError> {
    const MAX_BPS: u32 = 10_000;
    if bps > MAX_BPS {
        return Err(ContractError::InvalidBasisPoints);
    }
    Ok(())
}

/// Validate that oracle price deviation is within acceptable bounds.
///
/// Maximum deviation: 500 bps (5 %).
pub fn validate_oracle_deviation(deviation_bps: u32) -> Result<(), ContractError> {
    validate_basis_points(deviation_bps)?;
    const MAX_DEVIATION: u32 = 500;
    if deviation_bps > MAX_DEVIATION {
        return Err(ContractError::OracleValidationFailed);
    }
    Ok(())
}

/// Validate a governance quorum percentage.
///
/// Minimum acceptable quorum: 10 %.
pub fn validate_quorum_percent(percent: u32) -> Result<(), ContractError> {
    validate_percentage(percent)?;
    const MIN_QUORUM: u32 = 10;
    if percent < MIN_QUORUM {
        return Err(ContractError::QuorumTooLow);
    }
    Ok(())
}

/// Validate a governance approval threshold.
///
/// Threshold must be > 50 % (simple majority).
pub fn validate_voting_threshold(percent: u32) -> Result<(), ContractError> {
    validate_percentage(percent)?;
    if percent <= 50 {
        return Err(ContractError::ThresholdTooLow);
    }
    Ok(())
}

/// Validate a reserve ratio is within safe operating bounds.
///
/// Range: 20 %–100 %.
pub fn validate_reserve_ratio(ratio_percent: u32) -> Result<(), ContractError> {
    const MIN_RATIO: u32 = 20;
    const MAX_RATIO: u32 = 100;
    if ratio_percent < MIN_RATIO || ratio_percent > MAX_RATIO {
        return Err(ContractError::InvalidReserveRatio);
    }
    Ok(())
}

// ============================================================
// ===== CONTRACT STATE VALIDATION ============================
// ============================================================

/// Validate that the contract is not paused.
pub fn validate_not_paused(is_paused: bool) -> Result<(), ContractError> {
    if is_paused {
        return Err(ContractError::Paused);
    }
    Ok(())
}

/// Validate that the contract is initialized.
pub fn validate_initialized(is_initialized: bool) -> Result<(), ContractError> {
    if !is_initialized {
        return Err(ContractError::NotInitialized);
    }
    Ok(())
}

/// Validate that the contract has not been initialized yet.
pub fn validate_not_initialized(is_initialized: bool) -> Result<(), ContractError> {
    if is_initialized {
        return Err(ContractError::AlreadyInitialized);
    }
    Ok(())
}

// ============================================================
// ===== EVIDENCE & HASH VALIDATION ===========================
// ============================================================

/// Validate a 32-byte evidence hash (SHA-256).
///
/// Rejects all-zero hashes as they are almost certainly placeholder values.
pub fn validate_evidence_hash(hash: &BytesN<32>) -> Result<(), ContractError> {
    // Reject all-zero hash
    let zero: BytesN<32> = BytesN::from_array(hash.env(), &[0u8; 32]);
    if hash == &zero {
        return Err(ContractError::InvalidEvidenceHash);
    }
    Ok(())
}

/// Validate raw bytes are non-empty and within `max_len`.
pub fn validate_bytes_length(
    data: &Bytes,
    max_len: u32,
) -> Result<(), ContractError> {
    if data.len() == 0 {
        return Err(ContractError::EmptyInput);
    }
    if data.len() > max_len {
        return Err(ContractError::InputTooLong);
    }
    Ok(())
}

/// Validate a `soroban_sdk::String` is non-empty and within `max_len` characters.
pub fn validate_string_length(s: &String, max_len: u32) -> Result<(), ContractError> {
    if s.len() == 0 {
        return Err(ContractError::EmptyInput);
    }
    if s.len() > max_len {
        return Err(ContractError::InputTooLong);
    }
    Ok(())
}

/// Validate that a metadata/description string is within the standard protocol limit.
///
/// Maximum: 1 024 UTF-8 encoded bytes.
pub fn validate_metadata(s: &String) -> Result<(), ContractError> {
    const MAX_METADATA_LEN: u32 = 1_024;
    validate_string_length(s, MAX_METADATA_LEN)
}

/// Validate a claim description is within the standard protocol limit.
///
/// Maximum: 2 048 UTF-8 encoded bytes.
pub fn validate_description(s: &String) -> Result<(), ContractError> {
    const MAX_DESCRIPTION_LEN: u32 = 2_048;
    validate_string_length(s, MAX_DESCRIPTION_LEN)
}

/// Validate a proposal title is within the standard protocol limit.
///
/// Minimum: 3 characters; Maximum: 200 characters.
pub fn validate_proposal_title(title: &String) -> Result<(), ContractError> {
    const MIN_TITLE_LEN: u32 = 3;
    const MAX_TITLE_LEN: u32 = 200;
    if title.len() < MIN_TITLE_LEN {
        return Err(ContractError::InputTooShort);
    }
    if title.len() > MAX_TITLE_LEN {
        return Err(ContractError::InputTooLong);
    }
    Ok(())
}

// ============================================================
// ===== GOVERNANCE PROPOSAL VALIDATION =======================
// ============================================================

/// Validate all parameters supplied to create a governance proposal.
///
/// Checks: title length, description length, voting duration.
pub fn validate_proposal_params(
    title: &String,
    description: &String,
    voting_duration_secs: u64,
) -> Result<(), ContractError> {
    validate_proposal_title(title)?;
    validate_description(description)?;
    validate_voting_duration(voting_duration_secs)?;
    Ok(())
}

// ============================================================
// ===== ORACLE VALIDATION ====================================
// ============================================================

/// Validate the number of oracle submissions is within operating bounds.
///
/// Range: 1–100 submissions.
pub fn validate_oracle_submissions(count: u32) -> Result<(), ContractError> {
    const MIN_SUBMISSIONS: u32 = 1;
    const MAX_SUBMISSIONS: u32 = 100;
    if count < MIN_SUBMISSIONS || count > MAX_SUBMISSIONS {
        return Err(ContractError::InsufficientOracleSubmissions);
    }
    Ok(())
}

/// Validate oracle data is not older than `max_age_seconds`.
pub fn validate_oracle_data_age(
    current_time: u64,
    data_time: u64,
    max_age_seconds: u64,
) -> Result<(), ContractError> {
    if data_time > current_time {
        return Err(ContractError::InvalidInput);
    }
    let age = current_time - data_time;
    if age > max_age_seconds {
        return Err(ContractError::OracleDataStale);
    }
    Ok(())
}

/// Validate the minimum oracle submissions configuration value.
pub fn validate_min_oracle_submissions(min_submissions: u32) -> Result<(), ContractError> {
    if min_submissions == 0 {
        return Err(ContractError::InvalidInput);
    }
    if min_submissions > 100 {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

// ============================================================
// ===== SLASHING VALIDATION ==================================
// ============================================================

/// Validate a slashing amount.
///
/// - Must be positive.
/// - Must not exceed `max_slashable` (the validator's total stake).
pub fn validate_slashing_amount(
    amount: i128,
    max_slashable: i128,
) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::AmountMustBePositive);
    }
    if amount > max_slashable {
        return Err(ContractError::SlashingExceedsStake);
    }
    Ok(())
}

/// Validate that a slashing percentage is within the maximum allowed.
///
/// Maximum: 10 % of stake per slashing event.
pub fn validate_slashing_percent(percent: u32) -> Result<(), ContractError> {
    validate_percentage(percent)?;
    const MAX_SLASH_PERCENT: u32 = 10;
    if percent > MAX_SLASH_PERCENT {
        return Err(ContractError::SlashingPercentTooHigh);
    }
    Ok(())
}

// ============================================================
// ===== PAGINATION VALIDATION ================================
// ============================================================

/// Validate pagination parameters.
///
/// - `limit` must be ≥ 1 and ≤ 1 000.
pub fn validate_pagination(limit: u32) -> Result<(), ContractError> {
    const MAX_PAGE_SIZE: u32 = 1_000;
    if limit == 0 || limit > MAX_PAGE_SIZE {
        return Err(ContractError::InvalidPaginationParams);
    }
    Ok(())
}

// ============================================================
// ===== SAFE ARITHMETIC =====================================
// ============================================================

/// Safely add two amounts, returning `ContractError::Overflow` on overflow.
pub fn safe_add(a: i128, b: i128) -> Result<i128, ContractError> {
    a.checked_add(b).ok_or(ContractError::Overflow)
}

/// Safely subtract two amounts, returning `ContractError::Underflow` on underflow.
pub fn safe_sub(a: i128, b: i128) -> Result<i128, ContractError> {
    a.checked_sub(b).ok_or(ContractError::Underflow)
}

/// Safely multiply two amounts, returning `ContractError::Overflow` on overflow.
pub fn safe_mul(a: i128, b: i128) -> Result<i128, ContractError> {
    a.checked_mul(b).ok_or(ContractError::Overflow)
}

/// Safely divide two amounts.
///
/// Returns `ContractError::DivisionByZero` if `b == 0`.
pub fn safe_div(a: i128, b: i128) -> Result<i128, ContractError> {
    if b == 0 {
        return Err(ContractError::DivisionByZero);
    }
    a.checked_div(b).ok_or(ContractError::Overflow)
}

// ============================================================
// ===== BATCH VALIDATION =====================================
// ============================================================

/// Validate multiple boolean conditions at once, short-circuiting on the first failure.
///
/// # Arguments
/// * `conditions` – slice of `(is_valid: bool, error: ContractError)` pairs
///
/// # Returns
/// `Ok(())` if all conditions are true, otherwise the first failing error.
pub fn validate_all(conditions: &[(bool, ContractError)]) -> Result<(), ContractError> {
    for &(is_valid, error) in conditions {
        if !is_valid {
            return Err(error);
        }
    }
    Ok(())
}

// ============================================================
// ===== CALCULATION HELPERS ==================================
// ============================================================

/// Calculate `percent` % of `amount`.
///
/// Validates `percent` is 0–100; uses checked arithmetic.
pub fn calculate_percentage(amount: i128, percent: u32) -> Result<i128, ContractError> {
    validate_percentage(percent)?;
    if percent == 0 {
        return Ok(0);
    }
    safe_mul(amount, percent as i128)?
        .checked_div(100)
        .ok_or(ContractError::Overflow)
}

/// Calculate `bps` basis points of `amount`.
///
/// Validates `bps` is 0–10 000; uses checked arithmetic.
pub fn calculate_basis_points(amount: i128, bps: u32) -> Result<i128, ContractError> {
    validate_basis_points(bps)?;
    if bps == 0 {
        return Ok(0);
    }
    safe_mul(amount, bps as i128)?
        .checked_div(10_000)
        .ok_or(ContractError::Overflow)
}

/// Calculate reserve ratio as a percentage.
pub fn calculate_reserve_ratio(reserve: i128, total_value: i128) -> Result<u32, ContractError> {
    validate_positive_amount(total_value)?;
    if reserve == 0 {
        return Ok(0);
    }
    let ratio = safe_div(safe_mul(reserve, 100)?, total_value)? as u32;
    Ok(ratio)
}

// ============================================================
// ===== POLICY-SPECIFIC COMPLETE VALIDATION ==================
// ============================================================

/// Full parameter validation for creating a new policy.
///
/// Enforces all business-rule constraints in one call — call this at the top
/// of `issue_policy` before any storage writes.
pub fn validate_policy_params(
    holder: &Address,
    coverage_amount: i128,
    premium_amount: i128,
    duration_days: u32,
    env: &Env,
) -> Result<(), ContractError> {
    validate_address(env, holder)?;
    validate_coverage_amount(coverage_amount)?;
    validate_premium_amount(premium_amount)?;
    validate_duration_days(duration_days)?;
    // Premium must not exceed coverage (sanity check)
    if premium_amount >= coverage_amount {
        return Err(ContractError::PremiumExceedsCoverage);
    }
    Ok(())
}

// ============================================================
// ===== CLAIM-SPECIFIC COMPLETE VALIDATION ===================
// ============================================================

/// Full parameter validation for submitting a claim.
///
/// Enforces all business-rule constraints in one call — call this at the top
/// of `submit_claim` before any storage writes.
pub fn validate_claim_params(
    claimant: &Address,
    claim_amount: i128,
    coverage_amount: i128,
    env: &Env,
) -> Result<(), ContractError> {
    validate_address(env, claimant)?;
    validate_claim_amount(claim_amount, coverage_amount)?;
    Ok(())
}

// ============================================================
// ===== RISK POOL COMPLETE VALIDATION ========================
// ============================================================

/// Full parameter validation for initializing a risk pool.
pub fn validate_risk_pool_init_params(
    admin: &Address,
    token: &Address,
    min_provider_stake: i128,
    env: &Env,
) -> Result<(), ContractError> {
    validate_address(env, admin)?;
    validate_address(env, token)?;
    validate_addresses_different(admin, token)?;
    validate_positive_amount(min_provider_stake)?;
    Ok(())
}
