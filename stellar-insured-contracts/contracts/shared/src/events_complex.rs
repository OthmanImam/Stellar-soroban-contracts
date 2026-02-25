#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol, String, BytesN, Vec};

/// Structured event definitions for analytics, monitoring, and audit compliance
/// This module provides standardized event formats across all insurance contracts

/// Event categories for organized analytics and monitoring
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventCategory {
    /// Policy lifecycle events (issue, renew, cancel, expire)
    Policy,
    /// Claim processing events (submit, review, approve, reject, settle)
    Claim,
    /// Risk pool operations (deposit, withdraw, reserve)
    RiskPool,
    /// Governance actions (propose, vote, execute)
    Governance,
    /// Treasury operations (deposit, withdraw, fee collection)
    Treasury,
    /// Oracle data submissions and validations
    Oracle,
    /// Authorization and role changes
    Authorization,
    /// Emergency pause/unpause operations
    Emergency,
    /// Cross-chain bridge operations
    CrossChain,
    /// Compliance and audit events
    Compliance,
    /// Performance and monitoring metrics
    Monitoring,
    /// Token trading and AMM operations
    Trading,
    /// Identity verification and KYC
    Identity,
}

/// Event severity levels for monitoring and alerting
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Standardized event structure for all contract operations
#[contracttype]
#[derive(Clone, Debug)]
pub struct StructuredEvent {
    /// Unique event identifier (timestamp + contract-specific sequence)
    pub event_id: BytesN<32>,
    /// Event category for filtering and analytics
    pub category: EventCategory,
    /// Specific event type (e.g., "PolicyIssued", "ClaimSubmitted")
    pub event_type: String,
    /// Severity level for monitoring and alerting
    pub severity: EventSeverity,
    /// Address that triggered event
    pub actor: Address,
    /// Contract address that emitted event
    pub source_contract: Address,
    /// Timestamp when event occurred
    pub timestamp: u64,
    /// Primary subject identifier (policy_id, claim_id, etc.)
    pub subject_id: Option<u64>,
    /// Event-specific data payload
    pub data: Vec<String>,
    /// Optional related event IDs for correlation
    pub related_events: Option<Vec<BytesN<32>>>,
    /// Hash of off-chain data for audit trail
    pub data_hash: Option<BytesN<32>>,
    /// Additional metadata for analytics
    pub metadata: Vec<String>,
}

impl StructuredEvent {
    /// Create a new structured event
    pub fn new(
        env: &Env,
        category: EventCategory,
        event_type: String,
        severity: EventSeverity,
        actor: Address,
        source_contract: Address,
        subject_id: Option<u64>,
        data: Vec<String>,
    ) -> Self {
        let timestamp = env.ledger().timestamp();
        let event_id = Self::generate_event_id(env, &source_contract, timestamp, &event_type);
        
        Self {
            event_id,
            category,
            event_type,
            severity,
            actor,
            source_contract,
            timestamp,
            subject_id,
            data,
            related_events: None,
            data_hash: None,
            metadata: Vec::new(env),
        }
    }

    /// Generate unique event ID from contract address, timestamp, and event type
    fn generate_event_id(env: &Env, contract: &Address, timestamp: u64, event_type: &str) -> BytesN<32> {
        use soroban_sdk::crypto::sha256;
        
        let mut data = Vec::new(env);
        data.push_back(contract.clone());
        data.push_back(timestamp.into_val(env));
        data.push_back(String::from_str(env, event_type));
        
        // Convert Vec to bytes for hashing
        let data_bytes = env.to_bytes(&data);
        sha256(&data_bytes)
    }

    /// Add related events for correlation
    pub fn with_related_events(mut self, related_events: Vec<BytesN<32>>) -> Self {
        self.related_events = Some(related_events);
        self
    }

    /// Add data hash for audit trail
    pub fn with_data_hash(mut self, data_hash: BytesN<32>) -> Self {
        self.data_hash = Some(data_hash);
        self
    }

    /// Add metadata for analytics
    pub fn with_metadata(mut self, metadata: Vec<String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Publish structured event to ledger
    pub fn publish(self, env: &Env) {
        env.events().publish(
            (Symbol::new(env, "structured_event"), self.event_id),
            (
                self.category,
                self.event_type,
                self.severity,
                self.actor,
                self.source_contract,
                self.timestamp,
                self.subject_id,
                self.data,
                self.related_events,
                self.data_hash,
                self.metadata,
            ),
        );
    }
}

/// Event builder for convenient event creation and publishing
pub struct EventBuilder<'a> {
    env: &'a Env,
    category: EventCategory,
    event_type: String,
    severity: EventSeverity,
    actor: Address,
    source_contract: Address,
    subject_id: Option<u64>,
    data: Vec<String>,
    related_events: Option<Vec<BytesN<32>>>,
    data_hash: Option<BytesN<32>>,
    metadata: Vec<String>,
}

