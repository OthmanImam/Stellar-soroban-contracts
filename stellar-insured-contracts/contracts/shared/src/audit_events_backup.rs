#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol, String, BytesN, Vec, IntoVal};

use super::events::{EventCategory, EventSeverity, EventBuilder};

/// Audit-specific event types for compliance and regulatory requirements
/// These events provide detailed audit trails for all critical operations

/// Audit event subcategories for granular filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditSubcategory {
    /// Policy-related audit events
    PolicyOperation,
    /// Claim processing audit events
    ClaimProcessing,
    /// Financial operations (deposits, withdrawals, payouts)
    FinancialOperation,
    /// Access control and authorization
    AccessControl,
    /// Configuration changes
    ConfigurationChange,
    /// Emergency operations
    EmergencyOperation,
    /// Cross-contract communications
    CrossContractCall,
    /// Data modifications
    DataModification,
    /// Compliance checks
    ComplianceCheck,
    /// System operations
    SystemOperation,
}

/// Audit event severity levels for compliance reporting
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditSeverity {
    /// Informational audit event (normal operation)
    Info,
    /// Warning event (potential issue detected)
    Warning,
    /// Error event (operation failed)
    Error,
    /// Critical event (security or compliance breach)
    Critical,
}

/// Comprehensive audit event structure for regulatory compliance
#[contracttype]
#[derive(Clone, Debug)]
pub struct AuditEvent {
    /// Unique audit event identifier
    pub audit_id: BytesN<32>,
    /// Main event category
    pub category: EventCategory,
    /// Specific audit subcategory
    pub subcategory: AuditSubcategory,
    /// Audit severity level
    pub severity: AuditSeverity,
    /// User or system that performed the action
    pub actor: Address,
    /// Contract that generated the audit event
    pub source_contract: Address,
    /// Timestamp of the event
    pub timestamp: u64,
    /// Primary subject identifier (policy_id, claim_id, etc.)
    pub subject_id: Option<u64>,
    /// Action performed
    pub action: String,
    /// Detailed description of the event
    pub description: String,
    /// Previous state before the action (if applicable)
    pub previous_state: Option<String>,
    /// New state after the action (if applicable)
    pub new_state: Option<String>,
    /// Amount involved (if applicable)
    pub amount: Option<i128>,
    /// Asset involved (if applicable)
    pub asset: Option<String>,
    /// Related audit events for correlation
    pub related_events: Option<Vec<BytesN<32>>>,
    /// Hash of off-chain data for verification
    pub data_hash: Option<BytesN<32>>,
    /// Compliance tags for regulatory filtering
    pub compliance_tags: Vec<String>,
    /// Additional metadata for analysis
    pub metadata: Vec<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        env: &Env,
        category: EventCategory,
        subcategory: AuditSubcategory,
        severity: AuditSeverity,
        actor: Address,
        source_contract: Address,
        action: &str,
        description: &str,
    ) -> Self {
        let timestamp = env.ledger().timestamp();
        let audit_id = Self::generate_audit_id(env, &source_contract, timestamp, action);
        
        Self {
            audit_id,
            category,
            subcategory,
            severity,
            actor,
            source_contract,
            timestamp,
            subject_id: None,
            action: String::from_str(env, action),
            description: String::from_str(env, description),
            previous_state: None,
            new_state: None,
            amount: None,
            asset: None,
            related_events: None,
            data_hash: None,
            compliance_tags: Vec::new(env),
            metadata: Vec::new(env),
        }
    }

    /// Generate unique audit ID
    fn generate_audit_id(env: &Env, contract: &Address, timestamp: u64, action: &str) -> BytesN<32> {
        use soroban_sdk::crypto::sha256;
        
        let mut data = Vec::new(env);
        data.push_back(contract.clone());
        data.push_back(timestamp.into_val(env));
        data.push_back(String::from_str(env, action));
        
        // Convert Vec to bytes for hashing
        let data_bytes = env.to_bytes(&data);
        sha256(&data_bytes)
    }

    /// Set subject ID
    pub fn subject_id(mut self, subject_id: u64) -> Self {
        self.subject_id = Some(subject_id);
        self
    }

    /// Set state transition
    pub fn state_transition(mut self, previous: String, new: String) -> Self {
        self.previous_state = Some(previous);
        self.new_state = Some(new);
        self
    }

    /// Set amount and asset
    pub fn amount(mut self, amount: i128, asset: String) -> Self {
        self.amount = Some(amount);
        self.asset = Some(asset);
        self
    }

    /// Add related events
    pub fn related_events(mut self, events: Vec<BytesN<32>>) -> Self {
        self.related_events = Some(events);
        self
    }

    /// Add data hash
    pub fn data_hash(mut self, hash: BytesN<32>) -> Self {
        self.data_hash = Some(hash);
        self
    }

    /// Add compliance tags
    pub fn compliance_tags(mut self, tags: Vec<String>) -> Self {
        self.compliance_tags = tags;
        self
    }

    /// Add metadata
    pub fn metadata(mut self, metadata: Vec<String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Publish audit event
    pub fn publish(self, env: &Env) {
        // Simplified event publishing with basic data
        env.events().publish(
            (Symbol::new(env, "audit_event"), self.audit_id),
            (
                self.action,
                self.description,
                self.actor,
                self.timestamp,
            ),
        );
    }
}

