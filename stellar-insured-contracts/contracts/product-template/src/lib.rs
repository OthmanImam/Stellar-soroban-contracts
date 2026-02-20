#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, Symbol, Vec,
};
use insurance_contracts::shared::types::{
    ProductTemplate, TemplateStatus, ProductCategory, RiskLevel, PremiumModel, 
    CoverageType, CustomParam, TemplateValidationRules, TemplatePolicy, CustomParamValue
};
use insurance_contracts::authorization::{
    get_role, initialize_admin, require_admin, require_governance, Role,
};

#[contract]
pub struct ProductTemplateContract;

// Storage keys
const ADMIN: Symbol = Symbol::short("ADMIN");
const PAUSED: Symbol = Symbol::short("PAUSED");
const CONFIG: Symbol = Symbol::short("CONFIG");
const TEMPLATE: Symbol = Symbol::short("TEMPLATE");
const TEMPLATE_COUNTER: Symbol = Symbol::short("TEMP_CNT");
const TEMPLATE_POLICY: Symbol = Symbol::short("TEMP_POL");
const TEMPLATE_POLICY_COUNTER: Symbol = Symbol::short("TPOL_CNT");
const VALIDATION_RULES: Symbol = Symbol::short("VAL_RULES");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    Unauthorized = 1,
    Paused = 2,
    InvalidInput = 3,
    NotFound = 4,
    AlreadyExists = 5,
    InvalidState = 6,
    NotInitialized = 7,
    AlreadyInitialized = 8,
    InvalidTemplateStatus = 9,
    InvalidParameterValue = 10,
    TemplateValidationFailed = 11,
    GovernanceApprovalRequired = 12,
    UpdateTooSoon = 13,
    InvalidCategory = 14,
    InvalidRiskLevel = 15,
    InvalidPremiumModel = 16,
    InvalidCoverageType = 17,
}

fn is_paused(env: &Env) -> bool {
    env.storage().persistent().get(&PAUSED).unwrap_or(false)
}

fn set_paused(env: &Env, paused: bool) {
    env.storage().persistent().set(&PAUSED, &paused);
}

fn validate_template(template: &ProductTemplate) -> Result<(), ContractError> {
    // Validate coverage amounts
    if template.min_coverage <= 0 || template.max_coverage <= 0 {
        return Err(ContractError::InvalidInput);
    }
    if template.min_coverage > template.max_coverage {
        return Err(ContractError::InvalidInput);
    }
    
    // Validate duration
    if template.min_duration_days == 0 || template.min_duration_days > template.max_duration_days {
        return Err(ContractError::InvalidInput);
    }
    
    // Validate premium rate
    if template.base_premium_rate_bps > 10000 {
        return Err(ContractError::InvalidInput);
    }
    
    // Validate collateral ratio
    if template.collateral_ratio_bps > 10000 {
        return Err(ContractError::InvalidInput);
    }
    
    // Validate deductible
    if template.min_deductible < 0 || template.max_deductible < 0 {
        return Err(ContractError::InvalidInput);
    }
    if template.min_deductible > template.max_deductible {
        return Err(ContractError::InvalidInput);
    }
    
    // Validate custom parameters
    for param in template.custom_params.iter() {
        match param {
            CustomParam::Integer((_, min_value, max_value, default_value)) => {
                if min_value > max_value || default_value < min_value || default_value > max_value {
                    return Err(ContractError::InvalidParameterValue);
                }
            }
            CustomParam::Decimal((_, min_value, max_value, default_value)) => {
                if min_value > max_value || default_value < min_value || default_value > max_value {
                    return Err(ContractError::InvalidParameterValue);
                }
            }
            CustomParam::Choice((_, options, default_index)) => {
                if *default_index >= options.len() as u32 {
                    return Err(ContractError::InvalidParameterValue);
                }
            }
            _ => {}
        }
    }
    
    Ok(())
}

