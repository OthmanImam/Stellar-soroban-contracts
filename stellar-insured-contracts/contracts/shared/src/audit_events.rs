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

/// Simplified audit event structure for regulatory compliance
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
    /// Action performed
    pub action: String,
    /// Detailed description of the event
    pub description: String,
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
            action: String::from_str(env, action),
            description: String::from_str(env, description),
        }
    }

    /// Generate unique audit ID
    fn generate_audit_id(env: &Env, contract: &Address, timestamp: u64, action: &str) -> BytesN<32> {
        use soroban_sdk::crypto::sha256;
        
        // Simple hash generation using contract address, timestamp, and action
        let contract_str = contract.to_string();
        let timestamp_str = timestamp.to_string();
        
        // Create a combined string without format! macro
        let mut combined = String::from_str(env, &contract_str);
        combined.push_back_str(&timestamp_str);
        combined.push_back_str("-");
        combined.push_back_str(action);
        
        sha256(combined.as_bytes())
    }

    /// Publish audit event
    pub fn publish(self, env: &Env) {
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
        }
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
            &self.action.to_string(),
            &self.description.to_string(),
        );

        event.publish(self.env);
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
        );

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
        .publish();
    }
}
