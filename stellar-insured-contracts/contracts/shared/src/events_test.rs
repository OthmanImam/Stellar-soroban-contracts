#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol, String, BytesN, Vec};

/// Simplified test version of events to verify compilation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TestEventCategory {
    Policy,
    Claim,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TestEventSeverity {
    Info,
    Warning,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TestStructuredEvent {
    pub event_id: BytesN<32>,
    pub category: TestEventCategory,
    pub event_type: String,
    pub severity: TestEventSeverity,
    pub actor: Address,
    pub source_contract: Address,
    pub timestamp: u64,
    pub subject_id: Option<u64>,
    pub data: Vec<String>,
}

impl TestStructuredEvent {
    pub fn new(
        env: &Env,
        category: TestEventCategory,
        event_type: String,
        severity: TestEventSeverity,
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
        }
    }

    fn generate_event_id(env: &Env, contract: &Address, timestamp: u64, event_type: &str) -> BytesN<32> {
        use soroban_sdk::crypto::sha256;
        
        let mut data = Vec::new(env);
        data.push_back(contract.clone());
        data.push_back(timestamp.into_val(env));
        data.push_back(String::from_str(env, event_type));
        
        sha256(&data.to_bytes())
    }

    pub fn publish(self, env: &Env) {
        env.events().publish(
            (Symbol::new(env, "test_structured_event"), self.event_id),
            (
                self.category,
                self.event_type,
                self.severity,
                self.actor,
                self.source_contract,
                self.timestamp,
                self.subject_id,
                self.data,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let env = Env::default();
        let contract = Address::generate(&env);
        let actor = Address::generate(&env);
        
        let event = TestStructuredEvent::new(
            &env,
            TestEventCategory::Policy,
            String::from_str(&env, "PolicyIssued"),
            TestEventSeverity::Info,
            actor.clone(),
            contract.clone(),
            Some(123),
            Vec::from_array(&env, [
                String::from_str(&env, "test_data"),
                String::from_str(&env, "more_data"),
            ]),
        );

        assert_eq!(event.category, TestEventCategory::Policy);
        assert_eq!(event.severity, TestEventSeverity::Info);
        assert_eq!(event.subject_id, Some(123));
        assert_eq!(event.data.len(), 2);
    }

    #[test]
    fn test_event_id_generation() {
        let env = Env::default();
        let contract = Address::generate(&env);
        
        let event_id1 = TestStructuredEvent::generate_event_id(&env, &contract, 12345, "test");
        let event_id2 = TestStructuredEvent::generate_event_id(&env, &contract, 12345, "test");
        let event_id3 = TestStructuredEvent::generate_event_id(&env, &contract, 12346, "test");
        
        // Same inputs should produce same ID
        assert_eq!(event_id1, event_id2);
        // Different timestamp should produce different ID
        assert_ne!(event_id1, event_id3);
    }
}