/// Builder for creating audit events with fluent interface
pub struct AuditEventBuilder<'a> {
    env: &'a Env,
    category: EventCategory,
    subcategory: AuditSubcategory,
    severity: AuditSeverity,
    actor: Address,
    source_contract: Address,
    action: String,
    description: String,
    subject_id: Option<u64>,
    previous_state: Option<String>,
    new_state: Option<String>,
    amount: Option<i128>,
    asset: Option<String>,
    related_events: Option<Vec<BytesN<32>>>,
    data_hash: Option<BytesN<32>>,
    compliance_tags: Vec<String>,
    metadata: Vec<String>,
}

impl<'a> AuditEventBuilder<'a> {
    /// Create a new audit event builder
    pub fn new(
        env: &'a Env,
        category: EventCategory,
        subcategory: AuditSubcategory,
        severity: AuditSeverity,
        actor: Address,
        source_contract: Address,
        action: &str,
        description: &str,
    ) -> Self {
        Self {
            env,
            category,
            subcategory,
            severity,
            actor,
            source_contract,
            action: String::from_str(env, action),
            description: String::from_str(env, description),
            subject_id: None,
            previous_state: None,
            new_state: None,
            amount: None,
            asset: None,
            related_events: None,
            data_hash: None,
            compliance_tags: Vec::new(env),
            metadata: Vec::new(env),
        }
    }

    /// Set subject ID
    pub fn subject_id(mut self, subject_id: u64) -> Self {
        self.subject_id = Some(subject_id);
        self
    }

    /// Set state transition
    pub fn state_transition(mut self, previous: &str, new: &str) -> Self {
        self.previous_state = Some(String::from_str(self.env, previous));
        self.new_state = Some(String::from_str(self.env, new));
        self
    }

    /// Set amount and asset
    pub fn amount(mut self, amount: i128, asset: &str) -> Self {
        self.amount = Some(amount);
        self.asset = Some(String::from_str(self.env, asset));
        self
    }

    /// Add related events
    pub fn related_events(mut self, events: Vec<BytesN<32>>) -> Self {
        self.related_events = Some(events);
        self
    }

    /// Add data hash
    pub fn data_hash(mut self, hash: BytesN<32>) -> Self {
        self.data_hash = Some(hash);
        self
    }

    /// Add compliance tags
    pub fn compliance_tags(mut self, tags: Vec<&str>) -> Self {
        for tag in tags.iter() {
            self.compliance_tags.push_back(String::from_str(self.env, tag));
        }
        self
    }

    /// Add metadata
    pub fn metadata(mut self, metadata: Vec<&str>) -> Self {
        for data in metadata.iter() {
            self.metadata.push_back(String::from_str(self.env, data));
        }
        self
    }

    /// Build and publish the audit event
    pub fn publish(self) {
        let event = AuditEvent::new(
            self.env,
            self.category,
            self.subcategory,
            self.severity,
            self.actor,
            self.source_contract,
            self.action,
            self.description,
        )
        .subject_id_option(self.subject_id)
        .state_transition_option(
            self.previous_state,
            self.new_state,
        )
        .amount_option(self.amount, self.asset)
        .related_events_option(self.related_events.unwrap_or_default())
        .data_hash_option(self.data_hash)
        .compliance_tags(self.compliance_tags)
        .metadata(self.metadata);

        event.publish(self.env);
    }
}

/// Helper trait for optional setters
trait AuditEventExt {
    fn subject_id_option(self, subject_id: Option<u64>) -> Self;
    fn state_transition_option(self, previous: Option<String>, new: Option<String>) -> Self;
    fn amount_option(self, amount: Option<i128>, asset: Option<String>) -> Self;
    fn related_events_option(self, events: Vec<BytesN<32>>) -> Self;
    fn data_hash_option(self, hash: Option<BytesN<32>>) -> Self;
}

impl AuditEventExt for AuditEvent {
    fn subject_id_option(mut self, subject_id: Option<u64>) -> Self {
        if let Some(id) = subject_id {
            self.subject_id = Some(id);
        }
        self
    }

    fn state_transition_option(mut self, previous: Option<String>, new: Option<String>) -> Self {
        self.previous_state = previous;
        self.new_state = new;
        self
    }