fn can_transition_status(current: TemplateStatus, next: TemplateStatus) -> bool {
    match (current, next) {
        // Draft can go to PendingReview or Archived
        (TemplateStatus::Draft, TemplateStatus::PendingReview) => true,
        (TemplateStatus::Draft, TemplateStatus::Archived) => true,
        
        // PendingReview can go to Approved or Rejected
        (TemplateStatus::PendingReview, TemplateStatus::Approved) => true,
        (TemplateStatus::PendingReview, TemplateStatus::Draft) => true, // Rejection sends back to draft
        
        // Approved can go to Active or Archived
        (TemplateStatus::Approved, TemplateStatus::Active) => true,
        (TemplateStatus::Approved, TemplateStatus::Archived) => true,
        
        // Active can go to Deprecated or Archived
        (TemplateStatus::Active, TemplateStatus::Deprecated) => true,
        (TemplateStatus::Active, TemplateStatus::Archived) => true,
        
        // Deprecated can go to Archived
        (TemplateStatus::Deprecated, TemplateStatus::Archived) => true,
        
        // Self-transitions are not allowed
        (current, next) if current == next => false,
        
        // All other transitions are invalid
        _ => false,
    }
}

#[contractimpl]
impl ProductTemplateContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        governance_contract: Address,
        validation_rules: TemplateValidationRules,
    ) -> Result<(), ContractError> {
        // Check if already initialized
        if env.storage().persistent().has(&ADMIN) {
            return Err(ContractError::AlreadyInitialized);
        }
        
        admin.require_auth();
        initialize_admin(&env, admin.clone());
        
        // Register governance contract as trusted for cross-contract calls
        insurance_contracts::authorization::register_trusted_contract(&env, &admin, &governance_contract)
            .map_err(|_| ContractError::InvalidInput)?;
        
        // Set initial validation rules
        env.storage().persistent().set(&VALIDATION_RULES, &validation_rules);
        env.storage().persistent().set(&TEMPLATE_COUNTER, &0u64);
        env.storage().persistent().set(&TEMPLATE_POLICY_COUNTER, &0u64);
        
        set_paused(&env, false);
        
        env.events().publish((Symbol::new(&env, "initialized"), ()), admin);
        
        Ok(())
    }
    
    pub fn create_template(
        env: Env,
        creator: Address,
        name: Symbol,
        description: Symbol,
        category: ProductCategory,
        risk_level: RiskLevel,
        premium_model: PremiumModel,
        coverage_type: CoverageType,
        min_coverage: i128,
        max_coverage: i128,
        min_duration_days: u32,
        max_duration_days: u32,
        base_premium_rate_bps: u32,
        min_deductible: i128,
        max_deductible: i128,
        collateral_ratio_bps: u32,
        custom_params: Vec<CustomParam>,
    ) -> Result<u64, ContractError> {
        creator.require_auth();
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        let template_id = env.storage().persistent().get(&TEMPLATE_COUNTER).unwrap_or(0) + 1;
        let current_time = env.ledger().timestamp();
        
        let template = ProductTemplate {
            id: template_id,
            name,
            description,
            category,
            status: TemplateStatus::Draft,
            risk_level,
            premium_model,
            coverage_type,
            min_coverage,
            max_coverage,
            min_duration_days,
            max_duration_days,
            base_premium_rate_bps,
            min_deductible,
            max_deductible,
            collateral_ratio_bps,
            custom_params,
            creator: creator.clone(),
            created_at: current_time,
            updated_at: current_time,
            version: 1,
        };
        
        validate_template(&template)?;
        
        env.storage().persistent().set(&(TEMPLATE, template_id), &template);
        env.storage().persistent().set(&TEMPLATE_COUNTER, &template_id);
        
        env.events().publish(
            (Symbol::new(&env, "template_created"), template_id),
            (creator, template.name, template.category),
        );
        
        Ok(template_id)
    }
    
    pub fn update_template(
        env: Env,
        updater: Address,
        template_id: u64,
        name: Option<Symbol>,
        description: Option<Symbol>,
        category: Option<ProductCategory>,
        risk_level: Option<RiskLevel>,
        premium_model: Option<PremiumModel>,
        coverage_type: Option<CoverageType>,
        min_coverage: Option<i128>,
        max_coverage: Option<i128>,
        min_duration_days: Option<u32>,
        max_duration_days: Option<u32>,
        base_premium_rate_bps: Option<u32>,
        min_deductible: Option<i128>,
        max_deductible: Option<i128>,
        collateral_ratio_bps: Option<u32>,
        custom_params: Option<Vec<CustomParam>>,
    ) -> Result<(), ContractError> {
        updater.require_auth();
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        let mut template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Only creator or admin can update
        if template.creator != updater && !matches!(get_role(&env, &updater), Role::Admin) {
            return Err(ContractError::Unauthorized);
        }
        
        // Can only update templates in Draft or Approved status
        if !matches!(template.status, TemplateStatus::Draft | TemplateStatus::Approved) {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Check update interval
        let validation_rules: TemplateValidationRules = env.storage().persistent().get(&VALIDATION_RULES)
            .unwrap_or_else(|| TemplateValidationRules {
                min_collateral_ratio_bps: 1000,
                max_premium_rate_bps: 5000,
                min_duration_days: 1,
                max_duration_days: 365,
                approval_threshold_bps: 5100,
                min_update_interval: 86400, // 24 hours
            });
        
        let current_time = env.ledger().timestamp();
        if current_time - template.updated_at < validation_rules.min_update_interval {
            return Err(ContractError::UpdateTooSoon);
        }
        
        // Update fields if provided
        if let Some(name) = name {
            template.name = name;
        }
        if let Some(description) = description {
            template.description = description;
        }
        if let Some(category) = category {
            template.category = category;
        }
        if let Some(risk_level) = risk_level {
            template.risk_level = risk_level;
        }
        if let Some(premium_model) = premium_model {
            template.premium_model = premium_model;
        }
        if let Some(coverage_type) = coverage_type {
            template.coverage_type = coverage_type;
        }
        if let Some(min_coverage) = min_coverage {
            template.min_coverage = min_coverage;
        }
        if let Some(max_coverage) = max_coverage {
            template.max_coverage = max_coverage;
        }
        if let Some(min_duration_days) = min_duration_days {
            template.min_duration_days = min_duration_days;
        }
        if let Some(max_duration_days) = max_duration_days {
            template.max_duration_days = max_duration_days;
        }
        if let Some(base_premium_rate_bps) = base_premium_rate_bps {
            template.base_premium_rate_bps = base_premium_rate_bps;
        }
        if let Some(min_deductible) = min_deductible {
            template.min_deductible = min_deductible;
        }
        if let Some(max_deductible) = max_deductible {
            template.max_deductible = max_deductible;
        }
        if let Some(collateral_ratio_bps) = collateral_ratio_bps {
            template.collateral_ratio_bps = collateral_ratio_bps;
        }
        if let Some(custom_params) = custom_params {
            template.custom_params = custom_params;
        }
        
        template.updated_at = current_time;
        template.version += 1;
        
        validate_template(&template)?;
        
        env.storage().persistent().set(&(TEMPLATE, template_id), &template);
        
        env.events().publish(
            (Symbol::new(&env, "template_updated"), template_id),
            (updater, template.version),
        );
        
        Ok(())
    }
    
    pub fn change_template_status(
        env: Env,
        admin: Address,
        template_id: u64,
        new_status: TemplateStatus,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        let mut template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        if !can_transition_status(template.status, new_status) {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Special handling for PendingReview status
        if new_status == TemplateStatus::PendingReview {
            // Reset to Draft if going back from PendingReview
            if template.status == TemplateStatus::PendingReview {
                new_status = TemplateStatus::Draft;
            }
        }
        
        template.status = new_status;
        template.updated_at = env.ledger().timestamp();
        
        env.storage().persistent().set(&(TEMPLATE, template_id), &template);
        
        env.events().publish(
            (Symbol::new(&env, "template_status_changed"), template_id),
            (template.status, admin),
        );
        
        Ok(())
    }
    
    pub fn submit_template_for_review(
        env: Env,
        creator: Address,
        template_id: u64,
    ) -> Result<(), ContractError> {
        creator.require_auth();
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        let mut template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Only creator can submit
        if template.creator != creator {
            return Err(ContractError::Unauthorized);
        }
        
        // Must be in Draft status
        if template.status != TemplateStatus::Draft {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        template.status = TemplateStatus::PendingReview;
        template.updated_at = env.ledger().timestamp();
        
        env.storage().persistent().set(&(TEMPLATE, template_id), &template);
        
        env.events().publish(
            (Symbol::new(&env, "template_submitted"), template_id),
            (creator, template.name),
        );
        
        Ok(())
    }
    
    pub fn get_template(env: Env, template_id: u64) -> Result<ProductTemplate, ContractError> {
        let template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        Ok(template)
    }
    
    pub fn get_templates_by_status(
        env: Env,
        status: TemplateStatus,
        start_index: u32,
        limit: u32,
    ) -> Result<Vec<ProductTemplate>, ContractError> {
        let mut templates = Vec::new(&env);
        let template_count = env.storage().persistent().get(&TEMPLATE_COUNTER).unwrap_or(0);
        
        let mut found_count = 0u32;
        let mut added_count = 0u32;
        
        for i in 1..=template_count {
            if let Some(template) = env.storage().persistent().get::<_, ProductTemplate>(&(TEMPLATE, i)) {
                if template.status == status {
                    found_count += 1;
                    if found_count > start_index && added_count < limit {
                        templates.push_back(template);
                        added_count += 1;
                    }
                }
            }
        }
        
        Ok(templates)
    }
    
    pub fn get_templates_by_category(
        env: Env,
        category: ProductCategory,
        start_index: u32,
        limit: u32,
    ) -> Result<Vec<ProductTemplate>, ContractError> {
        let mut templates = Vec::new(&env);
        let template_count = env.storage().persistent().get(&TEMPLATE_COUNTER).unwrap_or(0);
        
        let mut found_count = 0u32;
        let mut added_count = 0u32;
        
        for i in 1..=template_count {
            if let Some(template) = env.storage().persistent().get::<_, ProductTemplate>(&(TEMPLATE, i)) {
                if template.category == category {
                    found_count += 1;
                    if found_count > start_index && added_count < limit {
                        templates.push_back(template);
                        added_count += 1;
                    }
                }
            }
        }
        
        Ok(templates)
    }
    
    pub fn get_active_templates(env: Env) -> Result<Vec<ProductTemplate>, ContractError> {
        Self::get_templates_by_status(env, TemplateStatus::Active, 0, 100)
    }
    
    pub fn pause(env: Env, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;
        
        set_paused(&env, true);
        
        env.events().publish((Symbol::new(&env, "paused"), ()), admin);
        
        Ok(())
    }
    
    pub fn unpause(env: Env, admin: Address) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;
        
        set_paused(&env, false);
        
        env.events().publish((Symbol::new(&env, "unpaused"), ()), admin);
        
        Ok(())
    }
    
    pub fn is_contract_paused(env: Env) -> bool {
        is_paused(&env)
    }
    
    pub fn get_template_count(env: Env) -> Result<u64, ContractError> {
        let count = env.storage().persistent().get(&TEMPLATE_COUNTER).unwrap_or(0);
        Ok(count)
    }
    
    pub fn get_validation_rules(env: Env) -> Result<TemplateValidationRules, ContractError> {
        let rules: TemplateValidationRules = env.storage().persistent().get(&VALIDATION_RULES)
            .ok_or(ContractError::NotInitialized)?;
        Ok(rules)
    }
    
    pub fn update_validation_rules(
        env: Env,
        admin: Address,
        new_rules: TemplateValidationRules,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;
        
        // Validate the new rules
        if new_rules.min_collateral_ratio_bps > 10000 || 
           new_rules.max_premium_rate_bps > 10000 ||
           new_rules.approval_threshold_bps > 10000 {
            return Err(ContractError::InvalidInput);
        }
        
        if new_rules.min_duration_days > new_rules.max_duration_days {
            return Err(ContractError::InvalidInput);
        }
        
        env.storage().persistent().set(&VALIDATION_RULES, &new_rules);
        
        env.events().publish(
            (Symbol::new(&env, "validation_rules_updated"), ()),
            admin,
        );
        
        Ok(())
    }
    
    // ============================================================
    // TEMPLATE POLICY CREATION WITH CUSTOMIZATION
    // ============================================================
    
    pub fn create_policy_from_template(
        env: Env,
        holder: Address,
        template_id: u64,
        coverage_amount: i128,
        duration_days: u32,
        deductible: i128,
        custom_values: Vec<CustomParamValue>,
    ) -> Result<u64, ContractError> {
        holder.require_auth();
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        // Get template
        let template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Template must be active
        if template.status != TemplateStatus::Active {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Validate coverage amount
        if coverage_amount < template.min_coverage || coverage_amount > template.max_coverage {
            return Err(ContractError::InvalidInput);
        }
        
        // Validate duration
        if duration_days < template.min_duration_days || duration_days > template.max_duration_days {
            return Err(ContractError::InvalidInput);
        }
        
        // Validate deductible
        if deductible < template.min_deductible || deductible > template.max_deductible {
            return Err(ContractError::InvalidInput);
        }
        
        // Validate custom parameters
        Self::validate_custom_parameters(&env, &template, &custom_values)?;
        
        // Calculate premium based on template model
        let premium_amount = Self::calculate_premium(
            &env,
            &template,
            coverage_amount,
            duration_days,
            &custom_values,
        )?;
        
        // Generate policy ID
        let policy_id = env.storage().persistent().get(&TEMPLATE_POLICY_COUNTER).unwrap_or(0) + 1;
        let current_time = env.ledger().timestamp();
        let start_time = current_time;
        let end_time = current_time + (duration_days as u64 * 86400);
        
        // Create template policy
        let template_policy = TemplatePolicy {
            policy_id,
            template_id,
            holder: holder.clone(),
            coverage_amount,
            premium_amount,
            duration_days,
            deductible,
            custom_values,
            created_at: current_time,
            start_time,
            end_time,
        };
        
        // Store the policy
        env.storage().persistent().set(&(TEMPLATE_POLICY, policy_id), &template_policy);
        env.storage().persistent().set(&TEMPLATE_POLICY_COUNTER, &policy_id);
        
        // Emit event
        env.events().publish(
            (Symbol::new(&env, "policy_created_from_template"), policy_id),
            (holder, template_id, coverage_amount, premium_amount),
        );
        
        Ok(policy_id)
    }
    
    fn validate_custom_parameters(
        env: &Env,
        template: &ProductTemplate,
        custom_values: &Vec<CustomParamValue>,
    ) -> Result<(), ContractError> {
        // Create a map of expected parameters
        let mut expected_params = Vec::new(&env);
        for param in template.custom_params.iter() {
            match param {
                CustomParam::Integer((name, _, _, _)) => {
                    expected_params.push_back(name.clone());
                }
                CustomParam::Decimal((name, _, _, _)) => {
                    expected_params.push_back(name.clone());
                }
                CustomParam::Boolean((name, _)) => {
                    expected_params.push_back(name.clone());
                }
                CustomParam::Choice((name, _, _)) => {
                    expected_params.push_back(name.clone());
                }
            }
        }
        
        // Validate each provided value
        for value in custom_values.iter() {
            // Check if parameter exists in template
            let mut found = false;
            for param in template.custom_params.iter() {
                let param_name = match param {
                    CustomParam::Integer((name, _, _, _)) => name,
                    CustomParam::Decimal((name, _, _, _)) => name,
                    CustomParam::Boolean((name, _)) => name,
                    CustomParam::Choice((name, _, _)) => name,
                };
                
                if &value.name == param_name {
                    found = true;
                    
                    // Validate value type and constraints
                    match (param, &value.value) {
                        (CustomParam::Integer((_, min_value, max_value, _)), CustomParamValueData::Integer(val)) => {
                            if val < min_value || val > max_value {
                                return Err(ContractError::InvalidParameterValue);
                            }
                        }
                        (CustomParam::Decimal((_, min_value, max_value, _)), CustomParamValueData::Decimal(val)) => {
                            if val < min_value || val > max_value {
                                return Err(ContractError::InvalidParameterValue);
                            }
                        }
                        (CustomParam::Boolean((_, _)), CustomParamValueData::Boolean(_)) => {
                            // Boolean values are always valid
                        }
                        (CustomParam::Choice((_, options, _)), CustomParamValueData::Choice(index)) => {
                            if *index >= options.len() as u32 {
                                return Err(ContractError::InvalidParameterValue);
                            }
                        }
                        _ => {
                            // Type mismatch
                            return Err(ContractError::InvalidParameterValue);
                        }
                    }
                    break;
                }
            }
            
            if !found {
                return Err(ContractError::InvalidParameterValue);
            }
        }
        
        // Check if all required parameters are provided
        for expected_name in expected_params.iter() {
            let mut provided = false;
            for value in custom_values.iter() {
                if &value.name == expected_name {
                    provided = true;
                    break;
                }
            }
            if !provided {
                return Err(ContractError::InvalidParameterValue);
            }
        }
        
        Ok(())
    }
    
    fn calculate_premium(
        env: &Env,
        template: &ProductTemplate,
        coverage_amount: i128,
        duration_days: u32,
        custom_values: &Vec<CustomParamValue>,
    ) -> Result<i128, ContractError> {
        let mut premium: i128 = 0;
        
        match template.premium_model {
            PremiumModel::Fixed => {
                // Fixed premium amount
                premium = template.base_premium_rate_bps as i128 * 1000000; // Convert basis points
            }
            PremiumModel::Percentage => {
                // Percentage of coverage amount
                premium = (coverage_amount * template.base_premium_rate_bps as i128) / 10000;
            }
            PremiumModel::RiskBased => {
                // Risk-based calculation using risk level multiplier
                let risk_multiplier = match template.risk_level {
                    RiskLevel::Low => 8000,      // 0.8x
                    RiskLevel::Medium => 10000,  // 1.0x
                    RiskLevel::High => 15000,    // 1.5x
                    RiskLevel::VeryHigh => 25000, // 2.5x
                };
                
                let base_premium = (coverage_amount * template.base_premium_rate_bps as i128) / 10000;
                premium = (base_premium * risk_multiplier) / 10000;
            }
            PremiumModel::Tiered => {
                // Tiered pricing based on coverage amount
                let tier_multiplier = if coverage_amount <= 100000000 {
                    10000  // 1.0x for small coverage
                } else if coverage_amount <= 1000000000 {
                    9000   // 0.9x for medium coverage
                } else {
                    8000   // 0.8x for large coverage
                };
                
                let base_premium = (coverage_amount * template.base_premium_rate_bps as i128) / 10000;
                premium = (base_premium * tier_multiplier) / 10000;
            }
        }
        
        // Apply duration adjustment
        let duration_multiplier = (duration_days as i128 * 10000) / 365; // Pro-rate for year
        premium = (premium * duration_multiplier) / 10000;
        
        // Apply custom parameter adjustments
        for value in custom_values.iter() {
            // Example: Additional coverage options increase premium
            if value.name == Symbol::new(env, "additional_coverage") {
                if let CustomParamValueData::Boolean(true) = value.value {
                    premium = (premium * 12000) / 10000; // 20% increase
                }
            }
            
            // Example: Higher deductible reduces premium
            if value.name == Symbol::new(env, "high_deductible") {
                if let CustomParamValueData::Boolean(true) = value.value {
                    premium = (premium * 8000) / 10000; // 20% reduction
                }
            }
        }
        
        Ok(premium)
    }
    
    pub fn get_template_policy(env: Env, policy_id: u64) -> Result<TemplatePolicy, ContractError> {
        let policy: TemplatePolicy = env.storage().persistent().get(&(TEMPLATE_POLICY, policy_id))
            .ok_or(ContractError::NotFound)?;
        Ok(policy)
    }
    
    pub fn get_policies_by_holder(
        env: Env,
        holder: Address,
        start_index: u32,
        limit: u32,
    ) -> Result<Vec<TemplatePolicy>, ContractError> {
        let mut policies = Vec::new(&env);
        let policy_count = env.storage().persistent().get(&TEMPLATE_POLICY_COUNTER).unwrap_or(0);
        
        let mut found_count = 0u32;
        let mut added_count = 0u32;
        
        for i in 1..=policy_count {
            if let Some(policy) = env.storage().persistent().get::<_, TemplatePolicy>(&(TEMPLATE_POLICY, i)) {
                if policy.holder == holder {
                    found_count += 1;
                    if found_count > start_index && added_count < limit {
                        policies.push_back(policy);
                        added_count += 1;
                    }
                }
            }
        }
        
        Ok(policies)
    }
    
    pub fn get_policies_by_template(
        env: Env,
        template_id: u64,
        start_index: u32,
        limit: u32,
    ) -> Result<Vec<TemplatePolicy>, ContractError> {
        let mut policies = Vec::new(&env);
        let policy_count = env.storage().persistent().get(&TEMPLATE_POLICY_COUNTER).unwrap_or(0);
        
        let mut found_count = 0u32;
        let mut added_count = 0u32;
        
        for i in 1..=policy_count {
            if let Some(policy) = env.storage().persistent().get::<_, TemplatePolicy>(&(TEMPLATE_POLICY, i)) {
                if policy.template_id == template_id {
                    found_count += 1;
                    if found_count > start_index && added_count < limit {
                        policies.push_back(policy);
                        added_count += 1;
                    }
                }
            }
        }
        
        Ok(policies)
    }
    
    pub fn get_template_policy_count(env: Env) -> Result<u64, ContractError> {
        let count = env.storage().persistent().get(&TEMPLATE_POLICY_COUNTER).unwrap_or(0);
        Ok(count)
    }
    
    // ============================================================
    // GOVERNANCE INTEGRATION FOR TEMPLATE APPROVAL
    // ============================================================
    
    pub fn propose_template_approval(
        env: Env,
        proposer: Address,
        template_id: u64,
        title: Symbol,
        description: Symbol,
        threshold_percentage: u32,
    ) -> Result<u64, ContractError> {
        proposer.require_auth();
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        // Validate threshold
        if threshold_percentage == 0 || threshold_percentage > 100 {
            return Err(ContractError::InvalidInput);
        }
        
        // Get template
        let template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Template must be in PendingReview status
        if template.status != TemplateStatus::PendingReview {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Create governance proposal through cross-contract call
        // This would call the governance contract to create a proposal
        // For now, we'll emit an event and return a mock proposal ID
        
        let proposal_id = env.storage().persistent().get(&TEMPLATE_COUNTER).unwrap_or(0) + 1000000; // Mock ID space
        
        env.events().publish(
            (Symbol::new(&env, "template_approval_proposed"), proposal_id),
            (proposer, template_id, title, threshold_percentage),
        );
        
        Ok(proposal_id)
    }
    
    pub fn execute_template_approval(
        env: Env,
        executor: Address,
        proposal_id: u64,
        template_id: u64,
    ) -> Result<(), ContractError> {
        executor.require_auth();
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        // In a real implementation, this would verify the governance proposal passed
        // For now, we'll assume it passed and approve the template
        
        let mut template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Template must be in PendingReview status
        if template.status != TemplateStatus::PendingReview {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Approve the template
        template.status = TemplateStatus::Approved;
        template.updated_at = env.ledger().timestamp();
        
        env.storage().persistent().set(&(TEMPLATE, template_id), &template);
        
        env.events().publish(
            (Symbol::new(&env, "template_approved"), template_id),
            (executor, proposal_id),
        );
        
        Ok(())
    }
    
    pub fn propose_template_rejection(
        env: Env,
        proposer: Address,
        template_id: u64,
        title: Symbol,
        description: Symbol,
        reason: Symbol,
        threshold_percentage: u32,
    ) -> Result<u64, ContractError> {
        proposer.require_auth();
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        // Validate threshold
        if threshold_percentage == 0 || threshold_percentage > 100 {
            return Err(ContractError::InvalidInput);
        }
        
        // Get template
        let template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Template must be in PendingReview status
        if template.status != TemplateStatus::PendingReview {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Create governance proposal for rejection
        let proposal_id = env.storage().persistent().get(&TEMPLATE_COUNTER).unwrap_or(0) + 2000000; // Different ID space
        
        env.events().publish(
            (Symbol::new(&env, "template_rejection_proposed"), proposal_id),
            (proposer, template_id, title, reason, threshold_percentage),
        );
        
        Ok(proposal_id)
    }
    
    pub fn execute_template_rejection(
        env: Env,
        executor: Address,
        proposal_id: u64,
        template_id: u64,
        reason: Symbol,
    ) -> Result<(), ContractError> {
        executor.require_auth();
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        // In a real implementation, this would verify the governance proposal passed
        // For now, we'll assume it passed and reject the template
        
        let mut template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Template must be in PendingReview status
        if template.status != TemplateStatus::PendingReview {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Reject the template (send back to Draft)
        template.status = TemplateStatus::Draft;
        template.updated_at = env.ledger().timestamp();
        
        env.storage().persistent().set(&(TEMPLATE, template_id), &template);
        
        env.events().publish(
            (Symbol::new(&env, "template_rejected"), template_id),
            (executor, proposal_id, reason),
        );
        
        Ok(())
    }
    
    pub fn get_template_approval_status(
        env: Env,
        template_id: u64,
    ) -> Result<(TemplateStatus, Option<u64>, Option<u64>), ContractError> {
        let template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // In a real implementation, this would query the governance contract
        // for active proposals related to this template
        // For now, we'll return mock proposal IDs
        
        let approval_proposal_id = if template.status == TemplateStatus::PendingReview {
            Some(template_id + 1000000)
        } else {
            None
        };
        
        let rejection_proposal_id = if template.status == TemplateStatus::PendingReview {
            Some(template_id + 2000000)
        } else {
            None
        };
        
        Ok((template.status, approval_proposal_id, rejection_proposal_id))
    }
    
    // ============================================================
    // TEMPLATE DEPLOYMENT WORKFLOW
    // ============================================================
    
    pub fn deploy_template(
        env: Env,
        admin: Address,
        template_id: u64,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        let mut template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Template must be Approved to be deployed
        if template.status != TemplateStatus::Approved {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Deploy the template (make it Active)
        template.status = TemplateStatus::Active;
        template.updated_at = env.ledger().timestamp();
        
        env.storage().persistent().set(&(TEMPLATE, template_id), &template);
        
        env.events().publish(
            (Symbol::new(&env, "template_deployed"), template_id),
            (admin, template.name),
        );
        
        Ok(())
    }
    
    pub fn retire_template(
        env: Env,
        admin: Address,
        template_id: u64,
        reason: Symbol,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        let mut template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Template must be Active or Approved to be retired
        if !matches!(template.status, TemplateStatus::Active | TemplateStatus::Approved) {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Retire the template (mark as Deprecated)
        template.status = TemplateStatus::Deprecated;
        template.updated_at = env.ledger().timestamp();
        
        env.storage().persistent().set(&(TEMPLATE, template_id), &template);
        
        env.events().publish(
            (Symbol::new(&env, "template_retired"), template_id),
            (admin, reason),
        );
        
        Ok(())
    }
    
    pub fn archive_template(
        env: Env,
        admin: Address,
        template_id: u64,
        reason: Symbol,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin(&env, &admin)?;
        
        if is_paused(&env) {
            return Err(ContractError::Paused);
        }
        
        let mut template: ProductTemplate = env.storage().persistent().get(&(TEMPLATE, template_id))
            .ok_or(ContractError::NotFound)?;
        
        // Template can be archived from any status except already Archived
        if template.status == TemplateStatus::Archived {
            return Err(ContractError::InvalidTemplateStatus);
        }
        
        // Archive the template
        template.status = TemplateStatus::Archived;
        template.updated_at = env.ledger().timestamp();
        
        env.storage().persistent().set(&(TEMPLATE, template_id), &template);
        
        env.events().publish(
            (Symbol::new(&env, "template_archived"), template_id),
            (admin, reason),
        );
        
        Ok(())
    }
}

#[cfg(test)]
mod test;
