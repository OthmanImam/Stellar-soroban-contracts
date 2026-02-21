use proptest::prelude::*;
use soroban_sdk::{Env, Address};

use crate::Contract;

proptest! {
    #[test]
    fn fuzz_claim_amount(amount in 0u32..1_000_000) {
        let env = Env::default();
        let contract_id = env.register_contract(None, Contract);
        let client = crate::ContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);

        let result = client.create_claim(&user, &amount);

        if amount == 0 {
            prop_assert!(!result);
        } else {
            prop_assert!(result);
        }
    }
}