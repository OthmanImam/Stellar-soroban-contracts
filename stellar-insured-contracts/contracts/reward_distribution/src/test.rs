#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token, Address, Env, String,
};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(env, &contract_address),
        token::StellarAssetClient::new(env, &contract_address),
    )
}

fn setup_test_env() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    
    (env, admin, user1, user2)
}

#[test]
fn test_full_lifecycle() {
    let (env, admin, user1, _user2) = setup_test_env();
    
    // Initialize contract
    RewardDistribution::initialize(env.clone(), admin.clone()).unwrap();
    
    // Create pool
    let pool_id = RewardDistribution::create_pool(
        env.clone(),
        admin.clone(),
        String::from_str(&env, "Test Pool"),
        2_000, // 20% APY
        8_000, // Risk factor
        100_0000000,
        86400, // 1 day lock
    ).unwrap();
    
    assert_eq!(pool_id, 1);
    
    // Verify pool
    let pool = RewardDistribution::get_pool(env.clone(), pool_id).unwrap();
    assert_eq!(pool.base_apy, 2_000);
    assert_eq!(pool.status, RewardStatus::Active);
}

#[test]
fn test_stake_and_rewards() {
    let (env, admin, user1, _use