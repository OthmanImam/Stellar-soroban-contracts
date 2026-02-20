# Product Template System

## Overview

The Product Template System is a standardized insurance product template framework that enables rapid deployment of new insurance products with customization options. This system allows for the creation, management, and deployment of insurance product templates that can be used to generate customized policies.

## Key Features

- **Standardized Templates**: Pre-defined insurance product templates with configurable parameters
- **Customization Options**: Flexible parameter system for product customization
- **Governance Integration**: Template approval workflow through governance proposals
- **Template Lifecycle Management**: Complete template status management from Draft to Archived
- **Premium Calculation**: Multiple premium models (Fixed, Percentage, Risk-based, Tiered)
- **Validation System**: Comprehensive parameter validation and business rule enforcement

## Architecture

### Core Components

1. **ProductTemplateContract**: Main contract managing templates and policies
2. **Template Data Structures**: Enums and structs defining template properties
3. **Governance Integration**: Workflow for template approval and rejection
4. **Policy Engine**: System for creating policies from templates with custom parameters

### Template Lifecycle

```
Draft → PendingReview → Approved → Active → Deprecated → Archived
  ↑         ↓              ↓         ↓          ↓
  └─────────Rejected───────Deployed──Retired───Archive
```

### Status Transitions

- **Draft**: Initial template creation state
- **PendingReview**: Submitted for governance approval
- **Approved**: Governance approved, ready for deployment
- **Active**: Deployed and available for policy creation
- **Deprecated**: Retired but existing policies remain valid
- **Archived**: Completely removed from active use

## Template Structure

### Core Template Properties

```rust
pub struct ProductTemplate {
    pub id: u64,
    pub name: Symbol,
    pub description: Symbol,
    pub category: ProductCategory,
    pub status: TemplateStatus,
    pub risk_level: RiskLevel,
    pub premium_model: PremiumModel,
    pub coverage_type: CoverageType,
    pub min_coverage: i128,
    pub max_coverage: i128,
    pub min_duration_days: u32,
    pub max_duration_days: u32,
    pub base_premium_rate_bps: u32,
    pub min_deductible: i128,
    pub max_deductible: i128,
    pub collateral_ratio_bps: u32,
    pub custom_params: Vec<CustomParam>,
    pub creator: Address,
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u32,
}
```

### Customization Parameters

Templates support four types of customizable parameters:

1. **Integer**: Numeric values with min/max bounds
2. **Decimal**: Decimal values with min/max bounds  
3. **Boolean**: True/false options
4. **Choice**: Selection from predefined options

### Premium Models

1. **Fixed**: Constant premium amount
2. **Percentage**: Percentage of coverage amount
3. **RiskBased**: Risk-level adjusted calculation
4. **Tiered**: Coverage amount tier-based pricing

## Usage Examples

### 1. Creating a Basic Template

```rust
// Create a simple home insurance template
let template_id = ProductTemplateContract::create_template(
    env.clone(),
    creator.clone(),
    Symbol::new(&env, "Standard Home Insurance"),
    Symbol::new(&env, "Basic home insurance coverage"),
    ProductCategory::Property,
    RiskLevel::Medium,
    PremiumModel::Percentage,
    CoverageType::Full,
    1000000,        // 1 unit minimum coverage
    1000000000,     // 1000 units maximum coverage
    30,             // 30 days minimum duration
    365,            // 365 days maximum duration
    200,            // 2% base premium rate
    50000,          // 0.05 unit minimum deductible
    1000000,        // 1 unit maximum deductible
    1500,           // 15% collateral ratio
    Vec::new(&env), // No custom parameters
)?;
```

### 2. Creating a Template with Custom Parameters

```rust
// Create template with customization options
let mut custom_params = Vec::new(&env);
custom_params.push_back(CustomParam::Boolean {
    name: Symbol::new(&env, "additional_coverage"),
    default_value: false,
});
custom_params.push_back(CustomParam::Integer {
    name: Symbol::new(&env, "coverage_limit"),
    min_value: 1000000,
    max_value: 50000000,
    default_value: 10000000,
});

let template_id = ProductTemplateContract::create_template(
    env.clone(),
    creator.clone(),
    Symbol::new(&env, "Customizable Home Insurance"),
    Symbol::new(&env, "Home insurance with optional coverages"),
    ProductCategory::Property,
    RiskLevel::Medium,
    PremiumModel::Percentage,
    CoverageType::Full,
    1000000,
    1000000000,
    30,
    365,
    200,
    50000,
    1000000,
    1500,
    custom_params,
)?;
```

### 3. Template Approval Workflow

