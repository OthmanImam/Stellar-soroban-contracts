#![cfg(test)]

use soroban_sdk::{Env};

use super::events::{EventCategory, EventSeverity};

#[test]
fn test_event_categories() {
    let env = Env::default();
    
    // Test that all event categories can be created and compared
    let policy_category = EventCategory::Policy;
    let claim_category = EventCategory::Claim;
    let risk_pool_category = EventCategory::RiskPool;
    
    // Test category equality
    assert_eq!(policy_category, EventCategory::Policy);
    assert_eq!(claim_category, EventCategory::Claim);
    assert_eq!(risk_pool_category, EventCategory::RiskPool);
    
    // Test that categories are different
    assert_ne!(policy_category, claim_category);
    assert_ne!(claim_category, risk_pool_category);
}

#[test]
fn test_event_severity() {
    let env = Env::default();
    
    // Test that all event severity levels can be created
    let info_severity = EventSeverity::Info;
    let warning_severity = EventSeverity::Warning;
    let error_severity = EventSeverity::Error;
    let critical_severity = EventSeverity::Critical;
    
    // Test severity equality
    assert_eq!(info_severity, EventSeverity::Info);
    assert_eq!(warning_severity, EventSeverity::Warning);
    assert_eq!(error_severity, EventSeverity::Error);
    assert_eq!(critical_severity, EventSeverity::Critical);
    
    // Test that severities are different
    assert_ne!(info_severity, warning_severity);
    assert_ne!(warning_severity, error_severity);
    assert_ne!(error_severity, critical_severity);
}