    fn amount_option(mut self, amount: Option<i128>, asset: Option<String>) -> Self {
        self.amount = amount;
        self.asset = asset;
    }

    fn related_events_option(mut self, events: Vec<BytesN<32>>) -> Self {
        if !events.is_empty() {
            self.related_events = Some(events);
        }
        self
    }

    fn data_hash_option(mut self, hash: Option<BytesN<32>>) -> Self {
        self.data_hash = hash;
        self
    }
}

/// Convenience functions for common audit events
pub mod audit_events {
    use super::*;

    /// Policy issued audit event
    pub fn policy_issued(
        env: &Env,
        actor: Address,
        contract: Address,
        policy_id: u64,
        holder: Address,
        coverage_amount: i128,
        premium_amount: i128,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Policy,
            AuditSubcategory::PolicyOperation,
            AuditSeverity::Info,
            actor,
            contract,
            "policy_issued",
            "New insurance policy issued to holder",
        )
        .subject_id(policy_id)
        .amount(coverage_amount, "XLM")
        .compliance_tags(vec!["policy_creation", "financial_transaction"])
        .metadata(vec![
            &holder.to_string(),
            &premium_amount.to_string(),
        ])
        .publish();
    }

    /// Claim submitted audit event
    pub fn claim_submitted(
        env: &Env,
        actor: Address,
        contract: Address,
        claim_id: u64,
        policy_id: u64,
        amount: i128,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Claim,
            AuditSubcategory::ClaimProcessing,
            AuditSeverity::Info,
            actor,
            contract,
            "claim_submitted",
            "Insurance claim submitted for processing",
        )
        .subject_id(claim_id)
        .amount(amount, "XLM")
        .compliance_tags(vec!["claim_creation", "financial_transaction"])
        .metadata(vec![&policy_id.to_string()])
        .publish();
    }

    /// Claim approved audit event
    pub fn claim_approved(
        env: &Env,
        actor: Address,
        contract: Address,
        claim_id: u64,
        policy_id: u64,
        amount: i128,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Claim,
            AuditSubcategory::ClaimProcessing,
            AuditSeverity::Info,
            actor,
            contract,
            "claim_approved",
            "Insurance claim approved for payout",
        )
        .subject_id(claim_id)
        .state_transition("under_review", "approved")
        .amount(amount, "XLM")
        .compliance_tags(vec!["claim_approval", "financial_transaction"])
        .metadata(vec![&policy_id.to_string()])
        .publish();
    }

    /// Claim rejected audit event
    pub fn claim_rejected(
        env: &Env,
        actor: Address,
        contract: Address,
        claim_id: u64,
        policy_id: u64,
        amount: i128,
        reason: &str,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Claim,
            AuditSubcategory::ClaimProcessing,
            AuditSeverity::Warning,
            actor,
            contract,
            "claim_rejected",
            "Insurance claim rejected",
        )
        .subject_id(claim_id)
        .state_transition("under_review", "rejected")
        .amount(amount, "XLM")
        .compliance_tags(vec!["claim_rejection", "financial_decision"])
        .metadata(vec![&policy_id.to_string(), reason])
        .publish();
    }

    /// Claim settled audit event
    pub fn claim_settled(
        env: &Env,
        actor: Address,
        contract: Address,
        claim_id: u64,
        policy_id: u64,
        amount: i128,
        payout_asset: &str,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Claim,
            AuditSubcategory::FinancialOperation,
            AuditSeverity::Info,
            actor,
            contract,
            "claim_settled",
            "Insurance claim paid out to claimant",
        )
        .subject_id(claim_id)
        .state_transition("approved", "settled")
        .amount(amount, payout_asset)
        .compliance_tags(vec!["claim_payout", "financial_transaction"])
        .metadata(vec![&policy_id.to_string()])
        .publish();
    }

    /// Risk pool deposit audit event
    pub fn risk_pool_deposit(
        env: &Env,
        actor: Address,
        contract: Address,
        provider: Address,
        amount: i128,
        new_balance: i128,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::RiskPool,
            AuditSubcategory::FinancialOperation,
            AuditSeverity::Info,
            actor,
            contract,
            "risk_pool_deposit",
            "Liquidity deposited into risk pool",
        )
        .amount(amount, "XLM")
        .compliance_tags(vec!["liquidity_deposit", "financial_transaction"])
        .metadata(vec![
            &provider.to_string(),
            &new_balance.to_string(),
        ])
        .publish();
    }

    /// Risk pool withdrawal audit event
    pub fn risk_pool_withdrawal(
        env: &Env,
        actor: Address,
        contract: Address,
        provider: Address,
        amount: i128,
        new_balance: i128,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::RiskPool,
            AuditSubcategory::FinancialOperation,
            AuditSeverity::Info,
            actor,
            contract,
            "risk_pool_withdrawal",
            "Liquidity withdrawn from risk pool",
        )
        .amount(amount, "XLM")
        .compliance_tags(vec!["liquidity_withdrawal", "financial_transaction"])
        .metadata(vec![
            &provider.to_string(),
            &new_balance.to_string(),
        ])
        .publish();
    }

    /// Authorization success audit event
    pub fn authorization_success(
        env: &Env,
        actor: Address,
        contract: Address,
        operation: &str,
        role: &str,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Authorization,
            AuditSubcategory::AccessControl,
            AuditSeverity::Info,
            actor,
            contract,
            "authorization_success",
            "User successfully authorized for operation",
        )
        .compliance_tags(vec!["access_control", "authorization"])
        .metadata(vec![operation, role])
        .publish();
    }

    /// Authorization failure audit event
    pub fn authorization_failure(
        env: &Env,
        actor: Address,
        contract: Address,
        operation: &str,
        reason: &str,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Authorization,
            AuditSubcategory::AccessControl,
            AuditSeverity::Warning,
            actor,
            contract,
            "authorization_failure",
            "User authorization failed for operation",
        )
        .compliance_tags(vec!["access_control", "authorization_failure"])
        .metadata(vec![operation, reason])
        .publish();
    }

    /// Configuration change audit event
    pub fn configuration_change(
        env: &Env,
        actor: Address,
        contract: Address,
        parameter: &str,
        old_value: &str,
        new_value: &str,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Compliance,
            AuditSubcategory::ConfigurationChange,
            AuditSeverity::Warning,
            actor,
            contract,
            "configuration_change",
            "Contract configuration parameter modified",
        )
        .state_transition(old_value, new_value)
        .compliance_tags(vec!["configuration_change", "system_modification"])
        .metadata(vec![parameter])
        .publish();
    }

    /// Emergency pause activated audit event
    pub fn emergency_pause_activated(
        env: &Env,
        actor: Address,
        contract: Address,
        reason: &str,
        duration_seconds: u64,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Emergency,
            AuditSubcategory::EmergencyOperation,
            AuditSeverity::Critical,
            actor,
            contract,
            "emergency_pause_activated",
            "Emergency pause activated for contract operations",
        )
        .compliance_tags(vec!["emergency_operation", "system_pause"])
        .metadata(vec![
            reason,
            &duration_seconds.to_string(),
        ])
        .publish();
    }

    /// Cross-contract call audit event
    pub fn cross_contract_call(
        env: &Env,
        actor: Address,
        contract: Address,
        target_contract: Address,
        function: &str,
        amount: Option<i128>,
    ) {
        let mut builder = AuditEventBuilder::new(
            env,
            EventCategory::CrossChain,
            AuditSubcategory::CrossContractCall,
            AuditSeverity::Info,
            actor,
            contract,
            "cross_contract_call",
            "Cross-contract function call executed",
        )
        .compliance_tags(vec!["cross_contract", "inter_contract_communication"])
        .metadata(vec![
            &target_contract.to_string(),
            function,
        ]);

        if let Some(amt) = amount {
            builder = builder.amount(amt, "XLM");
        }

        builder.publish();
    }

    /// Data modification audit event
    pub fn data_modification(
        env: &Env,
        actor: Address,
        contract: Address,
        data_type: &str,
        record_id: u64,
        operation: &str,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Compliance,
            AuditSubcategory::DataModification,
            AuditSeverity::Info,
            actor,
            contract,
            "data_modification",
            "Data record modified in contract storage",
        )
        .subject_id(record_id)
        .compliance_tags(vec!["data_modification", "storage_operation"])
        .metadata(vec![data_type, operation])
        .publish();
    }

    /// Compliance check audit event
    pub fn compliance_check(
        env: &Env,
        actor: Address,
        contract: Address,
        check_type: &str,
        result: &str,
        details: &str,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Compliance,
            AuditSubcategory::ComplianceCheck,
            AuditSeverity::Info,
            actor,
            contract,
            "compliance_check",
            "Compliance check performed",
        )
        .compliance_tags(vec!["compliance", "regulatory_check"])
        .metadata(vec![check_type, result, details])
        .publish();
    }

    /// System operation audit event
    pub fn system_operation(
        env: &Env,
        actor: Address,
        contract: Address,
        operation: &str,
        details: &str,
    ) {
        AuditEventBuilder::new(
            env,
            EventCategory::Monitoring,
            AuditSubcategory::SystemOperation,
            AuditSeverity::Info,
            actor,
            contract,
            "system_operation",
            "System-level operation performed",
        )
        .compliance_tags(vec!["system_operation", "maintenance"])
        .metadata(vec![operation, details])
        .publish();
    }
}
