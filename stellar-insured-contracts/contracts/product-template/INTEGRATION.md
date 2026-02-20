# Product Template System Integration

## Integration with Existing Contracts

The Product Template System is designed to work seamlessly with your existing insurance contracts. Here's how to integrate it:

## 1. Integration with Policy Contract

The template system can generate policies that integrate with your existing policy contract:

```rust
// Create a policy from template that integrates with existing policy system
let template_policy_id = ProductTemplateContract::create_policy_from_template(
    env.clone(),
    holder.clone(),
    template_id,
    coverage_amount,
    duration_days,
    deductible,
    custom_values,
)?;

// The template policy can be used alongside regular policies
// Both follow the same lifecycle and validation rules
```

## 2. Cross-Contract Communication

### Template → Policy Integration
```rust
// When a template policy is created, it can trigger actions in the policy contract
// For example, transferring premium payments to the risk pool
fn handle_template_policy_creation(
    env: Env,
    template_policy: TemplatePolicy,
    risk_pool_address: Address,
) -> Result<(), ContractError> {
    // Transfer premium to risk pool
    // This would be a cross-contract call to your risk pool contract
    // env.invoke_contract(&risk_pool_address, ...)
    
    Ok(())
}
```

### Policy → Template Integration
```rust
// Existing policies can reference template IDs for categorization
fn create_policy_with_template_reference(
    env: Env,
    policy_contract: Address,
    template_id: u64, // Optional template reference
    holder: Address,
    coverage: i128,
    premium: i128,
) -> Result<u64, ContractError> {
    // Create policy in existing contract
    let policy_id = env.invoke_contract::<u64>(
        &policy_contract,
        &soroban_sdk::symbol_short!("issue_policy"),
        (
            holder,
            coverage,
            premium,
            template_id, // Store template reference
        ).into_val(&env),
    )?;
    
    Ok(policy_id)
}
```

## 3. Governance Integration Example

```rust
// Integration with existing governance contract
fn propose_template_approval_with_governance(
    env: Env,
    governance_contract: Address,
    template_id: u64,
    proposer: Address,
) -> Result<u64, ContractError> {
    // Create governance proposal for template approval
    let proposal_id = env.invoke_contract::<u64>(
        &governance_contract,
        &soroban_sdk::symbol_short!("create_proposal"),
        (
            proposer,
            soroban_sdk::symbol_short!("Approve Template"),
            soroban_sdk::symbol_short!("Template approval for insurance product"),
            // Execution data would contain template approval instructions
            soroban_sdk::symbol_short!("approve_template"),
            51u32, // 51% threshold
        ).into_val(&env),
    )?;
    
    Ok(proposal_id)
}
```

## 4. Risk Pool Integration

```rust
// Template policies can contribute to risk pools
fn calculate_template_collateral(
    env: Env,
    template: ProductTemplate,
    coverage_amount: i128,
) -> i128 {
    // Calculate required collateral based on template requirements
    let collateral_bps = template.collateral_ratio_bps as i128;
    let required_collateral = (coverage_amount * collateral_bps) / 10000;
    
    required_collateral
}

// Integrate with risk pool contract
fn deposit_template_collateral(
    env: Env,
    risk_pool_contract: Address,
    template_policy: TemplatePolicy,
    collateral_amount: i128,
) -> Result<(), ContractError> {
    // Transfer collateral to risk pool
    // env.invoke_contract(&risk_pool_contract, ...)
    Ok(())
}
```

## 5. Claims Integration

```rust
// Template policies can file claims through existing claims system
fn file_claim_for_template_policy(
    env: Env,
    claims_contract: Address,
    template_policy_id: u64,
    claim_amount: i128,
    evidence: BytesN<32>,
) -> Result<u64, ContractError> {
    // Get the underlying policy data
    let template_policy = ProductTemplateContract::get_template_policy(
        env.clone(),
        template_policy_id,
    )?;
    
    // File claim in existing claims contract
    let claim_id = env.invoke_contract::<u64>(
        &claims_contract,
        &soroban_sdk::symbol_short!("submit_claim"),
        (
            template_policy.policy_id, // or template_policy_id
            claim_amount,
            evidence,
        ).into_val(&env),
    )?;
    
    Ok(claim_id)
}
```