impl<'a> EventBuilder<'a> {
    /// Create a new event builder
    pub fn new(
        env: &'a Env,
        category: EventCategory,
        event_type: &str,
        severity: EventSeverity,
        actor: Address,
        source_contract: Address,
    ) -> Self {
        Self {
            env,
            category,
            event_type: String::from_str(env, event_type),
            severity,
            actor,
            source_contract,
            subject_id: None,
            data: Vec::new(env),
            related_events: None,
            data_hash: None,
            metadata: Vec::new(env),
        }
    }

    /// Set subject ID for event
    pub fn subject_id(mut self, subject_id: u64) -> Self {
        self.subject_id = Some(subject_id);
        self
    }

    /// Add a data field to event
    pub fn data(mut self, field: &str) -> Self {
        self.data.push_back(String::from_str(self.env, field));
        self
    }

    /// Add multiple data fields
    pub fn data_fields(mut self, fields: Vec<&str>) -> Self {
        for field in fields {
            self.data.push_back(String::from_str(self.env, field));
        }
        self
    }

    /// Set related events for correlation
    pub fn related_events(mut self, events: Vec<BytesN<32>>) -> Self {
        self.related_events = Some(events);
        self
    }

    /// Set data hash for audit trail
    pub fn data_hash(mut self, hash: BytesN<32>) -> Self {
        self.data_hash = Some(hash);
        self
    }

    /// Add metadata fields
    pub fn metadata(mut self, fields: Vec<&str>) -> Self {
        for field in fields {
            self.metadata.push_back(String::from_str(self.env, field));
        }
        self
    }

    /// Build and publish event
    pub fn publish(self) {
        let event = StructuredEvent::new(
            self.env,
            self.category,
            self.event_type,
            self.severity,
            self.actor,
            self.source_contract,
            self.subject_id,
            self.data,
        )
        .with_related_events(self.related_events.unwrap_or_default())
        .with_data_hash_option(self.data_hash)
        .with_metadata(self.metadata);

        event.publish(self.env);
    }
}

/// Helper trait to make data_hash optional in StructuredEvent
trait WithDataHashOption {
    fn with_data_hash_option(self, hash: Option<BytesN<32>>) -> Self;
}

impl WithDataHashOption for StructuredEvent {
    fn with_data_hash_option(self, hash: Option<BytesN<32>>) -> Self {
        if let Some(h) = hash {
            self.with_data_hash(h)
        } else {
            self
        }
    }
}

/// Convenience functions for common event patterns
pub mod events {
    use super::*;

    /// Policy issued event
    pub fn policy_issued(
        env: &Env,
        actor: Address,
        contract: Address,
        policy_id: u64,
        holder: Address,
        coverage_amount: i128,
        premium_amount: i128,
        duration_days: u32,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Policy,
            "PolicyIssued",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(policy_id)
        .data_fields(vec![
            &holder.to_string(),
            &coverage_amount.to_string(),
            &premium_amount.to_string(),
            &duration_days.to_string(),
        ])
        .publish();
    }

