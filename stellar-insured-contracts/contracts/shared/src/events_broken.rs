#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol, String, BytesN, Vec, IntoVal};

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

    /// Generate unique event ID
    fn generate_event_id(env: &Env, contract: &Address, timestamp: u64, event_type: &str) -> BytesN<32> {
        use soroban_sdk::crypto::sha256;
        
        // Simple hash generation - use direct conversion
        let contract_bytes = soroban_sdk::Bytes::from_slice(env, &contract.to_string().as_bytes());
        let timestamp_bytes = soroban_sdk::Bytes::from_slice(env, &timestamp.to_string().as_bytes());
        let event_type_bytes = soroban_sdk::Bytes::from_slice(env, event_type.as_bytes());
        
        // Create combined data
        let mut combined = Vec::new(env);
        combined.push_back(contract_bytes);
        combined.push_back(timestamp_bytes);
        combined.push_back(event_type_bytes);
        
        sha256(&soroban_sdk::Bytes::from_slice(env, &combined.to_string().as_bytes()))
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
            "event", // Use static string instead of conversion
            self.severity,
            self.actor,
            self.source_contract,
        );

        event.publish(self.env);
    }
}
