use soroban_sdk::{contracttype, Address, Env, Symbol, String, BytesN, Vec, Bytes};

/// Event categories for structured events
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventCategory {
    Policy,
    Claim,
    RiskPool,
    Governance,
    Treasury,
    Authorization,
    Compliance,
    Emergency,
    CrossChain,
    Monitoring,
}

/// Event severity levels
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Simplified structured event
#[contracttype]
#[derive(Clone, Debug)]
pub struct StructuredEvent {
    pub event_id: BytesN<32>,
    pub category: EventCategory,
    pub event_type: String,
    pub severity: EventSeverity,
    pub actor: Address,
    pub source_contract: Address,
    pub timestamp: u64,
    pub subject_id: Option<u64>,
    pub data: Vec<String>,
}

impl StructuredEvent {
    /// Create a new structured event
    pub fn new(
        env: &Env,
        category: EventCategory,
        event_type: &str,
        severity: EventSeverity,
        actor: Address,
        source_contract: Address,
    ) -> Self {
        let timestamp = env.ledger().timestamp();
        let event_id = Self::generate_event_id(env, &source_contract, timestamp, event_type);
        
        Self {
            event_id,
            category,
            event_type: String::from_str(env, event_type),
            severity,
            actor,
            source_contract,
            timestamp,
            subject_id: None,
            data: Vec::new(env),
        }
    }

    /// Generate unique event ID - simplified
    fn generate_event_id(env: &Env, _contract: &Address, timestamp: u64, _event_type: &str) -> BytesN<32> {
        // Use simple timestamp for ID generation - convert to BytesN<32>
        let timestamp_bytes = timestamp.to_le_bytes();
        let mut hash_bytes = [0u8; 32];
        
        // Simple hash: just use timestamp bytes (pad or truncate as needed)
        for i in 0..hash_bytes.len().min(timestamp_bytes.len()) {
            hash_bytes[i] = timestamp_bytes[i];
        }
        
        BytesN::from_array(env, &hash_bytes)
    }

    /// Add data
    pub fn add_data(mut self, env: &Env, data: &str) -> Self {
        self.data.push_back(String::from_str(env, data));
        self
    }

    /// Publish event
    pub fn publish(self, env: &Env) {
        env.events().publish(
            (Symbol::new(env, "structured_event"), self.event_id),
            (
                self.event_type,
                self.category,
                self.severity,
                self.actor,
                self.timestamp,
            ),
        );
    }
}

/// Builder for creating structured events
pub struct EventBuilder<'a> {
    env: &'a Env,
    category: EventCategory,
    event_type: String,
    severity: EventSeverity,
    actor: Address,
    source_contract: Address,
    subject_id: Option<u64>,
    data: Vec<String>,
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
        }
    }

    /// Set subject ID
    pub fn subject_id(mut self, subject_id: u64) -> Self {
        self.subject_id = Some(subject_id);
        self
    }

    /// Add data
    pub fn data(mut self, data: &str) -> Self {
        self.data.push_back(String::from_str(self.env, data));
        self
    }

    /// Build and publish the event
    pub fn publish(self) {
        let event = StructuredEvent::new(
            self.env,
            self.category,
            "event", // Use static string
            self.severity,
            self.actor,
            self.source_contract,
        );

        event.publish(self.env);
    }
}

/// Convenience functions for common events
pub mod events {
    use super::*;

    /// Policy issued event
    pub fn policy_issued(
        env: &Env,
        actor: Address,
        contract: Address,
        _policy_id: u64,
        _holder: Address,
        _coverage_amount: i128,
        _premium_amount: i128,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Policy,
            "policy_issued",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(_policy_id)
        .data("policy_issued")
        .publish();
    }

    /// Claim submitted event
    pub fn claim_submitted(
        env: &Env,
        actor: Address,
        contract: Address,
        _claim_id: u64,
        _policy_id: u64,
        _amount: i128,
    ) {
        EventBuilder::new(
            env,
            EventCategory::Claim,
            "claim_submitted",
            EventSeverity::Info,
            actor,
            contract,
        )
        .subject_id(_claim_id)
        .publish();
    }

    /// Risk pool deposit event
    pub fn risk_pool_deposit(
        env: &Env,
        actor: Address,
        contract: Address,
        _provider: Address,
        _amount: i128,
    ) {
        EventBuilder::new(
            env,
            EventCategory::RiskPool,
            "risk_pool_deposit",
            EventSeverity::Info,
            actor,
            contract,
        )
        .data("liquidity_deposited")
        .publish();
    }
}