    /// Policy cancelled event
    pub fn policy_cancelled(
        env: &Env,
        actor: Address,
        contract: Address,
        policy_id: u64,
        reason: &str,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Policy,
            "PolicyCancelled",
            EventSeverity::Warning,
            actor,
            contract,
        )
        .subject_id(policy_id)
        .data(reason)
        .publish();
    }

    /// Policy expired event
    pub fn policy_expired(
        env: &Env,
        actor: Address,
        contract: Address,
        policy_id: u64,
        expiry_time: u64,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Policy,
            "PolicyExpired",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(policy_id)
        .data_fields(vec![&expiry_time.to_string()])
        .publish();
    }

    /// Claim submitted event
    pub fn claim_submitted(
        env: &Env,
        actor: Address,
        contract: Address,
        claim_id: u64,
        policy_id: u64,
        amount: i128,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Claim,
            "ClaimSubmitted",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(claim_id)
        .data_fields(vec![
            &policy_id.to_string(),
            &amount.to_string(),
        ])
        .publish();
    }

    /// Claim approved event
    pub fn claim_approved(
        env: &Env,
        actor: Address,
        contract: Address,
        claim_id: u64,
        policy_id: u64,
        amount: i128,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Claim,
            "ClaimApproved",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(claim_id)
        .data_fields(vec![
            &policy_id.to_string(),
            &amount.to_string(),
        ])
        .publish();
    }

    /// Claim rejected event
    pub fn claim_rejected(
        env: &Env,
        actor: Address,
        contract: Address,
        claim_id: u64,
        policy_id: u64,
        amount: i128,
        reason: &str,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Claim,
            "ClaimRejected",
            EventSeverity::Warning,
            actor,
            contract,
        )
        .subject_id(claim_id)
        .data_fields(vec![
            &policy_id.to_string(),
            &amount.to_string(),
            reason,
        ])
        .publish();
    }

    /// Claim settled event
    pub fn claim_settled(
        env: &Env,
        actor: Address,
        contract: Address,
        claim_id: u64,
        policy_id: u64,
        amount: i128,
        payout_asset: &str,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Claim,
            "ClaimSettled",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(claim_id)
        .data_fields(vec![
            &policy_id.to_string(),
            &amount.to_string(),
            payout_asset,
        ])
        .publish();
    }

    /// Risk pool deposit event
    pub fn risk_pool_deposit(
        env: &Env,
        actor: Address,
        contract: Address,
        provider: Address,
        amount: i128,
        new_balance: i128,
    ) {
        EventBuilder::new(
            env,
            EventCategory::RiskPool,
            "RiskPoolDeposit",
            EventSeverity::Info,
            actor,
            contract,
        )
        .data_fields(vec![
            &provider.to_string(),
            &amount.to_string(),
            &new_balance.to_string(),
        ])
        .publish();
    }

    /// Risk pool withdrawal event
    pub fn risk_pool_withdrawal(
        env: &Env,
        actor: Address,
        contract: Address,
        provider: Address,
        amount: i128,
        new_balance: i128,
    ) {
        EventBuilder::new(
            env,
            EventCategory::RiskPool,
            "RiskPoolWithdrawal",
            EventSeverity::Info,
            actor,
            contract,
        )
        .data_fields(vec![
            &provider.to_string(),
            &amount.to_string(),
            &new_balance.to_string(),
        ])
        .publish();
    }

    /// Governance proposal created event
    pub fn proposal_created(
        env: &Env,
        actor: Address,
        contract: Address,
        proposal_id: u64,
        proposal_type: &str,
        description: &str,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Governance,
            "ProposalCreated",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(proposal_id)
        .data_fields(vec![proposal_type, description])
        .publish();
    }

    /// Vote cast event
    pub fn vote_cast(
        env: &Env,
        actor: Address,
        contract: Address,
        proposal_id: u64,
        vote_type: &str,
        voting_power: u64,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Governance,
            "VoteCast",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(proposal_id)
        .data_fields(vec![vote_type, &voting_power.to_string()])
        .publish();
    }

    /// Emergency pause activated event
    pub fn emergency_pause_activated(
        env: &Env,
        actor: Address,
        contract: Address,
        reason: &str,
        duration_seconds: u64,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Emergency,
            "EmergencyPauseActivated",
            EventSeverity::Critical,
            actor,
            contract,
        )
        .data_fields(vec![reason, &duration_seconds.to_string()])
        .publish();
    }

    /// Authorization error event
    pub fn authorization_error(
        env: &Env,
        actor: Address,
        contract: Address,
        operation: &str,
        reason: &str,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Authorization,
            "AuthorizationError",
            EventSeverity::Error,
            actor,
            contract,
        )
        .data_fields(vec![operation, reason])
        .publish();
    }

    /// Treasury fee collected event
    pub fn fee_collected(
        env: &Env,
        actor: Address,
        contract: Address,
        fee_type: &str,
        amount: i128,
        asset: &str,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Treasury,
            "FeeCollected",
            EventSeverity::Info,
            actor,
            contract,
        )
        .data_fields(vec![fee_type, &amount.to_string(), asset])
        .publish();
    }

    /// Oracle data submitted event
    pub fn oracle_data_submitted(
        env: &Env,
        actor: Address,
        contract: Address,
        oracle_id: u64,
        data_type: &str,
        value: i128,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Oracle,
            "OracleDataSubmitted",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(oracle_id)
        .data_fields(vec![data_type, &value.to_string()])
        .publish();
    }

    /// Compliance audit event
    pub fn compliance_audit(
        env: &Env,
        actor: Address,
        contract: Address,
        audit_type: &str,
        result: &str,
        details: &str,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Compliance,
            "ComplianceAudit",
            EventSeverity::Info,
            actor,
            contract,
        )
        .data_fields(vec![audit_type, result, details])
        .publish();
    }

    /// Performance metric event
    pub fn performance_metric(
        env: &Env,
        actor: Address,
        contract: Address,
        operation: &str,
        gas_used: u64,
        execution_time_ms: u64,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Monitoring,
            "PerformanceMetric",
            EventSeverity::Info,
            actor,
            contract,
        )
        .data_fields(vec![
            operation,
            &gas_used.to_string(),
            &execution_time_ms.to_string(),
        ])
        .publish();
    }
}