## 6. Event Integration

The template system emits events that can be consumed by other contracts:

```rust
// Listen for template events in other contracts
fn handle_template_events(
    env: Env,
    event_data: (Symbol, u64, Address, Symbol),
) {
    let (event_type, template_id, creator, template_name) = event_data;
    
    match event_type.to_str() {
        "template_created" => {
            // Handle new template creation
            println!("New template created: {} by {}", template_name, creator);
        }
        "template_approved" => {
            // Handle template approval
            println!("Template {} approved", template_name);
        }
        "template_deployed" => {
            // Handle template deployment
            println!("Template {} is now active", template_name);
        }
        _ => {}
    }
}
```

## 7. Storage Integration

The template system uses the same storage patterns as your existing contracts:

```rust
// Template storage keys follow the same pattern as existing contracts
const TEMPLATE: Symbol = Symbol::short("TEMPLATE");
const TEMPLATE_COUNTER: Symbol = Symbol::short("TEMP_CNT");
const TEMPLATE_POLICY: Symbol = Symbol::short("TEMP_POL");

// This maintains consistency with your existing contract architecture
```

## 8. Error Handling Integration

The template system uses the same error patterns:

```rust
// Template errors integrate with existing contract errors
#[contracterror]
pub enum ContractError {
    Unauthorized = 1,
    Paused = 2,
    InvalidInput = 3,
    NotFound = 4,
    // ... existing errors
    InvalidTemplateStatus = 9,      // Template-specific error
    InvalidParameterValue = 10,     // Template-specific error
    TemplateValidationFailed = 11,  // Template-specific error
}
```

## 9. Migration Path

To migrate existing custom insurance products to templates:

```rust
// Convert existing custom product to template
fn migrate_custom_product_to_template(
    env: Env,
    custom_product_data: CustomProduct,
) -> Result<u64, ContractError> {
    // Extract parameters from existing custom product
    let template_id = ProductTemplateContract::create_template(
        env.clone(),
        custom_product_data.creator,
        custom_product_data.name,
        custom_product_data.description,
        custom_product_data.category,
        custom_product_data.risk_level,
        custom_product_data.premium_model,
        custom_product_data.coverage_type,
        custom_product_data.min_coverage,
        custom_product_data.max_coverage,
        custom_product_data.min_duration,
        custom_product_data.max_duration,
        custom_product_data.base_rate,
        custom_product_data.min_deductible,
        custom_product_data.max_deductible,
        custom_product_data.collateral_ratio,
        custom_product_data.custom_params,
    )?;
    
    // Submit for approval
    ProductTemplateContract::submit_template_for_review(
        env.clone(),
        custom_product_data.creator,
        template_id,
    )?;
    
    Ok(template_id)
}
```

## 10. Monitoring and Analytics

```rust
// Monitor template performance
fn get_template_analytics(
    env: Env,
    template_id: u64,
) -> Result<TemplateAnalytics, ContractError> {
    let template = ProductTemplateContract::get_template(env.clone(), template_id)?;
    
    // Get policy count for this template
    let policies = ProductTemplateContract::get_policies_by_template(
        env.clone(),
        template_id,
        0,
        u32::MAX,
    )?;
    
    // Calculate total coverage issued
    let total_coverage: i128 = policies.iter().map(|p| p.coverage_amount).sum();
    
    // Calculate total premiums collected
    let total_premiums: i128 = policies.iter().map(|p| p.premium_amount).sum();
    
    Ok(TemplateAnalytics {
        template_id,
        policy_count: policies.len() as u32,
        total_coverage,
        total_premiums,
        template_status: template.status,
    })
}

#[contracttype]
pub struct TemplateAnalytics {
    pub template_id: u64,
    pub policy_count: u32,
    pub total_coverage: i128,
    pub total_premiums: i128,
    pub template_status: TemplateStatus,
}
```

This integration approach ensures that the template system works harmoniously with your existing insurance contract ecosystem while providing the flexibility and standardization needed for rapid product development.