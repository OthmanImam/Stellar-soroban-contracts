# Product Template System - Quick Start Guide

## Getting Started

This guide will walk you through creating your first insurance product template and generating policies from it.

## Prerequisites

- Rust and Soroban SDK installed
- Basic understanding of Stellar smart contracts
- Access to a Stellar testnet or local development environment

## Step 1: Initialize the Contract

```rust
// Initialize the product template contract
let admin = Address::generate(&env);
let governance_contract = Address::generate(&env);

let validation_rules = TemplateValidationRules {
    min_collateral_ratio_bps: 1000,    // 10%
    max_premium_rate_bps: 5000,        // 50%
    min_duration_days: 1,
    max_duration_days: 365,
    approval_threshold_bps: 5100,      // 51%
    min_update_interval: 86400,        // 24 hours
};

ProductTemplateContract::initialize(
    env.clone(),
    admin.clone(),
    governance_contract.clone(),
    validation_rules,
)?;
```

## Step 2: Create Your First Template

```rust
// Create a simple travel insurance template
let creator = Address::generate(&env);

let template_id = ProductTemplateContract::create_template(
    env.clone(),
    creator.clone(),
    Symbol::new(&env, "Basic Travel Insurance"),
    Symbol::new(&env, "Coverage for trip cancellations and delays"),
    ProductCategory::Travel,
    RiskLevel::Low,
    PremiumModel::Percentage,
    CoverageType::Full,
    1000000,        // Min: 1 unit
    100000000,      // Max: 100 units
    1,              // Min: 1 day
    30,             // Max: 30 days
    300,            // 3% base premium
    0,              // No deductible
    10000000,       // 10 units max deductible
    1000,           // 10% collateral
    Vec::new(&env), // No custom parameters
)?;

println!("Created template with ID: {}", template_id);
```

## Step 3: Submit for Governance Review

```rust
// Submit template for approval
ProductTemplateContract::submit_template_for_review(
    env.clone(),
    creator.clone(),
    template_id,
)?;

// Propose approval through governance
let proposal_id = ProductTemplateContract::propose_template_approval(
    env.clone(),
    creator.clone(), // or any governance participant
    template_id,
    Symbol::new(&env, "Approve Basic Travel Insurance"),
    Symbol::new(&env, "Standard travel insurance for short trips"),
    51, // 51% approval threshold
)?;

println!("Governance proposal created: {}", proposal_id);
```

## Step 4: Deploy the Template

```rust
// After governance approval, deploy the template
ProductTemplateContract::execute_template_approval(
    env.clone(),
    admin.clone(), // governance executor
    proposal_id,
    template_id,
)?;

// Deploy to make it active
ProductTemplateContract::deploy_template(
    env.clone(),
    admin.clone(),
    template_id,
)?;

println!("Template is now Active and ready for policies!");
```

## Step 5: Create Policies from Template

```rust
// Customer wants to create a policy
let holder = Address::generate(&env);

let policy_id = ProductTemplateContract::create_policy_from_template(
    env.clone(),
    holder.clone(),
    template_id,
    50000000,  // 50 units coverage
    14,        // 14 day trip
    500000,    // 0.5 unit deductible
    Vec::new(&env), // No customizations
)?;

println!("Created policy with ID: {}", policy_id);

// View the created policy
let policy = ProductTemplateContract::get_template_policy(env.clone(), policy_id)?;
println!("Policy details: {:?}", policy);
```

## Advanced Example: Custom Parameters

