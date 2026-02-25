# Structured Events Implementation

This document describes the comprehensive structured events system implemented for analytics, monitoring, and audit compliance across all Stellar Insured contracts.

## Overview

The structured events system provides:
- **Standardized event format** across all contracts for consistent analytics
- **Rich metadata** for detailed monitoring and analysis
- **Audit trail events** for regulatory compliance
- **Indexer-friendly structure** for efficient data processing
- **Backward compatibility** with existing event systems

## Architecture

### Core Components

1. **`events.rs`** - Main structured event framework
2. **`audit_events.rs`** - Compliance-focused audit events
3. **`event_verification.rs`** - Testing and monitoring utilities

### Event Types

#### Structured Events (`events.rs`)
- **EventCategory**: 13 categories for organized analytics
- **EventSeverity**: 4 levels for monitoring and alerting
- **StructuredEvent**: Comprehensive event structure with metadata
- **EventBuilder**: Fluent interface for event creation

#### Audit Events (`audit_events.rs`)
- **AuditSubcategory**: 10 granular subcategories
- **AuditSeverity**: 4 compliance-focused severity levels
- **AuditEvent**: Detailed audit structure with state transitions
- **AuditEventBuilder**: Builder for audit events

## Event Categories

### Main Categories
- `Policy` - Policy lifecycle events
- `Claim` - Claim processing events
- `RiskPool` - Liquidity operations
- `Governance` - Proposal and voting events
- `Treasury` - Financial operations
- `Oracle` - Data validation events
- `Authorization` - Access control events
- `Emergency` - Critical system events
- `CrossChain` - Bridge operations
- `Compliance` - Regulatory events
- `Monitoring` - Performance metrics
- `Trading` - AMM operations
- `Identity` - KYC/verification events

### Audit Subcategories
- `PolicyOperation` - Policy-specific actions
- `ClaimProcessing` - Claim workflow events
- `FinancialOperation` - Monetary transactions
- `AccessControl` - Authorization events
- `ConfigurationChange` - System modifications
- `EmergencyOperation` - Critical actions
- `CrossContractCall` - Inter-contract communication
- `DataModification` - Storage changes
- `ComplianceCheck` - Regulatory validations
- `SystemOperation` - Maintenance events

## Event Structure

### Structured Event Format
```rust
pub struct StructuredEvent {
    pub event_id: BytesN<32>,           // Unique identifier
    pub category: EventCategory,        // Main category
    pub event_type: String,             // Specific event type
    pub severity: EventSeverity,        // Severity level
    pub actor: Address,                 // Who triggered it
    pub source_contract: Address,       // Contract address
    pub timestamp: u64,                 // When it occurred
    pub subject_id: Option<u64>,        // Primary entity ID
    pub data: Vec<String>,              // Event-specific data
    pub related_events: Option<Vec<BytesN<32>>>, // Correlations
    pub data_hash: Option<BytesN<32>>,  // Off-chain data hash
    pub metadata: Vec<String>,          // Additional metadata
}
```

### Audit Event Format
```rust
pub struct AuditEvent {
    pub audit_id: BytesN<32>,          // Unique audit ID
    pub category: EventCategory,        // Main category
    pub subcategory: AuditSubcategory,  // Granular subcategory
    pub severity: AuditSeverity,        // Compliance severity
    pub actor: Address,                 // User/system actor
    pub source_contract: Address,       // Contract address
    pub timestamp: u64,                 // Event timestamp
    pub subject_id: Option<u64>,        // Primary subject
    pub action: String,                 // Action performed
    pub description: String,            // Detailed description
    pub previous_state: Option<String>,  // State before
    pub new_state: Option<String>,      // State after
    pub amount: Option<i128>,           // Amount involved
    pub asset: Option<String>,          // Asset type
    pub related_events: Option<Vec<BytesN<32>>>, // Related events
    pub data_hash: Option<BytesN<32>>,  // Data verification
    pub compliance_tags: Vec<String>,    // Regulatory tags
    pub metadata: Vec<String>,          // Additional data
}
```

## Usage Examples

### Basic Structured Event
```rust
use insurance_contracts::events::{EventCategory, EventSeverity, EventBuilder};

EventBuilder::new(
    &env,
    EventCategory::Policy,
    "PolicyIssued",
    EventSeverity::Info,
    manager,
    env.current_contract_address(),
)
.subject_id(policy_id)
.data_fields(vec![
    &holder.to_string(),
    &coverage_amount.to_string(),
    &premium_amount.to_string(),
])
.publish();
```