```rust
// 1. Submit template for review
ProductTemplateContract::submit_template_for_review(
    env.clone(),
    creator.clone(),
    template_id,
)?;

// 2. Propose approval through governance
let proposal_id = ProductTemplateContract::propose_template_approval(
    env.clone(),
    proposer.clone(),
    template_id,
    Symbol::new(&env, "Approve Home Insurance Template"),
    Symbol::new(&env, "Standard home insurance product"),
    51, // 51% approval threshold
)?;

// 3. Execute approval after governance vote
ProductTemplateContract::execute_template_approval(
    env.clone(),
    executor.clone(),
    proposal_id,
    template_id,
)?;

// 4. Deploy the approved template
ProductTemplateContract::deploy_template(
    env.clone(),
    admin.clone(),
    template_id,
)?;
```

### 4. Creating Policies from Templates

```rust
// Create custom parameter values
let mut custom_values = Vec::new(&env);
custom_values.push_back(CustomParamValue {
    name: Symbol::new(&env, "additional_coverage"),
    value: CustomParamValueData::Boolean(true),
});
custom_values.push_back(CustomParamValue {
    name: Symbol::new(&env, "coverage_limit"),
    value: CustomParamValueData::Integer(25000000),
});

// Create policy from template
let policy_id = ProductTemplateContract::create_policy_from_template(
    env.clone(),
    holder.clone(),
    template_id,
    50000000,  // 50 units coverage
    180,       // 180 days duration
    200000,    // 0.2 unit deductible
    custom_values,
)?;

// Get created policy details
let policy = ProductTemplateContract::get_template_policy(env.clone(), policy_id)?;
```

## Premium Calculation Examples

### Percentage Model
```rust
// 2% of $1000 coverage for 180 days
// Premium = 1000 * 0.02 * (180/365) = $9.86
```

### Risk-Based Model
```rust
// High risk multiplier: 1.5x base rate
// Base premium: $20, Risk multiplier: 1.5
// Final premium: $20 * 1.5 = $30
```

### Tiered Model
```rust
// Coverage tiers:
// $1000-10000: 1.0x rate
// $10001-100000: 0.9x rate  
// $100001+: 0.8x rate
```

## Governance Integration

The template system integrates with the governance contract for approval workflows:

### Approval Process
1. Template creator submits template for review
2. Governance proposal created for template approval
3. Community votes on proposal
4. If approved, template becomes Approved status
5. Admin can deploy the template to Active status

### Rejection Process
1. Governance proposal created for template rejection
2. Community votes on rejection proposal
3. If rejected, template returns to Draft status
4. Creator can revise and resubmit

## Security Features

### Access Control
- Only template creator can submit for review
- Only admin can approve/reject templates
- Only admin can deploy/retire/archive templates
- Policy holders can only create policies from Active templates

### Validation
- Comprehensive parameter bounds checking
- Coverage amount validation
- Duration validation
- Deductible validation
- Custom parameter type validation
- Template status validation

### Rate Limiting
- Minimum update intervals between template modifications
- Governance approval thresholds
- Time-based restrictions on template changes

## Integration Points

### With Policy Contract
- Templates can generate standard policy structures
- Premium calculation integrates with existing policy validation
- Template policies follow same lifecycle as regular policies

### With Governance Contract
- Template approval through governance proposals
- Rejection workflow through governance
- Threshold-based decision making

### With Risk Pool
- Collateral requirements enforced per template
- Risk level classification affects pool allocations
- Template-specific reserve requirements

## Best Practices

### Template Design
1. Start with simple templates and add complexity gradually
2. Use meaningful parameter names and descriptions
3. Set appropriate min/max bounds for all parameters
4. Consider the risk profile when setting premium models
5. Test templates thoroughly before deployment

### Governance Workflow
1. Provide detailed template documentation for governance review
2. Include risk assessment and pricing rationale
3. Consider community feedback during review process
4. Monitor template performance after deployment
5. Regular template updates should follow same approval process

### Customization
1. Limit the number of custom parameters to avoid complexity
2. Provide sensible default values
3. Validate parameter combinations for logical consistency
4. Document all customization options clearly
5. Test edge cases in parameter combinations

## Error Handling

Common error scenarios and their handling:

- **InvalidTemplateStatus**: Operations attempted on templates in wrong status
- **InvalidInput**: Parameter values outside defined bounds
- **InvalidParameterValue**: Custom parameter type mismatches
- **Unauthorized**: Access control violations
- **NotFound**: Referenced templates or policies don't exist
- **UpdateTooSoon**: Template modification rate limiting

## Testing

The system includes comprehensive unit tests covering:
- Template creation and validation
- Status transition workflows
- Custom parameter handling
- Premium calculation accuracy
- Governance integration
- Access control enforcement
- Error condition handling

## Future Enhancements

Planned improvements:
- Template versioning and rollback capabilities
- Advanced parameter dependencies and constraints
- Machine learning-based risk assessment integration
- Multi-signature deployment workflows
- Template analytics and performance monitoring
- Cross-chain template deployment
- Template marketplace functionality