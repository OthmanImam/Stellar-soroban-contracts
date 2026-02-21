use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::Contract;

#[test]
fn test_create_claim_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);
    let client = crate::ContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    let result = client.create_claim(&user, &100);

    assert_eq!(result, true);
}

#[test]
fn test_create_claim_invalid_amount() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);
    let client = crate::ContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    let result = client.create_claim(&user, &0);

    assert_eq!(result, false);
}