### Audit Event with State Transition
```rust
use insurance_contracts::audit_events::{AuditSubcategory, AuditSeverity as AuditSeverityLevel, audit_events};

audit_events::claim_approved(
    &env,
    processor,
    env.current_contract_address(),
    claim_id,
    policy_id,
    amount,
);
```

### Custom Event Builder
```rust
EventBuilder::new(&env, EventCategory::Claim, "CustomEvent", EventSeverity::Warning, actor, contract)
    .subject_id(entity_id)
    .data("custom_field")
    .related_events(related_ids)
    .data_hash(data_hash)
    .metadata(vec!["tag1", "tag2"])
    .publish();
```

## Implementation Status

### Completed Contracts
- ✅ **Policy Contract** - Policy issuance events
- ✅ **Claims Contract** - Claim submission events  
- ✅ **Risk Pool Contract** - Liquidity deposit events

### Pending Implementation
- 🔄 **Governance Contract** - Proposal and voting events
- 🔄 **Treasury Contract** - Fee collection events
- 🔄 **Oracle Contract** - Data submission events
- 🔄 **Audit Trail Contract** - Compliance events

## Event Verification

The system includes comprehensive verification utilities:

### Verification Checklist
```rust
use insurance_contracts::event_verification::EventVerificationChecklist;

let is_compliant = EventVerificationChecklist::verify_policy_events(&env, &policy_address);
```

### Monitoring
```rust
use insurance_contracts::event_verification::EventMonitor;

let report = EventMonitor::generate_compliance_report(&env, &contract_addresses);
```

## Indexer Integration

### Event Topics
- `structured_event` - Main structured events
- `audit_event` - Compliance audit events

### Event Data Structure
Events are designed for efficient indexer processing:
- Fixed event topic structure
- Consistent data field ordering
- JSON-serializable metadata
- Unique event IDs for deduplication

### Sample Indexer Query
```sql
-- Find all policy events
SELECT * FROM events 
WHERE topic_0 = 'structured_event' 
AND category = 'Policy'
AND timestamp > '2024-01-01';

-- Find audit events with compliance tags
SELECT * FROM events 
WHERE topic_0 = 'audit_event'
AND compliance_tags @> ARRAY['financial_transaction'];
```

## Compliance Features

### Regulatory Compliance
- **Audit Trail**: Complete record of all operations
- **State Tracking**: Before/after state for critical operations
- **Financial Tracking**: Amount and asset information for all transactions
- **Access Control**: Authorization success/failure logging
- **Data Integrity**: Hash verification for off-chain data

### Compliance Tags
Standardized tags for regulatory filtering:
- `policy_creation` - New policy issuance
- `financial_transaction` - Monetary operations
- `access_control` - Authorization events
- `compliance_check` - Regulatory validations
- `emergency_operation` - Critical system events

## Performance Considerations

### Gas Optimization
- Efficient event ID generation
- Minimal data duplication
- Optimized storage patterns
- Lazy evaluation where possible

### Event Size Limits
- Structured events: ~1KB typical
- Audit events: ~2KB typical
- Maximum event size: 5KB (Soroban limit)

## Migration Guide

### From Legacy Events
1. Keep existing events for backward compatibility
2. Add structured events alongside legacy events
3. Gradually migrate consumers to structured format
4. Remove legacy events in future major version

### Best Practices
1. Always emit both structured and audit events for critical operations
2. Use appropriate severity levels for monitoring
3. Include relevant metadata for analytics
4. Maintain backward compatibility during transition
5. Use verification utilities to ensure compliance

## Testing

### Unit Tests
```rust
#[test]
fn test_structured_event_creation() {
    // Test event creation and validation
}

#[test]
fn test_audit_event_compliance() {
    // Test audit event compliance requirements
}
```

### Integration Tests
```rust
#[test]
fn test_event_verification() {
    // Test comprehensive event verification
}
```

## Future Enhancements

### Planned Features
- Event aggregation and batching
- Real-time event streaming
- Advanced analytics queries
- Event subscription system
- Cross-chain event correlation

### Potential Improvements
- Event compression for large payloads
- Event encryption for sensitive data
- Event replay functionality
- Event-based triggers
- Advanced filtering capabilities

## Support

For questions or issues related to the structured events implementation:
1. Check this documentation
2. Review the verification utilities
3. Examine existing implementations
4. Contact the development team

## Version History

- **v1.0.0** - Initial implementation with core structured events
- **v1.1.0** - Added audit events and compliance features
- **v1.2.0** - Enhanced verification and monitoring capabilities
