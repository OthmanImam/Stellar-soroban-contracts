use soroban_sdk::{Env, Address};

use crate::Contract;

#[test]
fn test_claim_with_policy_interaction() {
    let env = Env::default();

    let contract_id = env.register_contract(None, Contract);
    let client = crate::ContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    let claim_result = client.create_claim(&user, &200);

    assert!(claim_result);
}