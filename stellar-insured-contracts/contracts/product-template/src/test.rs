#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Address, Env, Symbol, Vec};

    fn setup_test_env() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let governance_contract = Address::generate(&env);
        
        (env, admin, governance_contract)
    }
    
    fn initialize_contract(env: &Env, admin: &Address, governance: &Address) {
        let validation_rules = TemplateValidationRules {
            min_collateral_ratio_bps: 1000,
            max_premium_rate_bps: 5000,
            min_duration_days: 1,
            max_duration_days: 365,
            approval_threshold_bps: 5100,
            min_update_interval: 3600, // 1 hour for testing
        };
        
        ProductTemplateContract::initialize(
            env.clone(),
            admin.clone(),
            governance.clone(),
            validation_rules,
        ).unwrap();
    }
    
    fn create_test_template(env: &Env, creator: &Address) -> u64 {
        ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(env, "Home Insurance"),
            Symbol::new(env, "Standard home insurance template"),
            ProductCategory::Property,
            RiskLevel::Medium,
            PremiumModel::Percentage,
            CoverageType::Full,
            1000000, // 1 unit min coverage
            1000000000, // 1000 units max coverage
            30, // 30 days min
            365, // 365 days max
            200, // 2% base premium
            50000, // 0.05 unit min deductible
            1000000, // 1 unit max deductible
            1500, // 15% collateral ratio
            Vec::new(env),
        ).unwrap()
    }
    
    // ============================================================
    // INITIALIZATION TESTS
    // ============================================================
    
    #[test]
    fn test_initialize_success() {
        let (env, admin, governance) = setup_test_env();
        
        let validation_rules = TemplateValidationRules {
            min_collateral_ratio_bps: 1000,
            max_premium_rate_bps: 5000,
            min_duration_days: 1,
            max_duration_days: 365,
            approval_threshold_bps: 5100,
            min_update_interval: 3600,
        };
        
        let result = ProductTemplateContract::initialize(
            env.clone(),
            admin.clone(),
            governance.clone(),
            validation_rules.clone(),
        );
        
        assert!(result.is_ok());
        
        let rules = ProductTemplateContract::get_validation_rules(env.clone()).unwrap();
        assert_eq!(rules.min_collateral_ratio_bps, validation_rules.min_collateral_ratio_bps);
        assert_eq!(rules.max_premium_rate_bps, validation_rules.max_premium_rate_bps);
    }
    
    #[test]
    fn test_initialize_already_initialized() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let validation_rules = TemplateValidationRules {
            min_collateral_ratio_bps: 1000,
            max_premium_rate_bps: 5000,
            min_duration_days: 1,
            max_duration_days: 365,
            approval_threshold_bps: 5100,
            min_update_interval: 3600,
        };
        
        let result = ProductTemplateContract::initialize(
            env.clone(),
            admin.clone(),
            governance.clone(),
            validation_rules,
        );
        
        assert_eq!(result, Err(ContractError::AlreadyInitialized));
    }
    
    // ============================================================
    // TEMPLATE CREATION TESTS
    // ============================================================
    
    #[test]
    fn test_create_template_success() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        assert_eq!(template_id, 1);
        
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.id, template_id);
        assert_eq!(template.name, Symbol::new(&env, "Home Insurance"));
        assert_eq!(template.status, TemplateStatus::Draft);
        assert_eq!(template.creator, creator);
    }
    
    #[test]
    fn test_create_template_invalid_coverage() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        
        let result = ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(&env, "Invalid Template"),
            Symbol::new(&env, "Template with invalid coverage"),
            ProductCategory::Property,
            RiskLevel::Medium,
            PremiumModel::Percentage,
            CoverageType::Full,
            1000000, // min
            500000,  // max < min - INVALID
            30,
            365,
            200,
            50000,
            1000000,
            1500,
            Vec::new(&env),
        );
        
        assert_eq!(result, Err(ContractError::InvalidInput));
    }
    
    #[test]
    fn test_create_template_invalid_duration() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        
        let result = ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(&env, "Invalid Template"),
            Symbol::new(&env, "Template with invalid duration"),
            ProductCategory::Property,
            RiskLevel::Medium,
            PremiumModel::Percentage,
            CoverageType::Full,
            1000000,
            1000000000,
            365, // min
            30,  // max < min - INVALID
            200,
            50000,
            1000000,
            1500,
            Vec::new(&env),
        );
        
        assert_eq!(result, Err(ContractError::InvalidInput));
    }
    
    #[test]
    fn test_create_multiple_templates() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        
        let id1 = create_test_template(&env, &creator);
        let id2 = create_test_template(&env, &creator);
        let id3 = create_test_template(&env, &creator);
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        
        let count = ProductTemplateContract::get_template_count(env.clone()).unwrap();
        assert_eq!(count, 3);
    }
    
    // ============================================================
    // TEMPLATE STATUS TRANSITION TESTS
    // ============================================================
    
    #[test]
    fn test_submit_template_for_review() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        let result = ProductTemplateContract::submit_template_for_review(
            env.clone(),
            creator.clone(),
            template_id,
        );
        
        assert!(result.is_ok());
        
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.status, TemplateStatus::PendingReview);
    }
    
    #[test]
    fn test_submit_template_for_review_unauthorized() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let unauthorized = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        let result = ProductTemplateContract::submit_template_for_review(
            env.clone(),
            unauthorized.clone(),
            template_id,
        );
        
        assert_eq!(result, Err(ContractError::Unauthorized));
    }
    
    #[test]
    fn test_submit_template_for_review_wrong_status() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Submit for review first
        ProductTemplateContract::submit_template_for_review(
            env.clone(),
            creator.clone(),
            template_id,
        ).unwrap();
        
        // Try to submit again - should fail
        let result = ProductTemplateContract::submit_template_for_review(
            env.clone(),
            creator.clone(),
            template_id,
        );
        
        assert_eq!(result, Err(ContractError::InvalidTemplateStatus));
    }
    
    #[test]
    fn test_change_template_status_admin_only() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Submit for review
        ProductTemplateContract::submit_template_for_review(
            env.clone(),
            creator.clone(),
            template_id,
        ).unwrap();
        
        // Admin approves
        let result = ProductTemplateContract::change_template_status(
            env.clone(),
            admin.clone(),
            template_id,
            TemplateStatus::Approved,
        );
        
        assert!(result.is_ok());
        
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.status, TemplateStatus::Approved);
    }
    
    #[test]
    fn test_change_template_status_unauthorized() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let unauthorized = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        let result = ProductTemplateContract::change_template_status(
            env.clone(),
            unauthorized.clone(),
            template_id,
            TemplateStatus::Approved,
        );
        
        assert_eq!(result, Err(ContractError::Unauthorized));
    }
    
    #[test]
    fn test_template_status_transitions() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Draft -> PendingReview
        ProductTemplateContract::submit_template_for_review(
            env.clone(),
            creator.clone(),
            template_id,
        ).unwrap();
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.status, TemplateStatus::PendingReview);
        
        // PendingReview -> Approved
        ProductTemplateContract::change_template_status(
            env.clone(),
            admin.clone(),
            template_id,
            TemplateStatus::Approved,
        ).unwrap();
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.status, TemplateStatus::Approved);
        
        // Approved -> Active
        ProductTemplateContract::change_template_status(
            env.clone(),
            admin.clone(),
            template_id,
            TemplateStatus::Active,
        ).unwrap();
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.status, TemplateStatus::Active);
        
        // Active -> Deprecated
        ProductTemplateContract::change_template_status(
            env.clone(),
            admin.clone(),
            template_id,
            TemplateStatus::Deprecated,
        ).unwrap();
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.status, TemplateStatus::Deprecated);
        
        // Deprecated -> Archived
        ProductTemplateContract::change_template_status(
            env.clone(),
            admin.clone(),
            template_id,
            TemplateStatus::Archived,
        ).unwrap();
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.status, TemplateStatus::Archived);
    }
    
    // ============================================================
    // TEMPLATE UPDATE TESTS
    // ============================================================
    
    #[test]
    fn test_update_template_success() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Advance time to allow updates
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 3601,
            protocol_version: 20,
            sequence_number: env.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 100000,
        });
        
        let result = ProductTemplateContract::update_template(
            env.clone(),
            creator.clone(),
            template_id,
            Some(Symbol::new(&env, "Updated Home Insurance")),
            None, // description
            None, // category
            None, // risk_level
            None, // premium_model
            None, // coverage_type
            None, // min_coverage
            None, // max_coverage
            None, // min_duration_days
            None, // max_duration_days
            None, // base_premium_rate_bps
            None, // min_deductible
            None, // max_deductible
            None, // collateral_ratio_bps
            None, // custom_params
        );
        
        assert!(result.is_ok());
        
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.name, Symbol::new(&env, "Updated Home Insurance"));
        assert_eq!(template.version, 2);
    }
    
    #[test]
    fn test_update_template_unauthorized() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let unauthorized = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Advance time
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 3601,
            protocol_version: 20,
            sequence_number: env.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 100000,
        });
        
        let result = ProductTemplateContract::update_template(
            env.clone(),
            unauthorized.clone(),
            template_id,
            Some(Symbol::new(&env, "Unauthorized Update")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        
        assert_eq!(result, Err(ContractError::Unauthorized));
    }
    
    #[test]
    fn test_update_template_wrong_status() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Submit for review to change status
        ProductTemplateContract::submit_template_for_review(
            env.clone(),
            creator.clone(),
            template_id,
        ).unwrap();
        
        // Advance time
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 3601,
            protocol_version: 20,
            sequence_number: env.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 100000,
        });
        
        let result = ProductTemplateContract::update_template(
            env.clone(),
            creator.clone(),
            template_id,
            Some(Symbol::new(&env, "Update in wrong status")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        
        assert_eq!(result, Err(ContractError::InvalidTemplateStatus));
    }
    
    #[test]
    fn test_update_template_too_soon() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Don't advance time - update should fail
        let result = ProductTemplateContract::update_template(
            env.clone(),
            creator.clone(),
            template_id,
            Some(Symbol::new(&env, "Too soon update")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        
        assert_eq!(result, Err(ContractError::UpdateTooSoon));
    }
    
    // ============================================================
    // QUERY TESTS
    // ============================================================
    
    #[test]
    fn test_get_templates_by_status() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        
        // Create templates with different statuses
        let template1 = create_test_template(&env, &creator);
        let template2 = create_test_template(&env, &creator);
        let template3 = create_test_template(&env, &creator);
        
        // Set different statuses
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template1).unwrap();
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template2).unwrap();
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template3).unwrap();
        
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template1, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template2, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template3, TemplateStatus::Active).unwrap();
        
        let approved_templates = ProductTemplateContract::get_templates_by_status(
            env.clone(),
            TemplateStatus::Approved,
            0,
            10,
        ).unwrap();
        
        assert_eq!(approved_templates.len(), 2);
        
        let active_templates = ProductTemplateContract::get_templates_by_status(
            env.clone(),
            TemplateStatus::Active,
            0,
            10,
        ).unwrap();
        
        assert_eq!(active_templates.len(), 1);
    }
    
    #[test]
    fn test_get_templates_by_category() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        
        // Create templates with different categories
        let template1 = ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(&env, "Home Insurance"),
            Symbol::new(&env, "Property insurance"),
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
            Vec::new(&env),
        ).unwrap();
        
        let template2 = ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(&env, "Auto Insurance"),
            Symbol::new(&env, "Vehicle insurance"),
            ProductCategory::Auto,
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
            Vec::new(&env),
        ).unwrap();
        
        let property_templates = ProductTemplateContract::get_templates_by_category(
            env.clone(),
            ProductCategory::Property,
            0,
            10,
        ).unwrap();
        
        assert_eq!(property_templates.len(), 1);
        assert_eq!(property_templates.get(0).unwrap().id, template1);
        
        let auto_templates = ProductTemplateContract::get_templates_by_category(
            env.clone(),
            ProductCategory::Auto,
            0,
            10,
        ).unwrap();
        
        assert_eq!(auto_templates.len(), 1);
        assert_eq!(auto_templates.get(0).unwrap().id, template2);
    }
    
    #[test]
    fn test_get_active_templates() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        
        let template1 = create_test_template(&env, &creator);
        let template2 = create_test_template(&env, &creator);
        
        // Make template2 active
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template2).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template2, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template2, TemplateStatus::Active).unwrap();
        
        let active_templates = ProductTemplateContract::get_active_templates(env.clone()).unwrap();
        
        assert_eq!(active_templates.len(), 1);
        assert_eq!(active_templates.get(0).unwrap().id, template2);
    }
    
    // ============================================================
    // TEMPLATE POLICY CREATION TESTS
    // ============================================================
    
    #[test]
    fn test_create_policy_from_template_success() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let holder = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Make template active
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template_id).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Active).unwrap();
        
        let custom_values = Vec::new(&env);
        
        let policy_id = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template_id,
            10000000, // 10 units coverage
            90,       // 90 days duration
            100000,   // 0.1 unit deductible
            custom_values,
        ).unwrap();
        
        assert_eq!(policy_id, 1);
        
        let policy = ProductTemplateContract::get_template_policy(env.clone(), policy_id).unwrap();
        assert_eq!(policy.policy_id, policy_id);
        assert_eq!(policy.template_id, template_id);
        assert_eq!(policy.holder, holder);
        assert_eq!(policy.coverage_amount, 10000000);
        assert_eq!(policy.duration_days, 90);
        assert_eq!(policy.deductible, 100000);
    }
    
    #[test]
    fn test_create_policy_from_template_invalid_status() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let holder = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Template is still in Draft status
        let custom_values = Vec::new(&env);
        
        let result = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template_id,
            10000000,
            90,
            100000,
            custom_values,
        );
        
        assert_eq!(result, Err(ContractError::InvalidTemplateStatus));
    }
    
    #[test]
    fn test_create_policy_from_template_invalid_coverage() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let holder = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Make template active
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template_id).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Active).unwrap();
        
        let custom_values = Vec::new(&env);
        
        // Test coverage below minimum
        let result1 = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template_id,
            100000, // Below min of 1000000
            90,
            100000,
            custom_values.clone(),
        );
        
        assert_eq!(result1, Err(ContractError::InvalidInput));
        
        // Test coverage above maximum
        let result2 = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template_id,
            2000000000, // Above max of 1000000000
            90,
            100000,
            custom_values,
        );
        
        assert_eq!(result2, Err(ContractError::InvalidInput));
    }
    
    #[test]
    fn test_create_policy_from_template_invalid_duration() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let holder = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Make template active
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template_id).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Active).unwrap();
        
        let custom_values = Vec::new(&env);
        
        // Test duration below minimum
        let result1 = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template_id,
            10000000,
            15, // Below min of 30
            100000,
            custom_values.clone(),
        );
        
        assert_eq!(result1, Err(ContractError::InvalidInput));
        
        // Test duration above maximum
        let result2 = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template_id,
            10000000,
            500, // Above max of 365
            100000,
            custom_values,
        );
        
        assert_eq!(result2, Err(ContractError::InvalidInput));
    }
    
    #[test]
    fn test_create_policy_from_template_with_custom_parameters() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let holder = Address::generate(&env);
        
        // Create template with custom parameters
        let mut custom_params = Vec::new(&env);
        custom_params.push_back(CustomParam::Boolean((
            Symbol::new(&env, "additional_coverage"),
            false,
        )));
        custom_params.push_back(CustomParam::Integer((
            Symbol::new(&env, "extra_protection_level"),
            0,
            100,
            50,
        )));
        
        let template_id = ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(&env, "Custom Insurance"),
            Symbol::new(&env, "Template with custom parameters"),
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
        ).unwrap();
        
        // Make template active
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template_id).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Active).unwrap();
        
        // Create custom values
        let mut custom_values = Vec::new(&env);
        custom_values.push_back(CustomParamValue {
            name: Symbol::new(&env, "additional_coverage"),
            value: CustomParamValueData::Boolean(true),
        });
        custom_values.push_back(CustomParamValue {
            name: Symbol::new(&env, "extra_protection_level"),
            value: CustomParamValueData::Integer(75),
        });
        
        let policy_id = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template_id,
            10000000,
            90,
            100000,
            custom_values,
        ).unwrap();
        
        assert_eq!(policy_id, 1);
        
        let policy = ProductTemplateContract::get_template_policy(env.clone(), policy_id).unwrap();
        assert_eq!(policy.custom_values.len(), 2);
    }
    
    #[test]
    fn test_create_policy_from_template_invalid_custom_parameters() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let holder = Address::generate(&env);
        
        // Create template with custom parameters
        let mut custom_params = Vec::new(&env);
        custom_params.push_back(CustomParam::Integer((
            Symbol::new(&env, "protection_level"),
            0,
            100,
            50,
        )));
        
        let template_id = ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(&env, "Custom Insurance"),
            Symbol::new(&env, "Template with custom parameters"),
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
        ).unwrap();
        
        // Make template active
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template_id).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Active).unwrap();
        
        // Test invalid custom parameter value (out of range)
        let mut invalid_custom_values = Vec::new(&env);
        invalid_custom_values.push_back(CustomParamValue {
            name: Symbol::new(&env, "protection_level"),
            value: CustomParamValueData::Integer(150), // Above max of 100
        });
        
        let result = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template_id,
            10000000,
            90,
            100000,
            invalid_custom_values,
        );
        
        assert_eq!(result, Err(ContractError::InvalidParameterValue));
    }
    
    #[test]
    fn test_get_policies_by_holder() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let holder1 = Address::generate(&env);
        let holder2 = Address::generate(&env);
        
        let template_id = create_test_template(&env, &creator);
        
        // Make template active
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template_id).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Active).unwrap();
        
        let custom_values = Vec::new(&env);
        
        // Create policies for different holders
        ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder1.clone(),
            template_id,
            10000000,
            90,
            100000,
            custom_values.clone(),
        ).unwrap();
        
        ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder1.clone(),
            template_id,
            20000000,
            180,
            200000,
            custom_values.clone(),
        ).unwrap();
        
        ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder2.clone(),
            template_id,
            15000000,
            120,
            150000,
            custom_values,
        ).unwrap();
        
        let holder1_policies = ProductTemplateContract::get_policies_by_holder(
            env.clone(),
            holder1.clone(),
            0,
            10,
        ).unwrap();
        
        assert_eq!(holder1_policies.len(), 2);
        
        let holder2_policies = ProductTemplateContract::get_policies_by_holder(
            env.clone(),
            holder2.clone(),
            0,
            10,
        ).unwrap();
        
        assert_eq!(holder2_policies.len(), 1);
    }
    
    #[test]
    fn test_get_policies_by_template() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let holder = Address::generate(&env);
        
        let template1 = create_test_template(&env, &creator);
        let template2 = create_test_template(&env, &creator);
        
        // Make both templates active
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template1).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template1, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template1, TemplateStatus::Active).unwrap();
        
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template2).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template2, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template2, TemplateStatus::Active).unwrap();
        
        let custom_values = Vec::new(&env);
        
        // Create policies from different templates
        ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template1,
            10000000,
            90,
            100000,
            custom_values.clone(),
        ).unwrap();
        
        ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template1,
            20000000,
            180,
            200000,
            custom_values.clone(),
        ).unwrap();
        
        ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            template2,
            15000000,
            120,
            150000,
            custom_values,
        ).unwrap();
        
        let template1_policies = ProductTemplateContract::get_policies_by_template(
            env.clone(),
            template1,
            0,
            10,
        ).unwrap();
        
        assert_eq!(template1_policies.len(), 2);
        
        let template2_policies = ProductTemplateContract::get_policies_by_template(
            env.clone(),
            template2,
            0,
            10,
        ).unwrap();
        
        assert_eq!(template2_policies.len(), 1);
    }
    
    #[test]
    fn test_premium_calculation_models() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let holder = Address::generate(&env);
        let custom_values = Vec::new(&env);
        
        // Test Fixed premium model
        let fixed_template = ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(&env, "Fixed Premium"),
            Symbol::new(&env, "Fixed premium template"),
            ProductCategory::Property,
            RiskLevel::Medium,
            PremiumModel::Fixed,
            CoverageType::Full,
            1000000,
            1000000000,
            30,
            365,
            1000000, // 1 unit fixed premium
            50000,
            1000000,
            1500,
            Vec::new(&env),
        ).unwrap();
        
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), fixed_template).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), fixed_template, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), fixed_template, TemplateStatus::Active).unwrap();
        
        let fixed_policy = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            fixed_template,
            100000000, // 100 units coverage
            365,       // 1 year
            1000000,
            custom_values.clone(),
        ).unwrap();
        
        let fixed_policy_data = ProductTemplateContract::get_template_policy(env.clone(), fixed_policy).unwrap();
        assert_eq!(fixed_policy_data.premium_amount, 10000000000); // 10000 units (1000000 * 10000)
        
        // Test Percentage premium model
        let percentage_template = ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(&env, "Percentage Premium"),
            Symbol::new(&env, "Percentage premium template"),
            ProductCategory::Property,
            RiskLevel::Medium,
            PremiumModel::Percentage,
            CoverageType::Full,
            1000000,
            1000000000,
            30,
            365,
            200, // 2% of coverage
            50000,
            1000000,
            1500,
            Vec::new(&env),
        ).unwrap();
        
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), percentage_template).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), percentage_template, TemplateStatus::Approved).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), percentage_template, TemplateStatus::Active).unwrap();
        
        let percentage_policy = ProductTemplateContract::create_policy_from_template(
            env.clone(),
            holder.clone(),
            percentage_template,
            100000000, // 100 units coverage
            180,       // 180 days (half year)
            1000000,
            custom_values.clone(),
        ).unwrap();
        
        let percentage_policy_data = ProductTemplateContract::get_template_policy(env.clone(), percentage_policy).unwrap();
        // 2% of 100 units = 2 units, for 180 days = 2 * (180/365) = ~0.986 units = ~986000000 stroops
        assert!(percentage_policy_data.premium_amount > 980000000);
        assert!(percentage_policy_data.premium_amount < 990000000);
    }
    
    // ============================================================
    // GOVERNANCE INTEGRATION TESTS
    // ============================================================
    
    #[test]
    fn test_propose_template_approval() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let proposer = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Submit template for review first
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template_id).unwrap();
        
        let proposal_id = ProductTemplateContract::propose_template_approval(
            env.clone(),
            proposer.clone(),
            template_id,
            Symbol::new(&env, "Approve Home Insurance Template"),
            Symbol::new(&env, "This template provides standard home insurance coverage"),
            51, // 51% threshold
        ).unwrap();
        
        assert_eq!(proposal_id, template_id + 1000000);
    }
    
    #[test]
    fn test_execute_template_approval() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let executor = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Submit template for review
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template_id).unwrap();
        
        // Create mock proposal ID
        let proposal_id = template_id + 1000000;
        
        let result = ProductTemplateContract::execute_template_approval(
            env.clone(),
            executor.clone(),
            proposal_id,
            template_id,
        );
        
        assert!(result.is_ok());
        
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.status, TemplateStatus::Approved);
    }
    
    #[test]
    fn test_deploy_template() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        let template_id = create_test_template(&env, &creator);
        
        // Make template approved first
        ProductTemplateContract::submit_template_for_review(env.clone(), creator.clone(), template_id).unwrap();
        ProductTemplateContract::change_template_status(env.clone(), admin.clone(), template_id, TemplateStatus::Approved).unwrap();
        
        let result = ProductTemplateContract::deploy_template(
            env.clone(),
            admin.clone(),
            template_id,
        );
        
        assert!(result.is_ok());
        
        let template = ProductTemplateContract::get_template(env.clone(), template_id).unwrap();
        assert_eq!(template.status, TemplateStatus::Active);
    }
    
    // ============================================================
    // VALIDATION RULES TESTS
    // ============================================================
    
    #[test]
    fn test_get_validation_rules() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let rules = ProductTemplateContract::get_validation_rules(env.clone()).unwrap();
        
        assert_eq!(rules.min_collateral_ratio_bps, 1000);
        assert_eq!(rules.max_premium_rate_bps, 5000);
        assert_eq!(rules.min_duration_days, 1);
        assert_eq!(rules.max_duration_days, 365);
        assert_eq!(rules.approval_threshold_bps, 5100);
        assert_eq!(rules.min_update_interval, 3600);
    }
    
    #[test]
    fn test_update_validation_rules() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let new_rules = TemplateValidationRules {
            min_collateral_ratio_bps: 2000,
            max_premium_rate_bps: 4000,
            min_duration_days: 7,
            max_duration_days: 730,
            approval_threshold_bps: 6000,
            min_update_interval: 7200,
        };
        
        let result = ProductTemplateContract::update_validation_rules(
            env.clone(),
            admin.clone(),
            new_rules.clone(),
        );
        
        assert!(result.is_ok());
        
        let updated_rules = ProductTemplateContract::get_validation_rules(env.clone()).unwrap();
        assert_eq!(updated_rules.min_collateral_ratio_bps, new_rules.min_collateral_ratio_bps);
        assert_eq!(updated_rules.max_premium_rate_bps, new_rules.max_premium_rate_bps);
        assert_eq!(updated_rules.min_duration_days, new_rules.min_duration_days);
        assert_eq!(updated_rules.max_duration_days, new_rules.max_duration_days);
        assert_eq!(updated_rules.approval_threshold_bps, new_rules.approval_threshold_bps);
        assert_eq!(updated_rules.min_update_interval, new_rules.min_update_interval);
    }
    
    #[test]
    fn test_update_validation_rules_unauthorized() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let unauthorized = Address::generate(&env);
        let new_rules = TemplateValidationRules {
            min_collateral_ratio_bps: 2000,
            max_premium_rate_bps: 4000,
            min_duration_days: 7,
            max_duration_days: 730,
            approval_threshold_bps: 6000,
            min_update_interval: 7200,
        };
        
        let result = ProductTemplateContract::update_validation_rules(
            env.clone(),
            unauthorized.clone(),
            new_rules,
        );
        
        assert_eq!(result, Err(ContractError::Unauthorized));
    }
    
    #[test]
    fn test_update_validation_rules_invalid_values() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        // Test invalid collateral ratio (> 10000)
        let invalid_rules = TemplateValidationRules {
            min_collateral_ratio_bps: 15000, // Invalid - > 10000
            max_premium_rate_bps: 4000,
            min_duration_days: 7,
            max_duration_days: 730,
            approval_threshold_bps: 6000,
            min_update_interval: 7200,
        };
        
        let result = ProductTemplateContract::update_validation_rules(
            env.clone(),
            admin.clone(),
            invalid_rules,
        );
        
        assert_eq!(result, Err(ContractError::InvalidInput));
    }
    
    // ============================================================
    // PAUSE/UNPAUSE TESTS
    // ============================================================
    
    #[test]
    fn test_pause_unpause() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        // Test pause
        let pause_result = ProductTemplateContract::pause(env.clone(), admin.clone());
        assert!(pause_result.is_ok());
        assert!(ProductTemplateContract::is_contract_paused(env.clone()));
        
        // Test unpause
        let unpause_result = ProductTemplateContract::unpause(env.clone(), admin.clone());
        assert!(unpause_result.is_ok());
        assert!(!ProductTemplateContract::is_contract_paused(env.clone()));
    }
    
    #[test]
    fn test_pause_unauthorized() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let unauthorized = Address::generate(&env);
        
        let result = ProductTemplateContract::pause(env.clone(), unauthorized.clone());
        assert_eq!(result, Err(ContractError::Unauthorized));
    }
    
    #[test]
    fn test_operations_when_paused() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        ProductTemplateContract::pause(env.clone(), admin.clone()).unwrap();
        
        // Try to create template when paused
        let result = ProductTemplateContract::create_template(
            env.clone(),
            creator.clone(),
            Symbol::new(&env, "Paused Template"),
            Symbol::new(&env, "Template created while paused"),
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
            Vec::new(&env),
        );
        
        assert_eq!(result, Err(ContractError::Paused));
    }
    
    // ============================================================
    // ERROR CASE TESTS
    // ============================================================
    
    #[test]
    fn test_get_nonexistent_template() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let result = ProductTemplateContract::get_template(env.clone(), 999999);
        assert_eq!(result, Err(ContractError::NotFound));
    }
    
    #[test]
    fn test_update_nonexistent_template() {
        let (env, admin, governance) = setup_test_env();
        initialize_contract(&env, &admin, &governance);
        
        let creator = Address::generate(&env);
        
        // Advance time
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 3601,
            protocol_version: 20,
            sequence_number: env.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 100000,
        });
        
        let result = ProductTemplateContract::update_template(
            env.clone(),
            creator.clone(),
            999999, // Nonexistent ID
            Some(Symbol::new(&env, "Nonexistent Update")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        
        assert_eq!(result, Err(ContractError::NotFound));
    }
}