```rust
// Create template with customization options
let mut custom_params = Vec::new(&env);
custom_params.push_back(CustomParam::Boolean {
    name: Symbol::new(&env, "medical_coverage"),
    default_value: false,
});
custom_params.push_back(CustomParam::Choice {
    name: Symbol::new(&env, "coverage_area"),
    options: vec![
        Symbol::new(&env, "domestic"),
        Symbol::new(&env, "international"),
        Symbol::new(&env, "worldwide"),
    ].try_into().unwrap(),
    default_index: 0,
});

let advanced_template_id = ProductTemplateContract::create_template(
    env.clone(),
    creator.clone(),
    Symbol::new(&env, "Advanced Travel Insurance"),
    Symbol::new(&env, "Customizable travel coverage with medical options"),
    ProductCategory::Travel,
    RiskLevel::Medium,
    PremiumModel::RiskBased,
    CoverageType::Partial,
    1000000,
    500000000,
    1,
    365,
    250, // 2.5% base rate
    0,
    50000000,
    1500, // 15% collateral
    custom_params,
)?;

// Create policy with customizations
let mut custom_values = Vec::new(&env);
custom_values.push_back(CustomParamValue {
    name: Symbol::new(&env, "medical_coverage"),
    value: CustomParamValueData::Boolean(true),
});
custom_values.push_back(CustomParamValue {
    name: Symbol::new(&env, "coverage_area"),
    value: CustomParamValueData::Choice(2), // worldwide
});

let advanced_policy_id = ProductTemplateContract::create_policy_from_template(
    env.clone(),
    holder.clone(),
    advanced_template_id,
    100000000, // 100 units
    30,        // 30 days
    1000000,   // 1 unit deductible
    custom_values,
)?;
```

## Template Management Commands

### Check Template Status
```rust
let template = ProductTemplateContract::get_template(env.clone(), template_id)?;
println!("Template status: {:?}", template.status);
```

### List Active Templates
```rust
let active_templates = ProductTemplateContract::get_active_templates(env.clone())?;
println!("Active templates: {}", active_templates.len());
```

### Get User's Policies
```rust
let user_policies = ProductTemplateContract::get_policies_by_holder(
    env.clone(),
    holder.clone(),
    0,    // start index
    10,   // limit
)?;
println!("User has {} policies", user_policies.len());
```

## Common Operations

### Template Updates
```rust
// Update template (only when in Draft or Approved status)
ProductTemplateContract::update_template(
    env.clone(),
    creator.clone(),
    template_id,
    Some(Symbol::new(&env, "Updated Template Name")),
    None, // keep existing description
    None, // keep existing category
    None, // keep existing risk level
    None, // keep existing premium model
    None, // keep existing coverage type
    None, // keep existing min coverage
    None, // keep existing max coverage
    Some(45), // new max duration
    None, // keep existing base premium
    None, // keep existing min deductible
    None, // keep existing max deductible
    None, // keep existing collateral ratio
    None, // keep existing custom params
)?;
```

### Template Retirement
```rust
// Retire a template (existing policies remain valid)
ProductTemplateContract::retire_template(
    env.clone(),
    admin.clone(),
    template_id,
    Symbol::new(&env, "Product discontinued"),
)?;
```

### Template Archiving
```rust
// Archive a template completely
ProductTemplateContract::archive_template(
    env.clone(),
    admin.clone(),
    template_id,
    Symbol::new(&env, "No longer offered"),
)?;
```

## Error Handling

```rust
match ProductTemplateContract::create_policy_from_template(
    env.clone(),
    holder.clone(),
    template_id,
    coverage_amount,
    duration_days,
    deductible,
    custom_values,
) {
    Ok(policy_id) => {
        println!("Successfully created policy: {}", policy_id);
    }
    Err(ContractError::InvalidTemplateStatus) => {
        println!("Template is not active - cannot create policies");
    }
    Err(ContractError::InvalidInput) => {
        println!("Invalid coverage amount or duration");
    }
    Err(ContractError::InvalidParameterValue) => {
        println!("Invalid custom parameter values");
    }
    Err(e) => {
        println!("Error creating policy: {:?}", e);
    }
}
```

## Best Practices

1. **Start Simple**: Begin with basic templates without custom parameters
2. **Test Thoroughly**: Validate all parameter combinations before deployment
3. **Document Clearly**: Provide detailed descriptions for governance review
4. **Monitor Performance**: Track policy creation and claims from templates
5. **Regular Updates**: Update templates based on market conditions and feedback
6. **Governance Engagement**: Work closely with the governance community during approval

## Next Steps

- Explore different premium models and risk levels
- Add more sophisticated custom parameters
- Integrate with existing policy and claims contracts
- Implement advanced validation rules
- Add analytics and reporting features
- Consider template versioning for updates

## Need Help?

Check the full documentation in `README.md` for detailed API references and advanced usage patterns.